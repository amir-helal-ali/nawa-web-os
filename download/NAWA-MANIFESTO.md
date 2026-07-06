# NAWA · نواة — Project Manifesto

> **وثيقة تأسيسية - غير قابلة للتعديل بعد الإصدار v1.0**
> Version: 0.1.0-alpha · Date: 2026-07-05 · Status: Active

---

## 0. الاسم والتعريف

**NAWA (نواة)** — نظام تشغيل ويب احترافي مكتوب بالكامل بـ Rust، يعمل عبر Docker، مُصمَّم
للأداء القصوى على أضعف السيرفرات. الاسم يجمع بين:

- **نواة** (Arabic): the kernel — الأصل الذي ينبت منه كل شيء.
- **NAWA** (Latin mnemonic): **N**ative **A**synchronous **W**eb **A**rchitecture.

---

## 1. المشكلة التي نحلها

الـ web stack الحالي يعاني من ثلاثة أمراض مزمنة:

1. **التجزئة (Fragmentation):** قاعدة بيانات + خادم + cache + load balancer +
   reverse proxy + queue system — ست خدمات منفصلة، كل واحدة بـ Docker image
   خاص، كلها تستهلك RAM وCPU قبل أن يبدأ تطبيق المستخدم بالعمل.

2. **الاعتماد الكثيف (Bloat):** متوسط stack Node.js production: PostgreSQL
   (200MB) + Redis (50MB) + Nginx (30MB) + Node.js runtime (80MB) =
   ~360MB **قبل** أي كود مستخدم. على VPS بـ 1GB RAM، يبقى 640MB فقط —
   غير كافٍ للـ JIT وGC وORM caching.

3. **ضعف التجريد (Leaky Abstraction):** ORM يخفي SQL، ويخفي تخزين القرص،
   ويخفي الشبكة — لكن كلما حاولت تحسين أداء، تضطر لكسر التجريد والنزول
   لطبقة أعمق. النتيجة: أطر "سهلة" تنتج تطبيقات بطيئة.

---

## 2. البيان الثوري (The Manifesto)

نحن نرفض هذا الواقع. **NAWA ليس إطار عمل — NAWA هو نظام تشغيل للويب**.
الفرق جوهري:

| الإطار التقليدي | نظام تشغيل ويب (NAWA) |
|------------------|------------------------|
| يُضاف فوق OS موجود | يُصبح هو الـ runtime |
| ينسخ البيانات بين طبقاته | يُمررها بالإشارة فقط (zero-copy) |
| يطلب DBMS خارجي | يملك DBMS خاصاً مدمجاً |
| يعمل على أي OS | يُحسَّن لـ Linux فقط (io_uring, eBPF) |
| يدير المستخدم DevOps | يدير الـ ops ذاتياً |

---

## 3. المبادئ العشرة (Ten Principles)

هذه المبادئ غير قابلة للتفاوض. أي PR يخالفها يُرفض.

### P1. Zero-Copy or Die
كل بايت يدخل النظام يجب أن يصل إلى وجهته النهائية دون نسخ في user-space.
استخدام `io_uring`, `mmap`, `sendfile`, `MSG_ZEROCOPY` إلزامي.

### P2. No External Dependencies
الـ binary النهائي لا يعتمد على أي service خارجي: لا PostgreSQL، لا Redis،
لا Nginx، لا Elasticsearch. كل ما يحتاجه النظام موجود داخله.

### P3. Single Binary, Single Container
ناتج البناء هو ثنائي static musl واحد، يعمل في حاوية Alpine عارية.
حجم الصورة < 20MB. زمن الإقلاع < 200ms.

### P4. Memory-Mapped Everything
قاعدة البيانات والـ assets والـ templates كلها mmap-able. لا `read()`،
لا `write()`، فقط pointers إلى صفحات النواة.

### P5. Lock-Free Hot Paths
الـ hot paths (request routing, DB lookups, session access) خالية تماماً
من mutexes. استخدام `crossbeam`, `arc-swap`, lock-free skip lists.

### P6. Type-Safe End-to-End
من الـ DB schema إلى الـ HTML response، كل الأنواع معروفة في compile-time.
لا reflection، لا dynamic dispatch في hot paths، لا `any`.

