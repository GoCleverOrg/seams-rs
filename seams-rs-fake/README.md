# seams-rs-fake

Deterministic in-memory implementations of every `seams-rs-core` port
for use in unit tests.

- `ManualClock` — advances only when tests explicitly call `advance()`.
- `InstantSleeper` — returns immediately, records requested durations.
- `CurrentThreadSpawner` — runs `spawn_blocking` inline.
- `DeferredSpawner` — captures handles for explicit test-controlled join.

Depends only on `seams-rs-core`.
