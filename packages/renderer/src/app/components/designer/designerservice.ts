import { computed, effect, inject, Injectable, signal, untracked } from '@angular/core';
import { $dt, usePreset } from '@primeuix/styled';
import { MessageService } from 'primeng/api';
import { ConfigService } from '../config/config.service';
import { Logger } from '../../logging/logging';
import type { ITheme } from '@xterm/xterm';
import type { Preset } from '@primeuix/themes/types';
import { DOCUMENT } from '@angular/common';
import { themes } from '../../theme';

export interface Theme {
  key: string | null;
  name: string | null;
  preset: Preset | null;
  config: {
    font_size: string;
    font_family: string;
  } | null;
}

export interface Designer {
  active: boolean;
  activeView: string;
  activeTab: number;
  acTokens: any[];
  theme: Theme;
}

@Injectable({ providedIn: 'root' })
export class DesignerService {
  private readonly configService = inject(ConfigService);
  private readonly document = inject(DOCUMENT);
  private readonly logger = Logger.getInstance();
  private readonly messageService: MessageService = inject(MessageService);

  designer = signal<Designer>({
    active: false,
    activeView: 'create_theme',
    activeTab: 0,
    acTokens: [],
    theme: {
      key: null,
      name: null,
      preset: null,
      config: null,
    },
  });
  preset = signal<{ primitive: unknown; semantic: unknown }>({ primitive: null, semantic: null });
  acTokens = computed(() => this.designer().acTokens);
  status = signal<'preview' | 'updated' | null>(null);
  loading = signal<boolean>(false);
  themeName = signal<string | undefined>(undefined);
  basePreset = signal<string | null>(null);
  newPreset = signal<Record<string, unknown> | null>(null);

  private mustNotTriggerEffect = false;

  constructor() {
    effect(() => {
      if (this.configService.settings().activeTheme === 'Custom Themedesigner' && !this.mustNotTriggerEffect) {
        this.logger.debug('Custom theme was triggered, activating designer');
        this.mustNotTriggerEffect = true;

        if (untracked(this.configService.settings).customDesign !== null) {
          this.logger.debug('Custom design found, activating theme');
          void this.activateTheme(JSON.parse(this.configService.settings().customDesign as string) as Theme);
        } else {
          this.logger.debug('No custom design found, creating new theme from preset');
          this.configService.updateState('designerActive', true);
          void this.createThemeFromPreset();
        }
      } else if (this.configService.settings().activeTheme !== 'Custom Themedesigner' && this.mustNotTriggerEffect) {
        this.logger.debug('Custom theme was deactivated, resetting designer state');
        this.mustNotTriggerEffect = false;
      }
    });
  }

  /**
   * Resolve a design token to its actual value.
   * If the token is a CSS variable, it retrieves the value from the document's root element.
   * Otherwise, it returns the token as is.
   * @param token The design token to resolve.
   * @returns The resolved color value.
   */
  resolveColor(token: string): string {
    if (token && token.startsWith('{') && token.endsWith('}')) {
      const cssVariable = $dt(token).variable.slice(4, -1);
      return getComputedStyle(document.documentElement).getPropertyValue(cssVariable);
    } else {
      return token;
    }
  }

  /**
   * Refresh the design tokens for the AC system.
   * This function clears the existing tokens and generates new ones based on the current theme preset.
   */
  refreshACTokens() {
    this.designer.update((prev) => ({ ...prev, acTokens: [] }));
    if (this.designer().theme.preset) {
      // @ts-expect-error whatever .......
      this.generateACTokens(null, this.designer().theme.preset);
    }
  }

  /**
   * Create a new theme from an already defined preset.
   */
  async createThemeFromPreset() {
    if (this.designer().activeView === 'editor') {
      this.designer.update((prev) => ({ ...prev, activeView: 'create_theme' }));
    }
    if (this.newPreset() !== null) {
      await this.loadThemeEditor({
        preset: this.newPreset() as Preset,
        key: 'customTheme',
        name: 'Toolbox theme',
        config: {
          font_size: '14px',
          font_family: 'Inter var',
        },
      });
    }
  }

