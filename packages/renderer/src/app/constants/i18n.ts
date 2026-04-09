import { AvailableLangs, LangDefinition } from '@jsverse/transloco';

/**
 * Available languages for i18n translations.
 * This list should match the translation files in assets/i18n/
 */
export const AVAILABLE_LANGUAGES: AvailableLangs = [
  'am',
  'ar',
  'de',
  'en',
  'es',
  'eu',
  'fr',
  'gl',
  'hi',
  'hu',
  'id',
  'it',
  'ja',
  'ko',
  'pl',
  'pt',
  'ru',
  'sl',
  'sv',
  'sw',
  'tr',
  'ua',
  'uk',
  'uz',
  'zh-CN',
] as const;

export type AvailableLanguage = (typeof AVAILABLE_LANGUAGES)[number];
