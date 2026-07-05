"use client";

import { motion } from "framer-motion";
import {
  Hexagon,
  ArrowLeft,
  Zap,
  ShieldCheck,
  Feather,
  Cpu,
  Terminal,
} from "lucide-react";
import { Button } from "@/components/ui/button";

const STATS = [
  { value: "100%", label: "Rust", sub: "zero unsafe" },
  { value: "512MB", label: "RAM", sub: "minimum target" },
  { value: "2×", label: "Engines", sub: "backend + frontend" },
  { value: "0", label: "external deps", sub: "DB built-in" },
];

const PILLARS = [
  { icon: Zap, label: "القوة", en: "Power" },
  { icon: ShieldCheck, label: "الأمان", en: "Security" },
  { icon: Feather, label: "المرونة", en: "Flexibility" },
  { icon: Cpu, label: "كفاءة السيرفرات الضعيفة", en: "Low-spec efficient" },
];

export function Hero() {
  return (
    <section
      id="hero"
      className="relative min-h-screen flex items-center pt-16 overflow-hidden"
    >
      {/* Background layers */}
      <div className="absolute inset-0 bg-grid mask-fade-b pointer-events-none" />
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-1/4 -left-32 w-[480px] h-[480px] rounded-full bg-primary/15 blur-[120px]" />
        <div className="absolute bottom-1/4 -right-32 w-[480px] h-[480px] rounded-full bg-accent/10 blur-[120px]" />
      </div>

      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 w-full py-12 lg:py-20">
        <div className="grid lg:grid-cols-12 gap-10 lg:gap-16 items-center">
          {/* Left content */}
          <div className="lg:col-span-7">
            {/* Eyebrow */}
            <motion.div
              initial={{ opacity: 0, y: 16 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5 }}
              className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-primary/30 bg-primary/10 text-primary text-xs font-medium mb-6"
            >
              <span className="relative flex h-2 w-2">
                <span className="absolute inline-flex h-full w-full rounded-full bg-primary opacity-75 animate-nawa-pulse" />
                <span className="relative inline-flex rounded-full h-2 w-2 bg-primary" />
              </span>
              <span className="ar">نظام تشغيل ويب ثوري · مكتوب بالكامل بـ Rust</span>
              <span className="opacity-50">·</span>
              <span className="mono text-[10px]">v0.1.0-alpha</span>
            </motion.div>

            {/* Main title */}
            <motion.h1
              initial={{ opacity: 0, y: 24 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.05 }}
              className="text-5xl sm:text-6xl lg:text-7xl font-bold tracking-tight leading-[1.05]"
            >
              <span className="text-gradient-amber">NAWA</span>
              <span className="block mt-2 text-3xl sm:text-4xl lg:text-5xl ar">
                نظام تشغيل الويب
              </span>
            </motion.h1>

            {/* Subtitle */}
            <motion.p
              initial={{ opacity: 0, y: 24 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.15 }}
              className="mt-6 text-lg sm:text-xl text-muted-foreground leading-relaxed max-w-2xl ar"
            >
              منصة تطبيقات ويب متكاملة تعمل عبر Docker، مبنية بمحركين — محرك خلفية
              بمعدل إدخال/إخراج صفر-نسخ (<span className="mono text-primary">zero-copy</span>) ومحرك واجهة
              بمعمارية الجزر (<span className="mono text-primary">islands</span>) — مع قاعدة بيانات KV/Document مدمجة بالكامل.
            </motion.p>

            <motion.p
              initial={{ opacity: 0, y: 24 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.25 }}
              className="mt-3 text-base text-muted-foreground/80 max-w-2xl"
            >
              A complete Web App Platform — dual Rust engines, zero-copy kernel,
              built-in KV/Document database, optimized for 512MB-RAM servers.
            </motion.p>

            {/* CTAs */}
            <motion.div
              initial={{ opacity: 0, y: 24 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.35 }}
              className="mt-8 flex flex-wrap gap-3"
            >
              <Button
                size="lg"
                className="bg-primary hover:bg-primary/90 text-primary-foreground group"
                onClick={() =>
                  document.getElementById("architecture")?.scrollIntoView({ behavior: "smooth" })
                }
              >
                <span className="ar">استكشف المعمارية</span>
                <ArrowLeft className="w-4 h-4 mr-1 group-hover:-translate-x-1 transition-transform" />
              </Button>
              <Button
                size="lg"
                variant="outline"
                className="border-primary/40 hover:border-primary hover:bg-primary/10"
                onClick={() =>
                  document.getElementById("database")?.scrollIntoView({ behavior: "smooth" })
                }
              >
                <Terminal className="w-4 h-4 ml-1" />
                <span className="ar">جرّب قاعدة البيانات</span>
              </Button>
            </motion.div>

            {/* Stats */}
            <motion.div
              initial={{ opacity: 0, y: 24 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.45 }}
              className="mt-12 grid grid-cols-2 sm:grid-cols-4 gap-px bg-border/40 rounded-xl overflow-hidden border border-border/40"
            >
              {STATS.map((s) => (
                <div key={s.label} className="bg-card p-4">
                  <div className="text-2xl font-bold text-primary mono">{s.value}</div>
                  <div className="text-xs text-foreground mt-1">{s.label}</div>
                  <div className="text-[10px] text-muted-foreground mt-0.5">{s.sub}</div>
                </div>
              ))}
            </motion.div>
          </div>

          {/* Right visual */}
          <motion.div
            initial={{ opacity: 0, scale: 0.92 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.8, delay: 0.3 }}
            className="lg:col-span-5 relative"
          >
            <HeroVisual />
          </motion.div>
        </div>

        {/* Pillars */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.6 }}
          className="mt-16 lg:mt-20 grid grid-cols-2 lg:grid-cols-4 gap-3"
        >
          {PILLARS.map((p, i) => (
            <motion.div
              key={p.en}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.4, delay: 0.65 + i * 0.08 }}
              className="group relative p-4 rounded-xl border border-border/60 bg-card/40 hover:bg-card hover:border-primary/40 transition-all"
            >
              <p.icon className="w-6 h-6 text-primary mb-2 group-hover:scale-110 transition-transform" strokeWidth={1.5} />
              <div className="text-sm font-medium ar">{p.label}</div>
              <div className="text-[11px] text-muted-foreground mt-0.5">{p.en}</div>
            </motion.div>
          ))}
        </motion.div>
      </div>
    </section>
  );
}

