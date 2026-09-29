import React from 'react';

export interface DurationPickerProps {
  valueHours: number;
  onChange: (hours: number) => void;
  options?: number[];
}

export const DurationPicker: React.FC<DurationPickerProps> = ({
  valueHours,
  onChange,
  options = [1, 4, 8, 12, 24, 72],
}) => {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
      <label style={{ fontSize: '0.8rem', color: '#94A3B8' }}>Expedition Duration</label>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '8px' }}>
        {options.map((h) => {
          const isSelected = valueHours === h;
          return (
            <button
              key={h}
              type="button"
              onClick={() => onChange(h)}
              style={{
                backgroundColor: isSelected ? '#3B82F6' : '#1E293B',
                color: isSelected ? '#FFFFFF' : '#94A3B8',
                border: '1px solid #334155',
                borderRadius: '6px',
                padding: '6px 12px',
                fontSize: '0.8rem',
                cursor: 'pointer',
                fontWeight: 600,
              }}
            >
              {h >= 24 ? `${h / 24}d` : `${h}h`}
            </button>
          );
        })}
      </div>
    </div>
  );
};
