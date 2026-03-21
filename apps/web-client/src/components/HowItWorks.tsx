import { motion } from "framer-motion";
import { Upload, Cpu, Globe, CheckCircle } from "lucide-react";

const steps = [
  {
    icon: Upload,
    title: "1. Upload Song Metadata",
    description: "Submit your song details anonymously—no legal names needed.",
  },
  {
    icon: Cpu,
    title: "2. Verify Ownership",
    description: "Our secure system automatically creates a digital record of your music.",
  },
  {
    icon: Globe,
    title: "3. Distribute Globally",
    description: "Your music is instantly sent to 150+ stores like Spotify and Apple Music.",
  },
  {
    icon: CheckCircle,
    title: "4. Get Paid Instantly",
    description: "Your earnings are available immediately, without the traditional wait times.",
  },
];

const HowItWorks = () => {
  return (
    <section className="py-32 relative overflow-hidden bg-zinc-950">
      <div className="container mx-auto px-6">
        <motion.div
          className="text-center mb-20"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ duration: 0.6 }}
        >
          <h2 className="text-3xl sm:text-5xl font-bold mb-6 tracking-tight">How It <span className="text-gradient-primary">Works</span></h2>
          <p className="text-zinc-400 max-w-2xl mx-auto text-lg">
            We've automated every part of the distribution process so you can stay creative.
          </p>
        </motion.div>

        <div className="grid md:grid-cols-4 gap-8 relative">
          <div className="hidden md:block absolute top-12 left-0 w-full h-px bg-gradient-to-r from-transparent via-zinc-800 to-transparent z-0" />
          
          {steps.map((step, i) => (
            <motion.div
              key={step.title}
              className="relative z-10 flex flex-col items-center text-center group"
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.5, delay: i * 0.1 }}
            >
              <div className="w-20 h-20 rounded-2xl bg-zinc-900 border border-zinc-800 flex items-center justify-center mb-6 group-hover:border-primary/50 group-hover:bg-primary/5 transition-all duration-500 shadow-2xl">
                <step.icon className="w-8 h-8 text-primary" />
              </div>
              <h3 className="text-lg font-bold mb-3">{step.title}</h3>
              <p className="text-sm text-zinc-400 leading-relaxed px-4">{step.description}</p>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default HowItWorks;
