import { motion } from "framer-motion";

const platforms = [
  "Spotify", "Apple Music", "YouTube Music", "TikTok", "Amazon Music",
  "Deezer", "Tidal", "Pandora", "SoundCloud", "iHeartRadio",
  "Shazam", "Instagram", "Facebook", "Snapchat", "Tencent",
];

const Compliance = () => {
  return (
    <section className="relative py-32 overflow-hidden">
      <div className="container mx-auto px-6">
        <motion.div
          className="text-center mb-16"
          initial={{ opacity: 0, y: 20, filter: "blur(4px)" }}
          whileInView={{ opacity: 1, y: 0, filter: "blur(0px)" }}
          viewport={{ once: true, amount: 0.2 }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
        >
          <h2 className="text-3xl sm:text-4xl font-bold mb-4">
            Your Music,{" "}
            <span className="text-gradient-primary">Everywhere</span>
          </h2>
          <p className="text-muted-foreground max-w-xl mx-auto">
            We deliver to all the places your listeners already are.
          </p>
        </motion.div>

        <motion.div
          className="flex flex-wrap justify-center gap-3 max-w-3xl mx-auto"
          initial={{ opacity: 0 }}
          whileInView={{ opacity: 1 }}
          viewport={{ once: true, amount: 0.2 }}
          transition={{ duration: 0.8 }}
        >
          {platforms.map((platform, i) => (
            <motion.span
              key={platform}
              className="glass rounded-full px-5 py-2.5 text-sm text-muted-foreground hover:text-primary hover:border-primary/30 transition-colors cursor-default"
              initial={{ opacity: 0, scale: 0.9 }}
              whileInView={{ opacity: 1, scale: 1 }}
              viewport={{ once: true }}
              transition={{ duration: 0.3, delay: i * 0.03 }}
            >
              {platform}
            </motion.span>
          ))}
        </motion.div>

        <motion.div
          className="mt-16 grid sm:grid-cols-3 gap-6 max-w-3xl mx-auto"
          initial={{ opacity: 0, y: 20, filter: "blur(4px)" }}
          whileInView={{ opacity: 1, y: 0, filter: "blur(0px)" }}
          viewport={{ once: true, amount: 0.2 }}
          transition={{ delay: 0.2, duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
        >
          {[
            { label: "Your Rights Protected", desc: "We register and defend your copyright worldwide" },
            { label: "Your Data Is Private", desc: "Full control over your data — download or delete anytime" },
            { label: "Secure Payouts", desc: "Identity-verified payments straight to your wallet or bank" },
          ].map((item) => (
            <div key={item.label} className="glass rounded-xl p-5 text-center">
              <div className="text-accent font-bold mb-2">{item.label}</div>
              <div className="text-xs text-muted-foreground">{item.desc}</div>
            </div>
          ))}
        </motion.div>
      </div>
    </section>
  );
};

export default Compliance;
