"use client";

import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import {
  Activity,
  Eye,
  TrendingUp,
  AlertTriangle,
  Search,
  Gauge,
  Clock,
  Server,
  Zap,
  Database,
  Network,
  Cpu,
  HardDrive,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";
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
} from "recharts";

const PILLARS = [
  {
    icon: Eye,
    title: "Logs",
    titleAr: "السجلات",
    desc: "structured JSON logs مدمجة. كل request يولّد span كامل. لا تحتاج لإعداد logging.",
    color: "primary",
  },
  {
    icon: Activity,
    title: "Metrics",
    titleAr: "المقاييس",
    desc: "/metrics endpoint بـ Prometheus format. لا Grafana setup، لا exporters.",
    color: "accent",
  },
  {
    icon: Network,
    title: "Traces",
    titleAr: "التتبّع",
    desc: "OpenTelemetry traces مدمجة. كل DB query، كل HTTP call — موثّق تلقائياً.",
    color: "primary",
  },
  {
    icon: AlertTriangle,
    title: "Alerts",
    titleAr: "التنبيهات",
    desc: "قواعد تنبيه مدمجة: latency, error rate, memory. تكامل Slack/Discord/Email.",
    color: "accent",
  },
];

const SPANS = [
  { id: 1, name: "HTTP GET /api/users", duration: 1.42, color: "primary", level: 0, status: "ok" },
  { id: 2, name: "middleware.auth.verify", duration: 0.18, color: "accent", level: 1, status: "ok" },
  { id: 3, name: "router.match", duration: 0.05, color: "primary", level: 1, status: "ok" },
  { id: 4, name: "db.get(user:1001)", duration: 0.42, color: "accent", level: 1, status: "ok" },
  { id: 5, name: "mmap.read(sstable_03)", duration: 0.31, color: "primary", level: 2, status: "ok" },
  { id: 6, name: "bloom.check", duration: 0.02, color: "accent", level: 2, status: "ok" },
  { id: 7, name: "render.json", duration: 0.08, color: "primary", level: 1, status: "ok" },
  { id: 8, name: "stream.send", duration: 0.0, color: "accent", level: 1, status: "ok" },
];

