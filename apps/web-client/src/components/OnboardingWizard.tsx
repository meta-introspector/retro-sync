import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, Wallet, ShieldCheck, Music, Link2, CheckCircle2, ArrowRight, ArrowLeft } from "lucide-react";
import { type WalletState } from "@/types/wallet";

interface Props {
  wallet: WalletState;
  onClose: () => void;
}

type Step = "wallet" | "ipi" | "kyc" | "confirm";

const STEPS: { id: Step; label: string; icon: any }[] = [
  { id: "wallet", label: "Wallet", icon: Wallet },
  { id: "ipi", label: "IPI Number", icon: Music },
  { id: "kyc", label: "Verify IPI & Identity", icon: ShieldCheck },
  { id: "confirm", label: "Confirm", icon: Link2 },
];

const OnboardingWizard = ({ wallet, onClose }: Props) => {
  const [currentStep, setCurrentStep] = useState<Step>("wallet");
  const [ipiNumber, setIpiNumber] = useState("");
  const [ipiError, setIpiError] = useState("");
  const [kycConsent, setKycConsent] = useState(false);

  const stepIndex = STEPS.findIndex((s) => s.id === currentStep);

  const next = () => {
    if (stepIndex < STEPS.length - 1) {
      setCurrentStep(STEPS[stepIndex + 1].id);
    }
  };

  const back = () => {
    if (stepIndex > 0) {
      setCurrentStep(STEPS[stepIndex - 1].id);
    }
  };

  const validateIpi = (value: string) => {
    const clean = value.replace(/\D/g, "");
    if (clean.length < 9 || clean.length > 11) {
      setIpiError("IPI numbers are 9–11 digits. Check your PRO for yours.");
      return false;
    }
    setIpiError("");
    return true;
  };

  const handleConfirm = () => {
    console.log("IPI & Wallet Binding confirmed via KYC:", {
      address: wallet.address,
      walletType: wallet.walletType,
      ipi: ipiNumber.replace(/\D/g, ""),
    });
    onClose();
  };

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center p-4">
      <motion.div
        className="absolute inset-0 bg-background/80 backdrop-blur-sm"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        onClick={onClose}
      />

      <motion.div
        className="relative w-full max-w-lg glass rounded-2xl overflow-hidden shadow-2xl"
        initial={{ opacity: 0, scale: 0.95, y: 20 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: 20 }}
        transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
      >
        <div className="flex items-center justify-between px-6 py-4 border-b border-zinc-800">
          <h2 className="font-bold text-lg">Verify Artist Identity</h2>
          <button onClick={onClose} className="p-1 rounded-lg hover:bg-zinc-800 transition-colors">
            <X className="w-5 h-5 text-zinc-500" />
          </button>
        </div>

        <div className="px-6 pt-5">
          <div className="flex items-center justify-between mb-8">
            {STEPS.map((step, i) => (
              <div key={step.id} className="flex items-center">
                <div className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold transition-colors ${
                  i < stepIndex ? "bg-primary text-primary-foreground"
                  : i === stepIndex ? "bg-primary/20 border-2 border-primary text-primary"
                  : "bg-zinc-800 text-zinc-500"
                }`}>
                  {i < stepIndex ? <CheckCircle2 className="w-4 h-4" /> : i + 1}
                </div>
                {i < STEPS.length - 1 && (
                  <div className={`w-8 sm:w-16 h-px mx-1 transition-colors ${
                    i < stepIndex ? "bg-primary" : "bg-zinc-800"
                  }`} />
                )}
              </div>
            ))}
          </div>
        </div>

        <div className="px-6 pb-6 min-h-[320px]">
          <AnimatePresence mode="wait">
            {currentStep === "wallet" && (
              <motion.div key="wallet" initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -20 }} transition={{ duration: 0.2 }}>
                <h3 className="text-xl font-bold mb-2">Connected Artist Wallet</h3>
                <p className="text-sm text-zinc-400 mb-6">
                  Your wallet address is your unique identifier. We do not use or store artist names.
                </p>

                <div className="bg-zinc-900/50 rounded-xl p-4 border border-zinc-800">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center">
                      <Wallet className="w-5 h-5 text-primary" />
                    </div>
                    <div>
                      <div className="text-sm font-semibold capitalize">{wallet.walletType} Wallet</div>
                      <div className="text-xs font-mono text-zinc-500 break-all">{wallet.address}</div>
                    </div>
                  </div>
                </div>
              </motion.div>
            )}

            {currentStep === "ipi" && (
              <motion.div key="ipi" initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -20 }} transition={{ duration: 0.2 }}>
                <h3 className="text-xl font-bold mb-2">IPI Number</h3>
                <p className="text-sm text-zinc-400 mb-6">
                  Enter your Interested Party Information (IPI) number. This links your identity to your musical works.
                </p>

                <div className="mb-4">
                  <label className="text-sm font-semibold mb-2 block">IPI Number</label>
                  <input
                    type="text"
                    value={ipiNumber}
                    onChange={(e) => {
                      const v = e.target.value.replace(/[^0-9]/g, "").slice(0, 11);
                      setIpiNumber(v);
                      if (ipiError) setIpiError("");
                    }}
                    placeholder="e.g. 00523879412"
                    className="w-full px-4 py-3 rounded-lg bg-zinc-950 border border-zinc-800 text-sm font-mono placeholder:text-zinc-600 focus:outline-none focus:ring-2 focus:ring-primary"
                    maxLength={11}
                  />
                  {ipiError && (
                    <p className="text-xs text-red-500 mt-2">{ipiError}</p>
                  )}
                </div>

                <div className="bg-zinc-900/50 rounded-xl p-4 border border-zinc-800">
                  <p className="text-xs text-zinc-500">
                    Your IPI is a unique 9-11 digit number assigned to you by your Performing Rights Organization (PRO).
                  </p>
                </div>
              </motion.div>
            )}

            {currentStep === "kyc" && (
              <motion.div key="kyc" initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -20 }} transition={{ duration: 0.2 }}>
                <h3 className="text-xl font-bold mb-2">Verify IPI Ownership</h3>
                <p className="text-sm text-zinc-400 mb-6">
                  We must verify that IPI <strong>{ipiNumber}</strong> legally belongs to the owner of this wallet.
                </p>

                <div className="space-y-4">
                  <div className="bg-zinc-900/50 rounded-xl p-4 border border-zinc-800">
                    <div className="flex items-start gap-3">
                      <ShieldCheck className="w-5 h-5 text-primary mt-0.5" />
                      <div>
                        <div className="text-sm font-semibold mb-1">Identity & IPI Link</div>
                        <p className="text-xs text-zinc-500 leading-relaxed">
                          This automated KYC process will verify your government ID against the IPI registration. 
                          This prevents unauthorized parties from claiming your royalties.
                        </p>
                      </div>
                    </div>
                  </div>

                  <label className="flex items-start gap-3 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={kycConsent}
                      onChange={(e) => setKycConsent(e.target.checked)}
                      className="mt-1 rounded border-zinc-800 bg-zinc-950 text-primary focus:ring-primary"
                    />
                    <span className="text-xs text-zinc-500">
                      I consent to identity verification to authorize this wallet for IPI royalty payouts.
                    </span>
                  </label>
                </div>
              </motion.div>
            )}

            {currentStep === "confirm" && (
              <motion.div key="confirm" initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -20 }} transition={{ duration: 0.2 }}>
                <h3 className="text-xl font-bold mb-2">Final Review</h3>
                <p className="text-sm text-zinc-400 mb-6">
                  Your verification is successful. Click below to finalize the link between your wallet and IPI.
                </p>

                <div className="space-y-3 mb-6">
                  <div className="bg-zinc-900/50 rounded-xl p-4 border border-zinc-800 flex items-center justify-between">
                    <div className="text-sm text-zinc-500">Artist ID (Wallet)</div>
                    <div className="text-sm font-mono text-zinc-300">{wallet.address.slice(0, 8)}…{wallet.address.slice(-6)}</div>
                  </div>
                  <div className="bg-zinc-900/50 rounded-xl p-4 border border-zinc-800 flex items-center justify-between">
                    <div className="text-sm text-zinc-500">IPI Number</div>
                    <div className="text-sm font-mono text-primary font-bold">{ipiNumber}</div>
                  </div>
                  <div className="bg-zinc-900/50 rounded-xl p-4 border border-zinc-800 flex items-center justify-between">
                    <div className="text-sm text-zinc-500">KYC Status</div>
                    <div className="text-sm font-bold text-green-500">Verified</div>
                  </div>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>

        <div className="px-6 py-4 border-t border-zinc-800 flex items-center justify-between">
          {stepIndex > 0 ? (
            <button
              onClick={back}
              className="flex items-center gap-2 px-4 py-2 rounded-lg glass text-sm font-semibold hover:bg-zinc-800 transition-all"
            >
              <ArrowLeft className="w-4 h-4" /> Back
            </button>
          ) : (
            <div />
          )}

          {currentStep === "confirm" ? (
            <button
              onClick={handleConfirm}
              className="flex items-center gap-2 px-6 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-bold hover:brightness-110 transition-all glow-primary"
            >
              <CheckCircle2 className="w-4 h-4" /> Finalize Binding
            </button>
          ) : (
            <button
              onClick={() => {
                if (currentStep === "ipi") {
                  if (!validateIpi(ipiNumber)) return;
                }
                if (currentStep === "kyc" && !kycConsent) return;
                next();
              }}
              disabled={currentStep === "kyc" && !kycConsent}
              className="flex items-center gap-2 px-6 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-bold hover:brightness-110 transition-all active:scale-[0.97] disabled:opacity-40 disabled:cursor-not-allowed"
            >
              Continue <ArrowRight className="w-4 h-4" />
            </button>
          )}
        </div>
      </motion.div>
    </div>
  );
};

export default OnboardingWizard;
