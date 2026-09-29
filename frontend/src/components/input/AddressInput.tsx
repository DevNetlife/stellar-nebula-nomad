import React, { useState } from 'react';

export interface AddressInputProps {
  value: string;
  onChange: (val: string) => void;
  label?: string;
  placeholder?: string;
}

export const AddressInput: React.FC<AddressInputProps> = ({
  value,
  onChange,
  label = 'Stellar Account Address',
  placeholder = 'G...',
}) => {
  const [touched, setTouched] = useState(false);

  // Stellar G-address validation: 56 chars starting with G
  const isValidStellarAddress = (addr: string) => {
    if (!addr) return false;
    return addr.length === 56 && addr.startsWith('G');
  };

  const isInvalid = touched && value.length > 0 && !isValidStellarAddress(value);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '4px', width: '100%' }}>
      <label htmlFor="stellar-addr" style={{ fontSize: '0.8rem', color: '#94A3B8' }}>
        {label}
      </label>
      <input
        id="stellar-addr"
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value.trim())}
        onBlur={() => setTouched(true)}
        placeholder={placeholder}
        aria-invalid={isInvalid}
        style={{
          backgroundColor: '#0F172A',
          border: `1px solid ${isInvalid ? '#EF4444' : '#334155'}`,
          borderRadius: '6px',
          padding: '8px 12px',
          color: '#F8FAFC',
          fontSize: '0.85rem',
          fontFamily: 'monospace',
        }}
      />
      {isInvalid && (
        <span style={{ color: '#EF4444', fontSize: '0.75rem' }}>
          Invalid Stellar public key format (must be 56 characters starting with G)
        </span>
      )}
    </div>
  );
};
