import React from 'react';

export interface UpgradeButtonProps {
  onUpgrade: () => void | Promise<void>;
  targetLevel: number;
  costDescription?: string;
  isLoading?: boolean;
  disabled?: boolean;
}

export const UpgradeButton: React.FC<UpgradeButtonProps> = ({
  onUpgrade,
  targetLevel,
  costDescription = '100 Fuel + 50 Ore',
  isLoading = false,
  disabled = false,
}) => {
  return (
    <button
      type="button"
      onClick={onUpgrade}
      disabled={disabled || isLoading}
      aria-busy={isLoading}
      style={{
        backgroundColor: '#F59E0B',
        color: '#FFFFFF',
        border: 'none',
        borderRadius: '8px',
        padding: '10px 18px',
        fontSize: '0.9rem',
        fontWeight: 700,
        cursor: disabled || isLoading ? 'not-allowed' : 'pointer',
        opacity: disabled || isLoading ? 0.6 : 1,
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '6px',
      }}
    >
      {isLoading ? (
        <span>Upgrading...</span>
      ) : (
        <span>Upgrade to Lvl {targetLevel} ({costDescription})</span>
      )}
    </button>
  );
};
