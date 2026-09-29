import { AppState, AppStateStatus } from 'react-native';
import { SecureStorage } from '../utils/SecureStorage';

const AUTO_LOCK_MS = 5 * 60 * 1000;

type LockCallback = () => void;

export class LockManager {
  private static lockCallback: LockCallback | null = null;
  private static lastActiveAt: number = Date.now();
  private static isLocked: boolean = false;
  private static subscription: any = null;
  private static lockTimer: ReturnType<typeof setTimeout> | null = null;

  public static init(onLock: LockCallback): void {
    this.lockCallback = onLock;
    this.lastActiveAt = Date.now();

    this.subscription = AppState.addEventListener('change', (state: AppStateStatus) => {
      if (state === 'active') {
        this.checkAutoLock();
      } else if (state === 'background' || state === 'inactive') {
        this.lastActiveAt = Date.now();
        this.scheduleLockTimer();
      }
    });
  }

  public static recordActivity(): void {
    this.lastActiveAt = Date.now();
    this.isLocked = false;
    if (this.lockTimer) {
      clearTimeout(this.lockTimer);
      this.lockTimer = null;
    }
  }

  public static lock(): void {
    if (!this.isLocked) {
      this.isLocked = true;
      this.lockCallback?.();
    }
  }

  public static getIsLocked(): boolean {
    return this.isLocked;
  }

  public static destroy(): void {
    this.subscription?.remove();
    if (this.lockTimer) {
      clearTimeout(this.lockTimer);
      this.lockTimer = null;
    }
  }

  private static checkAutoLock(): void {
    const elapsed = Date.now() - this.lastActiveAt;
    if (elapsed >= AUTO_LOCK_MS) {
      this.lock();
    }
  }

  private static scheduleLockTimer(): void {
    if (this.lockTimer) {
      clearTimeout(this.lockTimer);
    }
    this.lockTimer = setTimeout(() => {
      this.lock();
    }, AUTO_LOCK_MS);
  }
}

export class AntiTamper {
  public static async detectTampering(): Promise<{ tampered: boolean; reason?: string }> {
    try {
      const testKey = '__tamper_probe__';
      const testValue = `probe_${Date.now()}`;
      await SecureStorage.set(testKey, testValue);
      const readBack = await SecureStorage.get(testKey);
      await SecureStorage.delete(testKey);

      if (readBack !== testValue) {
        return { tampered: true, reason: 'Secure storage integrity check failed' };
      }

      return { tampered: false };
    } catch (err: any) {
      return { tampered: true, reason: `Secure storage inaccessible: ${err?.message}` };
    }
  }
}
