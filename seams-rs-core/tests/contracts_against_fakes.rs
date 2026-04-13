//! Integration tests exercising every `contract_tests` helper against the
//! `seams-rs-fake` adapters. Mirrors the fake crate's internal coverage
//! but at the crate boundary (so mutations inside `contract_tests` are
//! caught by these tests too).

use std::time::Duration;

use seams_rs_core::contract_tests as ct;
use seams_rs_fake::{CurrentThreadSpawner, DeferredSpawner, InstantSleeper, ManualClock};

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
