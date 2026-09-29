import { useEffect, useState, useCallback, useRef } from 'react';
import {
  WebSocketClient,
  WebSocketClientOptions,
  ConnectionStatus,
  WebSocketEvent,
  EventHandler,
} from '../services/websocket';

export interface UseWebSocketReturn {
  status: ConnectionStatus;
  isConnected: boolean;
  send: (payload: Record<string, unknown>) => void;
  subscribe: (channel: string, handler: EventHandler) => () => void;
  client: WebSocketClient | null;
}

export function useWebSocket(options?: Partial<WebSocketClientOptions>): UseWebSocketReturn {
  const [status, setStatus] = useState<ConnectionStatus>('disconnected');
  const clientRef = useRef<WebSocketClient | null>(null);

  useEffect(() => {
    const wsUrl =
      options?.url ||
      (typeof window !== 'undefined'
        ? (window.location.protocol === 'https:' ? 'wss://' : 'ws://') +
          window.location.host +
          '/ws'
        : 'ws://localhost:8080');

    const client = new WebSocketClient({
      url: wsUrl,
      reconnect: true,
      ...options,
    });

    clientRef.current = client;

    const unsubscribeStatus = client.onStatusChange((newStatus) => {
      setStatus(newStatus);
    });

    client.connect();

    return () => {
      unsubscribeStatus();
      client.disconnect();
    };
  }, [options?.url]);

  const send = useCallback((payload: Record<string, unknown>) => {
    clientRef.current?.send(payload);
  }, []);

  const subscribe = useCallback((channel: string, handler: EventHandler) => {
    if (!clientRef.current) return () => {};
    return clientRef.current.subscribe(channel, handler);
  }, []);

  return {
    status,
    isConnected: status === 'connected',
    send,
    subscribe,
    client: clientRef.current,
  };
}

export function useWebSocketSubscription<T = unknown>(
  channel: string,
  onEvent?: (event: WebSocketEvent<T>) => void
): { latestEvent: WebSocketEvent<T> | null } {
  const { subscribe } = useWebSocket();
  const [latestEvent, setLatestEvent] = useState<WebSocketEvent<T> | null>(null);

  useEffect(() => {
    const unsub = subscribe(channel, (ev) => {
      setLatestEvent(ev as WebSocketEvent<T>);
      onEvent?.(ev as WebSocketEvent<T>);
    });
    return unsub;
  }, [channel, subscribe, onEvent]);

  return { latestEvent };
}
