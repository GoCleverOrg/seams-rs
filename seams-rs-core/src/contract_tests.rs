//! Reusable contract tests for every port implementation.
//! Each helper asserts contract invariants and panics via `assert!`
//! on violation.
//!
//! Filesystem helpers take a `base: &Path` — the caller guarantees the
//! directory is empty and unique. For `seams-rs-std` production tests
//! this is a `tempfile::TempDir`; for `seams-rs-fake` tests this can
//! be any path on the in-memory VFS.

use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::{AsyncFileSystem, Clock, FileSystem, JoinError, Sleeper, Spawner};

/// Two successive `now_ns` calls must not go backwards.
pub fn clock_now_ns_monotonic<C: Clock>(clock: &C) {
    let a = clock.now_ns();
    let b = clock.now_ns();
    assert!(b >= a, "now_ns went backwards: {a} -> {b}");
}

/// Two successive `now_instant` calls must not go backwards.
pub fn clock_now_instant_monotonic<C: Clock>(clock: &C) {
    let a = clock.now_instant();
    let b = clock.now_instant();
    assert!(b >= a, "now_instant went backwards");
}

/// Measures wall elapsed around `sleeper.sleep(duration)` and asserts it
/// does not exceed `upper_bound`. Lower-bound checks are left to the caller
/// because fake sleepers legitimately return instantly.
pub fn sleeper_sleep_waits<S: Sleeper>(sleeper: &S, duration: Duration, upper_bound: Duration) {
    let start = Instant::now();
    sleeper.sleep(duration);
    let elapsed = start.elapsed();
    assert!(
        elapsed <= upper_bound,
        "sleep took {elapsed:?}, bound {upper_bound:?}"
    );
}

/// When `shutdown` is already `true`, `sleep_responsive` must return
/// immediately with `true`.
pub fn sleeper_responsive_shutdown_before<S: Sleeper>(sleeper: &S) {
    let shutdown = AtomicBool::new(true);
    let start = Instant::now();
    let triggered = sleeper.sleep_responsive(Duration::from_secs(10), &shutdown);
    let elapsed = start.elapsed();
    assert!(triggered, "expected shutdown observed");
    assert!(
        elapsed <= Duration::from_millis(100),
        "returned too slowly: {elapsed:?}"
    );
}

/// When `shutdown` flips during a long sleep, the sleeper must observe it
/// and return early. For instant-returning fakes, the call completes before
/// the flag is flipped — in that case the helper tolerates a fast return.
pub fn sleeper_responsive_shutdown_during<S: Sleeper>(sleeper: &S) {
    let shutdown = Arc::new(AtomicBool::new(false));
    let flag = shutdown.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        flag.store(true, Ordering::SeqCst);
    });
    let start = Instant::now();
    let triggered = sleeper.sleep_responsive(Duration::from_secs(10), &shutdown);
    let elapsed = start.elapsed();
    // Either shutdown was observed during the call, or the sleeper returned
    // before the flag-flip thread had a chance to store (instant-fake case,
    // detectable because the flag is still `false` post-return).
    let observed_after = shutdown.load(Ordering::SeqCst);
    let ok = triggered || !observed_after;
    assert!(ok, "sleeper waited {elapsed:?} without observing shutdown");
    assert!(elapsed < Duration::from_secs(5));
}

/// Without shutdown, `sleep_responsive` returns `false`.
pub fn sleeper_responsive_no_shutdown<S: Sleeper>(sleeper: &S) {
    let shutdown = AtomicBool::new(false);
    let triggered = sleeper.sleep_responsive(Duration::from_millis(5), &shutdown);
    assert!(!triggered);
}

/// `spawn_blocking` must deliver the closure's return value to the joiner.
pub fn spawner_returns_value<S: Spawner>(spawner: &S) {
    let handle = spawner.spawn_blocking(|| 42i32);
    assert_eq!(handle.join().unwrap(), 42);
}

