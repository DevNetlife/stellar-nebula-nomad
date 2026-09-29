import dotenv from 'dotenv';
import { GameWebSocketServer } from './websocket-server';
import { StellarEventListener } from './stellar-listener';

dotenv.config();

const PORT = parseInt(process.env.WS_PORT || '8080', 10);
const HOST = process.env.WS_HOST || '0.0.0.0';

const listener = new StellarEventListener({
  rpcUrl: process.env.SOROBAN_RPC_URL,
  contractIds: process.env.CONTRACT_IDS ? process.env.CONTRACT_IDS.split(',') : [],
});

const server = new GameWebSocketServer(
  {
    port: PORT,
    host: HOST,
    batchIntervalMs: 50,
    maxBatchSize: 100,
  },
  listener
);

server.start().then(() => {
  // eslint-disable-next-line no-console
  console.log(`Stellar Nebula Nomad WebSocket server started on ${HOST}:${PORT}`);
});

process.on('SIGINT', async () => {
  await server.stop();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  await server.stop();
  process.exit(0);
});

export { GameWebSocketServer, StellarEventListener };
export * from './types';
export * from './event-formatter';
