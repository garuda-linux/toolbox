import { contextBridge } from 'electron';
import ElectronStore from 'electron-store';
import { readFileSync } from 'fs';

interface SplashVars {
  bg: string;
  fg: string;
  gradient: string;
}

const splashMap: Record<string, SplashVars> = {
  mokka: { bg: '#1e1e2e', fg: '#cdd6f4', gradient: 'linear-gradient(90deg, #89b4fa, #cba6fa, #f9e2af)' },
  dr460nized: { bg: '#090a0f', fg: '#ffffff', gradient: 'linear-gradient(90deg, #d03cdb, #e697ec, #f1c5f4)' },
};

const editionThemeMap: Record<string, string> = {
  mokka: 'mokka',
  dr460nized: 'dr460nized',
};

function resolveSplash(): SplashVars {
  const fallback = splashMap.mokka;
  try {
    const store = new ElectronStore({ encryptionKey: 'non-security-by-obscurity' }) as unknown as {
      has: (k: string) => boolean;
      get: (k: string) => unknown;
    };
    if (store.has('activeTheme')) {
      const theme = String(store.get('activeTheme') ?? '');
      if (theme.includes('Dr460nized')) return splashMap.dr460nized;
      if (theme.includes('Catppuccin')) return splashMap.mokka;
    }
  } catch {
    // ignore store errors, fall through to edition
  }

  try {
    const raw = readFileSync('/usr/lib/garuda/garuda-release', 'utf8');
    const line = raw.split('\n').find((l) => l.trim().startsWith('EDITION='));
    const edition =
      line
        ?.split('=')[1]
        ?.trim()
        .replace(/^["']|["']$/g, '')
        .toLowerCase() ?? '';
    const key = editionThemeMap[edition];
    if (key && splashMap[key]) return splashMap[key];
  } catch {
    // no garuda-release
  }
  return fallback;
}

try {
  const vars = resolveSplash();
  contextBridge.exposeInMainWorld('__garudaSplash', vars);
} catch {
  // contextBridge may be unavailable in tests
}
