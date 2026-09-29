import * as LocalAuthentication from 'expo-local-authentication';
import { SecureStorage } from '../utils/SecureStorage';

export type BiometricType = 'fingerprint' | 'face-id' | 'iris' | 'none';

export interface BiometricCapabilities {
  isSupported: boolean;
  isEnrolled: boolean;
  availableTypes: BiometricType[];
  primaryType: BiometricType;
}

export interface BiometricAuthResult {
  success: boolean;
  error?: string;
  requiresFallback?: boolean;
}

export class BiometricAuth {
  public static async getCapabilities(): Promise<BiometricCapabilities> {
    const isSupported = await LocalAuthentication.hasHardwareAsync();

    if (!isSupported) {
      return {
        isSupported: false,
        isEnrolled: false,
        availableTypes: [],
        primaryType: 'none',
      };
    }

    const isEnrolled = await LocalAuthentication.isEnrolledAsync();
    const rawTypes = await LocalAuthentication.supportedAuthenticationTypesAsync();

    const availableTypes: BiometricType[] = rawTypes.map((t) => {
      if (t === LocalAuthentication.AuthenticationType.FINGERPRINT) return 'fingerprint';
      if (t === LocalAuthentication.AuthenticationType.FACIAL_RECOGNITION) return 'face-id';
      if (t === LocalAuthentication.AuthenticationType.IRIS) return 'iris';
      return 'fingerprint';
    });

    const primaryType = availableTypes[0] || 'none';

    return { isSupported, isEnrolled, availableTypes, primaryType };
  }

  public static async authenticate(reason: string): Promise<BiometricAuthResult> {
    const capabilities = await this.getCapabilities();

    if (!capabilities.isSupported || !capabilities.isEnrolled) {
      return {
        success: false,
        error: 'Biometric authentication is not available on this device',
        requiresFallback: true,
      };
    }

    try {
      const result = await LocalAuthentication.authenticateAsync({
        promptMessage: reason,
        fallbackLabel: 'Use PIN',
        disableDeviceFallback: false,
        cancelLabel: 'Cancel',
      });

      if (result.success) {
        return { success: true };
      }

      if (result.error === 'user_cancel') {
        return { success: false, error: 'Authentication cancelled' };
      }

      if (
        result.error === 'lockout' ||
        result.error === 'lockout_permanent'
      ) {
        return {
          success: false,
          error: 'Too many failed attempts. Use your PIN.',
          requiresFallback: true,
        };
      }

      return {
        success: false,
        error: result.error || 'Authentication failed',
        requiresFallback: true,
      };
    } catch (err: any) {
      return {
        success: false,
        error: err?.message || 'Unexpected authentication error',
        requiresFallback: true,
      };
    }
  }

  public static async enable(): Promise<boolean> {
    const result = await this.authenticate('Verify your identity to enable biometric login');
    if (result.success) {
      await SecureStorage.setBiometricsEnabled(true);
      return true;
    }
    return false;
  }

  public static async disable(): Promise<void> {
    await SecureStorage.setBiometricsEnabled(false);
  }

  public static async isEnabled(): Promise<boolean> {
    return SecureStorage.isBiometricsEnabled();
  }

  public static getBiometricLabel(primaryType: BiometricType): string {
    switch (primaryType) {
      case 'face-id':
        return 'Face ID';
      case 'fingerprint':
        return 'Fingerprint';
      case 'iris':
        return 'Iris Scan';
      default:
        return 'Biometrics';
    }
  }
}
