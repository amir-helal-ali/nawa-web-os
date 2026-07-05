"use client";

import { Hexagon, Github, Twitter, BookOpen, Mail } from "lucide-react";
import { Button } from "@/components/ui/button";

export function Footer() {
  return (
    <footer className="relative mt-auto border-t border-border/60 bg-card/40">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
        <div className="grid lg:grid-cols-4 gap-8">
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

          {/* Links columns */}
          <div>
            <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-3 ar">
              المشروع
            </div>
            <ul className="space-y-2 text-sm">
              <li><a href="#concept" className="text-foreground/80 hover:text-primary ar">الفكرة</a></li>
              <li><a href="#architecture" className="text-foreground/80 hover:text-primary ar">المعمارية</a></li>
              <li><a href="#kernel" className="text-foreground/80 hover:text-primary ar">النواة</a></li>
              <li><a href="#roadmap" className="text-foreground/80 hover:text-primary ar">خارطة الطريق</a></li>
            </ul>
          </div>

          <div>
            <div className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-3 ar">
              موارد
            </div>
            <ul className="space-y-2 text-sm">
              <li><a href="#database" className="text-foreground/80 hover:text-primary ar">قاعدة البيانات</a></li>
              <li><a href="#performance" className="text-foreground/80 hover:text-primary ar">الأداء</a></li>
              <li><a href="#docker" className="text-foreground/80 hover:text-primary ar">Docker</a></li>
              <li>
                <span className="text-foreground/40 cursor-not-allowed ar flex items-center gap-1">
                  التوثيق <span className="text-[10px] mono">(soon)</span>
                </span>
              </li>
            </ul>
          </div>
        </div>

        {/* Bottom bar */}
        <div className="mt-10 pt-6 border-t border-border/40 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          <div className="text-xs text-muted-foreground">
            <span className="ar">© 2026 NAWA Project · مفتوح المصدر تحت MIT</span>
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
          <p className="text-xs text-muted-foreground/50 mono">
            <span className="text-primary">∞</span> built with Rust, io_uring & zero copies ·
            <span className="text-accent"> ∞</span> صُمم للسيرفرات الضعيفة
          </p>
        </div>
      </div>
    </footer>
  );
}
