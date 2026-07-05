"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  GitCommit,
  Tag,
  Sparkles,
  Bug,
  Zap,
  Shield,
  ArrowUp,
  Filter,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type ChangeType = "feature" | "improvement" | "fix" | "security";

type Change = {
  type: ChangeType;
  desc: string;
  descEn: string;
};

type Version = {
  version: string;
  date: string;
  status: "released" | "current" | "upcoming";
  highlight?: boolean;
  changes: Change[];
};

const VERSIONS: Version[] = [
  {
    version: "0.2.0-alpha",
    date: "2026-08-15",
    status: "upcoming",
    highlight: true,
    changes: [
      { type: "feature", desc: "NAWA-DB ACID transactions كاملة", descEn: "Full ACID transactions in NAWA-DB" },
      { type: "feature", desc: "WAL (Write-Ahead Log) مع fsync تكتيكي", descEn: "Tactical fsync WAL" },
      { type: "feature", desc: "Bloom filters لكل SSTable", descEn: "Per-SSTable bloom filters" },
      { type: "improvement", desc: "Compaction strategy L0-L7", descEn: "L0-L7 compaction strategy" },
      { type: "improvement", desc: "memtable flush أصبح async", descEn: "Async memtable flush" },
      { type: "fix", desc: "إصلاح race condition في skip-list", descEn: "Fix skip-list race condition" },
    ],
  },
  {
    version: "0.1.0-alpha",
    date: "2026-07-05",
    status: "current",
    highlight: true,
    changes: [
      { type: "feature", desc: "إصدار NAWA الأول للعموم", descEn: "First public release of NAWA" },
      { type: "feature", desc: "Zero-copy io_uring kernel", descEn: "Zero-copy io_uring kernel" },
      { type: "feature", desc: "HTTP/3 + QUIC server (quinn)", descEn: "HTTP/3 + QUIC server" },
      { type: "feature", desc: "Type-safe router بـ proc-macros", descEn: "Type-safe proc-macro router" },
      { type: "feature", desc: "NAWA-DB prototype (KV + Document)", descEn: "NAWA-DB prototype" },
      { type: "feature", desc: "WASM sandbox للمستخدم plugins", descEn: "WASM sandbox for plugins" },
      { type: "security", desc: "Zero-trust auth middleware", descEn: "Zero-trust auth middleware" },
    ],
  },
  {
    version: "0.0.3-internal",
    date: "2026-06-10",
    status: "released",
    changes: [
      { type: "improvement", desc: "تحسين أداء io_uring بـ 40%", descEn: "40% io_uring performance boost" },
      { type: "feature", desc: "Hot reload للـ dev server", descEn: "Hot reload dev server" },
      { type: "fix", desc: "إصلاح memory leak في connection pool", descEn: "Fix connection pool memory leak" },
    ],
  },
  {
    version: "0.0.2-internal",
    date: "2026-05-15",
    status: "released",
    changes: [
      { type: "feature", desc: "LSM tree storage engine", descEn: "LSM tree storage engine" },
      { type: "feature", desc: "mmap للوصول المباشر للملفات", descEn: "mmap for direct file access" },
      { type: "improvement", desc: "Binary size انخفض لـ 11MB", descEn: "Binary size down to 11MB" },
    ],
  },
  {
    version: "0.0.1-internal",
    date: "2026-04-01",
    status: "released",
    changes: [
      { type: "feature", desc: "بدء المشروع — أول commit", descEn: "Project started — first commit" },
      { type: "feature", desc: "Rust workspace + CI/CD", descEn: "Rust workspace + CI/CD" },
      { type: "feature", desc: "HTTP/1.1 server أساسي", descEn: "Basic HTTP/1.1 server" },
    ],
  },
];

const TYPE_CONFIG: Record<
  ChangeType,
  { icon: typeof Zap; color: string; bg: string; label: string; labelAr: string }
> = {
  feature: { icon: Sparkles, color: "text-primary", bg: "bg-primary/15", label: "feature", labelAr: "ميزة" },
  improvement: { icon: Zap, color: "text-accent", bg: "bg-accent/15", label: "improvement", labelAr: "تحسين" },
  fix: { icon: Bug, color: "text-yellow-500", bg: "bg-yellow-500/15", label: "fix", labelAr: "إصلاح" },
  security: { icon: Shield, color: "text-destructive", bg: "bg-destructive/15", label: "security", labelAr: "أمان" },
};

const FILTERS: Array<{ id: "all" | ChangeType; label: string; labelAr: string }> = [
  { id: "all", label: "all", labelAr: "الكل" },
  { id: "feature", label: "features", labelAr: "ميزات" },
  { id: "improvement", label: "improvements", labelAr: "تحسينات" },
  { id: "fix", label: "fixes", labelAr: "إصلاحات" },
  { id: "security", label: "security", labelAr: "أمان" },
];

