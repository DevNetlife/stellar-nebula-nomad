export type RootStackParamList = {
  AuthStack: undefined;
  MainTabs: undefined;
};

export type AuthStackParamList = {
  Onboarding: undefined;
  BiometricSetup: undefined;
  PinSetup: { fromBiometricFallback?: boolean };
  PinEntry: { reason?: string };
};

export type MainTabsParamList = {
  Fleet: undefined;
  Market: undefined;
  Explore: undefined;
  Guild: undefined;
  Settings: undefined;
};

export type SettingsStackParamList = {
  SettingsHome: undefined;
  BiometricSettings: undefined;
  SecuritySettings: undefined;
};

export interface StellarAccount {
  publicKey: string;
  encryptedSecret?: string;
}

export interface AppState {
  account: StellarAccount | null;
  isAuthenticated: boolean;
  isLocked: boolean;
  biometricsEnabled: boolean;
  lastActiveAt: number;
}
