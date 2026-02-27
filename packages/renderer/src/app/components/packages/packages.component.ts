import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  effect,
  inject,
  signal,
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
import { NgClass } from '@angular/common';
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
    NgClass,
  ],
  templateUrl: './packages.component.html',
  styleUrl: './packages.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PackagesComponent {
  tabIndex = signal<number>(0);
  packageSearch = signal<string>('');

  @ViewChild('packageTable') table!: Table;

  protected readonly configService = inject(ConfigService);
  protected readonly packagesService = inject(PackagesService);
  private readonly cdr = inject(ChangeDetectorRef);
  private readonly logger = Logger.getInstance();
  private readonly osInteractService = inject(OsInteractService);
  protected readonly shellService = inject(ElectronShellService);

  constructor() {
    effect(() => {
      const _packages: Map<string, boolean> = this.osInteractService.packages();
      const _tabIndex: number = this.tabIndex();
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
    this.packagesService.packages.update((data: PackageSections) => {
      for (const sections of data) {
        for (const pkg of sections.sections) {
          pkg.selected = installedPackages.get(pkg.pkgname[0]) === true;
        }
      }
      return data;
    });

    // Effect runs before ViewChild initializes the table (only available in AfterViewInit)
    // We need to wait for the table to be available before putting the data in it
    while (!this.table) {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    // We do it like this because via two-way binding, the table doesn't update the data
    // Very likely it is not compatible with zoneless change detection yet
    this.table.value = this.packagesService.packages()[this.tabIndex()].sections;
    this.table.totalRecords = this.table.value.length;

    this.cdr.markForCheck();
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
}
