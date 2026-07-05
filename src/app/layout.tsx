import type { Metadata } from "next";
import { Geist, Geist_Mono, Noto_Sans_Arabic, JetBrains_Mono } from "next/font/google";
import "./globals.css";
import { Toaster } from "@/components/ui/toaster";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

const notoArabic = Noto_Sans_Arabic({
  variable: "--font-arabic",
  subsets: ["arabic"],
  weight: ["400", "500", "600", "700", "800"],
});

const jetMono = JetBrains_Mono({
  variable: "--font-jet-mono",
  subsets: ["latin"],
  weight: ["400", "500", "600", "700"],
});

export const metadata: Metadata = {
  title: "NAWA · نواة — Web Operating System built in Rust",
  description:
    "نواة (NAWA) هو نظام تشغيل ويب احترافي مكتوب بـ Rust يعمل عبر Docker، بمحركين للخلفية والواجهة وقاعدة بيانات KV/Document مخصصة. مصمم للعمل بكفاءة على السيرفرات الضعيفة (512MB RAM).",
  keywords: [
    "NAWA",
    "Rust",
    "Web OS",
    "Docker",
    "KV Database",
    "io_uring",
    "zero-copy",
    "SSR",
    "Island Architecture",
    "نواة",
    "نظام تشغيل ويب",
  ],
  authors: [{ name: "NAWA Project" }],
  icons: {
    icon: "https://z-cdn.chatglm.cn/z-ai/static/logo.svg",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="ar" dir="ltr" suppressHydrationWarning className="dark">
      <body
        className={`${geistSans.variable} ${geistMono.variable} ${notoArabic.variable} ${jetMono.variable} antialiased bg-background text-foreground`}
      >
        {children}
        <Toaster />
      </body>
    </html>
  );
}
