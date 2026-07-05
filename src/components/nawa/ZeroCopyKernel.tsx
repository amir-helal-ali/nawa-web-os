"use client";

import { motion } from "framer-motion";
import {
  Cpu,
  Zap,
  Database,
  Network,
  ArrowRight,
  Copy,
  Check,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

const COMPARISON = [
  {
    label: "نظام تقليدي (Traditional)",
    en: "Traditional",
    steps: ["read() disk → user buffer", "parse → transform", "serialize → another buffer", "write() → kernel buffer", "send() → socket buffer"],
    copies: 4,
    syscalls: 6,
    color: "destructive",
  },
  {
    label: "نواة NAWA (Zero-Copy)",
    en: "NAWA Zero-Copy",
    steps: ["mmap() → user pointer", "in-place parse", "io_uring send(zero-copy)"],
    copies: 0,
    syscalls: 2,
    color: "primary",
    highlight: true,
  },
];

const RUST_CODE = `// nawa-kernel/src/io_uring.rs
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

        {/* Comparison */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-14 grid lg:grid-cols-2 gap-5"
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

              {/* Steps */}
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

              {/* Stats */}
              <div className="grid grid-cols-2 gap-2 pt-4 border-t border-border/40">
                <div className="text-center p-2 rounded bg-card/60">
                  <div className={`text-2xl font-bold mono ${c.highlight ? "text-primary" : "text-destructive"}`}>
                    {c.copies}
                  </div>
                  <div className="text-[10px] text-muted-foreground mt-0.5 ar">عمليات نسخ</div>
                </div>
                <div className="text-center p-2 rounded bg-card/60">
                  <div className={`text-2xl font-bold mono ${c.highlight ? "text-primary" : "text-destructive"}`}>
                    {c.syscalls}
                  </div>
                  <div className="text-[10px] text-muted-foreground mt-0.5 ar">syscalls</div>
                </div>
              </div>
            </div>
          ))}
        </motion.div>

        {/* Features grid */}
        <div className="mt-12 grid sm:grid-cols-2 lg:grid-cols-4 gap-3">
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
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-14"
        >
          <CodeBlock
            filename="nawa-kernel/src/io_uring.rs"
            description="مقتطف من النواة — خط الأنابيب zero-copy من القرص إلى الشبكة"
            descriptionEn="Kernel excerpt — zero-copy pipeline from disk to network"
            code={RUST_CODE}
          />
        </motion.div>
      </div>
    </section>
  );
}

function CodeBlock({
  filename,
  description,
  descriptionEn,
  code,
}: {
  filename: string;
  description: string;
  descriptionEn: string;
  code: string;
}) {
  return (
    <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
        <div className="flex items-center gap-2">
          <div className="flex gap-1.5">
            <span className="w-3 h-3 rounded-full bg-destructive/60" />
            <span className="w-3 h-3 rounded-full bg-yellow-500/60" />
            <span className="w-3 h-3 rounded-full bg-green-500/60" />
          </div>
          <span className="text-xs text-muted-foreground mono ml-2">{filename}</span>
        </div>
        <Badge variant="outline" className="mono text-[10px] border-primary/40 text-primary">
          Rust · #![no_std]-ready
        </Badge>
      </div>
      {/* Description bar */}
      <div className="px-4 py-2 border-b border-border/40 bg-primary/5 text-xs">
        <span className="text-foreground/80 ar">{description}</span>
        <span className="text-muted-foreground mono ml-2">— {descriptionEn}</span>
      </div>
      {/* Code */}
      <pre className="p-4 overflow-x-auto scrollbar-narrow text-xs leading-relaxed mono">
        <code className="text-foreground/90">{highlightRust(code)}</code>
      </pre>
    </div>
  );
}

function highlightRust(code: string): React.ReactNode {
  // Lightweight token highlighter (visual only — not a real parser)
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
      <span>
        {tokenize(line, keywords, types)}
      </span>
    </div>
  ));
}

function tokenize(line: string, keywords: Set<string>, types: Set<string>): React.ReactNode {
  if (line.trim().startsWith("//")) {
    return <span className="text-green-600/70 italic">{line}</span>;
  }
  // Split keeping whitespace and tokens
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
