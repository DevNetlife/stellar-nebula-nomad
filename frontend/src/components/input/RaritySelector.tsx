import React from 'react';

export type RarityType = 'All' | 'Common' | 'Rare' | 'Epic' | 'Legendary';

export interface RaritySelectorProps {
  selected: RarityType;
  onChange: (rarity: RarityType) => void;
  includeAll?: boolean;
}

export const RaritySelector: React.FC<RaritySelectorProps> = ({
  selected,
  onChange,
  includeAll = true,
}) => {
  const rarities: RarityType[] = includeAll
    ? ['All', 'Common', 'Rare', 'Epic', 'Legendary']
    : ['Common', 'Rare', 'Epic', 'Legendary'];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
      <label style={{ fontSize: '0.8rem', color: '#94A3B8' }}>Filter by Rarity</label>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px' }}>
        {rarities.map((r) => {
          const isSelected = selected === r;
          return (
            <button
              key={r}
              type="button"
              onClick={() => onChange(r)}
              style={{
                backgroundColor: isSelected ? '#6366F1' : '#0F172A',
                color: isSelected ? '#FFFFFF' : '#94A3B8',
                border: '1px solid #334155',
                borderRadius: '6px',
                padding: '4px 10px',
                fontSize: '0.75rem',
                cursor: 'pointer',
                fontWeight: 600,
              }}
            >
              {r}
            </button>
          );
        })}
      </div>
    </div>
  );
};
