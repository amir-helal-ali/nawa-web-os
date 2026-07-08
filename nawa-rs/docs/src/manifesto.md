# The Manifesto

> **وثيقة تأسيسية - غير قابلة للتعديل بعد الإصدار v1.0**

## The Ten Principles

### P1. Zero-Copy or Die
Every byte that enters the system must reach its final destination without being copied in user-space. `io_uring`, `mmap`, `sendfile`, `MSG_ZEROCOPY` are mandatory.

### P2. No External Dependencies
The final binary depends on no external service: no PostgreSQL, no Redis, no Nginx, no Elasticsearch. Everything the system needs is built in.

### P3. Single Binary, Single Container
The build output is one static musl binary running in a bare Alpine container. Image size < 20MB. Boot time < 200ms.

### P4. Memory-Mapped Everything
Database files, assets, and templates are all mmap-able. No `read()`, no `write()`, only pointers to kernel pages.

### P5. Lock-Free Hot Paths
Hot paths (request routing, DB lookups, session access) are completely free of mutexes. Uses `crossbeam`, `arc-swap`, lock-free skip lists.

### P6. Type-Safe End-to-End
From DB schema to HTML response, all types are known at compile-time. No reflection, no dynamic dispatch in hot paths, no `any`.

### P7. Async-Native, Not Async-Adopted
Built on `tokio` + `io_uring` from day one. No sync version, no "sync wrapper", no `block_on` in the runtime.

### P8. Observable by Default
Every request generates an OpenTelemetry span. Every DB op is logged in WAL. No observability setup needed — it's the default.

### P9. Secure by Default
TLS enabled by default. Auth on every endpoint. CSP strict. WASM sandbox for user code. Zero-trust between engines.

### P10. Cheap Hardware First
If the system doesn't run efficiently on a VPS with 512MB RAM and 1 vCPU, it's a failure. Performance on ARM Cortex-A53 (Raspberry Pi 4) is the baseline.

---

## The Pledge

> I, a NAWA developer, pledge that:
>
> 1. Performance on Raspberry Pi 4 is the baseline, not the exception.
> 2. Every PR must preserve all 10 principles without exception.
> 3. There is no "we'll optimize it later" — either it's built right from the start or it's not built.
> 4. `unsafe` Rust is only allowed in the kernel module, with `SAFETY:` doc mandatory.
> 5. Documentation is part of the code — every public API without a doc comment = rejected PR.
>
> — نواة · 2026
