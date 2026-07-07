# Architecture Overview

NAWA is built as a Rust workspace with 7 crates, each with a single responsibility.

```
┌─────────────────────────────────────────────────────────────┐
│                     nawa-cli (CLI tool)                      │
│  create · dev · build · deploy · benchmark · info            │
└────────────────────────┬────────────────────────────────────┘
                         │ spawns
┌────────────────────────▼────────────────────────────────────┐
│                     nawad (server binary)                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐  │
│  │ Router   │  │ Metrics  │  │ Handlers │  │ CLI parser │  │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────────────┘  │
└───────┼─────────────┼─────────────┼─────────────────────────┘
        │             │             │
┌───────▼─────┐ ┌─────▼─────┐ ┌────▼──────────┐
│  nawa-http  │ │ nawa-wasm │ │   nawa-db     │
│ HTTP/1.1    │ │ sandbox   │ │ MemTable      │
│ HTTP/3      │ │ wasmtime  │ │ SSTable       │
│ TLS + ACME  │ │           │ │ WAL + Bloom   │
│ Router      │ │           │ │ SkipList      │
└──────┬──────┘ └───────────┘ │ Compaction    │
       │                      └───────┬───────┘
┌──────▼──────────┐                  │
│  nawa-uring      │                  │
│  io_uring (Linux)│                  │
│  SQPOLL          │                  │
│  sendfile        │                  │
│  registered bufs │                  │
└──────┬───────────┘                  │
       │                              │
┌──────▼──────────────────────────────▼──┐
│           nawa-kernel                   │
│  zero-copy · mmap · ring buffer        │
└─────────────────────────────────────────┘
```

## Crate Dependencies

```
nawa-cli → nawad
nawad → nawa-http, nawa-db, nawa-uring, nawa-wasm, nawa-kernel
nawa-http → nawa-kernel, nawa-db
nawa-db → nawa-kernel, nawa-uring (optional)
nawa-uring → (standalone)
nawa-wasm → nawa-kernel
nawa-kernel → (standalone)
```

## Design Principles

1. **No cyclic dependencies** — each crate depends only on lower layers
2. **Feature flags** for optional functionality (e.g., `uring-wal`)
3. **Platform abstraction** — nawa-uring falls back to tokio on non-Linux
4. **Thread safety** — all shared state uses `Arc<Mutex<T>>` or atomics
5. **Async-native** — tokio runtime throughout
