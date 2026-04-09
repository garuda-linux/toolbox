import { computed, EventEmitter, inject, Injectable, signal } from '@angular/core';
import {
  type ChildProcess,
  ElectronFsService,
  ElectronPathService,
  ShellStreamingResult,
} from '../../electron-services';
import { ElectronShellSpawnService } from '../../electron-services/electron-shell-spawn.service';
import { ConfigService } from '../config/config.service';
import { LoadingService } from '../loading-indicator/loading-indicator.service';
import { Logger } from '../../logging/logging';
import { TranslocoService } from '@jsverse/transloco';

export class Task {
  constructor(priority: number, script: string, escalate: boolean, id: string, name: string, icon: string) {
    this.priority = priority;
    this.script = script;
    this.escalate = escalate;
    this.id = id;
    this.name = name;
    this.icon = icon;
  }

  priority: number;
  script: string;
  escalate: boolean;
  id: string;
  name: string;
  icon: string;
}

export class TrackedShell {
  private readonly logger = Logger.getInstance();
  private processId: string | undefined;
  private resolvePromise: (() => void) | null = null;
  private rejectPromise: ((error: Error) => void) | null = null;
  private outputs: EventEmitter<string>;

  public running = false; // Initial state

  public muted = true;
  private stdoutBuffer = '';

  public startMarker = `[G-START-${crypto.randomUUID()}]`;
  public endMarker = `[G-END-${crypto.randomUUID()}]`;

  constructor(
    private command: string,
    private args: string[],
    outputs: EventEmitter<string>,
    private shellSpawnService: ElectronShellSpawnService,
    private env?: Record<string, string>,
  ) {
    this.outputs = outputs;
  }

  async start(): Promise<void> {
    return new Promise<void>(async (resolve, reject) => {
      this.running = true;
      this.muted = true;
      this.resolvePromise = resolve;
      this.rejectPromise = reject;

      this.logger.trace(`Starting persistent shell: ${this.command} ${JSON.stringify(this.args)}`);

      try {
        // AWAIT the promise to get the actual ShellStreamingResult object
        const result: ShellStreamingResult = await this.shellSpawnService.spawnStreaming(this.command, this.args, {
          env: this.env,
          onStdout: (data) => {
            if (this.muted) {
              this.stdoutBuffer += data;
              const markerIndex = this.stdoutBuffer.indexOf(this.startMarker);
              if (markerIndex !== -1) {
                this.muted = false;
                let afterMarker = this.stdoutBuffer.substring(markerIndex + this.startMarker.length);
                if (afterMarker.startsWith('\n')) {
                  afterMarker = afterMarker.substring(1);
                }

                if (afterMarker) {
                  const endIndex = afterMarker.indexOf(this.endMarker);
                  if (endIndex !== -1) {
                    this.muted = true;
                    const beforeMarker = afterMarker.substring(0, endIndex);
                    if (beforeMarker) this.outputs.emit(beforeMarker);
                  } else {
                    this.outputs.emit(afterMarker);
                  }
                }
                this.stdoutBuffer = '';
              } else if (this.stdoutBuffer.length > 1000) {
                this.stdoutBuffer = this.stdoutBuffer.substring(this.stdoutBuffer.length - 100);
              }
              return;
            }

            this.stdoutBuffer += data;
            const endIndex = this.stdoutBuffer.indexOf(this.endMarker);

            if (endIndex !== -1) {
              this.muted = true;
              const beforeMarker = this.stdoutBuffer.substring(0, endIndex);
              if (beforeMarker) this.outputs.emit(beforeMarker);
              this.stdoutBuffer = '';
              return;
            }

            const lastBracket = this.stdoutBuffer.lastIndexOf('[');
            if (lastBracket !== -1 && this.endMarker.startsWith(this.stdoutBuffer.substring(lastBracket))) {
              if (lastBracket > 0) {
                this.outputs.emit(this.stdoutBuffer.substring(0, lastBracket));
                this.stdoutBuffer = this.stdoutBuffer.substring(lastBracket);
              }
            } else {
              this.outputs.emit(this.stdoutBuffer);
              this.stdoutBuffer = '';
            }
          },
          onStderr: (data) => {
            if (this.muted) return;
            this.outputs.emit(data);
          },
          onClose: (code: number | null, signal: string | null) => {
            this.running = false;
            this.logger.trace(`Persistent shell closed. Code: ${code}, Signal: ${signal}`);
            if (code === 0) {
              this.resolvePromise?.();
            } else {
              this.rejectPromise?.(new Error(`Persistent shell exited with code ${code}`));
            }
          },
          onError: (error: unknown) => {
            this.running = false;
            this.logger.error(`Persistent shell error: ${error instanceof Error ? error.message : String(error)}`);
            this.rejectPromise?.(error instanceof Error ? error : new Error(String(error)));
          },
        });

        this.processId = result.processId;
        if (!this.processId) {
          throw new Error(`Failed to get processId for spawned shell: ${this.command}`);
        }
        // Resolve immediately after spawning, as the shell is now "running"
        // and ready to accept commands. The close/error events will handle termination.
        resolve();
      } catch (error: any) {
        this.running = false; // Ensure running is false on spawn error
        this.logger.error(`Error starting persistent shell: ${error.message}`);
        reject(error); // Reject the outer promise if spawn fails
      }
    });
  }

