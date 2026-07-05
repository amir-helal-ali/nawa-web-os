import { Navigation } from "@/components/nawa/Navigation";
import { Hero } from "@/components/nawa/Hero";
import { Concept } from "@/components/nawa/Concept";
import { RequestFlow } from "@/components/nawa/RequestFlow";
import { Architecture } from "@/components/nawa/Architecture";
import { ZeroCopyKernel } from "@/components/nawa/ZeroCopyKernel";
import { DatabaseDemo } from "@/components/nawa/DatabaseDemo";
import { CodePlayground } from "@/components/nawa/CodePlayground";
import { PerformanceDashboard } from "@/components/nawa/PerformanceDashboard";
import { StackComparison } from "@/components/nawa/StackComparison";
import { SecurityLayer } from "@/components/nawa/SecurityLayer";
import { PluginMarketplace } from "@/components/nawa/PluginMarketplace";
import { AppBuilder } from "@/components/nawa/AppBuilder";
import { CLISimulator } from "@/components/nawa/CLISimulator";
import { DockerDeployment } from "@/components/nawa/DockerDeployment";
import { Roadmap } from "@/components/nawa/Roadmap";
import { Footer } from "@/components/nawa/Footer";

export default function Home() {
  return (
    <main className="relative min-h-screen flex flex-col bg-background overflow-x-hidden">
      <Navigation />
      <Hero />
      <Concept />
      <RequestFlow />
      <Architecture />
      <ZeroCopyKernel />
      <DatabaseDemo />
      <CodePlayground />
      <PerformanceDashboard />
      <StackComparison />
      <SecurityLayer />
      <PluginMarketplace />
      <AppBuilder />
      <CLISimulator />
      <DockerDeployment />
      <Roadmap />
      <Footer />
    </main>
  );
}
