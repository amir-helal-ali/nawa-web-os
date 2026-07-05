"use client";

import { Hexagon, Github, Twitter, BookOpen, Mail, Heart } from "lucide-react";
import { Button } from "@/components/ui/button";

const FOOTER_LINKS = [
  {
    title: "المشروع",
    titleEn: "Project",
    links: [
      { id: "concept", label: "الفكرة" },
      { id: "architecture", label: "المعمارية" },
      { id: "kernel", label: "النواة" },
      { id: "roadmap", label: "خارطة الطريق" },
      { id: "faq", label: "الأسئلة الشائعة" },
    ],
  },
  {
    title: "تقني",
    titleEn: "Technical",
    links: [
      { id: "database", label: "قاعدة البيانات" },
      { id: "playground", label: "ملعب الكود" },
      { id: "performance", label: "الأداء" },
      { id: "comparison", label: "المقارنة" },
      { id: "security", label: "الأمان" },
    ],
  },
  {
    title: "أدوات",
    titleEn: "Tools",
    links: [
      { id: "cli", label: "CLI" },
      { id: "marketplace", label: "الإضافات" },
      { id: "builder", label: "بناء التطبيقات" },
      { id: "observability", label: "الرصد" },
      { id: "dx", label: "تجربة المطور" },
    ],
  },
  {
    title: "مجتمع",
    titleEn: "Community",
    links: [
      { id: "ecosystem", label: "المنظومة" },
      { id: "docker", label: "Docker" },
      { id: "flow", label: "رحلة الطلب" },
    ],
  },
];

export function Footer() {
  return (
    <footer className="relative mt-auto border-t border-border/60 bg-card/40">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="grid lg:grid-cols-6 gap-8">
          {/* Brand */}
          <div className="lg:col-span-2">
            <div className="flex items-center gap-2.5 mb-3">
              <div className="relative">
                <Hexagon className="w-8 h-8 text-primary" strokeWidth={1.5} />
                <div className="absolute inset-0 grid place-items-center">
                  <span className="font-bold text-[10px] text-primary-foreground bg-primary rounded-sm w-4 h-4 grid place-items-center ar">
                    ن
                  </span>
                </div>
              </div>
              <div>
                <div className="font-bold text-base leading-none">
                  NAWA<span className="text-primary">.</span>
                </div>
                <div className="text-[10px] text-muted-foreground leading-none mt-0.5 ar">
                  نواة · Web OS
                </div>
              </div>
            </div>
            <p className="text-sm text-muted-foreground max-w-md leading-relaxed ar">
              نظام تشغيل ويب ثوري مكتوب بـ Rust — محركان، قاعدة بيانات مدمجة، ويعمل على
              أضعف السيرفرات. مفتوح المصدر وقابل للبناء عليه لأي مشروع ويب.
            </p>
            <p className="text-xs text-muted-foreground/70 mt-2">
              A revolutionary web operating system built in Rust. Open-source and project-ready.
            </p>
          </div>

          {/* Link columns */}
          {FOOTER_LINKS.map((col) => (
            <div key={col.titleEn}>
              <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-3 ar">
                {col.title}
              </div>
              <ul className="space-y-2 text-sm">
                {col.links.map((l) => (
                  <li key={l.id}>
                    <a
                      href={`#${l.id}`}
                      className="text-foreground/80 hover:text-primary ar transition-colors"
                    >
                      {l.label}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* Bottom bar */}
        <div className="mt-10 pt-6 border-t border-border/40 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div className="text-xs text-muted-foreground">
            <span className="ar">© 2026 NAWA Project · مفتوح المصدر تحت MIT + Apache 2.0</span>
            <span className="mx-2">·</span>
            <span className="mono">v0.1.0-alpha</span>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="icon" className="h-8 w-8">
              <Github className="w-4 h-4" />
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-8">
              <Twitter className="w-4 h-4" />
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-8">
              <BookOpen className="w-4 h-4" />
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-8">
              <Mail className="w-4 h-4" />
            </Button>
          </div>
        </div>

        {/* Tagline strip */}
        <div className="mt-8 text-center">
          <p className="text-xs text-muted-foreground/50 mono flex items-center justify-center gap-2">
            <span className="text-primary">∞</span>
            <span>built with Rust, io_uring &amp; zero copies</span>
            <span className="text-accent">·</span>
            <span className="ar">صُمم للسيرفرات الضعيفة</span>
            <Heart className="w-3 h-3 text-destructive fill-current" />
          </p>
        </div>
      </div>
    </footer>
  );
}
