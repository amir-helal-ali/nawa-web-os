# 🚀 NAWA — دليل البدء السريع

**NAWA Web Operating System** — نظام تشغيل ويب ثوري بـ Rust خالص.
Binary واحد، بدون Node.js، يعمل على 512MB RAM.

---

## ⚡ التثبيت بأمر واحد

### Linux / macOS
```bash
curl -fsSL https://raw.githubusercontent.com/amir-helal-ali/nawa-web-os/main/nawa-rs/scripts/install.sh | bash
```

### أو عبر Docker (بدون تثبيت)
```bash
docker run -p 8080:8080 -p 8081:8081 amirhelal/nawa-web-os:latest
```

### أو من المصدر
```bash
git clone https://github.com/amir-helal-ali/nawa-web-os.git
cd nawa-web-os/nawa-rs
cargo build --release
./target/release/nawad serve
```

---

## 🎯 أول مشروع في 30 ثانية

بعد التثبيت:

```bash
# 1. أنشئ مشروع جديد
nawa new my-first-app

# 2. ادخل للمشروع
cd my-first-app

# 3. شغّل الخادم
nawad serve
```

افتح المتصفح: **http://localhost:8080**

✅ **هذا كل شيء!** مشروعك يعمل مع:
- Dashboard كامل
- نظام مصادقة (أول مستخدم = admin)
- قاعدة بيانات NAWA-DB
- WebSocket للإشعارات اللحظية
- AION SEO Engine
- WASM SSR
- SvelteKit integration

---

## 📋 الأوامر الأساسية

| الأمر | الوصف |
|-------|-------|
| `nawa serve` | تشغيل الخادم |
| `nawa new <name>` | إنشاء مشروع جديد |
| `nawa info` | معلومات النظام |
| `nawa benchmark` | قياس الأداء |
| `nawa build-wasm <path>` | بناء WASM module |
| `nawa update` | تحديث NAWA |
| `nawa help` | المساعدة الكاملة |

---

## 🌐 الميزات المتاحة بعد التشغيل

### 1. Dashboard & Auth
- http://localhost:8080 — Dashboard
- http://localhost:8080/register — تسجيل أول مستخدم (يصبح admin)
- http://localhost:8080/login — تسجيل الدخول
- http://localhost:8080/settings — إعدادات النظام (admin)

### 2. قاعدة البيانات (NAWA-DB)
```bash
# كتابة
curl -X POST -d '{"name":"test"}' http://localhost:8080/my-key

# قراءة
curl http://localhost:8080/my-key

# مسح prefix
curl http://localhost:8080/scan/my
```

### 3. الإشعارات اللحظية (WebSocket)
```javascript
// في المتصفح
const ws = new WebSocket('ws://localhost:8081');
ws.onmessage = (e) => console.log('Live update:', JSON.parse(e.data));
// لا polling — كل شيء push
```

### 4. AION SEO Engine
- http://localhost:8080/__photon__ — Knowledge Graph كامل
- http://localhost:8080/sitemap.xml — خريطة الموقع
- http://localhost:8080/robots.txt — robots مع AI crawler allowlist
- http://localhost:8080/aion/stats — إحصائيات SEO

### 5. WASM SSR
```bash
curl -X POST http://localhost:8080/api/wasm-ssr \
  -H "Content-Type: application/json" \
  -d '{"title":"My Page","description":"Hello"}'
```

### 6. SvelteKit Integration
```bash
nawad serve --svelte-dir ./_nawa
# ثم افتح: http://localhost:8080/svelte/
```

---

## 🐳 Docker

### تشغيل سريع
```bash
docker run -p 8080:8080 -p 8081:8081 amirhelal/nawa-web-os:latest
```

### مع volumes للحفظ
```bash
docker run -d \
  -p 8080:8080 -p 8081:8081 \
  -v nawa-data:/opt/nawa/data \
  --name nawa \
  amirhelal/nawa-web-os:latest
```

### docker-compose
```bash
curl -o docker-compose.yml https://raw.githubusercontent.com/amir-helal-ali/nawa-web-os/main/nawa-rs/docker-compose.all-in-one.yml
docker-compose up -d
```

---

## 📊 الأداء

| العملية | الأداء |
|---------|--------|
| DB GET | 4,310,556 ops/sec |
| DB PUT | 713,872 ops/sec |
| DB SCAN | 8,658,087 ops/sec |
| HTTP dispatch | 2,592,548 calls/sec |
| JWT verification | 1,285,373 calls/sec |
| Svelte route matching | 5,701,764 calls/sec |
| AION negotiation | 17,873,226 calls/sec |

---

## 🛡️ الأمان

- ✅ **أول مستخدم = admin** تلقائياً
- ✅ JWT + RBAC (admin/user roles)
- ✅ Rate limiting قابل للتعديل
- ✅ Security headers (XSS, CSRF, clickjacking)
- ✅ WASM sandbox (عزل تام للـ plugins)
- ✅ لا password_hash leak في API responses
- ✅ Cookie-based auth (HttpOnly)

---

## 🔧 التخصيص

### تعديل الإعدادات
```bash
# عدّل nawa.toml
addr = "0.0.0.0:3000"           # المنفذ
rate_limit = 200                 # حد الطلبات
jwt_secret = "your-secret-here"  # سر JWT
```

### إضافة WASM plugin
```bash
# 1. اكتب Rust module يُصدّر: memory, alloc, render
# 2. ابنِه
nawa build-wasm ./my-module
# 3. انسخه لـ plugins/
cp target/wasm32-unknown-unknown/release/my_module.wasm plugins/
# 4. أعد تشغيل nawad
```

### تفعيل SvelteKit
```bash
# 1. ابنِ SvelteKit app
cd my-svelte-app && npm run build
# 2. شغّل nawad مع --svelte-dir
nawad serve --svelte-dir ./_nawa
```

### تفعيل HTTP/3
```bash
nawad serve --http3 --tls-cert cert.pem --tls-key key.pem
```

---

## 🆘 استكشاف الأخطاء

### المنفذ 8080 محجوز
```bash
nawad serve --addr 127.0.0.1:9090
```

### نسيت كلمة مرور admin
```bash
# احذف مجلد البيانات وأعد التشغيل (يفقد كل البيانات!)
rm -rf data/
nawad serve
```

### WASM module لا يُحمّل
```bash
# تحقق من السجلات
RUST_LOG=debug nawad serve
# تأكد أن الملف .wasm في مجلد plugins/
```

---

## 📚 المزيد

- **GitHub:** https://github.com/amir-helal-ali/nawa-web-os
- **التوثيق:** https://github.com/amir-helal-ali/nawa-web-os/tree/main/nawa-rs/docs
- **أمثلة:** https://github.com/amir-helal-ali/nawa-web-os/tree/main/nawa-rs/examples

---

**NAWA** — Revolutionary Web Operating System.
🦀 Built with Rust · 🚀 Zero Node.js · ⚡ Real-time push
