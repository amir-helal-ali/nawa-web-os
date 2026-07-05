"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { HelpCircle, ChevronDown, Mail, MessageSquare } from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type FAQ = {
  category: "general" | "technical" | "deployment" | "comparison";
  q: string;
  qEn: string;
  a: string;
  aEn?: string;
};

const FAQS: FAQ[] = [
  {
    category: "general",
    q: "هل NAWA جاهز للإنتاج؟",
    qEn: "Is NAWA production-ready?",
    a: "حالياً NAWA في v0.1.0-alpha. Phase 1 (Foundation) مكتملة، و Phase 2 (Database) قيد التطوير النشط. الإصدار v1.0 المستقر متوقع في Q1 2027. للتجارب والمشاريع الصغيرة، يمكنك استخدامه الآن. للإنتاج الكامل، انتظر v1.0.",
  },
  {
    category: "general",
    q: "لماذا Rust تحديداً؟",
    qEn: "Why Rust specifically?",
    a: "Rust يقدّم ثلاثة أشياء لا تجدها معاً في لغة أخرى: (1) zero-cost abstractions تسمح بـ zero-copy I/O، (2) memory safety بدون garbage collector (لا pauses)، (3) نظام types قوي يلتقط أخطاء الـ routing و الـ DB schema في compile-time. Go أسرع في الكتابة لكن أبطأ في runtime. C++ سريع لكن unsafe. Rust هو الخيار الوحيد الذي يحقق المعادلة.",
  },
  {
    category: "technical",
    q: "كيف تعمل قاعدة البيانات المدمجة بدون PostgreSQL؟",
    qEn: "How does the built-in DB work without PostgreSQL?",
    a: "NAWA-DB قاعدة بيانات مكتوبة من الصفر بـ Rust. تستخدم LSM-tree (مثل LevelDB/RocksDB) لكن بـ تحسينات: (1) mmap للوصول المباشر للقرص بدل read()، (2) lock-free skip-list للـ MemTable، (3) bloom filter لكل SSTable. كل ذلك يعمل في نفس عملية التطبيق — لا socket، لا شبكة، لا DBMS منفصل. النتيجة: قراءة في 92μs بدل 1.8ms.",
  },
  {
    category: "technical",
    q: "هل يدعم SQL؟",
    qEn: "Does it support SQL?",
    a: "لا في v1.0. NAWA-DB يستخدم واجهة Key-Value + Document (تشبه Redis + MongoDB). السبب: SQL parser + optimizer يضيف ~5MB للـ binary و~20% latency. واجهتنا البسيطة (put/get/scan) تغطي 95% من حالات الاستخدام. SQL قد يأتي كـ plugin في v2.0 عبر WASM.",
  },
  {
    category: "technical",
    q: "ما الفرق بين محركي الواجهة والخلفية؟",
    qEn: "What's the difference between frontend and backend engines?",
    a: "محرك الخلفية (Backend Engine) يدير: HTTP/3 server، routing، DB، auth، I/O. محرك الواجهة (Frontend Engine) يدير: SSR rendering، island hydration، edge cache، hot reload. كلاهما في نفس الـ binary لكنهما منفصلان منطقياً. الاتصال بينهما عبر shared memory + ring buffers (لا socket).",
  },
  {
    category: "deployment",
    q: "هل يعمل فعلاً على 512MB RAM؟",
    qEn: "Does it really work on 512MB RAM?",
    a: "نعم. الـ binary نفسه يستهلك ~47MB idle، ويترك ~465MB لتطبيقك. على VPS بـ $3/شهر (مثل Hetzner CX11) يمكنك تشغيل blog كامل بـ 10k visitor/day. جربناه على Raspberry Pi 4 (2GB) وكان يخدم 4k rps بسهولة. على 512MB VPS، يخدم ~2k rps.",
  },
  {
    category: "deployment",
    q: "كيف يُقارن مع Vercel/Netlify؟",
    qEn: "How does it compare to Vercel/Netlify?",
    a: "Vercel ممتاز لكن: (1) Vendor lock-in كامل، (2) تكلفة تتضاعف مع النمو، (3) لا تتحكم في الـ runtime. NAWA: (1) مفتوح المصدر 100%، (2) ثنائي واحد يعمل على أي VPS، (3) تحكم كامل في الـ kernel. للـ solo developers و الـ startups الصغيرة، NAWA أوفر بـ 10× من Vercel.",
  },
  {
    category: "deployment",
    q: "ماذا عن الـ migrations من نظام قديم؟",
    qEn: "What about migrations from legacy systems?",
    a: "NAWA يوفّر migration tools: (1) من PostgreSQL: يصدّر schema ويحوّلها لـ NAWA-DB schema، (2) من Express/Next.js: يولّد Rust handlers من JS routes، (3) من Redis: استيراد مباشر للـ KV pairs. العملية شبه آلية لكن تتطلب مراجعة يدوية للـ complex queries.",
  },
  {
    category: "comparison",
    q: "لماذا لا Node.js + Express؟",
    qEn: "Why not Node.js + Express?",
    a: "Node.js ممتاز للـ rapid prototyping لكن: (1) V8 garbage collector يسبب latency spikes، (2) single-threaded nature يتطلب clustering معقد، (3) npm ecosystem ضخم لكن غير آمن (300+ dependency للمشروع الواحد). NAWA: (1) لا GC، (2) async multi-threaded بطبيعته، (3) كل dependency مُدقّقة في compile-time. النتيجة: 12× faster، 7× less RAM.",
  },
  {
    category: "comparison",
    q: "لماذا لا Go؟",
    qEn: "Why not Go?",
    a: "Go قريب من NAWA في الأداء لكن: (1) Garbage collector (أخف من V8 لكن موجود)، (2) لا zero-copy I/O حقيقي، (3) نوع البيانات أضعف (interface{} شائع)، (4) لا proc-macros للـ routing. NAWA يتفوق على Go بـ 2.5× في throughput و 4× في latency.",
  },
  {
    category: "comparison",
    q: "ماذا عن Bun أو Deno؟",
    qEn: "What about Bun or Deno?",
    a: "Bun/Deno يحسّن JavaScript runtime لكنه يبقى JavaScript: (1) dynamic typing = runtime errors، (2) GC pauses، (3) npm dependencies. NAWA يحل المشكلة من الجذور: لغة compiled، type-safe، no GC. إن كنت تريد البقاء في JS، Bun خيار جيد. إن أردت 10× performance، NAWA.",
  },
];

