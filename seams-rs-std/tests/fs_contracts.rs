//! Runs every `seams-rs-core::contract_tests` filesystem helper against
//! `StdFileSystem` and `TokioFileSystem` using real temp directories.

use seams_rs_core::contract_tests as ct;
use seams_rs_std::{StdFileSystem, TokioFileSystem};
use tempfile::TempDir;

fn tmp() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

// ---------------- StdFileSystem ----------------

macro_rules! std_test {
    ($name:ident, $helper:path) => {
        #[test]
        fn $name() {
            let t = tmp();
            $helper(&StdFileSystem::new(), t.path());
        }
    };
}

std_test!(
    std_create_dir_all_missing,
    ct::fs_create_dir_all_missing_parents
);
std_test!(
    std_create_dir_all_idempotent,
    ct::fs_create_dir_all_idempotent
);
std_test!(
    std_remove_dir_all_missing_nf,
    ct::fs_remove_dir_all_missing_is_not_found
);
std_test!(std_remove_dir_all_nonempty, ct::fs_remove_dir_all_nonempty);
std_test!(std_try_exists_true, ct::fs_try_exists_true);
std_test!(std_try_exists_false, ct::fs_try_exists_false);
std_test!(
    std_open_read_existing,
    ct::fs_open_read_existing_yields_bytes
);
std_test!(
    std_open_read_missing_nf,
    ct::fs_open_read_missing_is_not_found
);
std_test!(
    std_open_write_missing_creates,
    ct::fs_open_write_missing_creates
);
std_test!(
    std_open_write_existing_truncates,
    ct::fs_open_write_existing_truncates
);
std_test!(std_metadata_existing, ct::fs_metadata_existing);
std_test!(
    std_metadata_missing_nf,
    ct::fs_metadata_missing_is_not_found
);
std_test!(std_rename_existing, ct::fs_rename_existing);
std_test!(
    std_rename_missing_nf,
    ct::fs_rename_missing_source_is_not_found
);
std_test!(std_file_read_exact, ct::fs_file_read_exact);
std_test!(std_file_read_seek, ct::fs_file_read_seek);
std_test!(std_file_write_flush, ct::fs_file_write_flush_observable);
std_test!(std_file_write_seek, ct::fs_file_write_seek);

// ---------------- TokioFileSystem ----------------

macro_rules! tokio_test {
    ($name:ident, $helper:path) => {
        #[test]
        fn $name() {
            let t = tmp();
            rt().block_on($helper(&TokioFileSystem::new(), t.path()));
        }
    };
}

tokio_test!(
    tk_create_dir_all_missing,
    ct::async_fs_create_dir_all_missing_parents
);
tokio_test!(
    tk_create_dir_all_idempotent,
    ct::async_fs_create_dir_all_idempotent
);
tokio_test!(
    tk_remove_dir_all_missing_nf,
    ct::async_fs_remove_dir_all_missing_is_not_found
);
tokio_test!(
    tk_remove_dir_all_nonempty,
    ct::async_fs_remove_dir_all_nonempty
);
tokio_test!(tk_try_exists_true, ct::async_fs_try_exists_true);
tokio_test!(tk_try_exists_false, ct::async_fs_try_exists_false);
tokio_test!(
    tk_open_read_existing,
    ct::async_fs_open_read_existing_yields_bytes
);
tokio_test!(
    tk_open_read_missing_nf,
    ct::async_fs_open_read_missing_is_not_found
);
tokio_test!(
    tk_open_write_missing_creates,
    ct::async_fs_open_write_missing_creates
);
tokio_test!(
    tk_open_write_existing_truncates,
    ct::async_fs_open_write_existing_truncates
);
tokio_test!(tk_metadata_existing, ct::async_fs_metadata_existing);
tokio_test!(
    tk_metadata_missing_nf,
    ct::async_fs_metadata_missing_is_not_found
);
tokio_test!(tk_rename_existing, ct::async_fs_rename_existing);
tokio_test!(
    tk_rename_missing_nf,
    ct::async_fs_rename_missing_source_is_not_found
);
tokio_test!(tk_file_read_exact, ct::async_fs_file_read_exact);
tokio_test!(tk_file_read_seek, ct::async_fs_file_read_seek);
tokio_test!(
    tk_file_write_flush,
    ct::async_fs_file_write_flush_observable
);
tokio_test!(tk_file_write_seek, ct::async_fs_file_write_seek);

// Sync-async interop against the real filesystem: std and tokio share the
// same tempdir root so a file written via one is visible via the other.
#[test]
fn std_tokio_interop() {
    let t = tmp();
    rt().block_on(ct::fs_sync_async_interop(
        &StdFileSystem::new(),
        &TokioFileSystem::new(),
        t.path(),
    ));
}