  async write(data: string): Promise<void> {
    if (!this.processId) {
      throw new Error('Tracked shell not started. Call start() first');
    }
    await this.shellSpawnService.writeStdin(this.processId, data + '\n');
  }

  async writeRaw(data: string): Promise<void> {
    if (!this.processId) {
      throw new Error('Tracked shell not started. Call start() first');
    }
    await this.shellSpawnService.writeStdin(this.processId, data);
  }

  async stop(): Promise<void> {
    if (!this.processId || !this.running) {
      this.logger.debug('Shell not running or already stopped');
      return;
    }

    this.logger.debug(`Stopping persistent shell ${this.processId}`);
    this.muted = true;
    await this.shellSpawnService.writeStdin(this.processId, 'exit 0\n');

    const startTime = Date.now();
    const timeout = 5000;
    while (this.running && Date.now() - startTime < timeout) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    if (this.running) {
      this.logger.warn(`Shell ${this.processId} did not exit gracefully within timeout, killing.`);
      await this.shellSpawnService.killProcess(this.processId, 'SIGTERM'); // Force kill if it didn't stop
    }
  }
}

export class TrackedShells {
  constructor(
    public normal: TrackedShell | null,
    public escalated: TrackedShell | null,
  ) {}

  async startAll(): Promise<void> {
    const promises: Promise<void>[] = [];
    if (this.normal) promises.push(this.normal.start());
    if (this.escalated) promises.push(this.escalated.start());
    await Promise.all(promises);
  }

  async stopAll(): Promise<void> {
    const promises: Promise<void>[] = [];
    // Stop escalated first, then normal (optional, but good practice)
    if (this.escalated) promises.push(this.escalated.stop());
    if (this.normal) promises.push(this.normal.stop());
    await Promise.all(promises);
  }
}

@Injectable({
  providedIn: 'root',
})
export class TaskManagerService {
  private readonly configService = inject(ConfigService);
  private readonly loadingService = inject(LoadingService);
  private readonly logger = Logger.getInstance();
  private readonly translocoService = inject(TranslocoService);
  private readonly fsService = inject(ElectronFsService); // Correctly injected
  private readonly pathService = inject(ElectronPathService);
  private readonly shellStreamingService = inject(ElectronShellSpawnService);

  readonly tasks = signal<Task[]>([]);
  readonly sortedTasks = computed(() => [...this.tasks()].sort((a, b) => a.priority - b.priority));
  readonly currentTask = signal<Task | null>(null);
  readonly running = signal<boolean>(false);
  readonly aborting = signal<boolean>(false);
  readonly count = computed(() => this.tasks().length);
  readonly cachedData = signal<string>('');

  // progress can be null when currentTask is null.
  // If currentTask is not in sortedList, currentIndex is 0.
  // In all other cases, currentIndex is the number of completed tasks (current task index).
  readonly progress = computed(() => {
    const currentTask = this.currentTask();
    if (currentTask === null) {
      return null;
    }
    const sortedList = this.sortedTasks();
    const currentIndex = sortedList.findIndex((task) => task.id === currentTask.id);
    if (currentIndex === -1) {
      return 0;
    }
    return currentIndex;
  });

