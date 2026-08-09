import { Service } from '@angular/core';
import { configNotifyChange } from './electron-api-utils.js';

@Service()
export class ElectronConfigService {
  async notifyConfigChange(key: string, value: unknown): Promise<boolean> {
    try {
      return configNotifyChange(key, value);
    } catch (error) {
      console.error('Config notify change error:', error);
      return false;
    }
  }
}

// Standalone function for direct use
export async function notifyConfigChange(key: string, value: unknown): Promise<boolean> {
  try {
    return configNotifyChange(key, value);
  } catch (error) {
    console.error('Config notify change error:', error);
    return false;
  }
}
