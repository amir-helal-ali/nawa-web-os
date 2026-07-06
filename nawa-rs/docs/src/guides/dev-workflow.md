# Development Workflow

## nawa dev — Hot Reload

```bash
nawa dev --addr 0.0.0.0:8080 --data-dir ./nawa-data
```

Hot reload watches `src/` and automatically:
1. Kills the running server
2. Rebuilds with `cargo build --release -p nawad`
3. Restarts the server

Build failures show the error and keep the old binary running.

## Manual Development

```bash
# Terminal 1: build + run
cargo build --release -p nawad
./target/release/nawad serve --addr 0.0.0.0:8080 --data-dir ./nawa-data

# Terminal 2: test
curl http://localhost:8080/health
curl -X POST http://localhost:8080/test -d 'hello'
curl http://localhost:8080/test
```

## Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p nawa-db
cargo test -p nawa-uring

# Benchmarks
cargo bench -p nawa-db
cargo bench -p nawa-uring
```

## Environment Variables

```bash
RUST_LOG=info,nawa=debug    # logging level
RUST_BACKTRACE=1            # panic backtrace
```
