import { effect, inject } from '@angular/core';
import { DOCUMENT } from '@angular/common';
import { Logger } from '../../logging/logging';
import { ConfigService } from '../config/config.service';
import { AppSettings } from '../config/interfaces';

export class ThemeService {
  private readonly configService = inject(ConfigService);
  private readonly document = inject(DOCUMENT);
  private readonly logger = Logger.getInstance();

  constructor() {
    effect(() => {
      const settings: AppSettings = this.configService.settings();
      this.logger.debug(`Enabled ${settings.darkMode ? 'dark mode' : 'light mode'} via effect`);
      this.setDarkMode(settings.darkMode);
    });
  }

  /**
   * Handles the dark mode setting.
   * @param darkMode The value of the dark mode setting.
   * @private
   */
  private setDarkMode(darkMode: boolean): void {
    if (darkMode) {
      this.document.documentElement.classList.add('p-dark');
    } else {
      this.document.documentElement.classList.remove('p-dark');
    }

    this.logger.debug(`Dark mode ${darkMode ? 'enabled' : 'disabled'}`);
  }
}
