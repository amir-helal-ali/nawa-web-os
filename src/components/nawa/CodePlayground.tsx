"use client";

import { useState, useMemo } from "react";
import { motion } from "framer-motion";
import {
  Code2,
  Play,
  RotateCcw,
  Copy,
  Check,
  Eye,
  Terminal,
  Zap,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

const TEMPLATES = {
  hello: `// Hello World handler
#[get("/")]
pub fn index() -> Html {
    Html::new()
        .h1("مرحباً من NAWA")
        .p("أول handler لك — مكتوب بـ Rust")
        .render()
}`,
  user: `// Dynamic route with typed params
#[get("/users/:id")]
pub fn user(Path(id): Path<u64>) -> Json<User> {
    let user = nawa_db::get(&format!("user:{}", id))
        .unwrap_or_default();

    Json(user)
}`,
  counter: `// Stateful handler with DB write
#[post("/increment")]
pub fn increment() -> Json<Counter> {
    let mut count: i64 = nawa_db::get("counter")
        .unwrap_or(0);
    count += 1;

    nawa_db::put("counter", count);

    Json(Counter { value: count })
}`,
  streaming: `// Streaming SSR with Suspense
#[get("/dashboard")]
pub fn dashboard() -> Stream {
    let header = render_header();           // instant
    let stats = Suspense::new(async {
        fetch_stats().await                  // ~200ms
    });

    Stream::new()
        .chunk(header)                       // sent first
        .chunk(stats)                        // streamed when ready
        .render()
}`,
  parallel: `// Parallel DB queries (async)
#[get("/feed")]
pub fn feed() -> Json<Feed> {
    let (posts, users, tags) = tokio::join!(
        nawa_db::scan("post:"),
        nawa_db::scan("user:"),
        nawa_db::scan("tag:"),
    );

    Json(Feed { posts, users, tags })
}`,
};

type TemplateKey = keyof typeof TEMPLATES;

const TEMPLATE_META: Array<{ key: TemplateKey; label: string; desc: string }> = [
  { key: "hello", label: "Hello World", desc: "أول handler" },
  { key: "user", label: "User Route", desc: "route ديناميكي" },
  { key: "counter", label: "DB Counter", desc: "كتابة لقاعدة البيانات" },
  { key: "streaming", label: "Streaming SSR", desc: "بث مع Suspense" },
  { key: "parallel", label: "Parallel Queries", desc: "استعلامات متوازية" },
];

export function CodePlayground() {
  const [active, setActive] = useState<TemplateKey>("hello");
  const [code, setCode] = useState(TEMPLATES.hello);
  const [output, setOutput] = useState<{ type: "html" | "json" | "log"; content: string } | null>(null);
  const [running, setRunning] = useState(false);
  const [copied, setCopied] = useState(false);

  const selectTemplate = (key: TemplateKey) => {
    setActive(key);
    setCode(TEMPLATES[key]);
    setOutput(null);
  };

  const run = () => {
    setRunning(true);
    setOutput(null);

    setTimeout(() => {
      const result = simulate(active);
      setOutput(result);
      setRunning(false);
    }, 800);
  };

  const reset = () => {
    setCode(TEMPLATES[active]);
    setOutput(null);
  };

  const copyCode = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const lineCount = useMemo(() => code.split("\n").length, [code]);

  return (
    <section id="playground" className="relative py-24 lg:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="ملعب الكود"
          eyebrowEn="Code Playground"
          title="اكتب Rust · شاهد النتيجة فوراً"
          titleEn="Write Rust · see the result instantly"
          desc="اكتب handler بـ Rust واضغط تشغيل لمحاكاة الـ output. كل قالب يُظهر جانباً مختلفاً من NAWA: SSR، JSON APIs، streaming، وasync parallelism."
          descEn="Write a Rust handler and hit run to simulate the output. Each template showcases a different aspect of NAWA: SSR, JSON APIs, streaming, and async parallelism."
        />

        {/* Template selector */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 flex flex-wrap gap-2"
        >
          {TEMPLATE_META.map((t) => (
            <button
              key={t.key}
              onClick={() => selectTemplate(t.key)}
              className={`group px-3 py-2 rounded-lg border text-right transition-all ${
                active === t.key
                  ? "border-primary bg-primary/10"
                  : "border-border/60 bg-card/40 hover:border-primary/40"
              }`}
            >
              <div className={`text-sm font-medium ${active === t.key ? "text-primary" : "text-foreground"}`}>
                {t.label}
              </div>
              <div className="text-[10px] text-muted-foreground ar">{t.desc}</div>
            </button>
          ))}
        </motion.div>

        {/* Editor + Output */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-6 grid lg:grid-cols-2 gap-4"
        >
          {/* Code editor */}
          <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
            <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
              <div className="flex items-center gap-2">
                <Code2 className="w-4 h-4 text-primary" />
                <span className="text-sm font-medium mono">handler.rs</span>
                <Badge variant="outline" className="mono text-[10px] border-primary/40 text-primary">
                  {lineCount} lines
                </Badge>
              </div>
              <div className="flex items-center gap-1">
                <Button variant="ghost" size="sm" onClick={copyCode} className="h-7 px-2 text-xs">
                  {copied ? <Check className="w-3 h-3 text-accent" /> : <Copy className="w-3 h-3" />}
                </Button>
                <Button variant="ghost" size="sm" onClick={reset} className="h-7 px-2 text-xs">
                  <RotateCcw className="w-3 h-3" />
                </Button>
                <Button
                  size="sm"
                  onClick={run}
                  disabled={running}
                  className="h-7 px-3 text-xs bg-primary hover:bg-primary/90 text-primary-foreground"
                >
                  <Play className="w-3 h-3" />
                  <span className="mr-1 ar">{running ? "يعمل..." : "تشغيل"}</span>
                </Button>
              </div>
            </div>
            <div className="relative">
              <textarea
                value={code}
                onChange={(e) => setCode(e.target.value)}
                spellCheck={false}
                className="w-full h-96 p-4 pl-12 bg-transparent font-mono text-xs leading-relaxed text-foreground/90 outline-none resize-none scrollbar-narrow"
                style={{ tabSize: 4 }}
              />
              {/* Line numbers gutter */}
              <div className="absolute top-0 left-0 h-96 py-4 px-2 text-right text-[10px] text-muted-foreground/40 mono select-none pointer-events-none overflow-hidden">
                {Array.from({ length: Math.max(lineCount, 20) }, (_, i) => (
                  <div key={i} style={{ height: "1.45em" }}>{i + 1}</div>
                ))}
              </div>
            </div>
          </div>

          {/* Output panel */}
          <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
            <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
              <div className="flex items-center gap-2">
                <Eye className="w-4 h-4 text-accent" />
                <span className="text-sm font-medium mono">output</span>
                {output && (
                  <Badge variant="outline" className="mono text-[10px] border-accent/40 text-accent">
                    {output.type.toUpperCase()}
                  </Badge>
                )}
              </div>
              {running && (
                <Badge variant="outline" className="mono text-[10px] border-yellow-500/40 text-yellow-500">
                  <Zap className="w-2.5 h-2.5 ml-1 animate-nawa-pulse" />
                  compiling
                </Badge>
              )}
            </div>

            <div className="h-96 overflow-y-auto scrollbar-narrow p-4 font-mono text-xs">
              {!output && !running && (
                <div className="h-full grid place-items-center text-center">
                  <div>
                    <Terminal className="w-8 h-8 text-muted-foreground/40 mx-auto mb-3" />
                    <p className="text-xs text-muted-foreground ar">
                      اضغط <span className="text-primary">تشغيل</span> لمحاكاة الـ handler
                    </p>
                  </div>
                </div>
              )}

              {running && (
                <div className="space-y-1.5">
                  <div className="text-yellow-400/80">$ cargo build --release</div>
                  <div className="text-muted-foreground">  Compiling handler v0.1.0</div>
                  <div className="text-muted-foreground">  Compiling nawa-runtime v0.1.0</div>
                  <div className="text-yellow-400/80 animate-nawa-pulse">  Linking...</div>
                </div>
              )}

              {output && (
                <motion.div
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  className="space-y-2"
                >
                  <div className="text-accent">✓ compiled in 1.2s · running...</div>
                  <div className="text-muted-foreground">─</div>
                  <div className="text-[10px] text-muted-foreground uppercase tracking-wide mono">
                    response
                  </div>
                  {output.type === "html" ? (
                    <pre className="text-foreground/90 whitespace-pre-wrap break-words leading-relaxed">
                      {output.content}
                    </pre>
                  ) : output.type === "json" ? (
                    <pre className="text-accent whitespace-pre-wrap break-words leading-relaxed">
                      {output.content}
                    </pre>
                  ) : (
                    <pre className="text-yellow-400/80 whitespace-pre-wrap break-words leading-relaxed">
                      {output.content}
                    </pre>
                  )}
                  <div className="text-muted-foreground">─</div>
                  <div className="text-[10px] text-muted-foreground">
                    <span className="text-accent">✓</span> 200 OK ·
                    latency: <span className="text-primary">0.42ms</span> ·
                    size: <span className="text-primary">{(output.content.length / 1024).toFixed(2)}KB</span>
                  </div>
                </motion.div>
              )}
            </div>
          </div>
        </motion.div>

        {/* Tips strip */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-6 grid sm:grid-cols-3 gap-3"
        >
          {[
            { tip: "Type-safe routing", desc: "كل path param يُستخرج كـ typed value في compile-time" },
            { tip: "Zero-cost extractors", desc: "Path, Json, Query — كلها zero-allocation" },
            { tip: "Async-native", desc: "كل handler هو async افتراضياً، لا wrapper" },
          ].map((t) => (
            <div key={t.tip} className="p-3 rounded-xl border border-border/60 bg-card/60">
              <div className="text-sm font-medium text-primary">{t.tip}</div>
              <div className="text-xs text-muted-foreground mt-1 ar">{t.desc}</div>
            </div>
          ))}
        </motion.div>
      </div>
    </section>
  );
}

function simulate(key: TemplateKey): { type: "html" | "json" | "log"; content: string } {
  switch (key) {
    case "hello":
      return {
        type: "html",
        content: `<HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8
Content-Length: 142
X-Powered-By: NAWA/0.1.0
X-Response-Time: 0.18ms

<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head><title>مرحباً من NAWA</title></head>
<body>
  <h1>مرحباً من NAWA</h1>
  <p>أول handler لك — مكتوب بـ Rust</p>
</body>
</html>>`,
      };
    case "user":
      return {
        type: "json",
        content: `<HTTP/1.1 200 OK
Content-Type: application/json
X-Response-Time: 0.42ms

{
  "_id": 1001,
  "name": "أحمد المصري",
  "email": "ahmed@example.com",
  "roles": ["admin", "editor"],
  "meta": {
    "joined": "2026-01-04",
    "plan": "pro"
  }
}>`,
      };
    case "counter":
      return {
        type: "json",
        content: `<HTTP/1.1 200 OK
Content-Type: application/json
X-Response-Time: 0.31ms
X-DB-Writes: 1

{
  "value": 14824
}>`,
      };
    case "streaming":
      return {
        type: "log",
        content: `<[stream] chunk 1 sent at 0.00ms (header)
  ├─ <html><head>...</head><body>
  └─ <header>Dashboard</header>

[stream] chunk 2 streaming at 187ms (stats)
  ├─ <section class="stats">
  │   <h2>Live Stats</h2>
  │   <ul>
  │     <li>Visitors: 1,482</li>
  │     <li>Revenue: $4,231</li>
  │   </ul>
  └─ </section>

[stream] chunk 3 sent at 189ms (footer)
  └─ </body></html>

[done] total time: 189ms (TTBV: 0ms, TTFB: 1.2ms)
>`,
      };
    case "parallel":
      return {
        type: "json",
        content: `<HTTP/1.1 200 OK
Content-Type: application/json
X-Response-Time: 1.18ms
X-DB-Queries: 3 (parallel)

{
  "posts": [
    { "id": 42, "title": "Inside NAWA" },
    { "id": 43, "title": "io_uring deep dive" }
  ],
  "users": [
    { "id": 1001, "name": "أحمد" },
    { "id": 1002, "name": "Sara" }
  ],
  "tags": ["rust", "kernel", "io_uring"]
}>`,
      };
  }
}
