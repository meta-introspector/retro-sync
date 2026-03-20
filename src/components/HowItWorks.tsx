import { motion } from "framer-motion";
import { Upload, Send, DollarSign } from "lucide-react";

const steps = [
  {
    icon: Upload,
    step: "01",
    title: "Upload Your Track",
    description:
      "Drag and drop your song, add a title, artist name, and cover art. We'll take care of all the technical stuff.",
  },
  {
    icon: Send,
    step: "02",
    title: "We Send It Everywhere",
    description:
      "Your release goes out to Spotify, Apple Music, TikTok, YouTube, Amazon, and 150+ other stores and platforms.",
  },
  {
    icon: DollarSign,
    step: "03",
    title: "You Get Paid",
    description:
      "As your music earns royalties, you can see exactly how much you made. Split earnings with collaborators automatically.",
  },
];

const HowItWorks = () => {
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
            Three Steps.{" "}
            <span className="text-gradient-primary">That's It.</span>
          </h2>
          <p className="text-muted-foreground max-w-xl mx-auto">
            No contracts, no approval process, no waiting around.
          </p>
        </motion.div>

        <div className="grid md:grid-cols-3 gap-8 max-w-5xl mx-auto">
          {steps.map((s, i) => (
            <motion.div
              key={s.step}
              className="relative text-center"
              initial={{ opacity: 0, y: 24, filter: "blur(4px)" }}
              whileInView={{ opacity: 1, y: 0, filter: "blur(0px)" }}
              viewport={{ once: true, amount: 0.2 }}
              transition={{ duration: 0.5, delay: i * 0.12, ease: [0.16, 1, 0.3, 1] }}
            >
              {i < steps.length - 1 && (
                <div className="hidden md:block absolute top-12 left-[60%] w-[80%] h-px bg-border" />
              )}
              <div className="w-20 h-20 rounded-2xl bg-primary/10 border border-primary/20 flex items-center justify-center mx-auto mb-6 relative">
                <s.icon className="w-8 h-8 text-primary" />
                <span className="absolute -top-2 -right-2 w-7 h-7 rounded-full bg-accent text-accent-foreground text-xs font-bold flex items-center justify-center font-mono">
                  {s.step}
                </span>
              </div>
              <h3 className="text-xl font-semibold mb-3">{s.title}</h3>
              <p className="text-sm text-muted-foreground leading-relaxed max-w-xs mx-auto">
                {s.description}
              </p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default HowItWorks;
