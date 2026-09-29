import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import {
  ShipCard,
  ResourceDisplay,
  ResourceAmount,
  AddressInput,
  MintButton,
  TradeButton,
  ToastNotification,
  ConfirmDialog,
} from '../components';

describe('Frontend Component Library', () => {
  test('ShipCard renders ship name, stats, and handles selection', () => {
    const handleSelect = jest.fn();
    render(
      <ShipCard
        id="scout-1"
        name="Vanguard Explorer"
        shipClass="Scout"
        rarity="Legendary"
        level={3}
        stats={{ speed: 100, hull: 50, scanPower: 80 }}
        onSelect={handleSelect}
      />
    );

    expect(screen.getByText('Vanguard Explorer')).toBeDefined();
    expect(screen.getByText('Legendary')).toBeDefined();
    expect(screen.getByText('100')).toBeDefined();

    fireEvent.click(screen.getByRole('article'));
    expect(handleSelect).toHaveBeenCalledWith('scout-1');
  });

  test('ResourceDisplay shows resource amount and percentage', () => {
    render(
      <ResourceDisplay
        name="Dark Matter"
        amount={500}
        maxCapacity={1000}
        change24h={12.5}
      />
    );

    expect(screen.getByText('Dark Matter')).toBeDefined();
    expect(screen.getByText('500')).toBeDefined();
    expect(screen.getByText('+12.5%')).toBeDefined();
  });

  test('ResourceAmount handles input change and MAX button', () => {
    const handleChange = jest.fn();
    render(
      <ResourceAmount
        value={100}
        onChange={handleChange}
        max={1000}
        resourceName="Fuel"
      />
    );

    fireEvent.click(screen.getByText('MAX'));
    expect(handleChange).toHaveBeenCalledWith(1000);
  });

  test('AddressInput validates Stellar address', () => {
    const handleChange = jest.fn();
    const { rerender } = render(
      <AddressInput value="GBShort" onChange={handleChange} />
    );

    const input = screen.getByRole('textbox');
    fireEvent.blur(input);

    expect(
      screen.getByText(/Invalid Stellar public key format/i)
    ).toBeDefined();

    // Valid 56-character Stellar key
    const validKey = 'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';
    rerender(<AddressInput value={validKey} onChange={handleChange} />);
    expect(
      screen.queryByText(/Invalid Stellar public key format/i)
    ).toBeNull();
  });

  test('MintButton displays loading state and triggers callback', () => {
    const handleMint = jest.fn();
    const { rerender } = render(
      <MintButton onMint={handleMint} costXlm={100} isLoading={false} />
    );

    const btn = screen.getByRole('button');
    expect(btn.textContent).toContain('Mint Ship (100 XLM)');
    fireEvent.click(btn);
    expect(handleMint).toHaveBeenCalledTimes(1);

    rerender(<MintButton onMint={handleMint} costXlm={100} isLoading={true} />);
    expect(screen.getByRole('button').textContent).toContain('Minting on Soroban...');
  });

  test('TradeButton renders buy/sell action', () => {
    const handleTrade = jest.fn();
    render(
      <TradeButton
        onTrade={handleTrade}
        tradeType="buy"
        price="45"
        currency="XLM"
      />
    );

    const btn = screen.getByRole('button');
    expect(btn.textContent).toContain('Buy Now (45 XLM)');
    fireEvent.click(btn);
    expect(handleTrade).toHaveBeenCalledTimes(1);
  });

  test('ToastNotification renders and dismisses', () => {
    const handleDismiss = jest.fn();
    render(
      <ToastNotification
        id="t-1"
        title="Success"
        message="Transaction validated"
        type="success"
        onDismiss={handleDismiss}
      />
    );

    expect(screen.getByText('Success')).toBeDefined();
    fireEvent.click(screen.getByLabelText('Dismiss notification'));
    expect(handleDismiss).toHaveBeenCalledWith('t-1');
  });

  test('ConfirmDialog renders and triggers confirm / cancel callbacks', () => {
    const handleConfirm = jest.fn();
    const handleCancel = jest.fn();

    render(
      <ConfirmDialog
        isOpen={true}
        title="Dismantle Ship"
        message="Are you sure you want to dismantle this vessel?"
        confirmLabel="Dismantle"
        isDestructive={true}
        onConfirm={handleConfirm}
        onCancel={handleCancel}
      />
    );

    expect(screen.getByText('Dismantle Ship')).toBeDefined();
    fireEvent.click(screen.getByText('Dismantle'));
    expect(handleConfirm).toHaveBeenCalledTimes(1);

    fireEvent.click(screen.getByText('Cancel'));
    expect(handleCancel).toHaveBeenCalledTimes(1);
  });
});
