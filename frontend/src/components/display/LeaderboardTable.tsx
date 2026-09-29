import React from 'react';

export interface LeaderboardRow {
  rank: number;
  playerId: string;
  score: number;
  guild?: string;
}

export interface LeaderboardTableProps {
  entries: LeaderboardRow[];
  currentPlayerId?: string;
}

export const LeaderboardTable: React.FC<LeaderboardTableProps> = ({
  entries,
  currentPlayerId,
}) => {
  return (
    <div style={{ overflowX: 'auto', width: '100%' }}>
      <table
        role="table"
        aria-label="Leaderboard Rankings"
        style={{
          width: '100%',
          borderCollapse: 'collapse',
          fontSize: '0.875rem',
          textAlign: 'left',
        }}
      >
        <thead>
          <tr style={{ borderBottom: '1px solid #334155', color: '#94A3B8' }}>
            <th style={{ padding: '10px 12px' }}>Rank</th>
            <th style={{ padding: '10px 12px' }}>Explorer</th>
            <th style={{ padding: '10px 12px' }}>Guild</th>
            <th style={{ padding: '10px 12px', textAlign: 'right' }}>Score</th>
          </tr>
        </thead>
        <tbody>
          {entries.map((entry) => {
            const isCurrent = currentPlayerId && entry.playerId.toLowerCase() === currentPlayerId.toLowerCase();
            return (
              <tr
                key={entry.rank}
                style={{
                  backgroundColor: isCurrent ? 'rgba(56, 189, 248, 0.1)' : 'transparent',
                  borderBottom: '1px solid #1E293B',
                }}
              >
                <td style={{ padding: '10px 12px', fontWeight: 700, color: entry.rank <= 3 ? '#F59E0B' : '#94A3B8' }}>
                  #{entry.rank}
                </td>
                <td style={{ padding: '10px 12px', color: '#F8FAFC', fontFamily: 'monospace' }}>
                  {entry.playerId.slice(0, 6)}...{entry.playerId.slice(-4)}
                </td>
                <td style={{ padding: '10px 12px', color: '#38BDF8' }}>
                  {entry.guild || '-'}
                </td>
                <td style={{ padding: '10px 12px', textAlign: 'right', fontWeight: 600, color: '#10B981' }}>
                  {entry.score.toLocaleString()}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
};
