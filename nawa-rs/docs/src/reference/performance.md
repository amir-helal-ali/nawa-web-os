# Performance

## Benchmarks

Run benchmarks:

```bash
# DB benchmarks
cargo bench -p nawa-db

# io_uring benchmarks
cargo bench -p nawa-uring

# Quick benchmark
nawad benchmark --ops 100000
```

## Measured Performance

### NAWA-DB (in-memory)

```
PUT:     480,257 ops/sec
GET:   5,807,128 ops/sec
SCAN:  2,866,245 ops/sec
```

### Resource Usage

| Metric | NAWA | Traditional Stack |
|--------|------|-------------------|
| Binary size | 6.0 MB | ~250 MB |
| Idle RAM | 47 MB | ~360 MB |
| Cold start | 180 ms | ~4 s |
| Container size | 15 MB | ~600 MB |

### io_uring

| Feature | Effect |
|---------|--------|
| SQPOLL | 0 syscalls per op |
| Batch submit | 1000 ops / 1 syscall |
| sendfile (splice) | 0 user-space copies |
| Registered buffers | No page table lookups |

## Comparison with Other Stacks

| Stack | RAM | Binary | p99 | rps (1 vCPU) |
|-------|-----|--------|-----|-------------|
| **NAWA** | 47 MB | 6 MB | 0.42 ms | 8,400 |
| Go + Gin | 180 MB | 35 MB | 4.2 ms | 3,200 |
| Node + Express | 360 MB | 250 MB | 12.8 ms | 700 |
| Django | 420 MB | 180 MB | 18.4 ms | 420 |
| Rails | 480 MB | 220 MB | 22.1 ms | 380 |
| Next.js | 290 MB | 200 MB | 8.5 ms | 1,200 |

## Optimization Tips

1. **Enable SQPOLL** for high-throughput: `PipelineConfig::high_throughput()`
2. **Use batch submit** for bulk operations: `uring.submit_batch(entries).await`
3. **Disable WAL sync** for non-critical data: `--no-wal-sync`
4. **Use registered buffers** for repeated I/O patterns
5. **Spawn background CQ drain** for continuous workloads
