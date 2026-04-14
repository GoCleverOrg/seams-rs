# seams-rs-std

Production implementations of every `seams-rs-core` port, backed by
the Rust standard library and `tokio`.

- `SystemClock` wraps `std::time::SystemTime::now`.
- `StdSleeper` wraps `std::thread::sleep`.
- `StdSpawner` wraps `std::thread::spawn`.
- `StdFileSystem` wraps `std::fs::*`.
- `TokioFileSystem` wraps `tokio::fs::*`.

Depends on `seams-rs-core` and `tokio` (with the `fs`, `io-util`, and
`rt` features). This is what production code should wire up via the
`seams-rs` facade.
