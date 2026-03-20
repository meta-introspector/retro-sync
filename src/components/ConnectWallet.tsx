import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Wallet, Loader2, CheckCircle2, XCircle, Usb } from "lucide-react";

type Status = "idle" | "connecting" | "connected" | "error";

const ConnectWallet = () => {
  const [status, setStatus] = useState<Status>("idle");
  const [address, setAddress] = useState("");
  const [errorMsg, setErrorMsg] = useState("");

  const connect = async () => {
    setStatus("connecting");
    setErrorMsg("");

    try {
      const TransportWebUSB = (await import("@ledgerhq/hw-transport-webusb")).default;
      const Eth = (await import("@ledgerhq/hw-app-eth")).default;

      const transport = await TransportWebUSB.create();
      const eth = new Eth(transport);
      const result = await eth.getAddress("44'/60'/0'/0/0");
      const addr = result.address;

      setAddress(`${addr.slice(0, 6)}…${addr.slice(-4)}`);
      setStatus("connected");
      await transport.close();
    } catch (err: any) {
      console.error("Ledger connection failed:", err);
      if (err?.message?.includes("No device selected")) {
        setErrorMsg("No Ledger found. Plug it in and unlock it first.");
      } else if (err?.message?.includes("denied")) {
        setErrorMsg("Connection was cancelled. Try again.");
      } else {
        setErrorMsg("Couldn't connect. Make sure your Ledger is plugged in, unlocked, and the Ethereum app is open.");
      }
      setStatus("error");
    }
  };

  const disconnect = () => {
    setStatus("idle");
    setAddress("");
    setErrorMsg("");
  };

  return (
    <div className="relative">
      <AnimatePresence mode="wait">
        {status === "idle" && (
          <motion.button
            key="idle"
            onClick={connect}
            className="flex items-center gap-2 px-4 py-2 rounded-lg glass hover:bg-secondary text-sm font-semibold transition-all active:scale-[0.97]"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <Usb className="w-4 h-4 text-primary" />
            Connect Ledger
          </motion.button>
        )}

        {status === "connecting" && (
          <motion.button
            key="connecting"
            disabled
            className="flex items-center gap-2 px-4 py-2 rounded-lg glass text-sm font-semibold opacity-80 cursor-wait"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <Loader2 className="w-4 h-4 animate-spin text-primary" />
            Connecting…
          </motion.button>
        )}

        {status === "connected" && (
          <motion.button
            key="connected"
            onClick={disconnect}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-primary/10 border border-primary/30 text-sm font-semibold transition-all hover:bg-primary/20 active:scale-[0.97]"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <CheckCircle2 className="w-4 h-4 text-primary" />
            <span className="font-mono text-primary">{address}</span>
          </motion.button>
        )}

        {status === "error" && (
          <motion.button
            key="error"
            onClick={connect}
            className="flex items-center gap-2 px-4 py-2 rounded-lg glass border-destructive/30 border text-sm font-semibold transition-all hover:bg-secondary active:scale-[0.97]"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            title={errorMsg}
          >
            <XCircle className="w-4 h-4 text-destructive" />
            Retry
          </motion.button>
        )}
      </AnimatePresence>

      {/* Error tooltip */}
      <AnimatePresence>
        {status === "error" && errorMsg && (
          <motion.div
            className="absolute top-full right-0 mt-2 w-64 p-3 rounded-lg glass text-xs text-muted-foreground z-50"
            initial={{ opacity: 0, y: -4 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
          >
            {errorMsg}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export default ConnectWallet;
