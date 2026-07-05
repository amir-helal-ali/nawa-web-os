"use client";

import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Cpu,
  Zap,
  Database,
  Network,
  ArrowRight,
  Play,
  Pause,
  RotateCcw,
  Activity,
  HardDrive,
  Server,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

type Op = {
  id: number;
  type: "READ" | "WRITE" | "SEND" | "RECV" | "OPENAT";
  status: "queued" | "in-flight" | "complete";
  fd: number;
  size: number;
  born: number;
};

const OP_LABELS: Record<Op["type"], { color: string; en: string; ar: string }> = {
  READ: { color: "oklch(0.72 0.19 47)", en: "READ", ar: "قراءة" },
  WRITE: { color: "oklch(0.78 0.16 165)", en: "WRITE", ar: "كتابة" },
  SEND: { color: "oklch(0.80 0.15 90)", en: "SEND", ar: "إرسال" },
  RECV: { color: "oklch(0.65 0.20 300)", en: "RECV", ar: "استقبال" },
  OPENAT: { color: "oklch(0.70 0.20 5)", en: "OPENAT", ar: "فتح ملف" },
};

const COMPARISON = [
  {
    label: "نظام تقليدي (epoll + read/write)",
    en: "Traditional",
    steps: ["read() disk → user buffer", "parse → transform", "serialize → another buffer", "write() → kernel buffer", "send() → socket buffer"],
    copies: 4,
    syscalls: 6,
    color: "destructive",
  },
  {
    label: "نواة NAWA (io_uring + mmap)",
    en: "NAWA Zero-Copy",
    steps: ["mmap() → user pointer", "in-place parse", "io_uring send(zero-copy)"],
    copies: 0,
    syscalls: 2,
    color: "primary",
    highlight: true,
  },
];

const FEATURES = [
  {
    icon: Cpu,
    title: "io_uring async I/O",
    desc: "بدل epoll المتسلسل، نستخدم io_uring مع SQ poll — طلبات I/O تُجمع وتُنفذ دفعة واحدة في النواة دون context switches.",
    stat: "5×",
    statLabel: "throughput",
  },
  {
    icon: Database,
    title: "mmap للوصول المباشر",
    desc: "ملفات قاعدة البيانات تُmapped مباشرة إلى الذاكرة — أي وصول للصفحة يحدث دون نسخها إلى user buffer.",
    stat: "0",
    statLabel: "user copies",
  },
  {
    icon: Network,
    title: "sendfile + MSG_ZEROCOPY",
    desc: "الملفات الثابتة (CSS, JS, صور) تُرسل من القرص إلى الـ socket عبر syscall واحد دون المرور بـ user-space.",
    stat: "1",
    statLabel: "syscall",
  },
  {
    icon: Zap,
    title: "lock-free structures",
    desc: "الـ ring buffers وqueues مبنية بـ atomic operations — لا mutexes، لا contention، لا overhead.",
    stat: "<50ns",
    statLabel: "per op",
  },
];

