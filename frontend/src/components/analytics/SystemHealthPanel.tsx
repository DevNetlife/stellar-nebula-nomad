import React from 'react';

export interface SystemHealthData {
  contractStatus: 'healthy' | 'degraded' | 'maintenance';
  uptimeSeconds: number;
  transactionSuccessRate: number;
  totalTransactions24h: number;
  errorRatesByType: Record<string, number>;
  gasUsageTrends: Array<{ timestamp: string; avgGas: number; peakGas: number }>;
  storageUsageMb: number;
  maxStorageMb: number;
}

export interface SystemHealthPanelProps {
  data: SystemHealthData;
  onDrillDown: () => void;
}

export const SystemHealthPanel: React.FC<SystemHealthPanelProps> = ({ data, onDrillDown }) => {
  const statusColor =
    data.contractStatus === 'healthy'
      ? '#10B981'
      : data.contractStatus === 'degraded'
      ? '#F59E0B'
      : '#EF4444';

  const maxGas = Math.max(...data.gasUsageTrends.map((g) => g.peakGas), 40000);

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
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <h3 style={{ margin: 0, color: '#F8FAFC', fontSize: '1.1rem' }}>System Health</h3>
          <span
            style={{
              padding: '2px 8px',
              borderRadius: '9999px',
              backgroundColor: `${statusColor}22`,
              color: statusColor,
              fontSize: '0.75rem',
              fontWeight: 600,
              textTransform: 'uppercase',
            }}
          >
            {data.contractStatus}
          </span>
        </div>
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
          Details ↗
        </button>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(130px, 1fr))', gap: '12px' }}>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Success Rate</div>
          <div style={{ color: '#10B981', fontSize: '1.25rem', fontWeight: 700 }}>
            {data.transactionSuccessRate}%
          </div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>24h Transactions</div>
          <div style={{ color: '#F8FAFC', fontSize: '1.25rem', fontWeight: 700 }}>
            {data.totalTransactions24h.toLocaleString()}
          </div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Storage Used</div>
          <div style={{ color: '#F8FAFC', fontSize: '1.25rem', fontWeight: 700 }}>
            {data.storageUsageMb} MB
          </div>
        </div>
      </div>

      <div>
        <div style={{ color: '#94A3B8', fontSize: '0.8rem', marginBottom: '8px', fontWeight: 500 }}>
          Gas Usage Trends (Soroban CPU/Mem Units)
        </div>
        <div style={{ display: 'flex', alignItems: 'flex-end', height: '90px', gap: '8px', paddingTop: '10px' }}>
          {data.gasUsageTrends.map((trend, idx) => {
            const heightPct = Math.round((trend.avgGas / maxGas) * 100);
            return (
              <div
                key={idx}
                style={{
                  flex: 1,
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  height: '100%',
                  justifyContent: 'flex-end',
                }}
              >
                <div
                  title={`Avg: ${trend.avgGas}, Peak: ${trend.peakGas}`}
                  style={{
                    width: '100%',
                    height: `${heightPct}%`,
                    minHeight: '6px',
                    backgroundColor: '#6366F1',
                    borderRadius: '4px 4px 0 0',
                    transition: 'height 0.3s ease',
                  }}
                />
                <span style={{ color: '#64748B', fontSize: '0.65rem', marginTop: '4px' }}>
                  {trend.timestamp}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
