export type GameEventType =
  | 'mint'
  | 'trade'
  | 'upgrade'
  | 'scan'
  | 'guild'
  | 'leaderboard'
  | 'market_price'
  | 'system';

export interface GameEventPayloads {
  mint: {
    tokenId: string;
    owner: string;
    itemType: string;
    rarity: string;
    timestamp: number;
    transactionHash: string;
  };
  trade: {
    tradeId: string;
    seller: string;
    buyer: string;
    tokenId: string;
    price: string;
    currency: string;
    timestamp: number;
  };
  upgrade: {
    entityId: string;
    owner: string;
    upgradeType: string;
    newLevel: number;
    cost: Record<string, string>;
    timestamp: number;
  };
  scan: {
    scanner: string;
    sectorId: string;
    anomalyDiscovered: boolean;
    resourcesFound: Record<string, number>;
    timestamp: number;
  };
  guild: {
    guildId: string;
    actor: string;
    action: 'created' | 'joined' | 'left' | 'rank_up' | 'mission_completed';
    details: Record<string, unknown>;
    timestamp: number;
  };
  leaderboard: {
    category: 'wealth' | 'exploration' | 'pvp' | 'upgrades';
    rankings: Array<{
      rank: number;
      playerId: string;
      score: number;
      tag?: string;
    }>;
    updatedAt: number;
  };
  market_price: {
    asset: string;
    price: number;
    change24h: number;
    volume24h: number;
    updatedAt: number;
  };
  system: {
    level: 'info' | 'warn' | 'error';
    message: string;
    timestamp: number;
  };
}

export interface GameEvent<T extends GameEventType = GameEventType> {
  id: string;
  type: T;
  channel: string;
  payload: GameEventPayloads[T];
  sequence: number;
  timestamp: number;
}

export type ClientMessage =
  | { type: 'subscribe'; channels: string[] }
  | { type: 'unsubscribe'; channels: string[] }
  | { type: 'ping'; timestamp: number }
  | { type: 'get_snapshot'; channel: string };

export type ServerMessage =
  | { type: 'event'; data: GameEvent }
  | { type: 'batch'; events: GameEvent[]; count: number }
  | { type: 'subscribed'; channels: string[] }
  | { type: 'unsubscribed'; channels: string[] }
  | { type: 'pong'; timestamp: number }
  | { type: 'error'; message: string; code?: string }
  | { type: 'snapshot'; channel: string; data: unknown };

export interface ServerConfig {
  port: number;
  host?: string;
  batchIntervalMs?: number;
  maxBatchSize?: number;
  heartbeatIntervalMs?: number;
  maxEventsPerSecondPerClient?: number;
}
