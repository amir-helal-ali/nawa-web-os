# مشروع NAWA — {{PROJECT_NAME}}

مشروع ويب مبني على NAWA Web Operating System.

## 🚀 التشغيل السريع

```bash
# تشغيل الخادم
nawad serve

# أو بأمر nawa الموحد
nawa serve
```

ثم افتح المتصفح على: http://localhost:8080

## 🌐 الميزات المتاحة

| الميزة | الرابط | الوصف |
|--------|--------|-------|
| Dashboard | http://localhost:8080 | لوحة التحكم الرئيسية |
| WebSocket | ws://localhost:8081 | إشعارات لحظية (لا polling) |
| AION SEO | http://localhost:8080/__photon__ | Knowledge Graph + SEO |
| Sitemap | http://localhost:8080/sitemap.xml | خريطة الموقع الديناميكية |
| Robots | http://localhost:8080/robots.txt | ملف robots مع AI crawlers |
| WASM SSR | POST http://localhost:8080/api/wasm-ssr | Server-Side Rendering بـ WASM |
| SvelteKit | http://localhost:8080/svelte/ | تطبيق SvelteKit مُدمج |
| Metrics | http://localhost:8080/metrics | Prometheus metrics |
| Health | http://localhost:8080/health | فحص الصحة |

## 📁 هيكل المشروع

```
{{PROJECT_NAME}}/
├── nawa.toml          # إعدادات النظام
├── data/              # بيانات NAWA-DB (تُنشأ تلقائياً)
├── plugins/           # إضافات WASM
│   └── nawa_ssr_demo.wasm  # مثال SSR module
└── static/            # ملفات ثابتة (CSS, JS, صور)
```

## 🔧 الأوامر

```bash
nawad serve                    # تشغيل الخادم
nawad serve --addr 0.0.0.0:3000  # منفذ مخصص
nawad info                     # معلومات النظام
nawad benchmark                # قياس الأداء
nawad init                     # توليد ملف nawa.toml
```

## 🛡️ الأمان

- أول مستخدم يُسجّل يصبح **admin** تلقائياً
- JWT tokens مع RBAC (admin/user)
- Rate limiting (100 req/min افتراضياً)
- Security headers مُفعّلة
- WASM sandbox (لا وصول للملفات)

## 📊 الأداء

- DB GET: 4.3M ops/sec
- DB PUT: 714K ops/sec
- HTTP dispatch: 2.6M calls/sec
- WebSocket: push فوري (لا polling)
- Binary: 9.4MB فقط

## 🆘 المساعدة

```bash
nawa help          # المساعدة الكاملة
nawad --help       # خيارات nawad
```

---

**NAWA Web Operating System** — Binary واحد خالص، بدون Node.js.
