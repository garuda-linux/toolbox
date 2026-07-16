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
  private readonly logger = Logger.getInstance();

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
   * Get the xterm.js theme based on the current design settings.
   * @param darkMode Whether to use the dark mode theme.
   * @return The xterm.js theme object.
   */
  getXtermTheme(darkMode: boolean): ITheme {
    const theme = themes[this.configService.settings().activeTheme] || null;
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
    const colorScheme = darkMode ? theme.semantic?.colorScheme?.dark : theme.semantic?.colorScheme?.light;

    if (!colorScheme) {
      this.logger.warn('Theme structure is incomplete, returning default xterm theme');
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
