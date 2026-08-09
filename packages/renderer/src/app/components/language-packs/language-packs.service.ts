import { inject, Service, signal } from '@angular/core';
import { LanguagePack, LanguagePacks } from './types';
import { ChildProcess } from '../../types/shell';
import { Logger } from '../../logging/logging';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { OsInteractService } from '../task-manager/os-interact.service';
import { LoadingService } from '../loading-indicator/loading-indicator.service';

@Service()
export class LanguagePacksService {
  languagePacks = signal<LanguagePacks>([]);
  loading = signal<boolean>(true);

  private readonly loadingService = inject(LoadingService);
  private readonly logger = Logger.getInstance();
  private readonly osInteractService = inject(OsInteractService);
  private readonly taskManagerService = inject(TaskManagerService);

  constructor() {
    void this.init();
  }

  async init(): Promise<void> {
    this.loadingService.loadingOn();
    await this.getAvailableLangPacks();
    this.loadingService.loadingOff();
    this.loading.set(false);
  }

  /**
   * Get the available language packs from the system and process them.
   */
  async getAvailableLangPacks(): Promise<void> {
    const locales: string[] = await this.getLocales();
    const promises: Promise<{
      packages: string[];
      locales: LanguagePackMeta;
    }>[] = [];

    for (const locale of locales) {
      this.logger.trace(`Checking locale: ${locale}`);
      const [language, territory] = locale.split('_');
      if (!territory) continue;

      const regex = `^.*-${language}(_${territory}|-${territory})?$`;
      promises.push(this.prepareLangPackExecution(regex, { language, territory, locale }));
    }

    const promiseResults: { packages: string[]; locales: LanguagePackMeta }[] = await Promise.all(promises);
    const totalPacks: LanguagePacks = this.processPackages(promiseResults);

    this.logger.info(`Found ${totalPacks.length} language packs for ${locales.length} locales`);
    this.languagePacks.set(totalPacks);
  }

  /**
   * Get the available language packs from the system.
   */
  async getLocales(): Promise<string[]> {
    const cmd = 'locale -a';
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);

    if (result.code !== 0) {
      this.logger.error(`Failed to get available language packs: ${result.stderr}`);
    }

    const locales = result.stdout
      .trim()
      .split('\n')
      .filter((locale: string) => locale.includes('utf8'))
      .map((locale: string) => locale.split('.')[0]);
    this.logger.debug(`Available locales: ${locales.length}`);
    return locales;
  }

  /**
   * Prepare the execution of the language pack command.
   * @param regex The regex to match the language pack packages.
   * @param locales The locales to filter the packages.
   * @returns An object containing the packages and locales.
   */
  async prepareLangPackExecution(
    regex: string,
    locales: LanguagePackMeta,
  ): Promise<{ packages: string[]; locales: LanguagePackMeta }> {
    const cmd = `pacman -Ssq "${regex}"`;
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);

    if (result.code !== 0) {
      this.logger.error(`Failed to get available language packs: ${result.stderr}`);
      return { packages: [], locales };
    }

    const packages: string[] = result.stdout.trim().split('\n');
    return { packages, locales };
  }

  /**
   * Process the packages and locales to create a list of language packs.
   * @param preparedData The prepared data containing packages and locales.
   * @returns An array of language packs.
   */
  processPackages(
    preparedData: {
      packages: string[];
      locales: LanguagePackMeta;
    }[],
  ): LanguagePacks {
    const totalPacks: LanguagePack[] = [];

    for (const { packages, locales } of preparedData) {
      this.logger.trace(`Found ${packages.length} packages for locale: ${locales.locale}`);
      const { language, territory, locale } = locales;

      for (const pkgname of packages) {
        let langString: string;
        const regexLangOnly = `^.*-${language}$`;
        switch (true) {
          case pkgname.match(new RegExp(regexLangOnly)) !== null:
            langString = `${language}`;
            break;
          case pkgname.includes(`${language}-${territory}`):
            langString = `${language}-${territory}`;
            break;
          case pkgname.includes(`${language}_${territory}`):
            langString = `${language}_${territory}`;
            break;
          case pkgname.includes(`${language}-${territory}`.toLowerCase()):
            langString = `${language}-${territory}`.toLowerCase();
            break;
          case pkgname.includes(`${language}_${territory}`.toLowerCase()):
            langString = `${language}_${territory}`.toLowerCase();
            break;
          default:
            continue;
        }

        const base: string = pkgname.replace(`-${langString}`, '');
        if (!this.osInteractService.packages().get(base)) {
          continue;
        }

        const languagePack: LanguagePack = {
          pkgname: [pkgname],
          locale,
          selected: this.osInteractService.packages().get(pkgname) === true,
          base,
        };
        totalPacks.push(languagePack);
      }
    }

    return totalPacks;
  }
}

interface LanguagePackMeta {
  language: string;
  territory: string;
  locale: string;
}
