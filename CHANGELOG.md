# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] — 2026-04-14

First release shipped to crates.io. (The `v0.1.0` tag was cut but the
publish workflow failed on the first-ever `cargo publish` of the
workspace — see "Fixed" below — so `v0.1.0` was never on crates.io and
has been deleted. `v0.1.1` is functionally the initial release.)

### Added

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

### Fixed

- First-publish chicken-and-egg: intra-workspace dev-dependencies on
  `seams-rs-fake` in `seams-rs-core` and `seams-rs` are now path-only
  (no `version` field), so `cargo publish` does not try to resolve
  `seams-rs-fake` from crates.io before it exists there.
- `cargo-deny` `bans.wildcards = "deny"` blocked the path-only dev-deps;
  set `bans.allow-wildcard-paths = true` to exempt intra-workspace path
  deps while keeping the rule for external crates.
