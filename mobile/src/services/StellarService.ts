import { StellarAccount } from '../types';
import { NetworkType } from '../store';

export interface ContractEvent {
  id: string;
  type: 'mint' | 'trade' | 'upgrade' | 'transfer';
  contractId: string;
  data: Record<string, any>;
  timestamp: number;
}

export class StellarService {
  private network: NetworkType;
  private rpcUrl: string;

  constructor(network: NetworkType = 'testnet') {
    this.network = network;
    this.rpcUrl = this.getRpcEndpoint(network);
  }

  public getRpcEndpoint(network: NetworkType): string {
    switch (network) {
      case 'public':
        return 'https://horizon.stellar.org';
      case 'futurenet':
        return 'https://rpc-futurenet.stellar.org';
      case 'testnet':
      default:
        return 'https://horizon-testnet.stellar.org';
    }
  }

  public setNetwork(network: NetworkType): void {
    this.network = network;
    this.rpcUrl = this.getRpcEndpoint(network);
  }

  public async getAccountBalance(publicKey: string): Promise<string> {
    try {
      const response = await fetch(`${this.rpcUrl}/accounts/${publicKey}`);
      if (!response.ok) {
        if (response.status === 404) {
          return '0.0000000';
        }
        throw new Error(`Failed to fetch account balance: ${response.statusText}`);
      }

      const data = await response.json();
      const nativeBalance = data.balances?.find(
        (b: { asset_type: string }) => b.asset_type === 'native'
      );

      return nativeBalance ? nativeBalance.balance : '0.0000000';
    } catch {
      return '0.0000000';
    }
  }

  public validatePublicKey(publicKey: string): boolean {
    return /^G[A-Z2-7]{55}$/.test(publicKey);
  }

  public createAccountFromKey(publicKey: string): StellarAccount {
    if (!this.validatePublicKey(publicKey)) {
      throw new Error('Invalid Stellar public key format');
    }
    return { publicKey };
  }
}

export const stellarService = new StellarService();
