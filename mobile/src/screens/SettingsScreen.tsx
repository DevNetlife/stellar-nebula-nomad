import React from 'react';
import { View, Text, StyleSheet, ScrollView, TouchableOpacity } from 'react-native';
import { useAppStore, NetworkType } from '../store';
import { BiometricSettings } from '../components/BiometricSettings';

export const SettingsScreen: React.FC = () => {
  const { network, setNetwork, reset } = useAppStore();

  const networks: NetworkType[] = ['testnet', 'public', 'futurenet'];

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <Text style={styles.title}>Settings</Text>

      <View style={styles.section}>
        <Text style={styles.sectionHeader}>Security</Text>
        <BiometricSettings />
      </View>

      <View style={styles.section}>
        <Text style={styles.sectionHeader}>Stellar Network</Text>
        <View style={styles.networkPicker}>
          {networks.map((net) => (
            <TouchableOpacity
              key={net}
              style={[styles.networkOption, network === net && styles.networkSelected]}
              onPress={() => setNetwork(net)}
            >
              <Text
                style={[
                  styles.networkOptionText,
                  network === net && styles.networkSelectedText,
                ]}
              >
                {net.toUpperCase()}
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      </View>

      <View style={styles.section}>
        <TouchableOpacity style={styles.resetButton} onPress={reset}>
          <Text style={styles.resetButtonText}>Reset Application Data</Text>
        </TouchableOpacity>
      </View>
    </ScrollView>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0a0d1a',
  },
  content: {
    padding: 20,
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#ffffff',
    marginVertical: 18,
  },
  section: {
    marginBottom: 24,
  },
  sectionHeader: {
    fontSize: 14,
    fontWeight: '700',
    color: '#64748b',
    textTransform: 'uppercase',
    letterSpacing: 1,
    marginBottom: 10,
  },
  networkPicker: {
    flexDirection: 'row',
    gap: 8,
  },
  networkOption: {
    flex: 1,
    backgroundColor: '#131b2e',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#1e293b',
  },
  networkSelected: {
    backgroundColor: '#1e3a8a',
    borderColor: '#3b82f6',
  },
  networkOptionText: {
    color: '#94a3b8',
    fontWeight: '600',
    fontSize: 13,
  },
  networkSelectedText: {
    color: '#60a5fa',
    fontWeight: 'bold',
  },
  resetButton: {
    backgroundColor: '#331515',
    paddingVertical: 14,
    borderRadius: 8,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#7f1d1d',
  },
  resetButtonText: {
    color: '#f87171',
    fontWeight: 'bold',
  },
});
