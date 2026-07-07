# Configuration

## nawad serve

```
nawad serve [OPTIONS]

Options:
  --addr <ADDR>           Bind address [default: 0.0.0.0:8080]
  --data-dir <DIR>        Data directory [default: ./nawa-data]
  --no-wal-sync           Disable WAL fsync (faster, less durable)
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (debug, info, warn, error) |
| `RUST_BACKTRACE` | `0` | Show panic backtrace (1=short, full=full) |

## Pipeline Configuration

```rust
use nawa_uring::PipelineConfig;

// Default: 256 entries, no SQPOLL
let config = PipelineConfig::default();

// High throughput: 1024 entries + SQPOLL
let config = PipelineConfig::high_throughput();

// Low latency: 64 entries + IOPOLL
let config = PipelineConfig::low_latency();

// Minimal: 32 entries
let config = PipelineConfig::minimal();

// Custom
let config = PipelineConfig {
    entries: 512,
    sqpoll: Some(SqPollConfig::aggressive()),
    iopoll: false,
    completion_timeout: Duration::from_secs(60),
};
```

## Database Configuration

```rust
use nawa_db::DbConfig;

let config = DbConfig {
    data_dir: "/var/lib/nawa".into(),
    memtable_max_size: 4 * 1024 * 1024, // 4 MB
    wal_sync: true,
};
```

## WASM Sandbox Configuration

```rust
use nawa_wasm::SandboxConfig;

let config = SandboxConfig {
    fuel_limit: 1_000_000,           // 1M instructions
    memory_limit: 64 * 1024 * 1024,  // 64 MB
    allow_wasi: false,                // no filesystem/network
};
```

## Docker

```yaml
# docker-compose.yml
services:
  nawa:
    image: nawa/os:0.1.0
    ports:
      - "80:8080"
    volumes:
      - nawa-data:/var/lib/nawa
    environment:
      RUST_LOG: "info,nawa=debug"
    deploy:
      resources:
        limits:
          cpus: "1.0"
          memory: 512M
```