export function Changelog() {
  const [filter, setFilter] = useState<"all" | ChangeType>("all");

  return (
    <section id="changelog" className="relative py-24 lg:py-32">
      <div className="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="سجل التغييرات"
          eyebrowEn="Changelog"
          title="كل تحسين موثّق"
          titleEn="Every improvement documented"
          desc="تتبّع تطور NAWA عبر الإصدارات. الشفافية الكاملة — كل ميزة، كل إصلاح، كل تحسين أمان مسجّل هنا."
          descEn="Track NAWA's evolution across releases. Full transparency — every feature, fix, and security improvement."
        />

        {/* Filter */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 flex items-center gap-2 flex-wrap"
        >
          <Filter className="w-3.5 h-3.5 text-muted-foreground" />
          <span className="text-xs text-muted-foreground mono mr-1">filter:</span>
          {FILTERS.map((f) => (
            <button
              key={f.id}
              onClick={() => setFilter(f.id)}
              className={`px-2.5 py-1 rounded-md text-xs border transition-all ${
                filter === f.id
                  ? "border-primary bg-primary/10 text-primary"
                  : "border-border/60 text-muted-foreground hover:text-foreground"
              }`}
            >
              <span className="ar">{f.labelAr}</span>
              <span className="mono opacity-60 mr-1">/{f.label}</span>
            </button>
          ))}
        </motion.div>

        {/* Timeline */}
        <div className="mt-10 relative">
          {/* Vertical line */}
          <div className="absolute right-4 top-0 bottom-0 w-px bg-gradient-to-b from-primary/40 via-border/40 to-transparent" />

          <div className="space-y-8">
            {VERSIONS.map((v, i) => {
              const filteredChanges = v.changes.filter((c) => filter === "all" || c.type === filter);
              if (filteredChanges.length === 0 && filter !== "all") return null;

              return (
                <motion.div
                  key={v.version}
                  initial={{ opacity: 0, x: 20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.5, delay: i * 0.1 }}
                  className="relative pr-12"
                >
                  {/* Version marker */}
                  <div className="absolute right-0 top-1.5 w-8 h-8 rounded-full bg-background border-2 border-primary grid place-items-center">
                    {v.status === "current" ? (
                      <Tag className="w-3.5 h-3.5 text-primary" />
                    ) : v.status === "upcoming" ? (
                      <ArrowUp className="w-3.5 h-3.5 text-accent" />
                    ) : (
                      <GitCommit className="w-3.5 h-3.5 text-muted-foreground" />
                    )}
                  </div>

                  {/* Version header */}
                  <div className="flex items-center gap-3 flex-wrap mb-4">
                    <h3 className="text-xl font-bold mono">{v.version}</h3>
                    {v.status === "current" && (
                      <Badge className="bg-primary text-primary-foreground mono text-[10px]">
                        current
                      </Badge>
                    )}
                    {v.status === "upcoming" && (
                      <Badge variant="outline" className="border-accent/40 text-accent mono text-[10px]">
                        <span className="w-1.5 h-1.5 rounded-full bg-accent mr-1 animate-nawa-pulse" />
                        upcoming
                      </Badge>
                    )}
                    {v.highlight && (
                      <Badge variant="outline" className="border-primary/40 text-primary mono text-[10px]">
                        ✦ major
                      </Badge>
                    )}
                    <span className="text-xs text-muted-foreground mono">{v.date}</span>
                  </div>

                  {/* Changes list */}
                  <div className="space-y-2">
                    {filteredChanges.map((c, idx) => {
                      const cfg = TYPE_CONFIG[c.type];
                      return (
                        <motion.div
                          key={idx}
                          initial={{ opacity: 0, y: 10 }}
                          whileInView={{ opacity: 1, y: 0 }}
                          viewport={{ once: true }}
                          transition={{ duration: 0.3, delay: idx * 0.05 }}
                          className="flex items-start gap-3 p-3 rounded-lg border border-border/60 bg-card/40 hover:bg-card/70 transition-colors"
                        >
                          <div className={`p-1.5 rounded ${cfg.bg} shrink-0`}>
                            <cfg.icon className={`w-3.5 h-3.5 ${cfg.color}`} strokeWidth={1.5} />
                          </div>
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2 mb-0.5">
                              <span className={`text-[10px] mono font-medium ${cfg.color}`}>
                                {cfg.label}
                              </span>
                              <span className="text-[10px] text-muted-foreground">·</span>
                              <span className="text-[10px] text-muted-foreground ar">{cfg.labelAr}</span>
                            </div>
                            <p className="text-sm text-foreground/90 ar">{c.desc}</p>
                            <p className="text-[11px] text-muted-foreground italic mt-0.5">{c.descEn}</p>
                          </div>
                        </motion.div>
                      );
                    })}
                  </div>
                </motion.div>
              );
            })}
          </div>
        </div>

        {/* Bottom note */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-12 p-4 rounded-xl border border-border/60 bg-card/60 flex items-center gap-3"
        >
          <GitCommit className="w-4 h-4 text-primary shrink-0" />
          <p className="text-xs text-muted-foreground ar">
            السجل الكامل متاح على{" "}
            <a
              href="https://github.com/amir-helal-ali/nawa-web-os/releases"
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary hover:underline mono"
            >
              GitHub Releases
            </a>
            . اشترك في notifications لتصلك تحديثات الإصدارات.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
