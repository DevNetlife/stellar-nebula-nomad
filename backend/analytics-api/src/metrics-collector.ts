export interface SystemHealthMetrics {
  contractStatus: 'healthy' | 'degraded' | 'maintenance';
  uptimeSeconds: number;
  transactionSuccessRate: number;
  totalTransactions24h: number;
  errorRatesByType: Record<string, number>;
  gasUsageTrends: Array<{ timestamp: string; avgGas: number; peakGas: number }>;
  storageUsageMb: number;
  maxStorageMb: number;
}

export interface UserMetrics {
  dau: number;
  mau: number;
  retentionCohorts: Array<{
    cohortDate: string;
    day1: number;
    day7: number;
    day14: number;
    day30: number;
  }>;
  userSegments: {
    casual: number;
    regular: number;
    whale: number;
  };
  conversionFunnels: Array<{
    step: string;
    users: number;
    dropoffRate: number;
  }>;
}

export interface EconomicMetrics {
  totalResourcesCreated: Record<string, number>;
  totalResourcesDestroyed: Record<string, number>;
  tradingVolume24hXlm: number;
  liquidityDepthXlm: number;
  tvlXlm: number;
  resourcePriceHistory: Array<{
    date: string;
    fuelPrice: number;
    orePrice: number;
    crystalPrice: number;
  }>;
}

export interface GameMetrics {
  shipsRegistered: number;
  nebulaScansPerformed: number;
  upgradesCompleted: number;
  averagePlayerLevel: number;
  levelDistribution: Record<string, number>;
}

export interface SocialMetrics {
  activeGuilds: number;
  bondingActivityScore: number;
  leaderboardCompetitors: number;
  guildTerritoriesControlled: number;
}

export interface FullAnalyticsData {
  timeframe: string;
  generatedAt: number;
  systemHealth: SystemHealthMetrics;
  userMetrics: UserMetrics;
  economicMetrics: EconomicMetrics;
  gameMetrics: GameMetrics;
  socialMetrics: SocialMetrics;
}

export class MetricsCollector {
  private startTime = Date.now();

  public getFullAnalytics(timeframe = 'week'): FullAnalyticsData {
    return {
      timeframe,
      generatedAt: Date.now(),
      systemHealth: this.getSystemHealth(),
      userMetrics: this.getUserMetrics(timeframe),
      economicMetrics: this.getEconomicMetrics(timeframe),
      gameMetrics: this.getGameMetrics(),
      socialMetrics: this.getSocialMetrics(),
    };
  }

  public getSystemHealth(): SystemHealthMetrics {
    return {
      contractStatus: 'healthy',
      uptimeSeconds: Math.floor((Date.now() - this.startTime) / 1000) + 86400 * 14,
      transactionSuccessRate: 99.42,
      totalTransactions24h: 38419,
      errorRatesByType: {
        INSUFFICIENT_BALANCE: 0.31,
        SLIPPAGE_EXCEEDED: 0.18,
        NONCE_MISMATCH: 0.06,
        CONTRACT_TIMEOUT: 0.03,
      },
      gasUsageTrends: [
        { timestamp: '00:00', avgGas: 12400, peakGas: 18900 },
        { timestamp: '04:00', avgGas: 11200, peakGas: 15400 },
        { timestamp: '08:00', avgGas: 18900, peakGas: 24500 },
        { timestamp: '12:00', avgGas: 22400, peakGas: 31000 },
        { timestamp: '16:00', avgGas: 28900, peakGas: 38200 },
        { timestamp: '20:00', avgGas: 24100, peakGas: 33400 },
      ],
      storageUsageMb: 142.8,
      maxStorageMb: 1024.0,
    };
  }