  readonly events = new EventEmitter<string>();
  readonly dataEvents = new EventEmitter<string>();
  data = '';

  terminalCols = 80;
  terminalRows = 24;

  // Holds the active persistent shell instances
  private activeShells: TrackedShells | null = null;
  private terminalReadyResolver: ((cols: number, rows: number) => void) | null = null;
  private waitForTerminalPromise: Promise<{ cols: number; rows: number }> | null = null;

  constructor() {
    this.logger.debug('TaskManagerService constructor initialized');
    this.dataEvents.subscribe((data) => {
      this.data += data;
      this.cachedData.update((current) => current + data);
    });
  }

  /**
   * Resizes the underlying active shell, if any.
   * @param cols Columns count
   * @param rows Rows count
   */
  async resizeActiveShell(cols: number, rows: number): Promise<void> {
    this.terminalCols = cols;
    this.terminalRows = rows;
  }

  /**
   * Writes raw data (e.g. keystrokes) directly to the currently active shell's stdin.
   * @param data The raw data to write.
   */
  async writeRawToActiveShell(data: string): Promise<void> {
    if (!this.activeShells) return;

    const task = this.currentTask();
    if (!task) return;

    const shell = task.escalate ? this.activeShells.escalated : this.activeShells.normal;
    if (shell && shell.running) {
      await shell.writeRaw(data);
    }
  }

  /**
   * Modifies the provided command to include a specific wrapper for commands using `pkexec`.
   * @param command The command string to be validated and potentially modified.
   * @returns A modified command string that includes the `garuda-toolbox-pty-wrapper`
   *          if the command contains `pkexec` but does not already include the wrapper component.
   */
  private wrapPkexec(command: string): string {
    let newCommand = command;
    if (newCommand.includes('pkexec ') && !newCommand.includes('garuda-toolbox-pty-wrapper')) {
      newCommand = newCommand.replaceAll('pkexec ', 'pkexec /usr/libexec/garuda-toolbox-pty-wrapper ');
    }
    return newCommand;
  }

  /**
   * Execute a bash scriptlet using basic bash and wait for it to finish.
   * This uses the non-streaming `execute` path, likely for one-off commands.
   * @param script The bash scriptlet to execute
   * @param options Options for execution
   */
  async executeAndWaitBash(
    script: string,
    options?: { reinit?: boolean; timeout?: number; forceLang?: boolean },
  ): Promise<ChildProcess<string>> {
    // Change ChildProcess<string> to any due to varied return type
    let result: any; // Type 'any' for now, should be specific return of ipcRenderer.invoke('shell:execute')
    try {
      this.logger.debug(`Executing bash code: ${script}`);
      let command: string = options?.forceLang ? `LANG=C ${script}` : script;
      command = this.wrapPkexec(command);

      // For now, let's directly call invoke if shellService is not defined yet.
      // If you're only using ElectronShellSpawnService for *all* shell interaction,
      // this method should also use the persistent shells.
      result = await this.shellStreamingService.execute('bash', ['--norc', '--noprofile', '-c', command], {
        timeout: options?.timeout,
      });
    } catch (error) {
      this.logger.error(`Unexpected error while executing bash script: ${error}`);
      result = {
        signal: '',
        code: -1,
        stdout: '',
        stderr: error instanceof Error ? error.message : String(error),
      };
    }

    if (options?.reinit) void this.configService.init(false);
    return result;
  }

  /**
   * Execute a bash script in a terminal using garuda-libs and wait for it to finish.
   * @param script The bash scriptlet to execute in the terminal.
   * @param reinit Whether to reinitialize the config service or not.
   * @param timeout Optional timeout in milliseconds for the execution (0 means no timeout).
   */
  async executeAndWaitBashTerminal(
    script: string,
    reinit = false,
    timeout = 0,
  ): Promise<ChildProcess<string> | undefined> {
    let result;
    try {
      this.logger.debug(`Executing bash code in terminal: ${script}`);
      this.loadingService.loadingOn();
      const wrappedScript = this.wrapPkexec(script);
      result = await this.shellStreamingService.execute('/usr/lib/garuda/launch-terminal', [wrappedScript], {
        timeout,
      });
    } catch (error) {
      this.logger.error(`Unexpected error while executing bash script in terminal: ${error}`);
      result = {
        signal: '',
        code: -1,
        stdout: '',
        stderr: error instanceof Error ? error.message : String(error),
      };
    } finally {
      this.loadingService.loadingOff();
    }

    if (reinit) void this.configService.init(false);
    return result;
  }

