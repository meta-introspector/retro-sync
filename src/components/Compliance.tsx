import { motion } from "framer-motion";

const societies = [
  "ASCAP", "BMI", "SESAC", "PRS", "GEMA", "SACEM", "JASRAC",
  "SOCAN", "APRA", "STIM", "TONO", "KODA", "SAMRO", "SoundExchange", "MLC",
];

const Compliance = () => {
  return (
    <section className="relative py-32 overflow-hidden">
      <div className="container mx-auto px-6">
        <motion.div
          className="text-center mb-16"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
        >
          <h2 className="text-3xl sm:text-4xl font-bold mb-4">
            Global <span className="text-gradient-primary">Rights Compliance</span>
          </h2>
          <p className="text-muted-foreground max-w-xl mx-auto">
            Automated CWR 2.1/2.2 generation and submission to collecting societies in every major territory.
          </p>
        </motion.div>

        <motion.div
          className="flex flex-wrap justify-center gap-3 max-w-3xl mx-auto"
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true }}
          transition={{ duration: 0.8 }}
        >
          {societies.map((society, i) => (
            <motion.span
              key={society}
              className="glass rounded-full px-4 py-2 text-sm font-mono text-muted-foreground hover:text-primary hover:border-primary/30 transition-colors cursor-default"
              initial={{ opacity: 0, scale: 0.8 }}
              whileInView={{ opacity: 1, scale: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 0.3, delay: i * 0.04 }}
            >
              {society}
            </motion.span>
          ))}
        </motion.div>

        <motion.div
          className="mt-16 grid sm:grid-cols-3 gap-6 max-w-3xl mx-auto"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.3 }}
        >
          {[
            { label: "DMCA §512", desc: "Notice-and-takedown with counter-notice workflow" },
            { label: "EU Art. 17", desc: "Copyright Directive compliance built in" },
            { label: "MMA §115", desc: "Mechanical licensing via The MLC" },
          ].map((item) => (
            <div key={item.label} className="glass rounded-xl p-5 text-center">
              <div className="text-accent font-bold font-mono mb-2">{item.label}</div>
              <div className="text-xs text-muted-foreground">{item.desc}</div>
            </div>
          ))}
        </motion.div>
      </div>
    </section>
  );
};

export default Compliance;
