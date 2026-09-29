import React from 'react';

export interface ShipCardProps {
  id: string;
  name: string;
  shipClass: 'Scout' | 'Frigate' | 'Cruiser' | 'Dreadnought';
  rarity: 'Common' | 'Rare' | 'Epic' | 'Legendary';
  level: number;
  stats: {
    speed: number;
    hull: number;
    scanPower: number;
  };
  onSelect?: (id: string) => void;
}

export const ShipCard: React.FC<ShipCardProps> = ({
  id,
  name,
  shipClass,
  rarity,
  level,
  stats,
  onSelect,
}) => {
  const rarityColors: Record<string, string> = {
    Common: '#94A3B8',
    Rare: '#38BDF8',
    Epic: '#A855F7',
    Legendary: '#F59E0B',
  };

  const color = rarityColors[rarity] || '#94A3B8';

  return (
    <div
      role="article"
      aria-label={`Ship: ${name}, ${rarity} ${shipClass}`}
      tabIndex={0}
      onClick={() => onSelect?.(id)}
      onKeyDown={(e) => e.key === 'Enter' && onSelect?.(id)}
      style={{
        backgroundColor: '#1E293B',
        border: `2px solid ${color}`,
        borderRadius: '12px',
        padding: '16px',
        color: '#F8FAFC',
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
        cursor: onSelect ? 'pointer' : 'default',
        transition: 'transform 0.2s ease, box-shadow 0.2s ease',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h4 style={{ margin: 0, fontSize: '1.1rem', color: '#F8FAFC' }}>{name}</h4>
        <span
          style={{
            backgroundColor: `${color}22`,
            color,
            padding: '2px 8px',
            borderRadius: '9999px',
            fontSize: '0.75rem',
            fontWeight: 700,
          }}
        >
          {rarity}
        </span>
      </div>

      <div style={{ fontSize: '0.85rem', color: '#94A3B8' }}>
        {shipClass} • Level {level}
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '8px', fontSize: '0.75rem' }}>
        <div style={{ backgroundColor: '#0F172A', padding: '8px', borderRadius: '6px', textAlign: 'center' }}>
          <div style={{ color: '#64748B' }}>Speed</div>
          <div style={{ fontWeight: 700, color: '#38BDF8' }}>{stats.speed}</div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '8px', borderRadius: '6px', textAlign: 'center' }}>
          <div style={{ color: '#64748B' }}>Hull</div>
          <div style={{ fontWeight: 700, color: '#10B981' }}>{stats.hull}</div>
        </div>
        <div style={{ backgroundColor: '#0F172A', padding: '8px', borderRadius: '6px', textAlign: 'center' }}>
          <div style={{ color: '#64748B' }}>Scan</div>
          <div style={{ fontWeight: 700, color: '#A855F7' }}>{stats.scanPower}</div>
        </div>
      </div>
    </div>
  );
};
