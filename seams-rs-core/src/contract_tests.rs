//! Reusable contract tests for `Clock`, `Sleeper`, and `Spawner`
//! implementations. Each helper asserts contract invariants and panics
//! via `assert!` on violation.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::{Clock, JoinError, Sleeper, Spawner};

/// Two successive `now_ns` calls must not go backwards.
pub fn clock_now_ns_monotonic<C: Clock>(clock: &C) {
    let a = clock.now_ns();
    let b = clock.now_ns();
    assert!(b >= a, "now_ns went backwards: {a} -> {b}");
}

/// Two successive `now_instant` calls must not go backwards.
pub fn clock_now_instant_monotonic<C: Clock>(clock: &C) {
    let a = clock.now_instant();
    let b = clock.now_instant();
    assert!(b >= a, "now_instant went backwards");
}

/// Measures wall elapsed around `sleeper.sleep(duration)` and asserts it
/// does not exceed `upper_bound`. Lower-bound checks are left to the caller
/// because fake sleepers legitimately return instantly.
pub fn sleeper_sleep_waits<S: Sleeper>(sleeper: &S, duration: Duration, upper_bound: Duration) {
    let start = Instant::now();
    sleeper.sleep(duration);
    let elapsed = start.elapsed();
    assert!(
        elapsed <= upper_bound,
        "sleep took {elapsed:?}, bound {upper_bound:?}"
    );
}

/// When `shutdown` is already `true`, `sleep_responsive` must return
/// immediately with `true`.
pub fn sleeper_responsive_shutdown_before<S: Sleeper>(sleeper: &S) {
    let shutdown = AtomicBool::new(true);
    let start = Instant::now();
    let triggered = sleeper.sleep_responsive(Duration::from_secs(10), &shutdown);
    let elapsed = start.elapsed();
    assert!(triggered, "expected shutdown observed");
    assert!(
        elapsed <= Duration::from_millis(100),
        "returned too slowly: {elapsed:?}"
    );
}

/// When `shutdown` flips during a long sleep, the sleeper must observe it
/// and return early. For instant-returning fakes, the call completes before
/// the flag is flipped — in that case the helper tolerates a fast return.
pub fn sleeper_responsive_shutdown_during<S: Sleeper>(sleeper: &S) {
    let shutdown = Arc::new(AtomicBool::new(false));
    let flag = shutdown.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        flag.store(true, Ordering::SeqCst);
    });
    let start = Instant::now();
    let triggered = sleeper.sleep_responsive(Duration::from_secs(10), &shutdown);
    let elapsed = start.elapsed();
    // Either shutdown was observed, or the sleeper returned before the
    // flag-flip thread could store (fake case).
    let ok = triggered || elapsed < Duration::from_millis(20);
    assert!(ok, "sleeper waited {elapsed:?} without observing shutdown");
    assert!(elapsed < Duration::from_secs(5));
}

/// Without shutdown, `sleep_responsive` returns `false`.
pub fn sleeper_responsive_no_shutdown<S: Sleeper>(sleeper: &S) {
    let shutdown = AtomicBool::new(false);
    let triggered = sleeper.sleep_responsive(Duration::from_millis(5), &shutdown);
    assert!(!triggered);
}

/// `spawn_blocking` must deliver the closure's return value to the joiner.
pub fn spawner_returns_value<S: Spawner>(spawner: &S) {
    let handle = spawner.spawn_blocking(|| 42i32);
    assert_eq!(handle.join().unwrap(), 42);
}

/// A panic in the spawned closure must surface as `JoinError::Panicked`.
pub fn spawner_propagates_panic<S: Spawner>(spawner: &S) {
    let handle = spawner.spawn_blocking::<_, ()>(|| panic!("boom"));
    match handle.join() {
        Err(JoinError::Panicked(msg)) => {
            assert!(msg.contains("boom"), "got: {msg}")
        }
        other => panic!("expected Panicked, got {other:?}"),
    }
}
