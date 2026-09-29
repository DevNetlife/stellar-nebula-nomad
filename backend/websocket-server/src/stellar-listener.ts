import { EventEmitter } from 'events';
import { EventFormatter } from './event-formatter';
import { GameEvent } from './types';

export interface StellarListenerConfig {
  rpcUrl?: string;
  contractIds?: string[];
  pollIntervalMs?: number;
}

export class StellarEventListener extends EventEmitter {
  private isRunning: boolean = false;
  private pollTimer: NodeJS.Timeout | null = null;
  private config: StellarListenerConfig;

  constructor(config: StellarListenerConfig = {}) {
    super();
    this.config = {
      rpcUrl: config.rpcUrl || 'https://soroban-testnet.stellar.org',
      contractIds: config.contractIds || [],
      pollIntervalMs: config.pollIntervalMs || 2000,
    };
  }

  public start(): void {
    if (this.isRunning) return;
    this.isRunning = true;
    this.emit('started');
  }

  public stop(): void {
    if (!this.isRunning) return;
    this.isRunning = false;
    if (this.pollTimer) {
      clearInterval(this.pollTimer);
      this.pollTimer = null;
    }
    this.emit('stopped');
  }

  public ingestRawEvent(rawEvent: {
    contractId: string;
    topic: string[];
    data: unknown;
    txHash?: string;
  }): GameEvent | null {
    const formatted = EventFormatter.formatStellarContractEvent(rawEvent);
    if (formatted) {
      this.emit('event', formatted);
    }
    return formatted;
  }

  public dispatchCustomEvent(event: GameEvent): void {
    this.emit('event', event);
  }
}
