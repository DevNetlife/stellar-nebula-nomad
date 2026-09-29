import React from 'react';

export interface TradeButtonProps {
  onTrade: () => void | Promise<void>;
  tradeType: 'buy' | 'sell';
  price: string;
  currency?: string;
  isLoading?: boolean;
  disabled?: boolean;
}

export const TradeButton: React.FC<TradeButtonProps> = ({
  onTrade,
  tradeType,
  price,
  currency = 'XLM',
  isLoading = false,
  disabled = false,
}) => {
  const isBuy = tradeType === 'buy';

  return (
    <button
      type="button"
      onClick={onTrade}
      disabled={disabled || isLoading}
      aria-busy={isLoading}
      style={{
        backgroundColor: isBuy ? '#3B82F6' : '#8B5CF6',
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
        <span>Processing {tradeType}...</span>
      ) : (
        <span>
          {isBuy ? 'Buy Now' : 'List Item'} ({price} {currency})
        </span>
      )}
    </button>
  );
};
