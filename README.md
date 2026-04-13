# seams-rs

Hexagonal strict-DI seams for time, sleeping, and thread spawning.

Code that uses `std::time::SystemTime::now`, `std::thread::spawn`, or
`std::thread::sleep` directly cannot be deterministically unit-tested.
This workspace replaces those calls with injected trait dependencies —
production wires up std-backed implementations; tests wire up
deterministic in-memory implementations.

## Architecture

```text
seams-rs-core    Clock / Sleeper / Spawner traits + DTOs; zero std-runtime deps
    ↑
    ├── seams-rs-fake   ManualClock / InstantSleeper / CurrentThreadSpawner for unit tests
    └── seams-rs-std    SystemClock / StdSleeper / StdSpawner wrapping std
            ↑
            seams-rs    user-facing facade (re-exports core; seams-rs-std behind the `std` feature)
```

| Crate | Description |
| --- | --- |
| [`seams-rs-core`](./seams-rs-core) | Ports + DTOs. No std-runtime deps. |
| [`seams-rs-fake`](./seams-rs-fake) | Deterministic in-memory impls for unit tests. |
| [`seams-rs-std`](./seams-rs-std) | Production impls wrapping the standard library. |
| [`seams-rs`](./seams-rs) | User-facing facade. |

## Architectural contract

1. **`seams-rs-core` has zero std-runtime deps** beyond `thiserror`. It
   contains only traits, DTOs, and (optionally) pure-Rust helper logic.
2. **`seams-rs-fake` has zero std-runtime deps**. It depends only on
   `seams-rs-core`.
3. **`seams-rs-std` is the only crate that wraps `std::{time, thread}`
   primitives**. Downstream consumers must never call those stdlib
   functions directly in orchestration code — they must go through the
   `Clock` / `Sleeper` / `Spawner` trait surfaces.

## Development

```
make test           # cargo test --workspace
make lint           # fmt + clippy + deny + shear + taplo + typos
make mutants        # full workspace mutation bench
```

## License

Dual-licensed under either:

- Apache License, Version 2.0 ([LICENSE-APACHE](./LICENSE-APACHE))
- MIT license ([LICENSE-MIT](./LICENSE-MIT))
