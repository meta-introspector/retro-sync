import { motion } from "framer-motion";
import { Upload, Cpu, Globe, CheckCircle } from "lucide-react";

const steps = [
  { icon: Upload, title: "Upload Metadata", description: "Submit song details anonymously — no legal names needed." },
  { icon: Cpu, title: "Verify Ownership", description: "Automatic digital provenance record for your music." },
  { icon: Globe, title: "Distribute Globally", description: "Instantly delivered to 150+ stores worldwide." },
  { icon: CheckCircle, title: "Get Paid Instantly", description: "Earnings available immediately, no waiting." },
];

const HowItWorks = () => {
  return (
    <section className="py-24 md:py-32 relative overflow-hidden bg-card">
      <div className="container mx-auto px-6">
        <motion.div
          className="mb-16 md:mb-20 max-w-lg lg:ml-auto lg:mr-[10%] lg:text-right"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
        >
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold mb-4 tracking-tight">
            How It <span className="text-gradient-primary">Works</span>
          </h2>
          <p className="text-muted-foreground text-base">
            Every part of distribution automated so you stay creative.
          </p>
        </motion.div>

        {/* Staggered two-column layout */}
        <div className="grid sm:grid-cols-2 gap-6 lg:gap-8 max-w-4xl mx-auto">
          {steps.map((step, i) => (
            <motion.div
              key={step.title}
              className={`p-6 md:p-8 bg-background border border-border hover:border-primary/30 transition-all group ${
                i % 2 !== 0 ? "sm:mt-12" : ""
              }`}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.1 }}
            >
              <div className="flex items-start gap-4">
                <div className="w-12 h-12 shrink-0 bg-secondary border border-border flex items-center justify-center group-hover:border-primary/50 transition-colors">
                  <step.icon className="w-5 h-5 text-primary" />
                </div>
                <div>
                  <div className="text-xs font-mono text-muted-foreground mb-1">Step {i + 1}</div>
                  <h3 className="text-lg font-bold mb-2 tracking-tight">{step.title}</h3>
                  <p className="text-sm text-muted-foreground leading-relaxed">{step.description}</p>
                </div>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default HowItWorks;
