"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Boxes,
  Search,
  Lock,
  Zap,
  Download,
  Star,
  Check,
  Shield,
  Code2,
  Database,
  Mail,
  CreditCard,
  Search as SearchIcon,
  BarChart3,
  Globe,
  Lock as LockIcon,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

type Plugin = {
  id: string;
  name: string;
  nameAr: string;
  author: string;
  version: string;
  size: string;
  downloads: number;
  rating: number;
  category: "auth" | "db" | "search" | "analytics" | "payment" | "comms" | "security";
  icon: typeof Lock;
  color: string;
  desc: string;
  descEn: string;
  capabilities: string[];
  verified: boolean;
};

const PLUGINS: Plugin[] = [
  {
    id: "auth-jwt",
    name: "auth-jwt",
    nameAr: "مصادقة JWT",
    author: "@nawa-official",
    version: "0.2.1",
    size: "12 KB",
    downloads: 18432,
    rating: 4.9,
    category: "auth",
    icon: Lock,
    color: "oklch(0.72 0.19 47)",
    desc: "مصادقة JWT كاملة بـ EdDSA + refresh tokens + revocation.",
    descEn: "Complete JWT auth with EdDSA + refresh tokens + revocation.",
    capabilities: ["sign", "verify", "refresh", "revoke"],
    verified: true,
  },
  {
    id: "auth-oauth",
    name: "auth-oauth",
    nameAr: "مصادقة OAuth",
    author: "@nawa-community",
    version: "0.1.4",
    size: "18 KB",
    downloads: 8721,
    rating: 4.7,
    category: "auth",
    icon: Lock,
    color: "oklch(0.78 0.16 165)",
    desc: "OAuth 2.0 مع Google, GitHub, Apple, GitLab providers.",
    descEn: "OAuth 2.0 with Google, GitHub, Apple, GitLab providers.",
    capabilities: ["google", "github", "apple", "gitlab"],
    verified: true,
  },
  {
    id: "cache-redis",
    name: "cache-redis",
    nameAr: "كاش متوافق مع Redis",
    author: "@nawa-official",
    version: "0.1.0",
    size: "8 KB",
    downloads: 12044,
    rating: 4.6,
    category: "db",
    icon: Database,
    color: "oklch(0.65 0.20 25)",
    desc: "Redis-compatible cache layer فوق NAWA-DB.",
    descEn: "Redis-compatible cache layer on top of NAWA-DB.",
    capabilities: ["GET", "SET", "EXPIRE", "TTL"],
    verified: true,
  },
  {
    id: "search-fts",
    name: "search-fts",
    nameAr: "بحث نصي كامل",
    author: "@nawa-official",
    version: "0.3.0",
    size: "24 KB",
    downloads: 9821,
    rating: 4.8,
    category: "search",
    icon: Search,
    color: "oklch(0.65 0.20 300)",
    desc: "بحث نصي كامل بـ BM25 + stemming + Arabic support.",
    descEn: "Full-text search with BM25 + stemming + Arabic support.",
    capabilities: ["index", "search", "highlight", "suggest"],
    verified: true,
  },
  {
    id: "analytics-privacy",
    name: "analytics",
    nameAr: "تحليلات تحترم الخصوصية",
    author: "@nawa-community",
    version: "0.1.2",
    size: "15 KB",
    downloads: 6432,
    rating: 4.5,
    category: "analytics",
    icon: BarChart3,
    color: "oklch(0.80 0.15 90)",
    desc: "تحليلات استخدام بدون cookies، GDPR compliant.",
    descEn: "Cookieless analytics, GDPR compliant.",
    capabilities: ["pageview", "event", "funnel", "retention"],
    verified: true,
  },
  {
    id: "payment-stripe",
    name: "payment-stripe",
    nameAr: "بوابات دفع Stripe",
    author: "@nawa-official",
    version: "0.2.0",
    size: "32 KB",
    downloads: 11203,
    rating: 4.9,
    category: "payment",
    icon: CreditCard,
    color: "oklch(0.65 0.18 200)",
    desc: "تكامل Stripe كامل: checkout, subscriptions, webhooks.",
    descEn: "Full Stripe integration: checkout, subscriptions, webhooks.",
    capabilities: ["checkout", "subscribe", "webhook", "refund"],
    verified: true,
  },
  {
    id: "email-smtp",
    name: "email-smtp",
    nameAr: "إرسال بريد SMTP",
    author: "@nawa-community",
    version: "0.1.5",
    size: "14 KB",
    downloads: 5234,
    rating: 4.4,
    category: "comms",
    icon: Mail,
    color: "oklch(0.70 0.20 5)",
    desc: "إرسال بريد عبر SMTP + templating + attachments.",
    descEn: "Send email via SMTP + templating + attachments.",
    capabilities: ["send", "template", "attach", "queue"],
    verified: true,
  },
  {
    id: "waf-shield",
    name: "waf-shield",
    nameAr: "جدار حماية معزز",
    author: "@nawa-official",
    version: "0.4.1",
    size: "28 KB",
    downloads: 14200,
    rating: 4.8,
    category: "security",
    icon: Shield,
    color: "oklch(0.65 0.22 25)",
    desc: "WAF مع ML-based anomaly detection + OWASP Top 10 rules.",
    descEn: "WAF with ML-based anomaly detection + OWASP Top 10.",
    capabilities: ["detect", "block", "rate-limit", "log"],
    verified: true,
  },
  {
    id: "cdn-edge",
    name: "cdn-edge",
    nameAr: "CDN على الـ edge",
    author: "@nawa-community",
    version: "0.0.9",
    size: "11 KB",
    downloads: 3120,
    rating: 4.2,
    category: "comms",
    icon: Globe,
    color: "oklch(0.78 0.16 165)",
    desc: "تكامل CDN مع 200+ edge locations + cache invalidation.",
    descEn: "CDN integration with 200+ edge locations + invalidation.",
    capabilities: ["push", "invalidate", "purge", "stats"],
    verified: false,
  },
];

