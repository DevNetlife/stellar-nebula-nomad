import { create } from 'zustand';
import { StellarAccount } from '../types';

export type NetworkType = 'testnet' | 'public' | 'futurenet';

export interface WalletState {
  account: StellarAccount | null;
  balance: string;
  network: NetworkType;
  isConnecting: boolean;
  isAuthenticated: boolean;
  isLocked: boolean;
  biometricsEnabled: boolean;
  lastActiveAt: number;

  setAccount: (account: StellarAccount | null) => void;
  setBalance: (balance: string) => void;
  setNetwork: (network: NetworkType) => void;
  setIsConnecting: (isConnecting: boolean) => void;
  setAuthenticated: (isAuthenticated: boolean) => void;
  setIsLocked: (isLocked: boolean) => void;
  setBiometricsEnabled: (enabled: boolean) => void;
  recordActivity: () => void;
  reset: () => void;
}

const initialState = {
  account: null,
  balance: '0.0000000',
  network: 'testnet' as NetworkType,
  isConnecting: false,
  isAuthenticated: false,
  isLocked: false,
  biometricsEnabled: false,
  lastActiveAt: Date.now(),
};

export const useAppStore = create<WalletState>((set) => ({
  ...initialState,

  setAccount: (account) =>
    set({
      account,
      isAuthenticated: !!account,
      isLocked: false,
      lastActiveAt: Date.now(),
    }),

  setBalance: (balance) => set({ balance }),

  setNetwork: (network) => set({ network }),

  setIsConnecting: (isConnecting) => set({ isConnecting }),

  setAuthenticated: (isAuthenticated) => set({ isAuthenticated }),

  setIsLocked: (isLocked) => set({ isLocked }),

  setBiometricsEnabled: (biometricsEnabled) => set({ biometricsEnabled }),

  recordActivity: () => set({ lastActiveAt: Date.now(), isLocked: false }),

  reset: () => set(initialState),
}));
