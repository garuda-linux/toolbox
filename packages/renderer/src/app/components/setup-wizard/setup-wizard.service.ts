import { inject, Injectable, signal } from '@angular/core';
import { OsInteractService } from '../task-manager/os-interact.service';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { Logger } from '../../logging/logging';
import { setupWizardData } from './setup-wizard.data';
import { SetupSoftwareCategory, SetupSoftwareItem } from './interfaces';

@Injectable({
  providedIn: 'root',
})
export class SetupWizardService {
  private readonly osInteractService = inject(OsInteractService);
  private readonly taskManagerService = inject(TaskManagerService);
  private readonly logger = Logger.getInstance();

  readonly categories = signal<SetupSoftwareCategory[]>(setupWizardData);

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

  /**
   * Helper to find an item by ID
   */
  private findItemById(id: string): SetupSoftwareItem | undefined {
    for (const category of this.categories()) {
      const item = category.items.find((i) => i.id === id);
      if (item) return item;
    }
    return undefined;
  }

  /**
   * Toggle a software item in the setup wizard.
   * This uses the OsInteractService to manage the "wanted" state of packages.
   * @param item The item to toggle.
   */
  toggleSoftwareItem(item: SetupSoftwareItem): void {
    const newState = !this.isItemSelected(item);

    if (newState) {
      if (item.id === 'libreOfficeFresh') {
        const stillItem = this.findItemById('libreOfficeStill');
        if (stillItem && this.isItemSelected(stillItem)) {
          this.osInteractService.setPackage(stillItem.packages[0], false);
        }
      } else if (item.id === 'libreOfficeStill') {
        const freshItem = this.findItemById('libreOfficeFresh');
        if (freshItem && this.isItemSelected(freshItem)) {
          this.osInteractService.setPackage(freshItem.packages[0], false);
        }
      }
    }

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
   * This includes installing selected software and executing tasks.
   */
  async applyChanges(): Promise<void> {
    const rootCommands: string[] = [];
    const userCommands: string[] = [];

    this.categories().forEach((category) => {
      category.items.forEach((item) => {
        if (this.isItemSelected(item)) {
          if (item.rootCommands) {
            rootCommands.push(...item.rootCommands);
          }
          if (item.userCommands) {
            userCommands.push(...item.userCommands);
          }
        }
      });
    });

    if (rootCommands.length > 0) {
      this.taskManagerService.scheduleTask(
        this.taskManagerService.createTask(
          50,
          'setup-commands-root',
          true,
          'Setup Commands',
          'pi pi-cog',
          rootCommands.join('\n'),
        ),
      );
    }

    if (userCommands.length > 0) {
      this.taskManagerService.scheduleTask(
        this.taskManagerService.createTask(
          51,
          'setup-commands-user',
          false,
          'Prepare Commands',
          'pi pi-cog',
          userCommands.join('\n'),
        ),
      );
    }

    await this.taskManagerService.executeTasks();
  }
}
