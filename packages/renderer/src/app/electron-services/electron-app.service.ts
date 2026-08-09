import { Service } from '@angular/core';
import { windowRelaunch } from './electron-api-utils';

@Service()
export class ElectronAppService {
  async relaunch(): Promise<void> {
    windowRelaunch();
  }
}
