# NAWA v0.1.0-alpha.2

**Revolutionary Web Operating System — built in pure Rust.**

Binary واحد خالص، يعمل على 512MB RAM، بدون أي تبعيات خارجية في الإنتاج.

## 🆕 ما الجديد منذ v0.1.0-alpha

### 1. SvelteKit Integration (`nawa-svelte` crate)
- دمج كامل لتطبيقات SvelteKit في binary الـ Rust بدون Node.js في الإنتاج
- `adapter-nawa` npm package لتجميع SvelteKit إلى `_nawa/` directory
- دعم routes: `[id]`, `[...slug]`, `[[optional]]`, catch-all `**`
- `nawad --svelte-dir ./_nawa` لتفعيل الـ integration
- 25 اختبار وحدة ✓

### 2. AION SEO Engine (`nawa-aion` crate)
**Adaptive Intelligent Ontological Network** — ثورة في SEO:

- **Ontological Engine**: يستنتج أنواع الكيانات (Person, Article, Product, Event, etc.) تلقائياً من بنية DB
- **Knowledge Graph**: يبني entities + relationships تلقائياً
- **Adaptive Negotiation**: 9 صيغ استجابة — كل crawler يحصل على الأنسب له:
  - Googlebot/Bingbot → HTML + JSON-LD
  - GPTBot/ClaudeBot → Markdown + JSON-LD (cleaner for AI training)
  - Facebookbot → HTML + Open Graph
  - Twitterbot → HTML + Twitter Cards
  - RSS readers → RSS XML
  - Atom readers → Atom XML
- **Photon Protocol** (`GET /__photon__`): endpoint واحد يُرجع كامل Knowledge Graph + crawl hints
- **Dynamic sitemap.xml** و **robots.txt** مع AI crawler allowlist
- **Self-Healing Loop**: يكتشف ويُصلح SEO issues تلقائياً (404s, duplicate content, missing structured data, missing canonical)
- 49 اختبار وحدة ✓

### 3. nawa-http Router Enhancements
- إضافة `**` catch-all pattern segment
- 3-pass matching: literals → params → catch-alls (يضمن أولوية صحيحة)
- إصلاح تضارب `/:key` مع `/svelte/**`

### 4. RFC 6455 WebSocket Compliance
- استبدال SHA-256 بـ SHA-1 في الـ handshake (المتصفحات ترفض SHA-256)
- إصلاح 64-bit payload length parsing
- إضافة heartbeat 30s (pure async, لا polling)
- إضافة OpCode enum كامل (Text/Binary/Close/Ping/Pong)
- مُتحقق بـ RFC 6455 test vector الرسمي

### 5. Performance Benchmarks (`benches/`)
نتائج benchmarks على Linux x86_64:

| العملية | الأداء |
|---------|--------|
| DB PUT | 713,872 ops/sec |
| DB GET | 4,310,556 ops/sec |
| DB SCAN | 8,658,087 ops/sec |
| DB DELETE | 1,002,486 ops/sec |
| Engine render | 951,352 pages/sec |
| AION Knowledge Graph (1500 entities) | 1.79 ms/build |
| AION Photon response | 0.99 ms/build |
| AION Negotiation | 17,873,226 calls/sec |
| JWT verification | 1,285,373 calls/sec |
| Svelte route matching | 5,701,764 calls/sec |
| HTTP router dispatch | 2,592,548 calls/sec |

### 6. Real SvelteKit App (Vite build)
- `examples/svelte-app/` — تطبيق Svelte 5 حقيقي مُجمّع بـ Vite
- `npm run build` يُنتج `_nawa/` directory جاهز لـ nawad
- 28KB JS فقط (11KB gzipped) — أصغر bundle في فئته

### 7. Comprehensive Integration Audit
- **0 warnings, 0 errors** في `cargo build --release`
- **0 warnings, 0 errors** في `cargo clippy --release --workspace`
- **241 اختبار وحدة ناجح** عبر 10 crates
- **35/35 E2E integration checks ناجح** (Auth + DB + WebSocket + Svelte + AION)
- **لا polling** — 100% event-driven
- **Binary 9.4MB** — لا Node.js في الـ runtime

## 📦 الأنواع (Crates)

| Crate | الوصف | الاختبارات |
|-------|-------|-----------|
| nawa-kernel | io_uring + mmap abstractions | 8 |
| nawa-db | KV/Document DB (LSM+WAL+Bloom) | 28 |
| nawa-http | HTTP/1.1 + type-safe router | 13 |
| nawa-uring | Real io_uring (Linux 5.1+) | 28 |
| nawa-wasm | WASM sandbox (wasmtime) | 8 |
| nawa-auth | JWT + RBAC + password hashing | 27 |
| nawa-frontend | SSR + islands + streaming | 9 |
| nawa-engine | ZeroCopyHtml + UnifiedEngine | 27 |
| nawa-svelte | SvelteKit integration | 25 |
| nawa-aion | AION SEO Engine | 49 |
| nawad | Binary integration | 19 (E2E) |
| nawa-cli | CLI tool | — |

## 🚀 التثبيت

### من المصدر
```bash
git clone https://github.com/amir-helal-ali/nawa-web-os.git
cd nawa-web-os/nawa-rs
cargo build --release
./target/release/nawad serve
```

### تشغيل مع SvelteKit
```bash
# ابنِ تطبيق SvelteKit
cd examples/svelte-app && npm install && npm run build

# شغّل nawad مع SvelteKit
../../target/release/nawad serve --svelte-dir ./_nawa
```

## 🌐 Endpoints (42 إجمالاً)

- Dashboard: `GET /`
- Auth: `POST /register`, `POST /login`, `GET /logout`, `GET /auth/me`, `GET /auth/users`
- DB API: `GET /:key`, `POST /:key`, `DELETE /:key`, `GET /scan/:prefix`
- System: `GET /system`, `GET /health`, `GET /uring`, `GET /metrics`, `GET /plugins`
- Real-time: `GET /notifications/stats` + WebSocket on port+1
- SvelteKit: `GET /svelte/_info`, `GET /svelte/**`
- AION SEO: `GET /__photon__`, `GET /sitemap.xml`, `GET /robots.txt`, `GET /aion/stats`, `POST /aion/heal`
- Admin: `GET /settings`, `POST /settings`, `POST /admin/verify`, `POST /admin/role`, `POST /admin/delete`
- Backup: `GET /backup`, `POST /restore`
- Static: `GET /static/:path`

## 📊 الإحصائيات

- **Lines of Rust**: ~15,000+
- **Lines of JS/Svelte**: ~200
- **Crates**: 12 (10 lib + 2 bin)
- **Tests**: 241 unit + 35 E2E
- **Binary size**: 9.4MB (nawad)
- **RAM usage**: ~5MB idle
- **Dependencies**: 0 runtime (Node.js غير مطلوب)

## 🎯 ما يجب القيام به لاحقاً

- [ ] WASM-based SSR (لتقديم محتوى ديناميكي بدون Node.js)
- [ ] Predictive Caching في AION
- [ ] تكامل Google Search Console API حقيقي
- [ ] HTTP/3 + QUIC support
- [ ] TLS مدمج

## 📄 الترخيص

MIT OR Apache-2.0

## 🔗 الروابط

- Repository: https://github.com/amir-helal-ali/nawa-web-os
- Documentation: https://github.com/amir-helal-ali/nawa-web-os/tree/main/nawa-rs/docs
- Examples: https://github.com/amir-helal-ali/nawa-web-os/tree/main/nawa-rs/examples
