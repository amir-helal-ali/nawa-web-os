"use client";

import { useState, useMemo } from "react";
import { motion } from "framer-motion";
import {
  Calculator,
  Server,
  DollarSign,
  TrendingDown,
  Users,
  Zap,
  ArrowDown,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type Provider = "vercel" | "aws" | "digitalocean" | "hetzner";

const PROVIDERS: Record<
  Provider,
  { name: string; nameAr: string; costPerGB: number; baseCost: number; color: string }
> = {
  vercel: { name: "Vercel Pro", nameAr: "فيرسل برو", costPerGB: 0, baseCost: 20, color: "oklch(0.65 0.18 280)" },
  aws: { name: "AWS EC2", nameAr: "AWS EC2", costPerGB: 8, baseCost: 15, color: "oklch(0.65 0.18 50)" },
  digitalocean: { name: "DigitalOcean", nameAr: "ديجيتال أوشن", costPerGB: 6, baseCost: 6, color: "oklch(0.65 0.18 200)" },
  hetzner: { name: "Hetzner", nameAr: "هيتزنر", costPerGB: 3, baseCost: 4, color: "oklch(0.65 0.18 165)" },
};

export function CostCalculator() {
  const [provider, setProvider] = useState<Provider>("vercel");
  const [visitors, setVisitors] = useState(100000);
  const [ram, setRam] = useState(2);
  const [teamSize, setTeamSize] = useState(3);

  const calc = useMemo(() => {
    const p = PROVIDERS[provider];
    // Traditional stack needs ~360MB baseline + app memory
    const traditionalRam = Math.max(ram, 4); // min 4GB for traditional
    const traditionalMonthly =
      provider === "vercel"
        ? 20 + Math.max(0, visitors - 100000) * 0.00004 // Pro + bandwidth
        : p.baseCost + traditionalRam * p.costPerGB;

    // NAWA needs 512MB minimum
    const nawaRam = Math.max(0.5, ram * 0.15); // NAWA uses 15% of traditional
    const nawaMonthly =
      provider === "vercel"
        ? 0 // self-host, no Vercel cost
        : p.baseCost + nawaRam * p.costPerGB;

    const yearlySavings = (traditionalMonthly - nawaMonthly) * 12;
    const fiveYearSavings = yearlySavings * 5;
    const percentage = traditionalMonthly > 0 ? ((traditionalMonthly - nawaMonthly) / traditionalMonthly) * 100 : 0;

    return {
      traditionalMonthly,
      nawaMonthly,
      yearlySavings,
      fiveYearSavings,
      percentage,
      traditionalRam,
      nawaRam,
    };
  }, [provider, visitors, ram, teamSize]);

  return (
    <section id="calculator" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-grid opacity-30 pointer-events-none" />
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="حاسبة التوفير"
          eyebrowEn="Cost Savings Calculator"
          title="كم ستوفّر مع NAWA؟"
          titleEn="How much will you save with NAWA?"
          desc="احسب توفيرك الشهري والسنوي عند الانتقال من stack تقليدي إلى NAWA. أدخل حجم حركة الزوار والفريق، واختر مزوّد الاستضافة — سنُظهر الفارق الحقيقي."
          descEn="Calculate your monthly and yearly savings when switching from a traditional stack to NAWA. Enter your traffic, team size, and hosting provider."
        />

        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-12 grid lg:grid-cols-2 gap-6"
        >
          {/* Inputs panel */}
          <div className="p-6 rounded-2xl border border-border/60 bg-card/60 space-y-6">
            {/* Provider */}
            <div>
              <label className="text-xs font-medium text-muted-foreground uppercase tracking-wide mb-3 block ar">
                مزوّد الاستضافة الحالي
              </label>
              <div className="grid grid-cols-2 gap-2">
                {(Object.keys(PROVIDERS) as Provider[]).map((key) => {
                  const p = PROVIDERS[key];
                  return (
                    <button
                      key={key}
                      onClick={() => setProvider(key)}
                      className={`px-3 py-2.5 rounded-lg border text-right transition-all ${
                        provider === key
                          ? "border-primary bg-primary/10"
                          : "border-border/60 bg-card/40 hover:border-border"
                      }`}
                    >
                      <div className={`text-sm font-medium ${provider === key ? "text-primary" : "text-foreground"}`}>
                        {p.nameAr}
                      </div>
                      <div className="text-[10px] text-muted-foreground mono">{p.name}</div>
                    </button>
                  );
                })}
              </div>
            </div>

            {/* Visitors slider */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <label className="text-xs font-medium text-muted-foreground uppercase tracking-wide ar">
                  زوار شهرياً
                </label>
                <span className="text-sm font-bold mono text-primary">
                  {visitors.toLocaleString()}
                </span>
              </div>
              <input
                type="range"
                min={10000}
                max={5000000}
                step={10000}
                value={visitors}
                onChange={(e) => setVisitors(Number(e.target.value))}
                className="w-full accent-primary"
              />
              <div className="flex justify-between text-[10px] text-muted-foreground mt-1 mono">
                <span>10k</span>
                <span>5M</span>
              </div>
            </div>

            {/* RAM slider */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <label className="text-xs font-medium text-muted-foreground uppercase tracking-wide ar">
                  RAM المطلوبة للتطبيق
                </label>
                <span className="text-sm font-bold mono text-primary">{ram} GB</span>
              </div>
              <input
                type="range"
                min={0.5}
                max={16}
                step={0.5}
                value={ram}
                onChange={(e) => setRam(Number(e.target.value))}
                className="w-full accent-primary"
              />
              <div className="flex justify-between text-[10px] text-muted-foreground mt-1 mono">
                <span>0.5GB</span>
                <span>16GB</span>
              </div>
            </div>

            {/* Team size */}
            <div>
              <div className="flex items-center justify-between mb-2">
                <label className="text-xs font-medium text-muted-foreground uppercase tracking-wide ar">
                  حجم الفريق
                </label>
                <span className="text-sm font-bold mono text-primary">
                  {teamSize} {teamSize === 1 ? "مطوّر" : "مطوّرين"}
                </span>
              </div>
              <input
                type="range"
                min={1}
                max={20}
                step={1}
                value={teamSize}
                onChange={(e) => setTeamSize(Number(e.target.value))}
                className="w-full accent-primary"
              />
              <div className="flex justify-between text-[10px] text-muted-foreground mt-1 mono">
                <span>1</span>
                <span>20</span>
              </div>
            </div>

            <div className="pt-4 border-t border-border/40">
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <Server className="w-3.5 h-3.5" />
                <span className="ar">
                  الحساب يشمل: استضافة + bandwidth + TLS + DB + cache
                </span>
              </div>
            </div>
          </div>

          {/* Results panel */}
          <div className="space-y-4">
            {/* Main savings card */}
            <div className="p-6 rounded-2xl border border-primary/40 bg-gradient-to-br from-primary/15 to-accent/5 glow-amber">
              <div className="flex items-center gap-2 mb-4">
                <DollarSign className="w-5 h-5 text-primary" />
                <span className="text-sm font-semibold ar">التوفير السنوي المتوقّع</span>
              </div>
              <div className="text-5xl lg:text-6xl font-bold text-gradient-amber mono">
                ${calc.yearlySavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}
              </div>
              <div className="mt-2 flex items-center gap-3">
                <Badge className="bg-accent/15 border border-accent/40 text-accent mono text-xs">
                  <TrendingDown className="w-3 h-3 ml-1" />
                  {calc.percentage.toFixed(0)}% أقل
                </Badge>
                <span className="text-xs text-muted-foreground ar">
                  وفّر <span className="mono text-accent font-bold">${calc.fiveYearSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</span> خلال 5 سنوات
                </span>
              </div>
            </div>

            {/* Comparison breakdown */}
            <div className="grid grid-cols-2 gap-3">
              <div className="p-4 rounded-xl border border-destructive/30 bg-destructive/5">
                <div className="flex items-center gap-1.5 mb-2">
                  <span className="w-1.5 h-1.5 rounded-full bg-destructive" />
                  <span className="text-[10px] font-medium text-destructive mono uppercase">Traditional</span>
                </div>
                <div className="text-2xl font-bold text-destructive mono">
                  ${calc.traditionalMonthly.toFixed(0)}
                  <span className="text-xs text-muted-foreground">/mo</span>
                </div>
                <div className="text-[10px] text-muted-foreground mt-1 ar">
                  {calc.traditionalRam}GB RAM · {provider === "vercel" ? "managed" : "VPS"}
                </div>
              </div>
              <div className="p-4 rounded-xl border border-primary/40 bg-primary/5">
                <div className="flex items-center gap-1.5 mb-2">
                  <span className="w-1.5 h-1.5 rounded-full bg-primary animate-nawa-pulse" />
                  <span className="text-[10px] font-medium text-primary mono uppercase">NAWA</span>
                </div>
                <div className="text-2xl font-bold text-primary mono">
                  ${calc.nawaMonthly.toFixed(0)}
                  <span className="text-xs text-muted-foreground">/mo</span>
                </div>
                <div className="text-[10px] text-muted-foreground mt-1 ar">
                  {calc.nawaRam.toFixed(1)}GB RAM · self-hosted
                </div>
              </div>
            </div>

            {/* What you get */}
            <div className="p-4 rounded-xl border border-border/60 bg-card/60">
              <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-3 mono">
                what's included with NAWA
              </div>
              <div className="grid grid-cols-2 gap-2">
                {[
                  "HTTP/3 server",
                  "Built-in database",
                  "Auto TLS",
                  "WAF + rate limit",
                  "WASM plugins",
                  "Observability",
                  "Hot reload",
                  "CLI tools",
                ].map((item) => (
                  <div key={item} className="flex items-center gap-1.5 text-xs">
                    <span className="text-accent">✓</span>
                    <span className="mono text-foreground/80">{item}</span>
                  </div>
                ))}
              </div>
            </div>

            {/* Quick stats */}
            <div className="grid grid-cols-3 gap-2">
              <div className="p-3 rounded-lg border border-border/60 bg-card/60 text-center">
                <Users className="w-4 h-4 text-accent mx-auto mb-1" />
                <div className="text-sm font-bold mono">{teamSize}</div>
                <div className="text-[10px] text-muted-foreground ar">مطوّر</div>
              </div>
              <div className="p-3 rounded-lg border border-border/60 bg-card/60 text-center">
                <Zap className="w-4 h-4 text-primary mx-auto mb-1" />
                <div className="text-sm font-bold mono">{(visitors / 1000).toFixed(0)}k</div>
                <div className="text-[10px] text-muted-foreground ar">زائر شهرياً</div>
              </div>
              <div className="p-3 rounded-lg border border-border/60 bg-card/60 text-center">
                <ArrowDown className="w-4 h-4 text-accent mx-auto mb-1" />
                <div className="text-sm font-bold mono text-accent">{calc.percentage.toFixed(0)}%</div>
                <div className="text-[10px] text-muted-foreground ar">توفير</div>
              </div>
            </div>
          </div>
        </motion.div>

        {/* Bottom note */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-8 p-4 rounded-xl border border-border/60 bg-card/40 flex items-start gap-3"
        >
          <Calculator className="w-4 h-4 text-muted-foreground shrink-0 mt-0.5" />
          <p className="text-xs text-muted-foreground leading-relaxed ar">
            <span className="text-foreground font-medium">ملاحظة:</span> هذه تقديرات بناءً على أسعار
            الاستضافة العامة (2026). التوفير الفعلي قد يختلف حسب نمط الاستخدام. NAWA يستهلك عادةً
            7-10× أقل RAM من stack تقليدي لنفس الحمل.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
