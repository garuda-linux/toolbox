import { inject, Injectable } from '@angular/core';
import { $dt } from '@primeuix/styled';
import { ConfigService } from '../config/config.service';
import { Logger } from '../../logging/logging';
import type { ITheme } from '@xterm/xterm';
import { DOCUMENT } from '@angular/common';
import { themes } from '../../theme';

@Injectable({ providedIn: 'root' })
export class TerminalThemeService {
  private readonly configService = inject(ConfigService);
  private readonly logger = Logger.getInstance();
  private readonly document = inject(DOCUMENT);

  /**
   * Resolve a design token to its actual value.
   * If the token is a CSS variable, it retrieves the value from the document's root element.
   * Otherwise, it returns the token as is.
   * @param token The design token to resolve.
   * @param darkMode Whether the app is currently in dark mode. Used to resolve light-dark() values.
   * @returns The resolved color value.
   */
  resolveColor(token: string, darkMode: boolean = this.configService.settings().darkMode): string {
    let value = token;
    if (token && token.startsWith('{') && token.endsWith('}')) {
      try {
        const dt = $dt(token);
        const cssVariable = dt?.variable.slice(4, -1) ?? '';
        value = cssVariable ? getComputedStyle(this.document.documentElement).getPropertyValue(cssVariable).trim() : '';
      } catch {
        value = '';
      }
    }
    return this.resolveColorScheme(value, darkMode);
  }

  /**
   * Get the xterm.js theme based on the active app theme.
   * @param darkMode Whether to use the dark mode theme.
   * @return The xterm.js theme object.
   */
  getXtermTheme(darkMode: boolean): ITheme {
    const theme = themes[this.configService.settings().activeTheme] || null;
    if (!theme) {
      this.logger.warn('Active theme not found, returning default xterm theme');
      return this.getDefaultTheme();
    }

    const defaults = this.getDefaultTheme();
    const tokens: Partial<Record<keyof ITheme, string>> = {
      foreground: '{text.color}',
      background: '{content.background}',
      cursor: '{primary.color}',
      cursorAccent: '{content.background}',
      black: '{surface.950}',
      red: '{formField.invalidBorderColor}',
      green: '{primary.color}',
      yellow: '{navigation.item.icon.color}',
      blue: '{highlight.background}',
      magenta: '{primary.hoverColor}',
      cyan: '{highlight.focusBackground}',
      white: '{formField.color}',
      brightBlack: '{surface.700}',
      brightRed: '{formField.invalidPlaceholderColor}',
      brightGreen: '{primary.hoverColor}',
      brightYellow: '{navigation.item.icon.focusColor}',
      brightBlue: '{primary.activeColor}',
      brightMagenta: '{highlight.focusColor}',
      brightCyan: '{formField.floatLabelFocusColor}',
      brightWhite: '{text.hoverColor}',
    };

    return Object.fromEntries(
      Object.entries(tokens).map(([key, token]) => [
        key,
        this.resolveColor(token, darkMode) || defaults[key as keyof ITheme] || '#ffffff',
      ]),
    ) as ITheme;
  }

  private getDefaultTheme(): ITheme {
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

  private resolveColorScheme(value: string, darkMode: boolean): string {
    const match = value.trim().match(/^light-dark\((.*)\)$/s);
    if (!match) {
      return value;
    }
    const branches = this.splitColorSchemeArgs(match[1]);
    const branch = branches[darkMode ? 1 : 0];
    return branch === undefined ? value : this.resolveColorScheme(branch, darkMode);
  }

  private splitColorSchemeArgs(inner: string): string[] {
    const branches: string[] = [];
    let depth = 0;
    let current = '';
    for (const char of inner) {
      if (char === '(') {
        depth++;
        current += char;
      } else if (char === ')') {
        depth--;
        current += char;
      } else if (char === ',' && depth === 0) {
        branches.push(current.trim());
        current = '';
      } else {
        current += char;
      }
    }
    if (current.trim()) {
      branches.push(current.trim());
    }
    return branches;
  }
}
