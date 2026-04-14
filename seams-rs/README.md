# seams-rs

Hexagonal strict-DI seams for time, sleeping, thread spawning, and
filesystem I/O. Mirrors the architecture of
[`vmb-rs`](https://github.com/GoCleverOrg/vmb-rs) and
[`opencv-rs`](https://github.com/GoCleverOrg/opencv-rs): pure ports in
`-core`, deterministic fakes in `-fake`, production backend in a
feature-gated `-std` crate, and a user-facing facade.

## Why

Code that uses `std::time::SystemTime::now`, `std::thread::spawn`,
`std::thread::sleep`, `std::fs::*`, or `tokio::fs::*` directly cannot
be deterministically unit-tested. The `seams-rs-core` traits replace
those calls with injected dependencies: production wires up
`seams-rs-std` implementations (`SystemClock`, `StdFileSystem`,
`TokioFileSystem`, etc.); tests wire up `seams-rs-fake`
implementations (`ManualClock`, `MemoryFileSystem`, etc.).

## Crates

| Crate | Description |
| --- | --- |
| [`seams-rs-core`](./seams-rs-core) | Traits + DTOs. Zero std-runtime deps. |
| [`seams-rs-fake`](./seams-rs-fake) | Deterministic in-memory impls for unit tests. |
| [`seams-rs-std`](./seams-rs-std) | Production impls wrapping `std` and `tokio`. |
| [`seams-rs`](./seams-rs) | User-facing facade. `seams-rs-std` behind the `std` feature. |

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
