"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Shield,
  Lock,
  KeyRound,
  Eye,
  Ban,
  Fingerprint,
  Server,
  Globe,
  Cpu,
  FileCode2,
  CheckCircle2,
  AlertTriangle,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type Layer = {
  id: string;
  name: string;
  nameAr: string;
  icon: typeof Shield;
  color: "primary" | "accent" | "destructive";
  desc: string;
  descEn: string;
  examples: string[];
};

const LAYERS: Layer[] = [
  {
    id: "edge",
    name: "Edge Protection",
    nameAr: "حماية الحافة",
    icon: Globe,
    color: "accent",
    desc: "أول خط دفاع. DDoS mitigation، IP reputation scoring، geo-blocking، rate limiting على مستوى الـ edge node.",
    descEn: "First line of defense: DDoS mitigation, IP reputation, geo-blocking, edge rate-limiting.",
    examples: ["DDoS mitigation", "IP reputation", "Geo-blocking", "Edge rate-limit"],
  },
  {
    id: "waf",
    name: "WAF (Web Application Firewall)",
    nameAr: "جدار حماية التطبيق",
    icon: Ban,
    color: "destructive",
    desc: "كشف SQL injection, XSS, path traversal, SSRF. قواعد OWASP Top 10 مدمجة + تعلّم تلقائي للأنماط المشبوهة.",
    descEn: "Detects SQLi, XSS, path traversal, SSRF. OWASP Top 10 rules + automatic anomaly learning.",
    examples: ["SQLi detection", "XSS blocking", "Path traversal", "SSRF prevention"],
  },
  {
    id: "tls",
    name: "TLS 1.3 + 0-RTT",
    nameAr: "تشفير TLS 1.3",
    icon: Lock,
    color: "primary",
    desc: "TLS 1.3 مفعَّل افتراضياً مع 0-RTT للجلسات العائدة. شهادات Let's Encrypt تُجدَّد تلقائياً قبل انتهائها بـ 30 يوم.",
    descEn: "TLS 1.3 default with 0-RTT for resumed sessions. Let's Encrypt auto-renewal 30 days before expiry.",
    examples: ["TLS 1.3", "0-RTT resume", "Auto Let's Encrypt", "HSTS preload"],
  },
  {
    id: "auth",
    name: "Zero-Trust Auth",
    nameAr: "مصادقة عدم الثقة",
    icon: Fingerprint,
    color: "accent",
    desc: "كل طلب يجب أن يُثبت هويته — لا endpoint \"موثوق\" افتراضياً. JWT + refresh tokens + session fingerprinting.",
    descEn: "Every request must prove identity — no trusted endpoint by default. JWT + refresh + fingerprinting.",
    examples: ["JWT (EdDSA)", "Refresh tokens", "Device fingerprint", "MFA support"],
  },
  {
    id: "rbac",
    name: "RBAC + ABAC",
    nameAr: "تحكم بالوصول",
    icon: KeyRound,
    color: "primary",
    desc: "Role-Based + Attribute-Based Access Control. كل route يُصرّح بـ #[authorize(role=\"admin\", scope=\"write\")]",
    descEn: "Role + Attribute Based Access Control. Routes declare: #[authorize(role=\"admin\", scope=\"write\")]",
    examples: ["Roles (admin, user)", "Scopes (read, write)", "Resource ownership", "Tenant isolation"],
  },
  {
    id: "sandbox",
    name: "WASM Sandbox",
    nameAr: "صندوق رمل WASM",
    icon: Cpu,
    color: "destructive",
    desc: "كود المستخدم (plugins, handlers) يُنفذ في wasmtime sandbox. لا filesystem، لا شبكة مباشرة، لا fork — فقط الـ APIs المرخّصة.",
    descEn: "User code runs in wasmtime sandbox. No fs, no direct net, no fork — only authorized APIs.",
    examples: ["wasmtime isolation", "Capability-based", "No filesystem", "No raw sockets"],
  },
  {
    id: "audit",
    name: "Audit + Observability",
    nameAr: "تدقيق ورصد",
    icon: Eye,
    color: "accent",
    desc: "كل request يُولِّد OpenTelemetry span. كل mutation يُسجَّل في audit log غير قابل للتعديل (append-only).",
    descEn: "Every request generates OTel span. Every mutation logged to append-only audit log.",
    examples: ["OpenTelemetry", "Append-only audit", "Tamper-evident", "SIEM-ready"],
  },
  {
    id: "data",
    name: "Data-at-Rest Encryption",
    nameAr: "تشفير البيانات المخزنة",
    icon: FileCode2,
    color: "primary",
    desc: "SSTables مشفّرة بـ AES-256-GCM. المفاتيح في keyring منفصل، يمكن rotation دون إعادة تشفير كامل.",
    descEn: "SSTables encrypted with AES-256-GCM. Keys in separate keyring, rotation without full re-encrypt.",
    examples: ["AES-256-GCM", "Key rotation", "Per-table keys", "Memory-safe crypto"],
  },
];

