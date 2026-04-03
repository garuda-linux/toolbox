import { computed, effect, inject, Injectable, signal, untracked } from '@angular/core';
import type { ChildProcess } from '../../types/shell';
import type { DkmsModules, DkmsModuleStatus, Kernel, Kernels } from './types';
import { ConfigService } from '../config/config.service';
import { LoadingService } from '../loading-indicator/loading-indicator.service';
import { Logger } from '../../logging/logging';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { OsInteractService } from '../task-manager/os-interact.service';

@Injectable({
  providedIn: 'root',
})
export class KernelsService {
  availableModules = signal<string[]>([]);
  dkmsModules = signal<DkmsModules>([]);
  dkmsModulesBroken = computed(() => this.dkmsModules().some((module) => module.status !== 'installed'));
  dkmsModulesMissing = signal<boolean>(false);
  kernels = signal<Kernels>([]);
  headersMissing = computed(() => this.kernels().some((kernel) => kernel.selected && !kernel.headersSelected));
  loading = signal<boolean>(true);

  protected readonly configService = inject(ConfigService);
  private readonly loadingService = inject(LoadingService);
  private readonly logger = Logger.getInstance();
  private readonly osInteractService = inject(OsInteractService);
  private readonly taskManagerService = inject(TaskManagerService);

  constructor() {
    effect(() => {
      const _packages: Map<string, boolean> = this.osInteractService.packages();
      const _kernels: Kernels = this.kernels();
      if (!untracked(this.loading)) {
        this.updateKernelStatus();
      }
    });

    void this.init();
  }

  async init() {
    this.loadingService.loadingOn();

    const promises: Promise<void>[] = [
      this.getAvailableKernels(),
      this.getDkmsStatus(),
      this.getAvailableDkmsModules(),
    ];
    await Promise.all(promises);

    this.updateKernelStatus();
    this.logger.trace('Kernel status update complete');

    this.loadingService.loadingOff();
    this.loading.set(false);
  }

  /**
   * Get the status of DKMS modules, if DKMS is installed.
   */
  async getDkmsStatus() {
    const cmd = 'which dkms &>/dev/null && dkms status';
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);

