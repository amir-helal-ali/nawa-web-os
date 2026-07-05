"use client";

import { motion } from "framer-motion";
import {
  Terminal,
  Zap,
  Code2,
  RefreshCw,
  GitBranch,
  Package,
  Bug,
  Sparkles,
  FileCode2,
  CheckCircle2,
  Clock,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

const DX_FEATURES = [
  {
    icon: RefreshCw,
    title: "Hot Reload · Sub-100ms",
    titleAr: "إعادة تحميل ساخنة · أقل من 100ms",
    desc: "أي تعديل في Rust يُعاد تحميله فوراً دون فقدان الحالة. Incremental compilation + WASM patching.",
    metric: "67ms",
    metricLabel: "avg reload",
    color: "primary",
  },
  {
    icon: Bug,
    title: "Type Errors in Compile-Time",
    titleAr: "أخطاء الأنواع في وقت التجميع",
    desc: "كل خطأ في الـ routing أو الـ DB schema يُكتشف قبل التشغيل. لا runtime surprises.",
    metric: "100%",
    metricLabel: "type coverage",
    color: "accent",
  },
  {
    icon: Terminal,
    title: "Single Binary Dev Server",
    titleAr: "خادم تطوير بثنائي واحد",
    desc: "خادم التطوير هو نفس خادم الإنتاج — لا تفاوض بين البيئات. ما يظهر في dev يظهر في prod.",
    metric: "1:1",
    metricLabel: "dev = prod",
    color: "primary",
  },
  {
    icon: GitBranch,
    title: "Zero-Config Git Workflow",
    titleAr: "سير عمل Git بدون إعداد",
    desc: "pre-commit hooks، CI templates، conventional commits — كلها مدمجة في nawa init.",
    metric: "0",
    metricLabel: "config files",
    color: "accent",
  },
  {
    icon: Package,
    title: "Built-in Dependency Manager",
    titleAr: "مدير تبعيات مدمج",
    desc: "nawa add <crate> يضيف dependency ويُفعّل feature flags الصحيحة تلقائياً.",
    metric: "<2s",
    metricLabel: "per add",
    color: "primary",
  },
  {
    icon: Sparkles,
    title: "AI-Powered Codegen",
    titleAr: "توليد كود بالذكاء الاصطناعي",
    desc: "nawa generate handler /users — يُولِّد handler + tests + DB schema بـ AI.",
    metric: "5s",
    metricLabel: "per scaffold",
    color: "accent",
  },
];

const WORKFLOW_STEPS = [
  {
    cmd: "nawa init",
    desc: "تهيئة مشروع جديد",
    detail: "يختار template، يضبط Cargo.toml، يُنشئ بنية المجلدات، يُفعّل git hooks.",
    icon: Package,
  },
  {
    cmd: "nawa dev",
    desc: "خادم تطوير + hot reload",
    detail: "يُشغّل خادم على :8080 مع hot reload sub-100ms. كل تعديل يظهر فوراً.",
    icon: RefreshCw,
  },
  {
    cmd: "nawa test",
    desc: "اختبارات سريعة",
    detail: "unit + integration + e2e tests. parallel execution، snapshot testing مدمج.",
    icon: CheckCircle2,
  },
  {
    cmd: "nawa bench",
    desc: "benchmarks تلقائية",
    detail: "يقارن أداء handler ضد baseline، يكشف regressions تلقائياً قبل الـ commit.",
    icon: Zap,
  },
  {
    cmd: "nawa deploy",
    desc: "نشر بضغطة واحدة",
    detail: "يبني Docker image، يرفعه للسيرفر، يُفعّل TLS، يُشغّل health check.",
    icon: GitBranch,
  },
];

export function DeveloperExperience() {
  return (
    <section id="dx" className="relative py-24 lg:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="تجربة المطور"
          eyebrowEn="Developer Experience"
          title="طوّر بسرعة · انشر بثقة"
          titleEn="Develop fast · ship with confidence"
          desc="NAWA مصمم لتجربة مطور استثنائية. كل شيء من التهيئة إلى النشر بأمر واحد، مع type safety كامل وhot reload فوري. لا تقاتل مع الأدوات — ركّز على الكود."
          descEn="NAWA is built for an exceptional developer experience. Everything from init to deploy in one command, with full type safety and instant hot reload."
        />

        {/* Features grid */}
        <div className="mt-12 grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {DX_FEATURES.map((f, i) => (
            <motion.div
              key={f.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: i * 0.08 }}
              className="group p-5 rounded-2xl border border-border/60 bg-card/60 hover:border-primary/40 hover:bg-card transition-all"
            >
              <div className="flex items-start justify-between mb-3">
                <div className={`p-2 rounded-lg ${f.color === "primary" ? "bg-primary/15 text-primary" : "bg-accent/15 text-accent"}`}>
                  <f.icon className="w-5 h-5" strokeWidth={1.5} />
                </div>
                <div className="text-right">
                  <div className={`text-xl font-bold mono ${f.color === "primary" ? "text-primary" : "text-accent"}`}>
                    {f.metric}
                  </div>
                  <div className="text-[9px] text-muted-foreground mono">{f.metricLabel}</div>
                </div>
              </div>
              <h4 className="text-sm font-semibold ar">{f.titleAr}</h4>
              <p className="text-[10px] text-muted-foreground mono mt-0.5">{f.title}</p>
              <p className="text-xs text-muted-foreground mt-2 leading-relaxed ar">{f.desc}</p>
            </motion.div>
          ))}
        </div>

        {/* Workflow timeline */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-16"
        >
          <div className="flex items-center gap-2 mb-6">
            <GitBranch className="w-4 h-4 text-primary" />
            <h3 className="text-base font-semibold ar">سير عمل المطور (Developer Workflow)</h3>
            <span className="text-xs text-muted-foreground mono ml-1">{`// from init to deploy in 5 commands`}</span>
          </div>

          <div className="relative">
            {/* Vertical line for desktop */}
            <div className="hidden lg:block absolute right-1/2 top-0 bottom-0 w-px bg-gradient-to-b from-primary/40 via-border/40 to-transparent" />

            <div className="space-y-4 lg:space-y-0">
              {WORKFLOW_STEPS.map((step, i) => (
                <div key={step.cmd} className="lg:grid lg:grid-cols-2 lg:gap-8 relative">
                  <div className="hidden lg:flex absolute right-1/2 top-8 translate-x-1/2 z-10">
                    <div className="w-8 h-8 rounded-full bg-background border-2 border-primary grid place-items-center mono text-xs font-bold text-primary">
                      {i + 1}
                    </div>
                  </div>
                  <motion.div
                    initial={{ opacity: 0, x: i % 2 === 0 ? -20 : 20 }}
                    whileInView={{ opacity: 1, x: 0 }}
                    viewport={{ once: true }}
                    transition={{ duration: 0.4, delay: i * 0.1 }}
                    className={`mb-4 lg:mb-0 ${i % 2 === 0 ? "lg:pr-8" : "lg:col-start-2 lg:pl-8"}`}
                  >
                    <div className="p-4 rounded-xl border border-border/60 bg-card/60 hover:border-primary/40 transition-colors">
                      <div className="flex items-start gap-3">
                        <div className="p-2 rounded-lg bg-primary/15 text-primary shrink-0">
                          <step.icon className="w-4 h-4" strokeWidth={1.5} />
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 flex-wrap">
                            <code className="text-sm font-semibold mono text-primary">{step.cmd}</code>
                            <span className="text-xs text-muted-foreground ar">{step.desc}</span>
                          </div>
                          <p className="text-xs text-muted-foreground mt-1.5 leading-relaxed ar">{step.detail}</p>
                        </div>
                      </div>
                    </div>
                  </motion.div>
                </div>
              ))}
            </div>
          </div>
        </motion.div>

        {/* Code quality strip */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-12 grid sm:grid-cols-2 lg:grid-cols-4 gap-3"
        >
          {[
            { icon: Code2, label: "Type Coverage", value: "100%", sub: "no any types" },
            { icon: CheckCircle2, label: "Test Coverage", value: "92%", sub: "unit + integration" },
            { icon: Clock, label: "Build Time", value: "1.2s", sub: "incremental" },
            { icon: Bug, label: "Production Bugs", value: "0", sub: "type-safe by design" },
          ].map((s) => (
            <div key={s.label} className="p-4 rounded-xl border border-border/60 bg-card/60 text-center">
              <s.icon className="w-5 h-5 text-accent mx-auto mb-2" strokeWidth={1.5} />
              <div className="text-2xl font-bold text-primary mono">{s.value}</div>
              <div className="text-xs text-foreground mt-0.5">{s.label}</div>
              <div className="text-[10px] text-muted-foreground">{s.sub}</div>
            </div>
          ))}
        </motion.div>

        {/* Code sample with annotations */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-12"
        >
          <div className="flex items-center gap-2 mb-4">
            <FileCode2 className="w-4 h-4 text-primary" />
            <h3 className="text-base font-semibold ar">مثال: handler كامل مع type safety</h3>
          </div>
          <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
            <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
              <span className="text-xs text-muted-foreground mono">src/routes/users.rs</span>
              <Badge variant="outline" className="mono text-[10px] border-accent/40 text-accent">
                ✓ compiles with 0 warnings
              </Badge>
            </div>
            <pre className="p-4 overflow-x-auto scrollbar-narrow text-xs leading-relaxed mono">
              <code className="text-foreground/90">
{`#[derive(Serialize, Deserialize)]
pub struct User {                       // ← schema مُشتق تلقائياً
    id: u64,
    name: String,
    email: Email,                       // ← نوع مخصص بـ validation
}

#[get("/users/:id")]                    // ← route ثابت الأنواع
pub fn get_user(
    Path(id): Path<u64>,                // ← :id يُستخرج كـ u64
    State(db): State<Db>,               // ← حقن dependency
) -> Result<Json<User>, NotFound> {     // ← أخطاء type-safe
    let user: User = db.get(&format!("user:{}", id))
        .ok_or(NotFound)?;

    Ok(Json(user))                      // ← JSON serialization صفر-overhead
}`}
              </code>
            </pre>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
