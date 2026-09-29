import React from 'react';

export interface UserMetricsData {
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

export interface UserMetricsPanelProps {
  data: UserMetricsData;
  onDrillDown: () => void;
}

export const UserMetricsPanel: React.FC<UserMetricsPanelProps> = ({ data, onDrillDown }) => {
  const totalUsers = data.userSegments.casual + data.userSegments.regular + data.userSegments.whale;

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
        <h3 style={{ margin: 0, color: '#F8FAFC', fontSize: '1.1rem' }}>User Engagement & Retention</h3>
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
          Cohorts ↗
        </button>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Daily Active Users (DAU)</div>
          <div style={{ color: '#38BDF8', fontSize: '1.35rem', fontWeight: 700 }}>
            {data.dau.toLocaleString()}
          </div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Monthly Active Users (MAU)</div>
          <div style={{ color: '#A855F7', fontSize: '1.35rem', fontWeight: 700 }}>
            {data.mau.toLocaleString()}
          </div>
        </div>
      </div>

      <div>
        <div style={{ color: '#94A3B8', fontSize: '0.8rem', marginBottom: '8px', fontWeight: 500 }}>
          User Segmentation
        </div>
        <div
          style={{
            display: 'flex',
            height: '14px',
            borderRadius: '9999px',
            overflow: 'hidden',
            backgroundColor: '#0F172A',
          }}
        >
          <div
            title={`Casual: ${data.userSegments.casual}`}
            style={{
              width: `${(data.userSegments.casual / totalUsers) * 100}%`,
              backgroundColor: '#38BDF8',
            }}
          />
          <div
            title={`Regular: ${data.userSegments.regular}`}
            style={{
              width: `${(data.userSegments.regular / totalUsers) * 100}%`,
              backgroundColor: '#818CF8',
            }}
          />
          <div
            title={`Whale: ${data.userSegments.whale}`}
            style={{
              width: `${(data.userSegments.whale / totalUsers) * 100}%`,
              backgroundColor: '#F59E0B',
            }}
          />
        </div>
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            fontSize: '0.75rem',
            color: '#94A3B8',
            marginTop: '6px',
          }}
        >
          <span>🔵 Casual ({Math.round((data.userSegments.casual / totalUsers) * 100)}%)</span>
          <span>🟣 Regular ({Math.round((data.userSegments.regular / totalUsers) * 100)}%)</span>
          <span>🟠 Whale ({Math.round((data.userSegments.whale / totalUsers) * 100)}%)</span>
        </div>
      </div>

      <div>
        <div style={{ color: '#94A3B8', fontSize: '0.8rem', marginBottom: '8px', fontWeight: 500 }}>
          Conversion Funnel
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
          {data.conversionFunnels.slice(0, 3).map((step, idx) => (
            <div
              key={idx}
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                fontSize: '0.75rem',
                color: '#CBD5E1',
                padding: '4px 8px',
                backgroundColor: '#0F172A',
                borderRadius: '4px',
              }}
            >
              <span>{step.step}</span>
              <span style={{ fontWeight: 600 }}>{step.users.toLocaleString()}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
