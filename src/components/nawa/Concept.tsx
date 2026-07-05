"use client";

import { motion } from "framer-motion";
import { Lightbulb, Layers, Boxes, GitBranch, Workflow } from "lucide-react";

const PROBLEMS = [
  {
    title: "أنظمة الويب منفصلة ومشتتة",
    en: "Fragmented Web Stacks",
    desc: "قاعدة بيانات من جهة، خادم من جهة، نظام تشغيل من جهة، أدوات نشر من جهة — تنسيقها يستهلك وقتاً وموارد.",
  },
  {
    title: "اعتماد على خدمات خارجية ضخمة",
    en: "Heavy External Dependencies",
    desc: "PostgreSQL + Redis + Nginx + Node.js = سيرفر 4GB كحد أدنى لاستضافة تطبيق صغير. مبالغ فيه.",
  },
  {
    title: "ضعف الأداء على البنى التحتية الصغيرة",
    en: "Poor Low-Spec Performance",
    desc: "أغلب الأطر مُحسّنة للأجهزة القوية وتتدهور بسرعة على VPS الاقتصادي الذي يستخدمه أغلب المطورين.",
  },
];

const SOLUTIONS = [
  {
    icon: Layers,
    title: "دمج كامل في نواة واحدة",
    en: "Unified Kernel",
    desc: "محرك خلفية + محرك واجهة + قاعدة بيانات + خادم HTTP + نظام ملفات افتراضي — كلها في ثنائي Rust واحد يعمل في حاوية.",
  },
  {
    icon: Boxes,
    title: "قاعدة بيانات مدمجة بدون تبعيات",
    en: "Self-Contained DB",
    desc: "قاعدة بيانات KV/Document مكتوبة من الصفر بـ mmap وlock-free structures — لا PostgreSQL ولا Redis ولا أي مكتبة خارجية.",
  },
  {
    icon: Workflow,
    title: "zero-copy من الـ kernel إلى الـ socket",
    en: "End-to-End Zero-Copy",
    desc: "استخدام io_uring وmmap يعني أن البيانات تمر من القرص إلى الشبكة دون نسخ في الذاكرة — أداء يقارب الـ hardware.",
  },
  {
    icon: GitBranch,
    title: "بنية جاهزة لأي مشروع ويب",
    en: "Project-Ready",
    desc: "تطبيقات SSR، APIs، خدمات real-time، static sites — كلها تُبنى فوق نفس النواة بنفس الـ tooling.",
  },
];

