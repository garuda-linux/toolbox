import { ipcRenderer } from 'electron';
import { error } from './logging.js';

export async function writeHomeConfig(
  relativePath: string,
  content: string,
  createDir = true,
): Promise<{ success: boolean; path?: string; error?: string }> {
  try {
    return await ipcRenderer.invoke('homeConfig:write', relativePath, content, createDir);
  } catch (err) {
    error(`Home config write error: ${err instanceof Error ? err.message : String(err)}`);
    return { success: false, error: err instanceof Error ? err.message : String(err) };
  }
}

export async function readHomeConfig(relativePath: string): Promise<{
  success: boolean;
  content?: string;
  path?: string;
  exists?: boolean;
  error?: string;
}> {
  try {
    return await ipcRenderer.invoke('homeConfig:read', relativePath);
  } catch (err) {
    error(`Home config read error: ${err instanceof Error ? err.message : String(err)}`);
    return { success: false, error: err instanceof Error ? err.message : String(err) };
  }
}

export async function homeConfigExists(
  relativePath: string,
): Promise<{ exists: boolean; path?: string; error?: string }> {
  try {
    return await ipcRenderer.invoke('homeConfig:exists', relativePath);
  } catch (err) {
    error(`Home config exists error: ${err instanceof Error ? err.message : String(err)}`);
    return { exists: false, error: err instanceof Error ? err.message : String(err) };
  }
}

export async function removeHomeConfig(relativePath: string): Promise<{ success: boolean; error?: string }> {
  try {
    return await ipcRenderer.invoke('homeConfig:remove', relativePath);
  } catch (err) {
    error(`Home config remove error: ${err instanceof Error ? err.message : String(err)}`);
    return { success: false, error: err instanceof Error ? err.message : String(err) };
  }
}
