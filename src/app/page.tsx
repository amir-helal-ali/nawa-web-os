import { Navigation } from "@/components/nawa/Navigation";
import { ScrollProgress } from "@/components/nawa/ScrollProgress";
import { Hero } from "@/components/nawa/Hero";
import { Concept } from "@/components/nawa/Concept";
import { BuildFirstWebsite } from "@/components/nawa/BuildFirstWebsite";
import { RequestFlow } from "@/components/nawa/RequestFlow";
import { Architecture } from "@/components/nawa/Architecture";
import { ZeroCopyKernel } from "@/components/nawa/ZeroCopyKernel";
import { DatabaseDemo } from "@/components/nawa/DatabaseDemo";
import { CodePlayground } from "@/components/nawa/CodePlayground";
import { PerformanceDashboard } from "@/components/nawa/PerformanceDashboard";
import { StackComparison } from "@/components/nawa/StackComparison";
import { UseCases } from "@/components/nawa/UseCases";
import { SecurityLayer } from "@/components/nawa/SecurityLayer";
import { PluginMarketplace } from "@/components/nawa/PluginMarketplace";
import { DeveloperExperience } from "@/components/nawa/DeveloperExperience";
import { MigrationGuide } from "@/components/nawa/MigrationGuide";
import { AppBuilder } from "@/components/nawa/AppBuilder";
import { CLISimulator } from "@/components/nawa/CLISimulator";
import { Testimonials } from "@/components/nawa/Testimonials";
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
      <BuildFirstWebsite />
      <RequestFlow />
      <Architecture />
      <ZeroCopyKernel />
      <DatabaseDemo />
      <CodePlayground />
      <PerformanceDashboard />
      <StackComparison />
      <UseCases />
      <SecurityLayer />
      <PluginMarketplace />
      <DeveloperExperience />
      <MigrationGuide />
      <AppBuilder />
      <CLISimulator />
      <Testimonials />
      <FAQ />
      <FinalCTA />
      <Footer />
    </main>
  );
}