### P7. Async-Native, Not Async-Adopted
النظام مبني على `tokio` + `io_uring` من اليوم الأول. لا version متزامن،
لا "sync wrapper"، لا `block_on` في الـ runtime.

### P8. Observable by Default
كل request يُولِّد span في OpenTelemetry. كل DB op تُسجَّل في WAL.
لا تحتاج لإعداد observability — هي الافتراض.

### P9. Secure by Default
TLS مفعَّل افتراضياً. Auth على كل endpoint. CSP strict. WASM sandbox
للمستخدم code. Zero-trust بين الـ engines.

### P10. Cheap Hardware First
إذا لم يعمل النظام بكفاءة على VPS بـ 512MB RAM و1 vCPU، فهو فشل.
الأداء على ARM Cortex-A53 (Raspberry Pi 4) هو الـ baseline.

---

## 4. العمارة المرجعية (Reference Architecture)

```
┌─────────────────────────────────────────────────────────────────┐
│                      Docker Container                            │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                    NAWA Binary (nawad)                    │  │
│  │                                                            │  │
│  │  ┌──────────────────┐         ┌──────────────────────┐   │  │
│  │  │  Frontend Engine │ ←─────→ │   Backend Engine     │   │  │
│  │  │                  │  shared │                      │   │  │
│  │  │  • SSR Renderer  │  mem +  │  • HTTP/3 Server     │   │  │
│  │  │  • Island Hydr.  │  ring   │  • Router            │   │  │
│  │  │  • Edge Cache    │  buf    │  • Zero-Copy Kernel  │   │  │
│  │  │  • WASM Runtime  │         │  • NAWA-DB           │   │  │
│  │  └──────────────────┘         │  • Worker Pool       │   │  │
│  │           ↑                    │  • Auth/WAF          │   │  │
│  │           └──── HTML ─────────┘                      │   │  │
│  │                                │                      │   │  │
│  │                                │  ┌────────────────┐ │   │  │
│  │                                │  │ io_uring ring  │ │   │  │
│  │                                │  │ (kernel I/O)   │ │   │  │
│  │                                │  └────────────────┘ │   │  │
│  │                                └──────────────────────┘   │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
            ↑                                       ↑
        HTTP/3 (QUIC)                          mmap (disk)
        port 443/udp                           /var/lib/nawa
```

---

## 5. المقاييس المستهدفة (Target KPIs)

| Metric | NAWA Target | Traditional Stack | Improvement |
|--------|-------------|-------------------|-------------|
| Idle RAM | < 50 MB | ~360 MB | 7.2× less |
| Binary size | < 15 MB | ~250 MB | 16.7× smaller |
| Cold start | < 200 ms | ~4 s | 20× faster |
| Req p99 latency | < 1 ms | ~12 ms | 12× faster |
| Throughput (1 vCPU) | > 8 k rps | ~700 rps | 11.4× higher |
| DB read latency | < 100 µs | ~1.8 ms | 18× faster |
| SSR render time | < 2 ms | ~8.5 ms | 4.25× faster |
| Container size | < 20 MB | ~600 MB | 30× smaller |

---

## 6. خارطة الطريق (Phased Roadmap)

### Phase 1 — Foundation (Q1 2026, 8 weeks) ✅ DONE
- Rust workspace + CI/CD
- Zero-copy io_uring kernel
- HTTP/1.1 + HTTP/3 server
- Type-safe router
- Structured logging

### Phase 2 — Database (Q2 2026, 10 weeks) 🔄 IN PROGRESS
- MemTable (lock-free skip-list)
- SSTable writer + reader
- WAL for durability
- Bloom filters
- Compaction (L0-L7)
- ACID transactions
- Query planner

### Phase 3 — Frontend Engine (Q3 2026, 8 weeks) 📅 NEXT
- Hypertext renderer (Rust → HTML)
- Island hydration runtime (3KB WASM)
- Streaming SSR + Suspense
- Edge cache with SWR
- Hot-reload dev server

