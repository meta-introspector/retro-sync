import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Wallet, ChevronDown, ExternalLink, LogOut, Usb, Smartphone } from "lucide-react";
import { useWallet } from "@/hooks/useWallet";
import { CHAIN_INFO, type ChainId } from "@/types/wallet";
import OnboardingWizard from "./OnboardingWizard";

const ConnectWallet = () => {
  const { wallet, isConnecting, error, connectTronLink, connectWalletConnect, connectCoinbase, disconnect, shortenAddress, setError } = useWallet();
  const [menuOpen, setMenuOpen] = useState(false);
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [selectedChain, setSelectedChain] = useState<ChainId>("bttc");

  const handleConnect = async (type: "tronlink" | "walletconnect" | "coinbase") => {
    setMenuOpen(false);
    if (type === "tronlink") {
      await connectTronLink(selectedChain);
    } else if (type === "coinbase") {
      await connectCoinbase(selectedChain);
    } else {
      await connectWalletConnect(selectedChain);
    }
  };

  // After connecting, show onboarding wizard
  const handleStartOnboarding = () => {
    setShowOnboarding(true);
    setMenuOpen(false);
  };

  if (wallet.connected) {
    return (
      <>
        <div className="relative">
          <button
            onClick={() => setMenuOpen(!menuOpen)}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary/10 border border-primary/30 text-sm font-semibold transition-all hover:bg-primary/20 active:scale-[0.97]"
          >
            <div className="w-2 h-2 rounded-full bg-primary animate-pulse" />
            <span className="font-mono text-primary">{shortenAddress(wallet.address)}</span>
            <span className="text-xs text-muted-foreground">{CHAIN_INFO[wallet.chain!].symbol}</span>
            <ChevronDown className="w-3 h-3 text-muted-foreground" />
          </button>

          <AnimatePresence>
            {menuOpen && (
              <motion.div
                className="absolute top-full right-0 mt-2 w-56 glass rounded-xl p-2 z-50"
                initial={{ opacity: 0, y: -8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -8 }}
              >
                <div className="px-3 py-2 border-b border-border mb-1">
                  <div className="text-xs text-muted-foreground">Connected to</div>
                  <div className="text-sm font-semibold">{CHAIN_INFO[wallet.chain!].name}</div>
                  <div className="text-xs font-mono text-muted-foreground mt-1 break-all">{wallet.address}</div>
                </div>
                <button
                  onClick={handleStartOnboarding}
                  className="w-full text-left px-3 py-2 rounded-lg text-sm hover:bg-secondary transition-colors flex items-center gap-2"
                >
                  <Wallet className="w-4 h-4" />
                  Set Up Profile & IPI
                </button>
                <a
                  href={`${CHAIN_INFO[wallet.chain!].explorer}/address/${wallet.address}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="w-full text-left px-3 py-2 rounded-lg text-sm hover:bg-secondary transition-colors flex items-center gap-2 text-muted-foreground"
                >
                  <ExternalLink className="w-4 h-4" />
                  View on Explorer
                </a>
                <button
                  onClick={() => { disconnect(); setMenuOpen(false); }}
                  className="w-full text-left px-3 py-2 rounded-lg text-sm hover:bg-destructive/10 text-destructive transition-colors flex items-center gap-2"
                >
                  <LogOut className="w-4 h-4" />
                  Disconnect
                </button>
              </motion.div>
            )}
          </AnimatePresence>
        </div>

        {showOnboarding && (
          <OnboardingWizard
            wallet={wallet}
            onClose={() => setShowOnboarding(false)}
          />
        )}
      </>
    );
  }

  return (
    <div className="relative">
      <button
        onClick={() => setMenuOpen(!menuOpen)}
        disabled={isConnecting}
        className="flex items-center gap-2 px-4 py-2 rounded-lg glass hover:bg-secondary text-sm font-semibold transition-all active:scale-[0.97] disabled:opacity-60 disabled:cursor-wait"
      >
        {isConnecting ? (
          <>
            <div className="w-4 h-4 border-2 border-primary border-t-transparent rounded-full animate-spin" />
            Connecting…
          </>
        ) : (
          <>
            <Wallet className="w-4 h-4 text-primary" />
            Connect Wallet
            <ChevronDown className="w-3 h-3 text-muted-foreground" />
          </>
        )}
      </button>

      <AnimatePresence>
        {menuOpen && !isConnecting && (
          <motion.div
            className="absolute top-full right-0 mt-2 w-72 glass rounded-xl p-3 z-50"
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
          >
            {/* Chain selector */}
            <div className="mb-3">
              <div className="text-xs text-muted-foreground mb-2">Choose network</div>
              <div className="flex gap-2">
                {(["bttc", "tron"] as ChainId[]).map((c) => (
                  <button
                    key={c}
                    onClick={() => setSelectedChain(c)}
                    className={`flex-1 py-2 px-3 rounded-lg text-xs font-semibold transition-all active:scale-[0.97] ${
                      selectedChain === c
                        ? "bg-primary/15 border border-primary/40 text-primary"
                        : "glass hover:bg-secondary text-muted-foreground"
                    }`}
                  >
                    {CHAIN_INFO[c].name}
                  </button>
                ))}
              </div>
            </div>

            {/* Wallet options */}
            <div className="space-y-1.5">
              <button
                onClick={() => handleConnect("tronlink")}
                className="w-full flex items-center gap-3 px-3 py-3 rounded-lg hover:bg-secondary transition-colors active:scale-[0.98]"
              >
                <div className="w-9 h-9 rounded-lg bg-primary/10 flex items-center justify-center">
                  <Usb className="w-4 h-4 text-primary" />
                </div>
                <div className="text-left">
                  <div className="text-sm font-semibold">TronLink</div>
                  <div className="text-xs text-muted-foreground">Browser extension · Ledger supported</div>
                </div>
              </button>

              <button
                onClick={() => handleConnect("coinbase")}
                className="w-full flex items-center gap-3 px-3 py-3 rounded-lg hover:bg-secondary transition-colors active:scale-[0.98]"
              >
                <div className="w-9 h-9 rounded-lg bg-blue-500/10 flex items-center justify-center">
                  <div className="w-4 h-4 bg-blue-600 rounded-sm" />
                </div>
                <div className="text-left">
                  <div className="text-sm font-semibold">Coinbase Wallet</div>
                  <div className="text-xs text-muted-foreground">Direct connection · Fiat support</div>
                </div>
              </button>

              <button
                onClick={() => handleConnect("walletconnect")}
                className="w-full flex items-center gap-3 px-3 py-3 rounded-lg hover:bg-secondary transition-colors active:scale-[0.98]"
              >
                <div className="w-9 h-9 rounded-lg bg-accent/10 flex items-center justify-center">
                  <Smartphone className="w-4 h-4 text-accent" />
                </div>
                <div className="text-left">
                  <div className="text-sm font-semibold">WalletConnect</div>
                  <div className="text-xs text-muted-foreground">Mobile wallets · QR code</div>
                </div>
              </button>
            </div>

            {error && (
              <div className="mt-3 p-2.5 rounded-lg bg-destructive/10 border border-destructive/20 text-xs text-destructive">
                {error}
              </div>
            )}

            <div className="mt-3 pt-3 border-t border-border">
              <p className="text-[11px] text-muted-foreground leading-relaxed">
                Don't have a wallet?{" "}
                <a
                  href="https://www.tronlink.org/"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-primary hover:underline"
                >
                  Get TronLink →
                </a>
              </p>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export default ConnectWallet;
