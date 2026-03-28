import { SetupWizardService } from './setup-wizard.service';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { Router } from '@angular/router';
import { SetupSoftwareItem } from './interfaces';
import { OsInteractService } from '../task-manager/os-interact.service';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Step, StepItem, StepPanel, StepperModule } from 'primeng/stepper';
import { Checkbox } from 'primeng/checkbox';
import { Button } from 'primeng/button';
import { AccordionModule } from 'primeng/accordion';
import { Card } from 'primeng/card';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';

@Component({
  selector: 'toolbox-setup-wizard',
  imports: [
    CommonModule,
    FormsModule,
    StepperModule,
    StepPanel,
    Step,
    StepItem,
    Checkbox,
    Button,
    AccordionModule,
    Card,
    TranslocoDirective,
  ],
  templateUrl: './setup-wizard.component.html',
  styleUrl: './setup-wizard.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SetupWizardComponent {
  protected readonly setupWizardService = inject(SetupWizardService);
  protected readonly taskManagerService = inject(TaskManagerService);
  protected readonly osInteractService = inject(OsInteractService);
  protected readonly translocoService = inject(TranslocoService);
  private readonly router = inject(Router);
  private readonly cdr = inject(ChangeDetectorRef);

  protected hasApplied = signal<boolean>(false);

  get softwareCategories() {
    return this.setupWizardService.categories();
  }

  get totalSteps() {
    return this.softwareCategories.length + 1;
  }

  getCategoryStepValue(index: number) {
    return index + 1;
  }

  toggleSoftware(item: SetupSoftwareItem) {
    this.setupWizardService.toggleSoftwareItem(item);
    this.cdr.markForCheck();
  }

  isItemSelected(item: SetupSoftwareItem): boolean {
    return this.setupWizardService.isItemSelected(item);
  }

  getIconSrc(item: SetupSoftwareItem): string {
    const pkgname = item.packages[0]?.replace(/-(bin|git)$/, '');
    if (item.icon) {
      return `app-icon://${item.icon}`;
    }
    if (pkgname) {
      return `app-icon://package/${pkgname}`;
    }
    return 'app-icon://unknown';
  }

  async apply() {
    this.hasApplied.set(true);

    if (this.taskManagerService.count() > 0) {
      this.taskManagerService.toggleTerminal(true);
      await this.setupWizardService.applyChanges();
      this.taskManagerService.toggleTerminal(false);
    }

    void this.router.navigate(['/setup-wizard-success']);
  }
}
