"use client";

import { motion } from "framer-motion";
import {
  Container,
  Ship,
  Layers,
  Lock,
  Rocket,
  Copy,
  Check,
  Server,
  Network,
} from "lucide-react";
import { SectionHeader } from "./Concept";
import { Badge } from "@/components/ui/badge";

const DOCKERFILE = `# syntax=docker/dockerfile:1.7
# ====== Builder stage ======
FROM rust:1.83-alpine AS builder

WORKDIR /nawa
# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \\
    cargo build --release && rm -rf src target/release/deps/nawa*

# Build actual binary
COPY . .
RUN cargo build --release --bin nawad

# ====== Runtime stage (minimal) ======
FROM alpine:3.20

# musl + ca-certs only — no shell, no package manager needed
RUN apk add --no-cache ca-certificates tzdata && \\
    adduser -D -u 10001 nawa

COPY --from=builder /nawa/target/release/nawad /usr/local/bin/nawad
COPY config.toml /etc/nawa/config.toml

USER nawa
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=2s \\
  CMD ["/usr/local/bin/nawad", "healthcheck"]

ENTRYPOINT ["/usr/local/bin/nawad"]
CMD ["serve", "--config", "/etc/nawa/config.toml"]`;

const COMPOSE = `# docker-compose.yml — single-command deploy
services:
  nawa:
    image: nawa/os:0.1.0
    container_name: nawa-prod
    restart: unless-stopped
    ports:
      - "80:8080"
      - "443:8443/udp"   # HTTP/3 QUIC
    volumes:
      - nawa-data:/var/lib/nawa
      - ./certs:/etc/nawa/certs:ro
      - ./apps:/var/lib/nawa/apps:ro
    deploy:
      resources:
        limits:
          cpus: "1.0"
          memory: 512M      # works on the smallest VPS
        reservations:
          memory: 64M
    environment:
      NAWA_WORKERS: "4"      # auto-tuned if unset
      NAWA_DB_CACHE_MB: "32"
      NAWA_LOG_LEVEL: "info"

volumes:
  nawa-data:`;

const CONTAINER_LAYERS = [
  {
    icon: Container,
    title: "Single Binary Container",
    en: "One static binary",
    desc: "ثنائي static musl واحد — لا libc ديناميكي، لا Python، لا Node.js. الصورة النهائية أقل من 15MB.",
  },
  {
    icon: Lock,
    title: "Non-Root by Default",
    en: "runs as UID 10001",
    desc: "الحاوية تعمل بـ user غير جذري بـ UID ثابت — حتى لو اخترق المهاجم الحاوية، لا صلاحيات root.",
  },
  {
    icon: Layers,
    title: "Multi-Stage Build",
    en: "builder + runtime",
    desc: "مرحلة بناء منفصلة بـ rust toolchain كامل، ومرحلة runtime بـ Alpine عاري — حجم نهائي صغير.",
  },
  {
    icon: Network,
    title: "HTTP/3 + Auto TLS",
    en: "QUIC out of the box",
    desc: "المنفذ 443/udp مُفعّل افتراضياً مع TLS تلقائي عبر Let's Encrypt — لا حاجة لإعداد Nginx أو Caddy.",
  },
];

const DEPLOY_STEPS = [
  { cmd: "docker pull nawa/os:0.1.0", desc: "اسحب الصورة" },
  { cmd: "docker compose up -d", desc: "شغّل الحاوية" },
  { cmd: "curl https://your-vps/", desc: "تطبيقك يعمل" },
];

