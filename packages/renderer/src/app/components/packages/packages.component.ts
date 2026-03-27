import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  effect,
  inject,
  signal,
  viewChild,
  ViewChild,
} from '@angular/core';
import { OsInteractService } from '../task-manager/os-interact.service';
import type { PackageSections, StatefulPackage } from '../gaming/interfaces';
import { Tab, TabList, Tabs } from 'primeng/tabs';
import { TranslocoDirective } from '@jsverse/transloco';
import { Logger } from '../../logging/logging';
import { Button } from 'primeng/button';
import { ElectronShellService } from '../../electron-services';
import { IconField } from 'primeng/iconfield';
import { InputIcon } from 'primeng/inputicon';
import { InputText } from 'primeng/inputtext';
import { type Table, TableModule } from 'primeng/table';
import { Checkbox } from 'primeng/checkbox';
import { FormsModule } from '@angular/forms';

import { PackagesService } from './packages.service';
import { ConfigService } from '../config/config.service';

@Component({
  selector: 'toolbox-packages',
  imports: [
    Tab,
    TabList,
    Tabs,
    TranslocoDirective,
    Button,
    IconField,
    InputIcon,
    InputText,
    TableModule,
    Checkbox,
    FormsModule,
  ],
  templateUrl: './packages.component.html',
  styleUrl: './packages.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PackagesComponent {
  readonly tabIndex = signal<number>(0);
  readonly packageSearch = signal<string>('');

  readonly table = viewChild.required<Table>('packageTable');

  protected readonly configService = inject(ConfigService);
  protected readonly packagesService = inject(PackagesService);
  private readonly cdr = inject(ChangeDetectorRef);
  private readonly logger = Logger.getInstance();
  private readonly osInteractService = inject(OsInteractService);
  protected readonly shellService = inject(ElectronShellService);

  constructor() {
    effect(() => {
      this.osInteractService.packages();
      if (!this.packagesService.loading()) {
        void this.updateUi();
      }
    });
  }

  /**
   * Get the source path for an icon.
   * @param item The package entry.
   */
  getIconSrc(item: StatefulPackage & { icon: string }): string {
    const pkgname = item.pkgname[0];
    const icon = item.icon;

    if (!icon || icon === 'generic-dark.svg') {
      return this.configService.settings().darkMode
        ? './assets/gamer/generic-dark.svg'
        : './assets/gamer/generic-light.svg';
    }

    if (icon?.startsWith('/')) {
      return `app-icon://${icon}`;
    }

    if (icon) {
      if (icon.includes('.')) {
        if (icon.includes('assets/')) return icon.startsWith('.') ? icon : `./${icon.replace(/^\//, '')}`;
        return `./assets/icons/${icon}`;
      }
      return `app-icon://${icon}`;
    }

    if (pkgname) {
      return `app-icon://package/${pkgname}`;
    }

    return this.configService.settings().darkMode
      ? './assets/gamer/generic-dark.svg'
      : './assets/gamer/generic-light.svg';
  }

  /**
   * Update the state of the UI based on the installed packages.
   */
  async updateUi(): Promise<void> {
    this.logger.trace('Updating packages UI');

    const installedPackages: Map<string, boolean> = this.osInteractService.packages();
    const currentPackages = this.packagesService.packages();

    for (const sections of currentPackages) {
      for (const pkg of sections.sections) {
        pkg.selected = installedPackages.get(pkg.pkgname[0]) === true;
      }
    }
    this.packagesService.packages.set(currentPackages);
  }

  /**
   * Toggles the selected state of a package.
   * @param item The package to toggle.
   */
  togglePackage(item: StatefulPackage): void {
    for (const pkgname of item.pkgname) {
      this.osInteractService.togglePackage(pkgname);
    }
  }

  /**
   * Clear the systemd service table search and options.
   * @param table The table component to clear
   */
  clear(table: any): void {
    table.clear();
    this.packageSearch.set('');
  }

  /**
   * Reset page to 0, because this doesn't happen automatically.
   * @param $event The tab change event
   */
  tabChange($event: string | number | undefined) {
    this.table().first = 0;
  }
}