  /**
   * Generate design tokens for the AC system.
   * This function recursively traverses the provided object and generates tokens
   * for each property, converting camelCase to dot.case format.
   * @param parentPath The parent path for the current object.
   * @param obj The object containing design tokens.
   */
  generateACTokens(parentPath: string | null, obj: Record<string, unknown>) {
    for (const key in obj) {
      if (key === 'dark' || key === 'components' || key === 'directives') {
        continue;
      }
      if (key === 'primitive' || key === 'semantic' || key === 'colorScheme' || key === 'light' || key === 'extend') {
        this.generateACTokens(null, obj[key] as Record<string, unknown>);
      } else {
        if (typeof obj[key] === 'object' && obj[key] !== null) {
          this.generateACTokens(parentPath ? parentPath + '.' + key : key, obj[key] as Record<string, unknown>);
        } else {
          const regex = /\.\d+$/;
          const tokenName = this.camelCaseToDotCase(parentPath ? parentPath + '.' + key : key);
          const tokenValue = obj[key] as string;
          const isColor =
            tokenName.includes('color') ||
            tokenName.includes('background') ||
            regex.test(tokenName) ||
            (typeof tokenValue === 'string' &&
              (tokenValue.startsWith('#') ||
                tokenValue.startsWith('rgb') ||
                tokenValue.startsWith('hsl') ||
                tokenValue.startsWith('oklch')));
          this.designer.update((prev) => ({
            ...prev,
            acTokens: [
              ...prev.acTokens,
              {
                name: tokenName,
                label: '{' + tokenName + '}',
                variable: $dt(tokenName).variable,
                value: tokenValue,
                isColor: isColor,
              },
            ],
          }));
        }
      }
    }
  }

  camelCaseToDotCase(name: string): string {
    return name.replace(/([a-z])([A-Z])/g, '$1.$2').toLowerCase();
  }

  /**
   * Apply the selected font family to the document.
   * @param fontFamily The font family to apply.
   */
  async applyFont(fontFamily: string) {
    if (fontFamily !== 'Inter var') {
      await this.loadFont(fontFamily, 400);
      await this.loadFont(fontFamily, 500);
      await this.loadFont(fontFamily, 600);
      await this.loadFont(fontFamily, 700);
    } else {
      document.body.style.fontFamily = `"Inter var", sans-serif`;
    }
  }

  /**
   * Load a font from the Bunny Fonts CDN.
   * @param fontFamily The name of the font family to load.
   * @param weight The font weight to load.
   * @returns A promise that resolves to the loaded FontFace or undefined if loading fails.
   */
  async loadFont(fontFamily: string, weight: number): Promise<FontFace | undefined> {
    try {
      const fontFamilyPath: string = fontFamily.toLowerCase().replace(/\s+/g, '-');
      const fontUrl = `https://fonts.bunny.net/${fontFamilyPath}/files/${fontFamilyPath}-latin-${weight}-normal.woff2`;
      const font = new FontFace(fontFamily, `url(${fontUrl})`, {
        weight: weight.toString(),
        style: 'normal',
      });
      const loadedFont: FontFace = await font.load();

      this.document.fonts.add(loadedFont);
      this.document.body.style.fontFamily = `"${fontFamily}", sans-serif`;
      return loadedFont;
    } catch {
      // silent fail as some fonts may have not all the font weights
      return undefined;
    }
  }

  /**
   * Apply the selected theme preset to the design system.
   * @param theme The theme object containing the preset to apply.
   */
  async applyTheme(theme: { preset: Preset }) {
    usePreset(theme.preset);
    this.messageService.add({
      key: 'designer',
      severity: 'success',
      summary: 'Success',
      detail: 'Theme applied.',
      life: 3000,
    });
  }

  /**
   * Load the theme editor with the provided theme.
   * This function sets the designer state to the provided theme and applies the font and preset.
   * @param theme The theme object to load into the editor.
   */
  async loadThemeEditor(theme: Theme) {
    this.designer.set({
      ...this.designer(),
      theme: {
        name: this.themeName() ?? null,
        key: theme.key,
        preset: theme.preset,
        config: {
          font_size: '14px',
          font_family: 'Inter var',
        },
      },
    });

    await this.applyFont('Inter var');
    document.documentElement.style.fontSize = '14px';

    // @ts-expect-error - usePreset function may not have complete type definitions
    usePreset(theme.preset);

    this.designer.update((prev) => ({ ...prev, activeTab: 0, activeView: 'editor' }));
    this.themeName.set(undefined);
    this.basePreset.set(null);
    this.newPreset.set(null);
  }

