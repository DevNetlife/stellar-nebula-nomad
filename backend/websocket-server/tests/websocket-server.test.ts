import WebSocket from 'ws';
import { GameWebSocketServer } from '../src/websocket-server';
import { StellarEventListener } from '../src/stellar-listener';
import { EventFormatter } from '../src/event-formatter';

describe('GameWebSocketServer', () => {
  let server: GameWebSocketServer;
  let listener: StellarEventListener;
  const TEST_PORT = 9876;
  const WS_URL = `ws://127.0.0.1:${TEST_PORT}`;

  beforeEach(async () => {
    listener = new StellarEventListener();
    server = new GameWebSocketServer(
      {
        port: TEST_PORT,
        host: '127.0.0.1',
        batchIntervalMs: 20,
        maxBatchSize: 50,
      },
      listener
    );
    await server.start();
  });

  afterEach(async () => {
    await server.stop();
  });

  test('client connects and receives default subscriptions', (done) => {
    const ws = new WebSocket(WS_URL);

    ws.on('message', (data: Buffer) => {
      const msg = JSON.parse(data.toString());
      if (msg.type === 'subscribed') {
        expect(msg.channels).toContain('leaderboard');
        expect(msg.channels).toContain('market_prices');
        ws.close();
        done();
      }
    });
  });

  test('client can subscribe to mint and trade channels and receive events', (done) => {
    const ws = new WebSocket(WS_URL);

    ws.on('open', () => {
      ws.send(
        JSON.stringify({
          type: 'subscribe',
          channels: ['events:mint'],
        })
      );
    });

    let subscribed = false;
    ws.on('message', (data: Buffer) => {
      const msg = JSON.parse(data.toString());
      if (msg.type === 'subscribed' && msg.channels.includes('events:mint')) {
        subscribed = true;
        // Emit an event through the listener
        listener.dispatchCustomEvent(
          EventFormatter.formatEvent('mint', 'events:mint', {
            tokenId: 'ship-101',
            owner: 'GCXYZ...',
            itemType: 'Frigate',
            rarity: 'Legendary',
            timestamp: Date.now(),
            transactionHash: 'tx-123',
          })
        );
      } else if (subscribed && msg.type === 'event') {
        expect(msg.data.type).toBe('mint');
        expect(msg.data.payload.tokenId).toBe('ship-101');
        ws.close();
        done();
      }
    });
  });

  test('handles high event frequency (150+ events/sec) with batching', (done) => {
    const ws = new WebSocket(WS_URL);
    const TOTAL_EVENTS = 150;
    let receivedEventsCount = 0;
    let subscribed = false;

    ws.on('open', () => {
      ws.send(
        JSON.stringify({
          type: 'subscribe',
          channels: ['events:trade'],
        })
      );
    });

    ws.on('message', (data: Buffer) => {
      const msg = JSON.parse(data.toString());

      if (msg.type === 'subscribed' && msg.channels.includes('events:trade')) {
        subscribed = true;
        // Spam 150 events after subscription is confirmed
        for (let i = 0; i < TOTAL_EVENTS; i++) {
          listener.dispatchCustomEvent(
            EventFormatter.formatEvent('trade', 'events:trade', {
              tradeId: `trade-${i}`,
              seller: `seller-${i}`,
              buyer: `buyer-${i}`,
              tokenId: `token-${i}`,
              price: '10',
              currency: 'XLM',
              timestamp: Date.now(),
            })
          );
        }
        return;
      }

      if (subscribed) {
        if (msg.type === 'batch') {
          receivedEventsCount += msg.events.length;
        } else if (msg.type === 'event' && msg.data.type === 'trade') {
          receivedEventsCount += 1;
        }

        if (receivedEventsCount >= TOTAL_EVENTS) {
          expect(receivedEventsCount).toBe(TOTAL_EVENTS);
          ws.close();
          done();
        }
      }
    });
  }, 10000);
});
