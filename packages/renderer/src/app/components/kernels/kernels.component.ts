import { Component, inject } from '@angular/core';
import { DataView } from 'primeng/dataview';
import { FormsModule } from '@angular/forms';
import type { DkmsModule, DkmsModules, Kernel } from './types';
import { type Task, TaskManagerService } from '../task-manager/task-manager.service';
import { Tag } from 'primeng/tag';
import { OsInteractService } from '../task-manager/os-interact.service';
import { Checkbox } from 'primeng/checkbox';
import { TranslocoDirective } from '@jsverse/transloco';
import { ConfigService } from '../config/config.service';
import { Tooltip } from 'primeng/tooltip';
import { Skeleton } from 'primeng/skeleton';
import { KernelsService } from './kernels.service';
import { Router } from '@angular/router';
import { Button } from 'primeng/button';

@Component({
  selector: 'toolbox-kernels',
  imports: [DataView, FormsModule, Tag, Checkbox, TranslocoDirective, Tooltip, Skeleton, Button],
  templateUrl: './kernels.component.html',
  styleUrl: './kernels.component.css',
})
export class KernelsComponent {
  protected readonly configService = inject(ConfigService);
  protected readonly kernelsService = inject(KernelsService);
  private readonly osInteractService = inject(OsInteractService);
  private readonly taskManagerService = inject(TaskManagerService);
  private readonly router = inject(Router);

  /**
   * Toggles the selected state of a package.
   * If an installed kernel without headers is toggled, don't install headers and vice versa.
   * @param item The package to toggle.
   */
  togglePackage(item: Kernel): void {
    if (item.initialState && !item.selected && !item.headersSelected) {
      this.osInteractService.togglePackage(item.pkgname[0]);
    } else {
      for (const pkgname of item.pkgname) {
        this.osInteractService.togglePackage(pkgname);
      }
    }
  }

  /**
   * Install missing headers for a kernel.
   * @param kernel The kernel for which to install missing headers, or `true` to install all missing headers.
   */
  installMissingHeaders(kernel: Kernel | boolean): void {
    if (typeof kernel === 'boolean') {
      for (const kernel of this.kernelsService.kernels()) {
        if (kernel.selected && !kernel.headersSelected) {
          this.osInteractService.togglePackage(kernel.pkgname[1]);
        }
      }
    } else {
      this.osInteractService.togglePackage(kernel.pkgname[1]);
    }
  }

  /**
   * Reinstall DKMS modules that are labeled broken or missing via the task manager service.
   * @param kernel The kernel for which to reinstall DKMS modules, or `true` to reinstall all broken modules.
   * @param type The type of DKMS modules to reinstall, either "broken" or "missing".
   */
  reinstallDkmsModules(kernel: Kernel | boolean, type: 'broken' | 'missing'): void {
    let cmd = '';
    const modules: DkmsModules = this.kernelsService.dkmsModules();

    if (typeof kernel === 'boolean') {
      if (type === 'broken') {
        for (const module of modules) {
          cmd = this.InstallBrokenModules(module, cmd);
        }
      } else {
        for (const kernel of this.kernelsService.kernels()) {
          if (kernel.dkmsModulesMissing.length > 0) {
            cmd = this.InstallMissingModules(kernel, modules, cmd);
          }
        }
      }
    } else {
      if (type === 'broken') {
        const module: DkmsModule | undefined = modules.find(
          (m) => m.kernelVersion === kernel.version && m.moduleName === kernel.pkgname[0],
        );
        if (!module) return;

        cmd = this.InstallBrokenModules(module, cmd);
      } else {
        cmd = this.InstallMissingModules(kernel, modules, cmd);
      }
    }

    const task: Task = this.taskManagerService.createTask(
      10,
      'reinstallDkmsModules',
      true,
      'maintenance.reinstallDkmsModules',
      'pi pi-cog',
      cmd,
    );
    this.taskManagerService.scheduleTask(task);
  }

  /**
   * Install broken DKMS modules for a given kernel.
   * @param module The DKMS module to install.
   * @param cmd The command string to which the installation command will be appended.
   * @private
   */
  private InstallBrokenModules(module: DkmsModule, cmd: string) {
    if (module.status === 'broken') {
      cmd += `dkms install --no-depmod ${module.moduleName}/${module.moduleVersion} -k ${module.kernelVersion}; `;
    }
    return cmd;
  }

  /**
   * Install missing DKMS modules for a given kernel.
   * @param kernel The kernel for which to install missing DKMS modules.
   * @param modules The list of DKMS modules.
   * @param cmd The command string to which the installation command will be appended.
   * @private
   */
  private InstallMissingModules(kernel: Kernel, modules: DkmsModule[], cmd: string) {
    for (const module of kernel.dkmsModulesMissing) {
      const moduleObject: DkmsModule | undefined = modules.find((m) => m.moduleName === module);
      if (!moduleObject) continue;
      if (kernel.pkgname[0] !== 'linux') {
        cmd += `dkms install --no-depmod ${moduleObject?.moduleName}/${moduleObject?.moduleVersion} -k ${kernel.version}-${kernel.pkgname[0].split('linux-')[1]}; `;
      } else {
        cmd += `dkms install --no-depmod ${moduleObject?.moduleName}/${moduleObject?.moduleVersion} -k ${kernel.version}; `;
      }
    }
    return cmd;
  }

  counterArray(number: number): number[] {
    return Array.from({ length: number }, (_, i) => i);
  }

  navigateToDefaultKernel(): void {
    this.router.navigate(['/boot-options'], { queryParams: { highlight: 'default-entry' } });
  }
}
