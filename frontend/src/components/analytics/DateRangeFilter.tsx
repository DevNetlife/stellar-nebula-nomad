import React from 'react';

export type DateRange = 'today' | 'week' | 'month' | 'year' | 'custom';

export interface DateRangeFilterProps {
  selectedRange: DateRange;
  onChange: (range: DateRange) => void;
  customStartDate?: string;
  customEndDate?: string;
  onCustomChange?: (start: string, end: string) => void;
}

export const DateRangeFilter: React.FC<DateRangeFilterProps> = ({
  selectedRange,
  onChange,
  customStartDate,
  customEndDate,
  onCustomChange,
}) => {
  const ranges: { id: DateRange; label: string }[] = [
    { id: 'today', label: 'Today' },
    { id: 'week', label: '7 Days' },
    { id: 'month', label: '30 Days' },
    { id: 'year', label: '1 Year' },
    { id: 'custom', label: 'Custom' },
  ];

  return (
    <div
      style={{
        display: 'flex',
        flexWrap: 'wrap',
        alignItems: 'center',
        gap: '8px',
      }}
    >
      <div
        style={{
          display: 'inline-flex',
          backgroundColor: '#1E293B',
          borderRadius: '8px',
          padding: '4px',
          border: '1px solid #334155',
        }}
      >
        {ranges.map((r) => {
          const isActive = selectedRange === r.id;
          return (
            <button
              key={r.id}
              onClick={() => onChange(r.id)}
              style={{
                background: isActive ? '#3B82F6' : 'transparent',
                color: isActive ? '#FFFFFF' : '#94A3B8',
                border: 'none',
                padding: '6px 14px',
                borderRadius: '6px',
                fontSize: '0.85rem',
                fontWeight: 600,
                cursor: 'pointer',
                transition: 'all 0.2s',
              }}
            >
              {r.label}
            </button>
          );
        })}
      </div>

      {selectedRange === 'custom' && (
        <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <input
            type="date"
            value={customStartDate || ''}
            onChange={(e) => onCustomChange?.(e.target.value, customEndDate || '')}
            style={{
              backgroundColor: '#0F172A',
              color: '#F8FAFC',
              border: '1px solid #334155',
              borderRadius: '6px',
              padding: '6px 8px',
              fontSize: '0.85rem',
            }}
          />
          <span style={{ color: '#64748B' }}>to</span>
          <input
            type="date"
            value={customEndDate || ''}
            onChange={(e) => onCustomChange?.(customStartDate || '', e.target.value)}
            style={{
              backgroundColor: '#0F172A',
              color: '#F8FAFC',
              border: '1px solid #334155',
              borderRadius: '6px',
              padding: '6px 8px',
              fontSize: '0.85rem',
            }}
          />
        </div>
      )}
    </div>
  );
};
