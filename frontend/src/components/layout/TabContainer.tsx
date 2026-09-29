import React, { useState } from 'react';

export interface TabItem {
  id: string;
  label: string;
  content: React.ReactNode;
}

export interface TabContainerProps {
  tabs: TabItem[];
  defaultTabId?: string;
}

export const TabContainer: React.FC<TabContainerProps> = ({
  tabs,
  defaultTabId,
}) => {
  const [activeTab, setActiveTab] = useState<string>(
    defaultTabId || (tabs.length > 0 ? tabs[0].id : '')
  );

  const currentTab = tabs.find((t) => t.id === activeTab);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', width: '100%' }}>
      <div
        role="tablist"
        style={{
          display: 'flex',
          gap: '4px',
          borderBottom: '1px solid #334155',
          marginBottom: '16px',
        }}
      >
        {tabs.map((tab) => {
          const isActive = tab.id === activeTab;
          return (
            <button
              key={tab.id}
              role="tab"
              aria-selected={isActive}
              aria-controls={`panel-${tab.id}`}
              id={`tab-${tab.id}`}
              onClick={() => setActiveTab(tab.id)}
              style={{
                background: 'transparent',
                border: 'none',
                borderBottom: isActive ? '2px solid #38BDF8' : '2px solid transparent',
                color: isActive ? '#38BDF8' : '#94A3B8',
                padding: '10px 16px',
                fontSize: '0.9rem',
                fontWeight: 600,
                cursor: 'pointer',
                transition: 'all 0.2s',
              }}
            >
              {tab.label}
            </button>
          );
        })}
      </div>

      {currentTab && (
        <div
          role="tabpanel"
          id={`panel-${currentTab.id}`}
          aria-labelledby={`tab-${currentTab.id}`}
        >
          {currentTab.content}
        </div>
      )}
    </div>
  );
};
