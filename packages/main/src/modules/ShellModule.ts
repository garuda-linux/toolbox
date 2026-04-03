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

  private setupShellHandlers(_app: App): void {
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

        let stdoutBuffer = '';
        let stderrBuffer = '';
        let flushTimeout: NodeJS.Timeout | null = null;

        const flushBuffers = () => {
          if (stdoutBuffer) {
            event.sender.send('events:emit', 'shell:stdout', {
              processId,
              data: stdoutBuffer,
            });
            stdoutBuffer = '';
          }
          if (stderrBuffer) {
            event.sender.send('events:emit', 'shell:stderr', {
              processId,
              data: stderrBuffer,
            });
            stderrBuffer = '';
          }
          flushTimeout = null;
        };

        const queueFlush = () => {
          if (!flushTimeout) {
            flushTimeout = setTimeout(flushBuffers, 50);
          }
        };

        handle.stdout?.on('data', (data: Buffer) => {
          stdoutBuffer += data.toString();
          if (stdoutBuffer.length > 16384) {
            flushBuffers();
            if (flushTimeout) clearTimeout(flushTimeout);
          } else {
            queueFlush();
          }
        });

        handle.stderr?.on('data', (data: Buffer) => {
          stderrBuffer += data.toString();
          if (stderrBuffer.length > 16384) {
            flushBuffers();
            if (flushTimeout) clearTimeout(flushTimeout);
          } else {
            queueFlush();
          }
        });

        handle.on('close', (code: number | null, signal: string | null) => {
          if (flushTimeout) {
            clearTimeout(flushTimeout);
            flushBuffers();
          }
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
