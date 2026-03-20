import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, Wallet, ShieldCheck, Music, Link2, CheckCircle2, AlertTriangle, ArrowRight, ArrowLeft, KeyRound } from "lucide-react";
import { CHAIN_INFO, type WalletState } from "@/types/wallet";

interface Props {
  wallet: WalletState;
  onClose: () => void;
}

type Step = "wallet" | "kyc" | "ipi" | "confirm";

const STEPS: { id: Step; label: string; icon: typeof Wallet }[] = [
  { id: "wallet", label: "Wallet", icon: Wallet },
  { id: "kyc", label: "Verify Identity", icon: ShieldCheck },
  { id: "ipi", label: "Link IPI", icon: Music },
  { id: "confirm", label: "Confirm", icon: Link2 },
];

const OnboardingWizard = ({ wallet, onClose }: Props) => {
  const [currentStep, setCurrentStep] = useState<Step>("wallet");
  const [ipiNumber, setIpiNumber] = useState("");
  const [ipiError, setIpiError] = useState("");
  const [kycConsent, setKycConsent] = useState(false);
  const [showRecovery, setShowRecovery] = useState(false);

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
    // IPI is a 9-11 digit number assigned by CISAC
    const clean = value.replace(/\D/g, "");
    if (clean.length < 9 || clean.length > 11) {
      setIpiError("IPI numbers are 9–11 digits. Check your performing rights organization for yours.");
      return false;
    }
    setIpiError("");
    return true;
  };

  const handleConfirm = () => {
    // In production this would call backend to store binding
    console.log("Binding confirmed:", {
      address: wallet.address,
      chain: wallet.chain,
      ipi: ipiNumber.replace(/\D/g, ""),
    });
    onClose();
  };

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center p-4">
      {/* Backdrop */}
      <motion.div
        className="absolute inset-0 bg-background/80 backdrop-blur-sm"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        onClick={onClose}
      />

      {/* Modal */}
      <motion.div
        className="relative w-full max-w-lg glass rounded-2xl overflow-hidden"
        initial={{ opacity: 0, scale: 0.95, y: 20 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        exit={{ opacity: 0, scale: 0.95, y: 20 }}
        transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-border">
          <h2 className="font-bold text-lg">Set Up Your Account</h2>
          <button onClick={onClose} className="p-1 rounded-lg hover:bg-secondary transition-colors active:scale-[0.95]">
            <X className="w-5 h-5 text-muted-foreground" />
          </button>
        </div>

        {/* Step indicators */}
        <div className="px-6 pt-5">
          <div className="flex items-center justify-between mb-8">
            {STEPS.map((step, i) => (
              <div key={step.id} className="flex items-center">
                <div className={`w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold transition-colors ${
                  i < stepIndex ? "bg-primary text-primary-foreground"
                  : i === stepIndex ? "bg-primary/20 border-2 border-primary text-primary"
                  : "bg-secondary text-muted-foreground"
                }`}>
                  {i < stepIndex ? <CheckCircle2 className="w-4 h-4" /> : i + 1}
                </div>
                {i < STEPS.length - 1 && (
                  <div className={`w-8 sm:w-16 h-px mx-1 transition-colors ${
                    i < stepIndex ? "bg-primary" : "bg-border"
                  }`} />
                )}
              </div>
            ))}
          </div>
        </div>

        {/* Step content */}
        <div className="px-6 pb-6 min-h-[280px]">
          <AnimatePresence mode="wait">
            {/* Step 1: Wallet confirmation */}
            {currentStep === "wallet" && (
              <motion.div key="wallet" initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -20 }} transition={{ duration: 0.2 }}>
                <h3 className="text-xl font-bold mb-2">Wallet Connected</h3>
                <p className="text-sm text-muted-foreground mb-6">
                  Your wallet will be linked to your artist profile. If you ever lose access, you can rebind to a new wallet.
                </p>

                <div className="glass rounded-xl p-4 mb-4">
                  <div className="flex items-center gap-3 mb-3">
                    <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center">
                      <Wallet className="w-5 h-5 text-primary" />
                    </div>
                    <div>
                      <div className="text-sm font-semibold">{CHAIN_INFO[wallet.chain!].name}</div>
                      <div className="text-xs font-mono text-muted-foreground break-all">{wallet.address}</div>
                    </div>
                  </div>
                </div>

                <button
                  onClick={() => setShowRecovery(!showRecovery)}
                  className="text-xs text-muted-foreground hover:text-primary transition-colors flex items-center gap-1"
                >
                  <KeyRound className="w-3 h-3" />
                  Already have an account? Recover wallet binding
                </button>

                {showRecovery && (
                  <motion.div
                    className="mt-3 p-4 glass rounded-xl"
                    initial={{ opacity: 0, height: 0 }}
                    animate={{ opacity: 1, height: "auto" }}
                  >
                    <p className="text-xs text-muted-foreground mb-3">
                      If you lost access to your old wallet, you can rebind your IPI to this new wallet.
                      You'll need to verify your identity again for security.
                    </p>
                    <input
                      type="text"
                      placeholder="Enter your IPI number"
                      className="w-full px-3 py-2 rounded-lg bg-background border border-input text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring mb-2"
                      maxLength={11}
                    />
                    <button className="w-full py-2 rounded-lg bg-accent/10 text-accent text-sm font-semibold hover:bg-accent/20 transition-colors active:scale-[0.97]">
                      Start Recovery
                    </button>
                  </motion.div>
                )}
              </motion.div>
            )}

            {/* Step 2: KYC */}
            {currentStep === "kyc" && (
              <motion.div key="kyc" initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -20 }} transition={{ duration: 0.2 }}>
                <h3 className="text-xl font-bold mb-2">Verify Your Identity</h3>
                <p className="text-sm text-muted-foreground mb-6">
                  We need to confirm who you are before you can receive royalty payments. This keeps your earnings safe.
                </p>

                <div className="space-y-4">
                  <div className="glass rounded-xl p-4">
                    <div className="flex items-start gap-3">
                      <ShieldCheck className="w-5 h-5 text-primary mt-0.5" />
                      <div>
                        <div className="text-sm font-semibold mb-1">What we need</div>
                        <ul className="text-xs text-muted-foreground space-y-1.5">
                          <li>• A government-issued ID (passport, driver's license)</li>
                          <li>• A selfie to match your photo</li>
                          <li>• Takes about 2 minutes</li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  <div className="glass rounded-xl p-4">
                    <div className="flex items-start gap-3">
                      <AlertTriangle className="w-5 h-5 text-accent mt-0.5" />
                      <div>
                        <div className="text-sm font-semibold mb-1">Why this matters</div>
                        <p className="text-xs text-muted-foreground">
                          KYC verification is required by law for financial transactions. It also protects your account — 
                          nobody can claim your royalties without passing this check.
                        </p>
                      </div>
                    </div>
                  </div>

                  <label className="flex items-start gap-3 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={kycConsent}
                      onChange={(e) => setKycConsent(e.target.checked)}
                      className="mt-1 rounded border-input"
                    />
                    <span className="text-xs text-muted-foreground">
                      I consent to identity verification. My data is processed securely and never shared with third parties beyond what's legally required.
                    </span>
                  </label>
                </div>
              </motion.div>
            )}

            {/* Step 3: IPI */}
            {currentStep === "ipi" && (
              <motion.div key="ipi" initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -20 }} transition={{ duration: 0.2 }}>
                <h3 className="text-xl font-bold mb-2">Link Your IPI Number</h3>
                <p className="text-sm text-muted-foreground mb-6">
                  Your IPI (Interested Party Information) number is your unique ID in the music industry.
                  It's how collecting societies know to pay you.
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
                    className="w-full px-4 py-3 rounded-lg bg-background border border-input text-sm font-mono placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                    maxLength={11}
                  />
                  {ipiError && (
                    <p className="text-xs text-destructive mt-2">{ipiError}</p>
                  )}
                </div>

                <div className="glass rounded-xl p-4">
                  <div className="text-sm font-semibold mb-2">Don't have an IPI number?</div>
                  <p className="text-xs text-muted-foreground mb-3">
                    Your IPI is assigned when you register with a performing rights organization (PRO) like ASCAP, BMI, PRS, or GEMA.
                  </p>
                  <div className="flex flex-wrap gap-2">
                    {["ASCAP", "BMI", "PRS", "SESAC", "GEMA", "SOCAN"].map((pro) => (
                      <span key={pro} className="px-2.5 py-1 rounded-md bg-secondary text-xs text-muted-foreground">
                        {pro}
                      </span>
                    ))}
                  </div>
                </div>
              </motion.div>
            )}

            {/* Step 4: Confirm */}
            {currentStep === "confirm" && (
              <motion.div key="confirm" initial={{ opacity: 0, x: 20 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -20 }} transition={{ duration: 0.2 }}>
                <h3 className="text-xl font-bold mb-2">Confirm Your Binding</h3>
                <p className="text-sm text-muted-foreground mb-6">
                  Review the details below. Once confirmed, your IPI will be linked to this wallet for receiving royalties.
                </p>

                <div className="space-y-3 mb-6">
                  <div className="glass rounded-xl p-4 flex items-center justify-between">
                    <div className="text-sm text-muted-foreground">Wallet</div>
                    <div className="text-sm font-mono font-semibold">{wallet.address.slice(0, 8)}…{wallet.address.slice(-6)}</div>
                  </div>
                  <div className="glass rounded-xl p-4 flex items-center justify-between">
                    <div className="text-sm text-muted-foreground">Network</div>
                    <div className="text-sm font-semibold">{CHAIN_INFO[wallet.chain!].name}</div>
                  </div>
                  <div className="glass rounded-xl p-4 flex items-center justify-between">
                    <div className="text-sm text-muted-foreground">IPI Number</div>
                    <div className="text-sm font-mono font-semibold">{ipiNumber || "—"}</div>
                  </div>
                  <div className="glass rounded-xl p-4 flex items-center justify-between">
                    <div className="text-sm text-muted-foreground">Identity</div>
                    <div className="text-sm font-semibold text-accent">KYC Pending</div>
                  </div>
                </div>

                <div className="glass rounded-xl p-4 mb-4">
                  <div className="flex items-start gap-3">
                    <KeyRound className="w-5 h-5 text-primary mt-0.5" />
                    <p className="text-xs text-muted-foreground">
                      <strong className="text-foreground">Lost your wallet?</strong> You can always rebind your IPI to a new wallet address 
                      by verifying your identity again. Your royalty history is never lost.
                    </p>
                  </div>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>

        {/* Footer navigation */}
        <div className="px-6 py-4 border-t border-border flex items-center justify-between">
          {stepIndex > 0 ? (
            <button
              onClick={back}
              className="flex items-center gap-2 px-4 py-2 rounded-lg glass text-sm font-semibold hover:bg-secondary transition-all active:scale-[0.97]"
            >
              <ArrowLeft className="w-4 h-4" /> Back
            </button>
          ) : (
            <div />
          )}

          {currentStep === "confirm" ? (
            <button
              onClick={handleConfirm}
              className="flex items-center gap-2 px-6 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-semibold hover:brightness-110 transition-all glow-primary active:scale-[0.97]"
            >
              <CheckCircle2 className="w-4 h-4" /> Confirm Binding
            </button>
          ) : (
            <button
              onClick={() => {
                if (currentStep === "ipi" && ipiNumber) {
                  if (!validateIpi(ipiNumber)) return;
                }
                if (currentStep === "kyc" && !kycConsent) return;
                next();
              }}
              disabled={currentStep === "kyc" && !kycConsent}
              className="flex items-center gap-2 px-6 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-semibold hover:brightness-110 transition-all active:scale-[0.97] disabled:opacity-40 disabled:cursor-not-allowed"
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
