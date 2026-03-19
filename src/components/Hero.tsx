import { motion } from "framer-motion";

const Hero = () => {
  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden">
      {/* Grid background */}
      <div className="absolute inset-0 grid-pattern opacity-40" />
      
      {/* Radial glow */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[800px] rounded-full bg-primary/5 blur-[120px]" />
      <div className="absolute top-1/3 right-1/4 w-[400px] h-[400px] rounded-full bg-accent/5 blur-[100px]" />

      <div className="relative z-10 container mx-auto px-6 text-center">
        <motion.div
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease: "easeOut" }}
        >
          <div className="inline-flex items-center gap-2 glass rounded-full px-4 py-1.5 mb-8">
            <span className="w-2 h-2 rounded-full bg-primary animate-pulse-glow" />
            <span className="text-sm text-muted-foreground font-mono tracking-wide">
              Enterprise Music Infrastructure
            </span>
          </div>
        </motion.div>

        <motion.h1
          className="text-5xl sm:text-7xl lg:text-8xl font-bold tracking-tight leading-[0.9] mb-6"
          initial={{ opacity: 0, y: 40 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.15 }}
        >
          <span className="text-gradient-primary">Retrosync</span>
        </motion.h1>

        <motion.p
          className="text-lg sm:text-xl text-muted-foreground max-w-2xl mx-auto mb-10 leading-relaxed"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.3 }}
        >
          Decentralized music distribution with zero-knowledge royalty verification, 
          DDEX compliance, and blockchain-backed provenance — all in one platform.
        </motion.p>

        <motion.div
          className="flex flex-col sm:flex-row items-center justify-center gap-4"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, delay: 0.45 }}
        >
          <button className="px-8 py-3.5 rounded-lg bg-primary text-primary-foreground font-semibold hover:brightness-110 transition-all glow-primary">
            Get Started
          </button>
          <button className="px-8 py-3.5 rounded-lg glass text-foreground font-semibold hover:bg-secondary transition-all">
            View Documentation
          </button>
        </motion.div>

        {/* Stats bar */}
        <motion.div
          className="mt-20 grid grid-cols-2 md:grid-cols-4 gap-6 max-w-3xl mx-auto"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 1, delay: 0.7 }}
        >
          {[
            { value: "ZK", label: "Groth16 Proofs" },
            { value: "DDEX", label: "ERN 4.1 Compliant" },
            { value: "BTFS", label: "Decentralized Storage" },
            { value: "BTTC", label: "On-Chain Royalties" },
          ].map((stat) => (
            <div key={stat.label} className="text-center">
              <div className="text-2xl font-bold text-primary font-mono">{stat.value}</div>
              <div className="text-xs text-muted-foreground mt-1">{stat.label}</div>
            </div>
          ))}
        </motion.div>
      </div>
    </section>
  );
};

export default Hero;
