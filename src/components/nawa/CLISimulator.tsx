"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Terminal, ChevronRight, CornerDownLeft, Cpu, Zap } from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type Line = {
  type: "in" | "out" | "success" | "err" | "wait";
  text: string;
  delay?: number;
};

const HELP = `nawa CLI v0.1.0 — available commands:
  nawa create <name> [--template <t>]   Scaffold a new project
  nawa dev                               Start dev server with hot reload
  nawa build                             Build optimized production binary
  nawa deploy [--target <ssh>]           Deploy to remote server
  nawa db shell                          Open NAWA-DB shell
  nawa plugins list                      List installed WASM plugins
  nawa plugins install <name>            Install a plugin
  nawa benchmark                         Run performance benchmarks
  nawa --version                         Show version
  help                                   Show this help
  clear                                  Clear screen`;

const TEMPLATES = ["blog", "saas", "shop", "realtime", "booking", "portfolio"];

export function CLISimulator() {
  const [lines, setLines] = useState<Line[]>([
    { type: "out", text: "NAWA CLI v0.1.0 (alpha) — type 'help' to get started" },
    { type: "out", text: "" },
  ]);
  const [input, setInput] = useState("");
  const [history, setHistory] = useState<string[]>([]);
  const [historyIdx, setHistoryIdx] = useState(-1);
  const [busy, setBusy] = useState(false);
  const [project, setProject] = useState<{ name: string; template: string } | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const outputRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    outputRef.current?.scrollTo({ top: outputRef.current.scrollHeight, behavior: "smooth" });
  }, [lines]);

  const addLines = useCallback((newLines: Line[]) => {
    setLines((l) => [...l, ...newLines]);
  }, []);

  const typeLine = useCallback(async (line: Line) => {
    setLines((l) => [...l, line]);
  }, []);

  const runCommand = useCallback(
    async (raw: string) => {
      const cmd = raw.trim();
      if (!cmd) return;

      addLines([{ type: "in", text: cmd }]);
      setHistory((h) => [...h, cmd]);
      setHistoryIdx(-1);

      const parts = cmd.split(/\s+/);
      const verb = parts[0].toLowerCase();

      if (verb === "help") {
        addLines(HELP.split("\n").map((t) => ({ type: "out" as const, text: t })));
        addLines([{ type: "out", text: "" }]);
        return;
      }

      if (verb === "clear") {
        setLines([]);
        return;
      }

      if (verb === "nawa") {
        const sub = parts[1]?.toLowerCase();

        if (sub === "--version" || sub === "-v") {
          addLines([{ type: "out", text: "nawa 0.1.0-alpha (commit: a4f3c21)" }]);
          addLines([{ type: "out", text: "  rust: 1.83.0 (musl, static)" }]);
          addLines([{ type: "out", text: "  kernel: linux 6.6+ (io_uring)" }]);
          addLines([{ type: "out", text: "" }]);
          return;
        }

        if (sub === "create") {
          if (parts.length < 3) {
            addLines([{ type: "err", text: "Usage: nawa create <name> [--template <t>]" }]);
            addLines([{ type: "out", text: "" }]);
            return;
          }
          const name = parts[2];
          const tmplIdx = parts.indexOf("--template");
          const template = tmplIdx > -1 ? parts[tmplIdx + 1] : "blog";
          if (!TEMPLATES.includes(template)) {
            addLines([{ type: "err", text: `Unknown template: ${template}. Available: ${TEMPLATES.join(", ")}` }]);
            addLines([{ type: "out", text: "" }]);
            return;
          }
          setBusy(true);
          setProject({ name, template });

          const seq: Line[] = [
            { type: "wait", text: `Creating project "${name}" from template "${template}"...` },
            { type: "out", text: `  ✓ Cargo.toml` },
            { type: "out", text: `  ✓ nawa.toml` },
            { type: "out", text: `  ✓ src/main.rs` },
            { type: "out", text: `  ✓ src/routes/ (5 files)` },
            { type: "out", text: `  ✓ src/db/schema.nawa` },
            { type: "out", text: `  ✓ templates/ (4 files)` },
            { type: "out", text: `  ✓ Dockerfile` },
            { type: "out", text: `  ✓ docker-compose.yml` },
            { type: "out", text: `  ✓ .gitignore` },
            { type: "success", text: `✓ Project created in ./${name}/ (12 files, 47KB)` },
            { type: "out", text: "" },
            { type: "out", text: `Next steps:` },
            { type: "out", text: `  cd ${name}` },
            { type: "out", text: `  nawa dev` },
            { type: "out", text: "" },
          ];

          for (const line of seq) {
            await new Promise((r) => setTimeout(r, 180));
            addLines([line]);
          }
          setBusy(false);
          return;
        }

        if (sub === "dev") {
          if (!project) {
            addLines([{ type: "err", text: "No project found. Run 'nawa create <name>' first." }]);
            addLines([{ type: "out", text: "" }]);
            return;
          }
          setBusy(true);
          const seq: Line[] = [
            { type: "wait", text: "Compiling..." },
            { type: "out", text: `    Compiling ${project.name} v0.1.0` },
            { type: "out", text: `    Compiling nawa-runtime v0.1.0` },
            { type: "out", text: `    Compiling nawa-db v0.1.0` },
            { type: "wait", text: "Linking..." },
            { type: "success", text: `✓ Built in 1.2s` },
            { type: "out", text: "" },
            { type: "success", text: `▶ NAWA dev server running` },
            { type: "out", text: `  → http://localhost:8080` },
            { type: "out", text: `  → http://[::1]:8080` },
            { type: "out", text: `  template: ${project.template}` },
            { type: "out", text: `  hot reload: enabled` },
            { type: "out", text: `  Press Ctrl+C to stop` },
            { type: "out", text: "" },
            { type: "wait", text: "[HMR] waiting for changes..." },
            { type: "out", text: "" },
          ];
          for (const line of seq) {
            await new Promise((r) => setTimeout(r, 220));
            addLines([line]);
          }
          setBusy(false);
          return;
        }

        if (sub === "build") {
          setBusy(true);
          const seq: Line[] = [
            { type: "wait", text: "Building optimized release binary..." },
            { type: "out", text: `   Compiling ${project?.name || "nawa-app"} v0.1.0 (release)` },
            { type: "wait", text: "Optimizing (LTO + strip)..." },
            { type: "success", text: `✓ Release binary built in 8.4s` },
            { type: "out", text: `  target/release/nawad` },
            { type: "out", text: `  size: 11.2 MB (stripped, musl static)` },
            { type: "out", text: "" },
            { type: "out", text: `Docker image:` },
            { type: "wait", text: "Building Docker image..." },
            { type: "success", text: `✓ nawa/os:0.1.0 (14.8 MB)` },
            { type: "out", text: "" },
          ];
          for (const line of seq) {
            await new Promise((r) => setTimeout(r, 280));
            addLines([line]);
          }
          setBusy(false);
          return;
        }

        if (sub === "deploy") {
          if (!project) {
            addLines([{ type: "err", text: "No project to deploy. Run 'nawa create' first." }]);
            addLines([{ type: "out", text: "" }]);
            return;
          }
          setBusy(true);
          const seq: Line[] = [
            { type: "wait", text: "Connecting to remote server..." },
            { type: "out", text: `  → ssh://user@your-vps.com` },
            { type: "success", text: "✓ Connected" },
            { type: "wait", text: "Uploading Docker image..." },
            { type: "out", text: `  14.8 MB / 14.8 MB [####################] 100%` },
            { type: "success", text: "✓ Image uploaded" },
            { type: "wait", text: "Starting container..." },
            { type: "out", text: `  pulling nawa/os:0.1.0...` },
            { type: "out", text: `  starting container nawa-prod...` },
            { type: "success", text: "✓ Container running (PID 8421, UID 10001)" },
            { type: "wait", text: "Acquiring TLS certificate..." },
            { type: "success", text: "✓ Let's Encrypt cert issued (RSA 2048, 90 days)" },
            { type: "out", text: "" },
            { type: "success", text: "🎉 Deployed successfully!" },
            { type: "out", text: `  → https://your-vps.com/` },
            { type: "out", text: `  → HTTP/3 enabled (443/udp)` },
            { type: "out", text: `  → Health: OK (latency 0.4ms)` },
            { type: "out", text: "" },
          ];
          for (const line of seq) {
            await new Promise((r) => setTimeout(r, 260));
            addLines([line]);
          }
          setBusy(false);
          return;
        }

        if (sub === "benchmark") {
          setBusy(true);
          const seq: Line[] = [
            { type: "wait", text: "Running NAWA benchmarks..." },
            { type: "out", text: "" },
            { type: "out", text: "→ HTTP throughput (1 vCPU, 512MB):" },
            { type: "wait", text: "  warming up (5s)..." },
            { type: "wait", text: "  running (15s)..." },
            { type: "success", text: "  ✓ 8,432 req/sec (p50: 0.18ms, p99: 0.42ms)" },
            { type: "out", text: "" },
            { type: "out", text: "→ DB read latency:" },
            { type: "success", text: "  ✓ 92μs avg (1,000,000 ops in 92s)" },
            { type: "out", text: "" },
            { type: "out", text: "→ SSR render time:" },
            { type: "success", text: "  ✓ 1.2ms avg (complex page, 50 elements)" },
            { type: "out", text: "" },
            { type: "out", text: "→ Cold start:" },
            { type: "success", text: "  ✓ 180ms (container boot + first request)" },
            { type: "out", text: "" },
            { type: "success", text: "✓ All benchmarks passed" },
            { type: "out", text: "" },
          ];
          for (const line of seq) {
            await new Promise((r) => setTimeout(r, 200));
            addLines([line]);
          }
          setBusy(false);
          return;
        }

        if (sub === "plugins") {
          const action = parts[2]?.toLowerCase();
          if (action === "list") {
            addLines([
              { type: "out", text: "Installed WASM plugins:" },
              { type: "out", text: "  ✓ auth-jwt     v0.2.1   (12KB)   — JWT auth handler" },
              { type: "out", text: "  ✓ cache-redis  v0.1.0   (8KB)    — Redis-compatible cache" },
              { type: "out", text: "  ✓ search-fts   v0.3.0   (24KB)   — Full-text search" },
              { type: "out", text: "  ✓ analytics    v0.1.2   (15KB)   — Privacy-first analytics" },
              { type: "out", text: "" },
              { type: "out", text: "4 plugins, 59KB total" },
              { type: "out", text: "" },
            ]);
            return;
          }
          if (action === "install") {
            const name = parts[3];
            if (!name) {
              addLines([{ type: "err", text: "Usage: nawa plugins install <name>" }]);
              addLines([{ type: "out", text: "" }]);
              return;
            }
            setBusy(true);
            const seq: Line[] = [
              { type: "wait", text: `Downloading plugin "${name}"...` },
              { type: "out", text: `  fetching from registry.nawa.dev/${name}.wasm` },
              { type: "success", text: `✓ Downloaded (18KB)` },
              { type: "wait", text: "Verifying signature..." },
              { type: "success", text: "✓ Signature verified (ed25519)" },
              { type: "wait", text: "Installing..." },
              { type: "success", text: `✓ Plugin "${name}" installed` },
              { type: "out", text: "" },
            ];
            for (const line of seq) {
              await new Promise((r) => setTimeout(r, 220));
              addLines([line]);
            }
            setBusy(false);
            return;
          }
          addLines([{ type: "err", text: "Usage: nawa plugins [list|install]" }]);
          addLines([{ type: "out", text: "" }]);
          return;
        }

        if (sub === "db") {
          if (parts[2]?.toLowerCase() === "shell") {
            addLines([
              { type: "out", text: "Connecting to NAWA-DB (in-process)..." },
              { type: "success", text: "✓ Connected (memtable: 24 entries, sstables: 3)" },
              { type: "out", text: "" },
              { type: "out", text: "Use the NAWA-DB shell below for interactive queries." },
              { type: "out", text: "Commands: PUT, GET, DEL, SCAN, COUNT, BEGIN, COMMIT" },
              { type: "out", text: "" },
            ]);
            return;
          }
        }

        addLines([{ type: "err", text: `Unknown nawa subcommand: ${sub || "(none)"}` }]);
        addLines([{ type: "out", text: "Type 'help' for available commands." }]);
        addLines([{ type: "out", text: "" }]);
        return;
      }

      addLines([{ type: "err", text: `Unknown command: ${verb}. Type 'help' for help.` }]);
      addLines([{ type: "out", text: "" }]);
    },
    [project, addLines]
  );

  const onSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (busy) return;
    const v = input;
    setInput("");
    runCommand(v);
  };

  const onKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "ArrowUp") {
      e.preventDefault();
      if (history.length === 0) return;
      const newIdx = historyIdx === -1 ? history.length - 1 : Math.max(0, historyIdx - 1);
      setHistoryIdx(newIdx);
      setInput(history[newIdx]);
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (historyIdx === -1) return;
      const newIdx = historyIdx + 1;
      if (newIdx >= history.length) {
        setHistoryIdx(-1);
        setInput("");
      } else {
        setHistoryIdx(newIdx);
        setInput(history[newIdx]);
      }
    }
  };

  const lineClass = (t: Line["type"]) =>
    t === "in"
      ? "text-primary"
      : t === "err"
      ? "text-destructive"
      : t === "success"
      ? "text-accent"
      : t === "wait"
      ? "text-yellow-400/80"
      : "text-foreground/80";

  return (
    <section id="cli" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-grid opacity-30 pointer-events-none" />
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="CLI تفاعلي"
          eyebrowEn="CLI Simulator"
          title="nawa CLI · جرّب الأوامر بنفسك"
          titleEn="nawa CLI — try the commands yourself"
          desc="محاكي حقيقي لـ nawa CLI. اكتب الأوامر مثل nawa create، nawa dev، nawa deploy وراقب تجربة المطور الكاملة — من السكافولد حتى النشر."
          descEn="A real nawa CLI simulator. Run commands like nawa create, nawa dev, nawa deploy and watch the full developer experience unfold."
        />

        {/* Feature highlights */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 grid grid-cols-2 lg:grid-cols-4 gap-3"
        >
          {[
            { cmd: "nawa create", desc: "سكافولد مشروع جديد", time: "~2s" },
            { cmd: "nawa dev", desc: "خادم تطوير + hot reload", time: "1.2s build" },
            { cmd: "nawa deploy", desc: "نشر بضغطة واحدة", time: "~30s" },
            { cmd: "nawa benchmark", desc: "اختبارات أداء", time: "15s run" },
          ].map((c) => (
            <div key={c.cmd} className="p-3 rounded-xl border border-border/60 bg-card/60">
              <code className="text-xs mono text-primary">{c.cmd}</code>
              <div className="text-xs text-muted-foreground mt-1 ar">{c.desc}</div>
              <div className="text-[10px] text-accent mt-1 mono">{c.time}</div>
            </div>
          ))}
        </motion.div>

        {/* Terminal */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-8 grid lg:grid-cols-3 gap-4"
        >
          {/* Terminal - 2/3 width */}
          <div className="lg:col-span-2 rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
            <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
              <div className="flex items-center gap-2">
                <Terminal className="w-4 h-4 text-primary" />
                <span className="text-sm font-medium mono">nawa-cli</span>
                {busy && (
                  <Badge variant="outline" className="mono text-[10px] border-yellow-500/40 text-yellow-500">
                    <Zap className="w-2.5 h-2.5 ml-1" />
                    running
                  </Badge>
                )}
              </div>
              <div className="flex items-center gap-2 text-[10px] text-muted-foreground mono">
                <Cpu className="w-3 h-3" />
                <span>zsh · nawa</span>
              </div>
            </div>

            <div
              ref={outputRef}
              onClick={() => !busy && inputRef.current?.focus()}
              className="h-96 overflow-y-auto scrollbar-narrow p-4 font-mono text-xs leading-relaxed cursor-text"
            >
              {lines.map((line, i) => (
                <div key={i} className={`whitespace-pre-wrap break-words ${lineClass(line.type)}`}>
                  {line.type === "in" ? (
                    <span className="flex gap-2">
                      <span className="text-accent shrink-0">$</span>
                      <span>{line.text}</span>
                    </span>
                  ) : (
                    line.text || " "
                  )}
                </div>
              ))}
              {/* Prompt */}
              {!busy && (
                <form onSubmit={onSubmit} className="flex items-center gap-2 mt-1">
                  <span className="text-accent shrink-0">$</span>
                  <input
                    ref={inputRef}
                    value={input}
                    onChange={(e) => setInput(e.target.value)}
                    onKeyDown={onKeyDown}
                    placeholder="type a command (try: nawa create my-app --template blog)"
                    className="flex-1 bg-transparent border-0 outline-none text-foreground placeholder:text-muted-foreground/50 font-mono text-xs"
                    autoFocus
                    spellCheck={false}
                    autoComplete="off"
                  />
                  <CornerDownLeft className="w-3 h-3 text-muted-foreground/50 shrink-0" />
                </form>
              )}
              {busy && (
                <div className="flex items-center gap-2 mt-1 text-yellow-400/80">
                  <span className="text-accent">$</span>
                  <span className="animate-nawa-pulse">▊ working...</span>
                </div>
              )}
            </div>

            {/* Quick commands */}
            <div className="px-3 py-2 border-t border-border/40 bg-card/40 flex flex-wrap gap-1">
              {[
                "help",
                "nawa create my-app --template saas",
                "nawa dev",
                "nawa build",
                "nawa deploy",
                "nawa benchmark",
                "nawa plugins list",
                "nawa --version",
              ].map((c) => (
                <button
                  key={c}
                  onClick={() => {
                    if (busy) return;
                    setInput(c);
                    inputRef.current?.focus();
                  }}
                  disabled={busy}
                  className="px-2 py-0.5 rounded text-[10px] mono bg-card/80 border border-border/40 hover:border-primary/40 hover:text-primary transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  {c}
                </button>
              ))}
            </div>
          </div>

          {/* Side panel: project status */}
          <div className="space-y-4">
            <div className="rounded-2xl border border-border/60 bg-card/60 p-4">
              <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-3 mono">
                project status
              </div>
              {project ? (
                <div className="space-y-2">
                  <div className="flex justify-between text-xs">
                    <span className="text-muted-foreground">name:</span>
                    <code className="mono text-primary">{project.name}</code>
                  </div>
                  <div className="flex justify-between text-xs">
                    <span className="text-muted-foreground">template:</span>
                    <code className="mono text-accent">{project.template}</code>
                  </div>
                  <div className="flex justify-between text-xs">
                    <span className="text-muted-foreground">status:</span>
                    <span className="text-accent text-xs">created ✓</span>
                  </div>
                  <div className="flex justify-between text-xs">
                    <span className="text-muted-foreground">size:</span>
                    <span className="mono">47 KB</span>
                  </div>
                </div>
              ) : (
                <div className="text-xs text-muted-foreground ar">
                  لا يوجد مشروع بعد. ابدأ بـ <code className="mono text-primary">nawa create</code>.
                </div>
              )}
            </div>

            <div className="rounded-2xl border border-border/60 bg-card/60 p-4">
              <div className="text-[10px] text-muted-foreground uppercase tracking-wide mb-3 mono">
                cli features
              </div>
              <ul className="space-y-2 text-xs">
                {[
                  "Hot reload dev server",
                  "Single-command deploy",
                  "Built-in benchmarking",
                  "WASM plugin manager",
                  "DB shell access",
                  "Auto TLS provisioning",
                ].map((f) => (
                  <li key={f} className="flex items-center gap-2">
                    <ChevronRight className="w-3 h-3 text-primary shrink-0" />
                    <span className="ar">{f}</span>
                  </li>
                ))}
              </ul>
            </div>

            <div className="rounded-2xl border border-primary/30 bg-gradient-to-br from-primary/10 to-accent/5 p-4">
              <div className="text-[10px] text-primary uppercase tracking-wide mb-2 mono">
                💡 try this
              </div>
              <p className="text-xs text-foreground/90 ar leading-relaxed">
                ابدأ بإنشاء مشروع، ثم شغّل <code className="mono text-primary">nawa dev</code>، ثم انشره بـ
                <code className="mono text-accent"> nawa deploy</code>.
              </p>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}
