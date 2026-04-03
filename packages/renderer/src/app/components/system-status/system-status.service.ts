import { effect, inject, Injectable, signal } from '@angular/core';
import type { HealthError, SystemUpdate, UpdateStatusOption, UpdateType } from './types';
import { type Task, TaskManagerService } from '../task-manager/task-manager.service';
import { ConfigService } from '../config/config.service';
import { LoadingService } from '../loading-indicator/loading-indicator.service';
import { Logger } from '../../logging/logging';
import { type ChildProcess, ElectronShellService } from '../../electron-services';

@Injectable({
  providedIn: 'root',
})
export class SystemStatusService {
  firstRun = true;
  pacFiles = signal<string[]>([]);
  updates = signal<SystemUpdate[]>([]);
  updatesAur = signal<boolean>(false);
  warnUpdate = signal<boolean>(false);
  healthErrors = signal<HealthError[]>([]);

  private readonly loadingService = inject(LoadingService);
  private readonly logger = Logger.getInstance();
  private readonly taskManagerService = inject(TaskManagerService);
  private readonly shellService = inject(ElectronShellService);
  private readonly configService = inject(ConfigService);

  constructor() {
    effect(async () => {
      const _tasks: Task[] = this.taskManagerService.tasks();
      if (!this.firstRun) {
        const allPromises: Promise<void>[] = this.refreshStatuses();
        await Promise.all(allPromises);
        this.checkAurUpdates();
      }
    });

    void this.init();
  }

  async init() {
    this.loadingService.loadingOn();

    const initPromises: Promise<void>[] = this.refreshStatuses();
    await Promise.all(initPromises);
    this.checkAurUpdates();

    this.loadingService.loadingOff();
    this.firstRun = false;
  }

  checkAurUpdates() {
    if (this.updates().some((update: SystemUpdate) => update.aur)) {
      this.updatesAur.set(true);
    } else {
      this.updatesAur.set(false);
    }

    this.updates.set([...this.updates()]);
  }

  /**
   * Refresh the system statuses.
   * @returns An array of promises that will be resolved when the statuses are refreshed.
   */
  private refreshStatuses(): Promise<void>[] {
    this.pacFiles.set([]);
    this.updates.set([]);
    this.healthErrors.set([]);

    return [
      this.getPacFiles(),
      this.checkSystemUpdate('checkupdates --nocolor', 'repo'),
      this.checkSystemUpdate('paru -Qua', 'aur'),
      this.checkLastUpdate(),
      this.checkGarudaHealth(),
    ];
  }

  /**
   * Check for system health issues with garuda-health.
   */
  private async checkGarudaHealth(): Promise<void> {
    if (this.configService.state().isLiveSystem === true) {
      this.logger.debug('Skipping garuda-health on live system');
      this.healthErrors.set([]);
      return;
    }

    try {
      this.logger.debug('Running garuda-health --json');
      const result = await this.taskManagerService.executeAndWaitBash('garuda-health --json || true');

      if (result.stdout.trim()) {
        const healthData = JSON.parse(result.stdout.trim());
        const errors: HealthError[] = [];

        if (healthData.INFO && Array.isArray(healthData.INFO)) {
          for (const item of healthData.INFO) {
            errors.push({
              title: 'Health info',
              message: item.description,
              fixAvailable: item.fix_available || false,
            });
          }
        }

        if (healthData.WARNING && Array.isArray(healthData.WARNING)) {
          for (const item of healthData.WARNING) {
            errors.push({
              title: 'Health warning',
              message: item.description,
              fixAvailable: item.fix_available || false,
            });
          }
        }

        if (healthData.ERROR && Array.isArray(healthData.ERROR)) {
          for (const item of healthData.ERROR) {
            errors.push({
              title: 'Health error',
              message: item.description,
              fixAvailable: item.fix_available || false,
            });
          }
        }

        this.healthErrors.set(errors);
      } else {
        this.healthErrors.set([]);
      }
    } catch (error) {
      this.logger.error(`Failed to run garuda-health: ${error}`);
      this.healthErrors.set([]);
    }
  }

  /**
   * Get a list of pacfiles to check and merge.
   */
  private async getPacFiles(): Promise<void> {
    const cmd = 'pacdiff -o';
    const result: ChildProcess<string> = await this.shellService.execute('bash', ['--norc', '--noprofile', '-c', cmd]);

    if (result.code === 0) {
      if (result.stdout.trim() === '') return;

      this.pacFiles.set(result.stdout.trim().split('\n') ?? []);
      this.logger.trace(`Pacfiles: ${this.pacFiles().join(', ')}`);
    } else {
      this.logger.error(`Failed to get pacfiles: ${result.stderr}`);
    }
  }

  /**
   * Check for system updates, either from the repo or AUR.
   * @param cmd The command to run to check for updates.
   * @param type The type of updates to check for.
   */
  private async checkSystemUpdate(cmd: string, type: UpdateStatusOption): Promise<void> {
    const result: ChildProcess<string> = await this.shellService.execute('bash', ['--norc', '--noprofile', '-c', cmd]);
    const updateString: UpdateType = type === 'repo' ? 'Updates' : 'AUR updates';

    if (result.code === 0) {
      const updates: string[] = result.stdout.trim().split('\n') ?? [];
      for (const update of updates) {
        this.logger.trace(`${updateString}: ${update}`);

        const [pkg, version, _invalid, newVersion] = update.split(' ');
        this.updates.update((updates: SystemUpdate[]) => {
          updates.push({
            pkg,
            version,
            newVersion: newVersion,
            aur: type === 'aur',
          });
          return updates;
        });
      }
    } else if ((type === 'repo' && result.code === 2) || (type === 'aur' && result.code === 1)) {
      this.logger.info(`No ${updateString.toLowerCase()} available`);
    } else {
      this.logger.error(`Failed to get ${updateString.toLowerCase()}: ${result.stderr}`);
    }
  }

  /**
   * Check the last update time.
   */
  private async checkLastUpdate(): Promise<void> {
    const cmd = 'awk \'END{sub(/\\[/,""); print $1}\' /var/log/pacman.log';
    const result: ChildProcess<string> = await this.shellService.execute('bash', ['--norc', '--noprofile', '-c', cmd]);

    if (result.code === 0) {
      const date = new Date(result.stdout.trim().replace(']', ''));
      this.logger.info(`Last update: ${date.toISOString()}`);

      if (date < new Date(new Date().setDate(new Date().getDate() - 14))) {
        this.logger.warn('Last update was more than two week ago');
        this.warnUpdate.set(true);
      }
    } else {
      this.logger.error(`Failed to get last update: ${result.stderr}`);
    }
  }
}