  /**
   * Activate the theme with the provided data.
   * This function updates the designer state with the new theme and applies the font and preset.
   * @param data The theme data to activate.
   */
  async activateTheme(data: Theme) {
    this.designer.update((prev) => ({
      ...prev,
      active: true,
      theme: {
        key: data.key,
        name: data.name,
        preset: data.preset,
        config: data.config,
      },
    }));

    usePreset(this.designer().theme.preset as Record<string, unknown>);

    await this.applyFont((this.designer().theme.config as Record<string, string>)['fontFamily']);
    document.documentElement.style.setProperty(
      'font-size',
      (this.designer().theme.config as Record<string, string>)['font_size'],
    );

    this.designer.update((prev) => ({ ...prev, activeTab: 0, activeView: 'editor' }));
    this.status.set(null);
  }

  /**
   * Save the current theme to the configuration, as a JSON string.
   * @param theme The theme object to save.
   */
  async saveTheme(theme: Theme) {
    await this.configService.updateConfig('customDesign', JSON.stringify(theme));
  }

  /**
   * Download the theme as a JSON file.
   * @param theme The theme object to download.
   */
  async downloadTheme(theme: Theme) {
    const file = new Blob([JSON.stringify(theme, null, 2)], { type: 'application/json' });
    const blobUrl: string = window.URL.createObjectURL(file);
    const link: HTMLAnchorElement = document.createElement('a');

    link.setAttribute('href', blobUrl);
    link.setAttribute('download', `${theme.name}.json`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.URL.revokeObjectURL(blobUrl);
  }

  /**
   * Get the xterm.js theme based on the current design settings.
   * @param darkMode Whether to use the dark mode theme.
   * @return The xterm.js theme object.
   */
  getXtermTheme(darkMode: boolean): ITheme {
    let theme: Preset;
    if (
      this.configService.settings().activeTheme === 'Custom Themedesigner' &&
      this.configService.settings().customDesign !== null
    ) {
      const designTheme = JSON.parse(this.configService.settings().customDesign as string) as Theme;
      theme = designTheme.preset as Preset;
    } else {
      theme = themes[this.configService.settings().activeTheme] || null;
    }

    if (!theme) {
      this.logger.warn('Custom design is not set, returning default xterm theme');
      return {
        foreground: '#ffffff',
        background: '#000000',
        cursor: '#ffffff',
        cursorAccent: '#000000',
        black: '#000000',
        red: '#ff0000',
        green: '#00ff00',
        yellow: '#ffff00',
        blue: '#0000ff',
        magenta: '#ff00ff',
        cyan: '#00ffff',
        white: '#ffffff',
        brightBlack: '#808080',
        brightRed: '#ff8080',
        brightGreen: '#80ff80',
        brightYellow: '#ffff80',
        brightBlue: '#8080ff',
        brightMagenta: '#ff80ff',
        brightCyan: '#80ffff',
        brightWhite: '#ffffff',
      };
    }

    // @ts-expect-error Preset type do not have complete type definitions
    const colorScheme = darkMode ? theme.semantic.colorScheme.dark : theme.semantic.colorScheme.light;

    return {
      // Basic terminal settings
      foreground: this.resolveColor(colorScheme.text.color),
      background: this.resolveColor(colorScheme.content.background),
      cursor: this.resolveColor(colorScheme.primary.color),
      cursorAccent: this.resolveColor(colorScheme.content.background),

      // Standard ANSI colors derived from colorScheme
      black: this.resolveColor(colorScheme.surface[950]),
      red: this.resolveColor(colorScheme.formField.invalidBorderColor),
      green: this.resolveColor(colorScheme.primary.color),
      yellow: this.resolveColor(colorScheme.navigation.item.icon.color),
      blue: this.resolveColor(colorScheme.highlight.background),
      magenta: this.resolveColor(colorScheme.primary.hoverColor),
      cyan: this.resolveColor(colorScheme.highlight.focusBackground),
      white: this.resolveColor(colorScheme.formField.color),

      // Bright variants
      brightBlack: this.resolveColor(colorScheme.surface[700]),
      brightRed: this.resolveColor(colorScheme.formField.invalidPlaceholderColor),
      brightGreen: this.resolveColor(colorScheme.primary.hoverColor),
      brightYellow: this.resolveColor(colorScheme.navigation.item.icon.focusColor),
      brightBlue: this.resolveColor(colorScheme.primary.activeColor),
      brightMagenta: this.resolveColor(colorScheme.highlight.focusColor),
      brightCyan: this.resolveColor(colorScheme.formField.floatLabelFocusColor),
      brightWhite: this.resolveColor(colorScheme.text.hoverColor),
    };
  }
}
