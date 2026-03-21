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

  // Simple calculation for demo: $0.004 per stream average
  const revenue = streams * 0.004;
  const legacyFees = revenue * 0.15 + 20; // 15% cut + $20 annual fee
  const retroFees = revenue * 0.025; // 2.5% protocol fee
  const savings = legacyFees - retroFees;

  return (
    <section className="relative py-32 bg-zinc-950">
      <div className="container mx-auto px-6">
        <motion.div
          className="text-center mb-16"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
        >
          <div className="inline-block px-3 py-1 rounded-full bg-primary/10 border border-primary/20 text-primary text-[10px] font-bold uppercase tracking-widest mb-4">
            Sustainable Economics
          </div>
          <h2 className="text-4xl sm:text-5xl font-bold mb-4 tracking-tight">
            Our <span className="text-gradient-primary">Revenue Model</span>
          </h2>
          <p className="text-zinc-400 max-w-xl mx-auto text-lg">
            We don't believe in charging artists. We only win when you do.
          </p>
        </motion.div>

        {/* Revenue Calculator */}
        <motion.div 
          className="max-w-4xl mx-auto mb-20 glass rounded-3xl p-8 border border-primary/20 bg-primary/5 relative overflow-hidden"
          initial={{ opacity: 0, scale: 0.95 }}
          whileInView={{ opacity: 1, scale: 1 }}
          viewport={{ once: true }}
        >
          <div className="absolute top-0 right-0 p-8 opacity-10">
            <Calculator className="w-32 h-32 text-primary" />
          </div>

          <div className="relative z-10 grid md:grid-cols-2 gap-12 items-center">
            <div>
              <div className="flex items-center gap-2 mb-6">
                <TrendingUp className="w-5 h-5 text-primary" />
                <h3 className="text-xl font-bold">Artist Revenue Calculator</h3>
              </div>
              
              <div className="space-y-6">
                <div>
                  <div className="flex justify-between mb-2">
                    <label className="text-sm font-medium text-zinc-300">Estimated Annual Streams</label>
                    <span className="text-primary font-mono font-bold">{streams.toLocaleString()}</span>
                  </div>
                  <input 
                    type="range" 
                    min="10000" 
                    max="1000000" 
                    step="10000"
                    value={streams}
                    onChange={(e) => setStreams(parseInt(e.target.value))}
                    className="w-full h-2 bg-zinc-800 rounded-lg appearance-none cursor-pointer accent-primary"
                  />
                </div>

                <div className="p-4 bg-zinc-900/50 rounded-xl border border-zinc-800 flex items-start gap-3">
                  <Info className="w-4 h-4 text-zinc-500 mt-0.5" />
                  <p className="text-[11px] text-zinc-500 leading-relaxed">
                    Calculation based on a global average of $0.004 per stream. Traditional platforms often take a 15% commission plus annual subscription fees.
                  </p>
                </div>
              </div>
            </div>

            <div className="bg-zinc-950/50 rounded-2xl p-6 border border-primary/10">
              <div className="space-y-4">
                <div className="flex justify-between items-center text-sm">
                  <span className="text-zinc-400">Total Revenue</span>
                  <span className="text-zinc-200 font-mono">${revenue.toLocaleString(undefined, {minimumFractionDigits: 2})}</span>
                </div>
                <div className="flex justify-between items-center text-sm">
                  <span className="text-zinc-400">Traditional Platform Fees</span>
                  <span className="text-red-500 font-mono">-${legacyFees.toLocaleString(undefined, {minimumFractionDigits: 2})}</span>
                </div>
                <div className="flex justify-between items-center text-sm border-b border-zinc-800 pb-4">
                  <span className="text-zinc-400">Retrosync Protocol Fee (2.5%)</span>
                  <span className="text-primary font-mono">-${retroFees.toLocaleString(undefined, {minimumFractionDigits: 2})}</span>
                </div>
                <div className="pt-2">
                  <div className="text-[10px] uppercase font-bold text-zinc-500 mb-1">Annual Savings</div>
                  <div className="text-3xl font-bold text-green-500 font-mono">+${savings.toLocaleString(undefined, {minimumFractionDigits: 2})}</div>
                </div>
              </div>
            </div>
          </div>
        </motion.div>

        <div className="grid md:grid-cols-2 gap-6 max-w-3xl mx-auto">
          {plans.map((plan, i) => (
            <motion.div
              key={plan.name}
              className={`rounded-xl p-6 flex flex-col ${
                plan.highlighted
                  ? "border-2 border-primary/50 bg-primary/5 glow-primary"
                  : "glass"
              }`}
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ delay: i * 0.1 }}
            >
              <h3 className="text-xl font-bold">{plan.name}</h3>
              <div className="mt-3 mb-1">
                <span className="text-4xl font-bold">{plan.price}</span>
                {plan.period && (
                  <span className="text-muted-foreground text-sm">{plan.period}</span>
                )}
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
                className={`w-full py-6 font-bold ${plan.highlighted ? "glow-primary" : "glass"}`}
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
