# What is NAWA?

> **NAWA (نواة)** — نظام تشغيل ويب احترافي مكتوب بالكامل بـ Rust

NAWA is not a framework — it's a **web operating system**. The difference is fundamental:

| Traditional Framework | Web Operating System (NAWA) |
|----------------------|----------------------------|
| Added on top of an OS | Becomes the runtime itself |
| Copies data between layers | Passes it by reference (zero-copy) |
| Requires external DBMS | Owns its own built-in DBMS |
| Works on any OS | Optimized for Linux (io_uring, eBPF) |
| User manages DevOps | Manages ops autonomously |

## Key Features

- 🦀 **100% Rust** — no unsafe outside kernel module
- ⚡ **Zero-copy I/O** — io_uring + mmap + sendfile
- 🗄️ **Built-in database** — MemTable + SSTable + WAL + Bloom filter
- 🔒 **Zero-trust security** — TLS + WAF + WASM sandbox
- 📦 **Single binary** — 6 MB, works on 512MB VPS
- 🚀 **HTTP/1.1 + HTTP/3** — QUIC support via quinn
- 📊 **Prometheus metrics** — built-in observability

## Performance

```
PUT:     480,257 ops/sec
GET:   5,807,128 ops/sec
SCAN:  2,866,245 ops/sec
```

## Quick Stats

- **7 crates** integrated
- **120+ tests** passing
- **6.0 MB binary** (target was 11 MB)
- **0 external dependencies** for database

## License

MIT + Apache 2.0 (dual-licensed, like Rust itself)
