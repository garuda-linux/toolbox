import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  effect,
  inject,
  type OnInit,
  signal,
} from '@angular/core';
import { TranslocoDirective } from '@jsverse/transloco';
import { TableModule } from 'primeng/table';
import { NgOptimizedImage } from '@angular/common';
import { DataViewModule } from 'primeng/dataview';
import { Card } from 'primeng/card';
import { TabsModule } from 'primeng/tabs';
import { Tooltip } from 'primeng/tooltip';
import { ConfigService } from '../config/config.service';
import type { StatefulPackage } from './interfaces';
import { OsInteractService } from '../task-manager/os-interact.service';
import { GamingService } from './gaming.service';
import { Router, type UrlTree } from '@angular/router';

@Component({
  selector: 'rani-gaming',
  imports: [TranslocoDirective, TableModule, DataViewModule, Card, NgOptimizedImage, TabsModule, Tooltip],
  templateUrl: './gaming.component.html',
  styleUrl: './gaming.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class GamingComponent implements OnInit {
  computedBackground = getComputedStyle(document.documentElement).getPropertyValue('--p-card-subtitle-color');
  tabIndex = signal<number>(0);

  protected readonly configService = inject(ConfigService);
  protected readonly gamingService = inject(GamingService);
  protected readonly osInteractService = inject(OsInteractService);
  protected readonly open = open;

  private readonly cdr = inject(ChangeDetectorRef);
  private readonly router = inject(Router);

  constructor() {
    effect(() => {
      const installedPackages: Map<string, boolean> = this.osInteractService.packages();
      this.gamingService.packages.update((packages) => {
        for (const sections of packages) {
          for (const pkg of sections.sections) {
            pkg.selected = installedPackages.get(pkg.pkgname[0]) === true;
          }
        }
        return packages;
      });

      this.cdr.markForCheck();
    });
  }

  ngOnInit(): void {
    const url: UrlTree = this.router.parseUrl(this.router.url);
    if (!url.fragment) {
      void this.router.navigate([], { fragment: 'launchers' });
      return;
    }

    switch (url.fragment) {
      case 'launchers':
        this.tabIndex.set(0);
        break;
      case 'wine':
        this.tabIndex.set(1);
        break;
      case 'tools':
        this.tabIndex.set(2);
        break;
      case 'misc':
        this.tabIndex.set(3);
        break;
      case 'controllers':
        this.tabIndex.set(4);
        break;
      case 'games':
        this.tabIndex.set(5);
        break;
      case 'emulators':
        this.tabIndex.set(6);
        break;
      default:
        this.tabIndex.set(0);
    }
  }

  /**
   * Set the fragment in the URL.
   * @param fragment The fragment to navigate to.
   */
  navigate(fragment: string) {
    void this.router.navigate([], { fragment });
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
        return `./assets/gamer/${icon}`;
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
   * Toggles the selected state of a package.
   * @param item The package to toggle.
   */
  togglePackage(item: StatefulPackage): void {
    for (const pkgname of item.pkgname) {
      this.osInteractService.togglePackage(pkgname);
    }
  }
}
