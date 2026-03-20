import { motion } from "framer-motion";
import { Check } from "lucide-react";

const plans = [
  {
    name: "Starter",
    price: "Free",
    period: "",
    description: "Try it out — no credit card needed",
    features: [
      "Unlimited song uploads",
      "Release on 15+ platforms",
      "See your earnings in real time",
      "Basic stats & reports",
      "Email support",
    ],
    cta: "Sign Up Free",
    highlighted: false,
  },
  {
    name: "Pro",
    price: "$19",
    period: "/mo",
    description: "For artists ready to grow",
    features: [
      "Everything in Starter",
      "Release on 150+ platforms",
      "Faster releases (under 12 hours)",
      "Split royalties with up to 16 people",
      "Priority support",
      "Detailed listener insights",
    ],
    cta: "Try Pro Free for 14 Days",
    highlighted: true,
  },
  {
    name: "Label",
    price: "Custom",
    period: "",
    description: "For labels managing multiple artists",
    features: [
      "Everything in Pro",
      "Unlimited artist accounts",
      "Connect to your existing tools",
      "Full API access",
      "Your own branding",
      "Dedicated account manager",
    ],
    cta: "Talk to Us",
    highlighted: false,
  },
];

const Pricing = () => {
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
            Simple{" "}
            <span className="text-gradient-primary">Pricing</span>
          </h2>
          <p className="text-muted-foreground max-w-xl mx-auto">
            No hidden fees. No surprise charges. You always know what you're paying.
          </p>
        </motion.div>

        <div className="grid md:grid-cols-3 gap-6 max-w-5xl mx-auto">
          {plans.map((plan, i) => (
            <motion.div
              key={plan.name}
              className={`rounded-xl p-6 flex flex-col ${
                plan.highlighted
                  ? "border-2 border-primary/50 bg-primary/5 glow-primary"
                  : "glass"
              }`}
              initial={{ opacity: 0, y: 20, filter: "blur(4px)" }}
              whileInView={{ opacity: 1, y: 0, filter: "blur(0px)" }}
              viewport={{ once: true, amount: 0.2 }}
              transition={{ duration: 0.5, delay: i * 0.1, ease: [0.16, 1, 0.3, 1] }}
            >
              {plan.highlighted && (
                <div className="text-xs font-semibold text-primary mb-3 tracking-wider uppercase">
                  Most Popular
                </div>
              )}
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

              <button
                className={`w-full py-3 rounded-lg font-semibold text-sm transition-all active:scale-[0.97] ${
                  plan.highlighted
                    ? "bg-primary text-primary-foreground hover:brightness-110"
                    : "glass hover:bg-secondary"
                }`}
              >
                {plan.cta}
              </button>
            </motion.div>
          ))}
        </div>
      </div>
    </section>
  );
};

export default Pricing;
