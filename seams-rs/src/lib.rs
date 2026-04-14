//! Hexagonal strict-DI seams for time, sleeping, thread spawning, and
//! filesystem I/O.
//!
//! Convenience facade re-exporting the ports defined in `seams-rs-core`.
//! When the `std` feature is enabled, the production adapters from
//! `seams-rs-std` are also re-exported.
//!
//! ```text
//! seams-rs-core    ← Clock / Sleeper / Spawner / FileSystem / AsyncFileSystem traits + DTOs
//!     ↑
//!     ├── seams-rs-fake   ← ManualClock, InstantSleeper, CurrentThreadSpawner, MemoryFileSystem
//!     └── seams-rs-std    ← SystemClock, StdSleeper, StdSpawner, StdFileSystem, TokioFileSystem
//!             ↑
//!             seams-rs    ← this facade
//! ```

pub use seams_rs_core::{
    contract_tests, AsyncFileRead, AsyncFileSystem, AsyncFileWrite, BoxFuture, Clock, FileRead,
    FileSystem, FileWrite, JoinError, JoinHandle, Metadata, Sleeper, Spawner,
};

#[cfg(feature = "std")]
pub use seams_rs_std::{StdFileSystem, StdSleeper, StdSpawner, SystemClock, TokioFileSystem};
