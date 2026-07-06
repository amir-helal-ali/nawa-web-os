# FAQ

## General

### Is NAWA production-ready?
Currently v0.1.0-alpha. Phase 1 (Foundation) is complete, Phase 2 (Database) is in active development. Stable v1.0 expected Q1 2027.

### Why Rust?
Rust provides: (1) zero-cost abstractions for zero-copy I/O, (2) memory safety without GC, (3) strong type system catching errors at compile-time. No other language offers all three.

### Does it support SQL?
No, not in v1.0. NAWA-DB uses a Key-Value + Document interface. SQL may come as a WASM plugin in v2.0.

## Technical

### How does the built-in DB work without PostgreSQL?
NAWA-DB is written from scratch in Rust. Uses LSM-tree (like LevelDB/RocksDB) with: mmap for direct disk access, lock-free skip-list for MemTable, bloom filters for SSTables.

### Does it really work on 512MB RAM?
Yes. The binary uses ~47MB idle, leaving ~465MB for your app. Tested on Hetzner CX11 ($3/mo) and Raspberry Pi 4.

### What's the difference between the two engines?
- **Backend Engine**: HTTP/3 server, routing, DB, auth, I/O
- **Frontend Engine**: SSR rendering, island hydration, edge cache, hot reload

Both are in the same binary, communicating via shared memory + ring buffers.

## Deployment

### How do I deploy to a VPS?
```bash
nawa deploy --target user@your-vps
```
This builds, packages, uploads via SCP, and starts the server via SSH.

### Can I use Docker?
Yes:
```bash
docker build -t nawa-app .
docker run -d -p 80:8080 -v nawa-data:/var/lib/nawa nawa-app
```

### Does it support HTTP/3?
Yes, via quinn + h3. Requires TLS 1.3 certificate.

## Community

### How do I contribute?
Read [Contributing](contributing.md) and the [Manifesto](../manifesto.md).

### What license?
MIT + Apache 2.0 (dual-licensed, like Rust itself).
