"use client";

import { useEffect, useRef, useState, useCallback } from "react";
import { motion } from "framer-motion";
import {
  Database,
  KeyRound,
  FileJson,
  Play,
  Trash2,
  Plus,
  Search,
  Activity,
  Hash,
  Layers3,
  Zap,
  Lock,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

type OpLog = {
  id: number;
  type: "PUT" | "GET" | "DEL" | "SCAN";
  key: string;
  status: "ok" | "miss" | "err";
  latency: number; // microseconds
  at: number;
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

export function DatabaseDemo() {
  const [store, setStore] = useState<Record<string, DocValue>>(SEED_DATA);
  const [log, setLog] = useState<OpLog[]>([]);
  const [keyInput, setKeyInput] = useState("");
  const [valueInput, setValueInput] = useState("");
  const [mode, setMode] = useState<"kv" | "doc">("kv");
  const [searchKey, setSearchKey] = useState("");
  const [opCounter, setOpCounter] = useState(0);
  const [totalLatency, setTotalLatency] = useState(0);
  const logRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    logRef.current?.scrollTo({ top: 0, behavior: "smooth" });
  }, [log]);

  const pushLog = useCallback((op: OpLog) => {
    setLog((l) => [op, ...l].slice(0, 50));
    setOpCounter((c) => c + 1);
    setTotalLatency((t) => t + op.latency);
  }, []);

  const simulateLatency = () => Math.floor(80 + Math.random() * 280); // microseconds

  const handlePut = () => {
    if (!keyInput.trim()) return;
    let parsedValue: any = valueInput;
    if (mode === "doc") {
      try {
        parsedValue = JSON.parse(valueInput || "{}");
      } catch {
        pushLog({
          id: Date.now(),
          type: "PUT",
          key: keyInput,
          status: "err",
          latency: simulateLatency(),
          at: Date.now(),
        });
        return;
      }
    } else {
      // try numeric
      if (/^-?\d+$/.test(valueInput)) parsedValue = parseInt(valueInput);
      else if (/^-?\d*\.\d+$/.test(valueInput)) parsedValue = parseFloat(valueInput);
    }
    setStore((s) => ({ ...s, [keyInput]: { type: mode, v: parsedValue } }));
    pushLog({
      id: Date.now(),
      type: "PUT",
      key: keyInput,
      status: "ok",
      latency: simulateLatency(),
      at: Date.now(),
    });
    setKeyInput("");
    setValueInput("");
  };

  const handleGet = (k: string) => {
    const found = !!store[k];
    pushLog({
      id: Date.now(),
      type: "GET",
      key: k,
      status: found ? "ok" : "miss",
      latency: simulateLatency(),
      at: Date.now(),
    });
  };

  const handleDel = (k: string) => {
    setStore((s) => {
      const next = { ...s };
      delete next[k];
      return next;
    });
    pushLog({
      id: Date.now(),
      type: "DEL",
      key: k,
      status: "ok",
      latency: simulateLatency(),
      at: Date.now(),
    });
  };

  const handleScan = () => {
    pushLog({
      id: Date.now(),
      type: "SCAN",
      key: "*",
      status: "ok",
      latency: Math.floor(400 + Math.random() * 600),
      at: Date.now(),
    });
  };

  const filteredKeys = Object.keys(store).filter((k) =>
    k.toLowerCase().includes(searchKey.toLowerCase())
  );

  const avgLatency = opCounter > 0 ? Math.round(totalLatency / opCounter) : 0;

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

        {/* Live demo */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-10"
        >
          <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
            {/* Terminal-style header */}
            <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
              <div className="flex items-center gap-2">
                <Database className="w-4 h-4 text-primary" />
                <span className="text-sm font-medium mono">nawa-db</span>
                <Badge variant="outline" className="mono text-[10px]">
                  in-process
                </Badge>
              </div>
              <div className="flex items-center gap-3 text-xs">
                <div className="flex items-center gap-1.5">
                  <span className="text-muted-foreground">keys:</span>
                  <span className="mono text-primary font-medium">{Object.keys(store).length}</span>
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

            <Tabs defaultValue="explorer" className="w-full">
              <div className="px-4 pt-3 border-b border-border/40">
                <TabsList className="bg-transparent h-auto p-0">
                  <TabsTrigger
                    value="explorer"
                    className="data-[state=active]:bg-primary/15 data-[state=active]:text-primary rounded-md px-3 py-1.5 text-xs mono"
                  >
                    EXPLORER
                  </TabsTrigger>
                  <TabsTrigger
                    value="ops"
                    className="data-[state=active]:bg-primary/15 data-[state=active]:text-primary rounded-md px-3 py-1.5 text-xs mono"
                  >
                    OP LOG
                  </TabsTrigger>
                  <TabsTrigger
                    value="schema"
                    className="data-[state=active]:bg-primary/15 data-[state=active]:text-primary rounded-md px-3 py-1.5 text-xs mono"
                  >
                    INTERNALS
                  </TabsTrigger>
                </TabsList>
              </div>

              {/* Explorer */}
              <TabsContent value="explorer" className="m-0 p-4">
                <div className="grid lg:grid-cols-2 gap-4">
                  {/* Left: write panel */}
                  <div className="space-y-3">
                    <div className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                      Write
                    </div>
                    <div className="flex gap-1.5 p-1 rounded-lg bg-card/60 border border-border/40">
                      <button
                        onClick={() => setMode("kv")}
                        className={`flex-1 px-3 py-1.5 rounded text-xs font-medium transition-colors flex items-center justify-center gap-1.5 ${
                          mode === "kv"
                            ? "bg-primary/20 text-primary"
                            : "text-muted-foreground hover:text-foreground"
                        }`}
                      >
                        <KeyRound className="w-3 h-3" /> KV
                      </button>
                      <button
                        onClick={() => setMode("doc")}
                        className={`flex-1 px-3 py-1.5 rounded text-xs font-medium transition-colors flex items-center justify-center gap-1.5 ${
                          mode === "doc"
                            ? "bg-primary/20 text-primary"
                            : "text-muted-foreground hover:text-foreground"
                        }`}
                      >
                        <FileJson className="w-3 h-3" /> Document
                      </button>
                    </div>
                    <Input
                      placeholder="key — e.g. user:1003"
                      value={keyInput}
                      onChange={(e) => setKeyInput(e.target.value)}
                      className="mono bg-card/60 border-border/60 text-sm"
                    />
                    <textarea
                      placeholder={
                        mode === "kv"
                          ? "value — string or number"
                          : '{ "name": "new user", "roles": [] }'
                      }
                      value={valueInput}
                      onChange={(e) => setValueInput(e.target.value)}
                      rows={mode === "doc" ? 5 : 2}
                      className="w-full mono bg-card/60 border border-border/60 rounded-md px-3 py-2 text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary/40"
                    />
                    <div className="flex gap-2">
                      <Button
                        size="sm"
                        className="bg-primary hover:bg-primary/90 text-primary-foreground flex-1"
                        onClick={handlePut}
                      >
                        <Plus className="w-3.5 h-3.5 ml-1" />
                        <span className="ar">نفّذ PUT</span>
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={handleScan}
                        className="border-primary/40 hover:bg-primary/10"
                      >
                        <Activity className="w-3.5 h-3.5 ml-1" />
                        SCAN *
                      </Button>
                    </div>
                  </div>

                  {/* Right: stored keys list */}
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <div className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                        Keys ({filteredKeys.length})
                      </div>
                      <div className="relative w-32">
                        <Search className="w-3 h-3 absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground" />
                        <Input
                          placeholder="filter..."
                          value={searchKey}
                          onChange={(e) => setSearchKey(e.target.value)}
                          className="h-7 text-xs pl-2 pr-7 mono bg-card/60 border-border/60"
                        />
                      </div>
                    </div>
                    <div className="max-h-72 overflow-y-auto scrollbar-narrow space-y-1 rounded-lg border border-border/40 bg-card/40">
                      {filteredKeys.length === 0 && (
                        <div className="p-4 text-center text-xs text-muted-foreground ar">
                          لا توجد مفاتيح مطابقة
                        </div>
                      )}
                      {filteredKeys.map((k) => {
                        const v = store[k];
                        const isDoc = v.type === "doc";
                        return (
                          <div
                            key={k}
                            className="group flex items-center gap-2 px-3 py-2 hover:bg-primary/5 border-b border-border/20 last:border-0"
                          >
                            <div
                              className={`p-1 rounded ${
                                isDoc ? "bg-accent/15 text-accent" : "bg-primary/15 text-primary"
                              }`}
                            >
                              {isDoc ? <FileJson className="w-3 h-3" /> : <KeyRound className="w-3 h-3" />}
                            </div>
                            <div className="flex-1 min-w-0">
                              <div className="text-xs mono truncate">{k}</div>
                              <div className="text-[10px] text-muted-foreground truncate">
                                {isDoc ? JSON.stringify(v.v).slice(0, 60) : String(v.v)}
                              </div>
                            </div>
                            <button
                              onClick={() => handleGet(k)}
                              className="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-primary/20 text-primary transition"
                              title="GET"
                            >
                              <Play className="w-3 h-3" />
                            </button>
                            <button
                              onClick={() => handleDel(k)}
                              className="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-destructive/20 text-destructive transition"
                              title="DEL"
                            >
                              <Trash2 className="w-3 h-3" />
                            </button>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                </div>
              </TabsContent>

              {/* Op log */}
              <TabsContent value="ops" className="m-0 p-4">
                <div ref={logRef} className="max-h-96 overflow-y-auto scrollbar-narrow space-y-1">
                  {log.length === 0 && (
                    <div className="p-8 text-center text-sm text-muted-foreground ar">
                      لا توجد عمليات بعد — جرّب PUT أو GET
                    </div>
                  )}
                  {log.map((op) => (
                    <motion.div
                      key={op.id}
                      initial={{ opacity: 0, x: -10 }}
                      animate={{ opacity: 1, x: 0 }}
                      className="flex items-center gap-3 px-3 py-1.5 rounded text-xs font-mono hover:bg-card/60"
                    >
                      <span className="text-muted-foreground/60 w-16">
                        {new Date(op.at).toLocaleTimeString("en-US", { hour12: false })}
                      </span>
                      <span
                        className={`w-12 font-bold ${
                          op.type === "PUT"
                            ? "text-primary"
                            : op.type === "GET"
                            ? "text-accent"
                            : op.type === "DEL"
                            ? "text-destructive"
                            : "text-yellow-400"
                        }`}
                      >
                        {op.type}
                      </span>
                      <span className="flex-1 truncate text-foreground/90">{op.key}</span>
                      <span
                        className={`px-1.5 py-0.5 rounded text-[10px] ${
                          op.status === "ok"
                            ? "bg-green-500/15 text-green-400"
                            : op.status === "miss"
                            ? "bg-yellow-500/15 text-yellow-400"
                            : "bg-destructive/15 text-destructive"
                        }`}
                      >
                        {op.status}
                      </span>
                      <span className="text-muted-foreground w-16 text-right">{op.latency}μs</span>
                    </motion.div>
                  ))}
                </div>
              </TabsContent>

              {/* Internals */}
              <TabsContent value="schema" className="m-0 p-4">
                <div className="grid sm:grid-cols-2 gap-3">
                  {INTERNALS.map((layer) => (
                    <div
                      key={layer.name}
                      className="p-4 rounded-lg border border-border/40 bg-card/40"
                    >
                      <div className="flex items-center gap-2 mb-2">
                        <layer.icon className="w-4 h-4 text-primary" />
                        <span className="text-sm font-medium mono">{layer.name}</span>
                      </div>
                      <p className="text-xs text-muted-foreground leading-relaxed ar">
                        {layer.desc}
                      </p>
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
              </TabsContent>
            </Tabs>
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
    tags: ["O(1)", "probabilistic", "false-positive: 1%"],
  },
];
