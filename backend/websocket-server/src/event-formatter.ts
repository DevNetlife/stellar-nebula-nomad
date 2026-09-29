import { GameEvent, GameEventType, GameEventPayloads } from './types';

let sequenceCounter = 0;

export class EventFormatter {
  public static resetSequence(): void {
    sequenceCounter = 0;
  }

  public static getNextSequence(): number {
    return ++sequenceCounter;
  }

  public static formatEvent<T extends GameEventType>(
    type: T,
    channel: string,
    payload: GameEventPayloads[T],
    idOverride?: string
  ): GameEvent<T> {
    const timestamp = Date.now();
    const id = idOverride || `${type}-${timestamp}-${Math.random().toString(36).slice(2, 9)}`;

    return {
      id,
      type,
      channel,
      payload,
      sequence: this.getNextSequence(),
      timestamp,
    };
  }

  public static formatStellarContractEvent(rawEvent: {
    contractId: string;
    topic: string[];
    data: unknown;
    txHash?: string;
  }): GameEvent | null {
    const topic0 = rawEvent.topic[0];
    const data = rawEvent.data as Record<string, unknown>;

    switch (topic0) {
      case 'mint':
      case 'mint_ship':
      case 'mint_item':
        return this.formatEvent('mint', 'events:mint', {
          tokenId: String(data.tokenId || data.id || 'token-0'),
          owner: String(data.to || data.owner || 'unknown'),
          itemType: String(data.itemType || 'Explorer'),
          rarity: String(data.rarity || 'Common'),
          timestamp: Date.now(),
          transactionHash: rawEvent.txHash || '0x0',
        });

      case 'trade':
      case 'marketplace_trade':
        return this.formatEvent('trade', 'events:trade', {
          tradeId: String(data.tradeId || `trade-${Date.now()}`),
          seller: String(data.seller || ''),
          buyer: String(data.buyer || ''),
          tokenId: String(data.tokenId || ''),
          price: String(data.price || '0'),
          currency: String(data.currency || 'XLM'),
          timestamp: Date.now(),
        });

      case 'upgrade':
      case 'ship_upgrade':
        return this.formatEvent('upgrade', 'events:upgrade', {
          entityId: String(data.entityId || data.shipId || ''),
          owner: String(data.owner || ''),
          upgradeType: String(data.upgradeType || 'engines'),
          newLevel: Number(data.newLevel || 1),
          cost: (data.cost as Record<string, string>) || {},
          timestamp: Date.now(),
        });

      case 'scan':
      case 'nebula_scan':
        return this.formatEvent('scan', 'events:scan', {
          scanner: String(data.scanner || data.owner || ''),
          sectorId: String(data.sectorId || 'sector-0'),
          anomalyDiscovered: Boolean(data.anomalyDiscovered),
          resourcesFound: (data.resourcesFound as Record<string, number>) || {},
          timestamp: Date.now(),
        });

      case 'guild':
      case 'guild_activity':
        return this.formatEvent('guild', `guild:${data.guildId || 'general'}`, {
          guildId: String(data.guildId || 'general'),
          actor: String(data.actor || ''),
          action: (data.action as any) || 'mission_completed',
          details: (data.details as Record<string, unknown>) || {},
          timestamp: Date.now(),
        });

      default:
        return null;
    }
  }
}
