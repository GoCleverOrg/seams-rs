# seams-rs-core

Runtime-agnostic seams for strict-hexagonal testability. Defines the
`Clock`, `Sleeper`, `Spawner`, `FileSystem`, and `AsyncFileSystem`
trait families plus the per-file-handle traits (`FileRead`,
`FileWrite`, `AsyncFileRead`, `AsyncFileWrite`), the `Metadata` DTO,
and the `BoxFuture` type alias.

Depends only on `thiserror`. No std-runtime or tokio dependency — only
trait + DTO definitions. Production implementations
(`StdFileSystem`, `TokioFileSystem`, `SystemClock`, etc.) live in
`seams-rs-std`; deterministic in-memory fakes (`MemoryFileSystem`,
`ManualClock`, etc.) live in `seams-rs-fake`. End users typically want
the `seams-rs` facade, not this crate directly.

## Contract tests

The public `contract_tests` module exposes generic helpers that exercise
each port's contract. Downstream adapter crates invoke every helper
against their own implementation — the same assertions are reused
verbatim for fakes and for the production impls. Sync and async
filesystem helpers are both provided, plus a `fs_sync_async_interop`
helper that verifies both trait objects see the same underlying state.
