"use client";

import { motion, AnimatePresence } from "framer-motion";
import { useState } from "react";
import {
  Newspaper,
  ShoppingBag,
  MessageSquare,
  Calendar,
  PenTool,
  BarChart3,
  Server,
  Cpu,
  GraduationCap,
  HeartPulse,
  Plane,
  Gamepad2,
  ArrowLeft,
  Zap,
  TrendingUp,
  Shield,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type UseCase = {
  id: string;
  name: string;
  nameAr: string;
  icon: typeof Newspaper;
  color: string;
  desc: string;
  descEn: string;
  whyNawa: string;
  metrics: { label: string; value: string; sub?: string }[];
  template: string;
  tags: string[];
};

const USE_CASES: UseCase[] = [
  {
    id: "blog",
    name: "Blog / CMS",
    nameAr: "مدوّنة / نظام محتوى",
    icon: Newspaper,
    color: "oklch(0.72 0.19 47)",
    desc: "مدونات شخصية، منصات محتوى عربية، مجلات إلكترونية بـ SEO قوي.",
    descEn: "Personal blogs, Arabic content platforms, online magazines with strong SEO.",
    whyNawa: "SSR فوري للـ SEO، DB مدمج يُغني عن WordPress + MySQL، ويعمل على VPS اقتصادي.",
    metrics: [
      { label: "زمن الإقلاع", value: "180ms" },
      { label: "p99 latency", value: "0.4ms" },
      { label: "VPS cost", value: "$3/mo", sub: "Hetzner" },
    ],
    template: "blog",
    tags: ["SSR", "SEO", "Markdown", "Admin Panel"],
  },
  {
    id: "ecommerce",
    name: "E-commerce",
    nameAr: "متجر إلكتروني",
    icon: ShoppingBag,
    color: "oklch(0.78 0.16 165)",
    desc: "متاجر صغيرة ومتوسطة، dropshipping، منصات B2B بـ inventory و payments.",
    descEn: "Small-to-mid stores, dropshipping, B2B platforms with inventory and payments.",
    whyNawa: "ACID transactions تضمن سلامة الطلبات، WASM sandbox للـ payment plugins، TLS تلقائي.",
    metrics: [
      { label: "Concurrent users", value: "5k" },
      { label: "Checkout p99", value: "0.8ms" },
      { label: "Stack cost", value: "$5/mo" },
    ],
    template: "shop",
    tags: ["Stripe", "Inventory", "Cart", "ACID"],
  },
  {
    id: "realtime",
    name: "Realtime Apps",
    nameAr: "تطبيقات حية",
    icon: MessageSquare,
    color: "oklch(0.65 0.20 300)",
    desc: "تطبيقات دردشة، collaboration tools، live dashboards، multiplayer games.",
    descEn: "Chat apps, collaboration tools, live dashboards, multiplayer games.",
    whyNawa: "WebSocket مدمج في نفس الـ binary، pub/sub عبر NAWA-DB، presence بدون Redis منفصل.",
    metrics: [
      { label: "WebSocket conns", value: "10k" },
      { label: "Message latency", value: "0.2ms" },
      { label: "RAM (1k users)", value: "120MB" },
    ],
    template: "realtime",
    tags: ["WebSocket", "Presence", "Pub/Sub", "Live"],
  },
  {
    id: "saas",
    name: "SaaS Platform",
    nameAr: "منصة SaaS",
    icon: Server,
    color: "oklch(0.65 0.18 200)",
    desc: "Multi-tenant SaaS مع subscriptions، team management، billing، dashboards.",
    descEn: "Multi-tenant SaaS with subscriptions, team management, billing, dashboards.",
    whyNawa: "Tenant isolation بـ WASM، JWT auth مدمج، /metrics endpoint للـ SaaS analytics.",
    metrics: [
      { label: "Tenants per GB", value: "200" },
      { label: "Cold start", value: "180ms" },
      { label: "Setup time", value: "2 days" },
    ],
    template: "saas",
    tags: ["Multi-tenant", "JWT", "Stripe", "Dashboard"],
  },
  {
    id: "booking",
    name: "Booking & Scheduling",
    nameAr: "حجوزات ومواعيد",
    icon: Calendar,
    color: "oklch(0.80 0.15 90)",
    desc: "حجوزات طبيين، صالونات، مطاعم، خدمات، rental، events.",
    descEn: "Medical, salon, restaurant, service, rental, event bookings.",
    whyNawa: "Transactional booking مع ACID، timezone-aware scheduling، reminder emails via WASM plugin.",
    metrics: [
      { label: "Bookings/sec", value: "2k" },
      { label: "Double-book risk", value: "0%" },
      { label: "Reminder cost", value: "$0" },
    ],
    template: "booking",
    tags: ["Calendar", "ACID", "Payments", "Reminders"],
  },
  {
    id: "portfolio",
    name: "Portfolio / Personal",
    nameAr: "بورتفوليو شخصي",
    icon: PenTool,
    color: "oklch(0.70 0.20 5)",
    desc: "مواقع شخصية، CVs، معارض أعمال، landing pages.",
    descEn: "Personal sites, CVs, portfolios, landing pages.",
    whyNawa: "Static-generation للسرعة القصوى، edge cache مجاني، analytics تحترم الخصوصية.",
    metrics: [
      { label: "TTFB", value: "5ms" },
      { label: "Lighthouse", value: "100" },
      { label: "Monthly cost", value: "$0" },
    ],
    template: "portfolio",
    tags: ["Static", "Edge", "Analytics", "Dark Mode"],
  },
  {
    id: "analytics",
    name: "Analytics Dashboard",
    nameAr: "لوحة تحليلات",
    icon: BarChart3,
    color: "oklch(0.65 0.18 50)",
    desc: "لوحات تحليلات، monitoring، BI tools، metrics visualization.",
    descEn: "Analytics dashboards, monitoring, BI tools, metrics visualization.",
    whyNawa: "LSM tree مثالي للـ time-series، /metrics مدمج، zero-copy للـ large scans.",
    metrics: [
      { label: "Points/sec", value: "100k" },
      { label: "Query p99", value: "1.2ms" },
      { label: "Storage", value: "10× compress" },
    ],
    template: "saas",
    tags: ["Time-series", "Charts", "Real-time", "Export"],
  },
  {
    id: "education",
    name: "Education / LMS",
    nameAr: "منصة تعليمية",
    icon: GraduationCap,
    color: "oklch(0.55 0.20 300)",
    desc: "منصات تعليم إلكتروني، courses، quizzes، progress tracking، certificates.",
    descEn: "E-learning platforms, courses, quizzes, progress tracking, certificates.",
    whyNawa: "Video streaming بـ sendfile، progress tracking في NAWA-DB، certificates آمنة.",
    metrics: [
      { label: "Concurrent students", value: "1k" },
      { label: "Video start", value: "200ms" },
      { label: "Cert verification", value: "0.3ms" },
    ],
    template: "saas",
    tags: ["LMS", "Video", "Certificates", "Progress"],
  },
  {
    id: "healthcare",
    name: "Healthcare Portal",
    nameAr: "بوابة طبية",
    icon: HeartPulse,
    color: "oklch(0.65 0.22 25)",
    desc: "بوابات مرضى، سجلات طبية، حجوزات، استشارات عن بُعد.",
    descEn: "Patient portals, medical records, appointments, telemedicine.",
    whyNawa: "HIPAA-ready encryption، audit log غير قابل للتعديل، RBAC صارم.",
    metrics: [
      { label: "Data encryption", value: "AES-256" },
      { label: "Audit log", value: "tamper-proof" },
      { label: "Compliance", value: "HIPAA-ready" },
    ],
    template: "booking",
    tags: ["HIPAA", "Audit", "Encryption", "RBAC"],
  },
  {
    id: "travel",
    name: "Travel / Hospitality",
    nameAr: "سفر وضيافة",
    icon: Plane,
    color: "oklch(0.65 0.18 200)",
    desc: "حجوزات فنادق، رحلات، car rental، reviews، loyalty programs.",
    descEn: "Hotel bookings, flights, car rental, reviews, loyalty programs.",
    whyNawa: "Complex queries بـ scan + filter، multi-currency، i18n مدمج، CDN edge.",
    metrics: [
      { label: "Search p99", value: "2.1ms" },
      { label: "Languages", value: "12" },
      { label: "Edge locations", value: "200+" },
    ],
    template: "shop",
    tags: ["i18n", "Multi-currency", "Search", "CDN"],
  },
  {
    id: "gaming",
    name: "Game Backend",
    nameAr: "خادم ألعاب",
    icon: Gamepad2,
    color: "oklch(0.55 0.22 25)",
    desc: "خوادم ألعاب multiplayer، leaderboards، matchmaking، player profiles.",
    descEn: "Multiplayer game servers, leaderboards, matchmaking, player profiles.",
    whyNawa: "Ultra-low latency للـ game state، pub/sub للأحداث، lock-free للـ leaderboards.",
    metrics: [
      { label: "Game tick", value: "16ms" },
      { label: "Players/instance", value: "500" },
      { label: "Leaderboard update", value: "0.1ms" },
    ],
    template: "realtime",
    tags: ["Multiplayer", "Leaderboard", "Matchmaking", "Real-time"],
  },
  {
    id: "iot",
    name: "IoT / Edge",
    nameAr: "إنترنت الأشياء",
    icon: Cpu,
    color: "oklch(0.55 0.18 165)",
    desc: "IoT gateways، edge processing، device management، telemetry.",
    descEn: "IoT gateways, edge processing, device management, telemetry.",
    whyNawa: "يعمل على Raspberry Pi 4 بكفاءة، MQTT عبر WASM plugin، time-series storage.",
    metrics: [
      { label: "Devices per node", value: "10k" },
      { label: "Edge RAM", value: "80MB" },
      { label: "MQTT throughput", value: "50k msg/s" },
    ],
    template: "realtime",
    tags: ["MQTT", "Edge", "Raspberry Pi", "Telemetry"],
  },
];

const INDUSTRIES = [
  { label: "Content", labelAr: "محتوى", count: 2 },
  { label: "Commerce", labelAr: "تجارة", count: 2 },
  { label: "Realtime", labelAr: "حي", count: 3 },
  { label: "Enterprise", labelAr: "مؤسسات", count: 3 },
  { label: "Specialized", labelAr: "متخصص", count: 2 },
];

export function UseCases() {
  const [active, setActive] = useState(0);
  const uc = USE_CASES[active];

  return (
    <section id="usecases" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-grid opacity-30 pointer-events-none" />
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="حالات الاستخدام"
          eyebrowEn="Use Cases"
          title="مبني لأي مشروع ويب"
          titleEn="Built for any web project"
          desc="من المدونات الشخصية لخوادم الألعاب متعددة اللاعبين — NAWA يتكيّف مع احتياجاتك. استكشف 12 حالة استخدام شائعة مع مقاييس حقيقية."
          descEn="From personal blogs to multiplayer game servers — NAWA adapts to your needs. Explore 12 common use cases with real metrics."
        />

        {/* Industry tags */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 flex flex-wrap gap-2"
        >
          {INDUSTRIES.map((ind) => (
            <div
              key={ind.label}
              className="px-3 py-1.5 rounded-full border border-border/60 bg-card/60 text-xs flex items-center gap-2"
            >
              <span className="ar">{ind.labelAr}</span>
              <span className="mono text-muted-foreground">·</span>
              <span className="mono text-primary">{ind.count}</span>
            </div>
          ))}
        </motion.div>

        {/* Use cases grid */}
        <div className="mt-8 grid sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
          {USE_CASES.map((u, i) => (
            <motion.button
              key={u.id}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.3, delay: (i % 4) * 0.05 }}
              onClick={() => setActive(i)}
              className={`group text-right p-4 rounded-xl border transition-all ${
                active === i
                  ? "border-primary bg-primary/10 scale-[1.02]"
                  : "border-border/60 bg-card/60 hover:border-primary/40"
              }`}
            >
              <div
                className="p-2 rounded-lg w-fit mb-2"
                style={{ background: u.color + "20", color: u.color }}
              >
                <u.icon className="w-4 h-4" strokeWidth={1.5} />
              </div>
              <div className={`text-sm font-medium ${active === i ? "text-primary" : "text-foreground"} ar`}>
                {u.nameAr}
              </div>
              <div className="text-[10px] text-muted-foreground mono mt-0.5">{u.name}</div>
            </motion.button>
          ))}
        </div>

        {/* Active use case detail */}
        <AnimatePresence mode="wait">
          <motion.div
            key={uc.id}
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.4 }}
            className="mt-10 grid lg:grid-cols-2 gap-6"
          >
            {/* Left: description */}
            <div className="p-6 rounded-2xl border border-border/60 bg-card/60">
              <div className="flex items-start gap-4 mb-4">
                <div
                  className="p-3 rounded-xl shrink-0"
                  style={{ background: uc.color + "20", color: uc.color }}
                >
                  <uc.icon className="w-7 h-7" strokeWidth={1.5} />
                </div>
                <div>
                  <h3 className="text-xl font-bold ar">{uc.nameAr}</h3>
                  <p className="text-xs text-muted-foreground mono">{uc.name}</p>
                </div>
              </div>

              <div className="space-y-4">
                <div>
                  <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-1 mono">
                    description
                  </div>
                  <p className="text-sm text-foreground/90 leading-relaxed ar">{uc.desc}</p>
                  <p className="text-xs text-muted-foreground italic mt-1">{uc.descEn}</p>
                </div>

                <div className="p-3 rounded-lg bg-primary/5 border border-primary/20">
                  <div className="text-[10px] text-primary uppercase tracking-wide mb-1 mono flex items-center gap-1">
                    <Zap className="w-3 h-3" />
                    why NAWA
                  </div>
                  <p className="text-sm text-foreground/90 leading-relaxed ar">{uc.whyNawa}</p>
                </div>

                <div>
                  <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-2 mono">
                    tags
                  </div>
                  <div className="flex flex-wrap gap-1.5">
                    {uc.tags.map((t) => (
                      <span
                        key={t}
                        className="px-2 py-0.5 rounded text-[10px] mono border"
                        style={{
                          borderColor: uc.color + "40",
                          color: uc.color,
                          background: uc.color + "10",
                        }}
                      >
                        {t}
                      </span>
                    ))}
                  </div>
                </div>

                <div className="pt-3 border-t border-border/40 flex items-center justify-between">
                  <span className="text-xs text-muted-foreground ar">القالب المقترح:</span>
                  <code className="text-xs mono text-primary">nawa create --template {uc.template}</code>
                </div>
              </div>
            </div>

            {/* Right: metrics */}
            <div className="space-y-4">
              <div className="grid grid-cols-3 gap-3">
                {uc.metrics.map((m, i) => (
                  <motion.div
                    key={m.label}
                    initial={{ opacity: 0, scale: 0.9 }}
                    animate={{ opacity: 1, scale: 1 }}
                    transition={{ duration: 0.3, delay: i * 0.1 }}
                    className="p-4 rounded-xl border border-border/60 bg-card/60 text-center"
                  >
                    <div
                      className="text-2xl font-bold mono"
                      style={{ color: uc.color }}
                    >
                      {m.value}
                    </div>
                    <div className="text-[10px] text-foreground mt-1 ar">{m.label}</div>
                    {m.sub && <div className="text-[9px] text-muted-foreground">{m.sub}</div>}
                  </motion.div>
                ))}
              </div>

              {/* Highlights */}
              <div className="p-5 rounded-2xl border border-border/60 bg-card/60">
                <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-3 mono">
                  highlights
                </div>
                <div className="space-y-2">
                  {[
                    { icon: TrendingUp, label: "أداء عالٍ", desc: `${uc.metrics[0].label}: ${uc.metrics[0].value}` },
                    { icon: Shield, label: "أمان مدمج", desc: "TLS + WAF + auth افتراضياً" },
                    { icon: Zap, label: "تكلفة منخفضة", desc: "يعمل على VPS اقتصادي" },
                  ].map((h) => (
                    <div key={h.label} className="flex items-center gap-3 p-2 rounded-lg bg-card/40">
                      <div className="p-1.5 rounded bg-primary/15 text-primary shrink-0">
                        <h.icon className="w-3.5 h-3.5" strokeWidth={1.5} />
                      </div>
                      <div className="flex-1">
                        <div className="text-xs font-medium ar">{h.label}</div>
                        <div className="text-[10px] text-muted-foreground ar">{h.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* CTA */}
              <div className="p-5 rounded-2xl border border-primary/30 bg-gradient-to-br from-primary/10 to-accent/5">
                <div className="flex items-center gap-2 mb-2">
                  <ArrowLeft className="w-4 h-4 text-primary" />
                  <span className="text-sm font-medium ar">جاهز للبدء؟</span>
                </div>
                <p className="text-xs text-muted-foreground mb-3 ar">
                  ابدأ بهذا القالب في أقل من دقيقتين
                </p>
                <code className="block px-3 py-2 rounded-lg bg-card/60 border border-border/40 text-xs mono text-primary">
                  $ nawa create my-{uc.id} --template {uc.template}
                </code>
              </div>
            </div>
          </motion.div>
        </AnimatePresence>
      </div>
    </section>
  );
}
