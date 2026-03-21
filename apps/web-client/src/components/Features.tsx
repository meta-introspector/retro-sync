import { motion } from "framer-motion";
import { ShieldCheck, UserCheck, Coins, Cpu, Globe, Zap, Terminal, Lock, EyeOff } from "lucide-react";

const features = [
  {
    icon: ShieldCheck,
    title: "Algorithmic Payouts",
    description: "Mathematics is the only arbiter. Groth16 proofs ensure royalty distribution is immutable and verified on-chain.",
  },
  {
    icon: EyeOff,
    title: "Anonymity by Default",
    description: "Your legal name is a liability. Retrosync operates via cryptographic digital IDs—no names, no PII, no trackers.",
  },
  {
    icon: Coins,
    title: "Non-Custodial Economy",
    description: "We never touch your money. Funds move peer-to-peer from audience to artist via BitTorrent Chain protocols.",
  },
  {
    icon: Cpu,
    title: "Spectrum Analysis",
    description: "High-fidelity Monster WAV monitoring. Real-time frequency breakdown ensures professional-grade quality benchmarks.",
  },
  {
    icon: Lock,
    title: "Censorship Resistant",
    description: "Distributed across the global BTFS network. Your art is unkillable, unblockable, and permanently accessible.",
  },
  {
    icon: Zap,
    title: "Zero Latency",
    description: "Instantaneous settlement. As soon as the transaction is mined, the capital is in your sovereign control.",
  },
];

const Features = () => {
  return (
    <section className="relative py-32 bg-black scanline">
      <div className="container mx-auto px-6">
        <motion.div
          className="text-left mb-20 border-l-4 border-primary pl-8"
          initial={{ opacity: 0, x: -20 }}
          whileInView={{ opacity: 1, x: 0 }}
          viewport={{ once: true }}
        >
          <div className="flex items-center gap-2 mb-4 text-primary">
            <Terminal className="w-4 h-4" />
            <span className="text-[10px] font-bold uppercase tracking-[0.5em]">Capabilities_List</span>
          </div>
          <h2 className="text-5xl sm:text-7xl font-black italic tracking-tighter mb-4">
            Total <span className="text-gradient-primary">Sovereignty.</span>
          </h2>
          <p className="text-zinc-500 max-w-2xl text-lg font-mono leading-tight">
            We've replaced corporate trust with cryptographic certainty.
          </p>
        </motion.div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-1">
          {features.map((feature, i) => (
            <motion.div
              key={feature.title}
              className="bg-zinc-950 border border-zinc-900 p-8 hover:border-primary/50 transition-all group relative overflow-hidden"
              initial={{ opacity: 0, y: 10 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.05 }}
            >
              <div className="absolute top-0 right-0 p-2 opacity-10 group-hover:opacity-30 transition-opacity">
                <span className="text-[40px] font-black italic text-zinc-800">{i + 1}</span>
              </div>
              
              <div className="w-12 h-12 rounded-none bg-zinc-900 border border-zinc-800 flex items-center justify-center mb-6 group-hover:border-primary transition-colors">
                <feature.icon className="w-6 h-6 text-primary" />
              </div>
              <h3 className="text-xl font-black italic uppercase mb-3 tracking-tighter">{feature.title}</h3>
              <p className="text-sm text-zinc-500 leading-tight font-mono">{feature.description}</p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Features;
