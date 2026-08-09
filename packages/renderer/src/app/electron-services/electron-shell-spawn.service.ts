import { Service } from '@angular/core';
import { Logger } from '../logging/logging';
import type { ShellEvent, ShellStreamingResult } from './electron-types';
import {
  eventsOff,
  eventsOn,
  execute,
  shellKillProcess,
  shellSpawnStreaming,
  shellWriteStdin,
} from './electron-api-utils.js';
import type { ChildProcess } from '../types/shell';

// Re-export ShellStreamingResult for backward compatibility
export type { ShellStreamingResult };

// Options for the streaming spawn method
export interface ShellStreamingOptions {
  onStdout?: (data: string) => void;
  onStderr?: (data: string) => void;
  onClose?: (code: number | null, signal: string | null) => void;
  onError?: (error: Error) => void;
  cwd?: string;
  env?: Record<string, string>;
}

@Service()
export class ElectronShellSpawnService {
  private readonly logger = Logger.getInstance();

  // Map to store cleanup functions for each streamed process
  private cleanupFunctions = new Map<string, () => void>();

  /**
   * Spawns a shell command with streaming stdout/stderr to callbacks.
   * This is designed for long-lived processes or processes where you need real-time output.
   *
   * @param command The command to execute (e.g., 'bash', 'pkexec').
   * @param args Arguments for the command.
   * @param options Callbacks for stdout, stderr, close, and error events.
   * @returns A Promise resolving to ShellStreamingResult containing processId and pid.
   */
  async spawnStreaming(
    command: string,
    args?: string[],
    options?: ShellStreamingOptions,
  ): Promise<ShellStreamingResult> {
    this.logger.debug(`Invoking spawnStreaming with command: ${command}, args: ${JSON.stringify(args)}`);
    const { processId, pid } = shellSpawnStreaming(command, args, {
      cwd: options?.cwd,
      env: options?.env,
    });

    if (!processId) {
      const errorMessage = `Failed to spawn process for command: ${command}. No processId returned.`;
      this.logger.error(errorMessage);
      throw new Error(errorMessage);
    }
    this.logger.debug(`Spawned process with ID: ${processId}, PID: ${pid}`);

    const cleanup = () => {
      eventsOff('shell:stdout', stdoutListener);
      eventsOff('shell:stderr', stderrListener);
      eventsOff('shell:close', closeListener);
      eventsOff('shell:error', errorListener);
      this.cleanupFunctions.delete(processId);
      this.logger.debug(`Cleaned up listeners for process ID: ${processId}`);
    };

    // Store the cleanup function
    this.cleanupFunctions.set(processId, cleanup);

    const stdoutListener = (...args: unknown[]) => {
      console.debug(`Received stdout for process ${processId}`);
      const event = args[0] as ShellEvent;
      if (event.processId !== processId) return;
      if (options?.onStdout && event.data) {
        options.onStdout(event.data);
      }
    };

    const stderrListener = (...args: unknown[]) => {
      const event = args[0] as ShellEvent;
      if (event.processId !== processId) return;
      if (options?.onStderr && event.data) {
        options.onStderr(event.data);
      }
    };

    const closeListener = (...args: unknown[]) => {
      const event = args[0] as ShellEvent;
      if (event.processId !== processId) return;
      this.logger.debug(`Process ${event.processId} closed with code: ${event.code}, signal: ${event.signal}`);
      if (options?.onClose) {
        options.onClose(event.code ?? null, event.signal ?? null);
      }
      const cleanupFn = this.cleanupFunctions.get(event.processId);
      if (cleanupFn) {
        cleanupFn();
      }
    };

    const errorListener = (...args: unknown[]) => {
      const event = args[0] as ShellEvent;
      if (event.processId !== processId) return;
      const errorMessage = event.error?.message ?? 'Unknown error';
      this.logger.error(`Process ${event.processId} encountered error: ${errorMessage}`);
      if (options?.onError) {
        options.onError(new Error(errorMessage));
      }
      const cleanupFn = this.cleanupFunctions.get(event.processId);
      if (cleanupFn) {
        cleanupFn();
      }
    };

    eventsOn('shell:stdout', stdoutListener);
    eventsOn('shell:stderr', stderrListener);
    eventsOn('shell:close', closeListener);
    eventsOn('shell:error', errorListener);

    return { processId, pid };
  }

  /**
   * Writes data to the stdin of a spawned process.
   *
   * @param processId The ID of the process to write to.
   * @param data The string data to write.
   */
  async writeStdin(processId: string, data: string): Promise<void> {
    this.logger.debug(`Writing to stdin of process ${processId}: ${data}`);
    shellWriteStdin(processId, data);
  }

  /**
   * Kills a spawned process.
   *
   * @param processId The ID of the process to kill.
   * @param signal The signal to send (e.g., 'SIGTERM', 'SIGKILL').
   */
  async killProcess(processId: string, signal?: string): Promise<void> {
    this.logger.warn(`Killing process ${processId} with signal: ${signal || 'SIGTERM'}`);
    shellKillProcess(processId, signal);

    const cleanupFn = this.cleanupFunctions.get(processId);
    if (cleanupFn) {
      cleanupFn();
    }
  }

  /**
   * Executes a command and waits for it to complete, returning the result (stdout, stderr, code).
   * This is for one-off commands where you only care about the final output.
   *
   * @param command The command to execute.
   * @param args Arguments for the command.
   * @param options Optional record for additional options like cwd, env.
   * @returns A Promise resolving to an object containing stdout, stderr, and exit code.
   */
  async execute(command: string, args?: string[], options?: Record<string, unknown>): Promise<ChildProcess<string>> {
    this.logger.debug(`Executing one-off command: ${command} ${JSON.stringify(args)}`);
    const result = await execute(command, args, options);
    this.logger.debug(`One-off command ${command} finished with code: ${result.code}`);
    return result as ChildProcess<string>;
  }

  /**
   * Cleans up all active event listeners for shell processes.
   * Call this when your application is shutting down or if you want to ensure no stale listeners.
   */
  cleanupAllListeners(): void {
    this.logger.debug('Cleaning up all shell process listeners');
    this.cleanupFunctions.forEach((cleanup) => cleanup());
    this.cleanupFunctions.clear();
  }
}
