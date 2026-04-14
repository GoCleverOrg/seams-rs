//! `FileSystem` and `AsyncFileSystem` trait families.
//!
//! Both trait families are dyn-compatible: methods take `&Path` rather
//! than `impl AsRef<Path>`, and async returns are boxed via
//! [`BoxFuture`] to preserve object-safety. Callers hold their
//! implementations as `Arc<dyn FileSystem>` / `Arc<dyn AsyncFileSystem>`
//! and convert paths explicitly with `Path::new(...)` or `.as_ref()`.
//!
//! Every method mirrors the corresponding `std::fs` / `tokio::fs`
//! primitive and returns `std::io::Result<T>` so migrations from
//! hard-wired stdlib calls are drop-in.

use std::future::Future;
use std::io;
use std::path::Path;
use std::pin::Pin;
use std::time::SystemTime;

/// Owned, pinned, `Send` future returned by [`AsyncFileSystem`] and the
/// per-handle async traits. Defined in `seams-rs-core` so callers do
/// not need a dependency on `futures` or `async-trait`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Filesystem metadata DTO — the cross-platform intersection of
/// `std::fs::Metadata` and `tokio::fs::Metadata`.
///
/// Owned, `Clone`-able, and cheap to copy. Permissions, ownership, and
/// symlink-metadata are intentionally excluded; implementations that
/// cannot supply modified-time return `ErrorKind::Unsupported` from
/// [`Metadata::modified`].
#[derive(Debug, Clone)]
pub struct Metadata {
    len: u64,
    is_file: bool,
    is_dir: bool,
    modified: Option<SystemTime>,
}

impl Metadata {
    /// Construct a new `Metadata` DTO.
    pub fn new(len: u64, is_file: bool, is_dir: bool, modified: Option<SystemTime>) -> Self {
        Self {
            len,
            is_file,
            is_dir,
            modified,
        }
    }

    /// Size in bytes. For directories, implementation-defined (fakes return 0).
    pub fn len(&self) -> u64 {
        self.len
    }

    /// True if `len` is zero.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// True if the entry is a regular file.
    pub fn is_file(&self) -> bool {
        self.is_file
    }

    /// True if the entry is a directory.
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }

    /// Last-modified time, or `ErrorKind::Unsupported` if unavailable.
    pub fn modified(&self) -> io::Result<SystemTime> {
        self.modified
            .ok_or_else(|| io::Error::new(io::ErrorKind::Unsupported, "modified time unavailable"))
    }
}

/// Synchronous filesystem port. Implementations must be
/// `Send + Sync + 'static` so they can be held as `Arc<dyn FileSystem>`
/// and shared across threads.
pub trait FileSystem: Send + Sync + 'static {
    /// Create a directory and all missing parents. Idempotent: succeeds
    /// if the directory already exists.
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;

    /// Remove a directory and all of its contents recursively.
    /// `ErrorKind::NotFound` if the path does not exist.
    fn remove_dir_all(&self, path: &Path) -> io::Result<()>;

    /// Returns `Ok(true)` if the path exists, `Ok(false)` if it does
    /// not, and a non-`NotFound` error for any other condition
    /// (for example permission denied).
    fn try_exists(&self, path: &Path) -> io::Result<bool>;

    /// Open an existing file for reading. `ErrorKind::NotFound` if the
    /// file does not exist.
    fn open_read(&self, path: &Path) -> io::Result<Box<dyn FileRead>>;

    /// Open a file for writing, creating it if absent and truncating
    /// it if present. Missing parent directories produce
    /// `ErrorKind::NotFound`.
    fn open_write(&self, path: &Path) -> io::Result<Box<dyn FileWrite>>;

    /// Return metadata for the path. `ErrorKind::NotFound` if the path
    /// does not exist.
    fn metadata(&self, path: &Path) -> io::Result<Metadata>;

    /// Rename (move) a file or directory. `ErrorKind::NotFound` if the
    /// source does not exist.
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
}

/// Per-file-handle synchronous read surface.
pub trait FileRead: Send {
    /// Read all remaining bytes into `buf`, returning the number read.
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize>;

