//! Runs every `seams-rs-core::contract_tests` filesystem helper against
//! `MemoryFileSystem`, plus scripted-error-injection coverage for the
//! four documented `ErrorKind` variants.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use seams_rs_core::contract_tests as ct;
use seams_rs_core::{AsyncFileSystem, FileSystem};
use seams_rs_fake::{FsOp, MemoryFileSystem};

fn fresh(base: &Path) -> MemoryFileSystem {
    let fs = MemoryFileSystem::new();
    FileSystem::create_dir_all(&fs, base).expect("mkdir base");
    fs
}

fn base(n: &str) -> PathBuf {
    PathBuf::from(format!("/base-{n}"))
}

#[test]
fn sync_create_dir_all_missing_parents() {
    let b = base("cdm");
    ct::fs_create_dir_all_missing_parents(&fresh(&b), &b);
}
#[test]
fn sync_create_dir_all_idempotent() {
    let b = base("cdi");
    ct::fs_create_dir_all_idempotent(&fresh(&b), &b);
}
#[test]
fn sync_remove_dir_all_missing_nf() {
    let b = base("rdm");
    ct::fs_remove_dir_all_missing_is_not_found(&fresh(&b), &b);
}
#[test]
fn sync_remove_dir_all_nonempty() {
    let b = base("rdn");
    ct::fs_remove_dir_all_nonempty(&fresh(&b), &b);
}
#[test]
fn sync_try_exists_true() {
    let b = base("tet");
    ct::fs_try_exists_true(&fresh(&b), &b);
}
#[test]
fn sync_try_exists_false() {
    let b = base("tef");
    ct::fs_try_exists_false(&fresh(&b), &b);
}
#[test]
fn sync_open_read_existing() {
    let b = base("ore");
    ct::fs_open_read_existing_yields_bytes(&fresh(&b), &b);
}
#[test]
fn sync_open_read_missing_nf() {
    let b = base("orm");
    ct::fs_open_read_missing_is_not_found(&fresh(&b), &b);
}
#[test]
fn sync_open_write_missing_creates() {
    let b = base("owc");
    ct::fs_open_write_missing_creates(&fresh(&b), &b);
}
#[test]
fn sync_open_write_existing_truncates() {
    let b = base("owt");
    ct::fs_open_write_existing_truncates(&fresh(&b), &b);
}
#[test]
fn sync_metadata_existing() {
    let b = base("mde");
    ct::fs_metadata_existing(&fresh(&b), &b);
}
#[test]
fn sync_metadata_missing_nf() {
    let b = base("mdm");
    ct::fs_metadata_missing_is_not_found(&fresh(&b), &b);
}
#[test]
fn sync_rename_existing() {
    let b = base("rne");
    ct::fs_rename_existing(&fresh(&b), &b);
}
#[test]
fn sync_rename_missing_nf() {
    let b = base("rnm");
    ct::fs_rename_missing_source_is_not_found(&fresh(&b), &b);
}
#[test]
fn sync_file_read_exact() {
    let b = base("fre");
    ct::fs_file_read_exact(&fresh(&b), &b);
}
#[test]
fn sync_file_read_seek() {
    let b = base("frs");
    ct::fs_file_read_seek(&fresh(&b), &b);
}
#[test]
fn sync_file_write_flush() {
    let b = base("fwf");
    ct::fs_file_write_flush_observable(&fresh(&b), &b);
}
#[test]
fn sync_file_write_seek() {
    let b = base("fws");
    ct::fs_file_write_seek(&fresh(&b), &b);
}

// Async helpers run under a current-thread tokio runtime.

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

