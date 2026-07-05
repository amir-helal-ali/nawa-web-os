"use client";

import { motion } from "framer-motion";
import {
  Rocket,
  ArrowLeft,
  Github,
  Star,
  Terminal,
  BookOpen,
  Heart,
  Sparkles,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

export function FinalCTA() {
  return (
    <section id="start" className="relative py-24 lg:py-32 overflow-hidden">
      {/* Background effects */}
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[600px] h-[600px] rounded-full bg-primary/15 blur-[150px]" />
        <div className="absolute bottom-0 right-1/4 w-[400px] h-[400px] rounded-full bg-accent/10 blur-[120px]" />
      </div>
      <div className="absolute inset-0 bg-grid opacity-20 pointer-events-none" />

      <div className="relative max-w-5xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Badge */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="flex justify-center mb-6"
        >
          <Badge className="bg-primary/15 border border-primary/40 text-primary mono text-xs px-4 py-1.5">
            <Sparkles className="w-3.5 h-3.5 ml-1.5" />
            <span className="ar">جاهز للانطلاق؟</span>
            <span className="opacity-50 mx-1.5">·</span>
            <span className="text-[10px]">v0.1.0-alpha</span>
          </Badge>
        </motion.div>

        {/* Main headline */}
        <motion.h2
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="text-4xl sm:text-5xl lg:text-6xl font-bold tracking-tight text-center leading-tight"
        >
          <span className="text-gradient-amber">ابنِ مستقبل الويب</span>
          <br />
          <span className="ar">بـ Rust وصفر تبعيات</span>
        </motion.h2>

        <motion.p
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: 0.1 }}
          className="mt-6 text-lg text-muted-foreground text-center max-w-2xl mx-auto leading-relaxed ar"
        >
          انضم إلى 4,000+ مطوّر يبنون تطبيقات الويب بـ NAWA. ابدأ بأمر واحد، انشر بضغطة،
          ووفّر 90% من تكاليف السيرفر.
        </motion.p>

        <motion.p
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: 0.15 }}
          className="mt-3 text-base text-muted-foreground/70 text-center max-w-2xl mx-auto"
        >
          Join 4,000+ developers building the future of web on Rust. Start with one command,
          deploy in seconds, save 90% on server costs.
        </motion.p>

        {/* CTAs */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: 0.2 }}
          className="mt-10 flex flex-col sm:flex-row items-center justify-center gap-3"
        >
          <Button
            size="lg"
            className="bg-primary hover:bg-primary/90 text-primary-foreground group px-6 h-12 text-base"
            onClick={() => document.getElementById("cli")?.scrollIntoView({ behavior: "smooth" })}
          >
            <Terminal className="w-4 h-4 ml-1.5" />
            <span className="ar">جرّب nawa CLI الآن</span>
            <ArrowLeft className="w-4 h-4 mr-1.5 group-hover:-translate-x-1 transition-transform" />
          </Button>
          <Button
            size="lg"
            variant="outline"
            className="border-primary/40 hover:border-primary hover:bg-primary/10 px-6 h-12 text-base"
            onClick={() => document.getElementById("builder")?.scrollIntoView({ behavior: "smooth" })}
          >
            <Rocket className="w-4 h-4 ml-1.5" />
            <span className="ar">ابدأ مشروعاً</span>
          </Button>
          <Button
            size="lg"
            variant="ghost"
            className="hover:bg-card px-6 h-12 text-base"
          >
            <BookOpen className="w-4 h-4 ml-1.5" />
            <span className="ar">اقرأ الوثائق</span>
          </Button>
        </motion.div>

        {/* Terminal snippet */}
        <motion.div
          initial={{ opacity: 0, y: 40 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.7, delay: 0.3 }}
          className="mt-12 max-w-2xl mx-auto"
        >
          <div className="rounded-2xl border border-primary/30 bg-[#0d0c0a] overflow-hidden glow-amber">
            <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
              <div className="flex items-center gap-2">
                <div className="flex gap-1.5">
                  <span className="w-3 h-3 rounded-full bg-destructive/60" />
                  <span className="w-3 h-3 rounded-full bg-yellow-500/60" />
                  <span className="w-3 h-3 rounded-full bg-green-500/60" />
                </div>
                <span className="text-xs text-muted-foreground mono ml-2">bash</span>
              </div>
              <Badge variant="outline" className="mono text-[10px] border-accent/40 text-accent">
                <span className="w-1.5 h-1.5 rounded-full bg-accent mr-1 animate-nawa-pulse" />
                ready in 5s
              </Badge>
            </div>
            <div className="p-4 font-mono text-sm leading-relaxed">
              <div className="flex items-center gap-2">
                <span className="text-accent shrink-0">$</span>
                <span className="text-foreground/90">nawa create my-app --template saas</span>
              </div>
              <div className="text-muted-foreground/70 mt-1 ar">
                <span className="text-accent">✓</span> Project created · 12 files · 47KB
              </div>
              <div className="flex items-center gap-2 mt-2">
                <span className="text-accent shrink-0">$</span>
                <span className="text-foreground/90">cd my-app && nawa dev</span>
              </div>
              <div className="text-muted-foreground/70 mt-1 ar">
                <span className="text-accent">✓</span> Running on <span className="text-primary">http://localhost:8080</span>
              </div>
              <div className="flex items-center gap-2 mt-2">
                <span className="text-accent shrink-0">$</span>
                <span className="text-foreground/90">nawa deploy --target ssh://user@your-vps</span>
              </div>
              <div className="text-muted-foreground/70 mt-1 ar">
                <span className="text-accent">✓</span> Live at <span className="text-primary">https://your-vps.com</span> · TLS auto
              </div>
              <div className="flex items-center gap-2 mt-3 pt-3 border-t border-border/40">
                <Heart className="w-3 h-3 text-destructive fill-current" />
                <span className="text-[10px] text-muted-foreground ar">
                  3 أوامر · 30 ثانية · 0 ملف إعداد
                </span>
              </div>
            </div>
          </div>
        </motion.div>

        {/* Trust badges */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: 0.4 }}
          className="mt-12 grid grid-cols-2 lg:grid-cols-4 gap-4 max-w-3xl mx-auto"
        >
          {[
            { icon: Star, value: "12.4k", label: "GitHub Stars", color: "yellow-500" },
            { icon: Github, value: "MIT", label: "+ Apache 2.0", color: "primary" },
            { icon: Rocket, value: "186", label: "Production Deploys", color: "accent" },
            { icon: Heart, value: "4k+", label: "Community", color: "destructive" },
          ].map((b, i) => (
            <motion.div
              key={b.label}
              initial={{ opacity: 0, scale: 0.9 }}
              whileInView={{ opacity: 1, scale: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: 0.45 + i * 0.08 }}
              className="flex flex-col items-center gap-1.5 p-4 rounded-xl border border-border/60 bg-card/60"
            >
              <b.icon
                className={`w-5 h-5 ${
                  b.color === "yellow-500"
                    ? "text-yellow-500"
                    : b.color === "primary"
                    ? "text-primary"
                    : b.color === "accent"
                    ? "text-accent"
                    : "text-destructive"
                }`}
                strokeWidth={1.5}
              />
              <div className="text-xl font-bold mono text-foreground">{b.value}</div>
              <div className="text-[10px] text-muted-foreground">{b.label}</div>
            </motion.div>
          ))}
        </motion.div>

        {/* Final manifesto quote */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6, delay: 0.5 }}
          className="mt-16 text-center"
        >
          <p className="text-base lg:text-lg text-muted-foreground italic max-w-2xl mx-auto ar leading-relaxed">
            «نواة واحدة. صفر نسخ. لا تبعيات خارجية. صُمم لأضعف السيرفرات.»
          </p>
          <p className="mt-2 text-xs text-muted-foreground/60 mono">
            — One kernel. Zero copies. No external deps. Built for the cheapest VPS on earth.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
