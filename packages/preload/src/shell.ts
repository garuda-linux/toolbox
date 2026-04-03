import { ipcRenderer, shell } from 'electron';
import { error } from './logging.js';

export function shellSpawn(command: string, args: string[], options?: Record<string, unknown>) {
  return new Promise((resolve, reject) => {
    ipcRenderer.invoke('shell:execute', { command, args, options }).then(resolve).catch(reject);
  });
}

export function shellSpawnStreaming(command: string, args: string[] = [], options?: Record<string, unknown>) {
  const processId = crypto.randomUUID();

  ipcRenderer.invoke('shell:spawnStreaming', { processId, command, args, options }).catch((err) => {
    error(`Failed to spawn streaming shell: ${err}`);

    ipcRenderer.emit('events:emit', {}, 'shell:error', {
      processId,
      error: { name: 'SpawnError', message: String(err) },
    });
  });

  return {
    processId,
    pid: 0,
  };
}

export function shellWriteStdin(processId: string, data: string): boolean {
  ipcRenderer.invoke('shell:writeStdin', { processId, data }).catch((err) => {
    error(`Failed to write to stdin: ${err}`);
  });
  return true;
}

export function shellKillProcess(processId: string, signal = 'SIGTERM'): boolean {
  ipcRenderer.invoke('shell:killProcess', { processId, signal }).catch((err) => {
    error(`Failed to kill process: ${err}`);
  });
  return true;
}

export async function open(url: string): Promise<boolean> {
  try {
    const urlPattern = /^(https?:\/\/)|(file:\/\/)|(mailto:)|(tel:)/;
    if (!urlPattern.test(url)) {
      throw new Error('Invalid URL protocol');
    }
    await shell.openExternal(url);
    return true;
  } catch (err: any) {
    error(`Shell open error: ${err instanceof Error ? err.message : String(err)}`);
    throw new Error(`Failed to open URL: ${err instanceof Error ? err.message : err}`, {
      cause: err,
    });
  }
}

export async function execute(
  command: string,
  args: string[] = [],
  options: Record<string, unknown> = {},
): Promise<{
  code: number;
  stdout: string;
  stderr: string;
  signal: string;
}> {
  return await ipcRenderer.invoke('shell:execute', { command, args, options });
}