export function ZeroCopyKernel() {
  const [ops, setOps] = useState<Op[]>([]);
  const [running, setRunning] = useState(true);
  const [stats, setStats] = useState({ submitted: 0, completed: 0, inFlight: 0 });
  const [speed, setSpeed] = useState(1);
  const idRef = useRef(0);
  const opsRef = useRef<Op[]>([]);

  // Simulation loop
  useEffect(() => {
    if (!running) return;
    const interval = setInterval(() => {
      // Spawn new ops
      const newOps: Op[] = [];
      const spawn = Math.floor(2 + Math.random() * 4);
      for (let i = 0; i < spawn; i++) {
        const types: Op["type"][] = ["READ", "WRITE", "SEND", "RECV", "OPENAT"];
        newOps.push({
          id: ++idRef.current,
          type: types[Math.floor(Math.random() * types.length)],
          status: "queued",
          fd: 3 + Math.floor(Math.random() * 10),
          size: 4096 * (1 + Math.floor(Math.random() * 16)),
          born: Date.now(),
        });
      }

      // Progress existing ops
      opsRef.current = opsRef.current
        .map((op) => {
          if (op.status === "queued" && Math.random() < 0.4) {
            return { ...op, status: "in-flight" as const };
          }
          if (op.status === "in-flight" && Math.random() < 0.5) {
            return { ...op, status: "complete" as const };
          }
          return op;
        })
        .filter((op) => !(op.status === "complete" && Date.now() - op.born > 1500))
        .concat(newOps)
        .slice(-24);

      setOps([...opsRef.current]);
      setStats({
        submitted: idRef.current,
        completed: opsRef.current.filter((o) => o.status === "complete").length,
        inFlight: opsRef.current.filter((o) => o.status === "in-flight").length,
      });
    }, 600 / speed);

    return () => clearInterval(interval);
  }, [running, speed]);

  const reset = () => {
    opsRef.current = [];
    setOps([]);
    setStats({ submitted: 0, completed: 0, inFlight: 0 });
    idRef.current = 0;
  };

  return (
    <section id="kernel" className="relative py-24 lg:py-32">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="النواة الثورية"
          eyebrowEn="Zero-Copy Kernel"
          title="حيث تُولد الثورة"
          titleEn="Where the revolution begins"
          desc="بدلاً من نسخ البيانات بين طبقات النواة وuser-space وbuffers التطبيق، نواة NAWA تستخدم أحدث تقنيات Linux لإبقاء البيانات في مكانها — تتحرك من القرص إلى الشبكة دون نسخة واحدة."
          descEn="Instead of copying data across kernel/user-space boundaries, NAWA's kernel uses the latest Linux techniques to keep data in place — from disk to network with zero copies."
        />

        {/* Live io_uring Ring Visualizer */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-14"
        >
          <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
            {/* Header */}
            <div className="flex items-center justify-between px-4 py-3 border-b border-border/40 bg-card/60">
              <div className="flex items-center gap-2">
                <Cpu className="w-4 h-4 text-primary" />
                <span className="text-sm font-medium mono">io_uring ring · live simulation</span>
                <Badge variant="outline" className="mono text-[10px] border-primary/40 text-primary">
                  SQ + CQ
                </Badge>
              </div>
              <div className="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setRunning((r) => !r)}
                  className="h-7 px-2 text-xs"
                >
                  {running ? <Pause className="w-3 h-3" /> : <Play className="w-3 h-3" />}
                  <span className="mr-1 ar">{running ? "إيقاف" : "تشغيل"}</span>
                </Button>
                <Button variant="ghost" size="sm" onClick={reset} className="h-7 px-2 text-xs">
                  <RotateCcw className="w-3 h-3" />
                </Button>
                <div className="flex items-center gap-1 ml-2 px-2 py-1 rounded bg-card/60 border border-border/40 text-xs">
                  <span className="text-muted-foreground">speed:</span>
                  {[1, 2, 4].map((s) => (
                    <button
                      key={s}
                      onClick={() => setSpeed(s)}
                      className={`px-1.5 rounded text-[10px] mono ${
                        speed === s ? "bg-primary/20 text-primary" : "text-muted-foreground"
                      }`}
                    >
                      {s}×
                    </button>
                  ))}
                </div>
              </div>
            </div>

            {/* Body */}
            <div className="grid lg:grid-cols-[280px_1fr_280px]">
              {/* SQ (submission queue) */}
              <div className="p-4 border-b lg:border-b-0 lg:border-l border-border/40">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-1.5">
                    <ArrowRight className="w-3.5 h-3.5 text-primary" />
                    <span className="text-xs font-semibold mono">SQ (Submission)</span>
                  </div>
                  <span className="text-[10px] text-muted-foreground mono">
                    {ops.filter((o) => o.status === "queued").length} queued
                  </span>
                </div>
                <div className="space-y-1 min-h-[180px]">
                  <AnimatePresence>
                    {ops
                      .filter((o) => o.status === "queued")
                      .map((op) => (
                        <OpPill key={op.id} op={op} />
                      ))}
                  </AnimatePresence>
                  {ops.filter((o) => o.status === "queued").length === 0 && (
                    <div className="text-center text-[10px] text-muted-foreground py-8 mono">
                      empty
                    </div>
                  )}
                </div>
              </div>

              {/* Kernel processing (center) */}
              <div className="relative p-4 border-b lg:border-b-0 lg:border-l border-border/40 min-h-[240px] grid place-items-center">
                <div className="absolute inset-0 bg-dots opacity-20" />
                <div className="relative text-center">
                  {/* Spinning kernel core */}
                  <div className="relative w-28 h-28 mx-auto mb-3">
                    <motion.div
                      animate={{ rotate: 360 }}
                      transition={{ duration: 8, repeat: Infinity, ease: "linear" }}
                      className="absolute inset-0 rounded-full border-2 border-dashed border-primary/40"
                    />
                    <motion.div
                      animate={{ rotate: -360 }}
                      transition={{ duration: 4, repeat: Infinity, ease: "linear" }}
                      className="absolute inset-3 rounded-full border border-accent/30"
                    />
                    <div className="absolute inset-0 grid place-items-center">
                      <div className="w-16 h-16 rounded-full bg-gradient-to-br from-primary/30 to-accent/20 border border-primary/40 grid place-items-center glow-amber">
                        <Cpu className="w-7 h-7 text-primary" strokeWidth={1.25} />
                      </div>
                    </div>
                  </div>
                  <div className="text-xs font-semibold ar">نواة لينكس</div>
                  <div className="text-[10px] text-muted-foreground mono">Linux Kernel</div>
                  <div className="mt-2 text-[10px] text-primary mono">
                    {stats.inFlight} in-flight
                  </div>
                </div>

                {/* In-flight ops */}
                <div className="absolute top-2 left-2 flex flex-col gap-1">
                  {ops
                    .filter((o) => o.status === "in-flight")
                    .slice(0, 4)
                    .map((op) => (
                      <motion.div
                        key={op.id}
                        initial={{ opacity: 0, scale: 0.5 }}
                        animate={{ opacity: 1, scale: 1, y: [0, -3, 0] }}
                        exit={{ opacity: 0 }}
                        transition={{ duration: 0.3, y: { repeat: Infinity, duration: 0.6 } }}
                        className="px-1.5 py-0.5 rounded text-[9px] mono border bg-card/80"
                        style={{
                          borderColor: OP_LABELS[op.type].color + "60",
                          color: OP_LABELS[op.type].color,
                        }}
                      >
                        {OP_LABELS[op.type].en}
                      </motion.div>
                    ))}
                </div>
              </div>

              {/* CQ (completion queue) */}
              <div className="p-4 lg:border-l border-border/40">
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-1.5">
                    <ArrowRight className="w-3.5 h-3.5 text-accent rotate-180" />
                    <span className="text-xs font-semibold mono">CQ (Completion)</span>
                  </div>
                  <span className="text-[10px] text-muted-foreground mono">
                    {stats.completed} done
                  </span>
                </div>
                <div className="space-y-1 min-h-[180px]">
                  <AnimatePresence>
                    {ops
                      .filter((o) => o.status === "complete")
                      .map((op) => (
                        <OpPill key={op.id} op={op} completed />
                      ))}
                  </AnimatePresence>
                  {ops.filter((o) => o.status === "complete").length === 0 && (
                    <div className="text-center text-[10px] text-muted-foreground py-8 mono">
                      empty
                    </div>
                  )}
                </div>
              </div>
            </div>

            {/* Footer stats */}
            <div className="px-4 py-3 border-t border-border/40 bg-card/60 grid grid-cols-3 gap-4 text-center">
              <div>
                <div className="text-[10px] text-muted-foreground mono">submitted</div>
                <div className="text-lg font-bold text-primary mono">{stats.submitted}</div>
              </div>
              <div>
                <div className="text-[10px] text-muted-foreground mono">in-flight</div>
                <div className="text-lg font-bold text-accent mono">{stats.inFlight}</div>
              </div>
              <div>
                <div className="text-[10px] text-muted-foreground mono">completed</div>
                <div className="text-lg font-bold text-primary mono">{stats.completed}</div>
              </div>
            </div>
          </div>
        </motion.div>

        {/* Comparison */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-8 grid lg:grid-cols-2 gap-5"
        >
          {COMPARISON.map((c) => (
            <div
              key={c.en}
              className={`relative p-6 rounded-2xl border ${
                c.highlight
                  ? "border-primary/50 bg-primary/5 glow-amber"
                  : "border-destructive/30 bg-destructive/5"
              }`}
            >
              {c.highlight && (
                <Badge className="absolute -top-2.5 right-4 bg-primary text-primary-foreground mono text-[10px]">
                  ✦ NAWA
                </Badge>
              )}
              <div className="flex items-start justify-between mb-4">
                <div>
                  <h4 className="text-base font-semibold ar">{c.label}</h4>
                  <p className="text-xs text-muted-foreground mono mt-0.5">{c.en}</p>
                </div>
              </div>
              <div className="space-y-1.5 mb-4">
                {c.steps.map((s, i) => (
                  <div
                    key={s}
                    className="flex items-center gap-2 text-xs font-mono p-2 rounded bg-card/60 border border-border/40"
                  >
                    <span className="text-muted-foreground w-4">{i + 1}.</span>
                    <span className={c.highlight ? "text-primary" : "text-destructive"}>{s}</span>
                  </div>
                ))}
              </div>
              <div className="grid grid-cols-2 gap-2 pt-4 border-t border-border/40">
                <div className="text-center p-2 rounded bg-card/60">
                  <div
                    className={`text-2xl font-bold mono ${
                      c.highlight ? "text-primary" : "text-destructive"
                    }`}
                  >
                    {c.copies}
                  </div>
                  <div className="text-[10px] text-muted-foreground mt-0.5 ar">عمليات نسخ</div>
                </div>
                <div className="text-center p-2 rounded bg-card/60">
                  <div
                    className={`text-2xl font-bold mono ${
                      c.highlight ? "text-primary" : "text-destructive"
                    }`}
                  >
                    {c.syscalls}
                  </div>
                  <div className="text-[10px] text-muted-foreground mt-0.5 ar">syscalls</div>
                </div>
              </div>
            </div>
          ))}
        </motion.div>

        {/* Features grid */}
        <div className="mt-8 grid sm:grid-cols-2 lg:grid-cols-4 gap-3">
          {FEATURES.map((f, i) => (
            <motion.div
              key={f.title}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: i * 0.08 }}
              className="p-5 rounded-xl border border-border/60 bg-card/60 hover:border-primary/40 hover:bg-card transition-all"
            >
              <div className="flex items-center justify-between mb-3">
                <div className="p-2 rounded-lg bg-primary/15 text-primary">
                  <f.icon className="w-5 h-5" strokeWidth={1.5} />
                </div>
                <div className="text-right">
                  <div className="text-xl font-bold text-primary mono">{f.stat}</div>
                  <div className="text-[9px] text-muted-foreground mono">{f.statLabel}</div>
                </div>
              </div>
              <h4 className="text-sm font-semibold mb-1.5">{f.title}</h4>
              <p className="text-xs text-muted-foreground leading-relaxed ar">{f.desc}</p>
            </motion.div>
          ))}
        </div>

        {/* Code block */}
        <KernelCodeBlock />
      </div>
    </section>
  );
}

