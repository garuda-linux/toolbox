import { join, resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { error } from './logging.js';

// Get common paths using environment variables and process properties
function getAppConfigDir(): string {
  // Use XDG_CONFIG_HOME or fallback to ~/.config for Linux
  if (process.platform === 'linux') {
    return process.env.XDG_CONFIG_HOME || join(process.env.HOME || '', '.config');
  }
  // For other platforms, use standard app data locations
  if (process.platform === 'win32') {
    return process.env.APPDATA || join(process.env.USERPROFILE || '', 'AppData', 'Roaming');
  }
  if (process.platform === 'darwin') {
    return join(process.env.HOME || '', 'Library', 'Application Support');
  }
  return process.env.HOME || '';
}

function getAppDataDir(): string {
  if (process.platform === 'linux') {
    return process.env.XDG_DATA_HOME || join(process.env.HOME || '', '.local', 'share');
  }
  if (process.platform === 'win32') {
    return process.env.APPDATA || join(process.env.USERPROFILE || '', 'AppData', 'Roaming');
  }
  if (process.platform === 'darwin') {
    return join(process.env.HOME || '', 'Library', 'Application Support');
  }
  return process.env.HOME || '';
}

function getAppCacheDir(): string {
  if (process.platform === 'linux') {
    return process.env.XDG_CACHE_HOME || join(process.env.HOME || '', '.cache');
  }
  if (process.platform === 'win32') {
    return process.env.TEMP || process.env.TMP || join(process.env.USERPROFILE || '', 'AppData', 'Local', 'Temp');
  }
  if (process.platform === 'darwin') {
    return join(process.env.HOME || '', 'Library', 'Caches');
  }
  return process.env.TMPDIR || '/tmp';
}

export async function appConfigDir(): Promise<string> {
  try {
    return getAppConfigDir();
  } catch (err: any) {
    error(`Path appConfigDir error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to get app config directory: ${err instanceof Error ? err.message : err}`, { cause: err });
  }
}

export async function appDataDir(): Promise<string> {
  try {
    return getAppDataDir();
  } catch (err: any) {
    error(`Path appDataDir error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to get app data directory: ${err instanceof Error ? err.message : err}`, { cause: err });
  }
}

export async function appLocalDataDir(): Promise<string> {
  try {
    return getAppDataDir();
  } catch (err: any) {
    error(`Path appLocalDataDir error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to get app local data directory: ${err instanceof Error ? err.message : err}`, {
      cause: err,
    });
  }
}

export async function appCacheDir(): Promise<string> {
  try {
    return getAppCacheDir();
  } catch (err: any) {
    error(`Path appCacheDir error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to get app cache directory: ${err instanceof Error ? err.message : err}`, { cause: err });
  }
}

export async function pathResolve(...paths: string[]): Promise<string> {
  try {
    return resolve(...paths);
  } catch (err: any) {
    error(`Path resolve error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to resolve path: ${err instanceof Error ? err.message : err}`, { cause: err });
  }
}

export async function pathJoin(...paths: string[]): Promise<string> {
  try {
    return join(...paths);
  } catch (err: any) {
    error(`Path JOIN error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to join paths: ${err instanceof Error ? err.message : err}`, { cause: err });
  }
}

export async function resolveResource(resourcePath: string): Promise<string> {
  try {
    const isDevelopment = process.env.NODE_ENV === 'development';

    if (isDevelopment) {
      return join(process.cwd(), 'src', 'assets', resourcePath);
    }

    const __filename = fileURLToPath(import.meta.url);
    const __dirname = dirname(__filename);
    return join(__dirname, '../dist/browser/assets', resourcePath);
  } catch (err: any) {
    error(`Path resolveResource error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to resolve resource path: ${err instanceof Error ? err.message : err}`, { cause: err });
  }
}
