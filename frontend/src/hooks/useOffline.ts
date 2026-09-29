import React, { useState, useEffect, useCallback } from 'react';
import {
  OfflineManager,
  QueuedTransaction,
} from '../utils/offline';

export interface UseOfflineReturn {
  isOnline: boolean;
  isOffline: boolean;
  queueCount: number;
  queuedTransactions: QueuedTransaction[];
  queueTransaction: (endpoint: string, payload: Record<string, unknown>) => QueuedTransaction;
  syncNow: () => Promise<{ succeeded: number; failed: number }>;
  requestNotificationPermission: () => Promise<boolean>;
}

export function useOffline(): UseOfflineReturn {
  const [isOnline, setIsOnline] = useState<boolean>(OfflineManager.isOnline());
  const [queuedTransactions, setQueuedTransactions] = useState<QueuedTransaction[]>(
    OfflineManager.getQueuedTransactions()
  );

  useEffect(() => {
    OfflineManager.init();
    const unsub = OfflineManager.onNetworkChange((online) => {
      setIsOnline(online);
      setQueuedTransactions(OfflineManager.getQueuedTransactions());
    });
    return unsub;
  }, []);

  const queueTransaction = useCallback(
    (endpoint: string, payload: Record<string, unknown>) => {
      const item = OfflineManager.queueTransaction(endpoint, payload);
      setQueuedTransactions(OfflineManager.getQueuedTransactions());
      return item;
    },
    []
  );

  const syncNow = useCallback(async () => {
    const result = await OfflineManager.syncQueuedTransactions();
    setQueuedTransactions(OfflineManager.getQueuedTransactions());
    return result;
  }, []);

  const requestNotificationPermission = useCallback(async () => {
    return await OfflineManager.requestPushNotifications();
  }, []);

  return {
    isOnline,
    isOffline: !isOnline,
    queueCount: queuedTransactions.length,
    queuedTransactions,
    queueTransaction,
    syncNow,
    requestNotificationPermission,
  };
}

export const OfflineIndicator: React.FC<{ className?: string }> = ({ className = '' }) => {
  const { isOffline, queueCount, syncNow } = useOffline();

  if (!isOffline && queueCount === 0) {
    return null;
  }

  return (
    <div
      role="alert"
      aria-live="assertive"
      className={`offline-banner ${className}`}
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '8px 16px',
        backgroundColor: isOffline ? '#B91C1C' : '#D97706',
        color: '#FFFFFF',
        fontSize: '0.875rem',
        fontWeight: 500,
        borderRadius: '6px',
        margin: '8px 0',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
        <span style={{ fontSize: '1rem' }}>{isOffline ? '⚡' : '🔄'}</span>
        <span>
          {isOffline
            ? `You are offline. Transactions will be queued locally (${queueCount} pending).`
            : `${queueCount} offline transactions queued for synchronization.`}
        </span>
      </div>
      {!isOffline && queueCount > 0 && (
        <button
          onClick={() => syncNow()}
          style={{
            backgroundColor: 'rgba(255, 255, 255, 0.2)',
            border: 'none',
            color: '#FFFFFF',
            padding: '4px 10px',
            borderRadius: '4px',
            cursor: 'pointer',
            fontWeight: 600,
          }}
        >
          Sync Now
        </button>
      )}
    </div>
  );
};
