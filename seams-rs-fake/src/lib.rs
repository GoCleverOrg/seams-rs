//! Deterministic in-memory implementations of every `seams-rs-core` port.
//!
//! `ManualClock` advances via explicit test calls. `InstantSleeper`
//! returns immediately and records requested durations. `CurrentThreadSpawner`
//! runs `spawn_blocking` closures inline; `DeferredSpawner` captures
//! handles for explicit test-controlled joining.

use std::any::Any;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use seams_rs_core::{Clock, JoinError, JoinHandle, Sleeper, Spawner};

// ---------------- ManualClock ----------------

/// Clock whose time advances only via explicit `advance` / `set_now_ns` calls.
#[derive(Debug, Clone)]
pub struct ManualClock {
    now_ns: Arc<Mutex<u64>>,
    start: Arc<Instant>,
    initial_ns: u64,
}

impl ManualClock {
    /// Create a manual clock anchored at 0 ns.
    pub fn new() -> Self {
        Self::from_ns(0)
    }

    /// Create a manual clock anchored at `ns`.
    pub fn from_ns(ns: u64) -> Self {
        Self {
            now_ns: Arc::new(Mutex::new(ns)),
            start: Arc::new(Instant::now()),
            initial_ns: ns,
        }
    }

    /// Advance the clock by `d`.
    pub fn advance(&self, d: Duration) {
        let mut g = self.now_ns.lock().unwrap();
        *g = g.saturating_add(d.as_nanos() as u64);
    }

    /// Set the absolute time to `ns`.
    pub fn set_now_ns(&self, ns: u64) {
        *self.now_ns.lock().unwrap() = ns;
    }
}

impl Default for ManualClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for ManualClock {
    fn now_ns(&self) -> u64 {
        *self.now_ns.lock().unwrap()
    }

    fn now_instant(&self) -> Instant {
        let offset = self.now_ns().saturating_sub(self.initial_ns);
        *self.start + Duration::from_nanos(offset)
    }
}

// ---------------- InstantSleeper ----------------

/// Sleeper that records each request and returns immediately.
#[derive(Debug, Default, Clone)]
pub struct InstantSleeper {
    calls: Arc<Mutex<Vec<(Duration, bool)>>>,
}

impl InstantSleeper {
    /// Construct a new recorder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot of all recorded calls: `(requested, shutdown_observed)`.
    pub fn calls(&self) -> Vec<(Duration, bool)> {
        self.calls.lock().unwrap().clone()
    }
}

impl Sleeper for InstantSleeper {
    fn sleep(&self, duration: Duration) {
        self.calls.lock().unwrap().push((duration, false));
    }

    fn sleep_responsive(&self, total: Duration, shutdown: &AtomicBool) -> bool {
        let flag = shutdown.load(Ordering::SeqCst);
        self.calls.lock().unwrap().push((total, flag));
        flag
    }
}

// ---------------- Spawners ----------------

fn panic_msg(payload: &(dyn Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "panic".to_string()
    }
}

/// Spawner that runs each closure synchronously on the calling thread.
#[derive(Debug, Default, Clone, Copy)]
pub struct CurrentThreadSpawner;

impl CurrentThreadSpawner {
    /// Create a new inline spawner.
    pub fn new() -> Self {
        Self
    }
}

struct CurrentThreadHandle<T> {
    result: Result<T, JoinError>,
}

impl<T: Send> JoinHandle<T> for CurrentThreadHandle<T> {
    fn join(self: Box<Self>) -> Result<T, JoinError> {
        self.result
    }
}

impl Spawner for CurrentThreadSpawner {
    fn spawn_blocking<F, T>(&self, f: F) -> Box<dyn JoinHandle<T>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
            .map_err(|payload| JoinError::Panicked(panic_msg(&*payload)));
        Box::new(CurrentThreadHandle { result })
    }
}

type Thunk = Box<dyn FnOnce() + Send + 'static>;
type ThunkSlot = Arc<Mutex<Option<Thunk>>>;

/// Spawner that defers closure execution until `run_pending` / `join_all`
/// or an explicit `join` on the returned handle.
#[derive(Default)]
pub struct DeferredSpawner {
    pending: Arc<Mutex<Vec<ThunkSlot>>>,
}

impl DeferredSpawner {
    /// Create a new deferred spawner with no pending work.
    pub fn new() -> Self {
        Self::default()
    }

