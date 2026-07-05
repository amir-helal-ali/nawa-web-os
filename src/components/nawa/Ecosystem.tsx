"use client";

import { motion } from "framer-motion";
import {
  Users,
  GitBranch,
  Star,
  BookOpen,
  MessageSquare,
  GraduationCap,
  Heart,
  Globe,
  Github,
  Twitter,
  Zap,
  Server,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

const STATS = [
  { icon: Star, value: "12.4k", label: "GitHub Stars", sub: "in 6 months" },
  { icon: Users, value: "847", label: "Contributors", sub: "active" },
  { icon: GitBranch, value: "3.2k", label: "Forks", sub: "community" },
  { icon: Server, value: "186", label: "Production Deploys", sub: "tracked" },
];

const RESOURCES = [
  {
    icon: BookOpen,
    title: "الوثائق",
    titleEn: "Documentation",
    desc: "وثائق شاملة بالعربية والإنجليزية: quickstart، tutorials، API reference، guides.",
    link: "docs.nawa.dev",
    color: "primary",
  },
  {
    icon: GraduationCap,
    title: "أكاديمية NAWA",
    titleEn: "NAWA Academy",
    desc: "دورات مجانية: من مبتدئ Rust إلى نشر تطبيق إنتاجي كامل خلال أسبوعين.",
    link: "academy.nawa.dev",
    color: "accent",
  },
  {
    icon: MessageSquare,
    title: "المجتمع",
    titleEn: "Community",
    desc: "Discord بـ 4k+ عضو، forum للأسئلة، monthly office hours مع الفريق الأساسي.",
    link: "discord.nawa.dev",
    color: "primary",
  },
  {
    icon: Heart,
    title: "الدعم التجاري",
    titleEn: "Commercial Support",
    desc: "باقات دفع مدفوعة: SLA 99.99%، دعم 24/7، استشارات architecture.",
    link: "enterprise.nawa.dev",
    color: "accent",
  },
];

const SHOWCASE = [
  {
    name: "Rihla Blog",
    nameAr: "مدونة رحلة",
    type: "Blog Platform",
    desc: "منصة تدوين عربية بـ 200k مقال، تعمل على VPS واحد 512MB.",
    stack: "blog template + auth-jwt + search-fts",
    metric: "200k articles · 0.4ms p99",
  },
  {
    name: "Suk online shop",
    nameAr: "متجر سوق",
    type: "E-commerce",
    desc: "متجر إلكتروني بـ 50k منتج، يدعم 5k زائر متزامن على Raspberry Pi 4.",
    stack: "shop template + payment-stripe + analytics",
    metric: "5k concurrent · $2.4k/month savings",
  },
  {
    name: "Mizan Chat",
    nameAr: "دردشة ميزان",
    type: "Realtime App",
    desc: "تطبيق دردشة بـ 12k مستخدم نشط يومياً، WebSocket على نفس الـ binary.",
    stack: "realtime template + auth-oauth + cdn-edge",
    metric: "12k DAU · 180ms cold start",
  },
];

export function Ecosystem() {
  return (
    <section id="ecosystem" className="relative py-24 lg:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="المجتمع والمنظومة"
          eyebrowEn="Ecosystem & Community"
          title="مجتمع ينمو معك"
          titleEn="A community that grows with you"
          desc="NAWA ليس فقط تقنية — بل منظومة كاملة: وثائق، أكاديمية، مجتمع نشط، ودعم تجاري. مفتوح المصدر بالكامل تحت MIT + Apache 2.0."
          descEn="NAWA isn't just technology — it's a complete ecosystem: docs, academy, active community, and commercial support. Fully open-source under MIT + Apache 2.0."
        />

        {/* Stats */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-12 grid grid-cols-2 lg:grid-cols-4 gap-3"
        >
          {STATS.map((s, i) => (
            <motion.div
              key={s.label}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: i * 0.08 }}
              className="relative p-5 rounded-2xl border border-border/60 bg-card/60 overflow-hidden"
            >
              <div className="absolute -top-6 -right-6 w-20 h-20 rounded-full bg-primary/10 blur-2xl pointer-events-none" />
              <div className="relative">
                <s.icon className="w-5 h-5 text-primary mb-2" strokeWidth={1.5} />
                <div className="text-3xl font-bold text-primary mono">{s.value}</div>
                <div className="text-sm font-medium mt-0.5">{s.label}</div>
                <div className="text-[10px] text-muted-foreground">{s.sub}</div>
              </div>
            </motion.div>
          ))}
        </motion.div>

        {/* Resources grid */}
        <div className="mt-8 grid sm:grid-cols-2 lg:grid-cols-4 gap-4">
          {RESOURCES.map((r, i) => (
            <motion.div
              key={r.titleEn}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: i * 0.08 }}
              className="group p-5 rounded-2xl border border-border/60 bg-card/60 hover:border-primary/40 hover:bg-card transition-all"
            >
              <div className={`p-2 rounded-lg w-fit mb-3 ${r.color === "primary" ? "bg-primary/15 text-primary" : "bg-accent/15 text-accent"}`}>
                <r.icon className="w-5 h-5" strokeWidth={1.5} />
              </div>
              <h4 className="text-sm font-semibold ar">{r.title}</h4>
              <p className="text-[10px] text-muted-foreground mono mt-0.5">{r.titleEn}</p>
              <p className="text-xs text-muted-foreground mt-2 leading-relaxed ar">{r.desc}</p>
              <div className="mt-3 pt-3 border-t border-border/40">
                <code className="text-xs mono text-primary group-hover:underline">{r.link}</code>
              </div>
            </motion.div>
          ))}
        </div>

        {/* Showcase */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-16"
        >
          <div className="flex items-center gap-2 mb-6">
            <Zap className="w-4 h-4 text-primary" />
            <h3 className="text-base font-semibold ar">مشاريع حقيقية مبنية على NAWA</h3>
            <span className="text-xs text-muted-foreground mono ml-1">{`// production showcase`}</span>
          </div>
          <div className="grid lg:grid-cols-3 gap-4">
            {SHOWCASE.map((s, i) => (
              <motion.div
                key={s.name}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.4, delay: i * 0.1 }}
                className="p-5 rounded-2xl border border-border/60 bg-card/60 hover:border-primary/40 transition-colors"
              >
                <div className="flex items-start justify-between mb-3">
                  <div>
                    <h4 className="text-base font-bold ar">{s.nameAr}</h4>
                    <p className="text-[10px] text-muted-foreground mono">{s.name}</p>
                  </div>
                  <Badge variant="outline" className="mono text-[10px] border-accent/40 text-accent">
                    {s.type}
                  </Badge>
                </div>
                <p className="text-xs text-muted-foreground leading-relaxed ar mb-3">{s.desc}</p>
                <div className="space-y-1.5">
                  <div>
                    <span className="text-[10px] text-muted-foreground mono">stack:</span>
                    <code className="text-[10px] mono text-primary mr-1">{s.stack}</code>
                  </div>
                  <div className="flex items-center gap-1.5 pt-2 border-t border-border/40">
                    <Zap className="w-3 h-3 text-accent" />
                    <span className="text-[11px] text-foreground mono">{s.metric}</span>
                  </div>
                </div>
              </motion.div>
            ))}
          </div>
        </motion.div>

        {/* Governance & License */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-16 grid lg:grid-cols-2 gap-4"
        >
          <div className="p-6 rounded-2xl border border-border/60 bg-card/60">
            <div className="flex items-center gap-2 mb-3">
              <Globe className="w-4 h-4 text-primary" />
              <h4 className="text-sm font-semibold ar">الحوكمة</h4>
            </div>
            <ul className="space-y-2 text-xs text-muted-foreground">
              <li className="flex items-start gap-2">
                <span className="text-primary">●</span>
                <span className="ar">BDFL حتى v1.0، ثم council منتخب</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-primary">●</span>
                <span className="ar">RFC process للـ breaking changes</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-primary">●</span>
                <span className="ar">2-reviewer approval لكل PR</span>
              </li>
              <li className="flex items-start gap-2">
                <span className="text-primary">●</span>
                <span className="ar">Semantic versioning صارم</span>
              </li>
            </ul>
          </div>
          <div className="p-6 rounded-2xl border border-primary/30 bg-gradient-to-br from-primary/10 to-accent/5">
            <div className="flex items-center gap-2 mb-3">
              <Heart className="w-4 h-4 text-primary" />
              <h4 className="text-sm font-semibold ar">الترخيص</h4>
            </div>
            <div className="flex items-center gap-2 mb-3">
              <Badge className="bg-primary text-primary-foreground mono">MIT</Badge>
              <Badge variant="outline" className="border-accent/40 text-accent mono">Apache 2.0</Badge>
              <span className="text-xs text-muted-foreground ar">ثنائي الترخيص</span>
            </div>
            <p className="text-xs text-muted-foreground leading-relaxed ar">
              مفتوح المصدر بالكامل — استخدمه تجارياً، عدّله، وزّعه دون قيود. نفس نموذج Rust نفسه.
            </p>
            <div className="mt-3 flex items-center gap-2">
              <Github className="w-3.5 h-3.5 text-muted-foreground" />
              <code className="text-xs mono text-primary">github.com/nawa-os/nawa</code>
            </div>
          </div>
        </motion.div>

        {/* Social CTA */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-8 relative overflow-hidden rounded-2xl border border-primary/30 bg-gradient-to-br from-primary/10 via-card to-accent/5 p-8 lg:p-10"
        >
          <div className="absolute inset-0 bg-dots opacity-20 pointer-events-none" />
          <div className="relative text-center">
            <Users className="w-10 h-10 text-primary mx-auto mb-4" strokeWidth={1.25} />
            <h3 className="text-xl lg:text-2xl font-bold ar">
              انضم إلى 4,000+ مطوّر يبنون مستقبل الويب
            </h3>
            <p className="text-sm text-muted-foreground mt-2 ar max-w-2xl mx-auto">
              Discord حي، monthly office hours، contributors monthly rewards — مجتمع NAWA ينتظرك.
            </p>
            <div className="mt-6 flex flex-wrap items-center justify-center gap-3">
              <a
                href="#"
                className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border/60 bg-card/60 hover:border-primary/40 transition-colors"
              >
                <Github className="w-4 h-4 text-primary" />
                <span className="text-sm ar">Star on GitHub</span>
                <Star className="w-3 h-3 text-yellow-500 fill-current" />
              </a>
              <a
                href="#"
                className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border/60 bg-card/60 hover:border-primary/40 transition-colors"
              >
                <MessageSquare className="w-4 h-4 text-accent" />
                <span className="text-sm ar">انضم لـ Discord</span>
              </a>
              <a
                href="#"
                className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border/60 bg-card/60 hover:border-primary/40 transition-colors"
              >
                <Twitter className="w-4 h-4 text-primary" />
                <span className="text-sm ar">تابعنا</span>
              </a>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
