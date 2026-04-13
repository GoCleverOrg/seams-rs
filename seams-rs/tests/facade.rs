//! Integration test exercising the facade re-exports.

use seams_rs::{contract_tests as ct, Clock};
use seams_rs_fake::ManualClock;

#[test]
fn facade_reexports_core() {
    let clock = ManualClock::new();
    ct::clock_now_ns_monotonic(&clock);
    // Prove the re-exported trait is identical to the core one.
    let _: &dyn Clock = &clock;
}

#[test]
#[cfg(feature = "std")]
fn facade_reexports_std_backend() {
    let _clock = seams_rs::SystemClock::new();
}
