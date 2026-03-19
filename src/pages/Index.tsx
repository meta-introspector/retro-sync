import Navbar from "@/components/Navbar";
import Hero from "@/components/Hero";
import Features from "@/components/Features";
import Architecture from "@/components/Architecture";
import Compliance from "@/components/Compliance";
import Footer from "@/components/Footer";

const Index = () => {
  return (
    <div className="min-h-screen">
      <Navbar />
      <Hero />
      <div id="features"><Features /></div>
      <div id="architecture"><Architecture /></div>
      <div id="compliance"><Compliance /></div>
      <Footer />
    </div>
  );
};

export default Index;
