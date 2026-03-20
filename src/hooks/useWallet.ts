import { useState, useCallback } from "react";
import type { WalletState, ChainId } from "@/types/wallet";

const INITIAL_STATE: WalletState = {
  connected: false,
  address: "",
  chain: null,
  walletType: null,
};

export function useWallet() {
  const [wallet, setWallet] = useState<WalletState>(INITIAL_STATE);
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState("");

  const connectTronLink = useCallback(async (chain: ChainId) => {
    setIsConnecting(true);
    setError("");

    try {
      // Check if TronLink is installed
      if (!window.tronLink && !window.tronWeb) {
        throw new Error(
          "TronLink is not installed. Please install the TronLink extension from tronlink.org"
        );
      }

      // Request account access
      if (window.tronLink) {
        await window.tronLink.request({ method: "tron_requestAccounts" });
      }

      // Wait briefly for tronWeb to initialize
      await new Promise((r) => setTimeout(r, 500));

      if (!window.tronWeb?.ready) {
        throw new Error(
          "TronLink is locked. Please unlock your wallet and try again."
        );
      }

      const address = window.tronWeb.defaultAddress.base58;
      if (!address) {
        throw new Error("No account found. Please create an account in TronLink first.");
      }

      setWallet({
        connected: true,
        address,
        chain,
        walletType: "tronlink",
      });
    } catch (err: any) {
      setError(err.message || "Failed to connect wallet.");
    } finally {
      setIsConnecting(false);
    }
  }, []);

  const connectWalletConnect = useCallback(async (_chain: ChainId) => {
    setError("WalletConnect support is coming soon. Please use TronLink for now.");
  }, []);

  const disconnect = useCallback(() => {
    setWallet(INITIAL_STATE);
    setError("");
  }, []);

  const shortenAddress = (addr: string) =>
    addr ? `${addr.slice(0, 6)}…${addr.slice(-4)}` : "";

  return {
    wallet,
    isConnecting,
    error,
    connectTronLink,
    connectWalletConnect,
    disconnect,
    shortenAddress,
    setError,
  };
}
