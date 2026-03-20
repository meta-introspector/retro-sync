// Tron/BTTC wallet types for window.tronWeb
export interface TronWebInstance {
  ready: boolean;
  defaultAddress: {
    base58: string;
    hex: string;
  };
  fullNode: { host: string };
  solidityNode: { host: string };
  eventServer: { host: string };
  trx: {
    getBalance: (address: string) => Promise<number>;
    getAccount: (address: string) => Promise<any>;
  };
}

export type ChainId = "bttc" | "tron";

export interface WalletState {
  connected: boolean;
  address: string;
  chain: ChainId | null;
  walletType: "tronlink" | "walletconnect" | null;
}

export interface OnboardingData {
  wallet: WalletState;
  ipiNumber: string;
  kycStatus: "pending" | "submitted" | "verified" | "failed";
  bindingConfirmed: boolean;
}

export const CHAIN_INFO: Record<ChainId, { name: string; symbol: string; explorer: string }> = {
  bttc: {
    name: "BitTorrent Chain",
    symbol: "BTT",
    explorer: "https://bttcscan.com",
  },
  tron: {
    name: "Tron",
    symbol: "TRX",
    explorer: "https://tronscan.org",
  },
};

// Extend Window for TronLink
declare global {
  interface Window {
    tronWeb?: TronWebInstance;
    tronLink?: {
      ready: boolean;
      request: (args: { method: string }) => Promise<any>;
    };
  }
}
