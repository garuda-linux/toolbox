import { ChangeDetectionStrategy, Component, computed, inject, signal } from '@angular/core';
import { ElectronShellService } from '../../electron-services';
import { OverlayBadge } from 'primeng/overlaybadge';
import { Tooltip } from 'primeng/tooltip';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { Dialog } from 'primeng/dialog';
import { Button } from 'primeng/button';
import { type Task, TaskManagerService } from '../task-manager/task-manager.service';
import type { SystemUpdate } from './types';
import { ConfigService } from '../config/config.service';
import { RouterLink } from '@angular/router';
import { ConfirmationService } from 'primeng/api';
import { SystemStatusService } from './system-status.service';
import { OsInteractService } from '../task-manager/os-interact.service';

@Component({
  selector: 'toolbox-system-status',
  imports: [OverlayBadge, Tooltip, TranslocoDirective, Dialog, Button, RouterLink],
  templateUrl: './system-status.component.html',
  styleUrl: './system-status.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SystemStatusComponent {
  dialogVisible = signal<boolean>(false);
  pacdiffDialogVisible = signal<boolean>(false);

  protected readonly configService = inject(ConfigService);
  protected readonly systemStatusService = inject(SystemStatusService);
  protected readonly shellService = inject(ElectronShellService);

  private readonly confirmationService = inject(ConfirmationService);
  private readonly osInteractionService = inject(OsInteractService);
  private readonly taskManagerService = inject(TaskManagerService);
  private readonly translocoService = inject(TranslocoService);

  updateButtonDisabled = computed(() => this.taskManagerService.findTaskById('updateSystem') !== null);

  /**
   * Schedule a system update, confirming with the user first. If confirmed, schedule the update.
   * @param confirmed Whether the user has confirmed the update.
   */
  scheduleUpdates(confirmed = false): void {
    if (!confirmed) {
      this.dialogVisible.set(true);
      return;
    }

    if (
      this.systemStatusService.updates().length > 0 &&
      this.systemStatusService.updates().some((update: SystemUpdate) => !update.aur)
    ) {
      const task: Task = this.taskManagerService.createTask(
        0,
        'updateSystem',
        true,
        'maintenance.updateSystem',
        'pi pi-refresh',
        `if command -v garuda-update >/dev/null 2>&1; then \
          GARUDA_UPDATE_RANI=1 garuda-update --skip-mirrorlist; \
          else pacman -Syu; fi`,
      );
      this.taskManagerService.scheduleTask(task);
    }

    this.dialogVisible.set(false);
  }

  /**
   * Prompt for confirmation before rebooting the system.
   * @param $event The event that triggered the confirmation.
   */
  rebootNow($event: Event): void {
    this.confirmationService.confirm({
      target: $event.target as EventTarget,
      message: this.translocoService.translate('systemStatus.rebootNowBody'),
      header: this.translocoService.translate('systemStatus.rebootNow'),
      icon: 'pi pi-exclamation-triangle',
      acceptButtonProps: {
        severity: 'danger',
        label: this.translocoService.translate('confirmation.accept'),
      },
      rejectButtonProps: {
        severity: 'secondary',
        label: this.translocoService.translate('confirmation.reject'),
      },
      accept: () => {
        void this.taskManagerService.executeAndWaitBash('systemctl reboot');
      },
    });
  }

  /**
   * Run AUR updates in a new terminal. We do it like this to be able to take full
   * advantage of reviewing the updates before installing them.
   */
  runAurUpdates() {
    void this.taskManagerService.executeAndWaitBashTerminal('paru -Sua', true);
  }

  /**
   * Refreshes all Toolbox system state (even though this is usually done automatically at the end of each task).
   * This is useful for manual refreshes, e.g. when a user manually outside the app.
   */
  async refreshState() {
    await Promise.allSettled([this.systemStatusService.init(), this.osInteractionService.update()]);
  }
}
