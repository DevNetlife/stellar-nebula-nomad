import React from 'react';

export interface AchievementBadgeProps {
  id: string;
  title: string;
  description: string;
  unlocked: boolean;
  unlockedAt?: string;
  icon?: string;
}

export const AchievementBadge: React.FC<AchievementBadgeProps> = ({
  title,
  description,
  unlocked,
  unlockedAt,
  icon = '🏆',
}) => {
  return (
    <div
      role="status"
      aria-label={`${title} achievement: ${unlocked ? 'Unlocked' : 'Locked'}`}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '12px',
        padding: '12px',
        borderRadius: '8px',
        backgroundColor: unlocked ? '#1E293B' : '#0F172A',
        border: `1px solid ${unlocked ? '#3B82F6' : '#334155'}`,
        opacity: unlocked ? 1 : 0.6,
      }}
    >
      <div
        style={{
          fontSize: '1.75rem',
          filter: unlocked ? 'none' : 'grayscale(100%)',
        }}
      >
        {icon}
      </div>
      <div style={{ display: 'flex', flexDirection: 'column' }}>
        <span style={{ fontWeight: 700, color: unlocked ? '#F8FAFC' : '#94A3B8', fontSize: '0.9rem' }}>
          {title}
        </span>
        <span style={{ color: '#64748B', fontSize: '0.75rem' }}>{description}</span>
        {unlocked && unlockedAt && (
          <span style={{ color: '#10B981', fontSize: '0.7rem', marginTop: '2px' }}>
            Unlocked on {unlockedAt}
          </span>
        )}
      </div>
    </div>
  );
};
