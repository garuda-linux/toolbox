import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { App, ipcMain } from 'electron';
import { spawn } from 'node:child_process';
import { Logger } from '../logging/logging.js';

class ShellModule implements AppModule {
  private readonly logger = Logger.getInstance();
  private readonly activeProcesses = new Map<string, any>();

  enable({ app }: ModuleContext): void {
    this.setupShellHandlers(app);
  }

  private setupShellHandlers(app: App): void {
    ipcMain.handle('shell:spawnStreaming', (event, { processId, command, args, options }) => {
      this.logger.trace(`Spawning shell process: ${command} ${JSON.stringify(args)}`);

      try {
        const mergedOptions = { ...options };
        if (options?.env) {
          mergedOptions.env = { ...process.env, ...options.env };
        }

        const handle = spawn(command, args, {
          ...mergedOptions,
          stdio: ['pipe', 'pipe', 'pipe'],
        });

        this.activeProcesses.set(processId, handle);

        handle.stdout?.on('data', (data: Buffer) => {
          event.sender.send('events:emit', 'shell:stdout', {
            processId,
            data: data.toString(),
          });
        });

        handle.stderr?.on('data', (data: Buffer) => {
          event.sender.send('events:emit', 'shell:stderr', {
            processId,
            data: data.toString(),
          });
        });

        handle.on('close', (code: number | null, signal: string | null) => {
          event.sender.send('events:emit', 'shell:close', {
            processId,
            code,
            signal,
          });
          this.activeProcesses.delete(processId);
        });

        handle.on('error', (error: Error) => {
          event.sender.send('events:emit', 'shell:error', {
            processId,
            error: {
              name: error.name,
              message: error.message,
              stack: error.stack,
            },
          });
          this.activeProcesses.delete(processId);
        });

        return {
          processId,
          pid: handle.pid || 0,
        };
      } catch (error: any) {
        this.logger.error(`Error spawning shell process: ${error.message}`);
        throw error;
      }
    });

    ipcMain.handle('shell:writeStdin', (event, { processId, data }) => {
      const handle = this.activeProcesses.get(processId);
      if (handle?.stdin && !handle.stdin.destroyed) {
        handle.stdin.write(data);
        return true;
      }
      return false;
    });

    ipcMain.handle('shell:killProcess', (event, { processId, signal }) => {
      const handle = this.activeProcesses.get(processId);
      if (handle && !handle.killed) {
        handle.kill(signal);
        this.activeProcesses.delete(processId);
        return true;
      }
      return false;
    });

    ipcMain.handle('shell:execute', (event, { command, args, options }) => {
      const timeout = (options.timeout as number) || 0;

      return new Promise((resolve, reject) => {
        const child = spawn(command, args, {
          stdio: ['pipe', 'pipe', 'pipe'],
          timeout,
          ...options,
        });

        let stdout = '';
        let stderr = '';
        let timeoutId: NodeJS.Timeout;

        if (timeout > 0) {
          timeoutId = setTimeout(() => {
            child.kill('SIGTERM');
            reject(new Error('Command execution timed out'));
          }, timeout);
        }

        const cleanup = () => {
          clearTimeout(timeoutId);
        };

        child.stdout?.on('data', (data) => {
          stdout += data.toString();
        });

        child.stderr?.on('data', (data) => {
          stderr += data.toString();
        });

        child.on('close', (code, signal) => {
          cleanup();
          resolve({
            code: code === null ? -1 : code,
            stdout: stdout.substring(0, 1024 * 1024),
            stderr: stderr.substring(0, 1024 * 1024),
            signal: signal || '',
          });
        });

        child.on('error', (err) => {
          cleanup();
          this.logger.error(`Command execution error: ${err.message}`);
          reject(new Error(`Command execution failed: ${err.message}`));
        });
      });
    });
  }
}

export function createShellModule() {
  return new ShellModule();
}
