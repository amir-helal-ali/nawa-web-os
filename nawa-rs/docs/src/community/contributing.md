# Contributing

We welcome contributions! Please read the [Manifesto](../manifesto.md) first — every PR must respect the 10 principles.

## Getting Started

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Run tests: `cargo test --workspace`
5. Run clippy: `cargo clippy --workspace -- -D warnings`
6. Check formatting: `cargo fmt --all -- --check`
7. Commit: `git commit -m 'Add amazing feature'`
8. Push: `git push origin feature/amazing-feature`
9. Open a Pull Request

## Code Style

- Follow Rust standard formatting (`rustfmt`)
- No `unsafe` outside `nawa-kernel` and `nawa-uring`
- All `unsafe` blocks need `// SAFETY:` comments
- Every public API needs doc comments
- Tests required for new functionality

## CI/CD

GitHub Actions runs on every PR:
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test --workspace`
- `cargo build --release`

## Areas Needing Help

- [ ] Real io_uring SQPOLL testing on bare metal
- [ ] HTTP/3 integration tests
- [ ] WASM plugin marketplace
- [ ] Documentation translations
- [ ] Performance benchmarks on ARM