export function SecurityLayer() {
  const [active, setActive] = useState<string>("edge");
  const activeLayer = LAYERS.find((l) => l.id === active)!;

  return (
    <section id="security" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-grid opacity-30 pointer-events-none" />
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="الأمان · Zero-Trust"
          eyebrowEn="Security & Zero-Trust"
          title="الأمان ليس إضافة — بل الطبقة الأولى"
          titleEn="Security isn't an add-on — it's the first layer"
          desc="NAWA مبني على مبدأ zero-trust: لا endpoint موثوق افتراضياً، لا طلب يصل دون مصادقة، لا كود مستخدم يعمل دون sandbox. ثماني طبقات حماية متداخلة، كل واحدة مستقلة وقابلة للتكوين."
          descEn="NAWA is built on zero-trust: no trusted endpoint by default, no request without auth, no user code without sandbox. Eight layered defenses, each independent and configurable."
        />

        {/* Visual: layered defense */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-14 grid lg:grid-cols-2 gap-8 items-start"
        >
          {/* Left: concentric layers visual */}
          <div className="relative aspect-square max-w-md mx-auto w-full">
            <svg viewBox="0 0 400 400" className="w-full h-full">
              {/* Concentric rings */}
              {LAYERS.map((layer, i) => {
                const isActive = layer.id === active;
                const radius = 180 - i * 22;
                const colorMap = {
                  primary: "oklch(0.72 0.19 47)",
                  accent: "oklch(0.78 0.16 165)",
                  destructive: "oklch(0.65 0.22 25)",
                };
                const color = colorMap[layer.color];
                return (
                  <motion.circle
                    key={layer.id}
                    cx="200"
                    cy="200"
                    r={radius}
                    fill={isActive ? color + "20" : "transparent"}
                    stroke={color}
                    strokeWidth={isActive ? 2.5 : 1}
                    strokeOpacity={isActive ? 1 : 0.4}
                    initial={{ scale: 0.95, opacity: 0 }}
                    whileInView={{ scale: 1, opacity: 1 }}
                    viewport={{ once: true }}
                    transition={{ duration: 0.4, delay: i * 0.05 }}
                    className="cursor-pointer transition-all"
                    onClick={() => setActive(layer.id)}
                  />
                );
              })}
              {/* Center core */}
              <circle cx="200" cy="200" r="28" fill="oklch(0.10 0.005 60)" stroke="oklch(0.72 0.19 47)" strokeWidth="2" />
              <text x="200" y="206" textAnchor="middle" fill="oklch(0.72 0.19 47)" fontSize="14" fontFamily="monospace" fontWeight="bold">
                APP
              </text>
            </svg>
            {/* Floating labels */}
            {LAYERS.slice(0, 4).map((layer, i) => {
              const angle = (i * 90 - 45) * (Math.PI / 180);
              const r = 200;
              const x = 200 + r * Math.cos(angle);
              const y = 200 + r * Math.sin(angle);
              return (
                <button
                  key={layer.id}
                  onClick={() => setActive(layer.id)}
                  className={`absolute -translate-x-1/2 -translate-y-1/2 px-2 py-1 rounded text-[10px] mono border transition-all ${
                    active === layer.id
                      ? "bg-card border-primary text-primary scale-110 z-10"
                      : "bg-card/60 border-border/60 text-muted-foreground hover:text-foreground"
                  }`}
                  style={{ left: `${(x / 400) * 100}%`, top: `${(y / 400) * 100}%` }}
                >
                  {layer.name}
                </button>
              );
            })}
            {LAYERS.slice(4).map((layer, i) => {
              const angle = (i * 90 + 45) * (Math.PI / 180);
              const r = 120;
              const x = 200 + r * Math.cos(angle);
              const y = 200 + r * Math.sin(angle);
              return (
                <button
                  key={layer.id}
                  onClick={() => setActive(layer.id)}
                  className={`absolute -translate-x-1/2 -translate-y-1/2 px-2 py-1 rounded text-[9px] mono border transition-all ${
                    active === layer.id
                      ? "bg-card border-primary text-primary scale-110 z-10"
                      : "bg-card/60 border-border/60 text-muted-foreground hover:text-foreground"
                  }`}
                  style={{ left: `${(x / 400) * 100}%`, top: `${(y / 400) * 100}%` }}
                >
                  {layer.name}
                </button>
              );
            })}
          </div>

          {/* Right: active layer details */}
          <div>
            <AnimatePresence mode="wait">
              <motion.div
                key={activeLayer.id}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                transition={{ duration: 0.3 }}
              >
                <div className="flex items-start gap-4 mb-5">
                  <div
                    className={`p-3 rounded-xl ${
                      activeLayer.color === "primary"
                        ? "bg-primary/15 text-primary"
                        : activeLayer.color === "accent"
                        ? "bg-accent/15 text-accent"
                        : "bg-destructive/15 text-destructive"
                    }`}
                  >
                    <activeLayer.icon className="w-6 h-6" strokeWidth={1.5} />
                  </div>
                  <div>
                    <h3 className="text-xl font-bold ar">{activeLayer.nameAr}</h3>
                    <p className="text-xs text-muted-foreground mono mt-0.5">{activeLayer.name}</p>
                  </div>
                </div>
                <p className="text-sm text-foreground/90 leading-relaxed ar mb-2">{activeLayer.desc}</p>
                <p className="text-xs text-muted-foreground italic mb-5">{activeLayer.descEn}</p>
                <div className="space-y-2">
                  {activeLayer.examples.map((ex) => (
                    <div key={ex} className="flex items-center gap-2">
                      <CheckCircle2 className="w-3.5 h-3.5 text-accent shrink-0" />
                      <code className="text-xs mono">{ex}</code>
                    </div>
                  ))}
                </div>
              </motion.div>
            </AnimatePresence>

            {/* Layer switcher */}
            <div className="mt-6 pt-6 border-t border-border/40">
              <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-2 ar">
                اختر طبقة
              </div>
              <div className="flex flex-wrap gap-1.5">
                {LAYERS.map((l) => (
                  <button
                    key={l.id}
                    onClick={() => setActive(l.id)}
                    className={`px-2.5 py-1 rounded text-[10px] mono border transition-all ${
                      active === l.id
                        ? l.color === "primary"
                          ? "bg-primary/15 border-primary text-primary"
                          : l.color === "accent"
                          ? "bg-accent/15 border-accent text-accent"
                          : "bg-destructive/15 border-destructive text-destructive"
                        : "border-border/60 text-muted-foreground hover:text-foreground"
                    }`}
                  >
                    {l.name}
                  </button>
                ))}
              </div>
            </div>
          </div>
        </motion.div>

        {/* Threat model strip */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-16 grid sm:grid-cols-3 gap-4"
        >
          {THREATS.map((t) => (
            <div
              key={t.threat}
              className="p-5 rounded-xl border border-border/60 bg-card/60"
            >
              <div className="flex items-center gap-2 mb-2">
                <AlertTriangle className="w-4 h-4 text-yellow-500/80" />
                <span className="text-sm font-medium ar">{t.threat}</span>
              </div>
              <p className="text-xs text-muted-foreground mb-3 ar">{t.desc}</p>
              <div className="flex items-center gap-2 pt-3 border-t border-border/40">
                <Shield className="w-3.5 h-3.5 text-accent" />
                <span className="text-xs text-accent ar">{t.mitigation}</span>
              </div>
            </div>
          ))}
        </motion.div>
      </div>
    </section>
  );
}

const THREATS = [
  {
    threat: "SQL Injection",
    desc: "محاولة حقن كود SQL خبيث عبر مدخلات المستخدم.",
    mitigation: "WAF + parameterized queries إلزامية",
  },
  {
    threat: "Container Escape",
    desc: "محاولة الخروج من حاوية Docker للوصول للنظام المضيف.",
    mitigation: "Non-root UID + seccomp + WASM sandbox",
  },
  {
    threat: "Credential Stuffing",
    desc: "محاولة تسجيل دخول بكميات كبيرة باستخدام بيانات مسروقة.",
    mitigation: "Rate limit + MFA + anomaly detection",
  },
];
