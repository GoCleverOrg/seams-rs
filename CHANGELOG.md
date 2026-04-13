# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] — TBD

### Fixed

- First-publish chicken-and-egg: intra-workspace dev-dependencies on
  `seams-rs-fake` in `seams-rs-core` and `seams-rs` are now path-only
  (no `version` field), so `cargo publish` does not try to resolve
  `seams-rs-fake` from crates.io before it exists there. No runtime
  behaviour change.

## [0.1.0] — TBD

First release.

### Added

- Initial workspace scaffold.
- `Clock` trait (`now_ns`, `now_instant`) in `seams-rs-core`.
- `Sleeper` trait (`sleep`, `sleep_responsive`) in `seams-rs-core`.
- `Spawner` trait (`spawn_blocking`) plus `JoinHandle` and `JoinError`
  in `seams-rs-core`.
- `contract_tests` module exposing reusable generic helpers that exercise
  each port's contract.
- `seams-rs-fake` adapters: `ManualClock`, `InstantSleeper`,
  `CurrentThreadSpawner`, `DeferredSpawner`.
- `seams-rs-std` adapters: `SystemClock`, `StdSleeper`, `StdSpawner`.
- `seams-rs` facade crate re-exporting the core traits and, under the
  `std` feature, the standard-library adapters.