  /**
   * Create a new task, returning the task.
   * @param priority The priority of the task.
   * @param id The id of the task.
   * @param escalate Whether the task should escalate to root.
   * @param name The name of the task.
   * @param icon The icon of the task.
   * @param script The script to execute.
   * @returns The created task.
   */
  createTask(priority: number, id: string, escalate: boolean, name: string, icon: string, script: string): Task {
    return new Task(priority, script, escalate, id, name, icon);
  }

  /**
   * Schedule a task to be executed.
   * @param task The task to schedule.
   */
  scheduleTask(task: Task): Task {
    this.tasks.update((tasks) => {
      const newTasks: Task[] = [...tasks];
      newTasks.push(task);
      return newTasks;
    });
    return task;
  }

  /**
   * Check if a task is in the task list.
   * @param task The task to check.
   * @returns Whether the task is in the task list as a boolean.
   */
  hasTask(task: Task): boolean {
    return this.tasks().findIndex((t) => t.id === task.id) !== -1;
  }

  /**
   * Remove a task from the task list.
   * @param task The task to remove.
   */
  removeTask(task: Task): void {
    this.tasks.update((tasks) => {
      tasks = tasks.filter((t) => t.id !== task.id);
      return tasks;
    });
  }

  /**
   * Find a task by its id.
   * @param id The id of the task to find.
   */
  findTaskById(id: string): Task | null {
    return this.tasks().find((task) => task.id === id) || null;
  }

  /**
   * Remove all tasks from the task list.
   */
  clearTasks(): void {
    this.tasks.set([]);
  }

  /**
   * Clear terminal and optionally set content to a string.
   * @param content The content to set the terminal to.
   */
  clearTerminal(content = ''): void {
    this.data = content;
    this.events.emit('clear');
    if (content !== '') {
      this.dataEvents.emit(content);
    }
  }

  /**
   * Show the terminal.
   * @param show Whether to show the terminal or not.
   */
  toggleTerminal(show: boolean): void {
    if (show) this.events.emit('show');
    else this.events.emit('hide');
  }

  /**
   * Wait for the terminal to have valid dimensions.
   * @returns Promise that resolves with cols and rows when terminal is ready.
   */
  private waitForTerminalReady(): Promise<{ cols: number; rows: number }> {
    if (this.waitForTerminalPromise) {
      return this.waitForTerminalPromise;
    }

    this.waitForTerminalPromise = new Promise((resolve) => {
      this.terminalReadyResolver = (cols: number, rows: number) => {
        this.terminalCols = cols;
        this.terminalRows = rows;
        this.terminalReadyResolver = null;
        this.waitForTerminalPromise = null;
        resolve({ cols, rows });
      };
    });

    return this.waitForTerminalPromise;
  }

  /**
   * Notify that terminal has valid dimensions (called by terminal component).
   * Only notifies if we're currently waiting.
   */
  notifyTerminalReady(cols: number, rows: number): void {
    if (this.terminalReadyResolver) {
      this.terminalReadyResolver(cols, rows);
    }
  }

  /**
   * Abort the current task.
   */
  abort(): void {
    if (!this.running()) {
      this.logger.error('Abort attempted while not running');
      return;
    }
    this.aborting.set(true);
  }

  /**
   * Execute a specific task with extra safeguards.
   * @param task The task to execute.
   * @param shells The shells to use.
   * @private
   */
  private async internalExecuteTask(task: Task, shells: TrackedShells): Promise<void> {
    this.logger.debug(`Executing task: ${task.script}`);

    const shell: TrackedShell | null = task.escalate ? shells.escalated : shells.normal;
    if (!shell) {
      throw new Error(`No ${task.escalate ? 'escalated' : 'normal'} shell available for task ${task.name}`);
    }

    const appLocalDataDirectory = await this.pathService.appLocalDataDir();
    const path: string = await this.pathService.resolve(appLocalDataDirectory, 'taskscript.tmp');
    const executorPath: string = await this.pathService.resolve(appLocalDataDirectory, 'executor.tmp');

    // Safety check, make sure path does not contain unsafe characters
    if (path.includes("'") || executorPath.includes("'")) {
      this.logger.error('Path contains unsafe character: ' + path);
      throw new Error('Unsafe path character detected');
    }

    // Write script to a temporary file
    const script: string = task.script.trim();
    await this.fsService.writeTextFile(path, script);
    const digest: ArrayBuffer = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(script));
    const hash = Array.from(new Uint8Array(digest))
      .map((b) => b.toString(16).padStart(2, '0'))
      .join('');
    const taskFinishedMessage: string = this.translocoService.translate('taskmanager.scriptExecuted');

