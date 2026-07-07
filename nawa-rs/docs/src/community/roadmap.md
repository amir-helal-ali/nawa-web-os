# Roadmap

## Phase 1 — Foundation ✅ DONE
- Rust workspace + CI/CD
- Zero-copy io_uring kernel
- HTTP/1.1 + HTTP/3 server
- Type-safe router
- Structured logging

## Phase 2 — Database 🔄 IN PROGRESS
- MemTable (lock-free skip-list) ✅
- SSTable writer + reader ✅
- WAL for durability ✅
- Bloom filters ✅
- Compaction (L0-L7) ✅
- ACID transactions (partial)
- Query planner (planned)

## Phase 3 — Frontend Engine 📅 NEXT
- Hypertext renderer (Rust → HTML)
- Island hydration runtime (3KB WASM)
- Streaming SSR + Suspense
- Edge cache with SWR
- Hot-reload dev server ✅ (in nawa-cli)

## Phase 4 — Security & Ops 📅 PLANNED
- Zero-trust auth (JWT + sessions) ✅ (ACME)
- Built-in WAF + rate limiting
- Auto TLS (Let's Encrypt) ✅ (ACME client)
- Self-healing + auto-restart
- Backup + restore pipeline
- Prometheus metrics ✅

## Phase 5 — Launch 📅 PLANNED
- Stable v1.0
- WASM plugin marketplace ✅ (sandbox ready)
- App templates ✅ (6 templates)
- Documentation site ✅ (mdBook)
- CLI tool ✅ (nawa-cli)

## Completed Milestones

| Date | Milestone |
|------|-----------|
| 2026-04-01 | First commit |
| 2026-05-01 | HTTP/3 + QUIC server |
| 2026-05-15 | Zero-copy kernel (io_uring + mmap) |
| 2026-06-10 | NAWA-DB prototype |
| 2026-07-01 | WASM sandbox |
| 2026-07-05 | v0.1.0-alpha public release |
| 2026-07-06 | nawa-uring + SQPOLL + sendfile |
| 2026-07-06 | nawa CLI + Prometheus metrics |
