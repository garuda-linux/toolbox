import { Service } from '@angular/core';
import type { Translation, TranslocoLoader } from '@jsverse/transloco';
import { Logger } from './logging/logging';
import { ElectronFsService, resolveResource } from './electron-services';

@Service()
export class TranslocoHttpLoader implements TranslocoLoader {
  private readonly logger = Logger.getInstance();
  private readonly fsService = new ElectronFsService();

  async getTranslation(lang: string): Promise<Translation> {
    try {
      // Validate language parameter
      if (!lang || lang.trim().length === 0) {
        this.logger.warn(`Invalid language parameter: ${lang}`);
        return {};
      }

      // Sanitize language code to prevent path traversal
      const sanitizedLang = lang.replace(/[^a-zA-Z0-9-_]/g, '');
      if (sanitizedLang !== lang) {
        this.logger.warn(`Language code sanitized from '${lang}' to '${sanitizedLang}'`);
      }

      const resourcePath: string = await resolveResource(`../../assets/i18n/${sanitizedLang}.json`);
      // Use the safe JSON reading method
      const translation = await this.fsService.safeReadJsonFile<Translation>(resourcePath, {} as Translation);

      this.logger.trace(`Loaded translation for ${sanitizedLang}, ${Object.keys(translation).length} keys`);
      return translation;
    } catch (err: unknown) {
      this.logger.error(`Failed to load translation for '${lang}': ${err}`);

      // Provide more specific error logging
      if (err instanceof Error) {
        if (err.message.includes('not found')) {
          this.logger.warn(`Translation file not found for language '${lang}', using empty translation`);
        } else if (err.message.includes('Invalid JSON')) {
          this.logger.error(`Invalid JSON in translation file for language '${lang}': ${err.message}`);
        }
      }

      return {};
    }
  }
}
