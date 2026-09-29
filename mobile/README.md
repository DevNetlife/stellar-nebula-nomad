# Stellar Nebula Nomad - Mobile Client

Mobile companion app for Stellar Nebula Nomad built with React Native and Expo.

## Features

- **Navigation**: Typed React Navigation setup featuring RootStack, AuthStack, and MainTabNavigator.
- **State Management**: Centralized state with Zustand for wallet account, balances, network preferences, and authentication.
- **Biometric Security**:
  - Secure hardware authentication via `expo-local-authentication` (Touch ID, Face ID, Iris).
  - Encrypted storage using `expo-secure-store` with iOS Keychain and Android Keystore hardware-backed security.
  - Automatic inactivity lock timer (5 minutes) with anti-tampering verification.
  - PIN fallback flow when biometrics are unavailable or locked out.
- **Stellar Horizon RPC**: Network client supporting testnet, public, and futurenet endpoints.

## Project Structure

```
mobile/
├── App.tsx                  # Main Expo application entry point
├── src/
│   ├── auth/                # Biometric authentication and lock manager
│   │   ├── BiometricAuth.ts
│   │   ├── BiometricAuth.test.ts
│   │   └── LockManager.ts
│   ├── components/          # Reusable UI components
│   │   └── BiometricSettings.tsx
│   ├── navigation/          # Navigation stacks and tabs
│   │   └── index.tsx
│   ├── screens/             # Application screens
│   │   ├── BiometricSetup.tsx
│   │   ├── HomeScreen.tsx
│   │   ├── LockScreen.tsx
│   │   ├── SettingsScreen.tsx
│   │   └── WalletScreen.tsx
│   ├── services/            # Stellar network and Soroban services
│   │   └── StellarService.ts
│   ├── store/               # Zustand application store
│   │   ├── index.ts
│   │   └── store.test.ts
│   ├── types/               # TypeScript definitions
│   │   └── index.ts
│   └── utils/               # Secure storage and helpers
│       └── SecureStorage.ts
```

## Running Tests

Run the test suite using Jest:

```bash
npm test
```
