import { motion } from "framer-motion";
import { ShieldCheck, Coins, Cpu, Zap, Lock, EyeOff } from "lucide-react";

const features = [
  {
    icon: ShieldCheck,
    title: "Algorithmic Payouts",
    description: "Groth16 proofs ensure royalty distribution is immutable and verified on-chain.",
  },
  {
    icon: EyeOff,
    title: "Anonymity by Default",
    description: "Cryptographic digital IDs — no names, no PII, no trackers.",
  },
  {
    icon: Coins,
    title: "Non-Custodial Economy",
    description: "Funds move peer-to-peer from audience to artist via BitTorrent Chain.",
  },
  {
    icon: Cpu,
    title: "Spectrum Analysis",
    description: "Real-time frequency breakdown ensures professional-grade quality benchmarks.",
  },
  {
    icon: Lock,
    title: "Censorship Resistant",
    description: "Distributed across the global BTFS network. Your art is permanently accessible.",
  },
  {
    icon: Zap,
    title: "Zero Latency",
    description: "Instantaneous settlement. Capital in your sovereign control the moment it's mined.",
  },
];

const Features = () => {
  return (
    <section className="relative py-24 md:py-32 bg-card">
      <div className="container mx-auto px-6">
        {/* Asymmetric header — left-aligned with offset */}
        <motion.div
          className="mb-16 md:mb-20 max-w-2xl"
          initial={{ opacity: 0, x: -20 }}
          whileInView={{ opacity: 1, x: 0 }}
          viewport={{ once: true }}
        >
          <span className="text-xs font-mono text-primary/70 tracking-widest uppercase mb-4 block">
            Capabilities
          </span>
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight mb-4">
            Total <span className="text-gradient-primary">Sovereignty.</span>
          </h2>
          <p className="text-muted-foreground max-w-md text-base leading-relaxed">
            Corporate trust replaced with cryptographic certainty.
          </p>
        </motion.div>

        {/* Asymmetric grid — 2 cols on mobile, 3 on desktop with varying sizes */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-px bg-border">
          {features.map((feature, i) => (
            <motion.div
              key={feature.title}
              className="bg-card p-6 sm:p-8 hover:bg-secondary/50 transition-all group relative"
              initial={{ opacity: 0, y: 10 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.05 }}
            >
              <div className="absolute top-4 right-4 text-3xl font-bold text-border opacity-50 group-hover:opacity-100 transition-opacity font-mono">
                {String(i + 1).padStart(2, "0")}
              </div>

              <div className="w-10 h-10 bg-secondary border border-border flex items-center justify-center mb-5 group-hover:border-primary/50 transition-colors">
                <feature.icon className="w-5 h-5 text-primary" />
              </div>
              <h3 className="text-lg font-bold mb-2 tracking-tight">{feature.title}</h3>
              <p className="text-sm text-muted-foreground leading-relaxed">{feature.description}</p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Features;
