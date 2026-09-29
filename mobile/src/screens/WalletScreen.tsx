import React, { useState } from 'react';
import { View, Text, StyleSheet, TextInput, TouchableOpacity, Alert } from 'react-native';
import { useAppStore } from '../store';
import { stellarService } from '../services/StellarService';

export const WalletScreen: React.FC = () => {
  const { account, balance, setAccount, setBalance } = useAppStore();
  const [inputKey, setInputKey] = useState('');

  const handleConnect = async () => {
    if (!stellarService.validatePublicKey(inputKey.trim())) {
      Alert.alert('Invalid Key', 'Please enter a valid 56-character Stellar public key (starts with G)');
      return;
    }

    try {
      const acct = stellarService.createAccountFromKey(inputKey.trim());
      setAccount(acct);
      const bal = await stellarService.getAccountBalance(acct.publicKey);
      setBalance(bal);
      setInputKey('');
    } catch (err: any) {
      Alert.alert('Error', err?.message || 'Failed to connect wallet');
    }
  };

  const handleDisconnect = () => {
    setAccount(null);
    setBalance('0.0000000');
  };

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Stellar Wallet</Text>

      {account ? (
        <View style={styles.connectedCard}>
          <Text style={styles.connectedLabel}>Active Account</Text>
          <Text style={styles.publicKey} numberOfLines={1} ellipsizeMode="middle">
            {account.publicKey}
          </Text>
          <Text style={styles.balanceText}>{balance} XLM</Text>

          <TouchableOpacity style={styles.disconnectButton} onPress={handleDisconnect}>
            <Text style={styles.disconnectButtonText}>Disconnect</Text>
          </TouchableOpacity>
        </View>
      ) : (
        <View style={styles.importCard}>
          <Text style={styles.importLabel}>Connect Stellar Public Key</Text>
          <TextInput
            style={styles.input}
            placeholder="G..."
            placeholderTextColor="#64748b"
            value={inputKey}
            onChangeText={setInputKey}
            autoCapitalize="characters"
            autoCorrect={false}
          />
          <TouchableOpacity style={styles.connectButton} onPress={handleConnect}>
            <Text style={styles.connectButtonText}>Import & Connect</Text>
          </TouchableOpacity>
        </View>
      )}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0a0d1a',
    padding: 20,
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#ffffff',
    marginVertical: 20,
  },
  connectedCard: {
    backgroundColor: '#131b2e',
    borderRadius: 12,
    padding: 20,
    borderWidth: 1,
    borderColor: '#22304d',
  },
  connectedLabel: {
    color: '#7b91b8',
    fontSize: 12,
    fontWeight: '700',
    textTransform: 'uppercase',
  },
  publicKey: {
    color: '#38bdf8',
    fontSize: 14,
    fontFamily: 'monospace',
    marginVertical: 8,
  },
  balanceText: {
    fontSize: 22,
    fontWeight: 'bold',
    color: '#ffffff',
    marginVertical: 12,
  },
  disconnectButton: {
    backgroundColor: '#334155',
    paddingVertical: 10,
    borderRadius: 8,
    alignItems: 'center',
    marginTop: 10,
  },
  disconnectButtonText: {
    color: '#f87171',
    fontWeight: '600',
  },
  importCard: {
    backgroundColor: '#131b2e',
    borderRadius: 12,
    padding: 20,
    borderWidth: 1,
    borderColor: '#22304d',
  },
  importLabel: {
    color: '#94a3b8',
    fontSize: 14,
    marginBottom: 12,
  },
  input: {
    backgroundColor: '#0a0d1a',
    borderWidth: 1,
    borderColor: '#334155',
    borderRadius: 8,
    color: '#ffffff',
    padding: 12,
    fontFamily: 'monospace',
    marginBottom: 16,
  },
  connectButton: {
    backgroundColor: '#3b82f6',
    paddingVertical: 12,
    borderRadius: 8,
    alignItems: 'center',
  },
  connectButtonText: {
    color: '#ffffff',
    fontWeight: 'bold',
    fontSize: 15,
  },
});
