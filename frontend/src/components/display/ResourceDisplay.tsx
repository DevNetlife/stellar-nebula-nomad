import React from 'react';

export interface ResourceDisplayProps {
  name: string;
  amount: number;
  icon?: string;
  change24h?: number;
  maxCapacity?: number;
}

export const ResourceDisplay: React.FC<ResourceDisplayProps> = ({
  name,
  amount,
  icon = '💎',
  change24h,
  maxCapacity,
}) => {
  const percentage = maxCapacity ? Math.min(Math.round((amount / maxCapacity) * 100), 100) : null;

  return (
    <div
      role="region"
      aria-label={`${name} Resource: ${amount}`}
      style={{
        backgroundColor: '#0F172A',
        border: '1px solid #334155',
        borderRadius: '8px',
        padding: '12px 16px',
        display: 'flex',
        flexDirection: 'column',
        gap: '6px',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span style={{ fontSize: '1.2rem' }}>{icon}</span>
          <span style={{ color: '#94A3B8', fontSize: '0.85rem', fontWeight: 600 }}>{name}</span>
        </div>
        {change24h !== undefined && (
          <span
            style={{
              fontSize: '0.75rem',
              fontWeight: 600,
              color: change24h >= 0 ? '#10B981' : '#EF4444',
            }}
          >
            {change24h >= 0 ? `+${change24h}%` : `${change24h}%`}
          </span>
        )}
      </div>

      <div style={{ display: 'flex', alignItems: 'baseline', gap: '6px' }}>
        <span style={{ fontSize: '1.25rem', fontWeight: 700, color: '#F8FAFC' }}>
          {amount.toLocaleString()}
        </span>
        {maxCapacity && (
          <span style={{ fontSize: '0.75rem', color: '#64748B' }}>
            / {maxCapacity.toLocaleString()}
          </span>
        )}
      </div>

      {percentage !== null && (
        <div style={{ backgroundColor: '#1E293B', height: '6px', borderRadius: '3px', overflow: 'hidden' }}>
          <div
            style={{
              width: `${percentage}%`,
              height: '100%',
              backgroundColor: percentage > 90 ? '#EF4444' : '#38BDF8',
            }}
          />
        </div>
      )}
    </div>
  );
};
