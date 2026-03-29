import { ipcRenderer } from 'electron';
import { error as logError } from './logging.js';

export async function open(options: Record<string, unknown>): Promise<string[]> {
  try {
    const result = await ipcRenderer.invoke('dialog:open', options);
    if (result.canceled) {
      return [];
    }
    return result.filePaths;
  } catch (err) {
    logError(`Dialog open error: ${err instanceof Error ? err.message : String(err)}`);
    return [];
  }
}

export async function save(options: Record<string, unknown>): Promise<string> {
  try {
    const result = await ipcRenderer.invoke('dialog:save', options);
    if (result.canceled) {
      return '';
    }
    return result.filePath;
  } catch (err) {
    logError(`Dialog save error: ${err instanceof Error ? err.message : String(err)}`);
    return '';
  }
}

export async function message(options: Record<string, unknown>): Promise<number> {
  try {
    const result = await ipcRenderer.invoke('dialog:message', options);
    return result.response;
  } catch (err) {
    logError(`Dialog message error: ${err instanceof Error ? err.message : String(err)}`);
    return -1;
  }
}

export async function error(title: string, content: string): Promise<boolean> {
  try {
    await ipcRenderer.invoke('dialog:error', title, content);
    return true;
  } catch (err) {
    logError(`Dialog error error: ${err instanceof Error ? err.message : String(err)}`);
    return false;
  }
}

export async function confirm(message: string, title?: string, detail?: string): Promise<boolean> {
  try {
    return await ipcRenderer.invoke('dialog:confirm', message, title, detail);
  } catch (err) {
    logError(`Dialog confirm error: ${err instanceof Error ? err.message : String(err)}`);
    return false;
  }
}

export async function warning(message: string, title?: string, detail?: string): Promise<boolean> {
  try {
    return await ipcRenderer.invoke('dialog:warning', message, title, detail);
  } catch (err) {
    logError(`Dialog warning error: ${err instanceof Error ? err.message : String(err)}`);
    return false;
  }
}

export async function info(message: string, title?: string, detail?: string): Promise<boolean> {
  try {
    return await ipcRenderer.invoke('dialog:info', message, title, detail);
  } catch (err) {
    logError(`Dialog info error: ${err instanceof Error ? err.message : String(err)}`);
    return false;
  }
}
