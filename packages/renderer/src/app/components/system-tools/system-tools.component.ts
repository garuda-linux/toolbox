import { Component, inject, type OnInit, signal } from '@angular/core';
import { TableModule } from '@openng/optimus-ui/table';
import { SystemdServicesComponent } from '../systemd-services/systemd-services.component';
import { Tab, TabList, TabPanel, TabPanels, Tabs } from '@openng/optimus-ui/tabs';
import { TranslocoDirective } from '@jsverse/transloco';
import { SystemSettingsComponent } from '../system-settings/system-settings.component';
import { SystemComponentsComponent } from '../system-components/system-components.component';
import { KernelsComponent } from '../kernels/kernels.component';
import { LanguagePacksComponent } from '../language-packs/language-packs.component';
import { LocalesComponent } from '../locales/locales.component';
import { UserManagerComponent } from '../user-manager/user-manager.component';
import { Router, type UrlTree } from '@angular/router';

@Component({
  selector: 'toolbox-system-tools',
  imports: [
    TableModule,
    SystemdServicesComponent,
    Tab,
    TabList,
    TabPanel,
    TabPanels,
    Tabs,
    TranslocoDirective,
    SystemSettingsComponent,
    SystemComponentsComponent,
    KernelsComponent,
    LanguagePacksComponent,
    LocalesComponent,
    UserManagerComponent,
  ],
  templateUrl: './system-tools.component.html',
  styleUrl: './system-tools.component.css',
})
export class SystemToolsComponent implements OnInit {
  tabIndex = signal<number>(0);

  private readonly router = inject(Router);

  ngOnInit(): void {
    this.checkRoute();

    this.router.events.subscribe(() => {
      const currentUrl: UrlTree = this.router.parseUrl(this.router.url);
      if (currentUrl.fragment) {
        this.navigateToTab(currentUrl.fragment);
      }
    });
  }

  /**
   * Check the current route and set the tab index accordingly on initial load.
   */
  private checkRoute(): void {
    const url: UrlTree = this.router.parseUrl(this.router.url);
    this.navigateToTab(url.fragment ?? '');
  }

  /**
   * Navigate to a specific tab based on the fragment in the URL.
   * @param fragment The fragment to navigate to.
   */
  navigateToTab(fragment: string) {
    switch (fragment) {
      case 'components':
        this.tabIndex.set(0);
        break;
      case 'settings':
        this.tabIndex.set(1);
        break;
      case 'kernels':
        this.tabIndex.set(2);
        break;
      case 'locales':
        this.tabIndex.set(3);
        break;
      case 'services':
        this.tabIndex.set(4);
        break;
      case 'users':
      case 'groups':
        this.tabIndex.set(5);
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
}
