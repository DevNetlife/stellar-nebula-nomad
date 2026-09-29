export type ConnectionStatus =
  | 'connecting'
  | 'connected'
  | 'disconnected'
  | 'reconnecting'
  | 'error';

export interface WebSocketEvent<T = unknown> {
  id: string;
  type: string;
  channel: string;
  payload: T;
  sequence: number;
  timestamp: number;
}

export type EventHandler<T = any> = (event: WebSocketEvent<T>) => void;
export type StatusHandler = (status: ConnectionStatus) => void;

export interface WebSocketClientOptions {
  url: string;
  reconnect?: boolean;
  initialReconnectDelayMs?: number;
  maxReconnectDelayMs?: number;
  reconnectBackoffFactor?: number;
  maxReconnectAttempts?: number;
  heartbeatIntervalMs?: number;
  throttleIntervalMs?: number;
}

export class WebSocketClient {
  private socket: WebSocket | null = null;
  private url: string;
  private status: ConnectionStatus = 'disconnected';
  private reconnectEnabled: boolean;
  private reconnectAttempts = 0;
  private reconnectDelay: number;
  private maxReconnectDelay: number;
  private backoffFactor: number;
  private maxReconnectAttempts: number;
  private reconnectTimeoutId: any = null;
  private heartbeatIntervalMs: number;
  private heartbeatTimer: any = null;

  private subscriptions: Set<string> = new Set();
  private eventListeners: Map<string, Set<EventHandler>> = new Map();
  private statusListeners: Set<StatusHandler> = new Set();

  private pendingQueue: string[] = [];
  private throttleIntervalMs: number;
  private throttledBuffer: WebSocketEvent[] = [];
  private throttleTimer: any = null;

  constructor(options: WebSocketClientOptions) {
    this.url = options.url;
    this.reconnectEnabled = options.reconnect ?? true;
    this.reconnectDelay = options.initialReconnectDelayMs ?? 1000;
    this.maxReconnectDelay = options.maxReconnectDelayMs ?? 30000;
    this.backoffFactor = options.reconnectBackoffFactor ?? 1.5;
    this.maxReconnectAttempts = options.maxReconnectAttempts ?? 10;
    this.heartbeatIntervalMs = options.heartbeatIntervalMs ?? 25000;
    this.throttleIntervalMs = options.throttleIntervalMs ?? 50;
  }

  public connect(): void {
    if (
      this.socket &&
      (this.socket.readyState === WebSocket.OPEN ||
        this.socket.readyState === WebSocket.CONNECTING)
    ) {
      return;
    }

    this.setStatus(this.reconnectAttempts > 0 ? 'reconnecting' : 'connecting');

    try {
      this.socket = new WebSocket(this.url);

      this.socket.onopen = () => {
        this.setStatus('connected');
        this.reconnectAttempts = 0;
        this.reconnectDelay = 1000;
        this.startHeartbeat();

        // Resubscribe to existing channels
        if (this.subscriptions.size > 0) {
          this.send({
            type: 'subscribe',
            channels: Array.from(this.subscriptions),
          });
        }

        // Flush offline pending queue
        while (this.pendingQueue.length > 0) {
          const msg = this.pendingQueue.shift()!;
          this.socket?.send(msg);
        }
      };

      this.socket.onmessage = (event: MessageEvent) => {
        this.handleMessage(event.data);
      };

      this.socket.onerror = () => {
        this.setStatus('error');
      };

      this.socket.onclose = () => {
        this.stopHeartbeat();
        this.socket = null;
        if (this.reconnectEnabled && this.reconnectAttempts < this.maxReconnectAttempts) {
          this.scheduleReconnect();
        } else {
          this.setStatus('disconnected');
        }
      };
    } catch {
      this.setStatus('error');
      this.scheduleReconnect();
    }
  }

  public disconnect(): void {
    this.reconnectEnabled = false;
    if (this.reconnectTimeoutId) {
      clearTimeout(this.reconnectTimeoutId);
      this.reconnectTimeoutId = null;
    }
    this.stopHeartbeat();
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }
    this.setStatus('disconnected');
  }

  public getStatus(): ConnectionStatus {
    return this.status;
  }

  public onStatusChange(handler: StatusHandler): () => void {
    this.statusListeners.add(handler);
    handler(this.status);
    return () => this.statusListeners.delete(handler);
  }

  public subscribe(channel: string, handler: EventHandler): () => void {
    this.subscriptions.add(channel);

    if (!this.eventListeners.has(channel)) {
      this.eventListeners.set(channel, new Set());
    }
    this.eventListeners.get(channel)!.add(handler);

    if (this.status === 'connected') {
      this.send({ type: 'subscribe', channels: [channel] });
    }

    return () => {
      const handlers = this.eventListeners.get(channel);
      if (handlers) {
        handlers.delete(handler);
        if (handlers.size === 0) {
          this.eventListeners.delete(channel);
          this.subscriptions.delete(channel);
          if (this.status === 'connected') {
            this.send({ type: 'unsubscribe', channels: [channel] });
          }
        }
      }
    };
  }

  public send(payload: Record<string, unknown>): void {
    const raw = JSON.stringify(payload);
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(raw);
    } else {
      this.pendingQueue.push(raw);
    }
  }

  private handleMessage(data: string): void {
    try {
      const parsed = JSON.parse(data);

      if (parsed.type === 'event' && parsed.data) {
        this.queueThrottledEvent(parsed.data);
      } else if (parsed.type === 'batch' && Array.isArray(parsed.events)) {
        for (const ev of parsed.events) {
          this.queueThrottledEvent(ev);
        }
      }
    } catch {
      // ignore malformed message
    }
  }

  private queueThrottledEvent(ev: WebSocketEvent): void {
    this.throttledBuffer.push(ev);

    if (!this.throttleTimer) {
      this.throttleTimer = setTimeout(() => {
        this.flushThrottledEvents();
      }, this.throttleIntervalMs);
    }
  }

  private flushThrottledEvents(): void {
    this.throttleTimer = null;
    const events = this.throttledBuffer.splice(0, this.throttledBuffer.length);

    for (const ev of events) {
      // Channel listeners
      const channelHandlers = this.eventListeners.get(ev.channel);
      if (channelHandlers) {
        for (const fn of channelHandlers) fn(ev);
      }

      // Event type listeners
      const typeHandlers = this.eventListeners.get(ev.type);
      if (typeHandlers) {
        for (const fn of typeHandlers) fn(ev);
      }

      // Wildcard listeners
      const wildcardHandlers = this.eventListeners.get('*');
      if (wildcardHandlers) {
        for (const fn of wildcardHandlers) fn(ev);
      }
    }
  }

  private setStatus(newStatus: ConnectionStatus): void {
    this.status = newStatus;
    for (const listener of this.statusListeners) {
      listener(newStatus);
    }
  }

  private scheduleReconnect(): void {
    this.setStatus('reconnecting');
    this.reconnectAttempts++;
    const jitter = Math.random() * 500;
    const delay = Math.min(
      this.reconnectDelay * Math.pow(this.backoffFactor, this.reconnectAttempts) + jitter,
      this.maxReconnectDelay
    );

    this.reconnectTimeoutId = setTimeout(() => {
      this.connect();
    }, delay);
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();
    this.heartbeatTimer = setInterval(() => {
      if (this.socket && this.socket.readyState === WebSocket.OPEN) {
        this.send({ type: 'ping', timestamp: Date.now() });
      }
    }, this.heartbeatIntervalMs);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }
}
