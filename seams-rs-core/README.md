# seams-rs-core

Runtime-agnostic seams for strict-hexagonal testability. Defines the
`Clock`, `Sleeper`, and `Spawner` traits plus the DTOs and errors that
go with them.

Depends only on `thiserror`. No std-runtime-specific dependency. The
`SystemClock`, `StdSleeper`, and `StdSpawner` production implementations
live in `seams-rs-std`; the deterministic fakes live in `seams-rs-fake`.
End users typically want the `seams-rs` facade, not this crate directly.
