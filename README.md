# seams-rs

Hexagonal strict-DI seams for time, sleeping, thread spawning, and
filesystem I/O.

Code that uses `std::time::SystemTime::now`, `std::thread::spawn`,
`std::thread::sleep`, `std::fs::*`, or `tokio::fs::*` directly cannot
be deterministically unit-tested. This workspace replaces those calls
with injected trait dependencies — production wires up std/tokio-backed
implementations; tests wire up deterministic in-memory implementations.

## Architecture

```text
seams-rs-core    Clock / Sleeper / Spawner / FileSystem / AsyncFileSystem traits + DTOs; zero std-runtime deps
    ↑
    ├── seams-rs-fake   ManualClock / InstantSleeper / CurrentThreadSpawner / MemoryFileSystem for unit tests
    └── seams-rs-std    SystemClock / StdSleeper / StdSpawner / StdFileSystem / TokioFileSystem
            ↑
            seams-rs    user-facing facade (re-exports core; seams-rs-std behind the `std` feature)
```

| Crate | Description |
| --- | --- |
| [`seams-rs-core`](./seams-rs-core) | Ports + DTOs. No std-runtime deps. |
| [`seams-rs-fake`](./seams-rs-fake) | Deterministic in-memory impls for unit tests. |
| [`seams-rs-std`](./seams-rs-std) | Production impls wrapping `std` / `tokio`. |
| [`seams-rs`](./seams-rs) | User-facing facade. |

## Trait families

- **`Clock` / `Sleeper` / `Spawner`** — time, sleeping, and blocking
  closure spawning.
- **`FileSystem` (sync) and `AsyncFileSystem` (tokio-shaped)** — the
  same set of directory and file operations in both sync and async
  flavors. A single `MemoryFileSystem` instance implements *both*
  traits over the same backing state, so a test can write asynchronously
  and read synchronously (or vice versa) without re-setup. Per-handle
  `FileRead` / `FileWrite` / `AsyncFileRead` / `AsyncFileWrite` traits
  provide `read_exact`, `seek`, `write_all`, and `flush` on open files.
- **Scripted error injection** — `MemoryFileSystem::inject_error(path,
  FsOp, ErrorKind)` queues a single-shot error for the next matching
  operation. Covers `NotFound`, `AlreadyExists`, `PermissionDenied`,
  `StorageFull`, and any other `io::ErrorKind` — enabling
  deterministic tests of error paths that are difficult to trigger on
  a real filesystem.

## Architectural contract

1. **`seams-rs-core` has zero std-runtime deps** beyond `thiserror`. It
   contains only traits, DTOs, and (optionally) pure-Rust helper logic.
2. **`seams-rs-fake` has zero std-runtime deps**. It depends only on
   `seams-rs-core`.
3. **`seams-rs-std` is the only crate that wraps `std::{time, thread,
   fs}` and `tokio::fs` primitives**. Downstream consumers must never
   call those stdlib/tokio functions directly in orchestration code —
   they must go through the `Clock` / `Sleeper` / `Spawner` /
   `FileSystem` / `AsyncFileSystem` trait surfaces.

## Development

```
make test           # cargo test --workspace
make lint           # fmt + clippy + deny + shear + taplo + typos
make mutants        # full workspace mutation bench
```

## License

Dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE))
- MIT license ([LICENSE-MIT](./LICENSE-MIT))
