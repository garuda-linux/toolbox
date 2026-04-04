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
import { CommandPaletteService } from '../command-palette/command-palette.service';

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
  healthDialogVisible = signal<boolean>(false);

  protected readonly configService = inject(ConfigService);
  protected readonly systemStatusService = inject(SystemStatusService);
  protected readonly shellService = inject(ElectronShellService);

  private readonly confirmationService = inject(ConfirmationService);
  private readonly osInteractionService = inject(OsInteractService);
  private readonly taskManagerService = inject(TaskManagerService);
  private readonly translocoService = inject(TranslocoService);
  private readonly commandPaletteService = inject(CommandPaletteService);

  constructor() {
    this.registerCommandPaletteActions();
  }

  private registerCommandPaletteActions(): void {
    this.commandPaletteService.registerActions(
      {
        id: 'system-update',
        label: 'systemStatus.scheduleUpdates',
        description: 'systemStatus.updates',
        icon: 'pi pi-refresh',
        keywords: ['update', 'system', 'upgrade', 'schedule'],
        category: 'system',
        command: (): void => this.scheduleUpdates(true),
      },
      {
        id: 'aur-update',
        label: 'systemStatus.runAurUpdates',
        icon: 'pi pi-upload',
        keywords: ['aur', 'update', 'paru', 'yay'],
        category: 'system',
        command: (): void => this.runAurUpdates(),
      },
      {
        id: 'garuda-health-fix',
        label: 'systemStatus.runHealthFix',
        icon: 'pi pi-heart-fill',
        keywords: ['health', 'fix', 'garuda-health', 'check'],
        category: 'system',
        command: (): Promise<void> => this.runHealthFix(),
      },
      {
        id: 'refresh-state',
        label: 'maintenance.refreshMirrors',
        icon: 'pi pi-sync',
        keywords: ['refresh', 'reload', 'state', 'sync', 'status'],
        category: 'system',
        command: (): Promise<void> => this.refreshState(),
      },
    );
  }

  updateButtonDisabled = computed(() => this.taskManagerService.findTaskById('updateSystem') !== null);
  healthFixAvailable = computed(() => this.systemStatusService.healthErrors().some((error) => error.fixAvailable));

  hasRegularUpdates = computed(() => this.systemStatusService.updates().some((update: SystemUpdate) => !update.aur));
  hasAurUpdates = computed(() => this.systemStatusService.updates().some((update: SystemUpdate) => update.aur));

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
   * Run the garuda-health --fix command in a terminal.
   */
  async runHealthFix() {
    this.healthDialogVisible.set(false);
    const task = this.taskManagerService.createTask(
      0,
      'healthFix',
      true,
      this.translocoService.translate('maintenance.garudaHealthFix'),
      'pi pi-heart-fill',
      'garuda-health --fix; read -p "Press Enter to close"',
    );
    await this.taskManagerService.executeTask(task);
    this.taskManagerService.toggleTerminal(true);
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
