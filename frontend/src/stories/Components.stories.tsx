import React from 'react';
import type { Meta, StoryObj } from '@storybook/react';
import {
  ShipCard,
  ResourceDisplay,
  NebulaVisualization,
  LeaderboardTable,
  AchievementBadge,
  ResourceAmount,
  AddressInput,
  DurationPicker,
  RaritySelector,
  MintButton,
  TradeButton,
  UpgradeButton,
  BondButton,
  LoadingSkeleton,
  ToastNotification,
  ConfirmDialog,
  TabContainer,
} from '../components';

const meta: Meta = {
  title: 'Stellar Nebula Nomad/Component Library',
  parameters: {
    layout: 'centered',
  },
};

export default meta;

export const AllDisplayComponents: StoryObj = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '20px', width: '400px' }}>
      <ShipCard
        id="ship-1"
        name="Nebula Phantom"
        shipClass="Frigate"
        rarity="Legendary"
        level={4}
        stats={{ speed: 85, hull: 120, scanPower: 92 }}
      />
      <ResourceDisplay name="Exotic Fuel" amount={14200} maxCapacity={20000} change24h={5.4} icon="⛽" />
      <NebulaVisualization sectorName="Sector Omega-9" density={78} anomalyDetected={true} colorTheme="magenta" />
      <AchievementBadge
        id="ach-1"
        title="First Warp"
        description="Jump into an uncharted nebula sector"
        unlocked={true}
        unlockedAt="2026-09-29"
      />
      <LeaderboardTable
        entries={[
          { rank: 1, playerId: 'GCX1234567890ABCDEF', score: 98000, guild: 'Nova' },
          { rank: 2, playerId: 'GDF9876543210FEDCBA', score: 84200, guild: 'Eclipse' },
        ]}
      />
    </div>
  ),
};

export const AllInputComponents: StoryObj = {
  render: () => {
    const [amt, setAmt] = React.useState(100);
    const [addr, setAddr] = React.useState('');
    const [dur, setDur] = React.useState(4);
    const [rarity, setRarity] = React.useState<any>('Rare');

    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', width: '360px' }}>
        <ResourceAmount value={amt} onChange={setAmt} max={5000} />
        <AddressInput value={addr} onChange={setAddr} />
        <DurationPicker valueHours={dur} onChange={setDur} />
        <RaritySelector selected={rarity} onChange={setRarity} />
      </div>
    );
  },
};

export const AllActionComponents: StoryObj = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', width: '320px' }}>
      <MintButton onMint={() => alert('Mint clicked')} costXlm={75} />
      <TradeButton onTrade={() => alert('Trade clicked')} tradeType="buy" price="25" />
      <UpgradeButton onUpgrade={() => alert('Upgrade clicked')} targetLevel={5} />
      <BondButton onBond={() => alert('Bond clicked')} stakeAmount="500 XLM" guildName="Stellar Vanguard" />
    </div>
  ),
};

export const AllFeedbackAndLayoutComponents: StoryObj = {
  render: () => (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', width: '380px' }}>
      <ToastNotification id="1" title="Transaction Confirmed" message="Ship upgraded successfully" type="success" onDismiss={() => {}} />
      <LoadingSkeleton count={3} height="18px" />
      <TabContainer
        tabs={[
          { id: 'fleet', label: 'Fleet', content: <div>Fleet Active View</div> },
          { id: 'cargo', label: 'Cargo', content: <div>Cargo Hold: 500 units</div> },
        ]}
      />
    </div>
  ),
};
