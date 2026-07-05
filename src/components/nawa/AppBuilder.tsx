"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Rocket,
  FileCode2,
  Server,
  Database,
  ShoppingBag,
  PenTool,
  Newspaper,
  MessageSquare,
  Calendar,
  CheckCircle2,
  ChevronLeft,
  Copy,
  Check,
  Terminal,
  Folder,
  FileText,
  Box,
  ArrowLeft,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

type Template = {
  id: string;
  name: string;
  nameAr: string;
  icon: typeof PenTool;
  desc: string;
  pages: string[];
  endpoints: string[];
  dbSchema: { table: string; fields: string[] }[];
  dockerSize: string;
  ramUsage: string;
};

const TEMPLATES: Template[] = [
  {
    id: "blog",
    name: "Blog / CMS",
    nameAr: "مدونة / نظام محتوى",
    icon: Newspaper,
    desc: "مدونة كاملة بـ SSR + SEO optimization + admin panel + comments + tags.",
    pages: ["/", "/post/:slug", "/admin", "/admin/new", "/tag/:tag"],
    endpoints: ["GET /api/posts", "POST /api/posts", "PUT /api/posts/:id", "DELETE /api/posts/:id"],
    dbSchema: [
      { table: "posts", fields: ["id", "title", "slug", "body", "tags[]", "published_at"] },
      { table: "comments", fields: ["id", "post_id", "author", "body", "created_at"] },
      { table: "authors", fields: ["id", "name", "email", "avatar"] },
    ],
    dockerSize: "16 MB",
    ramUsage: "52 MB",
  },
  {
    id: "saas",
    name: "SaaS Starter",
    nameAr: "تطبيق SaaS",
    icon: Box,
    desc: "Multi-tenant SaaS مع subscriptions + team management + billing + dashboard.",
    pages: ["/", "/login", "/dashboard", "/team", "/billing", "/settings"],
    endpoints: ["POST /api/auth/login", "GET /api/team", "POST /api/subscription", "GET /api/usage"],
    dbSchema: [
      { table: "users", fields: ["id", "email", "password_hash", "tenant_id"] },
      { table: "tenants", fields: ["id", "name", "plan", "seats"] },
      { table: "subscriptions", fields: ["id", "tenant_id", "stripe_id", "status"] },
      { table: "usage", fields: ["id", "tenant_id", "metric", "value", "at"] },
    ],
    dockerSize: "18 MB",
    ramUsage: "63 MB",
  },
  {
    id: "shop",
    name: "E-commerce",
    nameAr: "متجر إلكتروني",
    icon: ShoppingBag,
    desc: "متجر بـ product catalog + cart + checkout + orders + inventory + admin.",
    pages: ["/", "/product/:id", "/cart", "/checkout", "/orders", "/admin/products"],
    endpoints: ["GET /api/products", "POST /api/cart", "POST /api/checkout", "GET /api/orders/:id"],
    dbSchema: [
      { table: "products", fields: ["id", "name", "price", "stock", "image_url"] },
      { table: "carts", fields: ["id", "user_id", "items[]", "total"] },
      { table: "orders", fields: ["id", "user_id", "items[]", "total", "status"] },
      { table: "inventory", fields: ["product_id", "qty", "reserved"] },
    ],
    dockerSize: "19 MB",
    ramUsage: "68 MB",
  },
  {
    id: "realtime",
    name: "Realtime Chat",
    nameAr: "تطبيق دردشة",
    icon: MessageSquare,
    desc: "تطبيق chat حي بـ WebSocket + presence + typing indicators + message history.",
    pages: ["/", "/login", "/channels", "/channel/:id", "/dm/:userId"],
    endpoints: ["WS /ws", "GET /api/channels", "POST /api/messages", "GET /api/history/:channelId"],
    dbSchema: [
      { table: "users", fields: ["id", "name", "avatar", "status", "last_seen"] },
      { table: "channels", fields: ["id", "name", "type", "members[]"] },
      { table: "messages", fields: ["id", "channel_id", "user_id", "body", "at"] },
    ],
    dockerSize: "17 MB",
    ramUsage: "58 MB",
  },
  {
    id: "booking",
    name: "Booking System",
    nameAr: "نظام حجوزات",
    icon: Calendar,
    desc: "نظام حجز مواعيد بـ calendar + slots + payments + notifications + admin.",
    pages: ["/", "/book", "/calendar", "/my-bookings", "/admin/slots"],
    endpoints: ["GET /api/slots", "POST /api/book", "DELETE /api/booking/:id", "GET /api/calendar"],
    dbSchema: [
      { table: "services", fields: ["id", "name", "duration", "price"] },
      { table: "slots", fields: ["id", "service_id", "start", "end", "booked"] },
      { table: "bookings", fields: ["id", "slot_id", "user_id", "status"] },
    ],
    dockerSize: "16 MB",
    ramUsage: "55 MB",
  },
  {
    id: "portfolio",
    name: "Portfolio",
    nameAr: "بورتفوليو شخصي",
    icon: PenTool,
    desc: "بورتفوليو بـ projects + blog section + contact form + analytics + dark mode.",
    pages: ["/", "/projects", "/projects/:id", "/blog", "/contact"],
    endpoints: ["GET /api/projects", "POST /api/contact", "GET /api/stats"],
    dbSchema: [
      { table: "projects", fields: ["id", "title", "desc", "tags[]", "url"] },
      { table: "messages", fields: ["id", "name", "email", "body", "at"] },
      { table: "stats", fields: ["path", "views", "unique_visitors"] },
    ],
    dockerSize: "14 MB",
    ramUsage: "44 MB",
  },
];