function HeroVisual() {
  return (
    <div className="relative aspect-square max-w-md mx-auto">
      {/* Outer ring */}
      <div className="absolute inset-0 rounded-full border border-primary/20 animate-nawa-pulse" />
      <div className="absolute inset-8 rounded-full border border-primary/15" />
      <div className="absolute inset-16 rounded-full border border-accent/15" />

      {/* Scanning line */}
      <div className="absolute inset-0 rounded-full overflow-hidden">
        <div className="absolute inset-y-0 -left-full w-1/2 bg-gradient-to-r from-transparent via-primary/20 to-transparent animate-nawa-scan" />
      </div>

      {/* Core */}
      <div className="absolute inset-0 grid place-items-center">
        <motion.div
          animate={{ y: [0, -6, 0] }}
          transition={{ duration: 4, repeat: Infinity, ease: "easeInOut" }}
          className="relative w-32 h-32 grid place-items-center"
        >
          <div className="absolute inset-0 bg-primary/20 blur-2xl rounded-full" />
          <div className="absolute inset-0 rounded-3xl bg-gradient-to-br from-primary/30 via-primary/10 to-accent/20 border border-primary/40 glow-amber" />
          <Hexagon className="w-16 h-16 text-primary relative z-10" strokeWidth={1} />
          <div className="absolute inset-0 grid place-items-center">
            <span className="text-3xl font-bold text-primary-foreground bg-primary/90 backdrop-blur rounded-md w-12 h-12 grid place-items-center ar">
              ن
            </span>
          </div>
        </motion.div>
      </div>

      {/* Orbiting nodes */}
      {[0, 120, 240].map((deg, i) => (
        <div
          key={deg}
          className="absolute inset-0 animate-nawa-float"
          style={{ animationDelay: `${i * 0.6}s` }}
        >
          <div
            className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
            style={{ transform: `rotate(${deg}deg) translateY(-180px)` }}
          >
            <div
              className="px-3 py-1.5 rounded-md border border-border bg-card/80 backdrop-blur text-xs mono"
              style={{ transform: `rotate(-${deg}deg)` }}
            >
              {(["io_uring", "WASM", "SSR"] as const)[i]}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}
