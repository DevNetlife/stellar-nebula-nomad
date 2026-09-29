import express from 'express';
import cors from 'cors';
import dotenv from 'dotenv';
import { MetricsCollector } from './metrics-collector';
import { createAnalyticsRouter } from './routes';

dotenv.config();

export const createApp = (collector?: MetricsCollector) => {
  const app = express();
  const metrics = collector || new MetricsCollector();

  app.use(cors());
  app.use(express.json());

  app.get('/health', (_req, res) => {
    res.json({ status: 'ok', service: 'analytics-api' });
  });

  app.use('/api/analytics', createAnalyticsRouter(metrics));

  return app;
};

if (require.main === module) {
  const port = process.env.ANALYTICS_PORT || 3001;
  const app = createApp();
  app.listen(port, () => {
    // eslint-disable-next-line no-console
    console.log(`Analytics API listening on port ${port}`);
  });
}