/// A panic in the spawned closure must surface as `JoinError::Panicked`.
pub fn spawner_propagates_panic<S: Spawner>(spawner: &S) {
    let handle = spawner.spawn_blocking::<_, ()>(|| panic!("boom"));
    match handle.join() {
        Err(JoinError::Panicked(msg)) => {
            assert!(msg.contains("boom"), "got: {msg}")
        }
        other => panic!("expected Panicked, got {other:?}"),
    }
}

fn write_file<F: FileSystem>(fs: &F, path: &Path, bytes: &[u8]) {
    let mut w = fs.open_write(path).expect("open_write");
    w.write_all(bytes).expect("write_all");
    w.flush().expect("flush");
}

fn read_file<F: FileSystem>(fs: &F, path: &Path) -> Vec<u8> {
    let mut r = fs.open_read(path).expect("open_read");
    let mut buf = Vec::new();
    r.read_to_end(&mut buf).expect("read_to_end");
    buf
}

/// `create_dir_all` must create missing parent chain and make path exist.
pub fn fs_create_dir_all_missing_parents<F: FileSystem>(fs: &F, base: &Path) {
    let target = base.join("a/b/c");
    fs.create_dir_all(&target).expect("create_dir_all");
    assert!(fs.try_exists(&target).expect("try_exists"));
    let md = fs.metadata(&target).expect("metadata");
    assert!(md.is_dir() && !md.is_file());
}

/// `create_dir_all` on an existing directory must succeed (idempotent).
pub fn fs_create_dir_all_idempotent<F: FileSystem>(fs: &F, base: &Path) {
    let target = base.join("exists");
    fs.create_dir_all(&target).expect("first");
    fs.create_dir_all(&target).expect("second idempotent");
}

/// `remove_dir_all` on a missing path must return `ErrorKind::NotFound`.
pub fn fs_remove_dir_all_missing_is_not_found<F: FileSystem>(fs: &F, base: &Path) {
    let target = base.join("never-existed");
    let err = fs.remove_dir_all(&target).expect_err("must fail");
    assert_eq!(err.kind(), io::ErrorKind::NotFound, "{err:?}");
}

/// `remove_dir_all` must recursively delete non-empty directories.
pub fn fs_remove_dir_all_nonempty<F: FileSystem>(fs: &F, base: &Path) {
    let dir = base.join("tree");
    fs.create_dir_all(&dir.join("sub")).expect("mkdir");
    write_file(fs, &dir.join("sub/f"), b"x");
    fs.remove_dir_all(&dir).expect("remove");
    assert!(!fs.try_exists(&dir).expect("try_exists"));
}

/// `try_exists` must return `Ok(true)` for a present path.
pub fn fs_try_exists_true<F: FileSystem>(fs: &F, base: &Path) {
    let p = base.join("present");
    write_file(fs, &p, b"hi");
    assert!(fs.try_exists(&p).expect("ok"));
}

/// `try_exists` must return `Ok(false)` for an absent path.
pub fn fs_try_exists_false<F: FileSystem>(fs: &F, base: &Path) {
    let p = base.join("missing");
    assert!(!fs.try_exists(&p).expect("ok"));
}

/// `open_read` of an existing file must yield the written bytes.
pub fn fs_open_read_existing_yields_bytes<F: FileSystem>(fs: &F, base: &Path) {
    let p = base.join("data.bin");
    let payload = b"hello, seams";
    write_file(fs, &p, payload);
    assert_eq!(read_file(fs, &p), payload);
}

/// `open_read` of a missing file must return `ErrorKind::NotFound`.
pub fn fs_open_read_missing_is_not_found<F: FileSystem>(fs: &F, base: &Path) {
    match fs.open_read(&base.join("nope")) {
        Ok(_) => panic!("expected NotFound, got Ok"),
        Err(e) => assert_eq!(e.kind(), io::ErrorKind::NotFound, "{e:?}"),
    }
}

/// `open_write` on a missing path must create the file.
pub fn fs_open_write_missing_creates<F: FileSystem>(fs: &F, base: &Path) {
    let p = base.join("created.bin");
    write_file(fs, &p, b"abc");
    assert_eq!(read_file(fs, &p), b"abc");
}

