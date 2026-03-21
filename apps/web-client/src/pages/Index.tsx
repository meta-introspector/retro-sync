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

      <section className="py-20 bg-black"> {/* Or a suitable background color */}
        <div className="container mx-auto px-6">
          <h2 className="text-4xl lg:text-6xl font-black tracking-tighter leading-[0.8] mb-8 uppercase italic text-center text-gradient-primary">
            Our Mission: Driven by an Artist's Journey
          </h2>
          <div className="max-w-3xl mx-auto text-lg text-zinc-300 font-mono leading-relaxed text-center">
            <p>
              At RetroSync, our mission is deeply personal, born from a five-year journey navigating the music industry from an artist's perspective. As a professional artist and a dedicated mother, I've experienced firsthand the frustration of unreceived or lost payments – a reality that stifles creativity and makes sustainable careers incredibly challenging. This systemic issue affects countless talents worldwide who dream of earning a living from their art.
            </p>
            <p className="mt-6">
              Fueled by this passion for fairness and a deep understanding of artists' needs, RetroSync was conceived. We are building a transparent, artist-centric platform designed to empower creators. Our goal is to ensure artists are compensated fairly and equitably, putting control back into their hands. RetroSync is our answer to a broken system, a commitment to building a future where every artist is valued and rightfully rewarded for their work.
            </p>
          </div>
          <div className="text-center mt-12">
            <a href="/docs/whitepaper.md" className="text-primary font-bold text-lg hover:underline">
              Explore the Whitepaper
            </a>
          </div>
        </div>
      <Navbar />
      <Hero />

      <section className="py-20 bg-black"> {/* Or a suitable background color */}
        <div className="container mx-auto px-6">
          <h2 className="text-4xl lg:text-6xl font-black tracking-tighter leading-[0.8] mb-8 uppercase italic text-center text-gradient-primary">
            Our Mission: Driven by an Artist's Journey
          </h2>
          <div className="max-w-3xl mx-auto text-lg text-zinc-300 font-mono leading-relaxed text-center">
            <p>
              At RetroSync, our mission is deeply personal, born from a five-year journey navigating the music industry from an artist's perspective. As a professional artist and a dedicated mother, I've experienced firsthand the frustration of unreceived or lost payments – a reality that stifles creativity and makes sustainable careers incredibly challenging. This systemic issue affects countless talents worldwide who dream of earning a living from their art.
            </p>
            <p className="mt-6">
              Fueled by this passion for fairness and a deep understanding of artists' needs, RetroSync was conceived. We are building a transparent, artist-centric platform designed to empower creators. Our goal is to ensure artists are compensated fairly and equitably, putting control back into their hands. RetroSync is our answer to a broken system, a commitment to building a future where every artist is valued and rightfully rewarded for their work.
            </p>
          </div>
          <div className="text-center mt-12">
            <a href="/docs/whitepaper.md" className="text-primary font-bold text-lg hover:underline">
              Explore the Whitepaper
            </a>
          </div>
        </div>
      <Navbar />
      <Hero />

      <section className="py-20 bg-black"> {/* Or a suitable background color */}
        <div className="container mx-auto px-6">
          <h2 className="text-4xl lg:text-6xl font-black tracking-tighter leading-[0.8] mb-8 uppercase italic text-center text-gradient-primary">
            Our Mission: Driven by an Artist's Journey
          </h2>
          <div className="max-w-3xl mx-auto text-lg text-zinc-300 font-mono leading-relaxed text-center">
            <p>
              At RetroSync, our mission is deeply personal, born from a five-year journey navigating the music industry from an artist's perspective. As a professional artist and a dedicated mother, I've experienced firsthand the frustration of unreceived or lost payments – a reality that stifles creativity and makes sustainable careers incredibly challenging. This systemic issue affects countless talents worldwide who dream of earning a living from their art.
            </p>
            <p className="mt-6">
              Fueled by this passion for fairness and a deep understanding of artists' needs, RetroSync was conceived. We are building a transparent, artist-centric platform designed to empower creators. Our goal is to ensure artists are compensated fairly and equitably, putting control back into their hands. RetroSync is our answer to a broken system, a commitment to building a future where every artist is valued and rightfully rewarded for their work.
            </p>
          </div>
          <div className="text-center mt-12">
            <a href="/docs/whitepaper.md" className="text-primary font-bold text-lg hover:underline">
              Explore the Whitepaper
            </a>
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


