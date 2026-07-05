"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  MemoryStick,
  Cpu,
  Database,
  Layers,
  Zap,
  Box,
  Server,
  Info,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type Layout = "nawa" | "traditional";

type Segment = {
  name: string;
  nameAr: string;
  size: number; // MB
  color: string;
  desc: string;
  icon: typeof Cpu;
};

const LAYOUTS: Record<Layout, { title: string; titleAr: string; total: number; segments: Segment[] }> = {
  nawa: {
    title: "NAWA Memory Layout",
    titleAr: "تخطيط ذاكرة NAWA",
    total: 47,
    segments: [
      { name: "Binary + code", nameAr: "الكود + الثنائي", size: 11, color: "oklch(0.72 0.19 47)", desc: "ثنائي musl static + JIT'd code", icon: Box },
      { name: "MemTable", nameAr: "MemTable", size: 8, color: "oklch(0.78 0.16 165)", desc: "lock-free skip-list للكتابات الساخنة", icon: Database },
      { name: "SSTable mmap", nameAr: "SSTable mmap", size: 12, color: "oklch(0.65 0.20 300)", desc: "صفحات mmap'd من الـ SSTables", icon: Layers },
      { name: "Page cache", nameAr: "Page cache", size: 6, color: "oklch(0.80 0.15 90)", desc: "kernel page cache للملفات", icon: MemoryStick },
      { name: "Connection pool", nameAr: "Connection pool", size: 4, color: "oklch(0.70 0.20 5)", desc: "HTTP/3 connections + buffers", icon: Server },
      { name: "WASM runtime", nameAr: "WASM runtime", size: 3, color: "oklch(0.65 0.18 200)", desc: "wasmtime instance pools", icon: Cpu },
      { name: "Stack + heap", nameAr: "Stack + heap", size: 3, color: "oklch(0.55 0.10 60)", desc: "thread stacks + allocations", icon: Zap },
    ],
  },
  traditional: {
    title: "Traditional Stack Memory",
    titleAr: "تخطيط ذاكرة Stack تقليدي",
    total: 360,
    segments: [
      { name: "Node.js runtime", nameAr: "Node.js runtime", size: 80, color: "oklch(0.65 0.18 140)", desc: "V8 engine + JIT + GC overhead", icon: Cpu },
      { name: "PostgreSQL", nameAr: "PostgreSQL", size: 120, color: "oklch(0.55 0.18 200)", desc: "shared_buffers + work_mem + wal", icon: Database },
      { name: "Redis", nameAr: "Redis", size: 50, color: "oklch(0.65 0.22 25)", desc: "in-memory cache + replication buffers", icon: MemoryStick },
      { name: "Nginx", nameAr: "Nginx", size: 30, color: "oklch(0.70 0.18 50)", desc: "worker processes + connection pool", icon: Server },
      { name: "V8 GC + JIT", nameAr: "V8 GC + JIT", size: 40, color: "oklch(0.55 0.20 25)", desc: "garbage collector + JIT compiled code", icon: Zap },
      { name: "App code + deps", nameAr: "App code + deps", size: 30, color: "oklch(0.55 0.10 60)", desc: "node_modules + app logic + ORM", icon: Box },
      { name: "Connection pools", nameAr: "Connection pools", size: 10, color: "oklch(0.55 0.18 300)", desc: "pg-pool + redis-pool + http agents", icon: Layers },
    ],
  },
};

