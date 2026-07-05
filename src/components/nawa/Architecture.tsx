"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Server,
  Monitor,
  Database,
  Router,
  Shield,
  Cpu,
  Zap,
  GitMerge,
  Workflow,
  X,
  ChevronLeft,
  Layers,
  Lock,
  Network,
  Cloud,
  Code2,
  Boxes,
  Activity,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type Layer = {
  id: string;
  name: string;
  nameAr: string;
  en: string;
  desc: string;
  descEn: string;
  icon: typeof Server;
  color: "primary" | "accent";
  detail: string;
  apis: string[];
};

const BACKEND_LAYERS: Layer[] = [
  {
    id: "be-quic",
    name: "HTTP/3 + QUIC Server",
    nameAr: "خادم HTTP/3",
    en: "quinn + rustls",
    desc: "خادم HTTP/3 مبني على quinn، يدعم 0-RTT، keep-alive، connection pooling تلقائي.",
    descEn: "HTTP/3 server on quinn with 0-RTT, keep-alive, auto-pooling.",
    icon: Network,
    color: "primary",
    detail:
      "يعمل مباشرة فوق UDP مع QUIC. كل connection مستقلة (لا head-of-line blocking)، والدعم المدمج لـ TLS 1.3 يعني أن كل request مشفَّر افتراضياً. 0-RTT يسمح للعميل بإرسال البيانات في أول packet دون انتظار handshake.",
    apis: ["quinn::Endpoint", "h3::server", "rustls::ServerConfig", "tokio::io"],
  },
  {
    id: "be-router",
    name: "Type-Safe Router",
    nameAr: "موجِّه آمن الأنواع",
    en: "macro-based routing",
    desc: "توجيه ثابت الأنواع بـ proc-macro — أخطاء الـ routing تُكتشف في compile-time.",
    descEn: "Compile-time type-safe routing via proc-macros.",
    icon: Router,
    color: "primary",
    detail:
      "كل route يُعرَّف بـ #[get(\"/users/:id\")]. الـ macro يستخرج :id ويولِّد handler يستقبل typed parameter. لا string parsing في runtime، لا panic على route mismatch — كل شيء في compile-time.",
    apis: ["#[get(...)]", "#[post(...)]", "Extractor<T>", "Path<T>", "Json<T>"],
  },
  {
    id: "be-kernel",
    name: "Zero-Copy Kernel",
    nameAr: "النواة صفر-النسخ",
    en: "io_uring + mmap",
    desc: "النواة الثورية — I/O غير متزامن بالكامل دون نسخ البيانات في الذاكرة.",
    descEn: "The revolutionary kernel — fully async I/O with zero memory copies.",
    icon: Cpu,
    color: "primary",
    detail:
      "بدل epoll المتسلسل، نستخدم io_uring مع SQ poll — طلبات I/O تُجمع وتُنفذ دفعة واحدة في النواة دون context switches. mmap للوصول المباشر للملفات دون user-space buffer. sendfile + MSG_ZEROCOPY لإرسال البيانات من القرص إلى الشبكة بـ syscall واحد.",
    apis: ["io_uring::IoUring", "mmap::Mmap", "sendfile(2)", "MSG_ZEROCOPY"],
  },
  {
    id: "be-db",
    name: "NAWA-DB (built-in)",
    nameAr: "قاعدة بيانات مدمجة",
    en: "LSM tree + mmap",
    desc: "قاعدة بيانات مدمجة بـ mmap + LSM tree — لا حاجة لأي DBMS خارجي.",
    descEn: "In-process DB with mmap + LSM tree — no external DBMS required.",
    icon: Database,
    color: "primary",
    detail:
      "MemTable (lock-free skip-list) → SSTable (compressed, on-disk) → WAL (durability). كل الكتابات تذهب للـ MemTable أولاً، ثم تُflush لـ SSTable كل N MB. Compaction في الخلفية يدمج الـ SSTables. Bloom filter لكل SSTable يخبرك فوراً إن كان المفتاح غير موجود دون قراءة الملف.",
    apis: ["nawa_db::Engine", "Engine::put(k,v)", "Engine::get(k)", "Engine::scan(prefix)"],
  },
  {
    id: "be-workers",
    name: "Worker Pool + WASM Sandbox",
    nameAr: "عاملون + WASM sandbox",
    en: "user code sandboxed",
    desc: "عاملون خفيفون ينفذون كود المستخدم في sandbox آمن عبر WASM.",
    descEn: "Lightweight workers execute user code in WASM sandbox.",
    icon: Boxes,
    color: "primary",
    detail:
      "كل طلب مستخدم code (handlers, plugins) يُcompilation لـ WASM ويُنفذ في wasmtime sandbox. لا وصول للـ filesystem، لا شبكة مباشرة، لا fork — فقط APIs التي يوفّرها النواة. هذا يضمن أن plugin سيئ لا يُسقط النظام كله.",
    apis: ["wasmtime::Engine", "wasmtime::Linker", "Store::new()", "Func::call()"],
  },
  {
    id: "be-auth",
    name: "Zero-Trust Auth + WAF",
    nameAr: "مصادقة + جدار حماية",
    en: "JWT + WAF + rate-limit",
    desc: "مصادقة على كل طبقة + جدار حماية مدمج + rate limiting تلقائي.",
    descEn: "Per-layer auth + built-in WAF + auto rate-limiting.",
    icon: Shield,
    color: "primary",
    detail:
      "كل request يمر عبر middleware chain: IP allowlist → rate limiter → WAF (SQLi/XSS detection) → JWT verification → session validation. لا يوجد endpoint \"بدون حماية\" افتراضياً — يجب عليك أن تُصرّح بـ #[public] صراحةً.",
    apis: ["tower::Layer", "AuthLayer", "WafLayer", "RateLimitLayer"],
  },
];

