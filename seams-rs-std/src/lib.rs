//! Standard-library-backed production adapters for `seams-rs-core`.
//!
//! - `SystemClock` wraps `std::time::SystemTime::now`.
//! - `StdSleeper` wraps `std::thread::sleep`.
//! - `StdSpawner` wraps `std::thread::spawn`.
//!
//! Depends only on `seams-rs-core`. Populated by issue #1.

// Populated by issue #1.
