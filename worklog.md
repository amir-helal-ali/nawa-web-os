---
Task ID: all-in-one-bundle
Agent: main
Task: تجميع النظام بالكامل ليكون جاهزاً لبناء أول مشروع بأمر واحد

Work Log:

## 1. سكربت التثبيت الشامل (install.sh) ✓
- `scripts/install.sh` — سكربت bash شامل يُثبّت النظام بأمر واحد
- يكتشف النظام (Linux/macOS) والمعمارية (x86_64/arm64)
- يُثبّت Rust تلقائياً إذا لم يكن مثبّتاً
- يُحمّل binary من GitHub Releases أو يبني من المصدر
- يبني WASM SSR module من المصدر
- يُنشئ قوالب المشاريع
- يُضيف NAWA لـ PATH
- يُنشئ أمر `nawa` الموحد

## 2. أمر nawa الموحد ✓
- `nawa serve` — تشغيل الخادم
- `nawa new <name>` — إنشاء مشروع جديد (ينسخ القالب + WASM module)
- `nawa build-wasm <path>` — بناء WASM module
- `nawa info` — معلومات النظام
- `nawa benchmark` — قياس الأداء
- `nawa version` — الإصدار
- `nawa update` — تحديث NAWA
- `nawa help` — المساعدة

## 3. Dockerfile شامل ✓
- `Dockerfile.all-in-one` — multi-stage build
- Stage 1: بناء من المصدر (rust:1.96-slim)
- Stage 2: صورة نهائية صغيرة (debian:bookworm-slim)
- يبني nawad + nawa-cli + WASM SSR module
- `scripts/entrypoint.sh` — entrypoint يدعم serve/new/info/benchmark/help

## 4. docker-compose ✓
- `docker-compose.all-in-one.yml` — تشغيل بـ `docker-compose up`
- ports: 8080 (HTTP) + 8081 (WebSocket)
- volumes: nawa-data, nawa-plugins
- healthcheck مُدمج

## 5. قوالب المشاريع ✓
- `templates/basic/` — قالب مشروع أساسي
  - nawa.toml (إعدادات افتراضية)
  - README.md (دليل المشروع)
  - data/, plugins/, static/ (مجلدات فارغة)

## 6. دليل البدء السريع (QUICKSTART.md) ✓
- تثبيت بأمر واحد (curl | bash)
- أول مشروع في 30 ثانية
- الأوامر الأساسية
- الميزات المتاحة
- Docker usage
- تخصيص الإعدادات
- استكشاف الأخطاء

## E2E Test Results ✓
- install.sh: يُثبّت النظام بنجاح (binary + templates + nawa command)
- `nawa new test-project`: يُنشئ مشروع كامل (nawa.toml + README + plugins/nawa_ssr_demo.wasm)
- `nawad serve`: 39 routes، 6 endpoints تعمل (Dashboard, Health, Photon, Sitemap, Robots, WASM SSR)
- WASM SSR: HTTP 200, 864 bytes HTML كامل

## النتيجة النهائية
**أمر واحد يُثبّت كل شيء:**
```bash
curl -fsSL https://raw.githubusercontent.com/amir-helal-ali/nawa-web-os/main/nawa-rs/scripts/install.sh | bash
```

**ثم مشروع جديد بأمر واحد:**
```bash
nawa new my-app
cd my-app
nawad serve
```

النظام جاهز لبناء أول مشروع فوراً مع كل الميزات:
- Dashboard + Auth (أول مستخدم = admin)
- NAWA-DB (4.3M GET ops/sec)
- WebSocket (إشعارات لحظية، لا polling)
- AION SEO Engine (Knowledge Graph + 9 formats)
- WASM SSR (Rust → WASM → HTML)
- SvelteKit integration
- HTTP/3 + QUIC support
- Docker support
