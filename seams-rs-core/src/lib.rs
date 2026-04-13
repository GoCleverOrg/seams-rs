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

use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

pub mod contract_tests;

/// A source of time. Implementations must be `Send + Sync + 'static`.
pub trait Clock: Send + Sync + 'static {
    /// Returns the current time expressed as nanoseconds since an
    /// implementation-defined epoch.
    fn now_ns(&self) -> u64;

    /// Returns a monotonic `Instant` for elapsed-time calculations.
    fn now_instant(&self) -> Instant;
}

/// A sleep primitive. Implementations must be `Send + Sync + 'static`.
pub trait Sleeper: Send + Sync + 'static {
    /// Blocks the current thread for at least `duration`.
    fn sleep(&self, duration: Duration);

    /// Sleeps up to `total`, returning early if `shutdown` becomes `true`.
    /// Returns `true` iff shutdown was observed.
    fn sleep_responsive(&self, total: Duration, shutdown: &AtomicBool) -> bool;
}

/// Owned join handle for a spawned task.
pub trait JoinHandle<T>: Send {
    /// Blocks until the task completes, returning its result or the join error.
    fn join(self: Box<Self>) -> Result<T, JoinError>;
}

/// Error returned from `JoinHandle::join`.
#[derive(Debug, thiserror::Error)]
pub enum JoinError {
    /// Task panicked with the given message.
    #[error("spawned task panicked: {0}")]
    Panicked(String),
    /// Task was cancelled before it could complete.
    #[error("spawned task was cancelled")]
    Cancelled,
}

/// Spawn primitive for blocking closures.
pub trait Spawner: Send + Sync + 'static {
    /// Spawn a blocking closure and return a join handle.
    fn spawn_blocking<F, T>(&self, f: F) -> Box<dyn JoinHandle<T>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static;
}
