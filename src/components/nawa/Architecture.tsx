"use client";

import { motion } from "framer-motion";
import {
  Server,
  Monitor,
  Database,
  Router,
  Shield,
  Cpu,
  Zap,
  GitMerge,
  ArrowRight,
  Workflow,
} from "lucide-react";
import { SectionHeader } from "./Concept";

const BACKEND_LAYERS = [
  {
    name: "HTTP/3 + TLS Server",
    en: "Quic-based server",
    desc: "خادم HTTP/3 مبني على quinn، يدعم keep-alive وconnection pooling.",
    color: "primary",
  },
  {
    name: "Router + Middleware Chain",
    en: "Type-safe routing",
    desc: "توجيه ثابت الأنواع مع middlewares: auth، rate-limit، logging، cache.",
    color: "primary",
  },
  {
    name: "Zero-Copy Kernel",
    en: "io_uring + mmap",
    desc: "النواة الثورية — I/O غير متزامن بالكامل دون نسخ البيانات في الذاكرة.",
    color: "primary",
    highlight: true,
  },
  {
    name: "Built-in KV/Document DB",
    en: "Self-contained DB",
    desc: "قاعدة بيانات مدمجة بـ mmap + LSM tree — لا حاجة لأي DBMS خارجي.",
    color: "primary",
  },
  {
    name: "Worker Pool + WASM Sandbox",
    en: "Sandboxed compute",
    desc: "عاملون خفيفون ينفذون كود المستخدم في sandbox آمن عبر WASM.",
    color: "primary",
  },
];

const FRONTEND_LAYERS = [
  {
    name: "SSR Renderer (Rust → HTML)",
    en: "Server-side rendering",
    desc: "تصيير HTML على الخادم بـ hypertext — أول بايت يصل في أقل من 5ms.",
    color: "accent",
  },
  {
    name: "Island Hydration Engine",
    en: "Selective hydration",
    desc: "فقط المكونات التفاعلية تُحمَّل كـ JS/WASM — باقي الصفحة HTML ثابت.",
    color: "accent",
    highlight: true,
  },
  {
    name: "Streaming + Suspense",
    en: "Streaming SSR",
    desc: "بث الصفحة تدريجياً — يصل المحتوى الثابت فوراً ثم تتدفق الجزر التفاعلية.",
    color: "accent",
  },
  {
    name: "Edge Cache Layer",
    en: "Stale-while-revalidate",
    desc: "تخزين مؤقت ذكي على مستوى الـ edge مع invalidation تلقائي عند تغيّر البيانات.",
    color: "accent",
  },
  {
    name: "Client Runtime (3KB WASM)",
    en: "Tiny client runtime",
    desc: "وقت تشغيل عميل صغير جداً يربط الجزر ببعضها عبر event bus مركزي.",
    color: "accent",
  },
];

const BRIDGES = [
  { icon: Workflow, label: "WebSocket / SSE", desc: "اتصال حي ثنائي الاتجاه" },
  { icon: GitMerge, label: "Shared Types", desc: "أنواع مشتركة بين الواجهة والخلفية" },
  { icon: Shield, label: "Zero-Trust Auth", desc: "مصادقة على كل طبقة" },
];

export function Architecture() {
  return (
    <section id="architecture" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-grid opacity-30 pointer-events-none" />
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="المعمارية"
          eyebrowEn="System Architecture"
          title="محركان، نواة واحدة"
          titleEn="Two engines, one kernel"
          desc="النظام مبني على محركين منفصلين تماماً لكنهما يتشاركان نفس النواة الأساسية. محرك الخلفية يدير كل ما يخص الـ I/O والبيانات والأمان، ومحرك الواجهة يدير التصيير والـ hydration والـ caching."
          descEn="Two fully-separated engines sharing the same kernel. Backend owns I/O, data and security. Frontend owns rendering, hydration and caching."
        />

        {/* Diagram */}
        <div className="mt-16 grid lg:grid-cols-2 gap-6 lg:gap-8 relative">
          {/* Bridge in middle - desktop */}
          <div className="hidden lg:flex absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-10 flex-col items-center gap-3">
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
          </div>
          <div className="overflow-x-auto scrollbar-narrow -mx-4 px-4">
            <div className="flex items-stretch gap-2 min-w-max">
              {REQUEST_FLOW.map((step, i) => (
                <div key={step.label} className="flex items-center gap-2">
                  <div className="p-3 rounded-lg border border-border/60 bg-card min-w-[140px]">
                    <div className="text-[10px] text-muted-foreground mono">step {i + 1}</div>
                    <div className="text-sm font-medium mt-1">{step.label}</div>
                    <div className="text-[10px] text-primary mt-1 mono">{step.latency}</div>
                  </div>
                  {i < REQUEST_FLOW.length - 1 && (
                    <ArrowRight className="w-4 h-4 text-muted-foreground shrink-0" />
                  )}
                </div>
              ))}
            </div>
          </div>
        </motion.div>
      </div>
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
}: {
  title: string;
  titleEn: string;
  icon: typeof Server;
  color: "primary" | "accent";
  tagline: string;
  taglineEn: string;
  layers: Array<{ name: string; en: string; desc: string; color: string; highlight?: boolean }>;
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
          <div className={`text-xs font-medium ${color === "primary" ? "text-primary" : "text-accent"} ar`}>
            {tagline}
          </div>
          <div className="text-[10px] text-muted-foreground">{taglineEn}</div>
        </div>
      </div>

      {/* Layers */}
      <div className="space-y-2">
        {layers.map((l, i) => (
          <motion.div
            key={l.en}
            initial={{ opacity: 0, x: -10 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.3, delay: i * 0.05 }}
            className={`group relative p-3 rounded-lg border transition-all ${
              l.highlight
                ? color === "primary"
                  ? "border-primary/50 bg-primary/10"
                  : "border-accent/50 bg-accent/10"
                : "border-border/50 bg-card/40 hover:bg-card/70"
            }`}
          >
            <div className="flex items-start justify-between gap-2 mb-1">
              <div className="flex items-center gap-2">
                {l.highlight && (
                  <Zap className={`w-3.5 h-3.5 ${color === "primary" ? "text-primary" : "text-accent"}`} />
                )}
                <span className="text-sm font-medium ar">{l.name}</span>
              </div>
              <span className="text-[10px] text-muted-foreground mono shrink-0">{l.en}</span>
            </div>
            <p className="text-xs text-muted-foreground leading-relaxed ar">{l.desc}</p>
            {l.highlight && (
              <div className="absolute -top-1 -right-1 px-1.5 py-0.5 rounded-full bg-primary text-primary-foreground text-[9px] font-bold mono">
                CORE
              </div>
            )}
          </motion.div>
        ))}
      </div>
    </motion.div>
  );
}
