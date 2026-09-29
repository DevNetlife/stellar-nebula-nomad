import { OfflineManager } from '../utils/offline';

describe('OfflineManager', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  test('detects online status and queues transactions', () => {
    expect(OfflineManager.isOnline()).toBe(true);

    const queued = OfflineManager.queueTransaction('/api/trade', { tokenId: 'ship-1' });
    expect(queued.id).toBeDefined();
    expect(queued.endpoint).toBe('/api/trade');

    const list = OfflineManager.getQueuedTransactions();
    expect(list.length).toBe(1);
    expect(list[0].endpoint).toBe('/api/trade');
  });

  test('caches read only data and retrieves before expiry', () => {
    const data = { ships: ['ship-1', 'ship-2'] };
    OfflineManager.cacheReadOnlyData('user_ships', data);

    const cached = OfflineManager.getCachedReadOnlyData('user_ships', 10000);
    expect(cached).toEqual(data);
  });
});
