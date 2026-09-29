import { Router, Response } from 'express';
import { requireAdminRole, AuthenticatedRequest } from './auth';
import { MetricsCollector } from './metrics-collector';

export const createAnalyticsRouter = (collector: MetricsCollector): Router => {
  const router = Router();

  // Protect all analytics endpoints with admin role verification
  router.use(requireAdminRole);

  router.get('/overview', (req: AuthenticatedRequest, res: Response) => {
    const timeframe = (req.query.timeframe as string) || 'week';
    const data = collector.getFullAnalytics(timeframe);
    res.json(data);
  });

  router.get('/system-health', (_req: AuthenticatedRequest, res: Response) => {
    const data = collector.getSystemHealth();
    res.json(data);
  });

  router.get('/user-metrics', (req: AuthenticatedRequest, res: Response) => {
    const timeframe = (req.query.timeframe as string) || 'week';
    const data = collector.getUserMetrics(timeframe);
    res.json(data);
  });

  router.get('/economic-metrics', (req: AuthenticatedRequest, res: Response) => {
    const timeframe = (req.query.timeframe as string) || 'week';
    const data = collector.getEconomicMetrics(timeframe);
    res.json(data);
  });

  router.get('/game-metrics', (_req: AuthenticatedRequest, res: Response) => {
    const data = collector.getGameMetrics();
    res.json(data);
  });

  router.get('/social-metrics', (_req: AuthenticatedRequest, res: Response) => {
    const data = collector.getSocialMetrics();
    res.json(data);
  });

  router.get('/export-csv', (req: AuthenticatedRequest, res: Response) => {
    const section = (req.query.section as string) || 'overview';
    const csvData = collector.generateCsv(section);
    res.setHeader('Content-Type', 'text/csv');
    res.setHeader('Content-Disposition', `attachment; filename=analytics_${section}.csv`);
    res.send(csvData);
  });

  return router;
};
