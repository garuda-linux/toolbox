import { inject, Injectable, signal } from '@angular/core';
import { OsInteractService } from '../task-manager/os-interact.service';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { ConfigService } from '../config/config.service';
import { Logger } from '../../logging/logging';
import { setupWizardData } from './setup-wizard.data';
import { SetupSoftwareCategory, SetupSoftwareItem } from './interfaces';

@Injectable({
  providedIn: 'root',
})
export class SetupWizardService {
  private readonly osInteractService = inject(OsInteractService);
  private readonly taskManagerService = inject(TaskManagerService);
  private readonly configService = inject(ConfigService);
  private readonly logger = Logger.getInstance();

  readonly categories = signal<SetupSoftwareCategory[]>(setupWizardData);
  readonly nvidiaDetected = signal<boolean>(false);
  readonly applyNvidia = signal<boolean>(true);

  constructor() {
    setupWizardData.forEach((category) => {
      category.items.forEach((item) => {
        if (item.selected) {
          if (!this.osInteractService.packages().has(item.packages[0])) {
            this.toggleSoftwareItem(item);
          }
        }
      });
    });
  }

  async checkNvidia(): Promise<boolean> {
    try {
      this.logger.debug('Checking for Nvidia GPU...');
      const result = await this.taskManagerService.executeAndWaitBash('garuda-hardware-tool --nonfree --check');
      const detected = result.stdout.toLowerCase().includes('nvidia');
      this.nvidiaDetected.set(detected);
      this.logger.info(`Nvidia GPU detected: ${detected}`);
      return detected;
    } catch (error) {
      this.logger.error(`Error checking for Nvidia GPU: ${error}`);
      return false;
    }
  }

  /**
   * Toggle a software item in the setup wizard.
   * This uses the OsInteractService to manage the "wanted" state of packages.
   * @param item The item to toggle.
   */
  toggleSoftwareItem(item: SetupSoftwareItem): void {
    const newState = !this.isItemSelected(item);
    this.osInteractService.setPackage(item.packages[0], newState);

    const requiredSecondaryPackages = new Set<string>();
    this.categories().forEach((category) => {
      category.items.forEach((i) => {
        if (this.isItemSelected(i)) {
          for (let j = 1; j < i.packages.length; j++) {
            requiredSecondaryPackages.add(i.packages[j]);
          }
        }
      });
    });

    for (let j = 1; j < item.packages.length; j++) {
      const sharedPkg = item.packages[j];
      const isNeeded = requiredSecondaryPackages.has(sharedPkg);
      this.osInteractService.setPackage(sharedPkg, isNeeded);
    }
  }

  /**
   * Check if a software item is selected.
   * Only checks the primary package to prevent UI state desync when secondary packages are shared.
   */
  isItemSelected(item: SetupSoftwareItem): boolean {
    return this.osInteractService.packages().get(item.packages[0]) === true;
  }

  /**
   * Applies all pending changes to the system.
   * This includes installing selected software, applying Nvidia settings, and executing tasks.
   */
  async applyChanges(): Promise<void> {
    if (this.nvidiaDetected() && this.applyNvidia()) {
      this.taskManagerService.scheduleTask(
        this.taskManagerService.createTask(
          7,
          'setup-wizard-nvidia',
          true,
          'setupWizard.tasks.nvidia',
          'pi pi-bolt',
          'garuda-hardware-tool --nonfree',
        ),
      );
    }

    await this.configService.updateConfig('firstBoot', false);
    await this.taskManagerService.executeTasks();
  }
}