/// `open_write` on an existing file must truncate it.
pub fn fs_open_write_existing_truncates<F: FileSystem>(fs: &F, base: &Path) {
    let p = base.join("trunc.bin");
    write_file(fs, &p, b"longer-original");
    write_file(fs, &p, b"ab");
    assert_eq!(read_file(fs, &p), b"ab");
}

/// `metadata` of an existing file must report correct len and file-ness.
pub fn fs_metadata_existing<F: FileSystem>(fs: &F, base: &Path) {
    let p = base.join("md.bin");
    write_file(fs, &p, b"12345");
    let md = fs.metadata(&p).expect("metadata");
    assert_eq!(md.len(), 5);
    assert!(md.is_file() && !md.is_dir());
}

/// `metadata` of a missing path must return `ErrorKind::NotFound`.
pub fn fs_metadata_missing_is_not_found<F: FileSystem>(fs: &F, base: &Path) {
    let err = fs.metadata(&base.join("nope")).expect_err("must fail");
    assert_eq!(err.kind(), io::ErrorKind::NotFound, "{err:?}");
}

/// `rename` must move an existing file to a new path.
pub fn fs_rename_existing<F: FileSystem>(fs: &F, base: &Path) {
    let a = base.join("a");
    let b = base.join("b");
    write_file(fs, &a, b"payload");
    fs.rename(&a, &b).expect("rename");
    assert!(!fs.try_exists(&a).expect("a gone"));
    assert_eq!(read_file(fs, &b), b"payload");
}

/// `rename` where source is missing must return `ErrorKind::NotFound`.
pub fn fs_rename_missing_source_is_not_found<F: FileSystem>(fs: &F, base: &Path) {
    let err = fs
        .rename(&base.join("missing"), &base.join("dest"))
        .expect_err("must fail");
    assert_eq!(err.kind(), io::ErrorKind::NotFound, "{err:?}");
}

/// `FileRead::read_exact` must fill the buffer with the written bytes.
pub fn fs_file_read_exact<F: FileSystem>(fs: &F, base: &Path) {
    let p = base.join("rx.bin");
    write_file(fs, &p, b"0123456789");
    let mut r = fs.open_read(&p).expect("open_read");
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf).expect("read_exact");
    assert_eq!(&buf, b"0123");
}

/// `FileRead::seek` must reposition the cursor so subsequent reads
/// return bytes at the new offset.
pub fn fs_file_read_seek<F: FileSystem>(fs: &F, base: &Path) {
    let p = base.join("seek-r.bin");
    write_file(fs, &p, b"ABCDEFGH");
    let mut r = fs.open_read(&p).expect("open_read");
    let pos = r.seek(io::SeekFrom::Start(4)).expect("seek");
    assert_eq!(pos, 4);
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf).expect("read_exact");
    assert_eq!(&buf, b"EF");
}

/// `FileWrite::flush` plus a later `open_read` must observe the bytes.
pub fn fs_file_write_flush_observable<F: FileSystem>(fs: &F, base: &Path) {
    let p = base.join("flush.bin");
    {
        let mut w = fs.open_write(&p).expect("open_write");
        w.write_all(b"durable").expect("write_all");
        w.flush().expect("flush");
    }
    assert_eq!(read_file(fs, &p), b"durable");
}

/// `FileWrite::seek` must reposition the write cursor.
pub fn fs_file_write_seek<F: FileSystem>(fs: &F, base: &Path) {
    let p = base.join("seek-w.bin");
    {
        let mut w = fs.open_write(&p).expect("open_write");
        w.write_all(b"HELLO_WORLD").expect("write");
        let pos = w.seek(io::SeekFrom::Start(6)).expect("seek");
        assert_eq!(pos, 6);
        w.write_all(b"seams").expect("overwrite");
        w.flush().expect("flush");
    }
    assert_eq!(read_file(fs, &p), b"HELLO_seams");
}

