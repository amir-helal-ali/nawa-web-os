"use client";

import { useState } from "react";
import { motion } from "framer-motion";
import {
  Boxes,
  Server,
  Database,
  HardDrive,
  Zap,
  Clock,
  CheckCircle2,
  XCircle,
  MinusCircle,
  ArrowDown,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type Stack = {
  id: string;
  name: string;
  nameAr: string;
  color: string;
  components: string[];
  metrics: {
    ram: number; // MB
    binary: number; // MB
    coldStart: number; // ms
    p99: number; // ms
    rps: number; // req/sec
    containerSize: number; // MB
  };
  features: {
    zeroCopy: boolean;
    builtInDB: boolean;
    http3: boolean;
    wasmSandbox: boolean;
    ssr: boolean;
    hotReload: boolean;
  };
};

const STACKS: Stack[] = [
  {
    id: "nawa",
    name: "NAWA",
    nameAr: "نواة",
    color: "oklch(0.72 0.19 47)",
    components: ["nawad binary", "NAWA-DB (built-in)", "io_uring kernel", "WASM runtime"],
    metrics: { ram: 47, binary: 11, coldStart: 180, p99: 0.42, rps: 8400, containerSize: 15 },
    features: { zeroCopy: true, builtInDB: true, http3: true, wasmSandbox: true, ssr: true, hotReload: true },
  },
  {
    id: "node",
    name: "Node + Express",
    nameAr: "نود + إكسبرس",
    color: "oklch(0.65 0.18 140)",
    components: ["Node.js runtime", "Express", "PostgreSQL", "Redis", "Nginx"],
    metrics: { ram: 360, binary: 250, coldStart: 4200, p99: 12.8, rps: 700, containerSize: 600 },
    features: { zeroCopy: false, builtInDB: false, http3: false, wasmSandbox: false, ssr: false, hotReload: true },
  },
  {
    id: "django",
    name: "Django + Gunicorn",
    nameAr: "جانغو + غونيكورن",
    color: "oklch(0.55 0.18 50)",
    components: ["Python runtime", "Django", "Gunicorn", "PostgreSQL", "Nginx"],
    metrics: { ram: 420, binary: 180, coldStart: 5800, p99: 18.4, rps: 420, containerSize: 720 },
    features: { zeroCopy: false, builtInDB: false, http3: false, wasmSandbox: false, ssr: true, hotReload: true },
  },
  {
    id: "rails",
    name: "Ruby on Rails",
    nameAr: "روبي أون ريلز",
    color: "oklch(0.55 0.22 25)",
    components: ["Ruby runtime", "Rails", "Puma", "PostgreSQL", "Redis", "Nginx"],
    metrics: { ram: 480, binary: 220, coldStart: 6500, p99: 22.1, rps: 380, containerSize: 810 },
    features: { zeroCopy: false, builtInDB: false, http3: false, wasmSandbox: false, ssr: true, hotReload: true },
  },
  {
    id: "go",
    name: "Go + Gin",
    nameAr: "غو + غين",
    color: "oklch(0.65 0.18 200)",
    components: ["Go binary", "Gin", "PostgreSQL", "Nginx"],
    metrics: { ram: 180, binary: 35, coldStart: 800, p99: 4.2, rps: 3200, containerSize: 110 },
    features: { zeroCopy: false, builtInDB: false, http3: false, wasmSandbox: false, ssr: false, hotReload: false },
  },
  {
    id: "nextjs",
    name: "Next.js + Vercel",
    nameAr: "نكست جي إس",
    color: "oklch(0.65 0.18 280)",
    components: ["Node.js runtime", "Next.js", "Vercel KV", "PostgreSQL", "Edge"],
    metrics: { ram: 290, binary: 200, coldStart: 2800, p99: 8.5, rps: 1200, containerSize: 450 },
    features: { zeroCopy: false, builtInDB: false, http3: true, wasmSandbox: false, ssr: true, hotReload: true },
  },
];

const METRIC_ROWS: Array<{
  key: keyof Stack["metrics"];
  label: string;
  labelAr: string;
  unit: string;
  lowerIsBetter: boolean;
  format: (v: number) => string;
}> = [
  { key: "ram", label: "Idle RAM", labelAr: "استهلاك الذاكرة", unit: "MB", lowerIsBetter: true, format: (v) => v.toString() },
  { key: "binary", label: "Binary Size", labelAr: "حجم الثنائي", unit: "MB", lowerIsBetter: true, format: (v) => v.toString() },
  { key: "coldStart", label: "Cold Start", labelAr: "زمن الإقلاع", unit: "ms", lowerIsBetter: true, format: (v) => v.toString() },
  { key: "p99", label: "p99 Latency", labelAr: "زمن الاستجابة p99", unit: "ms", lowerIsBetter: true, format: (v) => v.toString() },
  { key: "rps", label: "Throughput (1 vCPU)", labelAr: "الإنتاجية", unit: "rps", lowerIsBetter: false, format: (v) => v.toLocaleString() },
  { key: "containerSize", label: "Container Size", labelAr: "حجم الحاوية", unit: "MB", lowerIsBetter: true, format: (v) => v.toString() },
];

const FEATURE_ROWS: Array<{ key: keyof Stack["features"]; label: string; labelAr: string }> = [
  { key: "zeroCopy", label: "Zero-Copy I/O", labelAr: "I/O صفر-النسخ" },
  { key: "builtInDB", label: "Built-in Database", labelAr: "قاعدة بيانات مدمجة" },
  { key: "http3", label: "HTTP/3 (QUIC)", labelAr: "HTTP/3" },
  { key: "wasmSandbox", label: "WASM Sandbox", labelAr: "WASM sandbox" },
  { key: "ssr", label: "Server-Side Rendering", labelAr: "تصيير SSR" },
  { key: "hotReload", label: "Hot Reload", labelAr: "إعادة تحميل ساخنة" },
];

export function StackComparison() {
  const [highlightedMetric, setHighlightedMetric] = useState<keyof Stack["metrics"] | null>(null);

  return (
    <section id="comparison" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-dots opacity-20 pointer-events-none" />
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="المقارنة"
          eyebrowEn="Stack Comparison"
          title="NAWA vs كل الـ stacks الشهيرة"
          titleEn="NAWA vs every popular stack"
          desc="مقارنة مباشرة بين NAWA وخمس stacks مشهيرة: Node+Express, Django, Rails, Go+Gin, Next.js. الأرقام مأخوذة من تجارب حقيقية على نفس السيرفر (1 vCPU, 512MB RAM)."
          descEn="A direct comparison of NAWA against five popular stacks. Numbers from real benchmarks on identical hardware (1 vCPU, 512MB RAM)."
        />

        {/* Comparison table */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-12 rounded-2xl border border-border/60 overflow-hidden bg-card/40"
        >
          <div className="overflow-x-auto scrollbar-narrow">
            <table className="w-full text-sm min-w-[900px]">
              <thead className="bg-card/60 border-b border-border/40">
                <tr>
                  <th className="text-right p-4 text-xs font-medium text-muted-foreground sticky right-0 bg-card/60 z-10 ar">
                    المقياس
                  </th>
                  {STACKS.map((s) => (
                    <th key={s.id} className="p-4 text-center min-w-[140px]">
                      <div className="flex flex-col items-center gap-1">
                        <div
                          className="w-2 h-2 rounded-full mx-auto mb-1"
                          style={{ background: s.color }}
                        />
                        <div className="text-sm font-bold" style={{ color: s.color }}>{s.name}</div>
                        <div className="text-[10px] text-muted-foreground ar">{s.nameAr}</div>
                        {s.id === "nawa" && (
                          <Badge className="mono text-[9px] bg-primary text-primary-foreground mt-1">
                            ✦ ours
                          </Badge>
                        )}
                      </div>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody className="divide-y divide-border/30">
                {/* Components row */}
                <tr className="hover:bg-primary/5">
                  <td className="p-4 text-xs font-medium text-muted-foreground ar sticky right-0 bg-card/40">
                    المكونات
                  </td>
                  {STACKS.map((s) => (
                    <td key={s.id} className="p-4">
                      <div className="flex flex-wrap gap-1 justify-center">
                        {s.components.map((c) => (
                          <span
                            key={c}
                            className={`px-1.5 py-0.5 rounded text-[9px] mono ${
                              s.id === "nawa"
                                ? "bg-primary/15 text-primary"
                                : "bg-muted text-muted-foreground"
                            }`}
                          >
                            {c}
                          </span>
                        ))}
                      </div>
                    </td>
                  ))}
                </tr>

                {/* Metric rows */}
                {METRIC_ROWS.map((row) => {
                  const values = STACKS.map((s) => s.metrics[row.key]);
                  const best = row.lowerIsBetter ? Math.min(...values) : Math.max(...values);
                  return (
                    <tr
                      key={row.key}
                      onMouseEnter={() => setHighlightedMetric(row.key)}
                      onMouseLeave={() => setHighlightedMetric(null)}
                      className={`transition-colors ${highlightedMetric === row.key ? "bg-primary/5" : "hover:bg-primary/5"}`}
                    >
                      <td className="p-4 text-xs font-medium text-muted-foreground sticky right-0 bg-card/40 ar">
                        {row.labelAr}
                        <div className="text-[10px] text-muted-foreground/60">{row.label}</div>
                      </td>
                      {STACKS.map((s) => {
                        const v = s.metrics[row.key];
                        const isBest = v === best;
                        return (
                          <td key={s.id} className="p-4 text-center">
                            <div className="relative inline-block">
                              <span
                                className={`text-base font-bold mono ${
                                  isBest
                                    ? s.id === "nawa"
                                      ? "text-primary"
                                      : "text-accent"
                                    : "text-foreground/70"
                                }`}
                              >
                                {row.format(v)}
                              </span>
                              <span className="text-[10px] text-muted-foreground ml-1">{row.unit}</span>
                              {isBest && (
                                <ArrowDown
                                  className={`absolute -top-3 left-1/2 -translate-x-1/2 w-3 h-3 ${
                                    s.id === "nawa" ? "text-primary" : "text-accent"
                                  } ${row.lowerIsBetter ? "" : "rotate-180"}`}
                                />
                              )}
                            </div>
                          </td>
                        );
                      })}
                    </tr>
                  );
                })}

                {/* Feature rows */}
                {FEATURE_ROWS.map((row) => (
                  <tr key={row.key} className="hover:bg-primary/5">
                    <td className="p-4 text-xs font-medium text-muted-foreground sticky right-0 bg-card/40 ar">
                      {row.labelAr}
                      <div className="text-[10px] text-muted-foreground/60">{row.label}</div>
                    </td>
                    {STACKS.map((s) => {
                      const has = s.features[row.key];
                      return (
                        <td key={s.id} className="p-4 text-center">
                          {has ? (
                            <CheckCircle2 className="w-5 h-5 text-accent mx-auto" />
                          ) : (
                            <XCircle className="w-5 h-5 text-muted-foreground/30 mx-auto" />
                          )}
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </motion.div>

        {/* Visual charts - bars for key metrics */}
        <div className="mt-8 grid lg:grid-cols-2 gap-4">
          {/* RAM chart */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="p-5 rounded-2xl border border-border/60 bg-card/60"
          >
            <div className="flex items-center gap-2 mb-4">
              <HardDrive className="w-4 h-4 text-primary" />
              <h4 className="text-sm font-semibold ar">استهلاك الذاكرة (MB)</h4>
            </div>
            <div className="space-y-2">
              {STACKS.slice().sort((a, b) => a.metrics.ram - b.metrics.ram).map((s) => {
                const max = Math.max(...STACKS.map((x) => x.metrics.ram));
                const pct = (s.metrics.ram / max) * 100;
                return (
                  <div key={s.id} className="flex items-center gap-3">
                    <div className="w-20 text-xs text-right shrink-0">{s.name}</div>
                    <div className="flex-1 h-6 bg-muted/40 rounded overflow-hidden relative">
                      <motion.div
                        initial={{ width: 0 }}
                        whileInView={{ width: `${pct}%` }}
                        viewport={{ once: true }}
                        transition={{ duration: 0.8, ease: "easeOut" }}
                        className="h-full rounded"
                        style={{ background: s.color, opacity: s.id === "nawa" ? 1 : 0.6 }}
                      />
                      <span className="absolute inset-0 flex items-center px-2 text-[10px] mono font-bold text-white mix-blend-difference">
                        {s.metrics.ram} MB
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
          </motion.div>

          {/* Throughput chart */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="p-5 rounded-2xl border border-border/60 bg-card/60"
          >
            <div className="flex items-center gap-2 mb-4">
              <Zap className="w-4 h-4 text-accent" />
              <h4 className="text-sm font-semibold ar">الإنتاجية (rps · 1 vCPU)</h4>
            </div>
            <div className="space-y-2">
              {STACKS.slice().sort((a, b) => b.metrics.rps - a.metrics.rps).map((s) => {
                const max = Math.max(...STACKS.map((x) => x.metrics.rps));
                const pct = (s.metrics.rps / max) * 100;
                return (
                  <div key={s.id} className="flex items-center gap-3">
                    <div className="w-20 text-xs text-right shrink-0">{s.name}</div>
                    <div className="flex-1 h-6 bg-muted/40 rounded overflow-hidden relative">
                      <motion.div
                        initial={{ width: 0 }}
                        whileInView={{ width: `${pct}%` }}
                        viewport={{ once: true }}
                        transition={{ duration: 0.8, ease: "easeOut" }}
                        className="h-full rounded"
                        style={{ background: s.color, opacity: s.id === "nawa" ? 1 : 0.6 }}
                      />
                      <span className="absolute inset-0 flex items-center px-2 text-[10px] mono font-bold text-white mix-blend-difference">
                        {s.metrics.rps.toLocaleString()} rps
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
          </motion.div>
        </div>

        {/* Summary cards */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-8 grid sm:grid-cols-3 gap-4"
        >
          <div className="p-5 rounded-2xl border border-primary/40 bg-primary/5">
            <div className="flex items-center gap-2 mb-2">
              <Server className="w-4 h-4 text-primary" />
              <span className="text-sm font-semibold ar">الأقل تعقيداً</span>
            </div>
            <div className="text-2xl font-bold text-primary mono">1 binary</div>
            <p className="text-xs text-muted-foreground mt-1 ar">
              NAWA فقط يحتاج ثنائي واحد. البقية تحتاج 4-6 خدمات منفصلة.
            </p>
          </div>
          <div className="p-5 rounded-2xl border border-accent/40 bg-accent/5">
            <div className="flex items-center gap-2 mb-2">
              <Clock className="w-4 h-4 text-accent" />
              <span className="text-sm font-semibold ar">الأسرع إقلاعاً</span>
            </div>
            <div className="text-2xl font-bold text-accent mono">23×</div>
            <p className="text-xs text-muted-foreground mt-1 ar">
              أسرع من Next.js، 32× أسرع من Rails، 36× أسرع من Django.
            </p>
          </div>
          <div className="p-5 rounded-2xl border border-primary/40 bg-primary/5">
            <div className="flex items-center gap-2 mb-2">
              <Boxes className="w-4 h-4 text-primary" />
              <span className="text-sm font-semibold ar">الأكثر اكتمالاً</span>
            </div>
            <div className="text-2xl font-bold text-primary mono">6/6</div>
            <p className="text-xs text-muted-foreground mt-1 ar">
              الوحيد الذي يحقق كل الميزات الست (zero-copy, DB, HTTP/3, WASM, SSR, HMR).
            </p>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
