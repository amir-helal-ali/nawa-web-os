"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Database,
  KeyRound,
  FileJson,
  Trash2,
  Search,
  Activity,
  Hash,
  Layers3,
  Zap,
  Lock,
  Terminal,
  ChevronRight,
  CornerDownLeft,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

type OpLog = {
  id: number;
  type: "PUT" | "GET" | "DEL" | "SCAN" | "COUNT" | "BEGIN" | "COMMIT" | "ERROR";
  key: string;
  status: "ok" | "miss" | "err";
  latency: number;
  at: number;
  detail?: string;
};

type DocValue = {
  v: any;
  type: "kv" | "doc";
};

const SEED_DATA: Record<string, DocValue> = {
  "user:1001": {
    type: "doc",
    v: {
      _id: "1001",
      name: "أحمد المصري",
      email: "ahmed@example.com",
      roles: ["admin", "editor"],
      meta: { joined: "2026-01-04", plan: "pro" },
    },
  },
  "user:1002": {
    type: "doc",
    v: {
      _id: "1002",
      name: "Sara Lee",
      email: "sara@example.com",
      roles: ["viewer"],
      meta: { joined: "2026-03-22", plan: "free" },
    },
  },
  "session:9f3a": { type: "kv", v: "user:1001" },
  "config:smtp": { type: "kv", v: "smtp://localhost:587" },
  "counter:visits": { type: "kv", v: 14823 },
  "post:42": {
    type: "doc",
    v: {
      _id: 42,
      title: "Inside NAWA's zero-copy kernel",
      tags: ["rust", "kernel", "io_uring"],
      views: 9211,
    },
  },
};

const HELP_TEXT = `Available commands:
  PUT <key> <value>              Store a key-value pair
  PUT <key> <json>               Store a document (auto-detected)
  GET <key>                      Fetch value by key
  DEL <key>                      Delete a key
  SCAN <prefix>                  List all keys with prefix
  COUNT                          Count all keys
  BEGIN / COMMIT                 Transaction (simulated)
  HELP                           Show this help
  CLEAR                          Clear the screen`;

