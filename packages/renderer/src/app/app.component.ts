import type { OnInit } from '@angular/core';
import {
  ChangeDetectionStrategy,
  Component,
  computed,
  ElementRef,
  HostListener,
  inject,
  Renderer2,
  signal,
  viewChild,
} from '@angular/core';
import { NavigationCancel, NavigationError, NavigationStart, Router, RouterModule } from '@angular/router';
import { ScrollTop } from 'primeng/scrolltop';
import { TranslocoDirective, TranslocoService } from '@jsverse/transloco';
import { eventsOn, windowClose, windowRequestClose } from './electron-services/electron-api-utils.js';
import { DialogModule } from 'primeng/dialog';
import { DrawerModule } from 'primeng/drawer';
import { TableModule } from 'primeng/table';
import { ToastModule } from 'primeng/toast';
import { FormsModule } from '@angular/forms';
import type { MenuItem } from 'primeng/api';
import { ConfirmationService } from 'primeng/api';
import { globalKeyHandler } from './key-handler';
import { ProgressSpinner } from 'primeng/progressspinner';
import { LoadingService } from './components/loading-indicator/loading-indicator.service';
import { TerminalComponent } from './components/terminal/terminal.component';
import { OperationManagerComponent } from './components/operation-manager/operation-manager.component';
import { ConfirmDialog } from 'primeng/confirmdialog';
import { ConfigService } from './components/config/config.service';
import { Logger } from './logging/logging';
import { TaskManagerService } from './components/task-manager/task-manager.service';
import { NotificationService } from './components/notification/notification.service';
import { ThemeService } from './components/theme-service/theme-service';
import { type AppMenuItem, ElectronAppMenuService, ElectronOsService, ElectronShellService } from './electron-services';
import { SplitButton } from 'primeng/splitbutton';
import { AutoComplete, AutoCompleteCompleteEvent, AutoCompleteSelectEvent } from 'primeng/autocomplete';
import { MODULE_SEARCH, ModuleSearchEntry } from './constants/module-search';
import { NgClass } from '@angular/common';
import { AppDesigner } from './components/designer/app.designer';
import { DesignerService } from './components/designer/designerservice';
import { WallpaperService } from './components/wallpaper/wallpaper.service';
import { ContextMenuModule } from 'primeng/contextmenu';
import { Menu } from 'primeng/menu';
import { ShellBarEndDirective, ShellComponent } from '@garudalinux/core';

