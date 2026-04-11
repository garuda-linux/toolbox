import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { ConfirmationService } from 'primeng/api';
import { MessageToastService } from '@garudalinux/core';
import { TranslocoService } from '@jsverse/transloco';
import { Logger } from '../../logging/logging';
import { TaskManagerService } from '../task-manager/task-manager.service';

@Component({
  selector: 'toolbox-operation-manager',
  imports: [],
  templateUrl: './operation-manager.component.html',
  styleUrl: './operation-manager.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class OperationManagerComponent {
  private readonly confirmationService = inject(ConfirmationService);
  private readonly logger = Logger.getInstance();
  private readonly messageToastService = inject(MessageToastService);
  private readonly taskManagerService = inject(TaskManagerService);
  private readonly translocoService = inject(TranslocoService);

  /**
   * Apply all pending operations, if any. Shows a confirmation dialog before applying. If the user cancels, a message is shown.
   */
  applyOperations() {
    this.logger.debug('Firing apply operations');
    this.confirmationService.confirm({
      message: this.translocoService.translate('confirmation.applyOperationsBody'),
      header: this.translocoService.translate('confirmation.applyOperations'),
      icon: 'pi pi-info-circle',
      rejectLabel: 'Cancel',
      acceptButtonProps: {
        severity: 'success',
        label: this.translocoService.translate('confirmation.accept'),
      },
      rejectButtonProps: {
        severity: 'secondary',
        label: this.translocoService.translate('confirmation.reject'),
      },

      accept: () => {
        this.logger.debug('Firing apply operations');
        void this.taskManagerService.executeTasks();
      },
      reject: () => {
        this.logger.debug('Rejected applying operations');
        this.messageToastService.error('Rejected', 'You have rejected');
      },
    });
  }

  /**
   * Apply all pending operations immediately without confirmation.
   * Used by the "Apply pending operations" dropdown action.
   */
  applyOperationsNow() {
    this.logger.debug('Applying operations immediately');
    void this.taskManagerService.executeTasks();
  }

  /**
   * Clear all pending operations. Shows a confirmation dialog before clearing. If the user cancels, a message is shown.
   */
  clearOperations(): void {
    this.logger.debug('Firing clear operations');
    const operations = this.taskManagerService.count() === 1 ? 'operation' : 'operations';
    this.confirmationService.confirm({
      message: `Do you want to delete ${this.taskManagerService.count()} ${operations}?`,
      header: 'Clear pending operations?',
      icon: 'pi pi-trash',
      rejectLabel: 'Cancel',
      rejectButtonProps: {
        label: 'Cancel',
        severity: 'secondary',
        outlined: true,
      },
      acceptButtonProps: {
        label: 'Delete',
        severity: 'danger',
      },

      accept: () => {
        this.taskManagerService.clearTasks();
        this.messageToastService.info('Confirmed', 'Pending operations cleared');
        this.logger.debug('Cleared pending operations');
      },
      reject: () => {
        this.messageToastService.error('Rejected', 'You have rejected');
        this.logger.debug('Rejected clearing pending operations');
      },
    });
  }
}