export function DockerDeployment() {
  return (
    <section id="docker" className="relative py-24 lg:py-32 bg-card/30">
      <div className="absolute inset-0 bg-grid opacity-30 pointer-events-none" />
      <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <SectionHeader
          eyebrow="النشر عبر Docker"
          eyebrowEn="Docker Deployment"
          title="حاوية واحدة، أمر واحد"
          titleEn="One container, one command"
          desc="NAWA مصمم ليكون Docker-first. صورة واحدة متعددة المراحل، حجمها أقل من 15MB، تعمل بـ non-root user، وتشمل كل ما يحتاجه تطبيق الويب — من HTTP/3 إلى قاعدة البيانات."
          descEn="NAWA is Docker-first. A single multi-stage image under 15MB, runs as non-root, and includes everything a web app needs — from HTTP/3 to the database."
        />

        {/* Container layer cards */}
        <div className="mt-12 grid sm:grid-cols-2 lg:grid-cols-4 gap-3">
          {CONTAINER_LAYERS.map((l, i) => (
            <motion.div
              key={l.en}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.4, delay: i * 0.08 }}
              className="p-5 rounded-xl border border-border/60 bg-card/60"
            >
              <div className="p-2 rounded-lg bg-primary/15 text-primary w-fit mb-3">
                <l.icon className="w-5 h-5" strokeWidth={1.5} />
              </div>
              <h4 className="text-sm font-semibold ar">{l.title}</h4>
              <p className="text-[10px] text-muted-foreground mono mt-0.5">{l.en}</p>
              <p className="text-xs text-muted-foreground mt-2 leading-relaxed ar">{l.desc}</p>
            </motion.div>
          ))}
        </div>

        {/* Code blocks */}
        <div className="mt-10 grid lg:grid-cols-2 gap-5">
          <CodeCard
            filename="Dockerfile"
            description="صورة متعددة المراحل — builder + runtime"
            descriptionEn="Multi-stage image — builder + runtime"
            code={DOCKERFILE}
            badge="14.8 MB"
          />
          <CodeCard
            filename="docker-compose.yml"
            description="نشر بأمر واحد على أي VPS"
            descriptionEn="One-command deploy on any VPS"
            code={COMPOSE}
            badge="single command"
          />
        </div>

        {/* Deploy steps */}
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
          className="mt-10 relative"
        >
          <div className="rounded-2xl border border-primary/30 bg-gradient-to-br from-primary/10 via-card to-accent/5 p-6 lg:p-8 overflow-hidden relative">
            <div className="absolute inset-0 bg-dots opacity-20 pointer-events-none" />
            <div className="relative flex flex-col lg:flex-row items-start lg:items-center gap-6 justify-between">
              <div>
                <div className="flex items-center gap-2 mb-2">
                  <Rocket className="w-5 h-5 text-primary" />
                  <h3 className="text-lg font-semibold ar">انطلق في 3 خطوات</h3>
                </div>
                <p className="text-sm text-muted-foreground ar">
                  لا حاجة لإعداد Nginx، لا PostgreSQL، لا Redis — كل شيء داخل الحاوية.
                </p>
              </div>
              <div className="flex flex-col sm:flex-row gap-2 w-full lg:w-auto">
                {DEPLOY_STEPS.map((s, i) => (
                  <div
                    key={s.cmd}
                    className="flex items-center gap-2 px-3 py-2 rounded-lg bg-card/80 border border-border/60 backdrop-blur"
                  >
                    <span className="text-xs mono text-primary shrink-0">{i + 1}.</span>
                    <div>
                      <code className="text-xs mono text-foreground/90">$ {s.cmd}</code>
                      <div className="text-[10px] text-muted-foreground ar">{s.desc}</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </motion.div>
      </div>
    </section>
  );
}

function CodeCard({
  filename,
  description,
  descriptionEn,
  code,
  badge,
}: {
  filename: string;
  description: string;
  descriptionEn: string;
  code: string;
  badge: string;
}) {
  return (
    <div className="rounded-2xl border border-border/60 overflow-hidden bg-[#0d0c0a]">
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border/40 bg-card/60">
        <div className="flex items-center gap-2">
          <Container className="w-4 h-4 text-primary" />
          <span className="text-xs text-muted-foreground mono">{filename}</span>
        </div>
        <Badge variant="outline" className="mono text-[10px] border-primary/40 text-primary">
          {badge}
        </Badge>
      </div>
      <div className="px-4 py-2 border-b border-border/40 bg-primary/5 text-xs">
        <span className="text-foreground/80 ar">{description}</span>
        <span className="text-muted-foreground mono ml-2">— {descriptionEn}</span>
      </div>
      <pre className="p-4 overflow-x-auto scrollbar-narrow text-xs leading-relaxed mono max-h-80">
        <code className="text-foreground/90">{code.split("\n").map((line, i) => (
          <div key={i} className="hover:bg-primary/5 -mx-4 px-4">
            <span className="text-muted-foreground/40 select-none mr-4 inline-block w-6 text-right">
              {i + 1}
            </span>
            <span className={line.trim().startsWith("#") ? "text-green-600/70 italic" : ""}>{line || " "}</span>
          </div>
        ))}</code>
      </pre>
    </div>
  );
}
