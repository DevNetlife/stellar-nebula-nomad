/// <reference lib="webworker" />

declare const self: ServiceWorkerGlobalScope;

const CACHE_VERSION = 'v1.1.0';
const STATIC_CACHE_NAME = `stellar-nebula-static-${CACHE_VERSION}`;
const DATA_CACHE_NAME = `stellar-nebula-data-${CACHE_VERSION}`;
const OFFLINE_FALLBACK_URL = '/offline.html';

const PRECACHE_ASSETS = [
  '/',
  '/index.html',
  '/manifest.json',
  '/manifest.webmanifest',
  '/offline.html',
  '/icons/icon.svg',
  '/icons/icon-maskable.svg',
  '/icons/icon-192x192.png',
  '/icons/icon-512x512.png',
];

self.addEventListener('install', (event: ExtendableEvent) => {
  event.waitUntil(
    caches
      .open(STATIC_CACHE_NAME)
      .then((cache) => cache.addAll(PRECACHE_ASSETS))
      .then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', (event: ExtendableEvent) => {
  event.waitUntil(
    caches
      .keys()
      .then((cacheNames) =>
        Promise.all(
          cacheNames.map((cacheName) => {
            if (
              cacheName !== STATIC_CACHE_NAME &&
              cacheName !== DATA_CACHE_NAME &&
              cacheName.startsWith('stellar-nebula-')
            ) {
              return caches.delete(cacheName);
            }
            return Promise.resolve();
          })
        )
      )
      .then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event: FetchEvent) => {
  const request = event.request;
  const url = new URL(request.url);

  // Skip non-GET requests
  if (request.method !== 'GET') {
    return;
  }

  // Skip WebSocket connections
  if (url.protocol === 'ws:' || url.protocol === 'wss:') {
    return;
  }

  // Soroban mutations / raw transaction submits should never be cached
  if (url.pathname.includes('/rpc/sendTransaction') || url.pathname.includes('/submit')) {
    return;
  }

  // 1. Static asset strategy: Cache-first
  if (
    request.destination === 'image' ||
    request.destination === 'style' ||
    request.destination === 'font' ||
    request.destination === 'script' ||
    url.pathname.startsWith('/icons/') ||
    url.pathname.endsWith('.png') ||
    url.pathname.endsWith('.svg') ||
    url.pathname.endsWith('.woff2')
  ) {
    event.respondWith(
      caches.match(request).then((cachedResponse) => {
        if (cachedResponse) {
          return cachedResponse;
        }
        return fetch(request).then((networkResponse) => {
          if (networkResponse && networkResponse.status === 200) {
            const clone = networkResponse.clone();
            caches.open(STATIC_CACHE_NAME).then((cache) => cache.put(request, clone));
          }
          return networkResponse;
        });
      })
    );
    return;
  }

  // 2. Read-only API queries: Network-first with cache fallback
  if (url.pathname.startsWith('/api/') || url.pathname.startsWith('/analytics/')) {
    event.respondWith(
      fetch(request)
        .then((networkResponse) => {
          if (networkResponse && networkResponse.status === 200) {
            const clone = networkResponse.clone();
            caches.open(DATA_CACHE_NAME).then((cache) => cache.put(request, clone));
          }
          return networkResponse;
        })
        .catch(() => {
          return caches.match(request).then((cached) => {
            if (cached) return cached;
            return new Response(
              JSON.stringify({ error: 'Offline', cached: true }),
              {
                status: 503,
                headers: { 'Content-Type': 'application/json' },
              }
            );
          });
        })
    );
    return;
  }

  // 3. Navigation requests: Stale-while-revalidate / Network fallback to offline.html
  if (request.mode === 'navigate') {
    event.respondWith(
      fetch(request)
        .then((response) => {
          const clone = response.clone();
          caches.open(STATIC_CACHE_NAME).then((cache) => cache.put(request, clone));
          return response;
        })
        .catch(async () => {
          const cachedPage = await caches.match(request);
          if (cachedPage) return cachedPage;
          const fallback = await caches.match(OFFLINE_FALLBACK_URL);
          return fallback || new Response('Offline', { status: 503 });
        })
    );
    return;
  }

  // Default: Network with cache fallback
  event.respondWith(
    caches.match(request).then((cached) => cached || fetch(request))
  );
});

// Background Sync
self.addEventListener('sync', (event: any) => {
  if (event.tag === 'sync-stellar-transactions') {
    event.waitUntil(syncQueuedTransactions());
  }
});

async function syncQueuedTransactions(): Promise<void> {
  const clients = await self.clients.matchAll();
  for (const client of clients) {
    client.postMessage({ type: 'SYNC_TRANSACTIONS_TRIGGERED' });
  }
}

// Push Notifications
self.addEventListener('push', (event: PushEvent) => {
  let title = 'Stellar Nebula Nomad';
  let body = 'New exploration anomaly or trade event detected!';
  let data: Record<string, unknown> = {};

  if (event.data) {
    try {
      const payload = event.data.json();
      title = payload.title || title;
      body = payload.body || body;
      data = payload.data || {};
    } catch {
      body = event.data.text();
    }
  }

  const options: NotificationOptions = {
    body,
    icon: '/icons/icon-192x192.png',
    badge: '/icons/icon-96x96.png',
    vibrate: [100, 50, 100],
    data,
  };

  event.waitUntil(self.registration.showNotification(title, options));
});

self.addEventListener('notificationclick', (event: NotificationEvent) => {
  event.notification.close();
  event.waitUntil(
    self.clients.matchAll({ type: 'window', includeUncontrolled: true }).then((clientsArr) => {
      for (const client of clientsArr) {
        if ('focus' in client) {
          return client.focus();
        }
      }
      return self.clients.openWindow('/');
    })
  );
});

export {};
