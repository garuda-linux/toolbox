import { ChangeDetectionStrategy, Component, computed, inject, type OnInit, signal } from '@angular/core';
import { Button } from 'primeng/button';
import { IconField } from 'primeng/iconfield';
import { InputIcon } from 'primeng/inputicon';
import { InputText } from 'primeng/inputtext';
import { type Table, TableModule } from 'primeng/table';
import type { SystemdService, SystemdServiceAction } from '../../interfaces';

import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { type Popover, PopoverModule } from 'primeng/popover';
import { MessageToastService } from '@garudalinux/core';
import type { Nullable } from 'primeng/ts-helpers';
import { Tooltip } from 'primeng/tooltip';
import { ConfigService } from '../config/config.service';
import { Logger } from '../../logging/logging';
import { TaskManagerService } from '../task-manager/task-manager.service';

@Component({
  selector: 'toolbox-systemd-services',
  imports: [Button, IconField, InputIcon, PopoverModule, InputText, TableModule, TranslocoDirective, Tooltip],
  templateUrl: './systemd-services.component.html',
  styleUrl: './systemd-services.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SystemdServicesComponent implements OnInit {
  activeService = signal<SystemdService | null>(null);
  includeDisabled = signal<boolean>(false);
  hideDead = signal<boolean>(true);
  loading = signal<boolean>(true);
  serviceSearch = signal<string>('');
  systemdServices = signal<SystemdService[]>([]);

  filteredServices = computed(() => {
    if (!this.hideDead()) {
      return this.systemdServices();
    }
    return this.systemdServices().filter((s) => s.sub !== 'dead' && s.sub !== 'failed' && s.sub !== 'exited');
  });

  intervalRef: ReturnType<typeof setInterval> | undefined = undefined;

  protected readonly configService = inject(ConfigService);
  private readonly logger = Logger.getInstance();
  private readonly messageToastService = inject(MessageToastService);
  private readonly translocoService = inject(TranslocoService);
  private readonly taskManagerService = inject(TaskManagerService);

  async ngOnInit() {
    this.logger.debug('Initializing system tools');
    this.systemdServices.set(await this.getServices());

    if (this.configService.settings().autoRefresh) {
      this.intervalRef = setInterval(async () => {
        this.systemdServices.set(await this.getServices());
      }, 5000);
      this.logger.debug('Started auto-refresh');
    }

    this.loading.set(false);
  }

  /**
   * Get the active systemd services, destined for the table. Searches either user or system services.
   */
  async getServices(): Promise<SystemdService[]> {
    const toDo: string[] = [
      `systemctl ${this.configService.settings().systemdUserContext ? '--user ' : ''}list-units --type service,socket --full --all --output json --no-pager`,
    ];
    if (this.includeDisabled()) {
      toDo.push(
        `systemctl ${this.configService.settings().systemdUserContext ? '--user ' : ''}list-unit-files --type=service,socket --state=disabled --full --all --output json --no-pager`,
      );
    }

    const servicePromises: Promise<Nullable<SystemdService[]>>[] = [];
    for (const cmd of toDo) {
      servicePromises.push(
        this.taskManagerService.executeAndWaitBash(cmd).then((out) => {
          const services = JSON.parse(out.stdout) as SystemdService[];
          for (const service of services) {
            if (service.unit && service.unit.length > 50) {
              service.tooltip = service.unit;
              service.unit = `${service.unit.slice(0, 50)}...`;
            } else if (service.unit_file) {
              if (service.unit_file.length > 50) {
                service.tooltip = service.unit_file;
                service.unit = `${service.unit_file.slice(0, 50)}...`;
              } else {
                service.unit = service.unit_file;
              }
              service.unit_file = undefined;
            }
          }
          return services;
        }),
      );
    }

    const finalResult: SystemdService[] = [];
    const results: Nullable<SystemdService[]>[] = await Promise.all(servicePromises);
    for (const result of results) {
      if (result) {
        finalResult.push(...result);
      }
    }
    return finalResult;
  }

  /**
   * Clear the systemd service table search and options.
   * @param table The table component to clear
   */
  clear(table: Table): void {
    table.clear();
    this.serviceSearch.set('');
  }

  /**
   * Execute a systemd service action.
   * @param event The systemd service action
   */
  async executeAction(event: SystemdServiceAction): Promise<void> {
    if (!this.activeService()) {
      this.logger.error('No active service selected, something went wrong here');
      return;
    }

    let action: string;
    switch (event) {
      case 'start':
        action = `systemctl ${this.configService.settings().systemdUserContext ? '--user ' : ''}start`;
        break;
      case 'stop':
        action = `systemctl ${this.configService.settings().systemdUserContext ? '--user ' : ''}stop`;
        break;
      case 'restart':
        action = `systemctl ${this.configService.settings().systemdUserContext ? '--user ' : ''}restart`;
        break;
      case 'reload':
        action = `systemctl ${this.configService.settings().systemdUserContext ? '--user ' : ''}reload`;
        break;
      case 'enable':
        action = `systemctl ${this.configService.settings().systemdUserContext ? '--user ' : ''}enable --now`;
        break;
      case 'disable':
        action = `systemctl ${this.configService.settings().systemdUserContext ? '--user ' : ''}disable --now`;
        break;
      case 'mask':
        action = `systemctl ${this.configService.settings().systemdUserContext ? '--user' : ''}mask`;
        break;
      case 'unmask':
        action = `systemctl ${this.configService.settings().systemdUserContext ? '--user ' : ''}unmask`;
        break;
      case 'logs':
        action = `journalctl ${this.configService.settings().systemdUserContext ? '--user ' : ''}--no-pager -eu`;
        break;
    }

    const activeService = this.activeService();
    if (!activeService) {
      throw new Error('No active service selected');
    }
    const command = `${action} ${activeService.unit}`;
    let output: any;
    if (!this.configService.settings().systemdUserContext) {
      output = await this.taskManagerService.executeAndWaitBash(`pkexec ${command}`);
    } else {
      output = await this.taskManagerService.executeAndWaitBash(command);
    }

    if (output.code !== 0) {
      this.messageToastService.error(
        this.translocoService.translate('systemdServices.errorTitle'),
        this.translocoService.translate('systemdServices.error', {
          action: event,
        }),
      );
      this.logger.error(`Could execute action ${action}`);
      return;
    }

    this.logger.trace(`Command '${command}' executed successfully`);
    if (event === 'logs') {
      this.taskManagerService.clearTerminal(output.stdout);
      this.taskManagerService.toggleTerminal(true);
    } else {
      this.systemdServices.set(await this.getServices());
    }
  }

  /**
   * Open the popover for the systemd service actions.
   * @param $event The mouse event
   * @param op The popover
   * @param service The systemd service
   */
  openPopover($event: MouseEvent, op: Popover, service: SystemdService): void {
    this.activeService.set(service);
    op.toggle($event);
  }

  /**
   * Toggle the auto-refresh of the systemd services, if enabled start the interval.
   */
  async toggleRefresh(): Promise<void> {
    await this.configService.updateConfig('autoRefresh', !this.configService.settings().autoRefresh);

    if (this.configService.settings().autoRefresh) {
      this.intervalRef = setInterval(async () => {
        this.systemdServices.set(await this.getServices());
      }, 5000);
      this.logger.debug('Started auto-refresh');
    } else if (this.intervalRef) {
      clearInterval(this.intervalRef);
      this.logger.debug('Stopped auto-refresh');
    }
  }

  /**
   * Toggle the context of the systemd services and reload the active services.
   */
  async toggleContext(): Promise<void> {
    this.loading.set(true);
    await this.configService.updateConfig('systemdUserContext', !this.configService.settings().systemdUserContext);
    this.systemdServices.set(await this.getServices());

    this.loading.set(false);
  }

  async toggleDisabled(): Promise<void> {
    this.loading.set(true);
    this.includeDisabled.set(!this.includeDisabled());
    this.systemdServices.set(await this.getServices());

    this.loading.set(false);
  }

  toggleHideDead(): void {
    this.hideDead.set(!this.hideDead());
  }
}