function OpPill({ op, completed = false }: { op: Op; completed?: boolean }) {
  const label = OP_LABELS[op.type];
  return (
    <motion.div
      layout
      initial={{ opacity: 0, scale: 0.8 }}
      animate={{ opacity: completed ? 0.5 : 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.8 }}
      transition={{ duration: 0.3 }}
      className="flex items-center gap-2 p-1.5 rounded border bg-card/60 text-[10px] mono"
      style={{ borderColor: label.color + "30" }}
    >
      <span
        className="px-1.5 py-0.5 rounded text-[9px] font-bold"
        style={{ background: label.color + "20", color: label.color }}
      >
        {label.en}
      </span>
      <span className="text-muted-foreground">fd:{op.fd}</span>
      <span className="text-muted-foreground/60">{(op.size / 1024).toFixed(0)}KB</span>
      {completed && <span className="text-accent mr-auto">✓</span>}
    </motion.div>
  );
}

function KernelCodeBlock() {
  const code = `// nawa-kernel/src/io_uring.rs
use io_uring::{IoUring, SubmissionQueue};
use std::os::unix::io::RawFd;

/// Zero-copy request pipeline: disk → socket without any buffer copy.
pub struct ZeroCopyPipeline {
    ring: IoUring,
    page_size: usize,
}

impl ZeroCopyPipeline {
    pub fn new(entries: u32) -> std::io::Result<Self> {
        Ok(Self {
            ring: IoUring::builder()
                .setup_sqpoll(1000)        // kernel-side polling
                .build(entries)?,
            page_size: page_size::get(),
        })
    }

    /// Serve a file straight to a socket — zero user-space copies.
    pub fn sendfile_zero_copy(
        &mut self,
        file_fd: RawFd,
        sock_fd: RawFd,
        offset: u64,
        len: usize,
    ) -> std::io::Result<()> {
        // 1) mmap the file region directly into our address space
        let mapped = unsafe {
            mmap(
                std::ptr::null_mut(),
                len,
                ProtFlags::PROT_READ,
                MapFlags::MAP_PRIVATE | MapFlags::MAP_POPULATE,
                Some(file_fd),
                offset as i64,
            )?
        };

        // 2) Submit a single fixed-buffer send via io_uring
        let buf = unsafe { std::slice::from_raw_parts(mapped as *const u8, len) };
        let entry = opcode::Send::new(types::Fd(sock_fd), buf.as_ptr(), len as _)
            .build()
            .user_data(0xDEAD_BEEF);

        unsafe { self.ring.submission().push(&entry)?; }
        self.ring.submit_and_wait(1)?;
        Ok(())
    }
}`;

  return (
    <motion.div
      initial={{ opacity: 0, y: 30 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.6 }}
      className="mt-14"
    >
      <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
        <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
          <div className="flex items-center gap-2">
            <div className="flex gap-1.5">
              <span className="w-3 h-3 rounded-full bg-destructive/60" />
              <span className="w-3 h-3 rounded-full bg-yellow-500/60" />
              <span className="w-3 h-3 rounded-full bg-green-500/60" />
            </div>
            <span className="text-xs text-muted-foreground mono ml-2">
              nawa-kernel/src/io_uring.rs
            </span>
          </div>
          <Badge variant="outline" className="mono text-[10px] border-primary/40 text-primary">
            Rust · #![no_std]-ready
          </Badge>
        </div>
        <div className="px-4 py-2 border-b border-border/40 bg-primary/5 text-xs">
          <span className="text-foreground/80 ar">
            مقتطف من النواة — خط الأنابيب zero-copy من القرص إلى الشبكة
          </span>
          <span className="text-muted-foreground mono ml-2">
            — Kernel excerpt: zero-copy pipeline disk → network
          </span>
        </div>
        <pre className="p-4 overflow-x-auto scrollbar-narrow text-xs leading-relaxed mono">
          <code className="text-foreground/90">{highlightRust(code)}</code>
        </pre>
      </div>
    </motion.div>
  );
}

