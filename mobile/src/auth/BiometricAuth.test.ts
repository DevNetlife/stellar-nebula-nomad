import { BiometricAuth } from './BiometricAuth';
import { SecureStorage } from '../utils/SecureStorage';
import * as LocalAuthentication from 'expo-local-authentication';

jest.mock('expo-local-authentication', () => ({
  hasHardwareAsync: jest.fn(),
  isEnrolledAsync: jest.fn(),
  supportedAuthenticationTypesAsync: jest.fn(),
  authenticateAsync: jest.fn(),
  AuthenticationType: {
    FINGERPRINT: 1,
    FACIAL_RECOGNITION: 2,
    IRIS: 3,
  },
}));

jest.mock('expo-secure-store', () => {
  const store = new Map<string, string>();
  return {
    setItemAsync: jest.fn(async (key: string, value: string) => {
      store.set(key, value);
    }),
    getItemAsync: jest.fn(async (key: string) => {
      return store.get(key) || null;
    }),
    deleteItemAsync: jest.fn(async (key: string) => {
      store.delete(key);
    }),
    WHEN_UNLOCKED_THIS_DEVICE_ONLY: 1,
  };
});

describe('BiometricAuth', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('detects when biometrics are unsupported', async () => {
    (LocalAuthentication.hasHardwareAsync as jest.Mock).mockResolvedValue(false);

    const caps = await BiometricAuth.getCapabilities();
    expect(caps.isSupported).toBe(false);
    expect(caps.isEnrolled).toBe(false);
    expect(caps.primaryType).toBe('none');
  });

  it('detects enrolled fingerprint hardware', async () => {
    (LocalAuthentication.hasHardwareAsync as jest.Mock).mockResolvedValue(true);
    (LocalAuthentication.isEnrolledAsync as jest.Mock).mockResolvedValue(true);
    (LocalAuthentication.supportedAuthenticationTypesAsync as jest.Mock).mockResolvedValue([
      LocalAuthentication.AuthenticationType.FINGERPRINT,
    ]);

    const caps = await BiometricAuth.getCapabilities();
    expect(caps.isSupported).toBe(true);
    expect(caps.isEnrolled).toBe(true);
    expect(caps.primaryType).toBe('fingerprint');
  });

  it('detects facial recognition hardware', async () => {
    (LocalAuthentication.hasHardwareAsync as jest.Mock).mockResolvedValue(true);
    (LocalAuthentication.isEnrolledAsync as jest.Mock).mockResolvedValue(true);
    (LocalAuthentication.supportedAuthenticationTypesAsync as jest.Mock).mockResolvedValue([
      LocalAuthentication.AuthenticationType.FACIAL_RECOGNITION,
    ]);

    const caps = await BiometricAuth.getCapabilities();
    expect(caps.primaryType).toBe('face-id');
    expect(BiometricAuth.getBiometricLabel('face-id')).toBe('Face ID');
  });

  it('handles successful authentication', async () => {
    (LocalAuthentication.hasHardwareAsync as jest.Mock).mockResolvedValue(true);
    (LocalAuthentication.isEnrolledAsync as jest.Mock).mockResolvedValue(true);
    (LocalAuthentication.supportedAuthenticationTypesAsync as jest.Mock).mockResolvedValue([
      LocalAuthentication.AuthenticationType.FINGERPRINT,
    ]);
    (LocalAuthentication.authenticateAsync as jest.Mock).mockResolvedValue({ success: true });

    const result = await BiometricAuth.authenticate('Test auth');
    expect(result.success).toBe(true);
  });

  it('handles user cancellation without fallback requirement', async () => {
    (LocalAuthentication.hasHardwareAsync as jest.Mock).mockResolvedValue(true);
    (LocalAuthentication.isEnrolledAsync as jest.Mock).mockResolvedValue(true);
    (LocalAuthentication.supportedAuthenticationTypesAsync as jest.Mock).mockResolvedValue([
      LocalAuthentication.AuthenticationType.FINGERPRINT,
    ]);
    (LocalAuthentication.authenticateAsync as jest.Mock).mockResolvedValue({
      success: false,
      error: 'user_cancel',
    });

    const result = await BiometricAuth.authenticate('Test auth');
    expect(result.success).toBe(false);
    expect(result.requiresFallback).toBeFalsy();
  });

  it('stores and reads biometric enabled state via SecureStorage', async () => {
    await SecureStorage.setBiometricsEnabled(true);
    const enabled = await SecureStorage.isBiometricsEnabled();
    expect(enabled).toBe(true);

    await SecureStorage.setBiometricsEnabled(false);
    const disabled = await SecureStorage.isBiometricsEnabled();
    expect(disabled).toBe(false);
  });
});
