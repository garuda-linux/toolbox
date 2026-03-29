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
  selector: 'toolbox-gaming',
  imports: [TranslocoDirective, TableModule, DataViewModule, Card, TabsModule, Tooltip],
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
      this.tabIndex.set(0);
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
    void this.router.navigate([], { fragment, info: { disableViewTransition: true } });
  }
  /**
   * Get the source path for an icon.
   * @param item The package entry.
   */
  getIconSrc(item: StatefulPackage & { icon?: string; _iconError?: boolean }): string {
    const pkgname = item.pkgname[0]?.replace(/-(bin|git)$/, '');
    const icon = item.icon;

    if (item._iconError) {
      if (icon) {
        if (icon.includes('.')) {
          if (icon.includes('assets/')) return icon.startsWith('.') ? icon : `./${icon.replace(/^\//, '')}`;
          if (icon.startsWith('/')) return `app-icon://${icon}`;
          return `./assets/gamer/${icon}`;
        }
        return `./assets/gamer/${icon}.png`;
      }
      return 'app-icon://unknown';
    }

    if (icon) {
      return `app-icon://${icon}?fallback=false`;
    }

    if (pkgname) {
      return `app-icon://package/${pkgname}`;
    }

    return 'app-icon://unknown';
  }

  handleIconError(event: Event, item: StatefulPackage & { _iconError?: boolean }): void {
    if (!item._iconError) {
      item._iconError = true;
      this.cdr.markForCheck();
    }
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
