import { Navigation } from "@/components/nawa/Navigation";
import { ScrollProgress } from "@/components/nawa/ScrollProgress";
import { Hero } from "@/components/nawa/Hero";
import { Concept } from "@/components/nawa/Concept";
import { RequestFlow } from "@/components/nawa/RequestFlow";
import { Architecture } from "@/components/nawa/Architecture";
import { ZeroCopyKernel } from "@/components/nawa/ZeroCopyKernel";
import { MemoryLayout } from "@/components/nawa/MemoryLayout";
import { DatabaseDemo } from "@/components/nawa/DatabaseDemo";
import { CodePlayground } from "@/components/nawa/CodePlayground";
import { PerformanceDashboard } from "@/components/nawa/PerformanceDashboard";
import { StackComparison } from "@/components/nawa/StackComparison";
import { CostCalculator } from "@/components/nawa/CostCalculator";
import { SecurityLayer } from "@/components/nawa/SecurityLayer";
import { Observability } from "@/components/nawa/Observability";
import { PluginMarketplace } from "@/components/nawa/PluginMarketplace";
import { DeveloperExperience } from "@/components/nawa/DeveloperExperience";
import { MigrationGuide } from "@/components/nawa/MigrationGuide";
import { AppBuilder } from "@/components/nawa/AppBuilder";
import { CLISimulator } from "@/components/nawa/CLISimulator";
import { Testimonials } from "@/components/nawa/Testimonials";
import { Ecosystem } from "@/components/nawa/Ecosystem";
import { DockerDeployment } from "@/components/nawa/DockerDeployment";
import { Roadmap } from "@/components/nawa/Roadmap";
import { FAQ } from "@/components/nawa/FAQ";
import { FinalCTA } from "@/components/nawa/FinalCTA";
import { Footer } from "@/components/nawa/Footer";

export default function Home() {
  return (
    <main className="relative min-h-screen flex flex-col bg-background overflow-x-hidden">
      <ScrollProgress />
      <Navigation />
      <Hero />
      <Concept />
      <RequestFlow />
      <Architecture />
      <ZeroCopyKernel />
      <MemoryLayout />
      <DatabaseDemo />
      <CodePlayground />
      <PerformanceDashboard />
      <StackComparison />
      <CostCalculator />
      <SecurityLayer />
      <Observability />
      <PluginMarketplace />
      <DeveloperExperience />
      <MigrationGuide />
      <AppBuilder />
      <CLISimulator />
      <Testimonials />
      <Ecosystem />
      <DockerDeployment />
      <Roadmap />
      <FAQ />
      <FinalCTA />
      <Footer />
    </main>
  );
}
