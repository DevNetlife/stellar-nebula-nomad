import React from 'react';

export interface EconomicMetricsData {
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

export interface EconomicMetricsPanelProps {
  data: EconomicMetricsData;
  onDrillDown: () => void;
}

export const EconomicMetricsPanel: React.FC<EconomicMetricsPanelProps> = ({
  data,
  onDrillDown,
}) => {
  return (
    <div
      style={{
        backgroundColor: '#1E293B',
        borderRadius: '12px',
        padding: '20px',
        border: '1px solid #334155',
        display: 'flex',
        flexDirection: 'column',
        gap: '16px',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h3 style={{ margin: 0, color: '#F8FAFC', fontSize: '1.1rem' }}>Economic Metrics & TVL</h3>
        <button
          onClick={onDrillDown}
          style={{
            background: 'none',
            border: 'none',
            color: '#38BDF8',
            fontSize: '0.85rem',
            cursor: 'pointer',
            fontWeight: 500,
          }}
        >
          Markets ↗
        </button>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '12px' }}>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>TVL (XLM)</div>
          <div style={{ color: '#10B981', fontSize: '1.25rem', fontWeight: 700 }}>
            {(data.tvlXlm / 1000000).toFixed(2)}M
          </div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>24h Volume (XLM)</div>
          <div style={{ color: '#F8FAFC', fontSize: '1.25rem', fontWeight: 700 }}>
            {(data.tradingVolume24hXlm / 1000).toFixed(0)}K
          </div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Liquidity Depth</div>
          <div style={{ color: '#38BDF8', fontSize: '1.25rem', fontWeight: 700 }}>
            {(data.liquidityDepthXlm / 1000000).toFixed(2)}M
          </div>
        </div>
      </div>

      <div>
        <div style={{ color: '#94A3B8', fontSize: '0.8rem', marginBottom: '8px', fontWeight: 500 }}>
          Resource Price Trend (Crystal / Ore / Fuel)
        </div>
        <div style={{ height: '80px', display: 'flex', alignItems: 'flex-end', gap: '6px' }}>
          {data.resourcePriceHistory.map((item, idx) => (
            <div
              key={idx}
              style={{
                flex: 1,
                display: 'flex',
                gap: '2px',
                alignItems: 'flex-end',
                height: '100%',
              }}
            >
              <div
                title={`Fuel: ${item.fuelPrice}`}
                style={{
                  flex: 1,
                  height: `${(item.fuelPrice / 20) * 100}%`,
                  backgroundColor: '#38BDF8',
                  borderRadius: '2px 2px 0 0',
                }}
              />
              <div
                title={`Ore: ${item.orePrice}`}
                style={{
                  flex: 1,
                  height: `${(item.orePrice / 20) * 100}%`,
                  backgroundColor: '#F59E0B',
                  borderRadius: '2px 2px 0 0',
                }}
              />
              <div
                title={`Crystal: ${item.crystalPrice}`}
                style={{
                  flex: 1,
                  height: `${(item.crystalPrice / 20) * 100}%`,
                  backgroundColor: '#A855F7',
                  borderRadius: '2px 2px 0 0',
                }}
              />
            </div>
          ))}
        </div>
        <div style={{ display: 'flex', justifyContent: 'center', gap: '12px', fontSize: '0.75rem', marginTop: '6px' }}>
          <span style={{ color: '#38BDF8' }}>■ Fuel</span>
          <span style={{ color: '#F59E0B' }}>■ Ore</span>
          <span style={{ color: '#A855F7' }}>■ Crystal</span>
        </div>
      </div>
    </div>
  );
};