const FRONTEND_LAYERS: Layer[] = [
  {
    id: "fe-ssr",
    name: "SSR Renderer",
    nameAr: "مُصيِّر SSR",
    en: "Rust → HTML",
    desc: "تصيير HTML على الخادم بـ hypertext — أول بايت يصل في أقل من 5ms.",
    descEn: "Server-side HTML rendering — first byte in under 5ms.",
    icon: Code2,
    color: "accent",
    detail:
      "مكتبة hypertext تُولِّد HTML string من Rust structs بأقل overhead ممكن. لا string concatenation، لا template engine — كل element هو typed struct يُسلسل مباشرةً للـ output buffer. النتيجة: SSR يأخذ ~1.2ms لصفحة معقدة.",
    apis: ["hypertext::maud!", "html! { }", "Render::render()", "stream::render()"],
  },
  {
    id: "fe-islands",
    name: "Island Hydration Engine",
    nameAr: "محرّك ترطيب الجزر",
    en: "selective hydration",
    desc: "فقط المكونات التفاعلية تُحمَّل كـ JS/WASM — باقي الصفحة HTML ثابت.",
    descEn: "Only interactive components ship as JS/WASM — rest is static HTML.",
    icon: Zap,
    color: "accent",
    detail:
      "كل صفحة SSR تُنتج HTML + قائمة بـ \"islands\" (المكونات التفاعلية). العميل يحمّل فقط الـ islands (3KB WASM runtime لكل island) ويربطها بالـ DOM الموجود. صفحة بـ 50KB HTML و island واحد بـ 5KB JS = أفضل من SPA بـ 200KB.",
    apis: ["#[island]", "Island::hydrate()", "ClientRuntime::new()", "EventBus::on()"],
  },
  {
    id: "fe-stream",
    name: "Streaming + Suspense",
    nameAr: "بثّ + Suspense",
    en: "streaming SSR",
    desc: "بث الصفحة تدريجياً — يصل المحتوى الثابت فوراً ثم تتدفق الجزر التفاعلية.",
    descEn: "Progressive page streaming — static first, then interactive islands.",
    icon: Cloud,
    color: "accent",
    detail:
      "بدل انتظار تصيير كامل الصفحة، الـ response يُstreamed: HTML ثابت أولاً، ثم placeholders للبيانات البطيئة، ثم تُستبدل بالبيانات الفعلية عند وصولها. العميل يرى الصفحة فوراً ويرى البيانات تتدفق تدريجياً.",
    apis: ["Suspense::new()", "Stream::chunk()", "Response::stream()", "chunked_transfer"],
  },
  {
    id: "fe-cache",
    name: "Edge Cache Layer",
    nameAr: "طبقة تخزين مؤقت",
    en: "SWR invalidation",
    desc: "تخزين مؤقت ذكي على مستوى الـ edge مع invalidation تلقائي عند تغيّر البيانات.",
    descEn: "Smart edge caching with auto-invalidation on data change.",
    icon: Activity,
    color: "accent",
    detail:
      "كل response يحمل ETag + Last-Modified + custom invalidation key. عند تغيّر البيانات في DB، الـ key يُبثّ لكل edge nodes لإبطال الكاش. الـ stale-while-revalidate يضمن أن المستخدم يرى محتوى قديم فوراً ثم يستبدل بالجديد في الخلفية.",
    apis: ["CachePolicy::Stale", "ETag::strong()", "InvalidateBus::publish()", "Edge::sync()"],
  },
  {
    id: "fe-runtime",
    name: "Client Runtime (3KB WASM)",
    nameAr: "وقت تشغيل العميل",
    en: "tiny WASM runtime",
    desc: "وقت تشغيل عميل صغير جداً يربط الجزر ببعضها عبر event bus مركزي.",
    descEn: "Tiny client runtime linking islands via central event bus.",
    icon: Layers,
    color: "accent",
    detail:
      "وقت تشغيل عميل بحجم 3KB (مُضغوط WASM) يُحمَّل مرة واحدة لكل session. يوفّر: event bus للمكونات، reactive state management، router-side navigation بدون reload، وclient-side caching. كل islands تستخدم نفس الـ runtime دون تكرار.",
    apis: ["runtime::init()", "Bus::emit()", "State::read()", "Router::navigate()"],
  },
  {
    id: "fe-hotreload",
    name: "Hot Reload Dev Server",
    nameAr: "إعادة تحميل ساخنة",
    en: "instant dev feedback",
    desc: "في وضع التطوير، أي تغيير في الكود يُعاد تحميله فوراً دون فقدان الحالة.",
    descEn: "In dev mode, code changes reload instantly with state preserved.",
    icon: Workflow,
    color: "accent",
    detail:
      "WebSocket بين dev server والمتصفح. عند تعديل ملف Rust، الـ incremental compilation تُنتج WASM جديد، يُبثّ للمتصفح، يُحقن في الـ runtime دون إعادة تحميل الصفحة. الحالة (forms, scroll, modals) تُحفظ.",
    apis: ["dev::watch()", "HotReload::patch()", "State::preserve()", "HMR::apply()"],
  },
];

