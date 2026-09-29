import React from 'react';

export interface GameMetricsData {
  shipsRegistered: number;
  nebulaScansPerformed: number;
  upgradesCompleted: number;
  averagePlayerLevel: number;
  levelDistribution: Record<string, number>;
}

export interface GameMetricsPanelProps {
  data: GameMetricsData;
  onDrillDown: () => void;
}

export const GameMetricsPanel: React.FC<GameMetricsPanelProps> = ({ data, onDrillDown }) => {
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
        <h3 style={{ margin: 0, color: '#F8FAFC', fontSize: '1.1rem' }}>Game Progression & Telemetry</h3>
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

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Ships Registered</div>
          <div style={{ color: '#38BDF8', fontSize: '1.25rem', fontWeight: 700 }}>
            {data.shipsRegistered.toLocaleString()}
          </div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Nebula Scans</div>
          <div style={{ color: '#10B981', fontSize: '1.25rem', fontWeight: 700 }}>
            {data.nebulaScansPerformed.toLocaleString()}
          </div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Upgrades Completed</div>
          <div style={{ color: '#F59E0B', fontSize: '1.25rem', fontWeight: 700 }}>
            {data.upgradesCompleted.toLocaleString()}
          </div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Average Level</div>
          <div style={{ color: '#A855F7', fontSize: '1.25rem', fontWeight: 700 }}>
            Lvl {data.averagePlayerLevel}
          </div>
        </div>
      </div>

      <div>
        <div style={{ color: '#94A3B8', fontSize: '0.8rem', marginBottom: '8px', fontWeight: 500 }}>
          Player Level Distribution
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
          {Object.entries(data.levelDistribution).map(([bracket, count]) => {
            const pct = Math.round((count / data.shipsRegistered) * 100);
            return (
              <div key={bracket} style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '0.75rem' }}>
                <span style={{ width: '40px', color: '#94A3B8' }}>{bracket}</span>
                <div style={{ flex: 1, backgroundColor: '#0F172A', height: '8px', borderRadius: '4px', overflow: 'hidden' }}>
                  <div style={{ width: `${pct}%`, height: '100%', backgroundColor: '#6366F1' }} />
                </div>
                <span style={{ width: '35px', textAlign: 'right', color: '#CBD5E1' }}>{pct}%</span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