export function DatabaseDemo() {
  const [store, setStore] = useState<Record<string, DocValue>>(SEED_DATA);
  const [log, setLog] = useState<OpLog[]>([]);
  const [history, setHistory] = useState<string[]>([]);
  const [historyIdx, setHistoryIdx] = useState(-1);
  const [input, setInput] = useState("");
  const [output, setOutput] = useState<Array<{ type: "in" | "out" | "err"; text: string }>>([
    { type: "out", text: "NAWA-DB shell v0.1.0 — type HELP for commands, ↑↓ for history" },
    { type: "out", text: "" },
  ]);
  const [opCounter, setOpCounter] = useState(0);
  const [totalLatency, setTotalLatency] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const outputRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    outputRef.current?.scrollTo({ top: outputRef.current.scrollHeight, behavior: "smooth" });
  }, [output]);

  const pushLog = useCallback((op: OpLog) => {
    setLog((l) => [op, ...l].slice(0, 50));
    setOpCounter((c) => c + 1);
    setTotalLatency((t) => t + op.latency);
  }, []);

  const simulateLatency = () => Math.floor(80 + Math.random() * 280);

  const addOutput = (lines: Array<{ type: "in" | "out" | "err"; text: string }>) => {
    setOutput((o) => [...o, ...lines]);
  };

  const execute = useCallback(
    (raw: string) => {
      const cmd = raw.trim();
      if (!cmd) return;

      // Echo input
      addOutput([{ type: "in", text: cmd }]);

      // Save history
      setHistory((h) => [...h, cmd]);
      setHistoryIdx(-1);

      const parts = cmd.split(/\s+/);
      const verb = parts[0].toUpperCase();

      switch (verb) {
        case "HELP": {
          addOutput(HELP_TEXT.split("\n").map((t) => ({ type: "out" as const, text: t })));
          break;
        }
        case "CLEAR": {
          setOutput([]);
          break;
        }
        case "PUT": {
          if (parts.length < 3) {
            addOutput([{ type: "err", text: "Usage: PUT <key> <value>" }]);
            pushLog({
              id: Date.now(),
              type: "ERROR",
              key: parts[1] || "?",
              status: "err",
              latency: 0,
              at: Date.now(),
              detail: "bad args",
            });
            break;
          }
          const key = parts[1];
          const valueStr = parts.slice(2).join(" ");
          let parsedValue: any = valueStr;
          let type: "kv" | "doc" = "kv";

          // Try JSON parse
          if (valueStr.startsWith("{") || valueStr.startsWith("[")) {
            try {
              parsedValue = JSON.parse(valueStr);
              type = "doc";
            } catch {
              addOutput([{ type: "err", text: `Invalid JSON: ${valueStr}` }]);
              pushLog({
                id: Date.now(),
                type: "ERROR",
                key,
                status: "err",
                latency: simulateLatency(),
                at: Date.now(),
                detail: "invalid JSON",
              });
              break;
            }
          } else if (/^-?\d+$/.test(valueStr)) {
            parsedValue = parseInt(valueStr);
          } else if (/^-?\d*\.\d+$/.test(valueStr)) {
            parsedValue = parseFloat(valueStr);
          }

          setStore((s) => ({ ...s, [key]: { type, v: parsedValue } }));
          const lat = simulateLatency();
          pushLog({
            id: Date.now(),
            type: "PUT",
            key,
            status: "ok",
            latency: lat,
            at: Date.now(),
          });
          addOutput([
            { type: "out", text: `OK  ${key} → ${type === "doc" ? "<doc>" : JSON.stringify(parsedValue)} (${lat}μs)` },
          ]);
          break;
        }
        case "GET": {
          if (parts.length < 2) {
            addOutput([{ type: "err", text: "Usage: GET <key>" }]);
            break;
          }
          const key = parts[1];
          const found = store[key];
          const lat = simulateLatency();
          pushLog({
            id: Date.now(),
            type: "GET",
            key,
            status: found ? "ok" : "miss",
            latency: lat,
            at: Date.now(),
          });
          if (found) {
            addOutput([
              {
                type: "out",
                text: `${JSON.stringify(found.v, null, 2)}  (${lat}μs, ${found.type})`,
              },
            ]);
          } else {
            addOutput([{ type: "err", text: `MISS  key "${key}" not found (${lat}μs)` }]);
          }
          break;
        }
        case "DEL": {
          if (parts.length < 2) {
            addOutput([{ type: "err", text: "Usage: DEL <key>" }]);
            break;
          }
          const key = parts[1];
          setStore((s) => {
            const next = { ...s };
            delete next[key];
            return next;
          });
          const lat = simulateLatency();
          pushLog({
            id: Date.now(),
            type: "DEL",
            key,
            status: "ok",
            latency: lat,
            at: Date.now(),
          });
          addOutput([{ type: "out", text: `OK  deleted ${key} (${lat}μs)` }]);
          break;
        }
        case "SCAN": {
          const prefix = parts[1] || "";
          const matches = Object.keys(store).filter((k) => k.startsWith(prefix));
          const lat = Math.floor(400 + Math.random() * 600);
          pushLog({
            id: Date.now(),
            type: "SCAN",
            key: prefix || "*",
            status: "ok",
            latency: lat,
            at: Date.now(),
          });
          if (matches.length === 0) {
            addOutput([{ type: "out", text: `(empty)  no keys matching "${prefix}*" (${lat}μs)` }]);
          } else {
            addOutput([
              { type: "out", text: `${matches.length} keys matching "${prefix}*" (${lat}μs):` },
              ...matches.map((k) => ({
                type: "out" as const,
                text: `  ${k}  →  ${
                  store[k].type === "doc"
                    ? "<doc>"
                    : JSON.stringify(store[k].v).slice(0, 40)
                }`,
              })),
            ]);
          }
          break;
        }
        case "COUNT": {
          const count = Object.keys(store).length;
          const lat = Math.floor(50 + Math.random() * 100);
          pushLog({
            id: Date.now(),
            type: "COUNT",
            key: "*",
            status: "ok",
            latency: lat,
            at: Date.now(),
          });
          addOutput([{ type: "out", text: `${count} keys in store (${lat}μs)` }]);
          break;
        }
        case "BEGIN": {
          const lat = Math.floor(30 + Math.random() * 60);
          pushLog({
            id: Date.now(),
            type: "BEGIN",
            key: "tx",
            status: "ok",
            latency: lat,
            at: Date.now(),
          });
          addOutput([{ type: "out", text: `transaction started (${lat}μs)` }]);
          break;
        }
        case "COMMIT": {
          const lat = Math.floor(100 + Math.random() * 200);
          pushLog({
            id: Date.now(),
            type: "COMMIT",
            key: "tx",
            status: "ok",
            latency: lat,
            at: Date.now(),
          });
          addOutput([{ type: "out", text: `transaction committed + WAL fsync'd (${lat}μs)` }]);
          break;
        }
        default: {
          addOutput([{ type: "err", text: `Unknown command: ${verb}. Type HELP for commands.` }]);
        }
      }
      addOutput([{ type: "out", text: "" }]);
    },
    [store, pushLog]
  );

  const onSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    execute(input);
    setInput("");
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

  const avgLatency = opCounter > 0 ? Math.round(totalLatency / opCounter) : 0;
  const filteredKeys = Object.keys(store);

  return (
    <section id="database" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-dots opacity-20 pointer-events-none" />
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="قاعدة البيانات المدمجة"
          eyebrowEn="Built-in KV/Document DB"
          title="NAWA-DB · بدون أي تبعية خارجية"
          titleEn="NAWA-DB — zero external dependencies"
          desc="قاعدة بيانات هجينة تجمع بين بساطة Key-Value ومرونة المستندات (Documents). مبنية بـ mmap وLSM tree، تعمل داخل نفس العملية — لا socket، لا شبكة، لا DBMS منفصل."
          descEn="A hybrid database combining Key-Value simplicity with Document flexibility. Built with mmap + LSM tree, runs in-process — no sockets, no network, no external DBMS."
        />

        {/* Features strip */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-10 grid grid-cols-2 lg:grid-cols-4 gap-3"
        >
          {DB_FEATURES.map((f) => (
            <div
              key={f.title}
              className="p-4 rounded-xl border border-border/60 bg-card/60 flex items-start gap-3"
            >
              <div className="p-1.5 rounded bg-primary/15 text-primary shrink-0">
                <f.icon className="w-4 h-4" strokeWidth={1.5} />
              </div>
              <div>
                <div className="text-sm font-medium">{f.title}</div>
                <div className="text-xs text-muted-foreground mt-0.5 ar">{f.desc}</div>
              </div>
            </div>
          ))}
        </motion.div>

        {/* Live Shell */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-10 grid lg:grid-cols-5 gap-4"
        >
          {/* Shell terminal - left, 3/5 width */}
          <div className="lg:col-span-3 rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
            {/* Terminal header */}
            <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
              <div className="flex items-center gap-2">
                <Terminal className="w-4 h-4 text-primary" />
                <span className="text-sm font-medium mono">nawa-db shell</span>
                <Badge variant="outline" className="mono text-[10px]">
                  in-process
                </Badge>
              </div>
              <div className="flex items-center gap-3 text-xs">
                <div className="flex items-center gap-1.5">
                  <span className="text-muted-foreground">keys:</span>
                  <span className="mono text-primary font-medium">{filteredKeys.length}</span>
                </div>
                <div className="flex items-center gap-1.5">
                  <span className="text-muted-foreground">avg:</span>
                  <span className="mono text-accent font-medium">{avgLatency}μs</span>
                </div>
                <div className="flex items-center gap-1.5">
                  <span className="text-muted-foreground">ops:</span>
                  <span className="mono font-medium">{opCounter}</span>
                </div>
              </div>
            </div>

            {/* Output */}
            <div
              ref={outputRef}
              onClick={() => inputRef.current?.focus()}
              className="h-80 overflow-y-auto scrollbar-narrow p-4 font-mono text-xs leading-relaxed cursor-text"
            >
              {output.map((line, i) => (
                <div
                  key={i}
                  className={`whitespace-pre-wrap break-words ${
                    line.type === "in"
                      ? "text-primary"
                      : line.type === "err"
                      ? "text-destructive"
                      : "text-foreground/80"
                  }`}
                >
                  {line.type === "in" ? (
                    <span className="flex gap-2">
                      <span className="text-accent">nawa&gt;</span>
                      <span>{line.text}</span>
                    </span>
                  ) : (
                    line.text || " "
                  )}
                </div>
              ))}
              {/* Prompt + input */}
              <form onSubmit={onSubmit} className="flex items-center gap-2 mt-1">
                <span className="text-accent shrink-0">nawa&gt;</span>
                <input
                  ref={inputRef}
                  value={input}
                  onChange={(e) => setInput(e.target.value)}
                  onKeyDown={onKeyDown}
                  placeholder={'type a command (try: HELP, PUT user:1 {"name":"test"})'}
                  className="flex-1 bg-transparent border-0 outline-none text-foreground placeholder:text-muted-foreground/50 font-mono text-xs"
                  autoFocus
                  spellCheck={false}
                  autoComplete="off"
                />
                <CornerDownLeft className="w-3 h-3 text-muted-foreground/50 shrink-0" />
              </form>
            </div>

            {/* Quick commands bar */}
            <div className="px-3 py-2 border-t border-border/40 bg-card/40 flex flex-wrap gap-1">
              {["HELP", "SCAN", "COUNT", "GET user:1001", "PUT counter:x 42", "BEGIN"].map(
                (c) => (
                  <button
                    key={c}
                    onClick={() => {
                      setInput(c);
                      inputRef.current?.focus();
                    }}
                    className="px-2 py-0.5 rounded text-[10px] mono bg-card/80 border border-border/40 hover:border-primary/40 hover:text-primary transition-colors"
                  >
                    {c}
                  </button>
                )
              )}
            </div>
          </div>

          {/* Right side: op log + internals */}
          <div className="lg:col-span-2 space-y-4">
            {/* Op log */}
            <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
              <div className="px-4 py-2.5 border-b border-border/40 bg-card/60 flex items-center gap-2">
                <Activity className="w-3.5 h-3.5 text-accent" />
                <span className="text-xs font-medium mono">OP LOG</span>
                <span className="text-[10px] text-muted-foreground mono ml-auto">live</span>
              </div>
              <div className="h-44 overflow-y-auto scrollbar-narrow p-2">
                {log.length === 0 && (
                  <div className="p-6 text-center text-xs text-muted-foreground ar">
                    لا توجد عمليات بعد
                  </div>
                )}
                {log.map((op) => (
                  <motion.div
                    key={op.id}
                    initial={{ opacity: 0, x: -10 }}
                    animate={{ opacity: 1, x: 0 }}
                    className="flex items-center gap-2 px-2 py-1 rounded text-[10px] font-mono hover:bg-card/60"
                  >
                    <span className="text-muted-foreground/60 w-14">
                      {new Date(op.at).toLocaleTimeString("en-US", { hour12: false })}
                    </span>
                    <span
                      className={`w-14 font-bold ${
                        op.type === "PUT"
                          ? "text-primary"
                          : op.type === "GET"
                          ? "text-accent"
                          : op.type === "DEL"
                          ? "text-destructive"
                          : op.type === "ERROR"
                          ? "text-destructive"
                          : "text-yellow-400"
                      }`}
                    >
                      {op.type}
                    </span>
                    <span className="flex-1 truncate text-foreground/90">{op.key}</span>
                    <span
                      className={`px-1 rounded text-[9px] ${
                        op.status === "ok"
                          ? "bg-green-500/15 text-green-400"
                          : op.status === "miss"
                          ? "bg-yellow-500/15 text-yellow-400"
                          : "bg-destructive/15 text-destructive"
                      }`}
                    >
                      {op.status}
                    </span>
                    <span className="text-muted-foreground w-12 text-right">{op.latency}μs</span>
                  </motion.div>
                ))}
              </div>
            </div>

            {/* Keys explorer */}
            <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
              <div className="px-4 py-2.5 border-b border-border/40 bg-card/60 flex items-center gap-2">
                <Database className="w-3.5 h-3.5 text-primary" />
                <span className="text-xs font-medium mono">KEYS ({filteredKeys.length})</span>
              </div>
              <div className="h-44 overflow-y-auto scrollbar-narrow">
                {filteredKeys.map((k) => {
                  const v = store[k];
                  const isDoc = v.type === "doc";
                  return (
                    <div
                      key={k}
                      className="group flex items-center gap-2 px-3 py-1.5 hover:bg-primary/5 border-b border-border/20 last:border-0"
                    >
                      <div
                        className={`p-1 rounded ${
                          isDoc ? "bg-accent/15 text-accent" : "bg-primary/15 text-primary"
                        }`}
                      >
                        {isDoc ? <FileJson className="w-2.5 h-2.5" /> : <KeyRound className="w-2.5 h-2.5" />}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="text-[10px] mono truncate">{k}</div>
                        <div className="text-[9px] text-muted-foreground truncate">
                          {isDoc ? JSON.stringify(v.v).slice(0, 50) : String(v.v)}
                        </div>
                      </div>
                      <button
                        onClick={() => {
                          setInput(`GET ${k}`);
                          inputRef.current?.focus();
                        }}
                        className="opacity-0 group-hover:opacity-100 text-[9px] mono text-primary hover:underline"
                      >
                        GET
                      </button>
                      <button
                        onClick={() => execute(`DEL ${k}`)}
                        className="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-destructive/20 text-destructive transition"
                      >
                        <Trash2 className="w-2.5 h-2.5" />
                      </button>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        </motion.div>

        {/* DB internals */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.5 }}
          className="mt-8"
        >
          <div className="flex items-center gap-2 mb-4">
            <Layers3 className="w-4 h-4 text-primary" />
            <h4 className="text-sm font-semibold ar">طبقات التخزين الداخلية</h4>
            <span className="text-xs text-muted-foreground mono">{`// storage internals`}</span>
          </div>
          <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-3">
            {INTERNALS.map((layer) => (
              <div
                key={layer.name}
                className="p-4 rounded-lg border border-border/40 bg-card/40 hover:border-primary/40 transition-colors"
              >
                <div className="flex items-center gap-2 mb-2">
                  <layer.icon className="w-4 h-4 text-primary" />
                  <span className="text-sm font-medium mono">{layer.name}</span>
                </div>
                <p className="text-xs text-muted-foreground leading-relaxed ar">{layer.desc}</p>
                <div className="mt-2 flex flex-wrap gap-1">
                  {layer.tags.map((t) => (
                    <span
                      key={t}
                      className="px-1.5 py-0.5 rounded text-[10px] mono bg-primary/10 text-primary"
                    >
                      {t}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </motion.div>
      </div>
    </section>
  );
}

const DB_FEATURES = [
  { icon: Layers3, title: "Hybrid KV + Document", desc: "مفتاح-قيمة بسيط أو مستند JSON كامل — نفس المحرك" },
  { icon: Hash, title: "LSM-Tree Storage", desc: "تخزين شجري مُحسّن للكتابة مع SSTables مضغوطة" },
  { icon: Zap, title: "mmap Direct Access", desc: "وصول مباشر للصفحات بدون نسخ user-space" },
  { icon: Lock, title: "ACID Transactions", desc: "معاملات آمنة مع WAL (Write-Ahead Log)" },
];

const INTERNALS = [
  {
    name: "MemTable",
    icon: Layers3,
    desc: "جدول في الذاكرة بـ skip list lock-free — كل الكتابات تذهب هنا أولاً قبل flush.",
    tags: ["lock-free", "skip-list", "in-mem"],
  },
  {
    name: "SSTable (L0-L7)",
    icon: Database,
    desc: "ملفات ثابتة على القرص بترتيب مفاتيح — تُدمج عبر compaction في الخلفية.",
    tags: ["mmap", "bloom-filter", "compressed"],
  },
  {
    name: "WAL (Write-Ahead Log)",
    icon: Lock,
    desc: "كل كتابة تُسجَّل أولاً في WAL قبل الالتزام — يضمن durability بعد الانهيار.",
    tags: ["fsync", "durability", "crash-safe"],
  },
  {
    name: "Bloom Filter",
    icon: Search,
    desc: "فلتر احتمالي لكل SSTable — يخبرك فوراً إن كان المفتاح غير موجود دون قراءة الملف.",
    tags: ["O(1)", "probabilistic", "fp: 1%"],
  },
];
