//! Standard-library-backed production adapters for `seams-rs-core`.
//!
//! - `SystemClock` wraps `std::time::SystemTime::now`.
//! - `StdSleeper` wraps `std::thread::sleep`.
//! - `StdSpawner` wraps `std::thread::spawn`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use seams_rs_core::{Clock, JoinError, JoinHandle, Sleeper, Spawner};

/// Clock driven by `SystemTime::now` + `Instant::now`.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl SystemClock {
    /// Construct a new system clock.
    pub fn new() -> Self {
        Self
    }
}

impl Clock for SystemClock {
    fn now_ns(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }

    fn now_instant(&self) -> Instant {
        Instant::now()
    }
}

/// Sleeper backed by `std::thread::sleep`.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdSleeper;

impl StdSleeper {
    /// Construct a new std sleeper.
    pub fn new() -> Self {
        Self
    }
}

const POLL_INTERVAL: Duration = Duration::from_millis(10);

impl Sleeper for StdSleeper {
    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }

    fn sleep_responsive(&self, total: Duration, shutdown: &AtomicBool) -> bool {
        let deadline = Instant::now() + total;
        loop {
            if shutdown.load(Ordering::SeqCst) {
                return true;
            }
            let now = Instant::now();
            if now >= deadline {
                return false;
            }
            let remaining = deadline - now;
            std::thread::sleep(remaining.min(POLL_INTERVAL));
        }
    }
}

/// Spawner backed by `std::thread::spawn`.
#[derive(Debug, Default, Clone, Copy)]
pub struct StdSpawner;

impl StdSpawner {
    /// Construct a new std spawner.
    pub fn new() -> Self {
        Self
    }
}

struct StdJoinHandle<T> {
    inner: std::thread::JoinHandle<T>,
}

impl<T: Send + 'static> JoinHandle<T> for StdJoinHandle<T> {
    fn join(self: Box<Self>) -> Result<T, JoinError> {
        self.inner.join().map_err(|payload| {
            let msg = if let Some(s) = payload.downcast_ref::<&'static str>() {
                (*s).to_string()
            } else if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else {
                "panic".to_string()
            };
            JoinError::Panicked(msg)
        })
    }
}

impl Spawner for StdSpawner {
    fn spawn_blocking<F, T>(&self, f: F) -> Box<dyn JoinHandle<T>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        Box::new(StdJoinHandle {
            inner: std::thread::spawn(f),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use seams_rs_core::contract_tests as ct;

    #[test]
    fn system_clock_now_ns() {
        ct::clock_now_ns_monotonic(&SystemClock::new());
    }

    #[test]
    fn system_clock_now_instant() {
        ct::clock_now_instant_monotonic(&SystemClock::new());
    }

    #[test]
    fn std_sleeper_sleep() {
        ct::sleeper_sleep_waits(
            &StdSleeper::new(),
            Duration::from_millis(5),
            Duration::from_millis(500),
        );
    }

    #[test]
    fn std_sleeper_before() {
        ct::sleeper_responsive_shutdown_before(&StdSleeper::new());
    }

    #[test]
    fn std_sleeper_during() {
        ct::sleeper_responsive_shutdown_during(&StdSleeper::new());
    }

    #[test]
    fn std_sleeper_no_shutdown() {
        ct::sleeper_responsive_no_shutdown(&StdSleeper::new());
    }

    #[test]
    fn std_spawner_value() {
        ct::spawner_returns_value(&StdSpawner::new());
    }

    #[test]
    fn std_spawner_panic() {
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        ct::spawner_propagates_panic(&StdSpawner::new());
        std::panic::set_hook(prev);
    }
}
