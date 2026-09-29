# Stellar Nebula Nomad WebSocket Server

Live game event streaming server connecting Soroban / Stellar smart contract events to frontend clients in real time.

## Architecture

- **StellarEventListener**: Connects to Soroban RPC or indexer, extracts contract events, and filters relevant game events (mint, trade, upgrade, scan, guild activity).
- **EventFormatter**: Converts raw contract topics and payloads into typed, sequenced frontend event structures.
- **GameWebSocketServer**: Manages client connections, topic subscriptions (`events:mint`, `events:trade`, `leaderboard`, `market_prices`, `guild:<id>`), batching buffers, and throttles transmissions.

## Supported Channels & Events

- `events:mint`: Ship and item mint events
- `events:trade`: Marketplace buy/sell trades
- `events:upgrade`: Ship upgrades and progression
- `events:scan`: Nebula exploration scans and anomaly discoveries
- `guild:<guildId>`: Guild-specific action feeds
- `leaderboard`: Live ranking changes
- `market_prices`: 24h asset price & volume feeds
- `system`: System maintenance notices and alerts

## Protocol Specification

### Client to Server Messages

#### Subscribe to Channels
```json
{
  "type": "subscribe",
  "channels": ["events:mint", "market_prices", "guild:alpha"]
}
```

#### Unsubscribe from Channels
```json
{
  "type": "unsubscribe",
  "channels": ["guild:alpha"]
}
```

#### Heartbeat Ping
```json
{
  "type": "ping",
  "timestamp": 1727599000000
}
```

### Server to Client Messages

#### Subscribed Confirmation
```json
{
  "type": "subscribed",
  "channels": ["events:mint"]
}
```

#### Single Event
```json
{
  "type": "event",
  "data": {
    "id": "mint-1727599000000-abc1234",
    "type": "mint",
    "channel": "events:mint",
    "sequence": 42,
    "timestamp": 1727599000000,
    "payload": {
      "tokenId": "ship-101",
      "owner": "GCXYZ...",
      "itemType": "Frigate",
      "rarity": "Legendary",
      "timestamp": 1727599000000,
      "transactionHash": "0x123..."
    }
  }
}
```

#### Batched Events (High Frequency)
```json
{
  "type": "batch",
  "count": 2,
  "events": [...]
}
```

## Running the Server

```bash
npm install
npm run build
npm start
```
