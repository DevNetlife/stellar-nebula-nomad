import React from 'react';

export interface BondButtonProps {
  onBond: () => void | Promise<void>;
  stakeAmount: string;
  guildName: string;
  isLoading?: boolean;
  disabled?: boolean;
}

export const BondButton: React.FC<BondButtonProps> = ({
  onBond,
  stakeAmount,
  guildName,
  isLoading = false,
  disabled = false,
}) => {
  return (
    <button
      type="button"
      onClick={onBond}
      disabled={disabled || isLoading}
      aria-busy={isLoading}
      style={{
        backgroundColor: '#EC4899',
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
        <span>Bonding Stake...</span>
      ) : (
        <span>Bond {stakeAmount} with {guildName}</span>
      )}
    </button>
  );
};
