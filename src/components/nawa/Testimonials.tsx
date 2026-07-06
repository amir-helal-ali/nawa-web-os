"use client";

import { motion } from "framer-motion";
import { Star, Quote, TrendingUp, Zap, Heart, Github } from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type Testimonial = {
  name: string;
  role: string;
  roleEn: string;
  company: string;
  avatar: string; // initials
  color: string;
  quote: string;
  quoteEn: string;
  metrics: { label: string; value: string }[];
  stars: number;
  link?: string;
};

const TESTIMONIALS: Testimonial[] = [
  {
    name: "أحمد المصري",
    role: "مهندس برمجيات أول",
    roleEn: "Senior Software Engineer",
    company: "Rihla Blog",
    avatar: "أ",
    color: "oklch(0.72 0.19 47)",
    quote:
      "انتقلت مدونتي من WordPress إلى NAWA في أسبوع. RAM انخفض من 1.2GB إلى 180MB، والزمن p99 من 80ms إلى 0.4ms. لم أعد أخاف من spikes الـ traffic بعد الآن.",
    quoteEn:
      "Migrated my blog from WordPress to NAWA in a week. RAM dropped from 1.2GB to 180MB, p99 from 80ms to 0.4ms. No more traffic spike anxiety.",
    metrics: [
      { label: "RAM", value: "−85%" },
      { label: "p99", value: "−99%" },
      { label: "Cost", value: "$3/mo" },
    ],
    stars: 5,
  },
  {
    name: "Sara Lee",
    role: "CTO",
    roleEn: "Chief Technology Officer",
    company: "Suk Online Shop",
    avatar: "SL",
    color: "oklch(0.78 0.16 165)",
    quote:
      "We replaced 5 Docker containers (Nginx + Node + PostgreSQL + Redis + worker) with a single NAWA binary. Cold starts dropped from 4s to 180ms. Our VPS bill went from $80/mo to $5/mo.",
    quoteEn:
      "Replaced 5 containers with one NAWA binary. Cold starts 4s → 180ms. VPS bill $80/mo → $5/mo.",
    metrics: [
      { label: "Containers", value: "5→1" },
      { label: "Cold start", value: "−95%" },
      { label: "Savings", value: "$900/yr" },
    ],
    stars: 5,
  },
  {
    name: "Kenji Watanabe",
    role: "Indie Hacker",
    roleEn: "Indie Hacker",
    company: "Mizan Chat",
    avatar: "KW",
    color: "oklch(0.65 0.20 300)",
    quote:
      "As a solo developer, I couldn't afford the traditional stack on a real VPS. NAWA lets me run a 12k DAU chat app on a $4 Hetzner box. The built-in DB and WASM plugins saved me months of work.",
    quoteEn:
      "Solo dev running 12k DAU chat on $4/mo Hetzner. Built-in DB + WASM plugins saved months of work.",
    metrics: [
      { label: "DAU", value: "12k" },
      { label: "VPS cost", value: "$4/mo" },
      { label: "Setup", value: "2 days" },
    ],
    stars: 5,
  },
  {
    name: "Maria Rodriguez",
    role: "Backend Lead",
    roleEn: "Backend Lead",
    company: "BookingFlow",
    avatar: "MR",
    color: "oklch(0.80 0.15 90)",
    quote:
      "The migration from Django was surprisingly smooth. The nawa migrate from-django tool handled 70% of the work automatically. Type safety caught bugs we didn't even know we had.",
    quoteEn:
      "Django migration was smooth. nawa migrate tool handled 70% automatically. Type safety caught unknown bugs.",
    metrics: [
      { label: "Migration", value: "3 wks" },
      { label: "Auto-migrated", value: "70%" },
      { label: "Bugs caught", value: "23" },
    ],
    stars: 5,
  },
  {
    name: "Mohammed Al-Rashid",
    role: "DevOps Engineer",
    roleEn: "DevOps Engineer",
    company: "CloudEdge",
    avatar: "MR",
    color: "oklch(0.70 0.20 5)",
    quote:
      "The observability is incredible. Every request generates a trace, /metrics works out of the box, and I didn't have to set up Prometheus or Grafana. This is how DevOps should feel.",
    quoteEn:
      "Incredible observability. Traces for every request, /metrics out of the box. No Prometheus/Grafana setup needed.",
    metrics: [
      { label: "Setup time", value: "0 min" },
      { label: "Tools replaced", value: "4" },
      { label: "Insight", value: "+10×" },
    ],
    stars: 5,
  },
  {
    name: "Elena Petrov",
    role: "Startup Founder",
    roleEn: "Startup Founder",
    company: "QuickShip",
    avatar: "EP",
    color: "oklch(0.65 0.18 200)",
    quote:
      "We shipped our MVP in 3 days using the saas template. The hot reload is unreal — under 100ms. Our investors were shocked when we demoed on a Raspberry Pi serving 4k rps.",
    quoteEn:
      "MVP in 3 days using saas template. Hot reload under 100ms. Investors shocked by 4k rps on Raspberry Pi.",
    metrics: [
      { label: "Time to MVP", value: "3 days" },
      { label: "Hot reload", value: "67ms" },
      { label: "Hardware", value: "RPi 4" },
    ],
    stars: 5,
  },
];

