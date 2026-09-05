import { SetupWizardService } from './setup-wizard.service';
import { TaskManagerService } from '../task-manager/task-manager.service';
import { Router } from '@angular/router';
import { SetupSoftwareItem } from './interfaces';
import { OsInteractService } from '../task-manager/os-interact.service';
import { ChangeDetectorRef, Component, ElementRef, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Tab, TabList, TabPanel, TabPanels, Tabs } from '@openng/optimus-ui/tabs';
import { Checkbox } from '@openng/optimus-ui/checkbox';
import { AccordionModule } from '@openng/optimus-ui/accordion';
import { Card } from '@openng/optimus-ui/card';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { ButtonDirective } from '@openng/optimus-ui/button';

@Component({
  selector: 'toolbox-setup-wizard',
  imports: [
    CommonModule,
    FormsModule,
    Tabs,
    TabList,
    Tab,
    TabPanels,
    TabPanel,
    Checkbox,
    ButtonDirective,
    AccordionModule,
    Card,
    TranslocoDirective,
  ],
  templateUrl: './setup-wizard.component.html',
  styleUrl: './setup-wizard.component.css',
})
export class SetupWizardComponent {
  protected readonly setupWizardService = inject(SetupWizardService);
  protected readonly taskManagerService = inject(TaskManagerService);
  protected readonly osInteractService = inject(OsInteractService);
  protected readonly translocoService = inject(TranslocoService);
  private readonly router = inject(Router);
  private readonly cdr = inject(ChangeDetectorRef);
  private readonly elementRef = inject(ElementRef);

  protected readonly hasApplied = signal<boolean>(false);
  protected readonly executionFailed = signal<boolean>(false);
  protected readonly activeTab = signal<number>(1);

  get softwareCategories() {
    return this.setupWizardService.categories();
  }

  getCategoryStepValue(index: number) {
    return index + 1;
  }

  onTabChange(value: string | number | undefined) {
    this.activeTab.set(Number(value));
    this.scrollToTop();
  }

  goToStep(value: number) {
    this.activeTab.set(value);
    this.scrollToTop();
  }

  scrollToTop() {
    requestAnimationFrame(() => {
      window.scrollTo(0, 0);
      document.documentElement.scrollTop = 0;
      document.body.scrollTop = 0;

      let el: Element | null = this.elementRef.nativeElement;
      while (el) {
        if (el instanceof HTMLElement && el.scrollTop > 0) {
          el.scrollTop = 0;
        }
        el = el.parentElement;
      }
    });
  }

  toggleSoftware(item: SetupSoftwareItem) {
    this.setupWizardService.toggleSoftwareItem(item);
    this.executionFailed.set(false);
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
    this.executionFailed.set(false);

    const hasTasks = this.taskManagerService.count() > 0;
    if (hasTasks) {
      this.taskManagerService.toggleTerminal(true);
      const success = await this.setupWizardService.applyChanges();
      if (success) {
        this.taskManagerService.toggleTerminal(false);
        void this.router.navigate(['/setup-wizard-success']);
      } else {
        this.executionFailed.set(true);
        this.cdr.markForCheck();
      }
    } else {
      void this.router.navigate(['/setup-wizard-success']);
    }
  }
}
