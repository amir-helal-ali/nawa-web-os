# Changelog

جميع التغييرات الجوهرية في مشروع NAWA موثقة هنا.

التنسيق مبني على [Keep a Changelog](https://keepachangelog.com/ar/)،
وهذا المشروع يتبع [Semantic Versioning](https://semver.org/lang/ar/).

## [1.0.0] — 2026-07-09

### 🎉 أول إصدار مستقر

**نظام تشغيل الويب الثوري NAWA — جاهز للإنتاج**

### ✨ مضاف

#### المعمارية الأساسية
- **nawa-kernel**: io_uring + mmap abstractions (7 اختبارات)
- **nawa-db**: قاعدة بيانات مدمجة (LSM + WAL + Bloom + SkipList) (28 اختبار)
- **nawa-http**: خادم HTTP/1.1 + HTTP/3 + type-safe router (12 اختبار)
- **nawa-uring**: io_uring bindings حقيقية على Linux 5.1+ (26 اختبار)
- **nawa-wasm**: WASM sandbox + SSR rendering (13 اختبار)
- **nawa-auth**: JWT + RBAC + password hashing (27 اختبار)
- **nawa-engine**: ZeroCopyHtml + UnifiedEngine (27 اختبار)
- **nawa-frontend**: SSR + islands + streaming (26 اختبار)

#### AION SEO Engine
- **nawa-aion**: Knowledge Graph تلقائي + 9 صيغ استجابة (59 اختبار)
  - Ontological Inference Engine
  - Adaptive Negotiation Protocol
  - Photon Protocol endpoint (`/__photon__`)
  - Self-Healing SEO Loop
  - Google Search Console API (RSA JWT signing)
  - Dynamic sitemap.xml + robots.txt

#### SvelteKit Integration
- **nawa-svelte**: دمج SvelteKit بدون Node.js (25 اختبار)
  - adapter-nawa (npm package)
  - manifest.json parser
  - SPA shell + pre-rendered HTML
  - NAWA bootstrap injection

#### الاستقرار والإنتاج
- **Connection Pool** لتتبع اتصالات WebSocket
- **Health Checker** لمراقبة صحة النظام
- **Retry Logic** مع exponential backoff
- **Graceful Shutdown** مع drain للطلبات النشطة
- **Audit Logging** لكل الأحداث الأمنية
- **CSRF Protection** (constant-time validation)
- **11 Security Headers** (CSP, HSTS, COOP, CORP, etc.)

#### WASM SSR
- مثال كامل بـ Rust → WASM → HTML (74KB module)
- `POST /api/wasm-ssr` endpoint
- wasmtime sandbox مع fuel limits

#### نظام التصميم
- 3 ثيمات (Dark/Light/Auto) مع localStorage
- Glassmorphism + gradient text
- Animations (ripple, fade, pulse, glow, spin)
- RTL أصلي + responsive
- Print-friendly styles

#### التوزيع
- `install.sh` — تثبيت بأمر واحد
- `nawa` unified command (serve, new, build-wasm, info, benchmark)
- Docker image (multi-stage build)
- docker-compose
- Project templates (basic)
- QUICKSTART.md

### 🔒 الأمان
- JWT (HS256) + RBAC (admin/user)
- CSRF protection (constant-time)
- Audit logging
- Rate limiting
- WASM sandbox
- 11 security headers
- Password hashing (SHA-256 + salt)

### ⚡ الأداء
- DB GET: 4,310,556 ops/sec
- DB PUT: 713,872 ops/sec
- DB SCAN: 8,658,087 ops/sec
- HTTP dispatch: 2,592,548 calls/sec
- AION negotiation: 17,873,226 calls/sec
- JWT verification: 1,285,373 calls/sec

### 📊 الإحصائيات
- **46 HTTP endpoint** (was 44)
- **250+ اختبار وحدة** ناجح
- **0 تحذيرات** في البناء و clippy
- **Binary**: 9.7 ميجابايت
- **WASM module**: 74 كيلوبايت
- **12 crate** متكامل

### 🐛 إصلاحات
- إصلاح RFC 6455 WebSocket compliance (SHA-1 بدلاً من SHA-256)
- إصلاح 64-bit payload length parsing
- إصلاح h3-quinn 0.0.10 API compatibility
- إصلاح Router catch-all pattern matching
- إصلاح WASM module loading (reference_types)

### 🔄 مُغيّر
- ترقية h3 من 0.0.6 إلى 0.0.8
- ترقية h3-quinn من 0.0.7 إلى 0.0.10
- إضافة sha2 oid feature لـ RSA signing
- إضافة rsa + pkcs8 crates

### ❌ محذوف
- إزالة Dockerfile القديم (استبدال بـ all-in-one)
- إزالة docker-compose القديم
- إزالة svelte-demo (استبدال بـ svelte-app حقيقي)
- إزالة RELEASE_NOTES القديمة

---

## [1.0.0-alpha] — 2026-07-08

أول إصدار تجريبي من نظام تشغيل الويب الثوري NAWA.

### ✨ مضاف
- 12 crate متكامل
- Binary واحد خالص (9.7 ميجابايت)
- AION SEO Engine
- WASM SSR
- SvelteKit integration
- HTTP/3 + QUIC support
- Docker image
- install.sh

---

## الروابط

- [GitHub](https://github.com/amir-helal-ali/nawa-web-os)
- [التوثيق](https://github.com/amir-helal-ali/nawa-web-os/tree/main/nawa-rs/docs)
- [الإصدارات](https://github.com/amir-helal-ali/nawa-web-os/releases)
