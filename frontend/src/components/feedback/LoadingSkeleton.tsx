import React from 'react';

export interface LoadingSkeletonProps {
  width?: string | number;
  height?: string | number;
  borderRadius?: string | number;
  count?: number;
}

export const LoadingSkeleton: React.FC<LoadingSkeletonProps> = ({
  width = '100%',
  height = '20px',
  borderRadius = '6px',
  count = 1,
}) => {
  const items = Array.from({ length: count });

  return (
    <div
      role="status"
      aria-label="Loading..."
      style={{ display: 'flex', flexDirection: 'column', gap: '8px', width: '100%' }}
    >
      {items.map((_, i) => (
        <div
          key={i}
          style={{
            width,
            height,
            borderRadius,
            backgroundColor: '#1E293B',
            opacity: 0.7,
            animation: 'pulse 1.5s cubic-bezier(0.4, 0, 0.6, 1) infinite',
          }}
        />
      ))}
    </div>
  );
};
