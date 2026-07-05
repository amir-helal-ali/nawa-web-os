"use client";

import { motion } from "framer-motion";
import {
  Rocket,
  Code2,
  Database,
  Shield,
  Globe,
  Star,
  Users,
  Zap,
  GitBranch,
  Heart,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type Milestone = {
  date: string;
  dateAr: string;
  title: string;
  titleAr: string;
  desc: string;
  descEn: string;
  icon: typeof Rocket;
  color: string;
  type: "release" | "community" | "technical" | "milestone";
  stats?: { label: string; value: string }[];
};

const MILESTONES: Milestone[] = [
  {
    date: "2026-04-01",
    dateAr: "أبريل 2026",
    title: "First Commit",
    titleAr: "أول commit",
    desc: "بدأ المشروع برؤية واضحة: نظام تشغيل ويب كامل بـ Rust، بدون تبعيات خارجية.",
    descEn: "Project started with a clear vision: a complete web OS in Rust, no external deps.",
    icon: GitBranch,
    color: "oklch(0.55 0.10 60)",
    type: "milestone",
  },
  {
    date: "2026-04-15",
    dateAr: "أبريل 2026",
    title: "Rust Workspace + CI/CD",
    titleAr: "Rust workspace + CI/CD",
    desc: "إعداد البنية الأساسية: workspace متعدد الـ crates، GitHub Actions، automated testing.",
    descEn: "Infrastructure setup: multi-crate workspace, GitHub Actions, automated testing.",
    icon: Code2,
    color: "oklch(0.72 0.19 47)",
    type: "technical",
  },
  {
    date: "2026-05-01",
    dateAr: "مايو 2026",
    title: "HTTP/3 + QUIC Server",
    titleAr: "خادم HTTP/3 + QUIC",
    desc: "أول خادم HTTP/3 يعمل بـ quinn + rustls. TLS 1.3 افتراضي، 0-RTT للجلسات العائدة.",
    descEn: "First HTTP/3 server on quinn + rustls. TLS 1.3 default, 0-RTT for resumed sessions.",
    icon: Zap,
    color: "oklch(0.78 0.16 165)",
    type: "technical",
    stats: [
      { label: "throughput", value: "8k rps" },
      { label: "p99", value: "0.42ms" },
    ],
  },
  {
    date: "2026-05-15",
    dateAr: "مايو 2026",
    title: "Zero-Copy Kernel",
    titleAr: "النواة صفر-النسخ",
    desc: "تم تركيب io_uring + mmap. أول benchmark أظهر 5× تحسّن في throughput.",
    descEn: "io_uring + mmap integrated. First benchmark showed 5× throughput improvement.",
    icon: Zap,
    color: "oklch(0.65 0.20 25)",
    type: "technical",
    stats: [
      { label: "improvement", value: "5×" },
      { label: "copies", value: "0" },
    ],
  },
  {
    date: "2026-06-01",
    dateAr: "يونيو 2026",
    title: "Type-Safe Router",
    titleAr: "موجِّه آمن الأنواع",
    desc: "proc-macros لتوجيه ثابت الأنواع. كل path params تُستخرج في compile-time.",
    descEn: "Proc-macros for type-safe routing. All path params extracted at compile-time.",
    icon: Code2,
    color: "oklch(0.65 0.20 300)",
    type: "technical",
  },
  {
    date: "2026-06-10",
    dateAr: "يونيو 2026",
    title: "NAWA-DB Prototype",
    titleAr: "نموذج NAWA-DB",
    desc: "أول نسخة من قاعدة البيانات المدمجة: KV + Document، LSM tree، mmap.",
    descEn: "First version of built-in database: KV + Document, LSM tree, mmap.",
    icon: Database,
    color: "oklch(0.65 0.18 50)",
    type: "technical",
    stats: [
      { label: "read latency", value: "92μs" },
      { label: "write latency", value: "180μs" },
    ],
  },
  {
    date: "2026-06-20",
    dateAr: "يونيو 2026",
    title: "1k GitHub Stars",
    titleAr: "ألف نجمة على GitHub",
    desc: "أول 1000 star في أسبوعين بعد الإعلان الأولي على Reddit و Hacker News.",
    descEn: "First 1000 stars in two weeks after initial announcement on Reddit and HN.",
    icon: Star,
    color: "oklch(0.80 0.15 90)",
    type: "community",
    stats: [
      { label: "stars", value: "1k" },
      { label: "forks", value: "120" },
    ],
  },
  {
    date: "2026-07-01",
    dateAr: "يوليو 2026",
    title: "WASM Sandbox",
    titleAr: "صندوق رمل WASM",
    desc: "wasmtime integration. كل plugin يُنفَّذ في sandbox آمن، لا filesystem ولا شبكة مباشرة.",
    descEn: "wasmtime integration. Every plugin runs in a secure sandbox, no fs, no direct net.",
    icon: Shield,
    color: "oklch(0.55 0.22 25)",
    type: "technical",
  },
  {
    date: "2026-07-05",
    dateAr: "يوليو 2026",
    title: "v0.1.0-alpha Public Release",
    titleAr: "الإصدار العلني v0.1.0-alpha",
    desc: "أول إصدار علني كامل. 8.4k rps على VPS بـ 512MB RAM. لا تبعيات خارجية.",
    descEn: "First full public release. 8.4k rps on 512MB VPS. No external dependencies.",
    icon: Rocket,
    color: "oklch(0.72 0.19 47)",
    type: "release",
    stats: [
      { label: "binary size", value: "11MB" },
      { label: "container", value: "14.8MB" },
      { label: "RAM idle", value: "47MB" },
    ],
  },
  {
    date: "2026-07-15",
    dateAr: "يوليو 2026",
    title: "10k Stars · 500 Contributors",
    titleAr: "10 آلاف نجمة · 500 مساهم",
    desc: "المجتمع ينمو بسرعة. Discord وصل 3k عضو، أول 50 plugin في الـ marketplace.",
    descEn: "Community growing fast. Discord hit 3k members, first 50 plugins in marketplace.",
    icon: Users,
    color: "oklch(0.65 0.18 200)",
    type: "community",
    stats: [
      { label: "stars", value: "10k" },
      { label: "contributors", value: "500" },
      { label: "plugins", value: "50" },
    ],
  },
  {
    date: "2026-08-15",
    dateAr: "أغسطس 2026",
    title: "NAWA-DB v2 (ACID)",
    titleAr: "NAWA-DB v2 (ACID)",
    desc: "معاملات ACID كاملة، WAL، bloom filters، compaction strategy. (قيد التطوير)",
    descEn: "Full ACID transactions, WAL, bloom filters, compaction strategy. (In progress)",
    icon: Database,
    color: "oklch(0.78 0.16 165)",
    type: "release",
  },
  {
    date: "2026-09-15",
    dateAr: "سبتمبر 2026",
    title: "Frontend Engine v1",
    titleAr: "محرك الواجهة v1",
    desc: "SSR renderer + island hydration + streaming + edge cache. (مخطط)",
    descEn: "SSR renderer + island hydration + streaming + edge cache. (Planned)",
    icon: Globe,
    color: "oklch(0.65 0.18 280)",
    type: "release",
  },
  {
    date: "2026-12-01",
    dateAr: "ديسمبر 2026",
    title: "Security & Ops Suite",
    titleAr: "حزمة الأمان والعمليات",
    desc: "WAF، auto-TLS، self-healing، backup pipeline. (مخطط)",
    descEn: "WAF, auto-TLS, self-healing, backup pipeline. (Planned)",
    icon: Shield,
    color: "oklch(0.65 0.22 25)",
    type: "release",
  },
  {
    date: "2027-01-15",
    dateAr: "يناير 2027",
    title: "v1.0 Stable Release",
    titleAr: "الإصدار المستقر v1.0",
    desc: "الإصدار المستقر الكامل مع WASM marketplace، app templates، توثيق شامل. (مخطط)",
    descEn: "Full stable release with WASM marketplace, app templates, complete docs. (Planned)",
    icon: Rocket,
    color: "oklch(0.72 0.19 47)",
    type: "release",
    stats: [
      { label: "target", value: "v1.0" },
      { label: "support", value: "LTS" },
    ],
  },
];

const TYPE_CONFIG = {
  release: { color: "border-primary/40 bg-primary/5", badge: "release", badgeAr: "إصدار" },
  community: { color: "border-accent/40 bg-accent/5", badge: "community", badgeAr: "مجتمع" },
  technical: { color: "border-yellow-500/40 bg-yellow-500/5", badge: "technical", badgeAr: "تقني" },
  milestone: { color: "border-border/60 bg-card/40", badge: "milestone", badgeAr: "محطة" },
};

export function Timeline() {
  return (
    <section id="timeline" className="relative py-24 lg:py-32">
      <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="تاريخ المشروع"
          eyebrowEn="Project Timeline"
          title="رحلة من أول commit للإصدار المستقر"
          titleEn="Journey from first commit to stable release"
          desc="تتبّع مراحل تطور NAWA — من البداية في أبريل 2026، عبر الإصدارات والإنجازات التقنية والمجتمعية، حتى الإصدار المستقر v1.0 في 2027."
          descEn="Track NAWA's evolution — from the start in April 2026, through releases and milestones, to stable v1.0 in 2027."
        />

        {/* Stats summary */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 grid grid-cols-2 lg:grid-cols-4 gap-3"
        >
          {[
            { label: "إجمالي الإصدارات", value: "3", sub: "alpha → stable", icon: Rocket },
            { label: "المساهمون", value: "847", sub: "active", icon: Users },
            { label: "GitHub Stars", value: "12.4k", sub: "in 6 months", icon: Star },
            { label: "Production Deploys", value: "186", sub: "tracked", icon: Globe },
          ].map((s) => (
            <div key={s.label} className="p-4 rounded-xl border border-border/60 bg-card/60 text-center">
              <s.icon className="w-5 h-5 text-primary mx-auto mb-2" strokeWidth={1.5} />
              <div className="text-2xl font-bold text-primary mono">{s.value}</div>
              <div className="text-xs text-foreground mt-0.5 ar">{s.label}</div>
              <div className="text-[10px] text-muted-foreground">{s.sub}</div>
            </div>
          ))}
        </motion.div>

        {/* Timeline */}
        <div className="mt-16 relative">
          {/* Center line */}
          <div className="absolute right-1/2 lg:right-1/2 top-0 bottom-0 w-px bg-gradient-to-b from-primary/40 via-border/40 to-transparent translate-x-1/2" />

          <div className="space-y-8 lg:space-y-0">
            {MILESTONES.map((m, i) => {
              const isLeft = i % 2 === 0;
              const cfg = TYPE_CONFIG[m.type];

              return (
                <div key={i} className="relative lg:grid lg:grid-cols-2 lg:gap-12 lg:items-center">
                  {/* Marker */}
                  <div className="hidden lg:flex absolute right-1/2 top-1/2 -translate-y-1/2 translate-x-1/2 z-10">
                    <div
                      className="w-10 h-10 rounded-full bg-background grid place-items-center border-2"
                      style={{ borderColor: m.color }}
                    >
                      <m.icon className="w-4 h-4" style={{ color: m.color }} strokeWidth={1.5} />
                    </div>
                  </div>

                  {/* Mobile marker */}
                  <div className="lg:hidden absolute right-0 top-2 z-10">
                    <div
                      className="w-8 h-8 rounded-full bg-background grid place-items-center border-2"
                      style={{ borderColor: m.color }}
                    >
                      <m.icon className="w-3.5 h-3.5" style={{ color: m.color }} strokeWidth={1.5} />
                    </div>
                  </div>

                  <motion.div
                    initial={{ opacity: 0, x: isLeft ? -20 : 20 }}
                    whileInView={{ opacity: 1, x: 0 }}
                    viewport={{ once: true }}
                    transition={{ duration: 0.5, delay: i * 0.05 }}
                    className={`pr-12 lg:pr-0 ${isLeft ? "lg:pr-12 lg:text-left" : "lg:col-start-2 lg:pl-12 lg:text-right"}`}
                  >
                    <div className={`p-5 rounded-2xl border ${cfg.color}`}>
                      {/* Date + badge */}
                      <div className={`flex items-center gap-2 mb-2 ${isLeft ? "" : "lg:flex-row-reverse"}`}>
                        <Badge variant="outline" className="mono text-[10px]" style={{ borderColor: m.color + "60", color: m.color }}>
                          {m.date}
                        </Badge>
                        <Badge variant="outline" className="mono text-[9px] text-muted-foreground">
                          {cfg.badgeAr} · {cfg.badge}
                        </Badge>
                      </div>

                      {/* Title */}
                      <h4 className="text-base font-bold ar mb-1">{m.titleAr}</h4>
                      <p className="text-[11px] text-muted-foreground mono mb-3">{m.title}</p>

                      {/* Description */}
                      <p className="text-sm text-foreground/90 leading-relaxed ar mb-1">{m.desc}</p>
                      <p className="text-[11px] text-muted-foreground italic">{m.descEn}</p>

                      {/* Stats */}
                      {m.stats && (
                        <div className={`mt-3 pt-3 border-t border-border/40 flex gap-4 ${isLeft ? "" : "lg:flex-row-reverse"}`}>
                          {m.stats.map((s) => (
                            <div key={s.label} className={isLeft ? "" : "lg:text-left"}>
                              <div className="text-sm font-bold mono" style={{ color: m.color }}>
                                {s.value}
                              </div>
                              <div className="text-[10px] text-muted-foreground">{s.label}</div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  </motion.div>
                </div>
              );
            })}
          </div>

          {/* Future indicator */}
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.5 }}
            className="mt-12 text-center"
          >
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full border border-primary/40 bg-primary/10">
              <Heart className="w-3.5 h-3.5 text-primary fill-current animate-nawa-pulse" />
              <span className="text-xs ar text-primary font-medium">المستقبل يبدأ الآن</span>
            </div>
          </motion.div>
        </div>
      </div>
    </section>
  );
}
