import { motion } from "framer-motion";
import { ArrowRight, Zap, ShieldAlert, Terminal, Lock, Cpu } from "lucide-react";
import { Link } from "react-router-dom";

const Hero = () => {
  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden scanline">
      <div className="absolute inset-0 grid-pattern opacity-20" />
      
      {/* Cypherpunk Orbs */}
      <div className="absolute top-1/4 left-1/4 w-[400px] h-[400px] rounded-full bg-primary/5 blur-[100px]" />
      <div className="absolute bottom-1/4 right-1/4 w-[400px] h-[400px] rounded-full bg-accent/5 blur-[100px]" />

      <div className="relative z-10 container mx-auto px-6 text-center">
        <motion.h1
          className="text-5xl sm:text-7xl lg:text-9xl font-black tracking-tighter leading-[0.8] mb-8 uppercase italic"
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.5, delay: 0.1 }}
        >
          RetroSync:
          <br />
          <span className="text-gradient-primary">Our Mission, Driven by an Artist's Journey</span>
        </motion.h1>

        <motion.p
          className="text-lg sm:text-xl text-zinc-500 max-w-2xl mx-auto mb-12 font-mono leading-tight border-l-2 border-primary/30 pl-6 text-left"
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
        >
          At RetroSync, our mission is deeply personal, born from a five-year journey navigating the music industry from an artist's perspective. As a professional artist and a dedicated mother, I've experienced firsthand the frustration of unreceived or lost payments – a reality that stifles creativity and makes sustainable careers incredibly challenging. This systemic issue affects countless talents worldwide who dream of earning a living from their art.
          <br /><br />
          Fueled by this passion for fairness and a deep understanding of artists' needs, RetroSync was conceived. We are building a transparent, artist-centric platform designed to empower creators. Our goal is to ensure artists are compensated fairly and equitably, putting control back into their hands. RetroSync is our answer to a broken system, a commitment to building a future where every artist is valued and rightfully rewarded for their work.
        </motion.p>

        <motion.div
          className="flex flex-col sm:flex-row items-center justify-center gap-4"
          initial={{ opacity: 0, y: 16 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.3 }}
        >
          <Link to="/upload" className="w-full sm:w-auto">
            <button className="w-full px-8 py-4 bg-primary text-primary-foreground font-black uppercase tracking-widest hover:bg-primary/90 transition-all shadow-[4px_4px_0px_0px_rgba(255,255,255,0.2)] active:translate-x-[2px] active:translate-y-[2px] active:shadow-none flex items-center justify-center gap-2">
              Join the Resistance <ArrowRight className="w-5 h-5" />
            </button>
          </Link>
          <Link to="/marketplace" className="w-full sm:w-auto">
            <button className="w-full px-8 py-4 border border-zinc-700 bg-zinc-950 text-foreground font-black uppercase tracking-widest hover:border-primary/50 transition-all flex items-center justify-center gap-2">
              <Cpu className="w-5 h-5" /> Market Access
            </button>
          </Link>
        </motion.div>

        <motion.div
          className="mt-24 grid grid-cols-2 md:grid-cols-4 gap-4 max-w-4xl mx-auto"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.8, delay: 0.5 }}
        >
          {[
            { value: "Sovereign", label: "Identity", icon: Lock },
            { value: "Peer-2-Peer", label: "Payments", icon: Zap },
            { value: "Encrypted", label: "Distribution", icon: ShieldAlert },
            { value: "Unstoppable", label: "Network", icon: Cpu },
          ].map((stat) => (
            <div key={stat.label} className="p-4 bg-zinc-950 border border-zinc-800 hover:border-primary/30 transition-colors group">
              <stat.icon className="w-4 h-4 text-zinc-600 group-hover:text-primary mb-3 mx-auto transition-colors" />
              <div className="text-xl font-black text-zinc-300 group-hover:text-primary transition-colors">{stat.value}</div>
              <div className="text-[10px] text-zinc-600 uppercase font-bold tracking-widest mt-1">{stat.label}</div>
            </div>
          ))}
        </motion.div>
      </div>
    </section>
  );
};

export default Hero;