    if (result.code === 0 && result.stdout.trim() !== '') {
      const lines: string[] = result.stdout
        .split('\n')
        .map((line: string) => line.trim())
        .filter((line: string) => line.match(/(: installed|: broken)/));

      const modules: DkmsModules = [];
      for (const line of lines) {
        const [moduleString, status] = line.split(': ');
        const [module, kernelVersion] = moduleString.split(', ');
        const [moduleName, moduleVersion] = module.split('/');

        this.logger.trace(`${module} for kernel version ${kernelVersion} is ${status}`);
        modules.push({
          moduleName: moduleName || '',
          moduleVersion: moduleVersion || '',
          kernelVersion: kernelVersion || '',
          status: (status.split(' ')[0] as DkmsModuleStatus) || 'unknown',
        });
      }

      this.dkmsModules.set(modules);
      this.logger.info(`Found ${this.dkmsModules().length} installed DKMS modules among kernels`);
      this.logger.trace(JSON.stringify(this.dkmsModules()));
    } else {
      this.logger.error(`Failed to get kernel DKMS status: ${result.stderr}`);
    }
  }

  /**
   * Report which DKMS modules are available, if DKMS is installed.
   */
  async getAvailableDkmsModules() {
    const cmd =
      'test -d /var/lib/dkms && find /var/lib/dkms -maxdepth 1 -type d | grep /var/lib/dkms/ | cut -d "/" -f 5';
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);

    if (result.code === 0 && result.stdout.trim() !== '') {
      const modules: string[] = result.stdout.trim().split('\n');
      this.availableModules.set(modules);
      this.logger.info(`Found ${this.availableModules().length} available DKMS modules`);
    } else {
      this.logger.error(`Failed to get available DKMS modules: ${result.stderr}`);
    }
  }

  /**
   * Get available kernels from the package manager, under the assumption that every kernel has a corresponding headers package.
   * This is a best-effort approach, and relies on proper packaging conventions.
   */
  async getAvailableKernels() {
    const cmd = 'pacman -Ss linux';
    const result: ChildProcess<string> = await this.taskManagerService.executeAndWaitBash(cmd);

    if (result.code === 0) {
      let kernels: Kernel[] = [];
      const kernelMap: Record<string, string> = {};
      const lines: string[] = result.stdout.split('\n').map((line: string) => line.trim());

      for (let i = 0; i < lines.length; i += 2) {
        const [originalKernelName, version] = lines[i].split(' ');
        if (originalKernelName.endsWith('-headers')) {
          const kernelName = originalKernelName.replace('-headers', '');
          const [repo, name] = kernelName.split('/');
          if (kernelMap[kernelName] === version) {
            kernels.push({
              pkgname: [name, `${name}-headers`],
              version,
              repo,
              description: '',
              dkmsModulesMissing: [],
              initialState: this.osInteractService.check(name, 'pkg', true),
            });
          }
        } else {
          kernelMap[originalKernelName] = version;
        }
      }

      // Extract real descriptions, because headers are different
      for (let i = 0; i < lines.length; i += 2) {
        const [nameWithRepo] = lines[i].split(' ');
        const [repo, name] = nameWithRepo.split('/');
        const kernel: Kernel | undefined = kernels.find((k) => k.pkgname[0] === name && k.repo === repo);
        if (kernel) {
          kernel.description = lines[i + 1];
        }
      }

      // Filter duplicates and show the one, which will be installed by pacman
      // Prioritize kernel based on the one pacman would pick in a default Garuda Linux installation.
      const repoPriority: Record<string, number> = {
        'garuda': 0,
        'core': 1,
        'extra': 2,
        'chaotic-aur': 3,
      };
      const uniqueKernelMap = new Map<string, Kernel>();
      for (const kernel of kernels) {
        const existing = uniqueKernelMap.get(kernel.pkgname[0]);
        if (!existing || (repoPriority[kernel.repo] ?? 999) < (repoPriority[existing.repo] ?? 999)) {
          uniqueKernelMap.set(kernel.pkgname[0], kernel);
        }
      }
      kernels = Array.from(uniqueKernelMap.values());

      this.kernels.set(kernels);
      this.logger.info(`Found ${kernels.length} available kernels`);
      this.logger.trace(JSON.stringify(kernels));
    } else {
      this.logger.error(`Failed to get available kernels: ${result.stderr}`);
    }
  }

  /**
   * Update the state of the UI based on the installed packages, and sort them.
   */
  updateKernelStatus(): void {
    this.logger.trace('Updating kernels UI');
    const installedPackages: Map<string, boolean> = this.osInteractService.packages();
    const modules: DkmsModules = this.dkmsModules();
    const availableModules: string[] = this.availableModules();
    let missingModules = false;

    this.kernels.update((kernels: Kernels) => {
      for (const kernel of kernels) {
        kernel.selected = installedPackages.get(kernel.pkgname[0]) === true;
        kernel.headersSelected = installedPackages.get(kernel.pkgname[1]) === true;
        kernel.dkmsModulesMissing = [];

        if (kernel.selected && kernel.headersSelected) {
          for (const module of availableModules) {
            this.logger.trace(`Checking DKMS module ${module} for kernel ${kernel.pkgname[0]}`);
            const versionMatch = kernel.version.match(/\d+\.\d+\.\d+/);
            if (!versionMatch) {
              this.logger.warn(`Could not extract version from ${kernel.version}`);
              continue;
            }
            const linuxVer: string = versionMatch[0];
            let regex: RegExp;

            if (kernel.pkgname[0] === 'linux') {
              regex = new RegExp(`^${linuxVer.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}-arch`);
            } else {
              const linuxType: string = kernel.pkgname[0].split('linux-')[1];
              regex = new RegExp(`^${linuxVer.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}.*${linuxType}`);
            }

            if (
              !modules.find((k) => k.kernelVersion && k.kernelVersion.match(regex) !== null && k.moduleName === module)
            ) {
              this.logger.warn(
                `DKMS Module ${module} for kernel ${kernel.pkgname[0]} is available but not installed, hinting at a possible issue`,
              );
              kernel.dkmsModulesMissing.push(module);
              missingModules = true;
            }
          }
        }
      }

      this.logger.trace('Done determining status, proceeding to sort kernels');

      // Show selected kernels first
      kernels.sort((a, b) => +(b.selected || false) - +(a.selected || false));
      return kernels;
    });

    this.logger.trace('Kernels after sorting: ' + JSON.stringify(this.kernels()));
    this.dkmsModulesMissing.set(missingModules);
  }
}
