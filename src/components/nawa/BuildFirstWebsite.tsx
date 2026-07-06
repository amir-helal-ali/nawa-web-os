"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Rocket,
  Terminal,
  Check,
  Copy,
  ArrowLeft,
  FileCode2,
  Play,
  Download,
  BookOpen,
  Lightbulb,
  ExternalLink,
  PartyPopper,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

type Step = {
  id: number;
  title: string;
  titleAr: string;
  desc: string;
  code: string;
  output?: string;
  tip?: string;
  duration: string;
};

const STEPS: Step[] = [
  {
    id: 1,
    title: "Install NAWA",
    titleAr: "تثبيت NAWA",
    desc: "حمّل الـ binary جاهز أو ابنِ من المصدر. يتطلب Linux 5.1+ لـ io_uring.",
    code: `# Download pre-built binary (Linux x86_64)
curl -L https://github.com/amir-helal-ali/nawa-web-os/releases/\\
download/v0.1.0-alpha/nawa-v0.1.0-alpha-linux-amd64.tar.gz \\
| tar xz

# Move to PATH
sudo mv nawad nawa /usr/local/bin/

# Verify installation
nawa info`,
    output: `nawa CLI v0.1.0-alpha (NAWA Web Operating System)
─────────────────────────────────────────────
Components:
  • nawa-kernel: io_uring + mmap + zero-copy
  • nawa-uring:  real io_uring on Linux 5.1+
  • nawa-db:     MemTable + SSTable + WAL + Bloom
  • nawa-http:   HTTP/1.1 + HTTP/3 + TLS + ACME
  • nawa-wasm:   WASM sandbox (wasmtime)
  • nawad:       server binary
Platform:
  OS: linux
  Arch: x86_64
  io_uring: supported ✓`,
    tip: "بدلاً من ذلك: git clone + cargo build --release (يتطلب Rust 1.83+)",
    duration: "~30 seconds",
  },
  {
    id: 2,
    title: "Create Your First Project",
    titleAr: "أنشئ مشروعك الأول",
    desc: "اختر قالباً من 6 قوالب جاهزة. كل قالب يُنشئ مشروعاً كاملاً بـ routes + DB + Docker.",
    code: `# List available templates
nawa templates

# Create a blog project
nawa create my-blog --template blog

# Enter the project directory
cd my-blog

# See the structure
ls -la`,
    output: `Creating NAWA project 'my-blog' from template 'blog'...
  ✓ Cargo.toml
  ✓ src/main.rs
  ✓ .gitignore
  ✓ README.md
  ✓ Dockerfile

✓ Project created in my-blog/

Next steps:
  cd my-blog
  nawa dev`,
    tip: "القوالب المتاحة: blog, saas, shop, realtime, booking, portfolio",
    duration: "~2 seconds",
  },
  {
    id: 3,
    title: "Start the Dev Server",
    titleAr: "ابدأ خادم التطوير",
    desc: "nawa dev يُشغّل الخادم مع hot reload — أي تعديل في src/ يُعيد التشغيل تلقائياً.",
    code: `# Start with hot reload
nawa dev --addr 0.0.0.0:8080 --data-dir ./nawa-data

# In another terminal, test the API:
curl http://localhost:8080/health`,
    output: `Starting NAWA dev server on 0.0.0.0:8080
Data directory: ./nawa-data
Hot reload: watching src/ for changes

Press Ctrl+C to stop

✓ NAWA dev server running on http://localhost:8080
Watching src for changes...

{"keys":0,"memtable_bytes":0,"stats":{"puts":0,"gets":0,...},"status":"ok"}`,
    tip: "Hot reload يفحص src/ كل 500ms. أي تعديل → kill → rebuild → restart تلقائياً.",
    duration: "~1 second",
  },
  {
    id: 4,
    title: "Store Your First Data",
    titleAr: "خزّن بياناتك الأولى",
    desc: "NAWA-DB مدمج — لا حاجة لـ PostgreSQL. استخدم curl لتخزين واسترجاع البيانات.",
    code: `# Store a blog post (JSON document)
curl -X POST http://localhost:8080/post:1 \\
  -d '{
    "title": "مرحباً من NAWA",
    "body": "أول مقال في مدونتي الجديدة",
    "tags": ["rust", "nawa"],
    "published": true
  }'

# Retrieve it
curl http://localhost:8080/post:1

# Store more posts
curl -X POST http://localhost:8080/post:2 \\
  -d '{"title": "io_uring deep dive", "tags": ["rust", "kernel"]}'

# Scan all posts
curl http://localhost:8080/scan/post:`,
    output: `# POST response:
stored: post:1

# GET response:
{
  "title": "مرحباً من NAWA",
  "body": "أول مقال في مدونتي الجديدة",
  "tags": ["rust", "nawa"],
  "published": true
}

# SCAN response:
{"count":2,"results":[
  {"key":"post:1","value":"{...}"},
  {"key":"post:2","value":"{...}"}
]}`,
    tip: "NAWA-DB يدعم: raw bytes + JSON documents. الـ server يكتشف النوع تلقائياً.",
    duration: "~instant",
  },
  {
    id: 5,
    title: "Add a Custom Route",
    titleAr: "أضف route مخصص",
    desc: "افتح src/main.rs وأضف route جديدة. الـ hot reload سيُعيد التشغيل تلقائياً.",
    code: `// src/main.rs — add after existing routes

// GET /posts — list all posts
router.get("/posts", move || {
    let db = db.clone();
    async move {
        let posts = db.scan_prefix("post:", 100);
        let body: Vec<_> = posts.iter()
            .map(|(k, v)| serde_json::json!({
                "key": String::from_utf8_lossy(k),
                "value": v.display()
            }))
            .collect();
        Response::json(&serde_json::json!({
            "posts": body,
            "count": body.len()
        }))
    }
});

// GET /health/detailed — detailed health
router.get("/health/detailed", move || {
    let db = db.clone();
    async move {
        let stats = db.stats();
        Response::json(&serde_json::json!({
            "status": "ok",
            "keys": db.len(),
            "memtable_bytes": db.memtable_size(),
            "ops": {
                "puts": stats.puts,
                "gets": stats.gets,
                "deletes": stats.deletes
            }
        }))
    }
});`,
    output: `⚡ Change detected — rebuilding...
  Compiling...
  ✓ Build complete — restarting server
✓ NAWA dev server restarted on http://localhost:8080

# Test the new route:
curl http://localhost:8080/posts
{"posts":[...],"count":2}

curl http://localhost:8080/health/detailed
{"status":"ok","keys":2,"memtable_bytes":256,...}`,
    tip: "الـ router يدعم: :params (مثل /users/:id)، *wildcards، و literal routes. Literal تُطابق أولاً.",
    duration: "~67ms (hot reload)",
  },
  {
    id: 6,
    title: "Deploy to Production",
    titleAr: "انشر للإنتاج",
    desc: "انشر موقعك على أي VPS بأمر واحد. NAWA يبني، يرفع، ويُشغّل تلقائياً.",
    code: `# Deploy to a VPS via SSH
nawa deploy --target user@your-vps.com

# Or build a Docker image
docker build -t my-blog .
docker run -d -p 80:8080 -v nawa-data:/var/lib/nawa my-blog

# Or use docker-compose
docker compose up -d

# Verify deployment
curl http://your-vps.com/health`,
    output: `🚀 Deploying to user@your-vps.com...
  Remote data: /var/lib/nawa

📦 Step 1/4: Building release binary...
  ✓ Binary built

📦 Step 2/4: Creating tarball...
  ✓ Tarball: /tmp/nawad-deploy.tar.gz (3.5 MB)

📦 Step 3/4: Uploading to user@your-vps.com...
  ✓ Uploaded

📦 Step 4/4: Remote install + start...
  ✓ Installed and started

✓ Deployment complete!
  Server: http://your-vps.com:8080
  Health: http://your-vps.com:8080/health
  Metrics: http://your-vps.com:8080/metrics`,
    tip: "NAWA يعمل على VPS بـ 512MB RAM ($3/شهر). لا حاجة لـ PostgreSQL أو Redis أو Nginx.",
    duration: "~30 seconds",
  },
  {
    id: 7,
    title: "Monitor Your Site",
    titleAr: "راقب موقعك",
    desc: "Prometheus metrics مدمجة. اربط Grafana لمراقبة الأداء في الوقت الحقيقي.",
    code: `# View Prometheus metrics
curl http://localhost:8080/metrics

# Check io_uring stats
curl http://localhost:8080/uring

# In Grafana, add Prometheus datasource:
# URL: http://your-server:8080/metrics
#
# Useful queries:
# rate(nawa_http_requests_total[1m])
# rate(nawa_db_puts_total[1m]) + rate(nawa_db_gets_total[1m])
# nawa_db_keys
# nawa_uring_in_flight`,
    output: `# HELP nawa_db_puts_total Total DB put operations
# TYPE nawa_db_puts_total counter
nawa_db_puts_total 2
# HELP nawa_db_gets_total Total DB get operations
# TYPE nawa_db_gets_total counter
nawa_db_gets_total 5
# HELP nawa_db_keys Current number of keys in DB
# TYPE nawa_db_keys gauge
nawa_db_keys 2
# HELP nawa_uring_submitted_total Total io_uring submissions
# TYPE nawa_uring_submitted_total counter
nawa_uring_submitted_total 12
# HELP nawa_http_requests_total Total HTTP requests
# TYPE nawa_http_requests_total counter
nawa_http_requests_total 15`,
    tip: "15 metrics جاهزة: DB ops, io_uring stats, HTTP requests, WASM plugins. كلها في Prometheus format.",
    duration: "~instant",
  },
];

