import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity, ScrollView } from 'react-native';
import { useAppStore } from '../store';

export const HomeScreen: React.FC = () => {
  const { account, balance, network, isLocked, biometricsEnabled, setIsLocked } = useAppStore();

  const truncateAddress = (addr: string) => {
    if (!addr || addr.length < 12) return addr;
    return `${addr.substring(0, 6)}...${addr.substring(addr.length - 6)}`;
  };

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <View style={styles.header}>
        <Text style={styles.title}>Stellar Nebula Nomad</Text>
        <Text style={styles.subtitle}>Mobile Fleet Commander</Text>
      </View>

      <View style={styles.card}>
        <Text style={styles.cardLabel}>Wallet Status</Text>
        <Text style={styles.address}>
          {account ? truncateAddress(account.publicKey) : 'Not Connected'}
        </Text>
        <View style={styles.row}>
          <Text style={styles.balanceLabel}>Balance:</Text>
          <Text style={styles.balanceValue}>{balance} XLM</Text>
        </View>
        <View style={styles.row}>
          <Text style={styles.balanceLabel}>Network:</Text>
          <Text style={styles.networkBadge}>{network.toUpperCase()}</Text>
        </View>
      </View>

      <View style={styles.card}>
        <Text style={styles.cardLabel}>Security Overview</Text>
        <View style={styles.row}>
          <Text style={styles.statusLabel}>Biometric Unlock:</Text>
          <Text style={[styles.statusValue, biometricsEnabled ? styles.statusActive : styles.statusInactive]}>
            {biometricsEnabled ? 'Enabled' : 'Disabled'}
          </Text>
        </View>
        <View style={styles.row}>
          <Text style={styles.statusLabel}>Auto-Lock:</Text>
          <Text style={styles.statusValue}>5 minutes</Text>
        </View>
        <TouchableOpacity
          style={styles.lockButton}
          onPress={() => setIsLocked(true)}
          testID="lock-wallet-button"
        >
          <Text style={styles.lockButtonText}>Lock Wallet Now</Text>
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
  header: {
    marginVertical: 24,
  },
  title: {
    fontSize: 26,
    fontWeight: 'bold',
    color: '#ffffff',
    letterSpacing: 0.5,
  },
  subtitle: {
    fontSize: 14,
    color: '#8b9bb4',
    marginTop: 4,
  },
  card: {
    backgroundColor: '#131b2e',
    borderRadius: 14,
    padding: 18,
    marginBottom: 16,
    borderWidth: 1,
    borderColor: '#22304d',
  },
  cardLabel: {
    fontSize: 12,
    fontWeight: '700',
    color: '#7b91b8',
    textTransform: 'uppercase',
    letterSpacing: 1,
    marginBottom: 10,
  },
  address: {
    fontSize: 16,
    fontWeight: '600',
    color: '#38bdf8',
    marginBottom: 12,
    fontFamily: 'monospace',
  },
  row: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginVertical: 4,
  },
  balanceLabel: {
    fontSize: 14,
    color: '#94a3b8',
  },
  balanceValue: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#e2e8f0',
  },
  networkBadge: {
    fontSize: 12,
    fontWeight: '700',
    color: '#34d399',
    backgroundColor: '#064e3b',
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderRadius: 6,
  },
  statusLabel: {
    fontSize: 14,
    color: '#94a3b8',
  },
  statusValue: {
    fontSize: 14,
    fontWeight: '600',
  },
  statusActive: {
    color: '#34d399',
  },
  statusInactive: {
    color: '#f87171',
  },
  lockButton: {
    marginTop: 14,
    backgroundColor: '#1e293b',
    paddingVertical: 10,
    borderRadius: 8,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#334155',
  },
  lockButtonText: {
    color: '#f87171',
    fontWeight: '600',
    fontSize: 14,
  },
});
