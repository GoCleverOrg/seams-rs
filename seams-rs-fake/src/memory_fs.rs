//! `MemoryFileSystem`: in-memory VFS implementing both the sync
//! `FileSystem` and async `AsyncFileSystem` ports over the same state.
//!
//! Scripted error injection via [`MemoryFileSystem::inject_error`]
//! causes the next matching operation on a path to return the supplied
//! `io::ErrorKind`. Useful for exercising error paths deterministically.

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use seams_rs_core::{
    AsyncFileRead, AsyncFileSystem, AsyncFileWrite, BoxFuture, FileRead, FileSystem, FileWrite,
    Metadata,
};

/// Operation tags used to target scripted error injection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FsOp {
    /// `create_dir_all`
    CreateDir,
    /// `remove_dir_all`
    RemoveDir,
    /// `try_exists`
    Exists,
    /// `open_read`
    OpenRead,
    /// `open_write`
    OpenWrite,
    /// `metadata`
    Metadata,
    /// `rename` (matches on the `from` path)
    Rename,
    /// `FileRead`/`AsyncFileRead` read-family methods
    Read,
    /// `FileWrite`/`AsyncFileWrite` `write_all`
    Write,
    /// `FileWrite`/`AsyncFileWrite` `flush`
    Flush,
    /// `FileRead`/`FileWrite` `seek`
    Seek,
}

#[derive(Debug, Clone)]
struct FileNode {
    data: Vec<u8>,
    modified: SystemTime,
}

#[derive(Debug, Default)]
struct VfsState {
    files: HashMap<PathBuf, FileNode>,
    dirs: HashSet<PathBuf>,
    pending: Vec<(PathBuf, FsOp, io::ErrorKind)>,
}

impl VfsState {
    fn take_injection(&mut self, path: &Path, op: FsOp) -> Option<io::ErrorKind> {
        let idx = self
            .pending
            .iter()
            .position(|(p, o, _)| p.as_path() == path && *o == op)?;
        Some(self.pending.remove(idx).2)
    }
}

/// In-memory VFS shared between sync and async trait objects via `Arc`.
/// Clone to hand the same state to multiple adapters.
#[derive(Debug, Clone, Default)]
pub struct MemoryFileSystem {
    state: Arc<Mutex<VfsState>>,
}

impl MemoryFileSystem {
    /// Construct an empty in-memory filesystem.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a scripted error. The next operation on `path` whose
    /// [`FsOp`] tag matches `op` returns `io::Error::from(kind)`,
    /// and the injection is consumed. Multiple injections on the same
    /// `(path, op)` queue in FIFO order.
    pub fn inject_error(&self, path: impl Into<PathBuf>, op: FsOp, kind: io::ErrorKind) {
        self.state
            .lock()
            .expect("poisoned")
            .pending
            .push((path.into(), op, kind));
    }

    fn with<R>(&self, f: impl FnOnce(&mut VfsState) -> R) -> R {
        let mut guard = self.state.lock().expect("poisoned");
        f(&mut guard)
    }
}

fn not_found() -> io::Error {
    io::Error::from(io::ErrorKind::NotFound)
}

// ---------------- core sync ops (shared by both traits) ----------------

fn op_create_dir_all(st: &mut VfsState, path: &Path) -> io::Result<()> {
    if let Some(k) = st.take_injection(path, FsOp::CreateDir) {
        return Err(io::Error::from(k));
    }
    for ancestor in path.ancestors() {
        if ancestor.as_os_str().is_empty() {
            continue;
        }
        if st.files.contains_key(ancestor) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "path exists as file",
            ));
        }
        st.dirs.insert(ancestor.to_path_buf());
    }
    Ok(())
}

fn op_remove_dir_all(st: &mut VfsState, path: &Path) -> io::Result<()> {
    if let Some(k) = st.take_injection(path, FsOp::RemoveDir) {
        return Err(io::Error::from(k));
    }
    if !st.dirs.contains(path) {
        return Err(not_found());
    }
    st.dirs.retain(|d| !d.starts_with(path));
    st.files.retain(|f, _| !f.starts_with(path));
    Ok(())
}

fn op_try_exists(st: &mut VfsState, path: &Path) -> io::Result<bool> {
    if let Some(k) = st.take_injection(path, FsOp::Exists) {
        return Err(io::Error::from(k));
    }
    Ok(st.files.contains_key(path) || st.dirs.contains(path))
}

fn op_metadata(st: &mut VfsState, path: &Path) -> io::Result<Metadata> {
    if let Some(k) = st.take_injection(path, FsOp::Metadata) {
        return Err(io::Error::from(k));
    }
    if let Some(f) = st.files.get(path) {
        return Ok(Metadata::new(
            f.data.len() as u64,
            true,
            false,
            Some(f.modified),
        ));
    }
    if st.dirs.contains(path) {
        return Ok(Metadata::new(0, false, true, Some(SystemTime::now())));
    }
    Err(not_found())
}