async fn async_write_file<F: AsyncFileSystem>(fs: &F, path: &Path, bytes: &[u8]) {
    let mut w = fs.open_write(path).await.expect("open_write");
    w.write_all(bytes).await.expect("write_all");
    w.flush().await.expect("flush");
}

async fn async_read_file<F: AsyncFileSystem>(fs: &F, path: &Path) -> Vec<u8> {
    let mut r = fs.open_read(path).await.expect("open_read");
    let mut buf = Vec::new();
    r.read_to_end(&mut buf).await.expect("read_to_end");
    buf
}

/// Async `create_dir_all` must create missing parent chain.
pub async fn async_fs_create_dir_all_missing_parents<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let target = base.join("a/b/c");
    fs.create_dir_all(&target).await.expect("create_dir_all");
    assert!(fs.try_exists(&target).await.expect("try_exists"));
}

/// Async `create_dir_all` must be idempotent.
pub async fn async_fs_create_dir_all_idempotent<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let target = base.join("ex-async");
    fs.create_dir_all(&target).await.expect("first");
    fs.create_dir_all(&target).await.expect("second");
}

/// Async `remove_dir_all` on missing path must return `NotFound`.
pub async fn async_fs_remove_dir_all_missing_is_not_found<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let err = fs
        .remove_dir_all(&base.join("never"))
        .await
        .expect_err("must fail");
    assert_eq!(err.kind(), io::ErrorKind::NotFound, "{err:?}");
}

/// Async `remove_dir_all` must delete non-empty trees.
pub async fn async_fs_remove_dir_all_nonempty<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let dir = base.join("tree-async");
    fs.create_dir_all(&dir.join("sub")).await.expect("mkdir");
    async_write_file(fs, &dir.join("sub/f"), b"x").await;
    fs.remove_dir_all(&dir).await.expect("remove");
    assert!(!fs.try_exists(&dir).await.expect("gone"));
}

/// Async `try_exists` must return `true` for a present path.
pub async fn async_fs_try_exists_true<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let p = base.join("present-async");
    async_write_file(fs, &p, b"hi").await;
    assert!(fs.try_exists(&p).await.expect("ok"));
}

/// Async `try_exists` must return `false` for an absent path.
pub async fn async_fs_try_exists_false<F: AsyncFileSystem>(fs: &F, base: &Path) {
    assert!(!fs
        .try_exists(&base.join("missing-async"))
        .await
        .expect("ok"));
}

/// Async `open_read` must yield the written bytes.
pub async fn async_fs_open_read_existing_yields_bytes<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let p = base.join("data-async.bin");
    async_write_file(fs, &p, b"hello-async").await;
    assert_eq!(async_read_file(fs, &p).await, b"hello-async");
}

/// Async `open_read` of a missing file must return `NotFound`.
pub async fn async_fs_open_read_missing_is_not_found<F: AsyncFileSystem>(fs: &F, base: &Path) {
    match fs.open_read(&base.join("nope-async")).await {
        Ok(_) => panic!("expected NotFound, got Ok"),
        Err(e) => assert_eq!(e.kind(), io::ErrorKind::NotFound, "{e:?}"),
    }
}

/// Async `open_write` on a missing path must create the file.
pub async fn async_fs_open_write_missing_creates<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let p = base.join("created-async.bin");
    async_write_file(fs, &p, b"abc").await;
    assert_eq!(async_read_file(fs, &p).await, b"abc");
}

/// Async `open_write` on an existing file must truncate it.
pub async fn async_fs_open_write_existing_truncates<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let p = base.join("trunc-async.bin");
    async_write_file(fs, &p, b"longer-original").await;
    async_write_file(fs, &p, b"ab").await;
    assert_eq!(async_read_file(fs, &p).await, b"ab");
}

/// Async `metadata` of existing file must report correct len.
pub async fn async_fs_metadata_existing<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let p = base.join("md-async.bin");
    async_write_file(fs, &p, b"12345").await;
    let md = fs.metadata(&p).await.expect("metadata");
    assert_eq!(md.len(), 5);
    assert!(md.is_file());
}

