import { useAppStore } from './index';

describe('useAppStore', () => {
  beforeEach(() => {
    useAppStore.getState().reset();
  });

  it('initializes with default state', () => {
    const state = useAppStore.getState();
    expect(state.account).toBeNull();
    expect(state.balance).toBe('0.0000000');
    expect(state.network).toBe('testnet');
    expect(state.isAuthenticated).toBe(false);
    expect(state.isLocked).toBe(false);
    expect(state.biometricsEnabled).toBe(false);
  });

  it('updates account and authenticates', () => {
    useAppStore.getState().setAccount({ publicKey: 'GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFDAGORAcctTest' });
    const state = useAppStore.getState();
    expect(state.account?.publicKey).toBe('GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFDAGORAcctTest');
    expect(state.isAuthenticated).toBe(true);
    expect(state.isLocked).toBe(false);
  });

  it('updates balance and network', () => {
    useAppStore.getState().setBalance('150.5000000');
    useAppStore.getState().setNetwork('public');
    const state = useAppStore.getState();
    expect(state.balance).toBe('150.5000000');
    expect(state.network).toBe('public');
  });

  it('toggles lock state and biometrics flag', () => {
    useAppStore.getState().setIsLocked(true);
    expect(useAppStore.getState().isLocked).toBe(true);

    useAppStore.getState().setBiometricsEnabled(true);
    expect(useAppStore.getState().biometricsEnabled).toBe(true);

    useAppStore.getState().recordActivity();
    expect(useAppStore.getState().isLocked).toBe(false);
  });

  it('resets state properly', () => {
    useAppStore.getState().setAccount({ publicKey: 'GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFDAGORAcctTest' });
    useAppStore.getState().setBalance('25.0000000');
    useAppStore.getState().reset();

    const state = useAppStore.getState();
    expect(state.account).toBeNull();
    expect(state.balance).toBe('0.0000000');
    expect(state.isAuthenticated).toBe(false);
  });
});
