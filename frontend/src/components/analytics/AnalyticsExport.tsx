import React, { useState } from 'react';

export interface AnalyticsExportProps {
  onExportCsv: (section: string) => Promise<void> | void;
  isLoading?: boolean;
}

export const AnalyticsExport: React.FC<AnalyticsExportProps> = ({
  onExportCsv,
  isLoading = false,
}) => {
  const [selectedSection, setSelectedSection] = useState('overview');

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
      <select
        value={selectedSection}
        onChange={(e) => setSelectedSection(e.target.value)}
        style={{
          backgroundColor: '#1E293B',
          color: '#F8FAFC',
          border: '1px solid #334155',
          borderRadius: '6px',
          padding: '6px 12px',
          fontSize: '0.85rem',
        }}
      >
        <option value="overview">All Metrics (Summary)</option>
        <option value="system">System Health Trends</option>
        <option value="economic">Economic Price History</option>
      </select>
      <button
        onClick={() => onExportCsv(selectedSection)}
        disabled={isLoading}
        style={{
          backgroundColor: '#2563EB',
          color: '#FFFFFF',
          border: 'none',
          padding: '6px 14px',
          borderRadius: '6px',
          fontSize: '0.85rem',
          fontWeight: 600,
          cursor: isLoading ? 'not-allowed' : 'pointer',
          opacity: isLoading ? 0.7 : 1,
          display: 'flex',
          alignItems: 'center',
          gap: '6px',
        }}
      >
        <span>⬇</span>
        <span>{isLoading ? 'Exporting...' : 'Export CSV'}</span>
      </button>
    </div>
  );
};
