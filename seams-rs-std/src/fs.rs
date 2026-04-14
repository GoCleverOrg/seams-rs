//! `StdFileSystem` and `TokioFileSystem`: thin wrappers around
//! `std::fs` and `tokio::fs` implementing the `seams-rs-core` port
//! surface. Every method delegates directly with no logic of its own.

use std::io::{self, Read, Seek, Write};
use std::path::Path;

use seams_rs_core::{
    AsyncFileRead, AsyncFileSystem, AsyncFileWrite, BoxFuture, FileRead, FileSystem, FileWrite,
    Metadata,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

// ---------------- std-backed sync ----------------

/// Production `FileSystem` backed by `std::fs`.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdFileSystem;

impl StdFileSystem {
    /// Construct a new `StdFileSystem`.
    pub fn new() -> Self {
        Self
    }
}

fn metadata_from_std(m: std::fs::Metadata) -> Metadata {
    Metadata::new(m.len(), m.is_file(), m.is_dir(), m.modified().ok())
}

impl FileSystem for StdFileSystem {
    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }

    fn remove_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_dir_all(path)
    }

    fn try_exists(&self, path: &Path) -> io::Result<bool> {
        std::fs::exists(path)
    }

    fn open_read(&self, path: &Path) -> io::Result<Box<dyn FileRead>> {
        Ok(Box::new(StdFileRead(std::fs::File::open(path)?)))
    }

    fn open_write(&self, path: &Path) -> io::Result<Box<dyn FileWrite>> {
        Ok(Box::new(StdFileWrite(std::fs::File::create(path)?)))
    }

    fn metadata(&self, path: &Path) -> io::Result<Metadata> {
        Ok(metadata_from_std(std::fs::metadata(path)?))
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(from, to)
    }
}

struct StdFileRead(std::fs::File);

impl FileRead for StdFileRead {
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        Read::read_to_end(&mut self.0, buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        Read::read_exact(&mut self.0, buf)
    }
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        Seek::seek(&mut self.0, pos)
    }
}

struct StdFileWrite(std::fs::File);

impl FileWrite for StdFileWrite {
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        Write::write_all(&mut self.0, buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        Write::flush(&mut self.0)
    }
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        Seek::seek(&mut self.0, pos)
    }
}

// ---------------- tokio-backed async ----------------

/// Production `AsyncFileSystem` backed by `tokio::fs`.
#[derive(Debug, Default, Clone, Copy)]
pub struct TokioFileSystem;

impl TokioFileSystem {
    /// Construct a new `TokioFileSystem`.
    pub fn new() -> Self {
        Self
    }
}

impl AsyncFileSystem for TokioFileSystem {
    fn create_dir_all<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(tokio::fs::create_dir_all(path))
    }

    fn remove_dir_all<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(tokio::fs::remove_dir_all(path))
    }

    fn try_exists<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, io::Result<bool>> {
        Box::pin(tokio::fs::try_exists(path))
    }

    fn open_read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxFuture<'a, io::Result<Box<dyn AsyncFileRead>>> {
        Box::pin(async move {
            let f = tokio::fs::File::open(path).await?;
            Ok(Box::new(TokioFileRead(f)) as Box<dyn AsyncFileRead>)
        })
    }

    fn open_write<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxFuture<'a, io::Result<Box<dyn AsyncFileWrite>>> {
        Box::pin(async move {
            let f = tokio::fs::File::create(path).await?;
            Ok(Box::new(TokioFileWrite(f)) as Box<dyn AsyncFileWrite>)
        })
    }

    fn metadata<'a>(&'a self, path: &'a Path) -> BoxFuture<'a, io::Result<Metadata>> {
        Box::pin(async move {
            let m = tokio::fs::metadata(path).await?;
            Ok(Metadata::new(
                m.len(),
                m.is_file(),
                m.is_dir(),
                m.modified().ok(),
            ))
        })
    }

    fn rename<'a>(&'a self, from: &'a Path, to: &'a Path) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(tokio::fs::rename(from, to))
    }
}

struct TokioFileRead(tokio::fs::File);

impl AsyncFileRead for TokioFileRead {
    fn read_to_end<'a>(&'a mut self, buf: &'a mut Vec<u8>) -> BoxFuture<'a, io::Result<usize>> {
        Box::pin(self.0.read_to_end(buf))
    }
    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(async move {
            self.0.read_exact(buf).await?;
            Ok(())
        })
    }
    fn seek(&mut self, pos: io::SeekFrom) -> BoxFuture<'_, io::Result<u64>> {
        Box::pin(self.0.seek(pos))
    }
}

struct TokioFileWrite(tokio::fs::File);

impl AsyncFileWrite for TokioFileWrite {
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> BoxFuture<'a, io::Result<()>> {
        Box::pin(self.0.write_all(buf))
    }
    fn flush(&mut self) -> BoxFuture<'_, io::Result<()>> {
        Box::pin(self.0.flush())
    }
    fn seek(&mut self, pos: io::SeekFrom) -> BoxFuture<'_, io::Result<u64>> {
        Box::pin(self.0.seek(pos))
    }
}
