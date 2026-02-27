import { ChangeDetectionStrategy, Component, effect, inject, input, signal } from '@angular/core';
import type { SystemToolsEntry, SystemToolsSubEntry } from '../../interfaces';
import { Checkbox } from 'primeng/checkbox';
import { TranslocoDirective } from '@jsverse/transloco';
import { FormsModule } from '@angular/forms';
import { NgClass } from '@angular/common';
import { Card } from 'primeng/card';
import { Logger } from '../../logging/logging';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { OsInteractService } from '../task-manager/os-interact.service';

@Component({
  selector: 'toolbox-dynamic-checkboxes',
  imports: [Checkbox, TranslocoDirective, FormsModule, NgClass, Card],
  templateUrl: './dynamic-checkboxes.component.html',
  styleUrl: './dynamic-checkboxes.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DynamicCheckboxesComponent {
  data = input.required<SystemToolsEntry[]>();
  transformed = signal<SystemToolsEntry[]>([]);

  protected readonly taskManagerService = inject(TaskManagerService);
  private readonly logger = Logger.getInstance();
  private readonly osInteractService = inject(OsInteractService);

  constructor() {
    effect(() => {
      this.refreshUi();
    });
  }

  /**
   * Refresh the UI based on the current state of the system.
   */
  refreshUi(): void {
    const data: SystemToolsEntry[] = structuredClone(this.data());
    for (const service of data) {
      this.logger.trace(`Checking ${service.name}`);

      for (const entry of service.sections) {
        if (this.checkState(entry)) {
          entry.checked = true;
        }
      }
    }

    this.checkDisabled(data);

    this.transformed.set(data);
  }

  /**
   * Handle the click action on a checkbox.
   * @param entry The entry to toggle
   */
  async clickAction(entry: SystemToolsSubEntry): Promise<void> {
    if (entry.disabled) {
      return;
    }
    this.osInteractService.toggle(entry.check.name, entry.check.type);
  }

  /**
   * Check the state of the entry, as defined by the check object.
   * @param entry The entry to check
   * @param current Whether to check the current state or the installed state
   * @returns Whether the entry is currently active in the system
   */
  private checkState(entry: SystemToolsSubEntry, current = false): boolean {
    this.logger.debug(`checkState for ${entry.check.name}, type=${entry.check.type}, current=${current}`);
    switch (entry.check.type) {
      case 'pkg': {
        this.logger.debug(`Checking package ${entry.check.name} as pkg`);
        return (
          this.osInteractService.check(entry.check.name, entry.check.type, current) ||
          this.osInteractService.check(`${entry.check.name}-git`, entry.check.type, current)
        );
      }
      default: {
        this.logger.debug(`Checking ${entry.check.name} as ${entry.check.type}`);
        return this.osInteractService.check(entry.check.name, entry.check.type, current);
      }
    }
  }

  /**
   * Check if the entry should be disabled based on the disabler.
   */
  private checkDisabled(entries: SystemToolsEntry[]): void {
    for (const section of entries) {
      for (const entry of section.sections) {
        if (entry.disabler === undefined) continue;

        let disabler: SystemToolsSubEntry | undefined;
        for (const section of entries) {
          if (typeof entry.disabler === 'string') {
            disabler = section.sections.find((e: SystemToolsSubEntry) => e.name === entry.disabler);
          } else {
            for (const disablerName of entry.disabler) {
              disabler = section.sections.find((e: SystemToolsSubEntry) => e.name === disablerName);
              if (disabler) break;
            }
          }
          if (disabler) break;
        }

        let disabled = false;

        if (!disabler) {
          disabled = true;
        } else {
          disabled = !disabler.checked;
        }

        if (disabled && !this.checkState(entry, true)) {
          this.osInteractService.toggle(entry.check.name, entry.check.type, true);
          entry.disabled = true;
        } else entry.disabled = false;
      }
    }
  }
}
