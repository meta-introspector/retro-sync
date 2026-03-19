import { motion } from "framer-motion";
import { Shield, Music, Zap, Globe, Lock, BarChart3 } from "lucide-react";

const features = [
  {
    icon: Shield,
    title: "Zero-Knowledge Royalties",
    description: "Groth16/BN254 proofs verify royalty splits on-chain without exposing individual artist addresses or payment details.",
  },
  {
    icon: Music,
    title: "DDEX ERN 4.1",
    description: "Automated rights registration with full DDEX compliance. CWR generation for 15+ collecting societies worldwide.",
  },
  {
    icon: Zap,
    title: "Master Pattern Protocol",
    description: "Mod-9 supersingular prime classification assigns rarity tiers — Common, Rare, and Legendary — to every track.",
  },
  {
    icon: Globe,
    title: "Decentralized Storage",
    description: "BTFS-powered content distribution with Internet Archive and BBS mirroring for permanent, censorship-resistant access.",
  },
  {
    icon: Lock,
    title: "Zero Trust Security",
    description: "SPIFFE/SPIRE identity framework with mTLS, OPA policy enforcement, and Ledger hardware signing for all transactions.",
  },
  {
    icon: BarChart3,
    title: "Six Sigma Analytics",
    description: "SPC control charts and process capability analysis ensure distribution quality meets ISO 9001 and GMP standards.",
  },
];

const Features = () => {
  return (
    <section className="relative py-32">
      <div className="container mx-auto px-6">
        <motion.div
          className="text-center mb-16"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
        >
          <h2 className="text-3xl sm:text-4xl font-bold mb-4">
            Enterprise-Grade <span className="text-gradient-primary">Music Infrastructure</span>
          </h2>
          <p className="text-muted-foreground max-w-xl mx-auto">
            Every component built with formal verification, LangSec parsing, and cryptographic guarantees.
          </p>
        </motion.div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, i) => (
            <motion.div
              key={feature.title}
              className="glass rounded-xl p-6 hover:border-primary/30 transition-colors group"
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: i * 0.1 }}
            >
              <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center mb-4 group-hover:bg-primary/20 transition-colors">
                <feature.icon className="w-5 h-5 text-primary" />
              </div>
              <h3 className="text-lg font-semibold mb-2">{feature.title}</h3>
              <p className="text-sm text-muted-foreground leading-relaxed">{feature.description}</p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Features;