    /// Fill `buf` exactly. `ErrorKind::UnexpectedEof` if EOF is reached
    /// before `buf` is full.
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()>;

    /// Reposition the read cursor, returning the new absolute offset.
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64>;
}

/// Per-file-handle synchronous write surface.
pub trait FileWrite: Send {
    /// Write all bytes of `buf`, retrying on partial writes.
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()>;

    /// Flush buffered writes to the underlying storage.
    fn flush(&mut self) -> io::Result<()>;

    /// Reposition the write cursor, returning the new absolute offset.
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64>;
}

/// Asynchronous filesystem port, shaped to match `tokio::fs`.
///
/// Methods return boxed, `Send` futures so the trait is dyn-compatible
/// and can be held as `Arc<dyn AsyncFileSystem>`.
pub trait AsyncFileSystem: Send + Sync + 'static {
    /// Async variant of [`FileSystem::create_dir_all`].
    fn create_dir_all<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, io::Result<()>>;

    /// Async variant of [`FileSystem::remove_dir_all`].
    fn remove_dir_all<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, io::Result<()>>;

    /// Async variant of [`FileSystem::try_exists`].
    fn try_exists<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, io::Result<bool>>;

    /// Async variant of [`FileSystem::open_read`].
    fn open_read<'a>(&'a self, path: &'a Path)
        -> BoxFuture<'a, io::Result<Box<dyn AsyncFileRead>>>;

    /// Async variant of [`FileSystem::open_write`].
    fn open_write<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxFuture<'a, io::Result<Box<dyn AsyncFileWrite>>>;

    /// Async variant of [`FileSystem::metadata`].
    fn metadata<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, io::Result<Metadata>>;

    /// Async variant of [`FileSystem::rename`].
    fn rename<'a>(&'a self, from: &'a Path, to: &'a Path) -> BoxFuture<'a, io::Result<()>>;
}

/// Per-file-handle asynchronous read surface.
pub trait AsyncFileRead: Send {
    /// Async variant of [`FileRead::read_to_end`].
    fn read_to_end<'a>(&'a mut self, buf: &'a mut Vec<u8>) -> BoxFuture<'a, io::Result<usize>>;

    /// Async variant of [`FileRead::read_exact`].
    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> BoxFuture<'a, io::Result<()>>;

    /// Async variant of [`FileRead::seek`].
    fn seek(&mut self, pos: io::SeekFrom) -> BoxFuture<'_, io::Result<u64>>;
}

/// Per-file-handle asynchronous write surface.
pub trait AsyncFileWrite: Send {
    /// Async variant of [`FileWrite::write_all`].
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> BoxFuture<'a, io::Result<()>>;

    /// Async variant of [`FileWrite::flush`].
    fn flush(&mut self) -> BoxFuture<'_, io::Result<()>>;

    /// Async variant of [`FileWrite::seek`].
    fn seek(&mut self, pos: io::SeekFrom) -> BoxFuture<'_, io::Result<u64>>;
}

#[cfg(test)]
mod metadata_tests {
    use super::*;

    #[test]
    fn accessors_reflect_ctor_args() {
        let now = SystemTime::UNIX_EPOCH;
        let md = Metadata::new(42, true, false, Some(now));
        assert_eq!(md.len(), 42);
        assert!(md.is_file());
        assert!(!md.is_dir());
        assert_eq!(md.modified().unwrap(), now);
    }

    #[test]
    fn len_and_is_empty() {
        let z = Metadata::new(0, true, false, None);
        let n = Metadata::new(7, true, false, None);
        assert_eq!(z.len(), 0);
        assert_eq!(n.len(), 7);
        assert!(z.is_empty());
        assert!(!n.is_empty());
    }

    #[test]
    fn missing_modified_returns_unsupported() {
        let md = Metadata::new(0, false, true, None);
        assert_eq!(
            md.modified().unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }

    #[test]
    fn dir_flags_distinct_from_file() {
        let md = Metadata::new(0, false, true, None);
        assert!(md.is_dir());
        assert!(!md.is_file());
    }
}
