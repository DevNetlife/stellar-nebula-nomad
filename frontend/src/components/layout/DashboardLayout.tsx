import React from 'react';

export interface DashboardLayoutProps {
  header: React.ReactNode;
  sidebar?: React.ReactNode;
  children: React.ReactNode;
}

export const DashboardLayout: React.FC<DashboardLayoutProps> = ({
  header,
  sidebar,
  children,
}) => {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        minHeight: '100vh',
        backgroundColor: '#0B1020',
        color: '#F8FAFC',
        fontFamily: 'Inter, system-ui, sans-serif',
      }}
    >
      <header
        style={{
          borderBottom: '1px solid #1E293B',
          padding: '12px 24px',
          backgroundColor: '#0F172A',
          position: 'sticky',
          top: 0,
          zIndex: 40,
        }}
      >
        {header}
      </header>

      <div style={{ display: 'flex', flex: 1 }}>
        {sidebar && (
          <aside
            style={{
              width: '260px',
              borderRight: '1px solid #1E293B',
              backgroundColor: '#0F172A',
              padding: '20px 16px',
            }}
          >
            {sidebar}
          </aside>
        )}
        <main style={{ flex: 1, padding: '24px 20px', overflowY: 'auto' }}>
          {children}
        </main>
      </div>
    </div>
  );
};