export function Concept() {
  return (
    <section id="concept" className="relative py-24 lg:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Section header */}
        <SectionHeader
          eyebrow="الفكرة الثورية"
          eyebrowEn="The Revolutionary Idea"
          title="ماذا لو بُني نظام الويب كنواة واحدة؟"
          titleEn="What if the entire web stack were a single kernel?"
        />

        {/* Problems → Solutions */}
        <div className="grid lg:grid-cols-2 gap-10 lg:gap-16 mt-16">
          {/* Problems */}
          <div>
            <div className="flex items-center gap-2 mb-6">
              <span className="w-1 h-6 bg-destructive rounded-full" />
              <h3 className="text-lg font-semibold ar">المشكلة</h3>
              <span className="text-xs text-muted-foreground mono">{`// status quo`}</span>
            </div>
            <div className="space-y-3">
              {PROBLEMS.map((p, i) => (
                <motion.div
                  key={p.en}
                  initial={{ opacity: 0, x: -20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true, margin: "-50px" }}
                  transition={{ duration: 0.4, delay: i * 0.08 }}
                  className="p-4 rounded-xl border border-destructive/20 bg-destructive/5"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="text-sm font-medium text-foreground ar">{p.title}</div>
                    <span className="text-[10px] text-muted-foreground mono shrink-0">{p.en}</span>
                  </div>
                  <p className="text-sm text-muted-foreground mt-2 leading-relaxed ar">{p.desc}</p>
                </motion.div>
              ))}
            </div>
          </div>

          {/* Solutions */}
          <div>
            <div className="flex items-center gap-2 mb-6">
              <span className="w-1 h-6 bg-primary rounded-full" />
              <h3 className="text-lg font-semibold ar">حل NAWA</h3>
              <span className="text-xs text-muted-foreground mono">{`// NAWA way`}</span>
            </div>
            <div className="space-y-3">
              {SOLUTIONS.map((s, i) => (
                <motion.div
                  key={s.en}
                  initial={{ opacity: 0, x: 20 }}
                  whileInView={{ opacity: 1, x: 0 }}
                  viewport={{ once: true, margin: "-50px" }}
                  transition={{ duration: 0.4, delay: i * 0.08 }}
                  className="group p-4 rounded-xl border border-primary/20 bg-primary/5 hover:bg-primary/10 hover:border-primary/40 transition-all"
                >
                  <div className="flex items-start gap-3">
                    <div className="p-2 rounded-lg bg-primary/15 text-primary group-hover:scale-110 transition-transform">
                      <s.icon className="w-4 h-4" strokeWidth={1.5} />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between gap-3">
                        <div className="text-sm font-medium text-foreground ar">{s.title}</div>
                        <span className="text-[10px] text-muted-foreground mono shrink-0">{s.en}</span>
                      </div>
                      <p className="text-sm text-muted-foreground mt-1.5 leading-relaxed ar">{s.desc}</p>
                    </div>
                  </div>
                </motion.div>
              ))}
            </div>
          </div>
        </div>

        {/* Manifesto strip */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-16 relative overflow-hidden rounded-2xl border border-primary/30 bg-gradient-to-br from-primary/10 via-card to-accent/5 p-8 lg:p-10"
        >
          <div className="absolute inset-0 bg-dots opacity-30 pointer-events-none" />
          <div className="relative flex flex-col lg:flex-row items-start lg:items-center gap-6">
            <div className="p-3 rounded-xl bg-primary/15 text-primary shrink-0">
              <Lightbulb className="w-8 h-8" strokeWidth={1.25} />
            </div>
            <div className="flex-1">
              <p className="text-xl lg:text-2xl font-medium leading-relaxed ar">
                «<span className="text-gradient-amber">نواة واحدة</span>، مكتوبة بـ Rust، تُدير كل شيء —
                من <span className="mono text-primary">io_uring</span> في النواة إلى <span className="mono text-primary">islands</span> في المتصفح.
                بدون تبعيات، بدون طبقات وسيطة، بدون تنازلات.»
              </p>
              <p className="text-sm text-muted-foreground mt-3">
                <span className="mono">One kernel. Zero copies. No external DB. Built for the cheapest VPS on earth.</span>
              </p>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}

export function SectionHeader({
  eyebrow,
  eyebrowEn,
  title,
  titleEn,
  desc,
  descEn,
}: {
  eyebrow: string;
  eyebrowEn: string;
  title: string;
  titleEn?: string;
  desc?: string;
  descEn?: string;
}) {
  return (
    <div className="max-w-3xl">
      <motion.div
        initial={{ opacity: 0, y: 12 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true }}
        transition={{ duration: 0.4 }}
        className="inline-flex items-center gap-2 px-2.5 py-1 rounded-full border border-primary/30 bg-primary/10 text-primary text-xs font-medium mono mb-4"
      >
        <span className="w-1 h-1 rounded-full bg-primary" />
        <span className="ar">{eyebrow}</span>
        <span className="opacity-40">·</span>
        <span className="text-[10px] opacity-80">{eyebrowEn}</span>
      </motion.div>
      <motion.h2
        initial={{ opacity: 0, y: 20 }}
        whileInView={{ opacity: 1, y: 0 }}
        viewport={{ once: true }}
        transition={{ duration: 0.5, delay: 0.05 }}
        className="text-3xl sm:text-4xl lg:text-5xl font-bold tracking-tight leading-tight ar"
      >
        {title}
      </motion.h2>
      {titleEn && (
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.1 }}
          className="mt-2 text-base text-muted-foreground"
        >
          {titleEn}
        </motion.p>
      )}
      {desc && (
        <motion.p
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5, delay: 0.15 }}
          className="mt-4 text-base lg:text-lg text-muted-foreground leading-relaxed ar"
        >
          {desc}
          {descEn && (
            <span className="block mt-2 text-sm text-muted-foreground/70">{descEn}</span>
          )}
        </motion.p>
      )}
    </div>
  );
}
