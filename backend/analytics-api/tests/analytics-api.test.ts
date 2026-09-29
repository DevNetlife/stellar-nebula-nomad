import request from 'supertest';
import { createApp } from '../src/index';
import { MetricsCollector } from '../src/metrics-collector';

describe('Analytics API', () => {
  const app = createApp(new MetricsCollector());
  const adminSecret = 'stellar-admin-secret-2026';

  test('health endpoint is accessible without auth', async () => {
    const res = await request(app).get('/health');
    expect(res.status).toBe(200);
    expect(res.body.status).toBe('ok');
  });

  test('blocks unauthenticated access to analytics overview', async () => {
    const res = await request(app).get('/api/analytics/overview');
    expect(res.status).toBe(403);
    expect(res.body.error).toContain('Admin role required');
  });

  test('allows authenticated access with x-admin-key', async () => {
    const res = await request(app)
      .get('/api/analytics/overview')
      .set('x-admin-key', adminSecret);
    expect(res.status).toBe(200);
    expect(res.body.systemHealth).toBeDefined();
    expect(res.body.userMetrics).toBeDefined();
    expect(res.body.economicMetrics).toBeDefined();
    expect(res.body.gameMetrics).toBeDefined();
    expect(res.body.socialMetrics).toBeDefined();
  });

  test('allows authenticated access with Bearer token', async () => {
    const res = await request(app)
      .get('/api/analytics/system-health')
      .set('Authorization', `Bearer ${adminSecret}`);
    expect(res.status).toBe(200);
    expect(res.body.contractStatus).toBe('healthy');
    expect(res.body.transactionSuccessRate).toBeGreaterThan(95);
  });

  test('exports CSV report for economic metrics', async () => {
    const res = await request(app)
      .get('/api/analytics/export-csv?section=economic')
      .set('x-admin-key', adminSecret);
    expect(res.status).toBe(200);
    expect(res.headers['content-type']).toContain('text/csv');
    expect(res.text).toContain('Date,FuelPrice,OrePrice,CrystalPrice');
  });
});
