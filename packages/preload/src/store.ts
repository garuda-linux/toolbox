import ElectronStore from 'electron-store';
import { error, trace } from './logging.js';

// Initialize the store
const store = new ElectronStore({
  encryptionKey: 'non-security-by-obscurity',
});

export async function get(key: string): Promise<unknown> {
  try {
    const result = store.get(key);
    trace(`Store get: ${key} => ${JSON.stringify(result)}`);
    return result;
  } catch (err: unknown) {
    error(`Store get error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to get store value: ${err instanceof Error ? err.message : String(err)}`, { cause: err });
  }
}

export async function set(key: string, value: unknown): Promise<boolean> {
  try {
    store.set(key, value);
    return true;
  } catch (err: unknown) {
    error(`Store set error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to set store value: ${err instanceof Error ? err.message : String(err)}`, { cause: err });
  }
}

export async function deleteKey(key: string): Promise<boolean> {
  try {
    store.delete(key);
    return true;
  } catch (err: unknown) {
    error(`Store delete error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to delete store value: ${err instanceof Error ? err.message : String(err)}`, {
      cause: err,
    });
  }
}

export async function clear(): Promise<boolean> {
  try {
    store.clear();
    return true;
  } catch (err: unknown) {
    error(`Store clear error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to clear store: ${err instanceof Error ? err.message : String(err)}`, { cause: err });
  }
}

export async function has(key: string): Promise<boolean> {
  try {
    return store.has(key);
  } catch (err: unknown) {
    error(`Store has error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to check store key: ${err instanceof Error ? err.message : String(err)}`, { cause: err });
  }
}
