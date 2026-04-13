//! Runtime-agnostic seams for strict-hexagonal testability.
//!
//! This crate defines the `Clock`, `Sleeper`, and `Spawner` traits plus
//! associated DTOs (`JoinHandle`, error types). Production code and test
//! code both depend on these traits; neither `std::time::SystemTime`,
//! `std::thread::spawn`, nor `std::thread::sleep` should appear
//! anywhere in downstream orchestration code.
//!
//! Production implementations live in `seams-rs-std`. Deterministic
//! in-memory implementations for unit tests live in `seams-rs-fake`.
//! End users depend on the `seams-rs` facade.
//!
//! The trait surface is filled in by issue #1 on this repo.

// Populated by issue #1.
