import { platform, arch, version, hostname, homedir, tmpdir } from 'node:os';
import { error } from './logging.js';
import { ipcRenderer } from 'electron';

export function getPlatform(): string {
  try {
    return platform();
  } catch (err) {
    error(`OS platform error: ${err instanceof Error ? err.message : String(err)}`);
    return 'unknown';
  }
}

export function getArch(): string {
  try {
    return arch();
  } catch (err) {
    error(`OS arch error: ${err instanceof Error ? err.message : String(err)}`);
    return 'unknown';
  }
}

export function getVersion(): string {
  try {
    return version();
  } catch (err) {
    error(`OS version error: ${err instanceof Error ? err.message : String(err)}`);
    return 'unknown';
  }
}

export function getLocale(): string {
  try {
    // Use Intl.DateTimeFormat to get the system locale
    return Intl.DateTimeFormat().resolvedOptions().locale || 'en-US';
  } catch (err) {
    error(`OS locale error: ${err instanceof Error ? err.message : String(err)}`);
    return 'en-US';
  }
}

export function getHostname(): string {
  try {
    return hostname();
  } catch (err) {
    error(`OS hostname error: ${err instanceof Error ? err.message : String(err)}`);
    return 'unknown';
  }
}

export function getHomedir(): string {
  try {
    return homedir();
  } catch (err) {
    error(`OS homedir error: ${err instanceof Error ? err.message : String(err)}`);
    return process.env.HOME || process.env.USERPROFILE || '';
  }
}

export function getTmpdir(): string {
  try {
    return tmpdir();
  } catch (err) {
    error(`OS tmpdir error: ${err instanceof Error ? err.message : String(err)}`);
    return process.env.TMPDIR || process.env.TEMP || '/tmp';
  }
}

export async function getArgv(): Promise<string[]> {
  try {
    return await ipcRenderer.invoke('os:argv');
  } catch (err) {
    error(`OS argv error: ${err instanceof Error ? err.message : String(err)}`);
    return [];
  }
}