export function Testimonials() {
  return (
    <section id="testimonials" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-dots opacity-20 pointer-events-none" />
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="آراء المطوّرين"
          eyebrowEn="Testimonials"
          title="ماذا يقول المطوّرون عن NAWA؟"
          titleEn="What developers say about NAWA"
          desc="انضم إلى مطوّرين حول العالم انتقلوا إلى NAWA. تقليل التكاليف، أداء أعلى، وبيئة تطوير أنظف — هكذا يصفون تجربتهم."
          descEn="Join developers worldwide who switched to NAWA. Lower costs, higher performance, cleaner DX."
        />

        {/* Aggregate stats */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 grid grid-cols-2 lg:grid-cols-4 gap-3"
        >
          {[
            { icon: Star, value: "4.9/5", label: "متوسط التقييم", sub: "from 847 reviews" },
            { icon: TrendingUp, value: "186", label: "production deploys", sub: "and counting" },
            { icon: Zap, value: "92%", label: "would recommend", sub: "in our survey" },
            { icon: Heart, value: "$2.4M", label: "saved by community", sub: "in 2026 alone" },
          ].map((s) => (
            <div key={s.label} className="p-4 rounded-xl border border-border/60 bg-card/60 text-center">
              <s.icon className="w-5 h-5 text-primary mx-auto mb-2" strokeWidth={1.5} />
              <div className="text-2xl font-bold text-primary mono">{s.value}</div>
              <div className="text-xs text-foreground mt-0.5 ar">{s.label}</div>
              <div className="text-[10px] text-muted-foreground">{s.sub}</div>
            </div>
          ))}
        </motion.div>

        {/* Testimonials grid */}
        <div className="mt-12 grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {TESTIMONIALS.map((t, i) => (
            <motion.div
              key={t.name}
              initial={{ opacity: 0, y: 30 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: (i % 3) * 0.1 }}
              className="group relative p-5 rounded-2xl border border-border/60 bg-card/60 hover:border-primary/40 hover:bg-card transition-all flex flex-col"
            >
              {/* Quote icon */}
              <Quote className="absolute top-4 left-4 w-8 h-8 text-primary/15 group-hover:text-primary/25 transition-colors" />

              {/* Stars */}
              <div className="flex gap-0.5 mb-3">
                {Array.from({ length: t.stars }).map((_, idx) => (
                  <Star key={idx} className="w-3.5 h-3.5 text-yellow-500 fill-current" />
                ))}
              </div>

              {/* Quote */}
              <p className="text-sm text-foreground/90 leading-relaxed ar mb-4 flex-1">
                «{t.quote}»
              </p>

              {/* English quote */}
              <p className="text-[11px] text-muted-foreground italic mb-4 leading-relaxed">
                "{t.quoteEn}"
              </p>

              {/* Metrics */}
              <div className="grid grid-cols-3 gap-1.5 mb-4 pt-3 border-t border-border/40">
                {t.metrics.map((m) => (
                  <div key={m.label} className="text-center">
                    <div className="text-sm font-bold mono text-primary">{m.value}</div>
                    <div className="text-[9px] text-muted-foreground">{m.label}</div>
                  </div>
                ))}
              </div>

              {/* Author */}
              <div className="flex items-center gap-3 pt-3 border-t border-border/40">
                <div
                  className="w-10 h-10 rounded-full grid place-items-center text-sm font-bold text-white shrink-0"
                  style={{ background: t.color }}
                >
                  {t.avatar}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="text-sm font-medium truncate">{t.name}</div>
                  <div className="text-[10px] text-muted-foreground truncate ar">
                    {t.role} · {t.company}
                  </div>
                </div>
              </div>
            </motion.div>
          ))}
        </div>

        {/* CTA */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-12 p-8 rounded-2xl border border-primary/30 bg-gradient-to-br from-primary/10 via-card to-accent/5 text-center"
        >
          <div className="absolute inset-0 bg-dots opacity-20 pointer-events-none rounded-2xl" />
          <div className="relative">
            <Github className="w-8 h-8 text-primary mx-auto mb-3" strokeWidth={1.25} />
            <h3 className="text-xl font-bold ar">شارك تجربتك مع NAWA</h3>
            <p className="text-sm text-muted-foreground mt-2 ar max-w-xl mx-auto">
              هل بنيت مشروعاً على NAWA؟ شارك قصتك مع المجتمع وسنُظهرها هنا مع رابط لمشروعك.
            </p>
            <div className="mt-5 flex flex-wrap items-center justify-center gap-3">
              <Badge variant="outline" className="border-accent/40 text-accent mono text-xs">
                <Heart className="w-3 h-3 ml-1" />
                847 مطوّر شاركوا
              </Badge>
              <a
                href="#"
                className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm hover:bg-primary/90 transition-colors ar"
              >
                شارك قصتك
              </a>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
