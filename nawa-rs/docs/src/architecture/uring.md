# nawa-uring

Real `io_uring` implementation for Linux 5.1+. Falls back to tokio simulation on other platforms.

## Why io_uring?

| Traditional (epoll) | io_uring |
|---------------------|----------|
| 2 syscalls per op | 0 syscalls (SQPOLL) |
| 2 context switches | 0 context switches |
| Memory copies | Zero-copy (mmap + sendfile) |
| Sequential | Batched (1000 ops, 1 syscall) |

## Configuration

```rust
use nawa_uring::{NawaUring, PipelineConfig, SqPollConfig};

// Default config (256 entries, no SQPOLL)
let uring = NawaUring::default()?;

// High throughput (1024 entries + SQPOLL)
let uring = NawaUring::high_throughput()?;

// Custom config
let config = PipelineConfig {
    entries: 512,
    sqpoll: Some(SqPollConfig::aggressive()),
    iopoll: false,
    completion_timeout: std::time::Duration::from_secs(30),
};
let uring = NawaUring::new(config)?;
```

## Supported Operations

| OpCode | Description |
|--------|-------------|
| Read | Read from fd into buffer |
| Write | Write buffer to fd |
| Send | Send data over socket |
| Recv | Receive data from socket |
| SendFile | Zero-copy file → socket (via splice) |
| Accept | Accept connection |
| Close | Close fd |
| Fsync | fsync for durability |
| OpenAt | Open file |

## SQPOLL Mode

When SQPOLL is enabled, a kernel thread polls the submission queue, eliminating syscalls entirely:

```rust
let config = PipelineConfig {
    sqpoll: Some(SqPollConfig::default()), // 1s idle timeout
    ..Default::default()
};
```

## Background CQ Drain

For high-throughput scenarios, spawn a background task to drain the completion queue:

```rust
let mut uring = NawaUring::high_throughput()?;
uring.spawn_background_drain()?;
```

## Statistics

```rust
let stats = uring.stats();
println!("submitted: {}", stats.submitted);
println!("completed: {}", stats.completed);
println!("in_flight: {}", stats.in_flight);
```

## Tests
- 28 unit tests
- 11 integration tests
- 10 real-file tests (with actual tempfiles)
