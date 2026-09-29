import React, { useCallback, useEffect, useRef, useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { NativeStackScreenProps } from '@react-navigation/native-stack';
import { BiometricAuth } from '../auth/BiometricAuth';
import { LockManager, AntiTamper } from '../auth/LockManager';
import { SecureStorage } from '../utils/SecureStorage';
import { AuthStackParamList } from '../types';

type Props = NativeStackScreenProps<AuthStackParamList, 'PinEntry'>;

export default function LockScreen({ navigation, route }: Props) {
  const [pin, setPin] = useState('');
  const [isBiometricEnabled, setIsBiometricEnabled] = useState(false);
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [failedAttempts, setFailedAttempts] = useState(0);
  const isMounted = useRef(true);
  const reason = route.params?.reason || 'Unlock your Stellar Nebula Nomad wallet';

  useEffect(() => {
    return () => { isMounted.current = false; };
  }, []);

  useEffect(() => {
    AntiTamper.detectTampering().then(({ tampered, reason: tamperReason }) => {
      if (tampered) {
        Alert.alert(
          'Security Warning',
          `Device integrity check failed: ${tamperReason}. Access is blocked.`,
          [{ text: 'OK' }]
        );
      }
    });

    SecureStorage.isBiometricsEnabled().then((enabled) => {
      if (isMounted.current) setIsBiometricEnabled(enabled);
      if (enabled) attemptBiometric();
    });
  }, []);

  const attemptBiometric = useCallback(async () => {
    if (isAuthenticating) return;
    setIsAuthenticating(true);
    const result = await BiometricAuth.authenticate(reason);
    if (!isMounted.current) return;

    setIsAuthenticating(false);

    if (result.success) {
      LockManager.recordActivity();
      navigation.replace('MainTabs' as any);
    } else if (!result.requiresFallback) {
      Alert.alert('Authentication Failed', result.error || 'Please try again.');
    }
  }, [isAuthenticating, navigation, reason]);

  const handlePinDigit = (digit: string) => {
    if (pin.length >= 6) return;
    const next = pin + digit;
    setPin(next);
    if (next.length === 6) {
      verifyPin(next);
    }
  };

  const verifyPin = async (entered: string) => {
    const storedHash = await SecureStorage.getPinHash();
    const enteredHash = await hashPin(entered);

    if (enteredHash === storedHash) {
      LockManager.recordActivity();
      navigation.replace('MainTabs' as any);
    } else {
      const next = failedAttempts + 1;
      setFailedAttempts(next);
      setPin('');

      if (next >= 5) {
        Alert.alert('Too Many Attempts', 'You have been temporarily locked out.');
      } else {
        Alert.alert('Incorrect PIN', `${5 - next} attempts remaining.`);
      }
    }
  };

  const digits = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '', '0', '<'];

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Unlock Wallet</Text>
      <Text style={styles.subtitle}>{reason}</Text>

      <View style={styles.pinDots}>
        {Array.from({ length: 6 }).map((_, i) => (
          <View
            key={i}
            style={[styles.dot, i < pin.length && styles.dotFilled]}
            accessibilityLabel={i < pin.length ? 'filled' : 'empty'}
          />
        ))}
      </View>

      <View style={styles.keypad}>
        {digits.map((d, i) => {
          if (d === '') return <View key={i} style={styles.keyEmpty} />;
          return (
            <TouchableOpacity
              key={i}
              style={styles.key}
              onPress={() => {
                if (d === '<') setPin((p) => p.slice(0, -1));
                else handlePinDigit(d);
              }}
              accessibilityLabel={d === '<' ? 'Delete' : d}
              accessibilityRole="button"
            >
              <Text style={styles.keyText}>{d}</Text>
            </TouchableOpacity>
          );
        })}
      </View>

      {isBiometricEnabled && (
        <TouchableOpacity
          style={styles.biometricButton}
          onPress={attemptBiometric}
          disabled={isAuthenticating}
        >
          {isAuthenticating ? (
            <ActivityIndicator color="#38BDF8" />
          ) : (
            <Text style={styles.biometricText}>Use Biometrics</Text>
          )}
        </TouchableOpacity>
      )}
    </View>
  );
}

async function hashPin(pin: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(`stellar_pin_${pin}`);
  const hash = await crypto.subtle.digest('SHA-256', data);
  return Array.from(new Uint8Array(hash))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0B1020',
    alignItems: 'center',
    justifyContent: 'center',
    padding: 24,
  },
  title: {
    fontSize: 22,
    fontWeight: '700',
    color: '#F8FAFC',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    color: '#94A3B8',
    marginBottom: 32,
    textAlign: 'center',
  },
  pinDots: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 40,
  },
  dot: {
    width: 14,
    height: 14,
    borderRadius: 7,
    borderWidth: 2,
    borderColor: '#475569',
    backgroundColor: 'transparent',
  },
  dotFilled: {
    backgroundColor: '#38BDF8',
    borderColor: '#38BDF8',
  },
  keypad: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    width: 240,
    justifyContent: 'center',
    gap: 12,
  },
  key: {
    width: 64,
    height: 64,
    borderRadius: 32,
    backgroundColor: '#1E293B',
    alignItems: 'center',
    justifyContent: 'center',
  },
  keyEmpty: {
    width: 64,
    height: 64,
  },
  keyText: {
    color: '#F8FAFC',
    fontSize: 22,
    fontWeight: '600',
  },
  biometricButton: {
    marginTop: 28,
    paddingVertical: 10,
    paddingHorizontal: 24,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#334155',
  },
  biometricText: {
    color: '#38BDF8',
    fontSize: 15,
    fontWeight: '600',
  },
});
