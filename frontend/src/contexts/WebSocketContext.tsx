import React, {
  createContext,
  useContext,
  useEffect,
  useState,
  useCallback,
  ReactNode,
} from 'react';
import {
  WebSocketClient,
  ConnectionStatus,
  WebSocketEvent,
} from '../services/websocket';

export interface ToastNotificationItem {
  id: string;
  title: string;
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
  timestamp: number;
}

export interface MarketPriceItem {
  asset: string;
  price: number;
  change24h: number;
  volume24h: number;
  updatedAt: number;
}

export interface LeaderboardEntry {
  rank: number;
  playerId: string;
  score: number;
  tag?: string;
}

export interface GuildActivityItem {
  id: string;
  guildId: string;
  actor: string;
  action: string;
  details: Record<string, unknown>;
  timestamp: number;
}

export interface WebSocketContextValue {
  status: ConnectionStatus;
  isConnected: boolean;
  toasts: ToastNotificationItem[];
  dismissToast: (id: string) => void;
  marketPrices: Record<string, MarketPriceItem>;
  leaderboard: LeaderboardEntry[];
  guildActivities: GuildActivityItem[];
  subscribe: (channel: string, handler: (ev: WebSocketEvent) => void) => () => void;
  sendMessage: (payload: Record<string, unknown>) => void;
}

const WebSocketContext = createContext<WebSocketContextValue | null>(null);

export interface WebSocketProviderProps {
  children: ReactNode;
  wsUrl?: string;
  currentPlayerAddress?: string;
}

export const WebSocketProvider: React.FC<WebSocketProviderProps> = ({
  children,
  wsUrl,
  currentPlayerAddress,
}) => {
  const [status, setStatus] = useState<ConnectionStatus>('disconnected');
  const [client, setClient] = useState<WebSocketClient | null>(null);
  const [toasts, setToasts] = useState<ToastNotificationItem[]>([]);
  const [marketPrices, setMarketPrices] = useState<Record<string, MarketPriceItem>>({});
  const [leaderboard, setLeaderboard] = useState<LeaderboardEntry[]>([]);
  const [guildActivities, setGuildActivities] = useState<GuildActivityItem[]>([]);

  const addToast = useCallback(
    (item: Omit<ToastNotificationItem, 'id' | 'timestamp'>) => {
      const id = `toast-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
      const newToast: ToastNotificationItem = {
        ...item,
        id,
        timestamp: Date.now(),
      };
      setToasts((prev) => [newToast, ...prev].slice(0, 10));

      setTimeout(() => {
        setToasts((prev) => prev.filter((t) => t.id !== id));
      }, 5000);
    },
    []
  );

  const dismissToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  useEffect(() => {
    const url =
      wsUrl ||
      (typeof window !== 'undefined'
        ? (window.location.protocol === 'https:' ? 'wss://' : 'ws://') +
          window.location.host +
          '/ws'
        : 'ws://localhost:8080');

    const wsClient = new WebSocketClient({
      url,
      reconnect: true,
      initialReconnectDelayMs: 1000,
      maxReconnectDelayMs: 20000,
    });

    setClient(wsClient);

    const unsubStatus = wsClient.onStatusChange((newStatus) => {
      setStatus(newStatus);
    });

    // Subscriptions to live feeds
    const unsubWildcard = wsClient.subscribe('*', (ev: WebSocketEvent<any>) => {
      const payload = ev.payload;

      switch (ev.type) {
        case 'mint':
          if (
            !currentPlayerAddress ||
            payload.owner === currentPlayerAddress
          ) {
            addToast({
              title: 'Asset Minted',
              message: `${payload.itemType} (${payload.rarity}) minted successfully`,
              type: 'success',
            });
          }
          break;

        case 'trade':
          if (
            !currentPlayerAddress ||
            payload.seller === currentPlayerAddress ||
            payload.buyer === currentPlayerAddress
          ) {
            addToast({
              title: 'Trade Executed',
              message: `Token ${payload.tokenId} exchanged for ${payload.price} ${payload.currency}`,
              type: 'info',
            });
          }
          break;

        case 'upgrade':
          if (
            !currentPlayerAddress ||
            payload.owner === currentPlayerAddress
          ) {
            addToast({
              title: 'Ship Upgraded',
              message: `${payload.upgradeType} upgraded to level ${payload.newLevel}`,
              type: 'success',
            });
          }
          break;

        case 'market_price':
          setMarketPrices((prev) => ({
            ...prev,
            [payload.asset]: payload,
          }));
          break;

        case 'leaderboard':
          if (payload.rankings) {
            setLeaderboard(payload.rankings);
          }
          break;

        case 'guild':
          setGuildActivities((prev) => [
            {
              id: ev.id,
              guildId: payload.guildId,
              actor: payload.actor,
              action: payload.action,
              details: payload.details,
              timestamp: ev.timestamp,
            },
            ...prev,
          ].slice(0, 30));
          break;

        default:
          break;
      }
    });

    wsClient.connect();

    return () => {
      unsubStatus();
      unsubWildcard();
      wsClient.disconnect();
    };
  }, [wsUrl, currentPlayerAddress, addToast]);

  const subscribe = useCallback(
    (channel: string, handler: (ev: WebSocketEvent) => void) => {
      if (!client) return () => {};
      return client.subscribe(channel, handler);
    },
    [client]
  );

  const sendMessage = useCallback(
    (payload: Record<string, unknown>) => {
      client?.send(payload);
    },
    [client]
  );

  return (
    <WebSocketContext.Provider
      value={{
        status,
        isConnected: status === 'connected',
        toasts,
        dismissToast,
        marketPrices,
        leaderboard,
        guildActivities,
        subscribe,
        sendMessage,
      }}
    >
      {children}
    </WebSocketContext.Provider>
  );
};

export const useWebSocketContext = (): WebSocketContextValue => {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error('useWebSocketContext must be used within a WebSocketProvider');
  }
  return context;
};

export const WebSocketStatusIndicator: React.FC<{ className?: string }> = ({
  className = '',
}) => {
  const { status } = useWebSocketContext();

  const colorMap: Record<ConnectionStatus, string> = {
    connected: '#10B981', // green
    connecting: '#F59E0B', // amber
    reconnecting: '#F59E0B',
    disconnected: '#6B7280', // gray
    error: '#EF4444', // red
  };

  const labelMap: Record<ConnectionStatus, string> = {
    connected: 'Live Network Connected',
    connecting: 'Connecting...',
    reconnecting: 'Reconnecting...',
    disconnected: 'Offline',
    error: 'Connection Error',
  };

  return (
    <div
      role="status"
      aria-live="polite"
      className={`websocket-status-badge ${className}`}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '6px',
        padding: '4px 8px',
        borderRadius: '9999px',
        fontSize: '0.75rem',
        fontWeight: 500,
        backgroundColor: 'rgba(255, 255, 255, 0.08)',
        color: '#E5E7EB',
      }}
    >
      <span
        style={{
          width: '8px',
          height: '8px',
          borderRadius: '50%',
          backgroundColor: colorMap[status] || '#6B7280',
          boxShadow:
            status === 'connected'
              ? '0 0 8px rgba(16, 185, 129, 0.6)'
              : 'none',
        }}
      />
      <span>{labelMap[status]}</span>
    </div>
  );
};
