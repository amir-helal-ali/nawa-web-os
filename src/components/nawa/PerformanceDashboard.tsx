"use client";

import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import {
  Activity,
  Cpu,
  HardDrive,
  Zap,
  TrendingDown,
  Gauge,
  Server,
  ArrowDown,
  Play,
  Pause,
  RotateCcw,
} from "lucide-react";
import {
  AreaChart,
  Area,
  LineChart,
  Line,
  ResponsiveContainer,
  XAxis,
  YAxis,
  Tooltip,
  CartesianGrid,
  BarChart,
  Bar,
  Cell,
} from "recharts";
import { SectionHeader } from "./Concept";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

type SeriesPoint = { t: number; nawa: number; traditional?: number };

const COMPARISON_BARS = [
  { name: "Cold Start", nawa: 0.18, traditional: 4.2, unit: "s" },
  { name: "Req p99", nawa: 0.42, traditional: 12.8, unit: "ms" },
  { name: "DB Read", nawa: 0.09, traditional: 1.8, unit: "ms" },
  { name: "SSR Render", nawa: 1.2, traditional: 8.5, unit: "ms" },
  { name: "Idle RAM", nawa: 47, traditional: 360, unit: "MB" },
];

const METRICS = [
  { icon: Cpu, label: "RAM Usage", value: "47MB", sub: "of 512MB target", trend: "↓ 94%", color: "primary" },
  { icon: Zap, label: "Throughput", value: "8.4k rps", sub: "on 1 vCPU", trend: "↑ 12×", color: "accent" },
  { icon: HardDrive, label: "Binary Size", value: "11MB", sub: "static + musl", trend: "↓ 87%", color: "primary" },
  { icon: Gauge, label: "Cold Start", value: "180ms", sub: "container boot", trend: "↓ 23×", color: "accent" },
];

