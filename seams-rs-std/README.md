# seams-rs-std

Production implementations of every `seams-rs-core` port, backed by the
Rust standard library.

- `SystemClock` wraps `std::time::SystemTime::now`.
- `StdSleeper` wraps `std::thread::sleep`.
- `StdSpawner` wraps `std::thread::spawn`.

Depends only on `seams-rs-core` (itself depending only on `thiserror`).
This is what production code should wire up via the `seams-rs` facade.
