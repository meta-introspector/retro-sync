import { motion } from "framer-motion";
import { Check, Calculator, TrendingUp, Info } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";

const plans = [
  {
    name: "Artist",
    price: "$0",
    period: "",
    description: "Made by an artist, for artists. We never charge you.",
    features: [
      "Unlimited song uploads",
      "Keep 100% of your rights",
      "Release on 150+ platforms",
      "Real-time royalty tracking",
      "Mathematically proven payouts",
    ],
    cta: "Start Releasing",
    highlighted: true,
  },
  {
    name: "Protocol",
    price: "Nodes",
    period: "",
    description: "How the infrastructure is sustained",
    features: [
      "Global file seeding",
      "2.5% transaction fee on payouts",
      "No monthly subscriptions",
      "No per-release charges",
      "Enterprise-grade security",
    ],
    cta: "View Node Stats",
    highlighted: false,
  },
];

const Pricing = () => {
  const [streams, setStreams] = useState(100000);
  const revenue = streams * 0.004;
  const legacyFees = revenue * 0.15 + 20;
  const retroFees = revenue * 0.025;
  const savings = legacyFees - retroFees;

  return (
    <section className="relative py-24 md:py-32 bg-background">
      <div className="container mx-auto px-6">
        <motion.div
          className="mb-16 md:mb-20 max-w-xl"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
        >
          <span className="text-xs font-mono text-primary/70 tracking-widest uppercase mb-4 block">
            Economics
          </span>
          <h2 className="text-3xl sm:text-4xl md:text-5xl font-bold mb-4 tracking-tight">
            Our <span className="text-gradient-primary">Revenue Model</span>
          </h2>
          <p className="text-muted-foreground text-base">
            We don't charge artists. We only win when you do.
          </p>
        </motion.div>

        {/* Revenue Calculator — full-width editorial card */}
        <motion.div
          className="max-w-5xl mx-auto mb-20 p-6 md:p-10 border border-primary/20 bg-primary/[0.02] relative overflow-hidden"
          initial={{ opacity: 0, scale: 0.98 }}
          whileInView={{ opacity: 1, scale: 1 }}
          viewport={{ once: true }}
        >
          <div className="absolute top-0 right-0 p-8 opacity-5">
            <Calculator className="w-40 h-40 text-primary" />
          </div>

          <div className="relative z-10 grid md:grid-cols-2 gap-10 items-center">
            <div>
              <div className="flex items-center gap-2 mb-6">
                <TrendingUp className="w-5 h-5 text-primary" />
                <h3 className="text-lg font-bold">Revenue Calculator</h3>
              </div>

              <div className="space-y-6">
                <div>
                  <div className="flex justify-between mb-2">
                    <label className="text-sm font-medium text-foreground">Annual Streams</label>
                    <span className="text-primary font-mono font-bold text-sm">{streams.toLocaleString()}</span>
                  </div>
                  <input
                    type="range"
                    min="10000"
                    max="1000000"
                    step="10000"
                    value={streams}
                    onChange={(e) => setStreams(parseInt(e.target.value))}
                    className="w-full h-1.5 bg-secondary rounded-none appearance-none cursor-pointer accent-primary"
                  />
                </div>

                <div className="p-3 bg-secondary/50 border border-border flex items-start gap-3">
                  <Info className="w-4 h-4 text-muted-foreground mt-0.5 shrink-0" />
                  <p className="text-xs text-muted-foreground leading-relaxed">
                    Based on $0.004/stream average. Traditional platforms take ~15% + annual fees.
                  </p>
                </div>
              </div>
            </div>

            <div className="bg-card p-6 border border-border">
              <div className="space-y-4">
                <div className="flex justify-between items-center text-sm">
                  <span className="text-muted-foreground">Total Revenue</span>
                  <span className="text-foreground font-mono">${revenue.toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>
                </div>
                <div className="flex justify-between items-center text-sm">
                  <span className="text-muted-foreground">Traditional Fees</span>
                  <span className="text-destructive font-mono">-${legacyFees.toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>
                </div>
                <div className="flex justify-between items-center text-sm border-b border-border pb-4">
                  <span className="text-muted-foreground">Retrosync Fee (2.5%)</span>
                  <span className="text-primary font-mono">-${retroFees.toLocaleString(undefined, { minimumFractionDigits: 2 })}</span>
                </div>
                <div className="pt-2">
                  <div className="text-[10px] uppercase font-medium text-muted-foreground mb-1 tracking-wider">Annual Savings</div>
                  <div className="text-2xl md:text-3xl font-bold text-primary font-mono">
                    +${savings.toLocaleString(undefined, { minimumFractionDigits: 2 })}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </motion.div>

        {/* Plan cards */}
        <div className="grid md:grid-cols-2 gap-4 max-w-3xl mx-auto">
          {plans.map((plan, i) => (
            <motion.div
              key={plan.name}
              className={`p-6 md:p-8 flex flex-col ${
                plan.highlighted
                  ? "border-2 border-primary/50 bg-primary/[0.03] glow-primary"
                  : "border border-border bg-card"
              }`}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.1 }}
            >
              <h3 className="text-xl font-bold">{plan.name}</h3>
              <div className="mt-3 mb-1">
                <span className="text-3xl font-bold">{plan.price}</span>
                {plan.period && <span className="text-muted-foreground text-sm">{plan.period}</span>}
              </div>
              <p className="text-sm text-muted-foreground mb-6">{plan.description}</p>

              <ul className="space-y-3 mb-8 flex-1">
                {plan.features.map((f) => (
                  <li key={f} className="flex items-start gap-2 text-sm">
                    <Check className="w-4 h-4 text-primary mt-0.5 shrink-0" />
                    <span className="text-muted-foreground">{f}</span>
                  </li>
                ))}
              </ul>

              <Button
                variant={plan.highlighted ? "default" : "outline"}
                className={`w-full py-6 font-bold ${plan.highlighted ? "glow-primary" : ""}`}
              >
                {plan.cta}
              </Button>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Pricing;
