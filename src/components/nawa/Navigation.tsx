"use client";

import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { Hexagon, Menu, X, Github } from "lucide-react";
import { Button } from "@/components/ui/button";

const NAV_ITEMS = [
  { id: "concept", label: "الفكرة", en: "Concept" },
  { id: "architecture", label: "المعمارية", en: "Architecture" },
  { id: "kernel", label: "النواة", en: "Kernel" },
  { id: "database", label: "قاعدة البيانات", en: "Database" },
  { id: "performance", label: "الأداء", en: "Performance" },
  { id: "docker", label: "Docker", en: "Docker" },
  { id: "roadmap", label: "خارطة الطريق", en: "Roadmap" },
];

export function Navigation() {
  const [scrolled, setScrolled] = useState(false);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState("hero");

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 24);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });

    const sections = ["hero", ...NAV_ITEMS.map((n) => n.id)];
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) setActive(e.target.id);
        });
      },
      { rootMargin: "-45% 0px -45% 0px" }
    );
    sections.forEach((id) => {
      const el = document.getElementById(id);
      if (el) observer.observe(el);
    });

    return () => {
      window.removeEventListener("scroll", onScroll);
      observer.disconnect();
    };
  }, []);

  const scrollTo = (id: string) => {
    document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
    setOpen(false);
  };

  return (
    <header
      className={`fixed top-0 inset-x-0 z-50 transition-all duration-300 ${
        scrolled
          ? "backdrop-blur-xl bg-background/75 border-b border-border/60"
          : "bg-transparent"
      }`}
    >
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <button
            onClick={() => scrollTo("hero")}
            className="flex items-center gap-2.5 group"
          >
            <div className="relative">
              <Hexagon className="w-8 h-8 text-primary group-hover:rotate-90 transition-transform duration-500" strokeWidth={1.5} />
              <div className="absolute inset-0 grid place-items-center">
                <span className="font-bold text-[10px] text-primary-foreground bg-primary rounded-sm w-4 h-4 grid place-items-center">ن</span>
              </div>
            </div>
            <div className="text-left">
              <div className="font-bold text-base leading-none tracking-tight">
                NAWA<span className="text-primary">.</span>
              </div>
              <div className="text-[10px] text-muted-foreground leading-none mt-0.5 ar">
                نواة · Web OS
              </div>
            </div>
          </button>

          {/* Desktop nav */}
          <nav className="hidden lg:flex items-center gap-1">
            {NAV_ITEMS.map((item) => (
              <button
                key={item.id}
                onClick={() => scrollTo(item.id)}
                className={`relative px-3 py-2 text-sm rounded-md transition-colors ar ${
                  active === item.id
                    ? "text-primary"
                    : "text-muted-foreground hover:text-foreground"
                }`}
              >
                {item.label}
                {active === item.id && (
                  <motion.div
                    layoutId="nav-active"
                    className="absolute inset-x-2 -bottom-px h-0.5 bg-primary rounded-full"
                  />
                )}
              </button>
            ))}
          </nav>

          {/* Right actions */}
          <div className="flex items-center gap-2">
            <Button variant="ghost" size="icon" className="hidden sm:inline-flex">
              <Github className="w-4 h-4" />
            </Button>
            <Button
              size="sm"
              className="bg-primary hover:bg-primary/90 text-primary-foreground"
              onClick={() => scrollTo("roadmap")}
            >
              ابدأ الرحلة
            </Button>
            <Button
              variant="ghost"
              size="icon"
              className="lg:hidden"
              onClick={() => setOpen((o) => !o)}
            >
              {open ? <X className="w-4 h-4" /> : <Menu className="w-4 h-4" />}
            </Button>
          </div>
        </div>

        {/* Mobile nav */}
        {open && (
          <motion.nav
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            className="lg:hidden pb-4 grid grid-cols-2 gap-1.5"
          >
            {NAV_ITEMS.map((item) => (
              <button
                key={item.id}
                onClick={() => scrollTo(item.id)}
                className={`px-3 py-2 text-sm rounded-md text-left ar ${
                  active === item.id
                    ? "bg-primary/10 text-primary"
                    : "text-muted-foreground hover:text-foreground hover:bg-muted"
                }`}
              >
                <span className="block">{item.label}</span>
                <span className="block text-[10px] opacity-60">{item.en}</span>
              </button>
            ))}
          </motion.nav>
        )}
      </div>
    </header>
  );
}