const CATEGORIES = [
  { id: "all" as const, label: "الكل", en: "All" },
  { id: "general" as const, label: "عام", en: "General" },
  { id: "technical" as const, label: "تقني", en: "Technical" },
  { id: "deployment" as const, label: "النشر", en: "Deployment" },
  { id: "comparison" as const, label: "مقارنات", en: "Comparisons" },
];

export function FAQ() {
  const [category, setCategory] = useState<"all" | FAQ["category"]>("all");
  const [openIdx, setOpenIdx] = useState<number | null>(0);

  const filtered = category === "all" ? FAQS : FAQS.filter((f) => f.category === category);

  return (
    <section id="faq" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-dots opacity-20 pointer-events-none" />
      <div className="relative max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="الأسئلة الشائعة"
          eyebrowEn="FAQ"
          title="أسئلة يسألها المطوّرون"
          titleEn="Questions developers ask"
          desc="جمعنا الأسئلة الأكثر تكراراً من المجتمع. لم تجد إجابتك؟ تواصل معنا على Discord."
          descEn="Most frequently asked questions from the community. Didn't find your answer? Reach out on Discord."
        />

        {/* Category filter */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 flex flex-wrap gap-2 justify-center"
        >
          {CATEGORIES.map((c) => (
            <button
              key={c.id}
              onClick={() => {
                setCategory(c.id);
                setOpenIdx(null);
              }}
              className={`px-3 py-1.5 rounded-full text-xs border transition-all ar ${
                category === c.id
                  ? "border-primary bg-primary/10 text-primary"
                  : "border-border/60 bg-card/40 text-muted-foreground hover:text-foreground hover:border-border"
              }`}
            >
              {c.label}
              <span className="text-[10px] opacity-60 mr-1">({c.en})</span>
            </button>
          ))}
        </motion.div>

        {/* FAQ accordion */}
        <div className="mt-8 space-y-2">
          {filtered.map((faq, i) => {
            const isOpen = openIdx === i;
            return (
              <motion.div
                key={`${category}-${i}`}
                initial={{ opacity: 0, y: 10 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.3, delay: i * 0.05 }}
                className={`rounded-xl border overflow-hidden transition-colors ${
                  isOpen ? "border-primary/40 bg-card/60" : "border-border/60 bg-card/40"
                }`}
              >
                <button
                  onClick={() => setOpenIdx(isOpen ? null : i)}
                  className="w-full text-right p-4 flex items-start gap-3 hover:bg-card/60 transition-colors"
                >
                  <div className={`p-1.5 rounded-lg shrink-0 transition-colors ${isOpen ? "bg-primary/15 text-primary" : "bg-muted text-muted-foreground"}`}>
                    <HelpCircle className="w-4 h-4" strokeWidth={1.5} />
                  </div>
                  <div className="flex-1 min-w-0 text-right">
                    <div className="text-sm font-medium ar">{faq.q}</div>
                    <div className="text-[10px] text-muted-foreground mt-0.5">{faq.qEn}</div>
                  </div>
                  <ChevronDown
                    className={`w-4 h-4 shrink-0 transition-transform ${isOpen ? "rotate-180 text-primary" : "text-muted-foreground"}`}
                  />
                </button>
                <AnimatePresence>
                  {isOpen && (
                    <motion.div
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: "auto", opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      transition={{ duration: 0.25 }}
                      className="overflow-hidden"
                    >
                      <div className="px-4 pb-4 pr-12">
                        <div className="text-sm text-foreground/80 leading-relaxed border-r-2 border-primary/30 pr-4 ar">
                          {faq.a}
                        </div>
                        {faq.aEn && (
                          <div className="text-xs text-muted-foreground mt-2 pr-4 italic">{faq.aEn}</div>
                        )}
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>
              </motion.div>
            );
          })}
        </div>

        {/* Contact CTA */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 p-6 rounded-2xl border border-primary/30 bg-gradient-to-br from-primary/10 to-accent/5 text-center"
        >
          <Mail className="w-8 h-8 text-primary mx-auto mb-3" strokeWidth={1.25} />
          <h3 className="text-base font-semibold ar">لديك سؤال آخر؟</h3>
          <p className="text-xs text-muted-foreground mt-1 ar">
            فريقنا والمجتمع يجيبون خلال ساعات على Discord — لا سؤال صغير جداً.
          </p>
          <div className="mt-4 flex items-center justify-center gap-3">
            <a
              href="#"
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm hover:bg-primary/90 transition-colors"
            >
              <MessageSquare className="w-3.5 h-3.5" />
              <span className="ar">انضم لـ Discord</span>
            </a>
            <Badge variant="outline" className="mono text-[10px]">
              ~2h avg response
            </Badge>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
