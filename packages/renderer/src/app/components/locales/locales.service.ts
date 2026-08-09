import { computed, inject, Service, signal } from '@angular/core';
import { Logger } from '../../logging/logging';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { ChildProcess } from '../../electron-services';
import { OsInteractService } from '../task-manager/os-interact.service';
import { LoadingService } from '../loading-indicator/loading-indicator.service';

@Service()
export class LocalesService {
  readonly allLocales = signal<string[]>([]);
  readonly availableLocales = computed(() => {
    const locales: Map<string, boolean> = this.osInteractService.locales();
    return this.allLocales().filter((entry) => !locales.has(entry));
  });
  readonly loading = signal<boolean>(true);
  readonly selectedLocales = computed(() => {
    const locales: Map<string, boolean> = this.osInteractService.locales();
    return this.allLocales().filter((entry) => locales.has(entry) && locales.get(entry));
  });

  private readonly loadingService = inject(LoadingService);
  private readonly logger = Logger.getInstance();
  private readonly taskManagerService = inject(TaskManagerService);
  private readonly osInteractService = inject(OsInteractService);

  constructor() {
    void this.init();
  }

  async init(): Promise<void> {
    this.loadingService.loadingOn();
    await this.initLocales();
    this.loadingService.loadingOff();
    this.loading.set(false);
  }

  /**
   * Get the available locales from the system and process them.
   */
  async initLocales(): Promise<void> {
    const cmd = "cat /usr/share/i18n/SUPPORTED | grep '\\.UTF-8' | cut -d ' ' -f 1";
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);

    if (result.code === 0) {
      this.allLocales.set(result.stdout.trim().split('\n'));
      this.logger.info(`Found ${this.allLocales().length} available locales`);
    } else {
      this.logger.error('Failed to get available locales');
    }
  }

  /**
   * Toggle the selected state of a locale.
   * @param locales The locales to toggle (Array comes from picklist component)
   */
  async toggleOne(locales: string[]): Promise<void> {
    for (const locale of locales) {
      this.osInteractService.toggleLocale(locale);
    }
    this.osInteractService.locales().forEach((string, locale) => {
      this.logger.trace(locale);
    });
  }

  /**
   * Toggle all locales at once.
   */
  async toggleAll(): Promise<void> {
    for (const locale of this.availableLocales()) {
      this.osInteractService.toggleLocale(locale);
    }
  }
}
