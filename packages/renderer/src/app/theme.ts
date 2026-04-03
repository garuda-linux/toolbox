import { flavors } from '@catppuccin/palette';
import type { Preset } from '@primeuix/themes/types';
import { Dr460nizedAura, Dr460nizedLara, Dr460nizedMaterial, Dr460nizedNora } from '@garudalinux/themes/dr460nized';
import {
  CatppuccinAura,
  CatppuccinAuraAlt,
  CatppuccinLara,
  CatppuccinLaraAlt,
  CatppuccinMaterial,
  CatppuccinMaterialAlt,
  CatppuccinNora,
  CatppuccinNoraAlt,
} from '@garudalinux/themes/catppuccin';
import { Vo1dedAura, Vo1dedLara, Vo1dedMaterial, Vo1dedNora } from '@garudalinux/themes/vo1ded';

const { latte, mocha } = flavors;

export const themes: AppThemes = {
  'Catppuccin Mocha/Latte Aura': CatppuccinAura,
  'Catppuccin Mocha/Latte Nora': CatppuccinNora,
  'Catppuccin Mocha/Latte Material': CatppuccinMaterial,
  'Catppuccin Mocha/Latte Lara': CatppuccinLara,
  'Catppuccin Macchiato/Frappe Aura': CatppuccinAuraAlt,
  'Catppuccin Macchiato/Frappe Nora': CatppuccinNoraAlt,
  'Catppuccin Macchiato/Frappe Material': CatppuccinMaterialAlt,
  'Catppuccin Macchiato/Frappe Lara': CatppuccinLaraAlt,
  'Dr460nized Aura': Dr460nizedAura,
  'Dr460nized Nora': Dr460nizedNora,
  'Dr460nized Material': Dr460nizedMaterial,
  'Dr460nized Lara': Dr460nizedLara,
  'Vo1ded Aura': Vo1dedAura,
  'Vo1ded Nora': Vo1dedNora,
  'Vo1ded Material': Vo1dedMaterial,
  'Vo1ded Lara': Vo1dedLara,
  'Custom Themedesigner': {},
};

export type AppTheme =
  | 'Catppuccin Mocha/Latte Aura'
  | 'Catppuccin Mocha/Latte Nora'
  | 'Catppuccin Mocha/Latte Material'
  | 'Catppuccin Mocha/Latte Lara'
  | 'Catppuccin Macchiato/Frappe Aura'
  | 'Catppuccin Macchiato/Frappe Nora'
  | 'Catppuccin Macchiato/Frappe Material'
  | 'Catppuccin Macchiato/Frappe Lara'
  | 'Dr460nized Aura'
  | 'Dr460nized Nora'
  | 'Dr460nized Material'
  | 'Dr460nized Lara'
  | 'Vo1ded Aura'
  | 'Vo1ded Nora'
  | 'Vo1ded Material'
  | 'Vo1ded Lara'
  | 'Custom Themedesigner';

export type AppThemes = Record<string, Preset>;

export const CatppuccinXtermJs = {
  light: {
    background: latte.colors.base.hex,
    black: latte.colors.surface1.hex,
    blue: latte.colors.blue.hex,
    brightBlack: latte.colors.surface1.hex,
    brightBlue: latte.colors.blue.hex,
    brightCyan: latte.colors.flamingo.hex,
    brightGreen: latte.colors.green.hex,
    brightMagenta: latte.colors.maroon.hex,
    brightRed: latte.colors.red.hex,
    brightWhite: latte.colors.text.hex,
    brightYellow: latte.colors.yellow.hex,
    cursor: latte.colors.text.hex,
    cursorAccent: latte.colors.text.hex,
    cyan: latte.colors.flamingo.hex,
    foreground: latte.colors.text.hex,
    green: latte.colors.green.hex,
    magenta: latte.colors.maroon.hex,
    red: latte.colors.red.hex,
    white: latte.colors.text.hex,
    yellow: latte.colors.yellow.hex,
  },
  dark: {
    background: mocha.colors.base.hex,
    black: mocha.colors.surface1.hex,
    blue: mocha.colors.blue.hex,
    brightBlack: mocha.colors.surface1.hex,
    brightBlue: mocha.colors.blue.hex,
    brightCyan: mocha.colors.flamingo.hex,
    brightGreen: mocha.colors.green.hex,
    brightMagenta: mocha.colors.maroon.hex,
    brightRed: mocha.colors.red.hex,
    brightWhite: mocha.colors.text.hex,
    brightYellow: mocha.colors.yellow.hex,
    cursor: mocha.colors.text.hex,
    cursorAccent: mocha.colors.text.hex,
    cyan: mocha.colors.flamingo.hex,
    foreground: mocha.colors.text.hex,
    green: mocha.colors.green.hex,
    magenta: mocha.colors.maroon.hex,
    red: mocha.colors.red.hex,
    white: mocha.colors.text.hex,
    yellow: mocha.colors.yellow.hex,
  },
};
