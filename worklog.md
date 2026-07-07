---
Task ID: sveltekit-aion-integration
Agent: main
Task: دمج SvelteKit في النواة + تنفيذ AION Engine بالكامل

Work Log:

## Part 1: SvelteKit Integration (إكمال)
- إنشاء `nawa-svelte` crate (3 modules: manifest, renderer, handler) — 25 tests
- إنشاء `adapter-nawa` npm package (index.js + index.cjs + README.md)
- إنشاء SvelteKit demo app (`examples/svelte-demo/_nawa/`) مع manifest + 7 routes
- دمج `--svelte-dir` CLI option في nawad
- إضافة routes: `/svelte` (root), `/svelte/_info` (discovery), `/svelte/**` (catch-all)
- إصلاح PathPattern في nawa-http: إضافة CatchAll variant + 3-pass matching
- إصلاح SvelteHandler::load لقبول `_nawa/` dir مباشرة
- إصلاح SvelteRenderer لاستخدام `pages/` و `assets/` بدون `_nawa/` prefix
- اختبار E2E كامل: 8/8 tests pass (dashboard, prerendered, SPA shell, auth redirect, assets)

## Part 2: AION Engine Implementation
- إنشاء `nawa-aion` crate (3 modules: ontology, negotiation, photon) — 38 tests
- **Ontological Engine**: يستنتج أنواع الكيانات من بنية DB (Person, Article, Product, Event, etc.)
- **Knowledge Graph**: يبني entities + relationships تلقائياً من DB scan
- **Adaptive Negotiation**: 9 صيغ استجابة (HTML+JSON-LD, Markdown+JSON-LD, OG, Twitter, RSS, Atom, JSON-LD, JSON, Markdown)
- **Photon Protocol** (`/__photon__`): endpoint واحد يُرجع كامل Knowledge Graph + crawl hints
- **Dynamic sitemap.xml**: يُولّد من DB لحظياً
- **Dynamic robots.txt**: مع AI crawler allowlist (GPTBot, ClaudeBot, PerplexityBot)
- **AION stats** (`/aion/stats`): إحصائيات الـ engine
- دمج AION مع nawad router (4 endpoints جديدة)
- اختبار E2E: 10/10 tests pass (5 entities مكتشفة، 1 relationship، Photon protocol يعمل)

## Architecture Summary
- 13 crates في الـ workspace (أضفنا nawa-svelte + nawa-aion)
- 158+ اختبار وحدة ناجح
- 0 warnings، 0 errors في entire workspace
- 40+ endpoints في nawad (34 + 3 SvelteKit + 4 AION)
- لا polling في أي مكان — كل شيء event-driven
- Binary واحد خالص، لا Node.js في الإنتاج

Stage Summary:
- **SvelteKit مُدمج بالكامل**: apps تُجمَّع في `_nawa/` directory وتُخدم من Rust binary
- **AION Engine مُنفّذ**: Knowledge Graph حي + Adaptive rendering + Photon Protocol
- **9 صيغ استجابة**: كل crawler يحصل على الصيغة المثلى له
- **AI crawler support**: GPTBot/ClaudeBot يحصلون على Markdown نقي
- الملفات المُنتجة:
  * `crates/nawa-svelte/` (manifest, renderer, handler — 25 tests)
  * `crates/nawa-aion/` (ontology, negotiation, photon — 38 tests)
  * `integrations/sveltekit-adapter/` (npm package)
  * `examples/svelte-demo/_nawa/` (demo app)
  * `scripts/svelte_test.sh`, `scripts/aion_test.sh`
- إجمالي الاختبارات: 158+ ناجح
