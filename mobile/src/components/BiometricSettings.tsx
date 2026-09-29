import React, { useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Switch,
  Alert,
  ActivityIndicator,
} from 'react-native';
import { BiometricAuth } from '../auth/BiometricAuth';
import { SecureStorage } from '../utils/SecureStorage';

interface BiometricSettingsProps {
  onToggle?: (enabled: boolean) => void;
}

export default function BiometricSettings({ onToggle }: BiometricSettingsProps) {
  const [enabled, setEnabled] = React.useState(false);
  const [primaryType, setPrimaryType] = React.useState('Biometrics');
  const [isLoading, setIsLoading] = React.useState(false);

  React.useEffect(() => {
    SecureStorage.isBiometricsEnabled().then(setEnabled);
    BiometricAuth.getCapabilities().then((caps) => {
      setPrimaryType(BiometricAuth.getBiometricLabel(caps.primaryType));
    });
  }, []);

  const handleToggle = async (value: boolean) => {
    if (value) {
      setIsLoading(true);
      const success = await BiometricAuth.enable();
      setIsLoading(false);
      if (success) {
        setEnabled(true);
        onToggle?.(true);
      } else {
        Alert.alert('Verification Required', 'Biometric verification failed. Try again.');
      }
    } else {
      Alert.alert(
        `Disable ${primaryType}`,
        `You will need to use your PIN to unlock the wallet. Continue?`,
        [
          { text: 'Cancel', style: 'cancel' },
          {
            text: 'Disable',
            style: 'destructive',
            onPress: async () => {
              await BiometricAuth.disable();
              setEnabled(false);
              onToggle?.(false);
            },
          },
        ]
      );
    }
  };

  return (
    <View style={styles.row}>
      <View style={styles.labelWrapper}>
        <Text style={styles.label}>{primaryType} Unlock</Text>
        <Text style={styles.desc}>
          Use {primaryType} to unlock the app and authorize transactions
        </Text>
      </View>
      {isLoading ? (
        <ActivityIndicator color="#38BDF8" />
      ) : (
        <Switch
          value={enabled}
          onValueChange={handleToggle}
          trackColor={{ false: '#334155', true: '#3B82F6' }}
          thumbColor={enabled ? '#38BDF8' : '#94A3B8'}
          accessibilityLabel={`Toggle ${primaryType} unlock`}
        />
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    backgroundColor: '#1E293B',
    borderRadius: 10,
    padding: 16,
    gap: 12,
  },
  labelWrapper: {
    flex: 1,
  },
  label: {
    color: '#F8FAFC',
    fontSize: 15,
    fontWeight: '600',
  },
  desc: {
    color: '#94A3B8',
    fontSize: 12,
    marginTop: 2,
  },
});