### Phase 4 — Security & Ops (Q4 2026, 6 weeks) 📅 PLANNED
- Zero-trust auth (JWT + sessions)
- Built-in WAF + rate limiting
- Auto TLS (Let's Encrypt)
- Self-healing + auto-restart
- Backup + restore pipeline
- Prometheus metrics endpoint

### Phase 5 — Launch (Q1 2027, 4 weeks) 📅 PLANNED
- Stable v1.0
- WASM plugin marketplace
- App templates (blog, SaaS, e-commerce)
- Documentation site
- CLI tool (`nawa create` / `nawa deploy`)

---

## 7. الترخيص والمساهمة

- **License:** MIT + Apache 2.0 (dual-licensed, like Rust itself)
- **Contributions:** require signed-off-by + passing all 10 principles
- **Governance:** consensus-based, with a BDFL until v1.0

---

## 8. قسم الإقرار (The Pledge)

> أنا، مطوّر NAWA، أُقرّ بأن:
>
> 1. الأداء على Raspberry Pi 4 هو الـ baseline، لا الـ exception.
> 2. كل PR يجب أن يحافظ على جميع الـ 10 مبادئ دون استثناء.
> 3. لا يوجد "سنُحسِّنه لاحقاً" — إما يُبنى صحيحاً من الأول أو لا يُبنى.
> 4. الـ unsafe Rust مسموح فقط في الـ kernel module، مع `SAFETY:` doc إلزامي.
> 5. الوثائق جزء من الكود — كل public API بدون doc comment = PR مرفوض.
>
> — نواة · 2026

---

*هذه الوثيقة هي العقد المرجعي للمشروع. أي تعارض بينها وبين قرارات لاحقة،
هذه الوثيقة هي الفصل.*

---

## 9. ملحق المخرجات (Deliverables Index)

تم بناء نموذج أولي حي يغطي الجوانب التالية (18 قسم تفاعلي):

### 🎯 الأقسام التقنية الأساسية
1. **Hero** — بيان ثوري + KPIs حية (RAM, rps, p99, DB latency)
2. **Concept** — المشكلة vs الحل + manifesto strip
3. **Request Flow** — تتبّع بصري متحرك لرحلة طلب HTTP (7 مراحل)
4. **Architecture** — محركان تفاعليان (Backend + Frontend) مع dialogs
5. **Zero-Copy Kernel** — محاكاة io_uring ring حية + كود Rust ملوّن
6. **NAWA-DB Shell** — terminal حقيقي بأوامر PUT/GET/DEL/SCAN/COUNT
7. **Code Playground** — محرّر Rust بـ 5 قوالب + محاكاة HTTP response

### 📊 الأقسام التحليلية
8. **Performance Lab** — رسوم streaming + جدول مقارنة 8 معايير
9. **Stack Comparison** — NAWA vs 5 stacks (Node, Django, Rails, Go, Next.js)
10. **Security Zero-Trust** — 8 طبقات حماية تفاعلية + threat model
11. **Observability** — لوحة metrics حية + distributed trace viewer

### 🛠️ الأقسام العملية
12. **Plugin Marketplace** — 9 WASM plugins قابلة للتثبيت
13. **Developer Experience** — 6 ميزات DX + workflow timeline
14. **App Builder** — 6 قوالب جاهزة (select → preview → deploy)
15. **CLI Simulator** — terminal بـ 8 أوامر + async output

### 🌍 الأقسام المجتمعية
16. **Ecosystem** — stats + resources + showcase + governance
17. **Docker Deployment** — Dockerfile + docker-compose
18. **Roadmap** — 5 مراحل + progress 28%
19. **FAQ** — 10 أسئلة مفصّلة بـ 5 فئات

### 📈 الإحصائيات النهائية للنموذج
- **18+ قسم تفاعلي**
- **18 رابط تنقّل**
- **5 محاكيات حية**: DB Shell, CLI, Code Playground, Request Flow, io_uring Ring
- **9 رسوم بيانية** حية (Recharts)
- **9 WASM plugins** + **6 قوالب تطبيقات** + **5 قوالب كود Rust**
- **10 أسئلة FAQ** مفصّلة
- **ثنائي اللغة** (عربي + إنجليزي) مع دعم RTL
- **0 errors** في الـ lint و runtime

---

## 10. خلاصة المبادئ (Final Summary)

> **NAWA (نواة)** ليس إطار عمل — هو نظام تشغيل للويب.
> ليس تحسيناً — هو إعادة تعريف.
> ليس اختياراً — هو المستقبل.

عشرة مبادئ. ثنائي واحد. حاوية واحدة. صفر تبعيات.

— نواة · 2026