function highlightRust(code: string): React.ReactNode {
  const keywords = new Set([
    "use", "pub", "struct", "impl", "fn", "let", "mut", "self", "Self",
    "Ok", "Result", "Some", "None", "unsafe", "as", "const", "static", "ref",
    "match", "if", "else", "for", "while", "loop", "return", "where", "trait",
  ]);
  const types = new Set(["IoUring", "SubmissionQueue", "RawFd", "i32", "u64", "u32", "usize"]);

  return code.split("\n").map((line, i) => (
    <div key={i} className="hover:bg-primary/5 -mx-4 px-4">
      <span className="text-muted-foreground/40 select-none mr-4 inline-block w-8 text-right">
        {i + 1}
      </span>
      <span>{tokenize(line, keywords, types)}</span>
    </div>
  ));
}

function tokenize(line: string, keywords: Set<string>, types: Set<string>): React.ReactNode {
  if (line.trim().startsWith("//")) {
    return <span className="text-green-600/70 italic">{line}</span>;
  }
  const parts = line.split(/(\s+|[(){}\[\];,.<>:&|?=!])/);
  return parts.map((p, i) => {
    if (!p) return null;
    if (/^\s+$/.test(p)) return p;
    if (keywords.has(p)) return <span key={i} className="text-primary">{p}</span>;
    if (types.has(p)) return <span key={i} className="text-accent">{p}</span>;
    if (/^[A-Z][A-Za-z0-9_]*$/.test(p)) return <span key={i} className="text-accent">{p}</span>;
    if (/^\d+$/.test(p)) return <span key={i} className="text-orange-300">{p}</span>;
    if (p.startsWith('"') || p.startsWith("'")) return <span key={i} className="text-green-400">{p}</span>;
    return <span key={i} className="text-foreground/80">{p}</span>;
  });
}