fn op_rename(st: &mut VfsState, from: &Path, to: &Path) -> io::Result<()> {
    if let Some(k) = st.take_injection(from, FsOp::Rename) {
        return Err(io::Error::from(k));
    }
    if let Some(node) = st.files.remove(from) {
        st.files.insert(to.to_path_buf(), node);
        return Ok(());
    }
    if st.dirs.contains(from) {
        // Collect and move all entries under `from` to `to`.
        let moved_dirs: Vec<PathBuf> = st
            .dirs
            .iter()
            .filter(|d| d.starts_with(from))
            .cloned()
            .collect();
        for d in &moved_dirs {
            st.dirs.remove(d);
            let rel = d.strip_prefix(from).unwrap();
            st.dirs.insert(to.join(rel));
        }
        let moved_files: Vec<(PathBuf, FileNode)> = st
            .files
            .iter()
            .filter(|(p, _)| p.starts_with(from))
            .map(|(p, n)| (p.clone(), n.clone()))
            .collect();
        for (p, n) in moved_files {
            st.files.remove(&p);
            let rel = p.strip_prefix(from).unwrap();
            st.files.insert(to.join(rel), n);
        }
        return Ok(());
    }
    Err(not_found())
}

fn op_ensure_parent_dir(st: &VfsState, path: &Path) -> io::Result<()> {
    match path.parent() {
        None => Ok(()),
        Some(p) if p.as_os_str().is_empty() => Ok(()),
        Some(p) => {
            if st.dirs.contains(p) {
                Ok(())
            } else {
                Err(not_found())
            }
        }
    }
}

// ---------------- FileSystem impl ----------------

impl FileSystem for MemoryFileSystem {
    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        self.with(|st| op_create_dir_all(st, path))
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        self.with(|st| op_remove_dir_all(st, path))
    }

    fn try_exists(&self, path: &Path) -> io::Result<bool> {
        self.with(|st| op_try_exists(st, path))
    }

    fn open_read(&self, path: &Path) -> io::Result<Box<dyn FileRead>> {
        self.with(|st| {
            if let Some(k) = st.take_injection(path, FsOp::OpenRead) {
                return Err(io::Error::from(k));
            }
            if !st.files.contains_key(path) {
                return Err(not_found());
            }
            Ok(Box::new(MemoryFileRead {
                state: Arc::clone(&self.state),
                path: path.to_path_buf(),
                pos: 0,
            }) as Box<dyn FileRead>)
        })
    }

    fn open_write(&self, path: &Path) -> io::Result<Box<dyn FileWrite>> {
        self.with(|st| {
            if let Some(k) = st.take_injection(path, FsOp::OpenWrite) {
                return Err(io::Error::from(k));
            }
            op_ensure_parent_dir(st, path)?;
            st.files.insert(
                path.to_path_buf(),
                FileNode {
                    data: Vec::new(),
                    modified: SystemTime::now(),
                },
            );
            Ok(Box::new(MemoryFileWrite {
                state: Arc::clone(&self.state),
                path: path.to_path_buf(),
                pos: 0,
            }) as Box<dyn FileWrite>)
        })
    }

    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        self.with(|st| op_metadata(st, path))
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.with(|st| op_rename(st, from, to))
    }
}

// ---------------- AsyncFileSystem impl ----------------

impl AsyncFileSystem for MemoryFileSystem {
    fn create_dir_all<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(async move { self.with(|st| op_create_dir_all(st, path)) })
    }

    fn remove_dir_all<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(async move { self.with(|st| op_remove_dir_all(st, path)) })
    }

    fn try_exists<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, io::Result<bool>> {
        Box::pin(async move { self.with(|st| op_try_exists(st, path)) })
    }

    fn open_read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxFuture<'a, io::Result<Box<dyn AsyncFileRead>>> {
        Box::pin(async move {
            self.with(|st| {
                if let Some(k) = st.take_injection(path, FsOp::OpenRead) {
                    return Err(io::Error::from(k));
                }
                if !st.files.contains_key(path) {
                    return Err(not_found());
                }
                Ok(Box::new(MemoryFileRead {
                    state: Arc::clone(&self.state),
                    path: path.to_path_buf(),
                    pos: 0,
                }) as Box<dyn AsyncFileRead>)
            })
        })
    }

    fn open_write<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxFuture<'a, io::Result<Box<dyn AsyncFileWrite>>> {
        Box::pin(async move {
            self.with(|st| {
                if let Some(k) = st.take_injection(path, FsOp::OpenWrite) {
                    return Err(io::Error::from(k));
                }
                op_ensure_parent_dir(st, path)?;
                st.files.insert(
                    path.to_path_buf(),
                    FileNode {
                        data: Vec::new(),
                        modified: SystemTime::now(),
                    },
                );
                Ok(Box::new(MemoryFileWrite {
                    state: Arc::clone(&self.state),
                    path: path.to_path_buf(),
                    pos: 0,
                }) as Box<dyn AsyncFileWrite>)
            })
        })
    }

    fn metadata<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, io::Result<Metadata>> {
        Box::pin(async move { self.with(|st| op_metadata(st, path)) })
    }

    fn rename<'a>(&'a self, from: &'a Path, to: &'a Path) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(async move { self.with(|st| op_rename(st, from, to)) })
    }
}

