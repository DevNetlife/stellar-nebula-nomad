import { WebSocketServer, WebSocket } from 'ws';
import { IncomingMessage } from 'http';
import {
  ClientMessage,
  GameEvent,
  ServerConfig,
  ServerMessage,
} from './types';
import { StellarEventListener } from './stellar-listener';

interface ClientState {
  ws: WebSocket;
  id: string;
  isAlive: boolean;
  subscriptions: Set<string>;
  outgoingQueue: GameEvent[];
}

export class GameWebSocketServer {
  private wss: WebSocketServer | null = null;
  private clients: Map<WebSocket, ClientState> = new Map();
  private config: Required<ServerConfig>;
  private batchTimer: NodeJS.Timeout | null = null;
  private heartbeatTimer: NodeJS.Timeout | null = null;
  private stellarListener: StellarEventListener;
  private clientIdCounter = 0;

  constructor(
    config: ServerConfig,
    stellarListener?: StellarEventListener
  ) {
    this.config = {
      port: config.port,
      host: config.host || '0.0.0.0',
      batchIntervalMs: config.batchIntervalMs ?? 50,
      maxBatchSize: config.maxBatchSize ?? 100,
      heartbeatIntervalMs: config.heartbeatIntervalMs ?? 30000,
      maxEventsPerSecondPerClient: config.maxEventsPerSecondPerClient ?? 500,
    };

    this.stellarListener = stellarListener || new StellarEventListener();
    this.stellarListener.on('event', (event: GameEvent) => {
      this.broadcastEvent(event);
    });
  }

  public start(): Promise<void> {
    return new Promise((resolve) => {
      this.wss = new WebSocketServer({
        port: this.config.port,
        host: this.config.host,
      });

      this.wss.on('connection', (ws: WebSocket, req: IncomingMessage) => {
        this.handleConnection(ws, req);
      });

      this.startBatchProcessor();
      this.startHeartbeat();
      this.stellarListener.start();
      resolve();
    });
  }

  public async stop(): Promise<void> {
    if (this.batchTimer) {
      clearInterval(this.batchTimer);
      this.batchTimer = null;
    }
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }

    this.stellarListener.stop();

    if (this.wss) {
      for (const [ws] of this.clients) {
        try {
          ws.close(1000, 'Server shutting down');
        } catch {
          // ignore
        }
      }
      this.clients.clear();
      await new Promise<void>((resolve) => {
        this.wss?.close(() => resolve());
      });
      this.wss = null;
    }
  }

  public getConnectedClientCount(): number {
    return this.clients.size;
  }

  public getListener(): StellarEventListener {
    return this.stellarListener;
  }

  private handleConnection(ws: WebSocket, _req: IncomingMessage): void {
    const id = `client-${++this.clientIdCounter}`;
    const state: ClientState = {
      ws,
      id,
      isAlive: true,
      subscriptions: new Set(['system', 'leaderboard', 'market_prices']),
      outgoingQueue: [],
    };

    this.clients.set(ws, state);

    // Send confirmation of default subscriptions
    this.send(ws, {
      type: 'subscribed',
      channels: Array.from(state.subscriptions),
    });

    ws.on('pong', () => {
      state.isAlive = true;
    });

    ws.on('message', (data: Buffer | string) => {
      this.handleClientMessage(state, data);
    });

    ws.on('close', () => {
      this.clients.delete(ws);
    });

    ws.on('error', (_err) => {
      this.clients.delete(ws);
    });
  }

  private handleClientMessage(client: ClientState, raw: Buffer | string): void {
    try {
      const parsed = JSON.parse(raw.toString()) as ClientMessage;

      switch (parsed.type) {
        case 'subscribe':
          for (const ch of parsed.channels) {
            client.subscriptions.add(ch);
          }
          this.send(client.ws, {
            type: 'subscribed',
            channels: parsed.channels,
          });
          break;

        case 'unsubscribe':
          for (const ch of parsed.channels) {
            client.subscriptions.delete(ch);
          }
          this.send(client.ws, {
            type: 'unsubscribed',
            channels: parsed.channels,
          });
          break;

        case 'ping':
          this.send(client.ws, {
            type: 'pong',
            timestamp: parsed.timestamp || Date.now(),
          });
          break;

        default:
          break;
      }
    } catch {
      this.send(client.ws, {
        type: 'error',
        message: 'Invalid JSON payload received',
      });
    }
  }

  public broadcastEvent(event: GameEvent): void {
    for (const client of this.clients.values()) {
      if (
        client.subscriptions.has(event.channel) ||
        client.subscriptions.has(event.type) ||
        client.subscriptions.has('*')
      ) {
        if (client.outgoingQueue.length < this.config.maxBatchSize * 5) {
          client.outgoingQueue.push(event);
        }
      }
    }
  }

  private startBatchProcessor(): void {
    this.batchTimer = setInterval(() => {
      for (const client of this.clients.values()) {
        if (client.outgoingQueue.length === 0) continue;

        if (client.outgoingQueue.length === 1) {
          const single = client.outgoingQueue.shift()!;
          this.send(client.ws, {
            type: 'event',
            data: single,
          });
        } else {
          const batch = client.outgoingQueue.splice(
            0,
            this.config.maxBatchSize
          );
          this.send(client.ws, {
            type: 'batch',
            events: batch,
            count: batch.length,
          });
        }
      }
    }, this.config.batchIntervalMs);
  }

  private startHeartbeat(): void {
    this.heartbeatTimer = setInterval(() => {
      for (const [ws, state] of this.clients) {
        if (!state.isAlive) {
          ws.terminate();
          this.clients.delete(ws);
          continue;
        }
        state.isAlive = false;
        try {
          ws.ping();
        } catch {
          this.clients.delete(ws);
        }
      }
    }, this.config.heartbeatIntervalMs);
  }

  private send(ws: WebSocket, message: ServerMessage): void {
    if (ws.readyState === WebSocket.OPEN) {
      try {
        ws.send(JSON.stringify(message));
      } catch {
        // socket might be closing
      }
    }
  }
}
