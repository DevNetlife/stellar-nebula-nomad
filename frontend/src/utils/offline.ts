export interface QueuedTransaction {
  id: string;
  endpoint: string;
  payload: Record<string, unknown>;
  timestamp: number;
  retryCount: number;
}

const STORAGE_QUEUE_KEY = 'stellar_nebula_tx_queue';
const CACHE_STORE_KEY = 'stellar_nebula_offline_cache';

export class OfflineManager {
  private static isOnlineStatus: boolean =
    typeof navigator !== 'undefined' ? navigator.onLine : true;
  private static listeners: Set<(online: boolean) => void> = new Set();
  private static isInitialized = false;

  public static init(): void {
    if (this.isInitialized || typeof window === 'undefined') return;
    this.isInitialized = true;

    window.addEventListener('online', () => {
      this.isOnlineStatus = true;
      this.notifyListeners(true);
      this.syncQueuedTransactions();
    });

    window.addEventListener('offline', () => {
      this.isOnlineStatus = false;
      this.notifyListeners(false);
    });

    if ('serviceWorker' in navigator) {
      navigator.serviceWorker.addEventListener('message', (event) => {
        if (event.data?.type === 'SYNC_TRANSACTIONS_TRIGGERED') {
          this.syncQueuedTransactions();
        }
      });
    }
  }

  public static isOnline(): boolean {
    if (typeof navigator !== 'undefined') {
      return navigator.onLine;
    }
    return this.isOnlineStatus;
  }

  public static onNetworkChange(callback: (online: boolean) => void): () => void {
    this.init();
    this.listeners.add(callback);
    callback(this.isOnline());
    return () => {
      this.listeners.delete(callback);
    };
  }

  private static notifyListeners(online: boolean): void {
    for (const listener of this.listeners) {
      listener(online);
    }
  }

  public static getQueuedTransactions(): QueuedTransaction[] {
    if (typeof localStorage === 'undefined') return [];
    try {
      const data = localStorage.getItem(STORAGE_QUEUE_KEY);
      return data ? JSON.parse(data) : [];
    } catch {
      return [];
    }
  }

  public static queueTransaction(
    endpoint: string,
    payload: Record<string, unknown>
  ): QueuedTransaction {
    const queue = this.getQueuedTransactions();
    const item: QueuedTransaction = {
      id: `tx-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      endpoint,
      payload,
      timestamp: Date.now(),
      retryCount: 0,
    };
    queue.push(item);
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_QUEUE_KEY, JSON.stringify(queue));
    }

    // Register background sync if supported
    if ('serviceWorker' in navigator && 'SyncManager' in window) {
      navigator.serviceWorker.ready.then((reg: any) => {
        reg.sync?.register('sync-stellar-transactions').catch(() => {});
      });
    }

    return item;
  }

  public static async syncQueuedTransactions(): Promise<{
    succeeded: number;
    failed: number;
  }> {
    const queue = this.getQueuedTransactions();
    if (queue.length === 0) return { succeeded: 0, failed: 0 };

    let succeeded = 0;
    let failed = 0;
    const remaining: QueuedTransaction[] = [];

    for (const tx of queue) {
      try {
        const response = await fetch(tx.endpoint, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(tx.payload),
        });

        if (response.ok) {
          succeeded++;
        } else {
          tx.retryCount++;
          if (tx.retryCount < 5) {
            remaining.push(tx);
          }
          failed++;
        }
      } catch {
        tx.retryCount++;
        remaining.push(tx);
        failed++;
      }
    }

    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_QUEUE_KEY, JSON.stringify(remaining));
    }

    return { succeeded, failed };
  }

  public static cacheReadOnlyData<T>(key: string, data: T): void {
    if (typeof localStorage === 'undefined') return;
    try {
      const envelope = {
        data,
        cachedAt: Date.now(),
      };
      localStorage.setItem(`${CACHE_STORE_KEY}_${key}`, JSON.stringify(envelope));
    } catch {
      // ignore quota errors
    }
  }

  public static getCachedReadOnlyData<T>(key: string, maxAgeMs = 3600000): T | null {
    if (typeof localStorage === 'undefined') return null;
    try {
      const raw = localStorage.getItem(`${CACHE_STORE_KEY}_${key}`);
      if (!raw) return null;
      const envelope = JSON.parse(raw);
      if (Date.now() - envelope.cachedAt > maxAgeMs) {
        return null;
      }
      return envelope.data as T;
    } catch {
      return null;
    }
  }

  public static async requestPushNotifications(): Promise<boolean> {
    if (typeof window === 'undefined' || !('Notification' in window)) {
      return false;
    }

    const permission = await Notification.requestPermission();
    return permission === 'granted';
  }
}