export function BuildFirstWebsite() {
  const [activeStep, setActiveStep] = useState(0);
  const [copied, setCopied] = useState(false);
  const step = STEPS[activeStep];

  const copyCode = () => {
    navigator.clipboard?.writeText(step.code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <section id="tutorial" className="relative py-24 lg:py-32">
      <div className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="ابدأ الآن"
          eyebrowEn="Build Your First Website"
          title="من الصفر لموقع منشور · 7 خطوات"
          titleEn="From zero to deployed — 7 steps"
          desc="دليل تفاعلي كامل لبناء أول موقع ويب بـ NAWA. كل خطوة تحتوي كود قابل للنسخ + expected output + نصائح."
          descEn="Interactive guide to build your first NAWA website. Each step has copyable code + expected output + tips."
        />

        {/* Progress bar */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 flex items-center gap-2"
        >
          {STEPS.map((s, i) => (
            <button
              key={s.id}
              onClick={() => setActiveStep(i)}
              className="group flex-1"
            >
              <div
                className={`h-1.5 rounded-full transition-all ${
                  i <= activeStep
                    ? "bg-primary"
                    : "bg-border/40 group-hover:bg-border/60"
                }`}
              />
              <div className={`mt-2 text-[10px] text-center mono transition-colors ${
                i === activeStep ? "text-primary font-bold" : "text-muted-foreground"
              }`}>
                {s.id}
              </div>
            </button>
          ))}
        </motion.div>

        {/* Step content */}
        <AnimatePresence mode="wait">
          <motion.div
            key={activeStep}
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            transition={{ duration: 0.3 }}
            className="mt-8 grid lg:grid-cols-5 gap-6"
          >
            {/* Left: step info */}
            <div className="lg:col-span-2 space-y-4">
              <div className="flex items-center gap-3">
                <div className="w-12 h-12 rounded-xl bg-primary/15 text-primary grid place-items-center font-bold text-xl mono">
                  {step.id}
                </div>
                <div>
                  <h3 className="text-lg font-bold ar">{step.titleAr}</h3>
                  <p className="text-xs text-muted-foreground mono">{step.title}</p>
                </div>
              </div>

              <p className="text-sm text-foreground/90 leading-relaxed ar">{step.desc}</p>

              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <Badge variant="outline" className="mono text-[10px]">
                  ⏱ {step.duration}
                </Badge>
                <Badge variant="outline" className="mono text-[10px] border-accent/40 text-accent">
                  Step {step.id}/{STEPS.length}
                </Badge>
              </div>

              {step.tip && (
                <div className="p-3 rounded-lg border border-primary/30 bg-primary/5">
                  <div className="flex items-start gap-2">
                    <Lightbulb className="w-4 h-4 text-primary shrink-0 mt-0.5" />
                    <p className="text-xs text-foreground/80 leading-relaxed ar">{step.tip}</p>
                  </div>
                </div>
              )}

              {/* Navigation */}
              <div className="flex items-center gap-2 pt-4">
                <Button
                  variant="outline"
                  size="sm"
                  disabled={activeStep === 0}
                  onClick={() => setActiveStep((s) => Math.max(0, s - 1))}
                  className="border-border/60"
                >
                  <ArrowLeft className="w-3.5 h-3.5 rotate-180" />
                  <span className="mr-1 ar">السابق</span>
                </Button>
                {activeStep < STEPS.length - 1 ? (
                  <Button
                    size="sm"
                    onClick={() => setActiveStep((s) => Math.min(STEPS.length - 1, s + 1))}
                    className="bg-primary hover:bg-primary/90 text-primary-foreground"
                  >
                    <span className="ar">التالي</span>
                    <ArrowLeft className="w-3.5 h-3.5" />
                  </Button>
                ) : (
                  <Button
                    size="sm"
                    onClick={() => document.getElementById("start")?.scrollIntoView({ behavior: "smooth" })}
                    className="bg-accent hover:bg-accent/90 text-accent-foreground"
                  >
                    <PartyPopper className="w-3.5 h-3.5" />
                    <span className="mr-1 ar">احتفل! 🎉</span>
                  </Button>
                )}
              </div>
            </div>

            {/* Right: code + output */}
            <div className="lg:col-span-3 space-y-3">
              {/* Code block */}
              <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
                <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
                  <div className="flex items-center gap-2">
                    <Terminal className="w-4 h-4 text-primary" />
                    <span className="text-xs font-medium mono">terminal</span>
                  </div>
                  <button
                    onClick={copyCode}
                    className="flex items-center gap-1 px-2 py-1 rounded text-[10px] mono hover:bg-muted transition-colors text-muted-foreground hover:text-foreground"
                  >
                    {copied ? <Check className="w-3 h-3 text-accent" /> : <Copy className="w-3 h-3" />}
                    {copied ? "copied!" : "copy"}
                  </button>
                </div>
                <pre className="p-4 overflow-x-auto scrollbar-narrow text-xs leading-relaxed mono">
                  <code className="text-foreground/90 whitespace-pre">
                    {step.code.split("\n").map((line, i) => (
                      <div key={i} className={line.startsWith("#") ? "text-green-600/70" : ""}>
                        {line || " "}
                      </div>
                    ))}
                  </code>
                </pre>
              </div>

              {/* Output block */}
              {step.output && (
                <div className="rounded-2xl border border-accent/30 overflow-hidden bg-[#0d0c0a]">
                  <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-accent/10">
                    <div className="flex items-center gap-2">
                      <Play className="w-4 h-4 text-accent" />
                      <span className="text-xs font-medium mono">expected output</span>
                    </div>
                    <Badge variant="outline" className="mono text-[9px] border-accent/40 text-accent">
                      live preview
                    </Badge>
                  </div>
                  <pre className="p-4 overflow-x-auto scrollbar-narrow text-xs leading-relaxed mono">
                    <code className="text-accent/80 whitespace-pre">
                      {step.output.split("\n").map((line, i) => (
                        <div key={i}>{line || " "}</div>
                      ))}
                    </code>
                  </pre>
                </div>
              )}
            </div>
          </motion.div>
        </AnimatePresence>

        {/* Quick links */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-12 grid sm:grid-cols-2 lg:grid-cols-4 gap-3"
        >
          {[
            { icon: Download, label: "تحميل NAWA", href: "https://github.com/amir-helal-ali/nawa-web-os/releases", ar: "Binary جاهز" },
            { icon: BookOpen, label: "التوثيق الكامل", href: "https://amir-helal-ali.github.io/nawa-web-os/", ar: "mdBook" },
            { icon: FileCode2, label: "مثال كامل", href: "#tutorial", ar: "كود Rust" },
            { icon: ExternalLink, label: "GitHub Repo", href: "https://github.com/amir-helal-ali/nawa-web-os", ar: "الكود المصدري" },
          ].map((link) => (
            <a
              key={link.label}
              href={link.href}
              target={link.href.startsWith("http") ? "_blank" : undefined}
              rel="noopener noreferrer"
              className="group flex items-center gap-3 p-4 rounded-xl border border-border/60 bg-card/60 hover:border-primary/40 hover:bg-card transition-all"
            >
              <div className="p-2 rounded-lg bg-primary/15 text-primary group-hover:scale-110 transition-transform">
                <link.icon className="w-4 h-4" strokeWidth={1.5} />
              </div>
              <div>
                <div className="text-sm font-medium ar">{link.label}</div>
                <div className="text-[10px] text-muted-foreground">{link.ar}</div>
              </div>
            </a>
          ))}
        </motion.div>
      </div>
    </section>
  );
}
