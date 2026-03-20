import { motion } from "framer-motion";
import { Shield, Music, Zap, Globe, BarChart3, Wallet } from "lucide-react";

const features = [
  {
    icon: Wallet,
    title: "See Every Dollar",
    description:
      "Know exactly how much you earned, where it came from, and when you'll get paid. No more mystery statements.",
  },
  {
    icon: Music,
    title: "Reach Every Listener",
    description:
      "Your music goes live on Spotify, Apple Music, TikTok, YouTube, and 150+ other platforms worldwide.",
  },
  {
    icon: Zap,
    title: "Fast Releases",
    description:
      "Upload your track today, and it can be live everywhere within 24 hours. No waiting weeks.",
  },
  {
    icon: Globe,
    title: "Your Music Is Safe Forever",
    description:
      "Your masters are stored with multiple backups so they're never lost, even if a service goes down.",
  },
  {
    icon: Shield,
    title: "Your Rights Are Protected",
    description:
      "We handle copyright registration and takedowns so nobody can steal or misuse your work.",
  },
  {
    icon: BarChart3,
    title: "Simple Analytics",
    description:
      "See which songs are performing, which countries love your music, and how your earnings are growing.",
  },
];

const Features = () => {
  return (
    <section className="relative py-32">
      <div className="container mx-auto px-6">
        <motion.div
          className="text-center mb-16"
          initial={{ opacity: 0, y: 20, filter: "blur(4px)" }}
          whileInView={{ opacity: 1, y: 0, filter: "blur(0px)" }}
          viewport={{ once: true, amount: 0.2 }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
        >
          <h2 className="text-3xl sm:text-4xl font-bold mb-4">
            Everything You Need,{" "}
            <span className="text-gradient-primary">Nothing You Don't</span>
          </h2>
          <p className="text-muted-foreground max-w-xl mx-auto">
            Built for artists who want to focus on making music — not paperwork.
          </p>
        </motion.div>

        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature, i) => (
            <motion.div
              key={feature.title}
              className="glass rounded-xl p-6 hover:border-primary/30 transition-colors group"
              initial={{ opacity: 0, y: 20, filter: "blur(4px)" }}
              whileInView={{ opacity: 1, y: 0, filter: "blur(0px)" }}
              viewport={{ once: true, amount: 0.2 }}
              transition={{ duration: 0.5, delay: i * 0.08, ease: [0.16, 1, 0.3, 1] }}
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
