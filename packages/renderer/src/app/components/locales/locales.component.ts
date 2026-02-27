import { ChangeDetectionStrategy, Component, inject, OnInit, signal } from '@angular/core';
import { PickList, PickListMoveToSourceEvent, PickListMoveToTargetEvent } from 'primeng/picklist';
import { TranslocoDirective } from '@jsverse/transloco';
import { LocalesService } from './locales.service';
import { LocalePipe } from '../lang-pipe/locale.pipe';
import { Logger } from '../../logging/logging';
import { Select } from 'primeng/select';
import { FormsModule } from '@angular/forms';
import { ConfigService } from '../config/config.service';
import { Task, TaskManagerService } from '../task-manager/task-manager.service';

@Component({
  selector: 'toolbox-locales',
  imports: [PickList, TranslocoDirective, LocalePipe, Select, FormsModule],
  templateUrl: './locales.component.html',
  styleUrl: './locales.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class LocalesComponent implements OnInit {
  selectedLocale = signal<string>('');

  protected readonly localesService = inject(LocalesService);
  private readonly configService = inject(ConfigService);
  private readonly logger = Logger.getInstance();
  private readonly taskManagerService = inject(TaskManagerService);

  async ngOnInit() {
    this.selectedLocale.set(this.configService.state().locale);

    while (this.localesService.loading()) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }

  /**
   * Toggles the selected state of a package.
   * @param $event The event from the picklist component.
   */
  async toggleOne($event: PickListMoveToSourceEvent | PickListMoveToTargetEvent) {
    await this.localesService.toggleOne($event.items);
  }

  /**
   * Toggles the selected state of all packages.
   */
  async toggleAll() {
    await this.localesService.toggleAll();
  }

  /**
   * Schedule a task to change the locale, if the selected locale is different from the current one.
   * @param $event The event from the select component.
   */
  selectLocale($event: { value: string }) {
    if ($event.value !== this.configService.state().locale) {
      this.logger.trace(`Setting locale to ${$event.value}`);

      const task: Task = this.taskManagerService.createTask(
        10,
        'change-locale',
        true,
        `locales.task`,
        'pi pi-language',
        `localectl set-locale ${$event.value}`,
      );
      this.taskManagerService.scheduleTask(task);
    }
  }
}
