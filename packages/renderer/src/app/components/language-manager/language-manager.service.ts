import { effect, inject, Service } from '@angular/core';
import { ConfigService } from '../config/config.service';
import { TranslocoService } from '@jsverse/transloco';
import { locale } from '../../electron-services';
import { Logger } from '../../logging/logging';
import { firstValueFrom, lastValueFrom } from 'rxjs';

@Service()
export class LanguageManagerService {
  private readonly configService = inject(ConfigService);
  private readonly logger = Logger.getInstance();
  private readonly translocoService = inject(TranslocoService);

  constructor() {
    effect(async () => {
      const savedLang: string = this.configService.settings().language;
      void this.updateLanguage(savedLang);
    });
  }

  /**
   * Wait for language to load by doing a dummy translation. Don't updateLanguage here
   * as the effect will already run once on startup.
   */
  async init() {
    await firstValueFrom(this.translocoService.selectTranslate('menu.welcome'));
  }

  /**
   * Update the active language.
   * @param lang The language to set.
   */
  async updateLanguage(lang: string) {
    const sysLang: string | null = await locale();
    let activeLang: string = lang ?? sysLang;
    if (activeLang.match(/en-/)) {
      activeLang = 'en';
    }
    this.logger.trace(`Active language: ${activeLang}`);

    if (
      activeLang &&
      activeLang !== this.translocoService.getActiveLang() &&
      (this.translocoService.getAvailableLangs() as string[]).includes(activeLang)
    ) {
      await lastValueFrom(this.translocoService.load(activeLang));
      this.translocoService.setActiveLang(activeLang);
    }
  }
}