    // Write the executor script
    const executorScript = `
#!/bin/bash
stty cols ${this.terminalCols} rows ${this.terminalRows} 2>/dev/null || true
echo "${shell.startMarker}"
# Read file into variable
script=$(<'${path}')
# Check if the script is the same as the one we wrote
if [ "$(printf '%s' "$script" | sha256sum | cut -d ' ' -f 1)" != "${hash}" ]; then
  echo "${this.translocoService.translate('taskmanager.scriptMismatch')}"
  echo "${shell.endMarker}"
  exit 1
fi
# Execute the script, -x for debugging output
bash -x "${path}"
rm '${path}'
printf "\\n${taskFinishedMessage}\\n"
rm '${executorPath}'
echo "${shell.endMarker}"
`;
    await this.fsService.writeTextFile(executorPath, executorScript);
    await shell.write(`bash "${executorPath}"`);

    while ((await this.fsService.exists(path)) && shell.running) {
      await new Promise((resolve) => setTimeout(resolve, 500));
    }
    this.logger.info(`Task ${task.name} has finished`);
  }

  private async getPtyCommandAndArgs(shellCommand: string): Promise<{ command: string; args: string[] }> {
    const wrapperPath = '/usr/libexec/garuda-toolbox-pty-wrapper';
    if (await this.fsService.exists(wrapperPath)) {
      return {
        command: wrapperPath,
        args: ['--pty', '-q', '-e', '-c', shellCommand, '/dev/null'],
      };
    }
    return {
      command: '/usr/bin/script',
      args: ['-q', '-e', '-c', shellCommand, '/dev/null'],
    };
  }

  /**
   * Execute a given task a single time.
   * @param task The task to execute.
   */
  async executeTask(task: Task): Promise<void> {
    if (this.running()) {
      this.logger.error('Task manager is already running a task');
      return;
    }
    this.running.set(true);
    this.clearTerminal();

    this.waitForTerminalPromise = null;
    this.terminalReadyResolver = null;

    const readyPromise = this.waitForTerminalReady();

    this.toggleTerminal(true);
    await new Promise((resolve) => setTimeout(resolve, 50));

    const { cols, rows } = await Promise.race([
      readyPromise,
      new Promise<{ cols: number; rows: number }>((resolve) =>
        setTimeout(() => {
          this.logger.warn(
            `Terminal ready timeout, using current dimensions: ${this.terminalCols}x${this.terminalRows}`,
          );
          resolve({ cols: this.terminalCols, rows: this.terminalRows });
        }, 2000),
      ),
    ]);

    this.terminalCols = cols;
    this.terminalRows = rows;

    const shellCommand = `stty cols ${this.terminalCols} rows ${this.terminalRows} 2>/dev/null || true; env PS1="" PROMPT_COMMAND="" bash --norc`;
    const ptySetup = await this.getPtyCommandAndArgs(shellCommand);

    this.activeShells = new TrackedShells(
      task.escalate
        ? null
        : new TrackedShell(ptySetup.command, ptySetup.args, this.dataEvents, this.shellStreamingService, {
            COLUMNS: this.terminalCols.toString(),
            LINES: this.terminalRows.toString(),
            TERM: 'xterm-256color',
            PS1: '',
            PROMPT_COMMAND: '',
          }), // Normal shell
      task.escalate
        ? new TrackedShell(
            'pkexec',
            [ptySetup.command, ...ptySetup.args],
            this.dataEvents,
            this.shellStreamingService,
            {
              COLUMNS: this.terminalCols.toString(),
              LINES: this.terminalRows.toString(),
              TERM: 'xterm-256color',
              PS1: '',
              PROMPT_COMMAND: '',
            },
          )
        : null, // Escalated shell
    );

    try {
      await this.activeShells.startAll();
      this.currentTask.set(task);
      await this.internalExecuteTask(task, this.activeShells);
    } catch (error: unknown) {
      this.logger.error(`Task execution failed: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      this.currentTask.set(null);
      this.removeTask(task);
      if (this.activeShells) {
        await this.activeShells.stopAll();
        this.activeShells = null;
      }
      this.running.set(false);
      this.aborting.set(false);
      void this.configService.init(false);
    }
  }

  /**
   * Execute all tasks in the task list.
   */
  async executeTasks(): Promise<void> {
    if (this.running()) {
      this.logger.error('Task manager is already running a task');
      return;
    }
    this.running.set(true);
    this.clearTerminal();

    this.waitForTerminalPromise = null;
    this.terminalReadyResolver = null;

    const readyPromise = this.waitForTerminalReady();

    this.toggleTerminal(true);
    await new Promise((resolve) => setTimeout(resolve, 50));

    const { cols, rows } = await Promise.race([
      readyPromise,
      new Promise<{ cols: number; rows: number }>((resolve) =>
        setTimeout(() => {
          this.logger.warn(
            `Terminal ready timeout, using current dimensions: ${this.terminalCols}x${this.terminalRows}`,
          );
          resolve({ cols: this.terminalCols, rows: this.terminalRows });
        }, 2000),
      ),
    ]);

    this.terminalCols = cols;
    this.terminalRows = rows;

    const needsNormal = this.tasks().some((task) => !task.escalate);
    const needsEscalated = this.tasks().some((task) => task.escalate);

    const shellCommand = `stty cols ${this.terminalCols} rows ${this.terminalRows} 2>/dev/null || true; env PS1="" PROMPT_COMMAND="" bash --norc`;
    const ptySetup = await this.getPtyCommandAndArgs(shellCommand);

    this.activeShells = new TrackedShells(
      needsNormal
        ? new TrackedShell(ptySetup.command, ptySetup.args, this.dataEvents, this.shellStreamingService, {
            COLUMNS: this.terminalCols.toString(),
            LINES: this.terminalRows.toString(),
            TERM: 'xterm-256color',
            PS1: '',
            PROMPT_COMMAND: '',
          })
        : null,
      needsEscalated
        ? new TrackedShell(
            'pkexec',
            [ptySetup.command, ...ptySetup.args],
            this.dataEvents,
            this.shellStreamingService,
            {
              COLUMNS: this.terminalCols.toString(),
              LINES: this.terminalRows.toString(),
              TERM: 'xterm-256color',
              PS1: '',
              PROMPT_COMMAND: '',
            },
          )
        : null,
    );

    try {
      await this.activeShells.startAll();
      for (const task of this.sortedTasks()) {
        if (this.aborting()) {
          break; // Stop if aborting is signaled
        }
        this.currentTask.set(task);
        try {
          await this.internalExecuteTask(task, this.activeShells);
        } catch (error: unknown) {
          this.logger.error(
            `Task execution failed for task ${task.name}: ${error instanceof Error ? error.message : String(error)}`,
          );
        }
      }
    } catch (error: unknown) {
      this.logger.error(
        `Error during shell startup or task execution: ${error instanceof Error ? error.message : String(error)}`,
      );
    } finally {
      this.currentTask.set(null);
      this.tasks.set([]);
      if (this.activeShells) {
        await this.activeShells.stopAll();
        this.activeShells = null;
      }
      this.running.set(false);
      this.aborting.set(false);
      void this.configService.init(false);
    }
  }

  /**
   * Only print the generated bash scripts to terminal without running them.
   * Can be used to debug tasks and verify the generated scripts, as well as
   * ensure that the scripts are correctly formatted and free of errors.
   */
  async printScripts(): Promise<void> {
    this.clearTerminal();

    let first = true;
    for (const task of this.sortedTasks()) {
      if (!first) {
        this.dataEvents.emit(`${'_'.repeat(75)}\n`);
      }
      first = false;

      const emitting: string = `
        Name: ${this.translocoService.translate(task.name)}
        Escalate: ${task.escalate}
        Priority: ${task.priority}

        Script:
        ${task.script}
      `.replaceAll(/[^\S\n]{2,}/g, '');

      this.dataEvents.emit(emitting);
    }
  }
}