  public getUserMetrics(_timeframe: string): UserMetrics {
    return {
      dau: 4210,
      mau: 28450,
      retentionCohorts: [
        { cohortDate: '2026-09-01', day1: 78.4, day7: 52.1, day14: 38.6, day30: 29.2 },
        { cohortDate: '2026-09-08', day1: 81.2, day7: 55.4, day14: 41.2, day30: 31.0 },
        { cohortDate: '2026-09-15', day1: 83.5, day7: 57.0, day14: 43.8, day30: 33.5 },
        { cohortDate: '2026-09-22', day1: 84.1, day7: 58.2, day14: 44.5, day30: 34.0 },
      ],
      userSegments: {
        casual: 18200,
        regular: 8950,
        whale: 1300,
      },
      conversionFunnels: [
        { step: 'Wallet Connected', users: 28450, dropoffRate: 0 },
        { step: 'Ship Minted', users: 22100, dropoffRate: 22.3 },
        { step: 'First Nebula Scan', users: 19400, dropoffRate: 12.2 },
        { step: 'Marketplace Trade', users: 14200, dropoffRate: 26.8 },
        { step: 'Joined Guild', users: 9800, dropoffRate: 31.0 },
      ],
    };
  }

  public getEconomicMetrics(_timeframe: string): EconomicMetrics {
    return {
      totalResourcesCreated: {
        Fuel: 1248000,
        Ore: 894000,
        DarkMatter: 142000,
      },
      totalResourcesDestroyed: {
        Fuel: 984000,
        Ore: 652000,
        DarkMatter: 98000,
      },
      tradingVolume24hXlm: 489200,
      liquidityDepthXlm: 2450000,
      tvlXlm: 8750000,
      resourcePriceHistory: [
        { date: 'Day 1', fuelPrice: 1.2, orePrice: 2.5, crystalPrice: 15.0 },
        { date: 'Day 2', fuelPrice: 1.25, orePrice: 2.6, crystalPrice: 15.4 },
        { date: 'Day 3', fuelPrice: 1.18, orePrice: 2.45, crystalPrice: 16.2 },
        { date: 'Day 4', fuelPrice: 1.32, orePrice: 2.7, crystalPrice: 16.8 },
        { date: 'Day 5', fuelPrice: 1.28, orePrice: 2.65, crystalPrice: 17.1 },
        { date: 'Day 6', fuelPrice: 1.35, orePrice: 2.8, crystalPrice: 17.5 },
        { date: 'Day 7', fuelPrice: 1.4, orePrice: 2.85, crystalPrice: 18.0 },
      ],
    };
  }

  public getGameMetrics(): GameMetrics {
    return {
      shipsRegistered: 18450,
      nebulaScansPerformed: 248900,
      upgradesCompleted: 67420,
      averagePlayerLevel: 14.8,
      levelDistribution: {
        '1-5': 5400,
        '6-15': 8200,
        '16-30': 3800,
        '31+': 1050,
      },
    };
  }

  public getSocialMetrics(): SocialMetrics {
    return {
      activeGuilds: 342,
      bondingActivityScore: 89.4,
      leaderboardCompetitors: 12800,
      guildTerritoriesControlled: 86,
    };
  }

  public generateCsv(section: string): string {
    if (section === 'economic') {
      const data = this.getEconomicMetrics('week');
      let csv = 'Date,FuelPrice,OrePrice,CrystalPrice\n';
      for (const row of data.resourcePriceHistory) {
        csv += `${row.date},${row.fuelPrice},${row.orePrice},${row.crystalPrice}\n`;
      }
      return csv;
    }

    if (section === 'system') {
      const data = this.getSystemHealth();
      let csv = 'Timestamp,AvgGas,PeakGas\n';
      for (const row of data.gasUsageTrends) {
        csv += `${row.timestamp},${row.avgGas},${row.peakGas}\n`;
      }
      return csv;
    }

    // Default overview CSV
    const full = this.getFullAnalytics();
    return [
      'Metric,Value',
      `ContractStatus,${full.systemHealth.contractStatus}`,
      `TxSuccessRate,${full.systemHealth.transactionSuccessRate}%`,
      `DAU,${full.userMetrics.dau}`,
      `MAU,${full.userMetrics.mau}`,
      `TVL_XLM,${full.economicMetrics.tvlXlm}`,
      `TradingVolume24h_XLM,${full.economicMetrics.tradingVolume24hXlm}`,
      `ShipsRegistered,${full.gameMetrics.shipsRegistered}`,
      `ActiveGuilds,${full.socialMetrics.activeGuilds}`,
    ].join('\n');
  }
}
