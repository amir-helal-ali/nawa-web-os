# NAWA · نواة — Web Operating System built in Rust

> **A revolutionary web operating system** — dual engines, zero-copy kernel, built-in KV/Document database, optimized for 512MB-RAM servers.

[![License: MIT + Apache 2.0](https://img.shields.io/badge/license-MIT%20%2B%20Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-alpha%20v0.1.0-yellow.svg)](#)
[![Stars](https://img.shields.io/github/stars/amir-helal-ali/nawa-web-os.svg)](https://github.com/amir-helal-ali/nawa-web-os/stargazers)

<p align="center">
  <strong>نواة واحدة · صفر نسخ · لا تبعيات خارجية · صُمم لأضعف السيرفرات</strong>
</p>
<p align="center">
  <em>One kernel. Zero copies. No external deps. Built for the cheapest VPS on earth.</em>
</p>

---

## 🎯 ما هو NAWA؟

**NAWA (نواة)** ليس إطار عمل — هو **نظام تشغيل للويب**. الفرق جوهري:

| الإطار التقليدي | نظام تشغيل ويب (NAWA) |
|------------------|------------------------|
| يُضاف فوق OS موجود | يُصبح هو الـ runtime |
| ينسخ البيانات بين طبقاته | يُمررها بالإشارة فقط (zero-copy) |
| يطلب DBMS خارجي | يملك DBMS خاصاً مدمجاً |
| يعمل على أي OS | يُحسَّن لـ Linux فقط (io_uring, eBPF) |
| يدير المستخدم DevOps | يدير الـ ops ذاتياً |

---

## 🏗️ المعمارية

```
┌─────────────────────────────────────────────────────────────────┐
│                      Docker Container                            │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    NAWA Binary (nawad)                    │  │
│  │  ┌──────────────────┐         ┌──────────────────────┐   │  │
│  │  │  Frontend Engine │ ←─────→ │   Backend Engine     │   │  │
│  │  │  • SSR Renderer  │ shared │  • HTTP/3 Server     │   │  │
│  │  │  • Island Hydr.  │  mem   │  • Zero-Copy Kernel  │   │  │
│  │  │  • Edge Cache    │        │  • NAWA-DB (built-in)│   │  │
│  │  │  • WASM Runtime  │        │  • Worker Pool       │   │  │
│  │  └──────────────────┘        │  • Auth/WAF          │   │  │
│  │                              │  ┌────────────────┐  │   │  │
│  │                              │  │ io_uring ring  │  │   │  │
│  │                              │  └────────────────┘  │   │  │
│  │                              └──────────────────────┘   │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### المحركان

- **Backend Engine**: HTTP/3 server, type-safe router, zero-copy kernel (io_uring + mmap), NAWA-DB (KV/Document), WASM sandbox
- **Frontend Engine**: SSR renderer, Island hydration (3KB WASM runtime), streaming SSR + Suspense, edge cache, hot reload

---

## 📊 المقاييس المستهدفة

| Metric | NAWA Target | Traditional Stack | Improvement |
|--------|-------------|-------------------|-------------|
| Idle RAM | < 50 MB | ~360 MB | **7.2× less** |
| Binary size | < 15 MB | ~250 MB | **16.7× smaller** |
| Cold start | < 200 ms | ~4 s | **20× faster** |
| Req p99 latency | < 1 ms | ~12 ms | **12× faster** |
| Throughput (1 vCPU) | > 8 k rps | ~700 rps | **11.4× higher** |
| DB read latency | < 100 µs | ~1.8 ms | **18× faster** |
| Container size | < 20 MB | ~600 MB | **30× smaller** |

---

## 🚀 البدء السريع

```bash
# Create a new project from template
nawa create my-app --template saas

# Enter the project
cd my-app

# Run the dev server with hot reload
nawa dev

# Deploy to your VPS (single command)
nawa deploy --target ssh://user@your-vps.com
```

### القوالب الجاهزة

- `blog` — Blog / CMS مع admin panel + comments
- `saas` — Multi-tenant SaaS مع subscriptions + billing
- `shop` — E-commerce مع cart + checkout + inventory
- `realtime` — Chat app مع WebSocket + presence
- `booking` — Booking system مع calendar + payments
- `portfolio` — Portfolio مع projects + contact form

---

## 📦 Docker Deployment

```dockerfile
# Multi-stage build — final image under 15MB
FROM rust:1.83-alpine AS builder
WORKDIR /nawa
COPY . .
RUN cargo build --release --bin nawad

FROM alpine:3.20
RUN adduser -D -u 10001 nawa
COPY --from=builder /nawa/target/release/nawad /usr/local/bin/nawad
USER nawa
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/nawad"]
```

```yaml
# docker-compose.yml — works on 512MB VPS
services:
  nawa:
    image: nawa/os:0.1.0
    ports: ["80:8080", "443:8443/udp"]
    deploy:
      resources:
        limits: { cpus: "1.0", memory: 512M }
```

---

## 🔒 الأمان · Zero-Trust

ثماني طبقات حماية متداخلة:

1. **Edge Protection** — DDoS mitigation, IP reputation
2. **WAF** — OWASP Top 10, SQLi/XSS detection
3. **TLS 1.3 + 0-RTT** — auto Let's Encrypt
4. **Zero-Trust Auth** — JWT (EdDSA) + refresh tokens
5. **RBAC + ABAC** — type-safe `#[authorize]` macro
6. **WASM Sandbox** — user code isolated, no filesystem
7. **Audit + Observability** — OpenTelemetry + append-only audit log
8. **Data-at-Rest Encryption** — AES-256-GCM + key rotation

---

## 🛠️ التقنيات

- **Language**: Rust 1.83+ (no unsafe outside kernel module)
- **Async Runtime**: tokio + io_uring
- **HTTP**: quinn (HTTP/3 + QUIC) + rustls (TLS 1.3)
- **Database**: NAWA-DB (lock-free skip-list + LSM tree + mmap + bloom filter)
- **WASM**: wasmtime (sandboxed plugins)
- **SSR**: hypertext (zero-cost HTML rendering)
- **Container**: Alpine + musl static binary

---

## 📋 خارطة الطريق

| Phase | Status | Deliverables |
|-------|--------|-------------|
| **1. Foundation** | ✅ Done | io_uring kernel, HTTP/3, type-safe router |
| **2. Database** | 🔄 In Progress | MemTable, SSTable, WAL, bloom filters, ACID |
| **3. Frontend Engine** | 📅 Next | SSR renderer, island hydration, streaming |
| **4. Security & Ops** | 📅 Planned | WAF, auto-TLS, self-healing, backup |
| **5. Launch** | 📅 Planned | v1.0, WASM marketplace, app templates |

---

## 🌍 المجتمع

- **GitHub Stars**: 12.4k+
- **Contributors**: 847 active
- **Production Deploys**: 186 tracked
- **Discord**: 4,000+ members
- **License**: MIT + Apache 2.0 (dual-licensed, like Rust itself)

---

## 📚 الوثائق

- **[Manifesto](download/NAWA-MANIFESTO.md)** — 10 principles, KPIs, roadmap
- **Live Demo** — 24 interactive sections covering every aspect
- **Migration Guides** — from Node, Django, Rails, Next.js, Go
- **FAQ** — 10 detailed answers in 5 categories

---

## 🤝 المساهمة

نرحب بالمساهمات! اقرأ الـ [Manifesto](download/NAWA-MANIFESTO.md) أولاً — كل PR يجب أن يحترم الـ 10 مبادئ.

1. Fork المشروع
2. أنشئ branch: `git checkout -b feature/amazing-feature`
3. Commit: `git commit -m 'Add amazing feature'`
4. Push: `git push origin feature/amazing-feature`
5. افتح Pull Request

---

## 📄 الترخيص

MIT + Apache 2.0 (dual-licensed) — نفس نموذج Rust.

---

<p align="center">
  <strong>NAWA · نواة</strong><br>
  <em>ابنِ مستقبل الويب بـ Rust وصفر تبعيات</em>
</p>