@Component({
  imports: [
    RouterModule,
    DialogModule,
    ScrollTop,
    ShellComponent,
    DrawerModule,
    TableModule,
    TranslocoDirective,
    ToastModule,
    FormsModule,
    ShellComponent,
    ProgressSpinner,
    ShellBarEndDirective,
    TerminalComponent,
    OperationManagerComponent,
    ConfirmDialog,
    SplitButton,
    AutoComplete,
    NgClass,
    ContextMenuModule,
    AppDesigner,
    Menu,
  ],
  selector: 'toolbox-root',
  templateUrl: './app.component.html',
  styleUrl: './app.component.css',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class AppComponent implements OnInit {
  readonly terminalComponent = viewChild.required<TerminalComponent>('terminalComponent');
  readonly operationManagerComponent = viewChild.required<OperationManagerComponent>('operationManagerComponent');

  private readonly appMenuService = inject(ElectronAppMenuService);
  private readonly elementRef = inject(ElementRef);
  private readonly notificationService = inject(NotificationService);
  private readonly renderer = inject(Renderer2);
  private readonly router = inject(Router);
  private readonly translocoService = inject(TranslocoService);
  private readonly wallpaperService = inject(WallpaperService);

  protected readonly confirmationService = inject(ConfirmationService);
  protected readonly loadingService = inject(LoadingService);
  protected readonly osService = inject(ElectronOsService);
  protected readonly shellService = inject(ElectronShellService);
  protected readonly taskManager = inject(TaskManagerService);

  applyButtonVisible = computed(() => this.taskManager.tasks().length > 0);

  rightClickMenu = signal<MenuItem[]>([
    {
      id: 'apply',
      label: 'Apply',
      icon: 'pi pi-check',
      visible: this.applyButtonVisible(),
      command: () => this.operationManagerComponent().applyOperations(),
    },
    {
      id: 'clear',
      label: 'Clear',
      icon: 'pi pi-trash',
      visible: this.applyButtonVisible(),
      command: () => this.operationManagerComponent().clearOperations(),
    },
    {
      id: 'show-terminal',
      label: 'Show terminal',
      icon: 'pi pi-hashtag',
      command: () => this.terminalComponent().visible.set(true),
    },
    {
      id: 'exit',
      label: 'Exit',
      icon: 'pi pi-times',
      command: () => windowRequestClose(),
    },
  ]);

  protected readonly configService = inject(ConfigService);
  protected readonly logger = Logger.getInstance();

  // Not used, but required for the app component to work properly!
  private readonly _designerService = inject(DesignerService);
  private readonly _themeService = inject(ThemeService);

  readonly moduleItems = [
    {
      id: 'welcome',
      icon: 'pi pi-home',
      label: 'Welcome',
      translocoKey: 'menu.welcome',
      command: () => this.router.navigate(['/']),
    },
    {
      id: 'maintenance',
      icon: 'pi pi-desktop',
      label: 'Maintenance',
      translocoKey: 'menu.maintenance',
      command: () => this.router.navigate(['/maintenance']),
    },
    {
      id: 'system-tools',
      icon: 'pi pi-microchip',
      label: 'System tools',
      translocoKey: 'menu.systemTools',
      command: () => this.router.navigate(['/system-tools']),
    },
    {
      id: 'gaming',
      icon: 'pi pi-play-circle',
      label: 'Gaming apps',
      translocoKey: 'menu.gaming',
      command: () => this.router.navigate(['/gaming']),
    },
    {
      id: 'packages',
      icon: 'pi pi-box',
      label: 'Discover applications',
      translocoKey: 'menu.packages',
      command: () => this.router.navigate(['/packages']),
    },
    {
      id: 'boot-tools',
      icon: 'pi pi-hammer',
      label: 'Boot Options',
      translocoKey: 'menu.boot',
      command: () => this.router.navigate(['/boot-options']),
      visible: true,
    },
    {
      id: 'boot-repair',
      icon: 'pi pi-hammer',
      label: 'Boot Repair',
      translocoKey: 'menu.bootRepair',
      command: () => this.router.navigate(['/boot-repair']),
      visible: true,
    },
    {
      id: 'network-tools',
      icon: 'pi pi-globe',
      label: 'Network Assistant',
      translocoKey: 'menu.network',
      command: () => this.router.navigate(['/network-assistant']),
      visible: true,
    },
    {
      id: 'config-files',
      icon: 'pi pi-file-edit',
      label: 'Config Files',
      translocoKey: 'menu.configFiles',
      command: () => this.router.navigate(['/config-files']),
    },
    {
      id: 'diagnostics',
      icon: 'pi pi-info-circle',
      label: 'Diagnostics',
      translocoKey: 'menu.diagnostics',
      command: () => this.router.navigate(['/diagnostics']),
    },
  ];
  helpItems = [
    {
      id: 'help-forum',
      icon: 'pi pi-info-circle',
      label: 'Get help on the forum',
      translocoKey: 'menu.help.getHelpForum',
      command: () => this.shellService.open('https://forum.garudalinux.org/'),
    },
    {
      id: 'help-wiki',
      icon: 'pi pi-info-circle',
      label: 'Search the wiki',
      translocoKey: 'menu.help.getHelpWiki',
      command: () => this.shellService.open('https://wiki.garudalinux.org/'),
    },
    {
      id: 'help-arch-wiki',
      icon: 'pi pi-info-circle',
      label: 'Search the Arch wiki',
      translocoKey: 'menu.help.getHelpArchWiki',
      command: () => this.shellService.open('https://wiki.archlinux.org/'),
    },
    {
      id: 'help-status',
      icon: 'pi pi-info-circle',
      label: 'Garuda Linux infra status',
      translocoKey: 'menu.help.garudaStatus',
      command: () => this.shellService.open('https://status.garudalinux.org'),
    },
    {
      id: 'help-exorcist',
      icon: 'pi pi-info-circle',
      label: 'About',
      translocoKey: 'menu.help.callExorcist',
      command: () =>
        this.notificationService.sendNotification({
          title: this.translocoService.translate('menu.help.callExorcistTitle'),
          body: `${this.translocoService.translate('menu.help.callExorcistBody')} 🐛`,
        }),
    } as MenuItem,
    {
      id: 'help-about',
      icon: 'pi pi-info-circle',
      label: 'About',
      translocoKey: 'menu.help.about',
      command: () =>
        this.notificationService.sendNotification({
          title: this.translocoService.translate('menu.help.about'),
          body: this.translocoService.translate('menu.help.aboutBody'),
        }),
    },
  ];
  private fileItems = [
    {
      id: 'file-preferences',
      icon: 'pi pi-cog',
      label: 'Preferences',
      translocoKey: 'menu.file.preferences',
      command: () => this.router.navigate(['/settings']),
    },
    {
      id: 'file-themes',
      icon: 'pi pi-palette',
      label: 'Theme Designer',
      translocoKey: 'menu.file.themeDesigner',
      command: () => this.configService.updateState('designerActive', true),
    },
    {
      id: 'file-quit',
      icon: 'pi pi-sign-out',
      label: 'Quit',
      translocoKey: 'menu.file.quit',
      command: () => {
        void this.shutdown();
      },
    },
  ];

  menuItems = signal<MenuItem[]>(
    this.setupLabels(this.translocoService.getActiveLang(), [
      {
        id: 'file',
        icon: 'pi pi-file',
        label: 'File',
        translocoKey: 'menu.file.title',
        items: [...this.fileItems],
      },
      {
        id: 'modules',
        icon: 'pi pi-cog',
        label: 'Modules',
        translocoKey: 'menu.modules.title',
        items: [...this.moduleItems],
      },
      {
        id: 'terminal',
        icon: 'pi pi-spinner',
        label: 'Terminal',
        translocoKey: 'menu.terminal',
        command: () => this.terminalComponent().visible.set(true),
      },
      {
        id: 'help',
        icon: 'pi pi-question-circle',
        label: 'Help',
        translocoKey: 'menu.help.title',
        items: [...this.helpItems],
      },
    ]),
  );

  /**
   * Menu items for the apply menu in case tasks are pending.
   * @protected
   */
  protected readonly applyMenuItems = [
    {
      label: this.translocoService.translate('menu.apply'),
      command: () => this.taskManager.toggleTerminal(true),
    },
    {
      label: this.translocoService.translate('menu.clearTasks'),
      command: () => this.taskManager.clearTasks(),
    },
  ];

  /**
   * App module search stuff.
   */
  selectedModule = signal<string>('');
  moduleSuggestions = signal<ModuleSearchEntry[]>([]);
  dropdownModel = signal<MenuItem[]>([]);
  dropdownVisible = signal<boolean>(false);
  dropdownTop = signal<number>(0);
  dropdownLeft = signal<number>(0);
  activeDropdownId = signal<string | undefined>(undefined);

  readonly menubarModel = computed<MenuItem[]>(() => {
    return this.menuItems().map((item) => {
      if (item.items && item.items.length > 0) {
        return {
          ...item,
          icon: item.icon,
          items: undefined,
          command: (event: { originalEvent: Event }) => this.toggleDropdown(event.originalEvent, item),
          styleClass: 'garuda-menubar-item--has-submenu',
        };
      }
      return {
        ...item,
        items: undefined,
      };
    });
  });

  private toggleDropdown(event: Event, item: MenuItem): void {
    const isSameItem = this.activeDropdownId() === item.id;

    if (isSameItem && this.dropdownVisible()) {
      this.dropdownVisible.set(false);
      this.activeDropdownId.set(undefined);
      return;
    }

    const target = (event.currentTarget || event.target) as HTMLElement;
    const anchor = target.closest('.p-menubar-item') || target;
    const rect = anchor.getBoundingClientRect();

    this.dropdownModel.set(item.items ?? []);
    this.dropdownTop.set(rect.bottom + 8);
    this.dropdownLeft.set(rect.left);
    this.activeDropdownId.set(item.id);
    this.dropdownVisible.set(true);
  }

  ngOnInit(): void {
    this.router.events.subscribe((event) => {
      if (event instanceof NavigationStart) {
        document.body.classList.add('is-transitioning');
      } else if (event instanceof NavigationCancel || event instanceof NavigationError) {
        document.body.classList.remove('is-transitioning');
      }
    });

    void this.osService.argv().then((args: string[]) => {
      if (args.includes('--setup-assistant')) {
        this.logger.info('Redirecting to setup wizard on startup');
        void this.router.navigate(['/setup-wizard'], { replaceUrl: true });
      }
    });

    window.addEventListener('unhandledrejection', (event) => {
      this.logger.error(`Unhandled promise rejection: ${event.reason}`);

      if (event.reason instanceof SyntaxError && event.reason.message.includes('JSON')) {
        this.logger.error(`JSON parsing error detected: ${event.reason.message}`);
        event.preventDefault();
      }
      if (event.reason && typeof event.reason === 'string' && event.reason.includes('HTTP')) {
        this.logger.error(`HTTP error detected: ${event.reason}`);
        event.preventDefault();
      }
    });

    // Handle global errors
    window.addEventListener('error', (event) => {
      this.logger.error(`Global error: ${event.error}`);

      if (event.error instanceof SyntaxError) {
        this.logger.error(`Syntax error (possibly JSON): ${event.error.message}`);
      }
    });

    this.translocoService.events$.subscribe((event) => {
      if (event.type === 'langChanged') {
        this.logger.trace('Updating menu labels via event');
        this.menuItems.update((items: MenuItem[]) => {
          return this.setupLabels(this.translocoService.getActiveLang(), items);
        });
      }
    });

    this.attachElectronListeners();

    this.setupAppMenuHandlers();
    void this.updateApplicationMenu(this.menuItems());
    this.wallpaperService.initialize(this.renderer, this.elementRef);

    this.renderer.listen('window', 'click', (e: Event) => {
      if (this.dropdownVisible() && !(e.target as HTMLElement).closest('.p-menubar-item')) {
        this.dropdownVisible.set(false);
        this.activeDropdownId.set(undefined);
      }
    });
  }

  /**
   * Handle all relevant keyboard events on the app window. Attaches to the document.
   * @param event The keyboard event
   */
  @HostListener('document:keydown', ['$event'])
  async handleKeyboardEvent(event: KeyboardEvent): Promise<void> {
    const thisBoundKeyHandler = globalKeyHandler.bind(this);
    await thisBoundKeyHandler(event);
  }

  /**
   * Updates the application menu with current menu items
   * @param _items The current menu items
   */
  private async updateApplicationMenu(_items: MenuItem[]): Promise<void> {
    const menubar = this.setupLabels(this.translocoService.getActiveLang(), [
      {
        id: 'file',
        icon: 'pi pi-file',
        label: 'File',
        translocoKey: 'menu.file.title',
        items: [...this.fileItems],
      },
      {
        id: 'modules',
        icon: 'pi pi-cog',
        label: 'Modules',
        translocoKey: 'menu.modules.title',
        items: [...this.moduleItems],
      },
      {
        id: 'help',
        icon: 'pi pi-question-circle',
        label: 'Help',
        translocoKey: 'menu.help.title',
        items: [...this.helpItems],
      },
    ]);

    try {
      const appMenuItems: AppMenuItem[] = this.convertToAppMenuItems(menubar);
      await this.appMenuService.updateAppMenu(appMenuItems);
    } catch (error) {
      this.logger.error(`Failed to update application menu: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  /**
   * Converts PrimeNG MenuItem[] to AppMenuItem[] for the app menu
   * @param items PrimeNG menu items
   * @returns Converted app menu items
   */
  private convertToAppMenuItems(items: MenuItem[]): AppMenuItem[] {
    const dontShow = new Set(['terminal']);

    return items
      .filter((item) => item.visible !== false && !dontShow.has(item.id ?? ''))
      .map((item) => ({
        id: item.id,
        label: item.label,
        icon: item.icon,
        enabled: !item.disabled,
        visible: item.visible !== false,
        routerLink: item.routerLink as string,
        command: item.command ? item.id : undefined,
        submenu: item.items ? this.convertToAppMenuItems(item.items) : undefined,
        accelerator: (item as any).shortcut as string,
      }));
  }

  /**
   * Sets up handlers for app menu item clicks
   */
  private setupAppMenuHandlers(): void {
    try {
      eventsOn('appMenu:itemClicked', (...args: unknown[]) => {
        const data = args[0] as any;
        this.logger.trace(`App menu item clicked event received: ${JSON.stringify(data)}`);

        const menuData = data;

        if (menuData.routerLink) {
          try {
            void this.router.navigate([menuData.routerLink]);
          } catch (error) {
            this.logger.error(`Navigation error: ${error instanceof Error ? error.message : String(error)}`);
          }
        }

        if (menuData.command) {
          try {
            this.handleMenuCommand(menuData.command);
          } catch (error) {
            this.logger.error(
              `Error handling menu command: ${menuData.command}, Error: ${
                error instanceof Error ? error.message : String(error)
              }`,
            );
          }
        }
      });
    } catch (error) {
      this.logger.error(`Setup app menu handlers error: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  private handleMenuCommand(command: string): void {
    this.logger.debug(`Handling menu command: ${command}`);

    switch (command) {
      case 'shutdown':
        void this.shutdown();
        break;
      default: {
        const menuItem: MenuItem | undefined = this.findMenuItemById(this.menuItems(), command);
        if (menuItem) {
          this.logger.debug(`Executing command for menu item: ${menuItem.label}`);
          if (menuItem.command) {
            // @ts-expect-error - irrelevant, we don't care about the arguments here
            menuItem.command();
          } else {
            this.logger.warn(`Menu item with command not found: ${command}`);
          }
          return;
        }
        this.logger.warn(`Unknown menu command: ${command}`);
        break;
      }
    }
  }

  /**
   * Finds a menu item by ID recursively
   * @param items Menu items to search
   * @param id ID to find
   * @returns Found menu item or undefined
   */
  private findMenuItemById(items: MenuItem[], id: string): MenuItem | undefined {
    for (const item of items) {
      if (item.id === id) {
        return item;
      }
      if (item.items) {
        const found = this.findMenuItemById(item.items, id);
        if (found) {
          return found;
        }
      }
    }
    return undefined;
  }

  /**
   * Sets up the labels for the menu items, based on the current language
   * @param lang The language to set the labels to
   * @param entries The menu items to set the labels for
   * @returns The menu items with the labels set
   */
  setupLabels(lang: string, entries: MenuItem[]): MenuItem[] {
    const newItems: MenuItem[] = [];

    for (const item of entries) {
      const newSubItems: MenuItem[] = [];
      if (item.items) {
        for (const subItem of item.items) {
          newSubItems.push(
            Object.assign({}, subItem, {
              label: this.translocoService.translate(
                (subItem as MenuItem & { translocoKey: string }).translocoKey,
                {},
                lang,
              ),
            }),
          );
        }
      }
      newItems.push(
        Object.assign({}, item, {
          label: this.translocoService.translate((item as MenuItem & { translocoKey: string }).translocoKey, {}, lang),
          items: newSubItems,
        }),
      );
    }

    return newItems;
  }

  /**
   * Shutdown the app, saving the settings and closing the window.
   * @private
   */
  private async shutdown(): Promise<void> {
    this.logger.info('Shutting down');
    windowClose();
  }

  /**
   * Attach the Electron listeners for the app window.
   * @private
   */
  private attachElectronListeners() {
    this.logger.trace('Electron window listeners initialized');

    // Handle close events through beforeunload if needed
    window.addEventListener('beforeunload', (event) => {
      if (this.taskManager.running() || this.taskManager.count()) {
        event.preventDefault();
        void this.requestShutdown();
      }
    });
  }

  /**
   * Request a shutdown of the app. If there are any tasks running, ask the user for confirmation.
   * @protected
   */
  protected async requestShutdown() {
    this.logger.debug(`Close requested, ${this.taskManager.currentTask() ? 'one' : 'no'} action is running`);

    if (!this.taskManager.running() && !this.taskManager.count()) {
      void this.shutdown();
      return;
    }

    this.confirmationService.confirm({
      message: this.taskManager.running()
        ? this.translocoService.translate('confirmation.exitAppRunningAction')
        : this.translocoService.translate('confirmation.exitApp'),
      header: this.translocoService.translate('confirmation.exitAppHeader'),
      icon: 'pi pi-exclamation-triangle',
      acceptIcon: 'pi pi-check',
      rejectIcon: 'pi pi-times',
      acceptButtonProps: {
        severity: this.taskManager.running() ? 'danger' : 'success',
        label: this.translocoService.translate('confirmation.accept'),
      },
      rejectButtonProps: {
        severity: 'secondary',
        label: this.translocoService.translate('confirmation.reject'),
      },
      accept: () => {
        void this.shutdown();
      },
    });
  }

  /**
   * Filter module suggestions based on the user's input.
   * @param $event The autocomplete event containing the query
   */
  filterModuleSuggestions($event: AutoCompleteCompleteEvent) {
    const filtered: ModuleSearchEntry[] = [];
    for (const entry of MODULE_SEARCH) {
      if (entry.keywords.some((entry) => entry.match($event.query.toLowerCase()))) {
        console.debug(`Matched module suggestion: ${JSON.stringify(entry)}`);
        filtered.push(entry);
      } else {
        console.debug(`No match for module suggestion: ${JSON.stringify(entry)}`);
      }
    }
    this.moduleSuggestions.set(filtered);
  }

  /**
   * Handle the selection of a module by navigating to its route.
   * @param $event The select event containing the selected module
   */
  selectModule($event: AutoCompleteSelectEvent) {
    this.logger.debug(`Selected module: ${$event.value}`);
    const module = $event.value as ModuleSearchEntry;

    this.logger.debug(`Navigating to module: ${module.moduleName} (${module.routerLink}) with hash ${module.hash}`);
    this.selectedModule.set('');
    void this.router.navigate([module.routerLink], { fragment: module.hash });
  }
}
