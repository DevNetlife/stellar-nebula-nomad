import React from 'react';

export interface MintButtonProps {
  onMint: () => void | Promise<void>;
  isLoading?: boolean;
  costXlm?: number;
  disabled?: boolean;
}

export const MintButton: React.FC<MintButtonProps> = ({
  onMint,
  isLoading = false,
  costXlm = 50,
  disabled = false,
}) => {
  return (
    <button
      type="button"
      onClick={onMint}
      disabled={disabled || isLoading}
      aria-busy={isLoading}
      style={{
        backgroundColor: '#10B981',
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
        gap: '8px',
      }}
    >
      {isLoading ? (
        <span>Minting on Soroban...</span>
      ) : (
        <span>Mint Ship ({costXlm} XLM)</span>
      )}
    </button>
  );
};
