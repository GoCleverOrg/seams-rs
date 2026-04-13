//! Deterministic in-memory implementations of every `seams-rs-core` port.
//!
//! `ManualClock` advances via explicit test calls. `InstantSleeper`
//! returns immediately and records requested durations. `CurrentThreadSpawner`
//! runs `spawn_blocking` closures inline; `DeferredSpawner` captures
//! handles for explicit test-controlled joining.
//!
//! Depends only on `seams-rs-core`. Populated by issue #1.

// Populated by issue #1.