export function PerformanceDashboard() {
  const [ramData, setRamData] = useState<SeriesPoint[]>(() =>
    Array.from({ length: 30 }, (_, i) => ({
      t: i,
      nawa: 38 + Math.sin(i * 0.4) * 8,
      traditional: 280 + Math.sin(i * 0.3) * 30,
    }))
  );
  const [rpsData, setRpsData] = useState<SeriesPoint[]>(() =>
    Array.from({ length: 20 }, (_, i) => ({
      t: i,
      nawa: 8200 + Math.sin(i * 0.5) * 600,
    }))
  );
  const [running, setRunning] = useState(true);
  const [liveRam, setLiveRam] = useState(47);
  const [liveRps, setLiveRps] = useState(8400);
  const [tick, setTick] = useState(0);

  // Live update loop
  useEffect(() => {
    if (!running) return;
    const id = setInterval(() => {
      const newRam = 38 + Math.random() * 18;
      const newRps = 8200 + Math.random() * 600;
      setLiveRam(newRam);
      setLiveRps(newRps);
      setTick((t) => t + 1);

      setRamData((prev) => {
        const next = [...prev.slice(1), {
          t: prev[prev.length - 1].t + 1,
          nawa: newRam,
          traditional: 280 + Math.random() * 40,
        }];
        return next;
      });
      setRpsData((prev) => {
        const next = [...prev.slice(1), {
          t: prev[prev.length - 1].t + 1,
          nawa: newRps,
        }];
        return next;
      });
    }, 1500);
    return () => clearInterval(id);
  }, [running]);

  const reset = () => {
    setRamData(Array.from({ length: 30 }, (_, i) => ({
      t: i,
      nawa: 38 + Math.sin(i * 0.4) * 8 + Math.random() * 4,
      traditional: 280 + Math.sin(i * 0.3) * 30 + Math.random() * 20,
    })));
    setRpsData(Array.from({ length: 20 }, (_, i) => ({
      t: i,
      nawa: 8200 + Math.sin(i * 0.5) * 600 + Math.random() * 300,
    })));
  };

  return (
    <section id="performance" className="relative py-24 lg:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="الأداء على السيرفرات الضعيفة"
          eyebrowEn="Low-Spec Performance"
          title="يعمل بكفاءة على 512MB RAM"
          titleEn="Runs efficiently on 512MB RAM"
          desc="الهدف الأساسي: تشغيل تطبيق إنتاجي كامل على VPS اقتصادي بـ 512MB RAM وvCPU واحد. NAWA يستخدم أقل من 10% من هذه الموارد، مما يترك الباقي لتطبيق المستخدم."
          descEn="The core goal: run a full production app on a cheap VPS with 512MB RAM and a single vCPU. NAWA uses less than 10% of these resources — the rest is yours."
        />

        {/* Live controls bar */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 flex items-center justify-between p-3 rounded-xl border border-border/60 bg-card/60"
        >
          <div className="flex items-center gap-2">
            <Activity className={`w-4 h-4 ${running ? "text-accent animate-nawa-pulse" : "text-muted-foreground"}`} />
            <span className="text-sm font-medium ar">المختبر الحي</span>
            <Badge variant="outline" className="mono text-[10px]">
              tick: {tick}
            </Badge>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setRunning((r) => !r)}
              className="h-7 px-2 text-xs"
            >
              {running ? <Pause className="w-3 h-3" /> : <Play className="w-3 h-3" />}
              <span className="mr-1 ar">{running ? "إيقاف" : "تشغيل"}</span>
            </Button>
            <Button variant="ghost" size="sm" onClick={reset} className="h-7 px-2 text-xs">
              <RotateCcw className="w-3 h-3" />
            </Button>
          </div>
        </motion.div>

        {/* Live metrics */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-4 grid grid-cols-2 lg:grid-cols-4 gap-3"
        >
          {METRICS.map((m, i) => (
            <motion.div
              key={m.label}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: i * 0.08 }}
              className="relative p-5 rounded-xl border border-border/60 bg-card/60 overflow-hidden"
            >
              <div className="absolute -top-8 -right-8 w-24 h-24 rounded-full bg-primary/10 blur-2xl pointer-events-none" />
              <div className="flex items-start justify-between mb-3 relative">
                <div className={`p-2 rounded-lg ${m.color === "primary" ? "bg-primary/15 text-primary" : "bg-accent/15 text-accent"}`}>
                  <m.icon className="w-5 h-5" strokeWidth={1.5} />
                </div>
                <span className={`text-[10px] mono ${m.color === "primary" ? "text-primary" : "text-accent"} flex items-center gap-1`}>
                  <TrendingDown className="w-3 h-3" />
                  {m.trend}
                </span>
              </div>
              <div className="text-2xl font-bold mono">{m.value}</div>
              <div className="text-sm font-medium mt-0.5">{m.label}</div>
              <div className="text-[11px] text-muted-foreground mt-0.5">{m.sub}</div>
            </motion.div>
          ))}
        </motion.div>

        {/* Charts grid */}
        <div className="mt-4 grid lg:grid-cols-3 gap-4">
          {/* RAM comparison - big */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="lg:col-span-2 p-5 rounded-2xl border border-border/60 bg-card/60"
          >
            <div className="flex items-start justify-between mb-4">
              <div>
                <h4 className="text-sm font-semibold flex items-center gap-2">
                  <Cpu className="w-4 h-4 text-primary" />
                  استهلاك الذاكرة — حي
                </h4>
                <p className="text-[11px] text-muted-foreground mt-0.5">
                  Live memory footprint — NAWA vs traditional Node+PG+Redis
                </p>
              </div>
              <div className="flex items-center gap-3 text-xs">
                <span className="flex items-center gap-1.5">
                  <span className="w-2 h-2 rounded-full bg-primary" />
                  NAWA
                </span>
                <span className="flex items-center gap-1.5">
                  <span className="w-2 h-2 rounded-full bg-destructive/70" />
                  Traditional
                </span>
              </div>
            </div>
            <div className="h-56">
              <ResponsiveContainer width="100%" height="100%">
                <AreaChart data={ramData}>
                  <defs>
                    <linearGradient id="grad-nawa" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="oklch(0.72 0.19 47)" stopOpacity={0.5} />
                      <stop offset="95%" stopColor="oklch(0.72 0.19 47)" stopOpacity={0} />
                    </linearGradient>
                    <linearGradient id="grad-trad" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="oklch(0.65 0.22 25)" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="oklch(0.65 0.22 25)" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="oklch(0.28 0.01 60)" strokeOpacity={0.3} />
                  <XAxis dataKey="t" tick={{ fontSize: 10, fill: "oklch(0.65 0.01 60)" }} />
                  <YAxis tick={{ fontSize: 10, fill: "oklch(0.65 0.01 60)" }} unit="MB" />
                  <Tooltip
                    contentStyle={{
                      background: "oklch(0.135 0.008 60)",
                      border: "1px solid oklch(0.28 0.01 60)",
                      borderRadius: 8,
                      fontSize: 12,
                    }}
                    labelStyle={{ color: "oklch(0.65 0.01 60)" }}
                  />
                  <Area
                    type="monotone"
                    dataKey="traditional"
                    stroke="oklch(0.65 0.22 25)"
                    strokeWidth={1.5}
                    fill="url(#grad-trad)"
                    name="Traditional"
                    isAnimationActive={false}
                  />
                  <Area
                    type="monotone"
                    dataKey="nawa"
                    stroke="oklch(0.72 0.19 47)"
                    strokeWidth={2}
                    fill="url(#grad-nawa)"
                    name="NAWA"
                    isAnimationActive={false}
                  />
                </AreaChart>
              </ResponsiveContainer>
            </div>
            <div className="mt-3 flex items-center gap-2 text-xs text-muted-foreground">
              <Server className="w-3 h-3" />
              <span className="ar">
                NAWA يستخدم ~<span className="text-primary mono font-medium">{Math.round(liveRam)}MB</span> فقط — أي
                <span className="text-primary mono font-medium ml-1">{Math.round((liveRam / 512) * 100)}%</span> من سيرفر 512MB.
              </span>
            </div>
          </motion.div>

          {/* Throughput gauge */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5, delay: 0.1 }}
            className="p-5 rounded-2xl border border-border/60 bg-card/60 flex flex-col"
          >
            <h4 className="text-sm font-semibold flex items-center gap-2 mb-4">
              <Activity className="w-4 h-4 text-accent" />
              الإنتاجية الحية
            </h4>
            <div className="flex-1 grid place-items-center">
              <div className="relative">
                <svg width="160" height="160" viewBox="0 0 160 160">
                  <circle
                    cx="80"
                    cy="80"
                    r="64"
                    fill="none"
                    stroke="oklch(0.28 0.01 60)"
                    strokeWidth="10"
                  />
                  <motion.circle
                    cx="80"
                    cy="80"
                    r="64"
                    fill="none"
                    stroke="oklch(0.78 0.16 165)"
                    strokeWidth="10"
                    strokeLinecap="round"
                    transform="rotate(-90 80 80)"
                    strokeDasharray={2 * Math.PI * 64}
                    initial={{ strokeDashoffset: 2 * Math.PI * 64 }}
                    animate={{
                      strokeDashoffset: 2 * Math.PI * 64 * (1 - Math.min(liveRps / 12000, 1)),
                    }}
                    transition={{ duration: 0.8, ease: "easeOut" }}
                  />
                </svg>
                <div className="absolute inset-0 grid place-items-center text-center">
                  <div>
                    <div className="text-2xl font-bold mono text-accent">
                      {(liveRps / 1000).toFixed(1)}k
                    </div>
                    <div className="text-[10px] text-muted-foreground">req/sec</div>
                  </div>
                </div>
              </div>
            </div>
            <div className="mt-3 text-center text-xs text-muted-foreground">
              <span className="ar">على سيرفر 1 vCPU · 512MB</span>
            </div>
          </motion.div>
        </div>

        {/* Latency comparison bars */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-4 grid lg:grid-cols-2 gap-4"
        >
          <div className="p-5 rounded-2xl border border-border/60 bg-card/60">
            <h4 className="text-sm font-semibold mb-4 flex items-center gap-2">
              <ArrowDown className="w-4 h-4 text-primary" />
              مقارنة الأداء — أقل أفضل
            </h4>
            <div className="h-56">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={COMPARISON_BARS} layout="vertical" margin={{ left: 30, right: 20 }}>
                  <CartesianGrid strokeDasharray="3 3" stroke="oklch(0.28 0.01 60)" strokeOpacity={0.3} horizontal={false} />
                  <XAxis type="number" tick={{ fontSize: 10, fill: "oklch(0.65 0.01 60)" }} />
                  <YAxis type="category" dataKey="name" tick={{ fontSize: 10, fill: "oklch(0.65 0.01 60)" }} width={80} />
                  <Tooltip
                    contentStyle={{
                      background: "oklch(0.135 0.008 60)",
                      border: "1px solid oklch(0.28 0.01 60)",
                      borderRadius: 8,
                      fontSize: 12,
                    }}
                    formatter={(value: any, name: any, props: any) => [`${value} ${props.payload.unit}`, name]}
                  />
                  <Bar dataKey="nawa" fill="oklch(0.72 0.19 47)" radius={[0, 4, 4, 0]} name="NAWA" />
                  <Bar dataKey="traditional" fill="oklch(0.65 0.22 25 / 60%)" radius={[0, 4, 4, 0]} name="Traditional" />
                </BarChart>
              </ResponsiveContainer>
            </div>
          </div>

          <div className="p-5 rounded-2xl border border-border/60 bg-card/60">
            <h4 className="text-sm font-semibold mb-4 flex items-center gap-2">
              <Activity className="w-4 h-4 text-accent" />
              إنتاجية NAWA المستمرة (rps)
            </h4>
            <div className="h-56">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={rpsData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="oklch(0.28 0.01 60)" strokeOpacity={0.3} />
                  <XAxis dataKey="t" tick={{ fontSize: 10, fill: "oklch(0.65 0.01 60)" }} />
                  <YAxis domain={[7000, 10000]} tick={{ fontSize: 10, fill: "oklch(0.65 0.01 60)" }} />
                  <Tooltip
                    contentStyle={{
                      background: "oklch(0.135 0.008 60)",
                      border: "1px solid oklch(0.28 0.01 60)",
                      borderRadius: 8,
                      fontSize: 12,
                    }}
                  />
                  <Line
                    type="monotone"
                    dataKey="nawa"
                    stroke="oklch(0.78 0.16 165)"
                    strokeWidth={2}
                    dot={false}
                    name="rps"
                    isAnimationActive={false}
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
          </div>
        </motion.div>

        {/* Benchmark table */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-8"
        >
          <h4 className="text-sm font-semibold mb-4 flex items-center gap-2">
            <Server className="w-4 h-4 text-primary" />
            جدول المقارنة الكامل
            <span className="text-xs text-muted-foreground mono ml-1">{`// NAWA vs traditional stack`}</span>
          </h4>
          <div className="rounded-2xl border border-border/60 overflow-hidden">
            <table className="w-full text-sm">
              <thead className="bg-card/60 border-b border-border/40">
                <tr>
                  <th className="text-right p-3 text-xs font-medium text-muted-foreground ar">المقياس</th>
                  <th className="text-right p-3 text-xs font-medium text-muted-foreground">NAWA Target</th>
                  <th className="text-right p-3 text-xs font-medium text-muted-foreground">Traditional</th>
                  <th className="text-right p-3 text-xs font-medium text-muted-foreground">Improvement</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-border/30">
                {[
                  ["Idle RAM", "< 50 MB", "~360 MB", "7.2×"],
                  ["Binary size", "< 15 MB", "~250 MB", "16.7×"],
                  ["Cold start", "< 200 ms", "~4 s", "20×"],
                  ["Req p99 latency", "< 1 ms", "~12 ms", "12×"],
                  ["Throughput (1 vCPU)", "> 8 k rps", "~700 rps", "11.4×"],
                  ["DB read latency", "< 100 µs", "~1.8 ms", "18×"],
                  ["SSR render time", "< 2 ms", "~8.5 ms", "4.25×"],
                  ["Container size", "< 20 MB", "~600 MB", "30×"],
                ].map(([metric, nawa, trad, improvement]) => (
                  <tr key={metric} className="hover:bg-primary/5">
                    <td className="p-3 ar">{metric}</td>
                    <td className="p-3 mono text-primary">{nawa}</td>
                    <td className="p-3 mono text-muted-foreground">{trad}</td>
                    <td className="p-3">
                      <Badge variant="outline" className="border-accent/40 text-accent mono text-[10px]">
                        {improvement}
                      </Badge>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