export function MemoryLayout() {
  const [layout, setLayout] = useState<Layout>("nawa");
  const [hoveredSeg, setHoveredSeg] = useState<number | null>(null);
  const data = LAYOUTS[layout];
  const isNawa = layout === "nawa";

  return (
    <section id="memory" className="relative py-24 lg:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="تخطيط الذاكرة"
          eyebrowEn="Memory Layout"
          title="كل بايت محسوب"
          titleEn="Every byte accounted for"
          desc="انظر بصرياً كيف تُقسَّم الذاكرة في NAWA مقابل stack تقليدي. NAWA يستخدم 47MB فقط، بينما stack تقليدي يستهلك 360MB — لنفس التطبيق. الفارق ليس تحسيناً، بل إعادة تصميم."
          descEn="Visualize how memory is divided in NAWA vs a traditional stack. NAWA uses 47MB; traditional uses 360MB — for the same app."
        />

        {/* Toggle */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 flex justify-center"
        >
          <div className="inline-flex p-1 rounded-lg bg-card/60 border border-border/60">
            <button
              onClick={() => setLayout("nawa")}
              className={`px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2 ${
                isNawa ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground"
              }`}
            >
              <Zap className="w-3.5 h-3.5" />
              <span className="ar">NAWA · 47MB</span>
            </button>
            <button
              onClick={() => setLayout("traditional")}
              className={`px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2 ${
                !isNawa ? "bg-destructive text-destructive-foreground" : "text-muted-foreground hover:text-foreground"
              }`}
            >
              <Layers className="w-3.5 h-3.5" />
              <span className="ar">Traditional · 360MB</span>
            </button>
          </div>
        </motion.div>

        {/* Visual memory bar */}
        <motion.div
          key={layout}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.4 }}
          className="mt-8"
        >
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <MemoryStick className={`w-4 h-4 ${isNawa ? "text-primary" : "text-destructive"}`} />
              <span className="text-sm font-medium ar">{data.titleAr}</span>
              <span className="text-xs text-muted-foreground mono">{data.title}</span>
            </div>
            <div className={`text-2xl font-bold mono ${isNawa ? "text-primary" : "text-destructive"}`}>
              {data.total}MB
            </div>
          </div>

          {/* Stacked bar */}
          <div className="relative h-16 rounded-xl overflow-hidden border border-border/60 bg-card/40 flex">
            {data.segments.map((seg, i) => {
              const pct = (seg.size / data.total) * 100;
              const isHovered = hoveredSeg === i;
              return (
                <motion.div
                  key={seg.name}
                  initial={{ width: 0 }}
                  animate={{ width: `${pct}%` }}
                  transition={{ duration: 0.6, delay: i * 0.08, ease: "easeOut" }}
                  onMouseEnter={() => setHoveredSeg(i)}
                  onMouseLeave={() => setHoveredSeg(null)}
                  className="relative h-full group cursor-pointer transition-all"
                  style={{ background: seg.color }}
                  title={`${seg.name}: ${seg.size}MB`}
                >
                  {/* Label inside if wide enough */}
                  {pct > 8 && (
                    <div className="absolute inset-0 flex flex-col items-center justify-center text-center px-1">
                      <seg.icon className="w-3 h-3 text-white/90 mb-0.5" strokeWidth={1.5} />
                      <span className="text-[9px] mono text-white font-bold">{seg.size}MB</span>
                    </div>
                  )}
                  {/* Hover glow */}
                  {isHovered && (
                    <motion.div
                      initial={{ opacity: 0 }}
                      animate={{ opacity: 1 }}
                      className="absolute inset-0 bg-white/20"
                    />
                  )}
                </motion.div>
              );
            })}
          </div>

          {/* Scale markers */}
          <div className="flex justify-between mt-2 text-[10px] text-muted-foreground mono">
            <span>0MB</span>
            <span>{Math.round(data.total / 4)}MB</span>
            <span>{Math.round(data.total / 2)}MB</span>
            <span>{Math.round((data.total * 3) / 4)}MB</span>
            <span>{data.total}MB</span>
          </div>
        </motion.div>

        {/* Segment breakdown */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-8 grid sm:grid-cols-2 lg:grid-cols-4 gap-3"
        >
          {data.segments.map((seg, i) => {
            const isHovered = hoveredSeg === i;
            return (
              <motion.div
                key={seg.name}
                onMouseEnter={() => setHoveredSeg(i)}
                onMouseLeave={() => setHoveredSeg(null)}
                animate={{
                  scale: isHovered ? 1.03 : 1,
                  borderColor: isHovered ? seg.color + "80" : "oklch(0.28 0.01 60 / 60%)",
                }}
                className="p-4 rounded-xl border bg-card/60 cursor-pointer"
                style={{ borderColor: isHovered ? seg.color + "80" : undefined }}
              >
                <div className="flex items-start justify-between mb-2">
                  <div
                    className="p-1.5 rounded"
                    style={{ background: seg.color + "20", color: seg.color }}
                  >
                    <seg.icon className="w-4 h-4" strokeWidth={1.5} />
                  </div>
                  <span className="text-base font-bold mono" style={{ color: seg.color }}>
                    {seg.size}MB
                  </span>
                </div>
                <div className="text-sm font-medium ar">{seg.nameAr}</div>
                <div className="text-[10px] text-muted-foreground mono mt-0.5">{seg.name}</div>
                <p className="text-[11px] text-muted-foreground mt-2 leading-relaxed ar">{seg.desc}</p>
                <div className="mt-2 pt-2 border-t border-border/40 text-[10px] mono text-muted-foreground">
                  {((seg.size / data.total) * 100).toFixed(1)}% of total
                </div>
              </motion.div>
            );
          })}
        </motion.div>

        {/* Comparison insight */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-12 grid lg:grid-cols-3 gap-4"
        >
          <div className="p-5 rounded-2xl border border-primary/30 bg-primary/5">
            <div className="flex items-center gap-2 mb-2">
              <Zap className="w-4 h-4 text-primary" />
              <h4 className="text-sm font-semibold ar">لماذا NAWA أقل؟</h4>
            </div>
            <ul className="space-y-1.5 text-xs text-muted-foreground">
              <li className="flex items-start gap-2">
                <span className="text-primary">▸</span>
                <span className="ar">ثنائي واحد بدل 5 خدمات</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-primary">▸</span>
                <span className="ar">لا V8 runtime ولا GC</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-primary">▸</span>
                <span className="ar">mmap بدل read() buffers</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-primary">▸</span>
                <span className="ar">لا connection pools متعددة</span>
              </li>
            </ul>
          </div>
          <div className="p-5 rounded-2xl border border-border/60 bg-card/60">
            <div className="flex items-center gap-2 mb-2">
              <Info className="w-4 h-4 text-accent" />
              <h4 className="text-sm font-semibold ar">المفاضلة</h4>
            </div>
            <p className="text-xs text-muted-foreground leading-relaxed ar">
              NAWA يضحي بـ <span className="text-foreground">SQL الديناميكي</span> و
              <span className="text-foreground"> hot reload للـ schema</span> مقابل
              <span className="text-primary"> 7.6× أقل ذاكرة</span>. للمشاريع التي تحتاج مرونة SQL
              كاملة، PostgreSQL قد يكون أنسب — لكن مع تكلفة 7× أعلى.
            </p>
          </div>
          <div className="p-5 rounded-2xl border border-accent/30 bg-accent/5">
            <div className="flex items-center gap-2 mb-2">
              <Cpu className="w-4 h-4 text-accent" />
              <h4 className="text-sm font-semibold ar">على 512MB VPS</h4>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between text-xs">
                <span className="text-muted-foreground ar">NAWA يستهلك</span>
                <span className="text-primary mono font-bold">9%</span>
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-muted-foreground ar">يتبقى لتطبيقك</span>
                <span className="text-accent mono font-bold">465MB</span>
              </div>
              <div className="flex justify-between text-xs pt-2 border-t border-border/40">
                <span className="text-muted-foreground ar">Traditional يستهلك</span>
                <span className="text-destructive mono font-bold">70%</span>
              </div>
              <div className="flex justify-between text-xs">
                <span className="text-muted-foreground ar">يتبقى لتطبيقك</span>
                <span className="text-destructive mono font-bold">152MB</span>
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
