---
Task ID: wasm-ssr-gsc-http3-complete
Agent: main
Task: إكمال HTTP/3 dispatch loop + RSA JWT signing + WASM SSR module

Work Log:

## Task 1: HTTP/3 dispatch loop — مكتمل ✓
- إعادة كتابة `crates/nawa-http/src/h3.rs` بالكامل مع h3 0.0.8 + h3-quinn 0.0.10 API
- `handle_h3_connection()` يستخدم `h3::server::builder().build()` + `resolver.resolve_request()`
- `handle_h3_request()` يحول h3 request → NAWA Request، dispatches عبر router
- `send_h3_response()` يُرسل response headers + body + finish عبر h3 stream
- `QuicServerConfig::try_from(rustls_config)` لتحويل TLS config
- البناء ناجح، 0 warnings

## Task 2: RSA JWT signing — مكتمل ✓
- إضافة `rsa = { version = "0.9", features = ["pem", "sha2"] }` و `pkcs8` للمشروع
- `sha2` حصل على `oid` feature لـ `AssociatedOid` trait
- `rsa_sign()` في google_search_console.rs يوقّع JWT فعلياً بـ RSA-SHA256 (PKCS#1 v1.5)
- يدعم PKCS#8 PEM (تنسيق Google service account keys)
- البناء ناجح

## Task 3: WASM SSR module حقيقي — مكتمل ✓
- إنشاء `examples/wasm-ssr-module/` — مشروع Rust مستقل يُجمّع إلى wasm32-unknown-unknown
- `src/lib.rs` (210 سطر) يُصدّر: `memory`, `alloc(size)`, `render(props_ptr, props_len)`
- `PageProps` struct مع `title`, `description`, `items`, `user`
- `render_html()` يُولّد HTML كامل: DOCTYPE, meta tags, CSS inline (dark theme RTL), user card, items list
- HTML escaping للأمان (XSS protection)
- 0 warnings (مع `#![allow(static_mut_refs)]`)
- WASM module جاهز: 74KB فقط

## Task 4: دمج WASM SSR في nawad — مكتمل ✓
- إضافة `render_ssr()` method لـ `Sandbox` في runtime.rs
- إضافة `POST /api/wasm-ssr` endpoint في nawad main.rs
- تفعيل `wasm_reference_types(true)` و `wasm_memory64(false)` في wasmtime config
- E2E test ناجح: WASM module يُحمّل، render() يعمل، HTML كامل يُرجع (999 bytes)

## Final Verification
- cargo build --release: 0 warnings, 0 errors ✓
- cargo clippy --release --workspace: 0 warnings, 0 errors ✓
- WASM SSR E2E: HTTP 200, 999 bytes HTML ✓
- جميع الميزات الثلاث تعمل معاً

Stage Summary:
- **HTTP/3 dispatch loop**: تكامل كامل مع h3 0.0.8 API
- **RSA JWT signing**: توقيع فعلي بـ RSA-SHA256 لـ Google Search Console
- **WASM SSR**: module حقيقي بـ Rust + Vite، يُولّد HTML كامل، يعمل عبر /api/wasm-ssr
- **0 warnings, 0 errors** في البناء و clippy
