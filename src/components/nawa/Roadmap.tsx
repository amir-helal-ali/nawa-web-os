"use client";

import { motion } from "framer-motion";
import {
  Circle,
  CheckCircle2,
  CircleDot,
  Sparkles,
  Code2,
  Database,
  Shield,
  Rocket,
  Globe,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type PhaseStatus = "done" | "active" | "next" | "planned";

const PHASES: Array<{
  id: number;
  status: PhaseStatus;
  name: string;
  nameEn: string;
  duration: string;
  icon: typeof Code2;
  goal: string;
  goalEn: string;
  deliverables: string[];
  icon_color: string;
}> = [
  {
    id: 1,
    status: "done",
    name: "الأساس",
    nameEn: "Foundation",
    duration: "Q1 2026 · 8 weeks",
    icon: Code2,
    goal: "تأسيس النواة والمحرك الأساسي",
    goalEn: "Establish the kernel and core engine",
    deliverables: [
      "Rust workspace + CI/CD pipeline",
      "Zero-copy io_uring kernel",
      "HTTP/1.1 + HTTP/3 server",
      "Basic router with type-safe handlers",
      "Logging + structured errors",
    ],
    icon_color: "primary",
  },
  {
    id: 2,
    status: "active",
    name: "قاعدة البيانات",
    nameEn: "Database",
    duration: "Q2 2026 · 10 weeks",
    icon: Database,
    goal: "بناء NAWA-DB من الصفر",
    goalEn: "Build NAWA-DB from scratch",
    deliverables: [
      "MemTable (lock-free skip-list)",
      "SSTable writer + reader",
      "WAL for durability",
      "Bloom filters per SSTable",
      "Compaction strategy (L0-L7)",
      "ACID transactions",
      "Query planner (GET/PUT/DEL/SCAN)",
    ],
    icon_color: "primary",
  },
  {
    id: 3,
    status: "next",
    name: "محرك الواجهة",
    nameEn: "Frontend Engine",
    duration: "Q3 2026 · 8 weeks",
    icon: Globe,
    goal: "SSR + Islands + Streaming",
    goalEn: "SSR + Islands + Streaming",
    deliverables: [
      "Hypertext renderer (Rust → HTML)",
      "Island hydration runtime (3KB WASM)",
      "Streaming SSR with Suspense",
      "Edge cache with SWR invalidation",
      "Hot-reload dev server",
    ],
    icon_color: "accent",
  },
  {
    id: 4,
    status: "planned",
    name: "الأمان والتشغيل",
    nameEn: "Security & Ops",
    duration: "Q4 2026 · 6 weeks",
    icon: Shield,
    goal: "حماية وإدارة ذاتية",
    goalEn: "Protection and self-management",
    deliverables: [
      "Zero-trust auth (JWT + sessions)",
      "Built-in WAF + rate limiting",
      "Auto TLS via Let's Encrypt",
      "Self-healing + auto-restart",
      "Backup + restore pipeline",
      "Metrics + Prometheus endpoint",
    ],
    icon_color: "primary",
  },
  {
    id: 5,
    status: "planned",
    name: "الإطلاق",
    nameEn: "Launch",
    duration: "Q1 2027 · 4 weeks",
    icon: Rocket,
    goal: "إطلاق عام + marketplace",
    goalEn: "Public launch + marketplace",
    deliverables: [
      "Stable v1.0 release",
      "WASM plugin marketplace",
      "App templates (blog, SaaS, e-commerce)",
      "Documentation site + tutorials",
      "CLI tool (nawa create / deploy)",
    ],
    icon_color: "accent",
  },
];

export function Roadmap() {
  return (
    <section id="roadmap" className="relative py-24 lg:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="خارطة الطريق"
          eyebrowEn="Phased Roadmap"
          title="خمس مراحل، من النواة إلى الإطلاق"
          titleEn="Five phases — from kernel to launch"
          desc="خطة تنفيذ واقعية مقسّمة إلى خمس مراحل، كل مرحلة لها هدف واضح ومخرجات قابلة للقياس. النواة والأساس مكتملان، وقاعدة البيانات قيد التطوير النشط حالياً."
          descEn="A realistic execution plan in five phases, each with clear goals and measurable deliverables. Foundation is done, the database is under active development."
        />

        {/* Progress overview */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 p-5 rounded-2xl border border-border/60 bg-card/60"
        >
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-2">
              <Sparkles className="w-4 h-4 text-primary" />
              <span className="text-sm font-medium ar">التقدّم الكلي</span>
              <span className="text-xs text-muted-foreground mono">{`// overall progress`}</span>
            </div>
            <span className="text-sm mono text-primary font-bold">28%</span>
          </div>
          <div className="relative h-2 bg-muted rounded-full overflow-hidden">
            <motion.div
              initial={{ width: 0 }}
              whileInView={{ width: "28%" }}
              viewport={{ once: true }}
              transition={{ duration: 1.2, ease: "easeOut" }}
              className="absolute inset-y-0 left-0 bg-gradient-to-r from-primary via-primary to-accent rounded-full"
            />
          </div>
          <div className="mt-3 grid grid-cols-5 gap-2 text-center">
            {PHASES.map((p) => (
              <div key={p.id} className="text-[10px]">
                <div className="text-muted-foreground mono">P{p.id}</div>
                <div className={`font-medium ar ${p.status === "active" ? "text-primary" : ""}`}>
                  {p.name}
                </div>
              </div>
            ))}
          </div>
        </motion.div>

        {/* Phases timeline */}
        <div className="mt-12 relative">
          {/* Vertical line on desktop */}
          <div className="hidden lg:block absolute right-1/2 top-0 bottom-0 w-px bg-gradient-to-b from-primary/40 via-primary/20 to-transparent" />

          <div className="space-y-6 lg:space-y-0">
            {PHASES.map((phase, i) => (
              <PhaseRow key={phase.id} phase={phase} index={i} />
            ))}
          </div>
        </div>

        {/* CTA */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-16 relative overflow-hidden rounded-2xl border border-primary/30 bg-gradient-to-br from-primary/10 via-card to-accent/5 p-8 lg:p-10"
        >
          <div className="absolute inset-0 bg-dots opacity-20 pointer-events-none" />
          <div className="relative text-center">
            <Sparkles className="w-8 h-8 text-primary mx-auto mb-3" />
            <h3 className="text-xl lg:text-2xl font-bold ar">
              جاهز للانضمام إلى الرحلة؟
            </h3>
            <p className="text-sm text-muted-foreground mt-2 ar max-w-2xl mx-auto">
              الكود مفتوح المصدر، والمرحلة الأولى قابلة للتشغيل الآن. ساهم، اختبر، أو ابنِ
              تطبيقك التالي فوق NAWA.
            </p>
            <div className="mt-5 flex flex-wrap items-center justify-center gap-3">
              <Badge variant="outline" className="border-primary/40 text-primary mono">
                ✓ Phase 1 runnable
              </Badge>
              <Badge variant="outline" className="border-accent/40 text-accent mono">
                ⚡ Phase 2 in progress
              </Badge>
              <Badge variant="outline" className="mono">
                🗓 Phase 3-5 planned
              </Badge>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}

function PhaseRow({
  phase,
  index,
}: {
  phase: (typeof PHASES)[number];
  index: number;
}) {
  const isLeft = index % 2 === 0;
  const statusConfig = {
    done: { color: "text-primary", label: "مكتمل", en: "done", icon: CheckCircle2 },
    active: { color: "text-accent", label: "قيد التنفيذ", en: "active", icon: CircleDot },
    next: { color: "text-yellow-400", label: "التالي", en: "next", icon: Circle },
    planned: { color: "text-muted-foreground", label: "مخطط", en: "planned", icon: Circle },
  }[phase.status];

  return (
    <div className="lg:grid lg:grid-cols-2 lg:gap-8 relative">
      {/* Marker on the line (desktop) */}
      <div className="hidden lg:block absolute right-1/2 top-8 translate-x-1/2 z-10">
        <div className={`p-2 rounded-full bg-background border-2 ${phase.status === "active" ? "border-accent" : phase.status === "done" ? "border-primary" : "border-border"}`}>
          <phase.icon className={`w-5 h-5 ${statusConfig.color}`} strokeWidth={1.5} />
        </div>
      </div>

      {/* Empty cell to keep layout */}
      {isLeft ? (
        <>
          <PhaseCard phase={phase} statusConfig={statusConfig} />
          <div className="hidden lg:block" />
        </>
      ) : (
        <>
          <div className="hidden lg:block" />
          <PhaseCard phase={phase} statusConfig={statusConfig} />
        </>
      )}
    </div>
  );
}

function PhaseCard({
  phase,
  statusConfig,
}: {
  phase: (typeof PHASES)[number];
  statusConfig: { color: string; label: string; en: string; icon: typeof Circle };
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-50px" }}
      transition={{ duration: 0.5 }}
      className={`mb-6 lg:mb-0 p-5 rounded-2xl border bg-card/60 ${
        phase.status === "active"
          ? "border-accent/50 glow-teal"
          : phase.status === "done"
          ? "border-primary/40"
          : "border-border/60"
      }`}
    >
      {/* Phase header */}
      <div className="flex items-start justify-between gap-3 mb-3">
        <div className="flex items-center gap-3">
          <div className={`p-2 rounded-lg ${phase.icon_color === "primary" ? "bg-primary/15" : "bg-accent/15"}`}>
            <phase.icon className={`w-5 h-5 ${phase.icon_color === "primary" ? "text-primary" : "text-accent"}`} strokeWidth={1.5} />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h4 className="text-base font-semibold ar">{phase.name}</h4>
              <span className="text-xs text-muted-foreground mono">P{phase.id}</span>
            </div>
            <p className="text-[11px] text-muted-foreground mono">{phase.nameEn}</p>
          </div>
        </div>
        <Badge
          variant="outline"
          className={`mono text-[10px] ${statusConfig.color} border-current/40 shrink-0`}
        >
          <statusConfig.icon className="w-2.5 h-2.5 ml-1" />
          {statusConfig.label}
        </Badge>
      </div>

      {/* Duration */}
      <div className="text-xs text-muted-foreground mono mb-3">{phase.duration}</div>

      {/* Goal */}
      <div className="mb-3 p-3 rounded-lg bg-primary/5 border border-primary/20">
        <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-1">goal</div>
        <div className="text-sm font-medium ar">{phase.goal}</div>
        <div className="text-[11px] text-muted-foreground mt-0.5">{phase.goalEn}</div>
      </div>

      {/* Deliverables */}
      <div className="space-y-1.5">
        {phase.deliverables.map((d) => (
          <div key={d} className="flex items-start gap-2 text-xs">
            <CheckCircle2 className={`w-3.5 h-3.5 mt-0.5 shrink-0 ${phase.status === "done" ? "text-primary" : "text-muted-foreground/60"}`} />
            <span className={`ar ${phase.status === "done" ? "text-foreground/80" : "text-muted-foreground"}`}>
              {d}
            </span>
          </div>
        ))}
      </div>
    </motion.div>
  );
}
