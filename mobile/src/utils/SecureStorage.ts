import * as SecureStore from 'expo-secure-store';

const KEY_PREFIX = 'stellar_nebula_';

export class SecureStorage {
  private static buildKey(key: string): string {
    return `${KEY_PREFIX}${key}`;
  }

  public static async set(key: string, value: string): Promise<void> {
    await SecureStore.setItemAsync(this.buildKey(key), value, {
      keychainAccessible: SecureStore.WHEN_UNLOCKED_THIS_DEVICE_ONLY,
    });
  }

  public static async get(key: string): Promise<string | null> {
    return SecureStore.getItemAsync(this.buildKey(key));
  }

  public static async delete(key: string): Promise<void> {
    await SecureStore.deleteItemAsync(this.buildKey(key));
  }

  public static async setJson<T>(key: string, value: T): Promise<void> {
    await this.set(key, JSON.stringify(value));
  }

  public static async getJson<T>(key: string): Promise<T | null> {
    const raw = await this.get(key);
    if (!raw) return null;
    try {
      return JSON.parse(raw) as T;
    } catch {
      return null;
    }
  }

  public static async storeEncryptedAccount(publicKey: string, encryptedSecret: string): Promise<void> {
    await this.set('account_public_key', publicKey);
    await this.set('account_encrypted_secret', encryptedSecret);
  }

  public static async getEncryptedAccount(): Promise<{ publicKey: string; encryptedSecret: string } | null> {
    const publicKey = await this.get('account_public_key');
    const encryptedSecret = await this.get('account_encrypted_secret');
    if (!publicKey || !encryptedSecret) return null;
    return { publicKey, encryptedSecret };
  }

  public static async clearAccount(): Promise<void> {
    await this.delete('account_public_key');
    await this.delete('account_encrypted_secret');
    await this.delete('pin_hash');
    await this.delete('biometrics_enabled');
  }

  public static async storePinHash(pinHash: string): Promise<void> {
    await this.set('pin_hash', pinHash);
  }

  public static async getPinHash(): Promise<string | null> {
    return this.get('pin_hash');
  }

  public static async setBiometricsEnabled(enabled: boolean): Promise<void> {
    await this.set('biometrics_enabled', enabled ? '1' : '0');
  }

  public static async isBiometricsEnabled(): Promise<boolean> {
    const val = await this.get('biometrics_enabled');
    return val === '1';
  }
}
