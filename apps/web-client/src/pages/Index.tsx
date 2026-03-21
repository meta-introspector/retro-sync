import Navbar from "@/components/Navbar";
import Hero from "@/components/Hero";
import Features from "@/components/Features";
import Comparison from "@/components/Comparison";
import HowItWorks from "@/components/HowItWorks";
import Pricing from "@/components/Pricing";
import Compliance from "@/components/Compliance";
import Footer from "@/components/Footer";

const Index = () => {
  return (
    <div className="min-h-screen">
      <Navbar />
      <Hero />

      {/* Mission — editorial offset layout */}
      <section className="py-24 md:py-32 bg-background">
        <div className="container mx-auto px-6">
          <div className="grid lg:grid-cols-12 gap-12 items-start">
            <div className="lg:col-span-5">
              <h2 className="text-3xl sm:text-4xl lg:text-5xl font-bold tracking-tight leading-tight mb-4">
                Driven by an{" "}
                <span className="text-gradient-primary">Artist's Journey</span>
              </h2>
              <a href="/docs/whitepaper.md" className="inline-flex items-center gap-2 text-primary font-medium text-sm hover:underline mt-2">
                Read the Whitepaper →
              </a>
            </div>
            <div className="lg:col-span-7 lg:col-start-6">
              <div className="space-y-5 text-base text-muted-foreground leading-relaxed">
                <p>
                  At RetroSync, our mission is deeply personal, born from a five-year journey navigating the music industry from an artist's perspective. As a professional artist and a dedicated mother, I've experienced firsthand the frustration of unreceived or lost payments — a reality that stifles creativity and makes sustainable careers incredibly challenging.
                </p>
                <p>
                  Fueled by this passion for fairness and a deep understanding of artists' needs, RetroSync was conceived. We are building a transparent, artist-centric platform designed to empower creators. Our goal is to ensure artists are compensated fairly and equitably, putting control back into their hands.
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

      <div id="features"><Features /></div>
      <div id="comparison"><Comparison /></div>
      <div id="how-it-works"><HowItWorks /></div>
      <div id="pricing"><Pricing /></div>
      <div id="trust"><Compliance /></div>
      <Footer />
    </div>
  );
};

export default Index;
