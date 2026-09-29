import React from 'react';

export interface SocialMetricsData {
  activeGuilds: number;
  bondingActivityScore: number;
  leaderboardCompetitors: number;
  guildTerritoriesControlled: number;
}

export interface SocialMetricsPanelProps {
  data: SocialMetricsData;
  onDrillDown: () => void;
}

export const SocialMetricsPanel: React.FC<SocialMetricsPanelProps> = ({ data, onDrillDown }) => {
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
        <h3 style={{ margin: 0, color: '#F8FAFC', fontSize: '1.1rem' }}>Social, Guilds & Bonding</h3>
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
          Guilds ↗
        </button>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Active Guilds</div>
          <div style={{ color: '#F8FAFC', fontSize: '1.25rem', fontWeight: 700 }}>
            {data.activeGuilds}
          </div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Bonding Score</div>
          <div style={{ color: '#10B981', fontSize: '1.25rem', fontWeight: 700 }}>
            {data.bondingActivityScore}/100
          </div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Leaderboard Players</div>
          <div style={{ color: '#38BDF8', fontSize: '1.25rem', fontWeight: 700 }}>
            {data.leaderboardCompetitors.toLocaleString()}
          </div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '12px', borderRadius: '8px' }}>
          <div style={{ color: '#94A3B8', fontSize: '0.75rem' }}>Sectors Controlled</div>
          <div style={{ color: '#A855F7', fontSize: '1.25rem', fontWeight: 700 }}>
            {data.guildTerritoriesControlled}
          </div>
        </div>
      </div>
    </div>
  );
};
