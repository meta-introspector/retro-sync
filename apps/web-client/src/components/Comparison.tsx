import { motion } from "framer-motion";
import { Check, X, Zap, Target, BarChart, Lock, UserX, Share2 } from "lucide-react";

const comparisonData = [
  { feature: "Annual Fee", legacy: "$20–$100/yr", retro: "$0 Forever", icon: Zap },
  { feature: "Identity", legacy: "Personal Info Required", retro: "Completely Private", icon: UserX },
  { feature: "Payment Speed", legacy: "60–90 Days", retro: "Instant", icon: Zap },
  { feature: "Payout Accuracy", legacy: "Unverified Estimates", retro: "ZK-Proven", icon: Target },
  { feature: "Audio Monitoring", legacy: "Basic Tools", retro: "Pro Spectrum", icon: BarChart },
  { feature: "Security", legacy: "Centralized", retro: "Global Encryption", icon: Lock },
  { feature: "Ownership", legacy: "Platform-Owned", retro: "Sovereign", icon: Share2 },
];

const Comparison = () => {
  return (
    <section className="py-24 md:py-32 relative bg-background overflow-hidden">
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-primary/3 blur-[120px] rounded-full pointer-events-none" />

      <div className="container mx-auto px-6 relative z-10">
        {/* Offset header */}
        <motion.div
          className="mb-16 md:mb-20 lg:ml-[8%]"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
        >
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold mb-4 tracking-tight">
            The <span className="text-gradient-primary">Upgrade</span>
          </h2>
          <p className="text-muted-foreground max-w-lg text-base">
            Built for the way music is made and sold today.
          </p>
        </motion.div>

        <div className="space-y-2 max-w-5xl mx-auto">
          {comparisonData.map((item, i) => (
            <motion.div
              key={item.feature}
              className="grid grid-cols-3 md:grid-cols-12 gap-px bg-border overflow-hidden"
              initial={{ opacity: 0, x: i % 2 === 0 ? -15 : 15 }}
              whileInView={{ opacity: 1, x: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.04 }}
            >
              {/* Feature */}
              <div className="col-span-1 md:col-span-4 bg-card p-4 md:p-5 flex items-center gap-3">
                <item.icon className="w-4 h-4 text-muted-foreground shrink-0 hidden sm:block" />
                <span className="font-medium text-sm text-foreground">{item.feature}</span>
              </div>

              {/* Legacy */}
              <div className="col-span-1 md:col-span-4 bg-card p-4 md:p-5 flex items-center gap-2">
                <X className="w-3.5 h-3.5 text-destructive/50 shrink-0" />
                <span className="text-sm text-muted-foreground">{item.legacy}</span>
              </div>

              {/* Retrosync */}
              <div className="col-span-1 md:col-span-4 bg-primary/5 p-4 md:p-5 flex items-center gap-2">
                <Check className="w-3.5 h-3.5 text-primary shrink-0" />
                <span className="text-sm font-medium text-primary">{item.retro}</span>
              </div>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Comparison;
