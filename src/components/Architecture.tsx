import { motion } from "framer-motion";

const layers = [
  {
    label: "Frontend",
    tech: "Rust · Yew · WASM · Trunk",
    color: "bg-primary/20 border-primary/40",
  },
  {
    label: "API Gateway",
    tech: "Axum · Tower · mTLS · OPA",
    color: "bg-accent/20 border-accent/40",
  },
  {
    label: "Business Logic",
    tech: "LangSec Parsers · DDEX · CWR · Master Pattern",
    color: "bg-primary/15 border-primary/30",
  },
  {
    label: "Crypto Layer",
    tech: "Groth16 ZK · Ledger Signing · BTTC Smart Contracts",
    color: "bg-accent/15 border-accent/30",
  },
  {
    label: "Storage",
    tech: "BTFS · LMDB · Internet Archive Mirror",
    color: "bg-primary/10 border-primary/20",
  },
];

const Architecture = () => {
  return (
    <section className="relative py-32">
      <div className="container mx-auto px-6">
        <div className="grid lg:grid-cols-2 gap-16 items-center">
          <motion.div
            initial={{ opacity: 0, x: -30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6 }}
          >
            <h2 className="text-3xl sm:text-4xl font-bold mb-4">
              Full-Stack <span className="text-gradient-primary">Architecture</span>
            </h2>
            <p className="text-muted-foreground mb-8 leading-relaxed">
              A Rust-native workspace with WASM frontend, Axum backend, arkworks cryptography, 
              and BTFS decentralized storage — all connected through Zero Trust mTLS.
            </p>
            <div className="glass rounded-xl p-5 font-mono text-sm">
              <div className="text-muted-foreground mb-2">$ cargo workspace</div>
              <div className="text-primary">├── frontend/</div>
              <div className="text-primary">├── backend/</div>
              <div className="text-primary">├── shared/</div>
              <div className="text-primary">├── zk_circuits/</div>
              <div className="text-accent">├── contracts/</div>
              <div className="text-muted-foreground">└── tools/</div>
            </div>
          </motion.div>

          <motion.div
            className="space-y-3"
            initial={{ opacity: 0, x: 30 }}
            whileInView={{ opacity: 1, x: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, delay: 0.2 }}
          >
            {layers.map((layer, i) => (
              <motion.div
                key={layer.label}
                className={`rounded-lg border p-4 ${layer.color} transition-all hover:scale-[1.02]`}
                initial={{ opacity: 0, x: 20 }}
                whileInView={{ opacity: 1, x: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.4, delay: 0.1 * i }}
              >
                <div className="font-semibold text-sm">{layer.label}</div>
                <div className="text-xs text-muted-foreground font-mono mt-1">{layer.tech}</div>
              </motion.div>
            ))}
          </motion.div>
        </div>
      </div>
    </section>
  );
};

export default Architecture;
