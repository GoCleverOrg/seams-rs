# seams-rs-fake

Deterministic in-memory implementations of every `seams-rs-core` port
for use in unit tests.

- `ManualClock` — advances only when tests explicitly call `advance()`.
- `InstantSleeper` — returns immediately, records requested durations.
- `CurrentThreadSpawner` — runs `spawn_blocking` inline.
- `DeferredSpawner` — captures handles for explicit test-controlled join.
- `MemoryFileSystem` — in-memory VFS implementing both the sync
  `FileSystem` and the async `AsyncFileSystem` over the same state
  (`Arc<Mutex<_>>`). Cloning returns another handle to the same state,
  so async writes are visible to sync reads and vice versa.
  `inject_error(path, FsOp, ErrorKind)` queues a single-shot error
  returned by the next matching operation, making error paths like
  `NotFound`, `AlreadyExists`, `PermissionDenied`, and `StorageFull`
  fully deterministic.

Depends only on `seams-rs-core`.
