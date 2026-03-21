import { motion } from "framer-motion";
import { Check, X, Shield, Zap, Target, BarChart, Lock, UserX, Share2 } from "lucide-react";

const comparisonData = [
  {
    feature: "Annual Fee",
    legacy: "$20 - $100 / year",
    retro: "$0 (Forever)",
    icon: Zap,
    description: "Keep 100% of your initial capital."
  },
  {
    feature: "Identity Protection",
    legacy: "Personal Info Required",
    retro: "Completely Private",
    icon: UserX,
    description: "Your wallet is your ID. No legal names needed."
  },
  {
    feature: "Payment Speed",
    legacy: "60-90 Days",
    retro: "Instant",
    icon: Zap,
    description: "Get paid the moment a sale is verified."
  },
  {
    feature: "Payout Accuracy",
    legacy: "Unverified Estimates",
    retro: "Mathematically Proven",
    icon: Target,
    description: "ZK-SNARK certainty on every cent."
  },
  {
    feature: "Audio Monitoring",
    legacy: "Basic Tools",
    retro: "Pro Frequency Monitor",
    icon: BarChart,
    description: "High-fidelity spectrum analysis for every track."
  },
  {
    feature: "Security",
    legacy: "Centralized Servers",
    retro: "Global Encryption",
    icon: Lock,
    description: "Distributed storage across the BTFS network."
  },
  {
    feature: "Ownership Control",
    legacy: "Platform-Owned",
    retro: "Sovereign Ownership",
    icon: Share2,
    description: "Direct peer-to-peer settlement via wallet."
  },
];

const Comparison = () => {
  return (
    <section className="py-32 relative bg-zinc-950 overflow-hidden">
      {/* Background Glow */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-primary/5 blur-[120px] rounded-full pointer-events-none" />

      <div className="container mx-auto px-6 relative z-10">
        <motion.div
          className="text-center mb-20"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
        >
          <h2 className="text-4xl sm:text-6xl font-bold mb-6 tracking-tight">
            The <span className="text-gradient-primary">Upgrade</span> You Deserve
          </h2>
          <p className="text-zinc-400 max-w-2xl mx-auto text-lg">
            Stop living in the past. Retrosync is built for the way music is made and sold today.
          </p>
        </motion.div>

        <div className="grid grid-cols-1 gap-4 max-w-5xl mx-auto">
          {/* Header Row - Hidden on small screens */}
          <div className="hidden md:grid grid-cols-12 gap-4 px-8 mb-4">
            <div className="col-span-4 text-[10px] uppercase tracking-widest font-bold text-zinc-500">Feature</div>
            <div className="col-span-4 text-[10px] uppercase tracking-widest font-bold text-zinc-500">Traditional Platforms</div>
            <div className="col-span-4 text-[10px] uppercase tracking-widest font-bold text-primary">Retrosync Protocol</div>
          </div>

          {comparisonData.map((item, i) => (
            <motion.div
              key={item.feature}
              className="glass rounded-2xl overflow-hidden group hover:border-primary/30 transition-all duration-500"
              initial={{ opacity: 0, x: -20 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.05 }}
            >
              <div className="grid grid-cols-1 md:grid-cols-12 items-center">
                {/* Feature Title */}
                <div className="col-span-1 md:col-span-4 p-6 md:p-8 flex items-center gap-4 bg-zinc-900/30">
                  <div className="w-10 h-10 rounded-xl bg-zinc-800 flex items-center justify-center group-hover:bg-primary/10 group-hover:text-primary transition-colors">
                    <item.icon className="w-5 h-5" />
                  </div>
                  <div>
                    <div className="font-bold text-zinc-200">{item.feature}</div>
                    <div className="text-[10px] text-zinc-500 uppercase tracking-tighter">{item.description}</div>
                  </div>
                </div>

                {/* Legacy Data */}
                <div className="col-span-1 md:col-span-4 p-6 md:p-8 border-t md:border-t-0 md:border-l border-zinc-800/50">
                  <div className="flex items-center gap-3 text-zinc-500">
                    <div className="w-5 h-5 rounded-full bg-red-500/10 flex items-center justify-center shrink-0">
                      <X className="w-3 h-3 text-red-500/50" />
                    </div>
                    <span className="text-sm font-medium">{item.legacy}</span>
                  </div>
                </div>

                {/* Retrosync Data */}
                <div className="col-span-1 md:col-span-4 p-6 md:p-8 bg-primary/5 border-t md:border-t-0 md:border-l border-primary/10">
                  <div className="flex items-center gap-3 text-primary">
                    <div className="w-6 h-6 rounded-full bg-primary/20 flex items-center justify-center shrink-0 shadow-[0_0_15px_rgba(124,58,237,0.3)]">
                      <Check className="w-4 h-4 text-primary" />
                    </div>
                    <span className="text-base font-bold">{item.retro}</span>
                  </div>
                </div>
              </div>
            </motion.div>
          ))}
        </div>

        <motion.div 
          className="mt-16 text-center"
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
        >
          <div className="inline-flex items-center gap-2 p-1 pr-4 rounded-full bg-zinc-900 border border-zinc-800">
            <div className="px-3 py-1 rounded-full bg-primary text-[10px] font-bold uppercase">Ready?</div>
            <span className="text-xs text-zinc-400">Join the sovereign sound revolution.</span>
          </div>
        </motion.div>
      </div>
    </section>
  );
};

export default Comparison;
