//! Hexagonal strict-DI seams for time, sleeping, and thread spawning.
//!
//! Convenience facade re-exporting the ports defined in `seams-rs-core`.
//! When the `std` feature is enabled, the production adapters from
//! `seams-rs-std` are also re-exported.
//!
//! ```text
//! seams-rs-core    ← Clock / Sleeper / Spawner traits + DTOs
//!     ↑
//!     ├── seams-rs-fake   ← ManualClock, InstantSleeper, CurrentThreadSpawner
//!     └── seams-rs-std    ← SystemClock, StdSleeper, StdSpawner
//!             ↑
//!             seams-rs    ← this facade
//! ```

pub use seams_rs_core as core;

#[cfg(feature = "std")]
pub use seams_rs_std as std_backend;
