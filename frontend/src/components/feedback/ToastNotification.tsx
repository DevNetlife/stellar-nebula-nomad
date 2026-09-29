import React from 'react';

export interface ToastProps {
  id: string;
  title: string;
  message: string;
  type?: 'info' | 'success' | 'warning' | 'error';
  onDismiss: (id: string) => void;
}

export const ToastNotification: React.FC<ToastProps> = ({
  id,
  title,
  message,
  type = 'info',
  onDismiss,
}) => {
  const typeColors = {
    info: '#3B82F6',
    success: '#10B981',
    warning: '#F59E0B',
    error: '#EF4444',
  };

  const color = typeColors[type];

  return (
    <div
      role="alert"
      style={{
        display: 'flex',
        alignItems: 'flex-start',
        justifyContent: 'space-between',
        padding: '12px 16px',
        backgroundColor: '#1E293B',
        borderLeft: `4px solid ${color}`,
        borderRadius: '8px',
        boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.4)',
        color: '#F8FAFC',
        minWidth: '280px',
        maxWidth: '400px',
      }}
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
        <span style={{ fontWeight: 700, fontSize: '0.875rem' }}>{title}</span>
        <span style={{ color: '#94A3B8', fontSize: '0.8rem' }}>{message}</span>
      </div>
      <button
        onClick={() => onDismiss(id)}
        aria-label="Dismiss notification"
        style={{
          background: 'none',
          border: 'none',
          color: '#64748B',
          cursor: 'pointer',
          fontSize: '1rem',
          padding: '2px',
        }}
      >
        ✕
      </button>
    </div>
  );
};
