import { Service } from '@angular/core';
import { clear, deleteKey, get, has, set } from './electron-api-utils.js';
import { Logger } from '../logging/logging';

export interface StoreOptions {
  key?: string;
}

export class Store {
  private logger = Logger.getInstance();

  constructor(private options: StoreOptions = {}) {}

  async get<T = unknown>(key: string): Promise<T | undefined> {
    return (await get(key)) as T | undefined;
  }

  async set<T = unknown>(key: string, value: T): Promise<void> {
    await set(key, value);
  }

  async delete(key: string): Promise<void> {
    await deleteKey(key);
  }

  async clear(): Promise<void> {
    await clear();
  }

  async has(key: string): Promise<boolean> {
    return await has(key);
  }
}

@Service()
export class ElectronStoreService {
  async load(options: StoreOptions = {}): Promise<Store> {
    return new Store(options);
  }

  createStore(options: StoreOptions = {}): Store {
    return new Store(options);
  }
}