/// Async `metadata` of a missing path must return `NotFound`.
pub async fn async_fs_metadata_missing_is_not_found<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let err = fs
        .metadata(&base.join("nope-async"))
        .await
        .expect_err("must fail");
    assert_eq!(err.kind(), io::ErrorKind::NotFound, "{err:?}");
}

/// Async `rename` must move an existing file.
pub async fn async_fs_rename_existing<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let a = base.join("a-async");
    let b = base.join("b-async");
    async_write_file(fs, &a, b"payload").await;
    fs.rename(&a, &b).await.expect("rename");
    assert!(!fs.try_exists(&a).await.expect("gone"));
    assert_eq!(async_read_file(fs, &b).await, b"payload");
}

/// Async `rename` with missing source must return `NotFound`.
pub async fn async_fs_rename_missing_source_is_not_found<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let err = fs
        .rename(&base.join("m-a"), &base.join("m-b"))
        .await
        .expect_err("must fail");
    assert_eq!(err.kind(), io::ErrorKind::NotFound, "{err:?}");
}

/// Async `AsyncFileRead::read_exact` must fill the buffer.
pub async fn async_fs_file_read_exact<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let p = base.join("rx-async.bin");
    async_write_file(fs, &p, b"0123456789").await;
    let mut r = fs.open_read(&p).await.expect("open_read");
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf).await.expect("read_exact");
    assert_eq!(&buf, b"0123");
}

/// Async `AsyncFileRead::seek` must reposition the cursor.
pub async fn async_fs_file_read_seek<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let p = base.join("seek-r-async.bin");
    async_write_file(fs, &p, b"ABCDEFGH").await;
    let mut r = fs.open_read(&p).await.expect("open_read");
    let pos = r.seek(io::SeekFrom::Start(4)).await.expect("seek");
    assert_eq!(pos, 4);
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf).await.expect("read_exact");
    assert_eq!(&buf, b"EF");
}

/// Async `AsyncFileWrite::flush` followed by `open_read` must observe bytes.
pub async fn async_fs_file_write_flush_observable<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let p = base.join("flush-async.bin");
    {
        let mut w = fs.open_write(&p).await.expect("open_write");
        w.write_all(b"durable-async").await.expect("write");
        w.flush().await.expect("flush");
    }
    assert_eq!(async_read_file(fs, &p).await, b"durable-async");
}

/// Async `AsyncFileWrite::seek` must reposition the write cursor.
pub async fn async_fs_file_write_seek<F: AsyncFileSystem>(fs: &F, base: &Path) {
    let p = base.join("seek-w-async.bin");
    {
        let mut w = fs.open_write(&p).await.expect("open_write");
        w.write_all(b"HELLO_WORLD").await.expect("write");
        let pos = w.seek(io::SeekFrom::Start(6)).await.expect("seek");
        assert_eq!(pos, 6);
        w.write_all(b"seams").await.expect("overwrite");
        w.flush().await.expect("flush");
    }
    assert_eq!(async_read_file(fs, &p).await, b"HELLO_seams");
}

/// Sync-async duality: a file written via `AsyncFileSystem::open_write`
/// must be readable via `FileSystem::open_read` on the same underlying
/// state. Only meaningful when the two trait objects share backing
/// storage (e.g., the `MemoryFileSystem` fake, or two adapters sharing
/// a real filesystem root).
pub async fn fs_sync_async_interop<S: FileSystem, A: AsyncFileSystem>(
    sync_fs: &S,
    async_fs: &A,
    base: &Path,
) {
    let p = base.join("duality.bin");
    async_write_file(async_fs, &p, b"from-async").await;
    assert_eq!(read_file(sync_fs, &p), b"from-async");

    let p2 = base.join("duality2.bin");
    write_file(sync_fs, &p2, b"from-sync");
    assert_eq!(async_read_file(async_fs, &p2).await, b"from-sync");
}
