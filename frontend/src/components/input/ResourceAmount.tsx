import React from 'react';

export interface ResourceAmountProps {
  value: number;
  onChange: (val: number) => void;
  min?: number;
  max?: number;
  step?: number;
  resourceName?: string;
  error?: string;
}

export const ResourceAmount: React.FC<ResourceAmountProps> = ({
  value,
  onChange,
  min = 0,
  max = 1000000,
  step = 1,
  resourceName = 'XLM',
  error,
}) => {
  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const parsed = Number(e.target.value);
    if (!isNaN(parsed)) {
      onChange(Math.max(min, Math.min(max, parsed)));
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px', width: '100%' }}>
      <label htmlFor="resource-input" style={{ fontSize: '0.8rem', color: '#94A3B8' }}>
        Amount ({resourceName})
      </label>
      <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
        <input
          id="resource-input"
          type="number"
          value={value}
          onChange={handleChange}
          min={min}
          max={max}
          step={step}
          aria-invalid={!!error}
          aria-describedby={error ? 'resource-error' : undefined}
          style={{
            flex: 1,
            backgroundColor: '#0F172A',
            border: `1px solid ${error ? '#EF4444' : '#334155'}`,
            borderRadius: '6px',
            padding: '8px 12px',
            color: '#F8FAFC',
            fontSize: '0.9rem',
          }}
        />
        <button
          type="button"
          onClick={() => onChange(max)}
          style={{
            backgroundColor: '#1E293B',
            border: '1px solid #334155',
            color: '#38BDF8',
            padding: '8px 12px',
            borderRadius: '6px',
            fontSize: '0.75rem',
            fontWeight: 700,
            cursor: 'pointer',
          }}
        >
          MAX
        </button>
      </div>
      {error && (
        <span id="resource-error" style={{ color: '#EF4444', fontSize: '0.75rem' }}>
          {error}
        </span>
      )}
    </div>
  );
};
