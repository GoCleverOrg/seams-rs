//! Integration test exercising the facade re-exports.

use std::path::PathBuf;

use seams_rs::{contract_tests as ct, AsyncFileSystem, Clock, FileSystem};
use seams_rs_fake::{ManualClock, MemoryFileSystem};

#[test]
fn facade_reexports_core() {
    let clock = ManualClock::new();
    ct::clock_now_ns_monotonic(&clock);
    // Prove the re-exported trait is identical to the core one.
    let _: &dyn Clock = &clock;
}

#[test]
fn facade_reexports_fs_traits() {
    let fs = MemoryFileSystem::new();
    let base = PathBuf::from("/facade-base");
    FileSystem::create_dir_all(&fs, &base).unwrap();
    ct::fs_create_dir_all_idempotent(&fs, &base);
    // Prove both trait objects work via the facade re-exports.
    let _: &dyn FileSystem = &fs;
    let _: &dyn AsyncFileSystem = &fs;
}

#[test]
#[cfg(feature = "std")]
fn facade_reexports_std_backend() {
    let _clock = seams_rs::SystemClock::new();
    let _fs = seams_rs::StdFileSystem::new();
    let _afs = seams_rs::TokioFileSystem::new();
}