export function Observability() {
  const [metrics, setMetrics] = useState({
    rps: 8432,
    p50: 0.18,
    p99: 0.42,
    errors: 0.02,
    cpu: 23,
    mem: 47,
    dbOps: 1240,
  });

  const [traceData, setTraceData] = useState(
    Array.from({ length: 20 }, (_, i) => ({
      t: i,
      rps: 8200 + Math.sin(i * 0.3) * 300,
      latency: 0.4 + Math.sin(i * 0.5) * 0.1,
    }))
  );

  useEffect(() => {
    const id = setInterval(() => {
      setMetrics({
        rps: 8200 + Math.floor(Math.random() * 600),
        p50: 0.15 + Math.random() * 0.08,
        p99: 0.35 + Math.random() * 0.2,
        errors: Math.random() * 0.05,
        cpu: 18 + Math.floor(Math.random() * 12),
        mem: 42 + Math.floor(Math.random() * 12),
        dbOps: 1100 + Math.floor(Math.random() * 300),
      });
      setTraceData((prev) =>
        [...prev.slice(1), {
          t: prev[prev.length - 1].t + 1,
          rps: 8200 + Math.random() * 600,
          latency: 0.3 + Math.random() * 0.2,
        }]
      );
    }, 1500);
    return () => clearInterval(id);
  }, []);

  return (
    <section id="observability" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-grid opacity-30 pointer-events-none" />
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="الرصد والمراقبة"
          eyebrowEn="Observability"
          title="مراقب افتراضياً · بدون إعداد"
          titleEn="Observable by default — zero setup"
          desc="NAWA يولّد logs + metrics + traces تلقائياً لكل request. لا تحتاج لتثبيت Prometheus، لا Grafana، لا Jaeger — كل شيء مدمج. فقط افتح /dashboard وراقب نظامك."
          descEn="NAWA generates logs + metrics + traces automatically for every request. No Prometheus, no Grafana, no Jaeger to install — everything is built-in."
        />

        {/* Pillars */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-12 grid sm:grid-cols-2 lg:grid-cols-4 gap-3"
        >
          {PILLARS.map((p) => (
            <div key={p.title} className="p-4 rounded-xl border border-border/60 bg-card/60">
              <div className={`p-2 rounded-lg w-fit mb-3 ${p.color === "primary" ? "bg-primary/15 text-primary" : "bg-accent/15 text-accent"}`}>
                <p.icon className="w-5 h-5" strokeWidth={1.5} />
              </div>
              <h4 className="text-sm font-semibold ar">{p.titleAr}</h4>
              <p className="text-[10px] text-muted-foreground mono mt-0.5">{p.title}</p>
              <p className="text-xs text-muted-foreground mt-2 leading-relaxed ar">{p.desc}</p>
            </div>
          ))}
        </motion.div>

        {/* Live metrics dashboard */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-8 rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]"
        >
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
            <div className="flex items-center gap-2">
              <Gauge className="w-4 h-4 text-primary" />
              <span className="text-sm font-medium mono">live dashboard</span>
              <Badge variant="outline" className="mono text-[10px] border-accent/40 text-accent">
                <span className="w-1.5 h-1.5 rounded-full bg-accent mr-1 animate-nawa-pulse" />
                live
              </Badge>
            </div>
            <span className="text-[10px] text-muted-foreground mono">/dashboard · refresh: 1.5s</span>
          </div>

          {/* Metrics grid */}
          <div className="grid grid-cols-2 lg:grid-cols-7 gap-px bg-border/40">
            <MetricCard icon={Activity} label="RPS" value={metrics.rps.toLocaleString()} sub="req/sec" color="primary" />
            <MetricCard icon={Clock} label="p50" value={`${metrics.p50.toFixed(2)}ms`} sub="latency" color="accent" />
            <MetricCard icon={Clock} label="p99" value={`${metrics.p99.toFixed(2)}ms`} sub="latency" color="primary" />
            <MetricCard icon={AlertTriangle} label="Errors" value={`${metrics.errors.toFixed(2)}%`} sub="error rate" color="accent" />
            <MetricCard icon={Cpu} label="CPU" value={`${metrics.cpu}%`} sub="1 vCPU" color="primary" />
            <MetricCard icon={HardDrive} label="RAM" value={`${metrics.mem}MB`} sub="of 512MB" color="accent" />
            <MetricCard icon={Database} label="DB ops" value={metrics.dbOps.toLocaleString()} sub="per sec" color="primary" />
          </div>

          {/* Charts */}
          <div className="grid lg:grid-cols-2 gap-px bg-border/40">
            {/* RPS chart */}
            <div className="bg-[#0d0c0a] p-4">
              <div className="flex items-center justify-between mb-3">
                <span className="text-xs font-medium ar">الإنتاجية (rps)</span>
                <Zap className="w-3.5 h-3.5 text-accent" />
              </div>
              <div className="h-32">
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart data={traceData}>
                    <defs>
                      <linearGradient id="grad-rps" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="5%" stopColor="oklch(0.72 0.19 47)" stopOpacity={0.5} />
                        <stop offset="95%" stopColor="oklch(0.72 0.19 47)" stopOpacity={0} />
                      </linearGradient>
                    </defs>
                    <YAxis hide domain={[7000, 10000]} />
                    <Area type="monotone" dataKey="rps" stroke="oklch(0.72 0.19 47)" strokeWidth={1.5} fill="url(#grad-rps)" isAnimationActive={false} />
                  </AreaChart>
                </ResponsiveContainer>
              </div>
            </div>
            {/* Latency chart */}
            <div className="bg-[#0d0c0a] p-4">
              <div className="flex items-center justify-between mb-3">
                <span className="text-xs font-medium ar">زمن الاستجابة (ms)</span>
                <Clock className="w-3.5 h-3.5 text-primary" />
              </div>
              <div className="h-32">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={traceData}>
                    <YAxis hide domain={[0.2, 0.7]} />
                    <Line type="monotone" dataKey="latency" stroke="oklch(0.78 0.16 165)" strokeWidth={1.5} dot={false} isAnimationActive={false} />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            </div>
          </div>
        </motion.div>

        {/* Distributed trace view */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-8 rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]"
        >
          <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
            <div className="flex items-center gap-2">
              <Search className="w-4 h-4 text-accent" />
              <span className="text-sm font-medium mono">distributed trace · GET /api/users/1001</span>
            </div>
            <span className="text-[10px] text-accent mono">total: 1.42ms</span>
          </div>
          <div className="p-4 space-y-1">
            {SPANS.map((span, i) => (
              <motion.div
                key={span.id}
                initial={{ opacity: 0, x: -10 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.3, delay: i * 0.05 }}
                className="flex items-center gap-2 text-xs"
                style={{ paddingRight: `${span.level * 2}rem` }}
              >
                <span className="text-muted-foreground/60 w-8 shrink-0">#{span.id}</span>
                <span
                  className={`w-2 h-2 rounded-full shrink-0 ${
                    span.color === "primary" ? "bg-primary" : "bg-accent"
                  }`}
                />
                <code className={`flex-1 truncate mono ${span.color === "primary" ? "text-primary" : "text-accent"}`}>
                  {span.name}
                </code>
                {/* Span bar */}
                <div className="w-32 h-4 bg-muted/30 rounded relative overflow-hidden">
                  <div
                    className={`absolute inset-y-0 rounded ${
                      span.color === "primary" ? "bg-primary/60" : "bg-accent/60"
                    }`}
                    style={{
                      left: `${(span.level * 8) + Math.random() * 10}%`,
                      width: `${Math.max((span.duration / 1.42) * 80, 4)}%`,
                    }}
                  />
                </div>
                <span className="mono text-muted-foreground w-12 text-right shrink-0">
                  {span.duration.toFixed(2)}ms
                </span>
              </motion.div>
            ))}
          </div>
          <div className="px-4 py-2 border-t border-border/40 bg-card/40 text-[10px] text-muted-foreground mono">
            ✓ 8 spans · 0 errors · trace id: 4f3a2b1c
          </div>
        </motion.div>

        {/* Built-in dashboards */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-8 grid sm:grid-cols-3 gap-3"
        >
          {[
            { path: "/dashboard", desc: "لوحة حية بكل المقاييس", icon: Gauge },
            { path: "/metrics", desc: "Prometheus endpoint", icon: TrendingUp },
            { path: "/traces/:id", desc: "تفاصيل trace مع waterfall", icon: Search },
          ].map((d) => (
            <div key={d.path} className="p-3 rounded-lg border border-border/60 bg-card/60 flex items-center gap-3">
              <d.icon className="w-4 h-4 text-accent shrink-0" />
              <div>
                <code className="text-xs mono text-primary">{d.path}</code>
                <div className="text-[10px] text-muted-foreground ar">{d.desc}</div>
              </div>
            </div>
          ))}
        </motion.div>
      </div>
    </section>
  );
}

function MetricCard({
  icon: Icon,
  label,
  value,
  sub,
  color,
}: {
  icon: typeof Activity;
  label: string;
  value: string;
  sub: string;
  color: "primary" | "accent";
}) {
  return (
    <div className="bg-card/60 p-3">
      <div className="flex items-center justify-between mb-1">
        <Icon className={`w-3.5 h-3.5 ${color === "primary" ? "text-primary" : "text-accent"}`} strokeWidth={1.5} />
        <span className="text-[9px] text-muted-foreground mono">{label}</span>
      </div>
      <div className={`text-lg font-bold mono ${color === "primary" ? "text-primary" : "text-accent"}`}>
        {value}
      </div>
      <div className="text-[9px] text-muted-foreground">{sub}</div>
    </div>
  );
}