// ---------------- file-handle impls ----------------

struct MemoryFileRead {
    state: Arc<Mutex<VfsState>>,
    path: PathBuf,
    pos: u64,
}

fn apply_seek(current: u64, total: u64, pos: io::SeekFrom) -> io::Result<u64> {
    let offset: i128 = match pos {
        io::SeekFrom::Start(n) => n as i128,
        io::SeekFrom::End(n) => total as i128 + n as i128,
        io::SeekFrom::Current(n) => current as i128 + n as i128,
    };
    if offset < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "seek to negative offset",
        ));
    }
    Ok(offset as u64)
}

impl FileRead for MemoryFileRead {
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut st = self.state.lock().expect("poisoned");
        if let Some(k) = st.take_injection(&self.path, FsOp::Read) {
            return Err(io::Error::from(k));
        }
        let node = st.files.get(&self.path).ok_or_else(not_found)?;
        let start = self.pos as usize;
        if start >= node.data.len() {
            return Ok(0);
        }
        let slice = &node.data[start..];
        buf.extend_from_slice(slice);
        let n = slice.len();
        self.pos += n as u64;
        Ok(n)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let mut st = self.state.lock().expect("poisoned");
        if let Some(k) = st.take_injection(&self.path, FsOp::Read) {
            return Err(io::Error::from(k));
        }
        let node = st.files.get(&self.path).ok_or_else(not_found)?;
        let start = self.pos as usize;
        let end = start + buf.len();
        let slice = node
            .data
            .get(start..end)
            .ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))?;
        buf.copy_from_slice(slice);
        self.pos = end as u64;
        Ok(())
    }

    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let mut st = self.state.lock().expect("poisoned");
        if let Some(k) = st.take_injection(&self.path, FsOp::Seek) {
            return Err(io::Error::from(k));
        }
        let total = st
            .files
            .get(&self.path)
            .map(|n| n.data.len() as u64)
            .unwrap_or(0);
        let new_pos = apply_seek(self.pos, total, pos)?;
        self.pos = new_pos;
        Ok(new_pos)
    }
}

impl AsyncFileRead for MemoryFileRead {
    fn read_to_end<'a>(&'a mut self, buf: &'a mut Vec<u8>) -> BoxFuture<'a, io::Result<usize>> {
        Box::pin(async move { <Self as FileRead>::read_to_end(self, buf) })
    }

    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(async move { <Self as FileRead>::read_exact(self, buf) })
    }

    fn seek(&mut self, pos: io::SeekFrom) -> BoxFuture<'_, io::Result<u64>> {
        Box::pin(async move { <Self as FileRead>::seek(self, pos) })
    }
}

struct MemoryFileWrite {
    state: Arc<Mutex<VfsState>>,
    path: PathBuf,
    pos: u64,
}

impl FileWrite for MemoryFileWrite {
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        let mut st = self.state.lock().expect("poisoned");
        if let Some(k) = st.take_injection(&self.path, FsOp::Write) {
            return Err(io::Error::from(k));
        }
        let node = st.files.get_mut(&self.path).ok_or_else(not_found)?;
        let start = self.pos as usize;
        let end = start + buf.len();
        let needed = node.data.len().max(end);
        node.data.resize(needed, 0);
        node.data[start..end].copy_from_slice(buf);
        node.modified = SystemTime::now();
        self.pos = end as u64;
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut st = self.state.lock().expect("poisoned");
        if let Some(k) = st.take_injection(&self.path, FsOp::Flush) {
            return Err(io::Error::from(k));
        }
        Ok(())
    }

    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let mut st = self.state.lock().expect("poisoned");
        if let Some(k) = st.take_injection(&self.path, FsOp::Seek) {
            return Err(io::Error::from(k));
        }
        let total = st
            .files
            .get(&self.path)
            .map(|n| n.data.len() as u64)
            .unwrap_or(0);
        let new_pos = apply_seek(self.pos, total, pos)?;
        self.pos = new_pos;
        Ok(new_pos)
    }
}

impl AsyncFileWrite for MemoryFileWrite {
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(async move { <Self as FileWrite>::write_all(self, buf) })
    }

    fn flush(&mut self) -> BoxFuture<'_, io::Result<()>> {
        Box::pin(async move { <Self as FileWrite>::flush(self) })
    }

    fn seek(&mut self, pos: io::SeekFrom) -> BoxFuture<'_, io::Result<u64>> {
        Box::pin(async move { <Self as FileWrite>::seek(self, pos) })
    }
}
