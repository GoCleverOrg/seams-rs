# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] — 2026-04-14

Adds the `FileSystem` and `AsyncFileSystem` port families plus
deterministic fake (`MemoryFileSystem`) and std/tokio production
adapters (`StdFileSystem`, `TokioFileSystem`).

### Added

- `seams-rs-core`:
  - `FileSystem` trait (sync) with `create_dir_all`, `remove_dir_all`,
    `try_exists`, `open_read`, `open_write`, `metadata`, `rename`.
  - `AsyncFileSystem` trait (tokio-shaped) with the same operations
    returning `BoxFuture`s for dyn-compatibility.
  - Per-file-handle traits: `FileRead`, `FileWrite`, `AsyncFileRead`,
    `AsyncFileWrite`, each supporting `read_to_end`/`write_all`,
    `read_exact`, `flush`, and `seek` as appropriate.
  - `Metadata` DTO exposing `len`, `is_file`, `is_dir`, `modified` —
    the cross-platform intersection of `std::fs::Metadata` and
    `tokio::fs::Metadata`.
  - `BoxFuture<'a, T>` type alias for the owned `Send` future returned
    from async trait methods.
  - `contract_tests` helpers covering every trait method: sync and
    async variants for each of `create_dir_all` (missing-parents and
    idempotent), `remove_dir_all` (missing-is-NotFound and nonempty),
    `try_exists` (true and false), `open_read` (existing-yields-bytes
    and missing-is-NotFound), `open_write` (missing-creates and
    existing-truncates), `metadata` (existing and missing-is-NotFound),
    `rename` (existing and missing-source-is-NotFound), plus per-handle
    `read_exact`, `seek`, and `write_flush_observable`. Includes
    `fs_sync_async_interop` exercising sync+async duality.
- `seams-rs-fake`:
  - `MemoryFileSystem`: in-memory VFS implementing both `FileSystem`
    and `AsyncFileSystem` over a shared `Arc<Mutex<_>>` so async writes
    are visible to sync reads and vice versa.
  - `FsOp` enum tagging every operation for scripted error injection:
    `CreateDir`, `RemoveDir`, `Exists`, `OpenRead`, `OpenWrite`,
    `Metadata`, `Rename`, `Read`, `Write`, `Flush`, `Seek`.
  - `MemoryFileSystem::inject_error(path, op, kind)` — single-shot
    injection that causes the next matching op to return the supplied
    `io::ErrorKind`. Covers `NotFound`, `AlreadyExists`,
    `PermissionDenied`, and `StorageFull` (plus any other
    `io::ErrorKind`).
- `seams-rs-std`:
  - `StdFileSystem`: thin wrapper over `std::fs`.
  - `TokioFileSystem`: thin wrapper over `tokio::fs`.
- `seams-rs` facade: re-exports the full new surface. `StdFileSystem`
  and `TokioFileSystem` are gated behind the existing optional `std`
  feature.

## [0.1.1] — 2026-04-14

First release shipped to crates.io. (The `v0.1.0` tag was cut but the
publish workflow failed on the first-ever `cargo publish` of the
workspace — see "Fixed" below — so `v0.1.0` was never on crates.io and
has been deleted. `v0.1.1` is functionally the initial release.)

### Added

- `Clock` trait (`now_ns`, `now_instant`) in `seams-rs-core`.
- `Sleeper` trait (`sleep`, `sleep_responsive`) in `seams-rs-core`.
- `Spawner` trait (`spawn_blocking`) plus `JoinHandle` and `JoinError`
  in `seams-rs-core`.
- `contract_tests` module exposing reusable generic helpers that exercise
  each port's contract.
- `seams-rs-fake` adapters: `ManualClock`, `InstantSleeper`,
  `CurrentThreadSpawner`, `DeferredSpawner`.
- `seams-rs-std` adapters: `SystemClock`, `StdSleeper`, `StdSpawner`.
- `seams-rs` facade crate re-exporting the core traits and, under the
  `std` feature, the standard-library adapters.

### Fixed

- First-publish chicken-and-egg: intra-workspace dev-dependencies on
  `seams-rs-fake` in `seams-rs-core` and `seams-rs` are now path-only
  (no `version` field), so `cargo publish` does not try to resolve
  `seams-rs-fake` from crates.io before it exists there.
- `cargo-deny` `bans.wildcards = "deny"` blocked the path-only dev-deps;
  set `bans.allow-wildcard-paths = true` to exempt intra-workspace path
  deps while keeping the rule for external crates.
