import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { NativeStackScreenProps } from '@react-navigation/native-stack';
import { BiometricAuth, BiometricCapabilities } from '../auth/BiometricAuth';
import { SecureStorage } from '../utils/SecureStorage';
import { AuthStackParamList } from '../types';

type Props = NativeStackScreenProps<AuthStackParamList, 'BiometricSetup'>;

export default function BiometricSetupScreen({ navigation }: Props) {
  const [capabilities, setCapabilities] = useState<BiometricCapabilities | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isEnabling, setIsEnabling] = useState(false);

  useEffect(() => {
    BiometricAuth.getCapabilities().then((caps) => {
      setCapabilities(caps);
      setIsLoading(false);
    });
  }, []);

  const handleEnable = async () => {
    setIsEnabling(true);
    try {
      const enabled = await BiometricAuth.enable();
      if (enabled) {
        navigation.replace('MainTabs' as any);
      } else {
        Alert.alert(
          'Verification Failed',
          'Could not verify your biometrics. Please set up a PIN instead.',
          [
            {
              text: 'Set Up PIN',
              onPress: () => navigation.replace('PinSetup', { fromBiometricFallback: true }),
            },
          ]
        );
      }
    } finally {
      setIsEnabling(false);
    }
  };

  const handleSkip = async () => {
    await SecureStorage.setBiometricsEnabled(false);
    navigation.replace('PinSetup', {});
  };

  if (isLoading || !capabilities) {
    return (
      <View style={styles.container}>
        <ActivityIndicator size="large" color="#38BDF8" />
      </View>
    );
  }

  if (!capabilities.isSupported || !capabilities.isEnrolled) {
    return (
      <View style={styles.container}>
        <Text style={styles.title}>Biometrics Unavailable</Text>
        <Text style={styles.subtitle}>
          {!capabilities.isSupported
            ? 'Your device does not support biometric authentication.'
            : 'No biometrics are enrolled on this device. You can add them in device Settings.'}
        </Text>
        <TouchableOpacity style={styles.primaryButton} onPress={handleSkip}>
          <Text style={styles.primaryButtonText}>Continue with PIN</Text>
        </TouchableOpacity>
      </View>
    );
  }

  const label = BiometricAuth.getBiometricLabel(capabilities.primaryType);

  return (
    <View style={styles.container}>
      <Text style={styles.iconLabel}>
        {capabilities.primaryType === 'face-id' ? '' : ''}
      </Text>
      <Text style={styles.title}>Enable {label}</Text>
      <Text style={styles.subtitle}>
        Use {label} for fast and secure access to your Stellar wallet. Your biometric data
        never leaves the secure enclave on your device.
      </Text>

      <TouchableOpacity
        style={[styles.primaryButton, isEnabling && styles.disabled]}
        onPress={handleEnable}
        disabled={isEnabling}
        accessibilityRole="button"
        accessibilityLabel={`Enable ${label}`}
      >
        {isEnabling ? (
          <ActivityIndicator color="#FFF" />
        ) : (
          <Text style={styles.primaryButtonText}>Enable {label}</Text>
        )}
      </TouchableOpacity>

      <TouchableOpacity
        style={styles.secondaryButton}
        onPress={handleSkip}
        accessibilityRole="button"
        accessibilityLabel="Skip and use PIN"
      >
        <Text style={styles.secondaryButtonText}>Skip, use PIN instead</Text>
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0B1020',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 32,
  },
  iconLabel: {
    fontSize: 64,
    marginBottom: 24,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#F8FAFC',
    textAlign: 'center',
    marginBottom: 12,
  },
  subtitle: {
    fontSize: 15,
    color: '#94A3B8',
    textAlign: 'center',
    lineHeight: 22,
    marginBottom: 40,
  },
  primaryButton: {
    backgroundColor: '#3B82F6',
    paddingVertical: 14,
    paddingHorizontal: 32,
    borderRadius: 10,
    width: '100%',
    alignItems: 'center',
    marginBottom: 12,
  },
  disabled: {
    opacity: 0.6,
  },
  primaryButtonText: {
    color: '#FFFFFF',
    fontSize: 16,
    fontWeight: '700',
  },
  secondaryButton: {
    paddingVertical: 12,
    alignItems: 'center',
  },
  secondaryButtonText: {
    color: '#64748B',
    fontSize: 14,
  },
});
