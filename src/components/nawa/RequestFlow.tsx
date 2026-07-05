"use client";

import { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Globe,
  Lock,
  Router,
  ShieldCheck,
  Database,
  Code2,
  Cloud,
  ArrowLeft,
  Activity,
  Zap,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

type StageId = "tls" | "router" | "middleware" | "db" | "render" | "stream" | "done";

const STAGES: Array<{
  id: StageId;
  name: string;
  nameAr: string;
  icon: typeof Globe;
  color: string;
  duration: number; // ms (simulated)
  detail: string;
}> = [
  {
    id: "tls",
    name: "TLS 1.3 + QUIC",
    nameAr: "TLS handshake",
    icon: Lock,
    color: "oklch(0.72 0.19 47)",
    duration: 0,
    detail: "0-RTT resumption — العميل يرسل البيانات في أول packet دون انتظار handshake كامل.",
  },
  {
    id: "router",
    name: "Router Match",
    nameAr: "مطابقة المسار",
    icon: Router,
    color: "oklch(0.78 0.16 165)",
    duration: 50,
    detail: "Type-safe routing بـ proc-macro. الـ path parameters تُستخرج في compile-time.",
  },
  {
    id: "middleware",
    name: "Middleware Chain",
    nameAr: "سلسلة الـ middleware",
    icon: ShieldCheck,
    color: "oklch(0.65 0.22 25)",
    duration: 200,
    detail: "Auth + WAF + rate-limit + CSP — كلها تمر بـ tower::Layer chain. لا overhead.",
  },
  {
    id: "db",
    name: "NAWA-DB Read",
    nameAr: "قراءة قاعدة البيانات",
    icon: Database,
    color: "oklch(0.65 0.20 300)",
    duration: 90,
    detail: "mmap للوصول المباشر للـ SSTable + Bloom filter لاستبعاد المفاتيح غير الموجودة.",
  },
  {
    id: "render",
    name: "SSR Render",
    nameAr: "تصيير SSR",
    icon: Code2,
    color: "oklch(0.80 0.15 90)",
    duration: 1200,
    detail: "hypertext تُولِّد HTML string من Rust structs بأقل overhead. لا template engine.",
  },
  {
    id: "stream",
    name: "Stream to Socket",
    nameAr: "بثّ للـ socket",
    icon: Cloud,
    color: "oklch(0.70 0.18 200)",
    duration: 0,
    detail: "sendfile + MSG_ZEROCOPY — HTML يُرسل من القرص للـ socket دون نسخ user-space.",
  },
];

export function RequestFlow() {
  const [activeStage, setActiveStage] = useState<number>(-1);
  const [running, setRunning] = useState(false);
  const [totalTime, setTotalTime] = useState(0);
  const [runs, setRuns] = useState(0);

  const runFlow = async () => {
    if (running) return;
    setRunning(true);
    setActiveStage(-1);
    setTotalTime(0);

    let cumulative = 0;
    for (let i = 0; i < STAGES.length; i++) {
      setActiveStage(i);
      cumulative += STAGES[i].duration;
      setTotalTime(cumulative);
      await new Promise((r) => setTimeout(r, Math.max(300, STAGES[i].duration / 3)));
    }
    setActiveStage(STAGES.length);
    setRuns((r) => r + 1);
    setRunning(false);
  };

  useEffect(() => {
    const id = setTimeout(runFlow, 800);
    return () => clearTimeout(id);
  }, []);

  return (
    <section id="flow" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-grid opacity-30 pointer-events-none" />
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="رحلة الطلب"
          eyebrowEn="Request Flow"
          title="من النقرة إلى الشاشة · في 1.5ms"
          titleEn="From click to screen — in 1.5ms"
          desc="تتبّع بصري لرحلة طلب HTTP داخل NAWA — من TLS handshake إلى بثّ HTML للعميل. كل مرحلة مُحسّنة بـ zero-copy وasync I/O."
          descEn="Visual trace of an HTTP request's journey through NAWA — from TLS handshake to HTML streaming. Every stage optimized with zero-copy and async I/O."
        />

        {/* Controls */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 flex items-center justify-between p-3 rounded-xl border border-border/60 bg-card/60"
        >
          <div className="flex items-center gap-3">
            <Activity className={`w-4 h-4 ${running ? "text-accent animate-nawa-pulse" : "text-muted-foreground"}`} />
            <span className="text-sm font-medium ar">رحلة طلب HTTP/3</span>
            {runs > 0 && (
              <Badge variant="outline" className="mono text-[10px]">
                runs: {runs}
              </Badge>
            )}
          </div>
          <div className="flex items-center gap-4">
            <div className="text-sm">
              <span className="text-muted-foreground">total:</span>{" "}
              <span className="font-bold text-primary mono">{(totalTime / 1000).toFixed(2)}ms</span>
            </div>
            <Button
              size="sm"
              onClick={runFlow}
              disabled={running}
              className="bg-primary hover:bg-primary/90 text-primary-foreground h-8"
            >
              <Zap className="w-3 h-3" />
              <span className="mr-1 ar">{running ? "جارٍ..." : "إعادة التشغيل"}</span>
            </Button>
          </div>
        </motion.div>

        {/* Visual flow */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-8"
        >
          {/* Desktop: horizontal flow */}
          <div className="hidden lg:block relative overflow-hidden rounded-2xl border border-border/60 bg-[#0d0c0a] p-8">
            {/* Track line */}
            <div className="absolute top-1/2 inset-x-12 h-0.5 bg-border/40 -translate-y-1/2" />
            <motion.div
              className="absolute top-1/2 left-12 h-0.5 bg-primary -translate-y-1/2"
              animate={{
                width: activeStage >= 0 ? `calc((100% - 6rem) * ${Math.min(activeStage / (STAGES.length - 1), 1)})` : "0%",
              }}
              transition={{ duration: 0.3 }}
            />

            {/* Stages */}
            <div className="relative grid grid-cols-6 gap-2">
              {STAGES.map((stage, i) => {
                const isActive = activeStage === i;
                const isPast = activeStage > i;
                return (
                  <div key={stage.id} className="flex flex-col items-center">
                    <motion.div
                      animate={{
                        scale: isActive ? 1.15 : 1,
                        y: isActive ? -4 : 0,
                      }}
                      transition={{ type: "spring", stiffness: 300, damping: 20 }}
                      className={`relative z-10 p-3 rounded-xl border-2 transition-colors ${
                        isActive
                          ? "border-primary bg-primary/15"
                          : isPast
                          ? "border-accent/40 bg-accent/10"
                          : "border-border/60 bg-card/40"
                      }`}
                      style={{
                        boxShadow: isActive ? `0 0 30px -5px ${stage.color}` : "none",
                      }}
                    >
                      <stage.icon
                        className="w-6 h-6"
                        strokeWidth={1.5}
                        style={{
                          color: isActive || isPast ? stage.color : "oklch(0.5 0 0)",
                        }}
                      />
                      {isPast && (
                        <motion.div
                          initial={{ scale: 0 }}
                          animate={{ scale: 1 }}
                          className="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-accent grid place-items-center"
                        >
                          <svg viewBox="0 0 12 12" className="w-2.5 h-2.5 text-accent-foreground">
                            <path d="M2 6l3 3 5-6" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
                          </svg>
                        </motion.div>
                      )}
                    </motion.div>
                    <div className={`mt-3 text-xs font-medium text-center ${isActive ? "text-primary" : isPast ? "text-foreground" : "text-muted-foreground"}`}>
                      {stage.name}
                    </div>
                    <div className="text-[10px] text-muted-foreground text-center mt-0.5 ar">
                      {stage.nameAr}
                    </div>
                    {isActive && (
                      <motion.div
                        initial={{ opacity: 0, y: 5 }}
                        animate={{ opacity: 1, y: 0 }}
                        className="text-[10px] mono mt-1"
                        style={{ color: stage.color }}
                      >
                        {stage.duration > 0 ? `${stage.duration}μs` : "~0ms"}
                      </motion.div>
                    )}
                  </div>
                );
              })}
            </div>

            {/* Active detail */}
            <AnimatePresence mode="wait">
              {activeStage >= 0 && activeStage < STAGES.length && (
                <motion.div
                  key={activeStage}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -10 }}
                  className="mt-6 p-4 rounded-lg border border-border/40 bg-card/40"
                >
                  <div className="flex items-center gap-2 mb-2">
                    <div
                      className="w-2 h-2 rounded-full animate-nawa-pulse"
                      style={{ background: STAGES[activeStage].color }}
                    />
                    <span className="text-xs font-medium mono">{STAGES[activeStage].name}</span>
                    <Badge variant="outline" className="mono text-[10px]">
                      step {activeStage + 1}/{STAGES.length}
                    </Badge>
                  </div>
                  <p className="text-xs text-foreground/80 leading-relaxed ar">
                    {STAGES[activeStage].detail}
                  </p>
                </motion.div>
              )}
            </AnimatePresence>

            {/* Final result */}
            {activeStage === STAGES.length && (
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                className="mt-6 p-4 rounded-lg border border-accent/40 bg-accent/10 flex items-center gap-3"
              >
                <ShieldCheck className="w-5 h-5 text-accent" />
                <div>
                  <div className="text-sm font-medium text-accent ar">اكتمل الطلب بنجاح</div>
                  <div className="text-xs text-muted-foreground mono">
                    200 OK · total: {(totalTime / 1000).toFixed(2)}ms · zero copies · 2 syscalls
                  </div>
                </div>
              </motion.div>
            )}
          </div>

          {/* Mobile: vertical flow */}
          <div className="lg:hidden rounded-2xl border border-border/60 bg-[#0d0c0a] p-4">
            <div className="space-y-3">
              {STAGES.map((stage, i) => {
                const isActive = activeStage === i;
                const isPast = activeStage > i;
                return (
                  <div key={stage.id} className="flex items-center gap-3">
                    <motion.div
                      animate={{ scale: isActive ? 1.1 : 1 }}
                      className={`relative p-2 rounded-lg border-2 shrink-0 ${
                        isActive
                          ? "border-primary bg-primary/15"
                          : isPast
                          ? "border-accent/40 bg-accent/10"
                          : "border-border/60 bg-card/40"
                      }`}
                    >
                      <stage.icon
                        className="w-4 h-4"
                        style={{ color: isActive || isPast ? stage.color : "oklch(0.5 0 0)" }}
                      />
                    </motion.div>
                    <div className="flex-1 min-w-0">
                      <div className={`text-xs font-medium ${isActive ? "text-primary" : isPast ? "text-foreground" : "text-muted-foreground"}`}>
                        {stage.name}
                      </div>
                      <div className="text-[10px] text-muted-foreground ar">{stage.nameAr}</div>
                    </div>
                    <span className="text-[10px] mono text-muted-foreground">
                      {stage.duration > 0 ? `${stage.duration}μs` : "~0ms"}
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        </motion.div>

        {/* Latency breakdown */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-8 grid lg:grid-cols-2 gap-4"
        >
          <div className="p-5 rounded-2xl border border-border/60 bg-card/60">
            <h4 className="text-sm font-semibold mb-4 ar">توزيع زمن الطلب (μs)</h4>
            <div className="space-y-2">
              {STAGES.filter((s) => s.duration > 0).map((s) => {
                const total = STAGES.reduce((sum, x) => sum + x.duration, 0);
                const pct = (s.duration / total) * 100;
                return (
                  <div key={s.id} className="flex items-center gap-3 text-xs">
                    <div className="w-24 text-muted-foreground text-right shrink-0">{s.name}</div>
                    <div className="flex-1 h-4 bg-muted/40 rounded overflow-hidden relative">
                      <motion.div
                        initial={{ width: 0 }}
                        whileInView={{ width: `${pct}%` }}
                        viewport={{ once: true }}
                        transition={{ duration: 0.8 }}
                        className="h-full rounded"
                        style={{ background: s.color }}
                      />
                    </div>
                    <span className="mono text-muted-foreground w-12 shrink-0">{s.duration}μs</span>
                    <span className="mono text-muted-foreground/60 w-10 shrink-0">{pct.toFixed(0)}%</span>
                  </div>
                );
              })}
            </div>
            <div className="mt-3 pt-3 border-t border-border/40 flex justify-between text-xs">
              <span className="text-muted-foreground ar">الإجمالي</span>
              <span className="font-bold mono text-primary">
                {STAGES.reduce((sum, x) => sum + x.duration, 0)}μs = {(STAGES.reduce((sum, x) => sum + x.duration, 0) / 1000).toFixed(2)}ms
              </span>
            </div>
          </div>

          <div className="p-5 rounded-2xl border border-border/60 bg-card/60">
            <h4 className="text-sm font-semibold mb-4 ar">لماذا بهذه السرعة؟</h4>
            <ul className="space-y-2.5 text-xs">
              {[
                { point: "TLS 0-RTT", desc: "تجاوز handshake كامل للجلسات العائدة" },
                { point: "Compile-time routing", desc: "لا string parsing في runtime" },
                { point: "Lock-free middleware", desc: "لا mutexes في الـ hot path" },
                { point: "mmap + Bloom filter", desc: "وصول مباشر للقرص دون نسخ" },
                { point: "hypertext renderer", desc: "HTML بدون template engine" },
                { point: "sendfile + MSG_ZEROCOPY", desc: "قرص → socket بـ syscall واحد" },
              ].map((x) => (
                <li key={x.point} className="flex items-start gap-2">
                  <ArrowLeft className="w-3 h-3 text-accent mt-0.5 shrink-0" />
                  <span>
                    <code className="text-primary mono">{x.point}</code>
                    <span className="text-muted-foreground mx-1">—</span>
                    <span className="ar">{x.desc}</span>
                  </span>
                </li>
              ))}
            </ul>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
