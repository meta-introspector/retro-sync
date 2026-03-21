import { useState, useCallback } from "react";
import type { WalletState, ChainId } from "@/types/wallet";

const INITIAL_STATE: WalletState = {
  connected: false,
  address: "",
  chain: null,
  walletType: null,
};

const AUTH_TOKEN_KEY = "retrosync_auth_token";
const AUTH_ADDRESS_KEY = "retrosync_auth_address";

// ── Auth token helpers ────────────────────────────────────────────────────────

export function getAuthToken(): string | null {
  return sessionStorage.getItem(AUTH_TOKEN_KEY);
}

export function clearAuthToken(): void {
  sessionStorage.removeItem(AUTH_TOKEN_KEY);
  sessionStorage.removeItem(AUTH_ADDRESS_KEY);
}

function storeAuthToken(token: string, address: string): void {
  sessionStorage.setItem(AUTH_TOKEN_KEY, token);
  sessionStorage.setItem(AUTH_ADDRESS_KEY, address);
}

/** Return headers for authenticated API calls */
export function authHeaders(): Record<string, string> {
  const token = getAuthToken();
  return token ? { Authorization: `Bearer ${token}` } : {};
}

// ── Challenge-response authentication ────────────────────────────────────────

/**
 * Authenticate with the backend using a wallet signature challenge.
 *
 * Flow:
 *   1. Fetch a random nonce from GET /api/auth/challenge/{address}
 *   2. Sign the nonce with the connected wallet
 *   3. POST the signature to /api/auth/verify → receive JWT
 *   4. Store the JWT in sessionStorage for subsequent API calls
 *
 * Supports:
 *   - TronLink on BTTC (EVM): uses window.tronWeb.eth.personal.sign
 *   - TronLink on Tron mainnet: uses window.tronWeb.trx.signMessageV2
 *   - Any window.ethereum wallet (MetaMask, Coinbase): uses personal_sign
 */
async function authenticateWithServer(
  address: string,
  walletType: "tronlink" | "evm"
): Promise<string> {
  // Step 1: Get challenge nonce
  const challengeRes = await fetch(`/api/auth/challenge/${address.toLowerCase()}`);
  if (!challengeRes.ok) {
    throw new Error(`Challenge request failed: ${challengeRes.status}`);
  }
  const { challenge_id, nonce } = await challengeRes.json();

  // Step 2: Sign nonce with wallet
  let signature: string;
  if (walletType === "evm" && window.ethereum) {
    // EVM personal_sign (EIP-191): MetaMask, Coinbase, TronLink on BTTC
    signature = (await window.ethereum.request({
      method: "personal_sign",
      params: [nonce, address],
    })) as string;
  } else if (window.tronWeb?.trx?.signMessageV2) {
    // TronLink on Tron mainnet: signMessageV2
    signature = await window.tronWeb.trx.signMessageV2(nonce);
  } else if (window.tronWeb?.trx?.sign) {
    // Fallback: older TronLink sign API (browser-compatible hex encoding)
    const enc = new TextEncoder();
    const bytes = enc.encode(nonce);
    const hexMsg = "0x" + Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("");
    signature = await window.tronWeb.trx.sign(hexMsg);
  } else {
    throw new Error("No supported wallet signing method found.");
  }

  if (!signature) {
    throw new Error("Signing was cancelled or failed.");
  }

  // Step 3: Verify with backend
  const verifyRes = await fetch("/api/auth/verify", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ challenge_id, address: address.toLowerCase(), signature }),
  });

  if (!verifyRes.ok) {
    const text = await verifyRes.text().catch(() => "");
    throw new Error(`Signature verification failed (${verifyRes.status}): ${text}`);
  }

  const { token } = await verifyRes.json();
  if (!token) {
    throw new Error("Backend did not return an auth token.");
  }

  storeAuthToken(token, address);
  return token;
}

// ── Hook ──────────────────────────────────────────────────────────────────────

export function useWallet() {
  const [wallet, setWallet] = useState<WalletState>(INITIAL_STATE);
  const [isConnecting, setIsConnecting] = useState(false);
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [error, setError] = useState("");

  const connectTronLink = useCallback(async (chain: ChainId) => {
    setIsConnecting(true);
    setError("");

    try {
      if (!window.tronLink && !window.tronWeb) {
        throw new Error(
          "TronLink is not installed. Please install the TronLink extension from tronlink.org"
        );
      }

      if (window.tronLink) {
        await window.tronLink.request({ method: "tron_requestAccounts" });
      }

      // Wait briefly for tronWeb to initialise
      await new Promise((r) => setTimeout(r, 500));

      if (!window.tronWeb?.ready) {
        throw new Error(
          "TronLink is locked. Please unlock your wallet and try again."
        );
      }

      const address = window.tronWeb.defaultAddress.base58;
      if (!address) {
        throw new Error(
          "No account found. Please create an account in TronLink first."
        );
      }

      setWallet({ connected: true, address, chain, walletType: "tronlink" });

      // Authenticate with the backend (non-blocking — failures are non-fatal)
      setIsAuthenticating(true);
      try {
        const isEvm = chain === "bttc";
        await authenticateWithServer(address, isEvm ? "evm" : "tronlink");
      } catch (authErr) {
        console.warn("Backend auth failed (API calls may be limited):", authErr);
      } finally {
        setIsAuthenticating(false);
      }
    } catch (err: unknown) {
      const message =
        err instanceof Error ? err.message : "Failed to connect wallet.";
      setError(message);
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
    clearAuthToken();
  }, []);

  const shortenAddress = (addr: string) =>
    addr ? `${addr.slice(0, 6)}\u2026${addr.slice(-4)}` : "";

  const connectCoinbase = useCallback(async (_chain: ChainId) => {
    // SECURITY FIX: Removed hardcoded stub address "0xCB0000...0001" that was
    // shared by ALL users, causing identity confusion and financial fraud.
    // Coinbase Wallet SDK integration is required before enabling this flow.
    setError(
      "Coinbase Wallet integration is being configured. Please use TronLink for now."
    );
  }, []);

  return {
    wallet,
    isConnecting,
    isAuthenticating,
    error,
    connectTronLink,
    connectWalletConnect,
    connectCoinbase,
    disconnect,
    shortenAddress,
    setError,
    getAuthToken,
    authHeaders,
  };
}
