import React from 'react';

export interface NebulaVisualizationProps {
  sectorName: string;
  density: number;
  anomalyDetected: boolean;
  colorTheme?: 'cyan' | 'magenta' | 'amber';
}

export const NebulaVisualization: React.FC<NebulaVisualizationProps> = ({
  sectorName,
  density,
  anomalyDetected,
  colorTheme = 'cyan',
}) => {
  const themeGradients = {
    cyan: 'radial-gradient(circle at 50% 50%, rgba(56, 189, 248, 0.4), rgba(15, 23, 42, 0.95))',
    magenta: 'radial-gradient(circle at 50% 50%, rgba(217, 70, 239, 0.4), rgba(15, 23, 42, 0.95))',
    amber: 'radial-gradient(circle at 50% 50%, rgba(245, 158, 11, 0.4), rgba(15, 23, 42, 0.95))',
  };

  return (
    <div
      role="img"
      aria-label={`Nebula sector: ${sectorName}, density ${density}%`}
      style={{
        width: '100%',
        height: '180px',
        borderRadius: '12px',
        background: themeGradients[colorTheme],
        border: '1px solid #334155',
        position: 'relative',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'space-between',
        padding: '16px',
        boxSizing: 'border-box',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{ fontWeight: 700, color: '#F8FAFC' }}>{sectorName}</span>
        {anomalyDetected && (
          <span
            style={{
              backgroundColor: '#EF4444',
              color: '#FFFFFF',
              fontSize: '0.7rem',
              padding: '2px 6px',
              borderRadius: '4px',
              fontWeight: 700,
            }}
          >
            ANOMALY
          </span>
        )}
      </div>

      <div style={{ color: '#94A3B8', fontSize: '0.8rem' }}>
        Nebula Density: {density}%
      </div>
    </div>
  );
};