const BRIDGES = [
  { icon: Workflow, label: "WebSocket / SSE", desc: "اتصال حي ثنائي الاتجاه" },
  { icon: GitMerge, label: "Shared Types", desc: "أنواع مشتركة بين الواجهة والخلفية" },
  { icon: Shield, label: "Zero-Trust Auth", desc: "مصادقة على كل طبقة" },
];

export function Architecture() {
  const [selected, setSelected] = useState<Layer | null>(null);

  return (
    <section id="architecture" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-grid opacity-30 pointer-events-none" />
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="المعمارية"
          eyebrowEn="System Architecture"
          title="محركان، نواة واحدة"
          titleEn="Two engines, one kernel"
          desc="النظام مبني على محركين منفصلين تماماً لكنهما يتشاركان نفس النواة الأساسية. محرك الخلفية يدير كل ما يخص الـ I/O والبيانات والأمان، ومحرك الواجهة يدير التصيير والـ hydration والـ caching. اضغط على أي طبقة لاستكشافها."
          descEn="Click any layer to explore its internals, APIs, and implementation."
        />

        {/* Diagram */}
        <div className="mt-16 grid lg:grid-cols-2 gap-6 lg:gap-8 relative">
          {/* Bridge in middle - desktop */}
          <div className="hidden lg:flex absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-10 flex-col items-center gap-3 pointer-events-none">
            <div className="w-px h-32 bg-gradient-to-b from-primary/0 via-primary/40 to-primary/0" />
            <div className="px-3 py-2 rounded-full border border-primary/40 bg-background backdrop-blur-md text-xs mono text-primary glow-amber">
              ⛓ shared kernel
            </div>
            <div className="w-px h-32 bg-gradient-to-b from-primary/0 via-primary/40 to-primary/0" />
          </div>

          {/* Backend Engine */}
          <EngineCard
            title="محرك الخلفية"
            titleEn="Backend Engine"
            icon={Server}
            color="primary"
            tagline="القوة والإنتاجية"
            taglineEn="Power & Throughput"
            layers={BACKEND_LAYERS}
            onSelect={setSelected}
          />

          {/* Frontend Engine */}
          <EngineCard
            title="محرك الواجهة"
            titleEn="Frontend Engine"
            icon={Monitor}
            color="accent"
            tagline="السرعة والخِفّة"
            taglineEn="Speed & Lightness"
            layers={FRONTEND_LAYERS}
            onSelect={setSelected}
          />
        </div>

        {/* Bridges */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-12 grid sm:grid-cols-3 gap-3"
        >
          {BRIDGES.map((b) => (
            <div
              key={b.label}
              className="flex items-center gap-3 p-4 rounded-xl border border-border/60 bg-card/60 hover:border-primary/40 transition-colors"
            >
              <div className="p-2 rounded-lg bg-primary/15 text-primary shrink-0">
                <b.icon className="w-4 h-4" strokeWidth={1.5} />
              </div>
              <div>
                <div className="text-sm font-medium">{b.label}</div>
                <div className="text-xs text-muted-foreground ar">{b.desc}</div>
              </div>
            </div>
          ))}
        </motion.div>

        {/* Request lifecycle */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-16"
        >
          <div className="flex items-center gap-2 mb-6">
            <Workflow className="w-4 h-4 text-primary" />
            <h3 className="text-base font-semibold ar">دورة حياة الطلب (Request Lifecycle)</h3>
            <Badge variant="outline" className="mono text-[10px] ml-1">
              total p50: ~2ms
            </Badge>
          </div>
          <div className="overflow-x-auto scrollbar-narrow -mx-4 px-4">
            <div className="flex items-stretch gap-2 min-w-max">
              {REQUEST_FLOW.map((step, i) => (
                <div key={step.label} className="flex items-center gap-2">
                  <div className="p-3 rounded-lg border border-border/60 bg-card min-w-[140px] hover:border-primary/40 transition-colors">
                    <div className="text-[10px] text-muted-foreground mono">step {i + 1}</div>
                    <div className="text-sm font-medium">{step.label}</div>
                    <div className="text-[10px] text-primary mt-1 mono">{step.latency}</div>
                  </div>
                  {i < REQUEST_FLOW.length - 1 && (
                    <ChevronLeft className="w-4 h-4 text-muted-foreground shrink-0" />
                  )}
                </div>
              ))}
            </div>
          </div>
        </motion.div>
      </div>

      {/* Layer detail dialog */}
      <AnimatePresence>
        {selected && (
          <LayerDetailDialog layer={selected} onClose={() => setSelected(null)} />
        )}
      </AnimatePresence>
    </section>
  );
}

const REQUEST_FLOW = [
  { label: "TLS handshake", latency: "~0ms (0-RTT)" },
  { label: "HTTP/3 parse", latency: "<0.1ms" },
  { label: "Router match", latency: "<0.05ms" },
  { label: "Middleware chain", latency: "<0.2ms" },
  { label: "Zero-copy DB read", latency: "~0.3ms" },
  { label: "SSR render", latency: "~1.2ms" },
  { label: "Stream to socket", latency: "~0ms (sendfile)" },
];

function EngineCard({
  title,
  titleEn,
  icon: Icon,
  color,
  tagline,
  taglineEn,
  layers,
  onSelect,
}: {
  title: string;
  titleEn: string;
  icon: typeof Server;
  color: "primary" | "accent";
  tagline: string;
  taglineEn: string;
  layers: Layer[];
  onSelect: (l: Layer) => void;
}) {
  const colorClasses =
    color === "primary"
      ? "from-primary/15 via-primary/5 to-transparent border-primary/30"
      : "from-accent/15 via-accent/5 to-transparent border-accent/30";
  const iconColor = color === "primary" ? "text-primary bg-primary/15" : "text-accent bg-accent/15";

  return (
    <motion.div
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-50px" }}
      transition={{ duration: 0.6 }}
      className={`relative rounded-2xl border bg-gradient-to-br p-6 lg:p-7 ${colorClasses}`}
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-4 mb-6">
        <div className="flex items-center gap-3">
          <div className={`p-3 rounded-xl ${iconColor}`}>
            <Icon className="w-6 h-6" strokeWidth={1.5} />
          </div>
          <div>
            <h3 className="text-xl font-bold ar">{title}</h3>
            <p className="text-xs text-muted-foreground mono">{titleEn}</p>
          </div>
        </div>
        <div className="text-right">
          <div
            className={`text-xs font-medium ${
              color === "primary" ? "text-primary" : "text-accent"
            } ar`}
          >
            {tagline}
          </div>
          <div className="text-[10px] text-muted-foreground">{taglineEn}</div>
        </div>
      </div>

      {/* Layers - clickable */}
      <div className="space-y-2">
        {layers.map((l, i) => (
          <motion.button
            key={l.id}
            initial={{ opacity: 0, x: -10 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.3, delay: i * 0.05 }}
            onClick={() => onSelect(l)}
            className={`group relative w-full text-right p-3 rounded-lg border transition-all ${
              l.color === "primary"
                ? "border-border/50 bg-card/40 hover:bg-card/80 hover:border-primary/50"
                : "border-border/50 bg-card/40 hover:bg-card/80 hover:border-accent/50"
            }`}
          >
            <div className="flex items-start justify-between gap-2 mb-1">
              <div className="flex items-center gap-2">
                <l.icon
                  className={`w-3.5 h-3.5 ${
                    l.color === "primary" ? "text-primary" : "text-accent"
                  }`}
                  strokeWidth={1.5}
                />
                <span className="text-sm font-medium ar">{l.nameAr}</span>
              </div>
              <span className="text-[10px] text-muted-foreground mono shrink-0">{l.en}</span>
            </div>
            <p className="text-xs text-muted-foreground leading-relaxed ar line-clamp-2">
              {l.desc}
            </p>
            <ChevronLeft
              className={`absolute top-1/2 -translate-y-1/2 left-2 w-3.5 h-3.5 opacity-0 group-hover:opacity-100 transition-opacity ${
                l.color === "primary" ? "text-primary" : "text-accent"
              }`}
            />
          </motion.button>
        ))}
      </div>
    </motion.div>
  );
}

function LayerDetailDialog({ layer, onClose }: { layer: Layer; onClose: () => void }) {
  const isPrimary = layer.color === "primary";
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 z-50 grid place-items-center p-4 bg-background/80 backdrop-blur-sm"
      onClick={onClose}
    >
      <motion.div
        initial={{ opacity: 0, scale: 0.95, y: 20 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: 20 }}
        transition={{ type: "spring", damping: 25, stiffness: 300 }}
        className="relative max-w-2xl w-full rounded-2xl border border-border/60 bg-card shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          className={`p-5 border-b border-border/40 bg-gradient-to-br ${
            isPrimary ? "from-primary/10 to-transparent" : "from-accent/10 to-transparent"
          }`}
        >
          <button
            onClick={onClose}
            className="absolute top-4 right-4 p-2 rounded-lg hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
          <div className="flex items-start gap-3">
            <div
              className={`p-3 rounded-xl ${
                isPrimary ? "bg-primary/15 text-primary" : "bg-accent/15 text-accent"
              }`}
            >
              <layer.icon className="w-6 h-6" strokeWidth={1.5} />
            </div>
            <div>
              <h3 className="text-xl font-bold ar">{layer.nameAr}</h3>
              <p className="text-xs text-muted-foreground mono mt-0.5">{layer.name}</p>
            </div>
          </div>
        </div>

        {/* Body */}
        <div className="p-5 space-y-5">
          {/* Description */}
          <div>
            <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-1.5 ar">
              الوصف
            </div>
            <p className="text-sm leading-relaxed ar">{layer.desc}</p>
            <p className="text-xs text-muted-foreground mt-2 italic">{layer.descEn}</p>
          </div>

          {/* Detail */}
          <div>
            <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-1.5 ar">
              تفاصيل التنفيذ
            </div>
            <p className="text-sm leading-relaxed text-foreground/90 ar">{layer.detail}</p>
          </div>

          {/* APIs */}
          <div>
            <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-2 ar">
              واجهات برمجية (APIs)
            </div>
            <div className="flex flex-wrap gap-1.5">
              {layer.apis.map((api) => (
                <code
                  key={api}
                  className={`px-2 py-1 rounded text-xs mono bg-card/60 border border-border/60 ${
                    isPrimary ? "text-primary" : "text-accent"
                  }`}
                >
                  {api}
                </code>
              ))}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="px-5 py-3 border-t border-border/40 bg-muted/30 flex items-center justify-between">
          <Badge
            variant="outline"
            className={`mono text-[10px] ${
              isPrimary ? "border-primary/40 text-primary" : "border-accent/40 text-accent"
            }`}
          >
            {isPrimary ? "Backend Engine" : "Frontend Engine"}
          </Badge>
          <span className="text-[10px] text-muted-foreground mono">click outside to close</span>
        </div>
      </motion.div>
    </motion.div>
  );
}