    /// Run every currently-pending closure in FIFO order.
    pub fn run_pending(&self) {
        let taken: Vec<ThunkSlot> = std::mem::take(&mut *self.pending.lock().unwrap());
        for slot in taken {
            if let Some(t) = slot.lock().unwrap().take() {
                t();
            }
        }
    }

    /// Alias for `run_pending`.
    pub fn join_all(&self) {
        self.run_pending();
    }

    /// Number of not-yet-run closures.
    pub fn pending_count(&self) -> usize {
        self.pending.lock().unwrap().len()
    }
}

struct DeferredHandle<T> {
    own_thunk: ThunkSlot,
    slot: Arc<Mutex<Option<Result<T, JoinError>>>>,
}

impl<T: Send + 'static> JoinHandle<T> for DeferredHandle<T> {
    fn join(self: Box<Self>) -> Result<T, JoinError> {
        if let Some(t) = self.own_thunk.lock().unwrap().take() {
            t();
        }
        match self.slot.lock().unwrap().take() {
            Some(r) => r,
            None => Err(JoinError::Cancelled),
        }
    }
}

impl Spawner for DeferredSpawner {
    fn spawn_blocking<F, T>(&self, f: F) -> Box<dyn JoinHandle<T>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let slot: Arc<Mutex<Option<Result<T, JoinError>>>> = Arc::new(Mutex::new(None));
        let slot_writer = slot.clone();
        let thunk: Thunk = Box::new(move || {
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
                .map_err(|payload| JoinError::Panicked(panic_msg(&*payload)));
            *slot_writer.lock().unwrap() = Some(r);
        });
        let own: ThunkSlot = Arc::new(Mutex::new(Some(thunk)));
        self.pending.lock().unwrap().push(own.clone());
        Box::new(DeferredHandle {
            own_thunk: own,
            slot,
        })
    }
}

// ---------------- Tests ----------------

#[cfg(test)]
mod tests {
    use super::*;
    use seams_rs_core::contract_tests as ct;
    use std::time::Duration;

    #[test]
    fn manual_clock_now_ns() {
        ct::clock_now_ns_monotonic(&ManualClock::new());
    }

    #[test]
    fn manual_clock_now_instant() {
        ct::clock_now_instant_monotonic(&ManualClock::new());
    }

    #[test]
    fn instant_sleeper_sleep() {
        ct::sleeper_sleep_waits(
            &InstantSleeper::new(),
            Duration::ZERO,
            Duration::from_millis(50),
        );
    }

    #[test]
    fn instant_sleeper_shutdown_before() {
        ct::sleeper_responsive_shutdown_before(&InstantSleeper::new());
    }

    #[test]
    fn instant_sleeper_shutdown_during() {
        ct::sleeper_responsive_shutdown_during(&InstantSleeper::new());
    }

    #[test]
    fn instant_sleeper_no_shutdown() {
        ct::sleeper_responsive_no_shutdown(&InstantSleeper::new());
    }

    #[test]
    fn current_thread_spawner_value() {
        ct::spawner_returns_value(&CurrentThreadSpawner::new());
    }

    #[test]
    fn current_thread_spawner_panic() {
        ct::spawner_propagates_panic(&CurrentThreadSpawner::new());
    }

    #[test]
    fn deferred_spawner_value() {
        ct::spawner_returns_value(&DeferredSpawner::new());
    }

    #[test]
    fn deferred_spawner_panic() {
        ct::spawner_propagates_panic(&DeferredSpawner::new());
    }

    #[test]
    fn manual_clock_advance() {
        let c = ManualClock::from_ns(1000);
        c.advance(Duration::from_nanos(500));
        assert_eq!(c.now_ns(), 1500);
    }

    #[test]
    fn manual_clock_set() {
        let c = ManualClock::new();
        c.set_now_ns(42);
        assert_eq!(c.now_ns(), 42);
    }

    #[test]
    fn instant_sleeper_records_calls() {
        let s = InstantSleeper::new();
        s.sleep(Duration::from_millis(10));
        assert_eq!(s.calls().len(), 1);
        assert_eq!(s.calls()[0].0, Duration::from_millis(10));
    }

    #[test]
    fn deferred_spawner_defers() {
        let s = DeferredSpawner::new();
        let h = s.spawn_blocking(|| 7);
        assert_eq!(s.pending_count(), 1);
        s.run_pending();
        assert_eq!(h.join().unwrap(), 7);
    }
}