const CATEGORIES: Array<{ id: Plugin["category"] | "all"; label: string; labelAr: string }> = [
  { id: "all", label: "All", labelAr: "الكل" },
  { id: "auth", label: "Auth", labelAr: "مصادقة" },
  { id: "db", label: "Database", labelAr: "قاعدة بيانات" },
  { id: "search", label: "Search", labelAr: "بحث" },
  { id: "analytics", label: "Analytics", labelAr: "تحليلات" },
  { id: "payment", label: "Payments", labelAr: "مدفوعات" },
  { id: "comms", label: "Comms", labelAr: "اتصالات" },
  { id: "security", label: "Security", labelAr: "أمان" },
];

export function PluginMarketplace() {
  const [category, setCategory] = useState<Plugin["category"] | "all">("all");
  const [query, setQuery] = useState("");
  const [installed, setInstalled] = useState<Set<string>>(new Set());
  const [installing, setInstalling] = useState<string | null>(null);

  const filtered = PLUGINS.filter(
    (p) =>
      (category === "all" || p.category === category) &&
      (query === "" ||
        p.name.toLowerCase().includes(query.toLowerCase()) ||
        p.nameAr.includes(query) ||
        p.desc.includes(query))
  );

  const toggleInstall = (id: string) => {
    if (installed.has(id)) {
      setInstalled((s) => {
        const next = new Set(s);
        next.delete(id);
        return next;
      });
      return;
    }
    setInstalling(id);
    setTimeout(() => {
      setInstalled((s) => new Set(s).add(id));
      setInstalling(null);
    }, 1200);
  };

  const totalDownloads = PLUGINS.reduce((sum, p) => sum + p.downloads, 0);

  return (
    <section id="marketplace" className="relative py-24 lg:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="سوق الإضافات"
          eyebrowEn="Plugin Marketplace"
          title="WASM Plugins · أمان بلا حدود"
          titleEn="WASM Plugins — endless extensibility"
          desc="كل plugin يعمل في sandbox WASM آمن — لا filesystem، لا شبكة مباشرة. فقط الـ capabilities المرخّصة. جرّب تثبيت إضافة وراقب كيف تُحمَّل وتُفعَّل دون إعادة تشغيل."
          descEn="Every plugin runs in a sandboxed WASM environment. Try installing one and watch it load and activate without restart."
        />

        {/* Stats strip */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 grid grid-cols-2 lg:grid-cols-4 gap-3"
        >
          {[
            { label: "إجمالي الإضافات", value: `${PLUGINS.length}+`, en: "plugins" },
            { label: "تحميلات", value: `${(totalDownloads / 1000).toFixed(0)}k`, en: "downloads" },
            { label: "متوسط الحجم", value: "17 KB", en: "avg size" },
            { label: "Sandboxed", value: "100%", en: "via WASM" },
          ].map((s) => (
            <div key={s.label} className="p-4 rounded-xl border border-border/60 bg-card/60 text-center">
              <div className="text-2xl font-bold text-primary mono">{s.value}</div>
              <div className="text-xs text-foreground mt-0.5 ar">{s.label}</div>
              <div className="text-[10px] text-muted-foreground">{s.en}</div>
            </div>
          ))}
        </motion.div>

        {/* Search + categories */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-8 flex flex-col sm:flex-row gap-3"
        >
          <div className="relative flex-1">
            <SearchIcon className="w-4 h-4 absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="ابحث عن plugin..."
              className="w-full bg-card/60 border border-border/60 rounded-lg pr-10 pl-3 py-2 text-sm outline-none focus:border-primary/40 ar"
            />
          </div>
          <div className="flex flex-wrap gap-1.5">
            {CATEGORIES.map((c) => (
              <button
                key={c.id}
                onClick={() => setCategory(c.id)}
                className={`px-3 py-2 rounded-lg text-xs border transition-all ar ${
                  category === c.id
                    ? "border-primary bg-primary/10 text-primary"
                    : "border-border/60 bg-card/40 text-muted-foreground hover:text-foreground hover:border-border"
                }`}
              >
                {c.labelAr}
              </button>
            ))}
          </div>
        </motion.div>

        {/* Plugin grid */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-6 grid sm:grid-cols-2 lg:grid-cols-3 gap-4"
        >
          <AnimatePresence>
            {filtered.map((p) => {
              const isInstalled = installed.has(p.id);
              const isInstalling = installing === p.id;
              return (
                <motion.div
                  key={p.id}
                  layout
                  initial={{ opacity: 0, scale: 0.95 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.95 }}
                  transition={{ duration: 0.3 }}
                  className="group relative p-5 rounded-2xl border border-border/60 bg-card/60 hover:border-primary/40 hover:bg-card transition-all"
                >
                  {/* Header */}
                  <div className="flex items-start justify-between mb-3">
                    <div
                      className="p-2 rounded-lg"
                      style={{ background: p.color + "20", color: p.color }}
                    >
                      <p.icon className="w-5 h-5" strokeWidth={1.5} />
                    </div>
                    {p.verified ? (
                      <Badge variant="outline" className="mono text-[9px] border-accent/40 text-accent">
                        <Check className="w-2.5 h-2.5 ml-0.5" />
                        verified
                      </Badge>
                    ) : (
                      <Badge variant="outline" className="mono text-[9px] border-muted-foreground/40 text-muted-foreground">
                        community
                      </Badge>
                    )}
                  </div>

                  {/* Name + desc */}
                  <div className="mb-3">
                    <div className="flex items-center gap-2">
                      <code className="text-sm font-semibold mono" style={{ color: p.color }}>
                        {p.name}
                      </code>
                      <span className="text-[10px] text-muted-foreground mono">v{p.version}</span>
                    </div>
                    <div className="text-xs text-foreground/90 mt-1 ar">{p.nameAr}</div>
                    <p className="text-xs text-muted-foreground mt-2 leading-relaxed ar line-clamp-2">
                      {p.desc}
                    </p>
                  </div>

                  {/* Capabilities */}
                  <div className="flex flex-wrap gap-1 mb-3">
                    {p.capabilities.map((c) => (
                      <code
                        key={c}
                        className="px-1.5 py-0.5 rounded text-[9px] mono bg-muted/40 text-muted-foreground"
                      >
                        {c}
                      </code>
                    ))}
                  </div>

                  {/* Meta */}
                  <div className="flex items-center justify-between text-[10px] text-muted-foreground mb-3">
                    <div className="flex items-center gap-2">
                      <span className="flex items-center gap-0.5">
                        <Download className="w-2.5 h-2.5" />
                        {(p.downloads / 1000).toFixed(1)}k
                      </span>
                      <span className="flex items-center gap-0.5">
                        <Star className="w-2.5 h-2.5 text-yellow-500" />
                        {p.rating}
                      </span>
                      <span className="mono">{p.size}</span>
                    </div>
                    <span className="mono">{p.author}</span>
                  </div>

                  {/* Install button */}
                  <Button
                    size="sm"
                    onClick={() => toggleInstall(p.id)}
                    disabled={isInstalling}
                    className={`w-full h-8 text-xs ${
                      isInstalled
                        ? "bg-accent/15 text-accent hover:bg-accent/25 border border-accent/40"
                        : "bg-primary hover:bg-primary/90 text-primary-foreground"
                    }`}
                    variant={isInstalled ? "outline" : "default"}
                  >
                    {isInstalling ? (
                      <>
                        <Zap className="w-3 h-3 ml-1 animate-nawa-pulse" />
                        <span className="ar">جاري التثبيت...</span>
                      </>
                    ) : isInstalled ? (
                      <>
                        <Check className="w-3 h-3 ml-1" />
                        <span className="ar">مُثبَّت</span>
                      </>
                    ) : (
                      <>
                        <Download className="w-3 h-3 ml-1" />
                        <span className="ar">تثبيت</span>
                      </>
                    )}
                  </Button>

                  {/* Sandbox badge */}
                  <div className="mt-2 pt-2 border-t border-border/40 flex items-center gap-1.5 text-[9px] text-muted-foreground">
                    <LockIcon className="w-2.5 h-2.5" />
                    <span className="mono">sandboxed · WASM · zero-fs</span>
                  </div>
                </motion.div>
              );
            })}
          </AnimatePresence>
        </motion.div>

        {/* Installed plugins summary */}
        {installed.size > 0 && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            className="mt-6 p-5 rounded-2xl border border-primary/30 bg-gradient-to-br from-primary/10 to-accent/5"
          >
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-2">
                <Boxes className="w-4 h-4 text-primary" />
                <h4 className="text-sm font-semibold ar">
                  الإضافات المُثبَّتة ({installed.size})
                </h4>
              </div>
              <Badge variant="outline" className="mono text-[10px] border-accent/40 text-accent">
                hot-loaded · no restart
              </Badge>
            </div>
            <div className="flex flex-wrap gap-2">
              {Array.from(installed).map((id) => {
                const p = PLUGINS.find((x) => x.id === id)!;
                return (
                  <div
                    key={id}
                    className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-card/60 border border-border/40"
                  >
                    <p.icon className="w-3 h-3" style={{ color: p.color }} />
                    <code className="text-xs mono">{p.name}</code>
                    <span className="text-[10px] text-muted-foreground">v{p.version}</span>
                    <span className="text-[10px] text-accent mono">· active</span>
                  </div>
                );
              })}
            </div>
            <div className="mt-3 text-xs text-muted-foreground ar">
              ✓ الإضافات تعمل الآن داخل الـ WASM sandbox. حجم إضافي للذاكرة: ~
              {Array.from(installed).reduce((sum, id) => {
                const p = PLUGINS.find((x) => x.id === id)!;
                return sum + parseInt(p.size);
              }, 0)}{" "}
              KB فقط.
            </div>
          </motion.div>
        )}
      </div>
    </section>
  );
}
