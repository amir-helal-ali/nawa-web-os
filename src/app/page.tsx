import { Navigation } from "@/components/nawa/Navigation";
import { Hero } from "@/components/nawa/Hero";
import { Concept } from "@/components/nawa/Concept";
import { Architecture } from "@/components/nawa/Architecture";
import { ZeroCopyKernel } from "@/components/nawa/ZeroCopyKernel";
import { DatabaseDemo } from "@/components/nawa/DatabaseDemo";
import { PerformanceDashboard } from "@/components/nawa/PerformanceDashboard";
import { DockerDeployment } from "@/components/nawa/DockerDeployment";
import { Roadmap } from "@/components/nawa/Roadmap";
import { Footer } from "@/components/nawa/Footer";

export default function Home() {
  return (
    <main className="relative min-h-screen flex flex-col bg-background overflow-x-hidden">
      <Navigation />
      <Hero />
      <Concept />
      <Architecture />
      <ZeroCopyKernel />
      <DatabaseDemo />
      <PerformanceDashboard />
      <DockerDeployment />
      <Roadmap />
      <Footer />
    </main>
  );
}