export function AppBuilder() {
  const [selected, setSelected] = useState<Template | null>(null);
  const [step, setStep] = useState<"select" | "preview" | "deploy">("select");
  const [copied, setCopied] = useState(false);

  const selectTemplate = (t: Template) => {
    setSelected(t);
    setStep("preview");
  };

  const generateCommand = (t: Template) =>
    `# Create a new NAWA project from template
nawa create my-app --template ${t.id}

# Enter the project
cd my-app

# Run the dev server
nawa dev

# Deploy to your VPS (single command)
nawa deploy --target ssh://user@your-vps.com`;

  const handleCopy = () => {
    if (selected) {
      navigator.clipboard.writeText(generateCommand(selected));
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <section id="builder" className="relative py-24 lg:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="بناء أي مشروع ويب"
          eyebrowEn="App Builder"
          title="جاهز لأي مشروع ويب"
          titleEn="Ready for any web project"
          desc="NAWA يأتي مع 6 قوالب جاهزة للإنتاج، كل واحدة بـ SSR + DB schema + API endpoints + auth. اختر القالب، شغّل أمر واحد، وستحصل على تطبيق كامل يعمل على Docker."
          descEn="Six production-ready templates — each with SSR, DB schema, API endpoints, and auth. Pick one, run a single command, get a full Docker-ready app."
        />

        {/* Steps indicator */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 flex items-center justify-center gap-2"
        >
          {[
            { id: "select", label: "اختر القالب" },
            { id: "preview", label: "معاينة" },
            { id: "deploy", label: "نشر" },
          ].map((s, i) => (
            <div key={s.id} className="flex items-center gap-2">
              <div
                className={`flex items-center gap-2 px-3 py-1.5 rounded-full border text-xs transition-all ${
                  step === s.id
                    ? "border-primary bg-primary/10 text-primary"
                    : "border-border/60 text-muted-foreground"
                }`}
              >
                <span className={`w-4 h-4 rounded-full grid place-items-center text-[10px] mono ${
                  step === s.id ? "bg-primary text-primary-foreground" : "bg-muted"
                }`}>
                  {i + 1}
                </span>
                <span className="ar">{s.label}</span>
              </div>
              {i < 2 && <ChevronLeft className="w-3 h-3 text-muted-foreground" />}
            </div>
          ))}
        </motion.div>

        {/* Content */}
        <div className="mt-10">
          <AnimatePresence mode="wait">
            {step === "select" && (
              <motion.div
                key="select"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="grid sm:grid-cols-2 lg:grid-cols-3 gap-4"
              >
                {TEMPLATES.map((t, i) => (
                  <motion.button
                    key={t.id}
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ duration: 0.3, delay: i * 0.05 }}
                    onClick={() => selectTemplate(t)}
                    className="group text-right p-5 rounded-2xl border border-border/60 bg-card/60 hover:border-primary/40 hover:bg-card transition-all"
                  >
                    <div className="flex items-start justify-between mb-3">
                      <div className="p-2 rounded-lg bg-primary/15 text-primary group-hover:scale-110 transition-transform">
                        <t.icon className="w-5 h-5" strokeWidth={1.5} />
                      </div>
                      <ChevronLeft className="w-4 h-4 text-muted-foreground group-hover:text-primary group-hover:-translate-x-1 transition-all" />
                    </div>
                    <h4 className="text-base font-semibold ar">{t.nameAr}</h4>
                    <p className="text-[10px] text-muted-foreground mono mt-0.5">{t.name}</p>
                    <p className="text-xs text-muted-foreground mt-2 leading-relaxed ar line-clamp-2">
                      {t.desc}
                    </p>
                    <div className="mt-3 flex items-center gap-3 text-[10px] text-muted-foreground">
                      <span className="mono">{t.dockerSize}</span>
                      <span>·</span>
                      <span className="mono">{t.ramUsage} RAM</span>
                    </div>
                  </motion.button>
                ))}
              </motion.div>
            )}

            {step === "preview" && selected && (
              <motion.div
                key="preview"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="grid lg:grid-cols-2 gap-6"
              >
                {/* Left: project structure */}
                <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
                  <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
                    <div className="flex items-center gap-2">
                      <Folder className="w-4 h-4 text-primary" />
                      <span className="text-sm font-medium mono">my-app/</span>
                    </div>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setStep("select")}
                      className="h-7 text-xs"
                    >
                      <ArrowLeft className="w-3 h-3" />
                      <span className="mr-1 ar">رجوع</span>
                    </Button>
                  </div>
                  <div className="p-4 font-mono text-xs space-y-1">
                    <FileTreeLine icon={Folder} name="my-app/" level={0} />
                    <FileTreeLine icon={FileText} name="Cargo.toml" level={1} comment="Rust deps" />
                    <FileTreeLine icon={FileText} name="nawa.toml" level={1} comment="NAWA config" />
                    <FileTreeLine icon={Folder} name="src/" level={1} />
                    <FileTreeLine icon={FileCode2} name="main.rs" level={2} comment="entry" />
                    <FileTreeLine icon={Folder} name="routes/" level={2} />
                    {selected.pages.map((p, i) => (
                      <FileTreeLine
                        key={p}
                        icon={FileCode2}
                        name={`${p === "/" ? "index" : p.replace(/\//g, "").replace(":", "")}.rs`}
                        level={3}
                        comment={i === 0 ? "SSR page" : ""}
                      />
                    ))}
                    <FileTreeLine icon={Folder} name="db/" level={2} />
                    <FileTreeLine icon={Database} name="schema.nawa" level={3} comment="NAWA-DB schema" />
                    <FileTreeLine icon={Folder} name="templates/" level={2} />
                    <FileTreeLine icon={FileText} name="*.html" level={3} comment="SSR templates" />
                    <FileTreeLine icon={FileText} name="Dockerfile" level={1} comment="auto-generated" />
                    <FileTreeLine icon={FileText} name="docker-compose.yml" level={1} />
                  </div>
                </div>

                {/* Right: template details */}
                <div className="space-y-4">
                  <div className="p-5 rounded-2xl border border-border/60 bg-card/60">
                    <div className="flex items-start gap-3 mb-4">
                      <div className="p-2.5 rounded-lg bg-primary/15 text-primary">
                        <selected.icon className="w-6 h-6" strokeWidth={1.5} />
                      </div>
                      <div className="flex-1">
                        <h3 className="text-lg font-bold ar">{selected.nameAr}</h3>
                        <p className="text-xs text-muted-foreground mono">{selected.name}</p>
                      </div>
                      <div className="text-right">
                        <div className="text-xs text-muted-foreground">Docker size</div>
                        <div className="text-sm font-bold mono text-primary">{selected.dockerSize}</div>
                      </div>
                    </div>
                    <p className="text-sm text-muted-foreground leading-relaxed ar">{selected.desc}</p>
                  </div>

                  {/* Stats grid */}
                  <div className="grid grid-cols-2 gap-3">
                    <div className="p-3 rounded-lg border border-border/60 bg-card/60">
                      <div className="text-[10px] text-muted-foreground mono mb-1">PAGES</div>
                      <div className="text-2xl font-bold text-primary mono">{selected.pages.length}</div>
                    </div>
                    <div className="p-3 rounded-lg border border-border/60 bg-card/60">
                      <div className="text-[10px] text-muted-foreground mono mb-1">API ENDPOINTS</div>
                      <div className="text-2xl font-bold text-accent mono">{selected.endpoints.length}</div>
                    </div>
                    <div className="p-3 rounded-lg border border-border/60 bg-card/60">
                      <div className="text-[10px] text-muted-foreground mono mb-1">DB TABLES</div>
                      <div className="text-2xl font-bold text-primary mono">{selected.dbSchema.length}</div>
                    </div>
                    <div className="p-3 rounded-lg border border-border/60 bg-card/60">
                      <div className="text-[10px] text-muted-foreground mono mb-1">RAM USAGE</div>
                      <div className="text-2xl font-bold text-accent mono">{selected.ramUsage}</div>
                    </div>
                  </div>

                  {/* DB schema preview */}
                  <div className="p-4 rounded-xl border border-border/60 bg-card/60">
                    <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-2 mono">
                      DB Schema · nawa-db
                    </div>
                    <div className="space-y-2">
                      {selected.dbSchema.map((tbl) => (
                        <div key={tbl.table} className="text-xs">
                          <code className="text-primary mono font-semibold">{tbl.table}</code>
                          <span className="text-muted-foreground mx-1">:</span>
                          <code className="text-muted-foreground mono">
                            {tbl.fields.join(", ")}
                          </code>
                        </div>
                      ))}
                    </div>
                  </div>

                  <Button
                    className="w-full bg-primary hover:bg-primary/90 text-primary-foreground"
                    size="lg"
                    onClick={() => setStep("deploy")}
                  >
                    <Rocket className="w-4 h-4 ml-1" />
                    <span className="ar">المتابعة للنشر</span>
                    <ChevronLeft className="w-4 h-4" />
                  </Button>
                </div>
              </motion.div>
            )}

            {step === "deploy" && selected && (
              <motion.div
                key="deploy"
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                className="max-w-3xl mx-auto"
              >
                <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
                  <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
                    <div className="flex items-center gap-2">
                      <Terminal className="w-4 h-4 text-primary" />
                      <span className="text-sm font-medium mono">deploy.sh</span>
                    </div>
                    <Button variant="ghost" size="sm" onClick={handleCopy} className="h-7 text-xs">
                      {copied ? <Check className="w-3 h-3 text-accent" /> : <Copy className="w-3 h-3" />}
                      <span className="mr-1">{copied ? "Copied" : "Copy"}</span>
                    </Button>
                  </div>
                  <pre className="p-4 font-mono text-xs leading-relaxed overflow-x-auto scrollbar-narrow">
                    <code className="text-foreground/90 whitespace-pre">
                      {generateCommand(selected).split("\n").map((line, i) => (
                        <div key={i} className="hover:bg-primary/5 -mx-4 px-4">
                          <span className="text-muted-foreground/40 select-none mr-3 inline-block w-4 text-right">
                            {i + 1}
                          </span>
                          <span className={line.startsWith("#") ? "text-green-600/70 italic" : ""}>
                            {line || " "}
                          </span>
                        </div>
                      ))}
                    </code>
                  </pre>
                </div>

                {/* Deploy timeline */}
                <div className="mt-6 space-y-3">
                  {[
                    { step: "nawa create", desc: "إنشاء المشروع من القالب", time: "~2s" },
                    { step: "nawa dev", desc: "تشغيل خادم التطوير مع hot reload", time: "~200ms" },
                    { step: "nawa deploy", desc: "بناء Docker image + رفع للسيرفر", time: "~30s" },
                    { step: "running", desc: "تطبيقك يعمل على https://...", time: "live" },
                  ].map((s, i) => (
                    <motion.div
                      key={s.step}
                      initial={{ opacity: 0, x: -20 }}
                      animate={{ opacity: 1, x: 0 }}
                      transition={{ delay: i * 0.15 }}
                      className="flex items-center gap-3 p-3 rounded-lg border border-border/60 bg-card/60"
                    >
                      <CheckCircle2 className="w-5 h-5 text-accent shrink-0" />
                      <code className="text-sm mono text-primary shrink-0">{s.step}</code>
                      <span className="text-xs text-muted-foreground flex-1 ar">{s.desc}</span>
                      <Badge variant="outline" className="mono text-[10px]">
                        {s.time}
                      </Badge>
                    </motion.div>
                  ))}
                </div>

                <div className="mt-6 flex gap-3">
                  <Button
                    variant="outline"
                    onClick={() => setStep("preview")}
                    className="flex-1"
                  >
                    <ArrowLeft className="w-4 h-4 ml-1" />
                    <span className="ar">رجوع للمعاينة</span>
                  </Button>
                  <Button
                    onClick={() => {
                      setStep("select");
                      setSelected(null);
                    }}
                    className="flex-1 bg-primary hover:bg-primary/90 text-primary-foreground"
                  >
                    <span className="ar">قالب آخر</span>
                  </Button>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </section>
  );
}

function FileTreeLine({
  icon: Icon,
  name,
  level,
  comment,
}: {
  icon: typeof Folder;
  name: string;
  level: number;
  comment?: string;
}) {
  return (
    <div
      className="flex items-center gap-2 hover:bg-primary/5 -mx-4 px-4 py-0.5"
      style={{ paddingRight: `${1 + level * 1.5}rem` }}
    >
      <Icon className="w-3 h-3 text-muted-foreground shrink-0" />
      <span className={name.endsWith("/") ? "text-primary" : "text-foreground/80"}>{name}</span>
      {comment && (
        <>
          <span className="text-muted-foreground/60 text-[10px]">—</span>
          <span className="text-muted-foreground/60 text-[10px] italic">{comment}</span>
        </>
      )}
    </div>
  );
}