#[test]
fn async_create_dir_all_missing_parents() {
    let b = base("a-cdm");
    rt().block_on(ct::async_fs_create_dir_all_missing_parents(&fresh(&b), &b));
}
#[test]
fn async_create_dir_all_idempotent() {
    let b = base("a-cdi");
    rt().block_on(ct::async_fs_create_dir_all_idempotent(&fresh(&b), &b));
}
#[test]
fn async_remove_dir_all_missing_nf() {
    let b = base("a-rdm");
    rt().block_on(ct::async_fs_remove_dir_all_missing_is_not_found(
        &fresh(&b),
        &b,
    ));
}
#[test]
fn async_remove_dir_all_nonempty() {
    let b = base("a-rdn");
    rt().block_on(ct::async_fs_remove_dir_all_nonempty(&fresh(&b), &b));
}
#[test]
fn async_try_exists_true() {
    let b = base("a-tet");
    rt().block_on(ct::async_fs_try_exists_true(&fresh(&b), &b));
}
#[test]
fn async_try_exists_false() {
    let b = base("a-tef");
    rt().block_on(ct::async_fs_try_exists_false(&fresh(&b), &b));
}
#[test]
fn async_open_read_existing() {
    let b = base("a-ore");
    rt().block_on(ct::async_fs_open_read_existing_yields_bytes(&fresh(&b), &b));
}
#[test]
fn async_open_read_missing_nf() {
    let b = base("a-orm");
    rt().block_on(ct::async_fs_open_read_missing_is_not_found(&fresh(&b), &b));
}
#[test]
fn async_open_write_missing_creates() {
    let b = base("a-owc");
    rt().block_on(ct::async_fs_open_write_missing_creates(&fresh(&b), &b));
}
#[test]
fn async_open_write_existing_truncates() {
    let b = base("a-owt");
    rt().block_on(ct::async_fs_open_write_existing_truncates(&fresh(&b), &b));
}
#[test]
fn async_metadata_existing() {
    let b = base("a-mde");
    rt().block_on(ct::async_fs_metadata_existing(&fresh(&b), &b));
}
#[test]
fn async_metadata_missing_nf() {
    let b = base("a-mdm");
    rt().block_on(ct::async_fs_metadata_missing_is_not_found(&fresh(&b), &b));
}
#[test]
fn async_rename_existing() {
    let b = base("a-rne");
    rt().block_on(ct::async_fs_rename_existing(&fresh(&b), &b));
}
#[test]
fn async_rename_missing_nf() {
    let b = base("a-rnm");
    rt().block_on(ct::async_fs_rename_missing_source_is_not_found(
        &fresh(&b),
        &b,
    ));
}
#[test]
fn async_file_read_exact() {
    let b = base("a-fre");
    rt().block_on(ct::async_fs_file_read_exact(&fresh(&b), &b));
}
#[test]
fn async_file_read_seek() {
    let b = base("a-frs");
    rt().block_on(ct::async_fs_file_read_seek(&fresh(&b), &b));
}
#[test]
fn async_file_write_flush() {
    let b = base("a-fwf");
    rt().block_on(ct::async_fs_file_write_flush_observable(&fresh(&b), &b));
}
#[test]
fn async_file_write_seek() {
    let b = base("a-fws");
    rt().block_on(ct::async_fs_file_write_seek(&fresh(&b), &b));
}

#[test]
fn sync_async_interop_same_state() {
    let b = base("interop");
    let sync_fs = fresh(&b);
    let async_fs = sync_fs.clone();
    rt().block_on(ct::fs_sync_async_interop(&sync_fs, &async_fs, &b));
}

// ---------------- scripted error injection ----------------

fn expect_err_kind<T>(res: std::io::Result<T>, want: ErrorKind) {
    match res {
        Ok(_) => panic!("expected {want:?}, got Ok"),
        Err(e) => assert_eq!(e.kind(), want, "got {e:?}"),
    }
}

#[test]
fn inject_not_found_on_exists() {
    let b = base("inj-nf");
    let fs = fresh(&b);
    let p = b.join("x");
    FileSystem::create_dir_all(&fs, &p).unwrap();
    fs.inject_error(&p, FsOp::Exists, ErrorKind::NotFound);
    expect_err_kind(FileSystem::try_exists(&fs, &p), ErrorKind::NotFound);
    // Injection is consumed — next call succeeds.
    assert!(FileSystem::try_exists(&fs, &p).unwrap());
}

#[test]
fn inject_already_exists_on_create_dir() {
    let b = base("inj-ae");
    let fs = fresh(&b);
    let p = b.join("y");
    fs.inject_error(&p, FsOp::CreateDir, ErrorKind::AlreadyExists);
    expect_err_kind(
        FileSystem::create_dir_all(&fs, &p),
        ErrorKind::AlreadyExists,
    );
}

#[test]
fn inject_permission_denied_on_open_read() {
    let b = base("inj-pd");
    let fs = fresh(&b);
    let p = b.join("z");
    let mut w = FileSystem::open_write(&fs, &p).unwrap();
    w.write_all(b"data").unwrap();
    w.flush().unwrap();
    drop(w);
    fs.inject_error(&p, FsOp::OpenRead, ErrorKind::PermissionDenied);
    expect_err_kind(FileSystem::open_read(&fs, &p), ErrorKind::PermissionDenied);
}

#[test]
fn inject_storage_full_on_write() {
    let b = base("inj-sf");
    let fs = fresh(&b);
    let p = b.join("q");
    let mut w = FileSystem::open_write(&fs, &p).unwrap();
    fs.inject_error(&p, FsOp::Write, ErrorKind::StorageFull);
    expect_err_kind(w.write_all(b"data"), ErrorKind::StorageFull);
}

#[test]
fn inject_async_not_found_on_metadata() {
    let b = base("inj-a-nf");
    let fs = fresh(&b);
    let p = b.join("m");
    FileSystem::create_dir_all(&fs, &p).unwrap();
    fs.inject_error(&p, FsOp::Metadata, ErrorKind::NotFound);
    rt().block_on(async {
        expect_err_kind(
            AsyncFileSystem::metadata(&fs, &p).await,
            ErrorKind::NotFound,
        );
    });
}
