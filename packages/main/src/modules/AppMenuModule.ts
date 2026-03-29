import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { ipcMain, Menu, BrowserWindow, app } from 'electron';
import { Logger } from '../logging/logging.js';

interface AppMenuItem {
  id?: string;
  label?: string;
  icon?: string;
  enabled?: boolean;
  visible?: boolean;
  type?: 'normal' | 'separator' | 'submenu' | 'checkbox' | 'radio';
  checked?: boolean;
  accelerator?: string;
  role?: string;
  submenu?: AppMenuItem[];
  routerLink?: string;
  command?: string;
  items?: AppMenuItem[];
}

class AppMenuModule implements AppModule {
  private readonly logger = Logger.getInstance();
  private currentMenuItems: AppMenuItem[] = [];

  enable(_context: ModuleContext): void {
    this.setupMenuHandlers();
  }

  private setupMenuHandlers(): void {
    const getMainWindow = (): BrowserWindow | null => {
      const windows = BrowserWindow.getAllWindows();
      return windows.length > 0 ? windows[0] : null;
    };

    // Handle menu updates from renderer
    ipcMain.handle('appMenu:update', async (_, menuItems: AppMenuItem[]) => {
      try {
        this.currentMenuItems = menuItems;

        const menu = this.buildApplicationMenu(menuItems);
        Menu.setApplicationMenu(menu);

        this.logger.debug('Application menu updated successfully');
        return true;
      } catch (error: unknown) {
        this.logger.error(`App menu update error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to update app menu: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    // Handle getting current menu items
    ipcMain.handle('appMenu:getItems', async () => {
      return this.currentMenuItems;
    });

    // Handle menu item clicks
    ipcMain.handle('appMenu:itemClicked', async (_, itemId: string) => {
      try {
        const win = getMainWindow();
        if (!win || win.isDestroyed()) {
          return false;
        }

        // Send the click event to the renderer
        win.webContents.send('appMenu:itemClicked', itemId);
        return true;
      } catch (error: unknown) {
        this.logger.error(`App menu item click error: ${error instanceof Error ? error.message : String(error)}`);
        return false;
      }
    });

    // Handle adding a menu item
    ipcMain.handle('appMenu:addItem', async (_, item: AppMenuItem, position?: number) => {
      try {
        if (!item || typeof item !== 'object') {
          throw new Error('Invalid menu item provided');
        }

        if (!item.label && item.type !== 'separator') {
          throw new Error('Menu item must have a label (except separators)');
        }

        const newItem = { ...item };

        if (position !== undefined && position >= 0 && position <= this.currentMenuItems.length) {
          this.currentMenuItems.splice(position, 0, newItem);
        } else {
          this.currentMenuItems.push(newItem);
        }

        // Rebuild and apply the menu
        const menu = this.buildApplicationMenu(this.currentMenuItems);
        Menu.setApplicationMenu(menu);

        this.logger.debug(`Added menu item: ${item.label || 'separator'}`);
        return true;
      } catch (error: unknown) {
        this.logger.error(`App menu addItem error: ${error instanceof Error ? error.message : String(error)}`);
        return false;
      }
    });

    // Handle removing a menu item
    ipcMain.handle('appMenu:removeItem', async (_, id: string) => {
      try {
        if (!id || typeof id !== 'string') {
          throw new Error('Item ID must be a non-empty string');
        }

        const initialLength = this.currentMenuItems.length;
        this.currentMenuItems = this.currentMenuItems.filter((item) => item.id !== id);

        const removed = this.currentMenuItems.length < initialLength;

        if (removed) {
          // Rebuild and apply the menu
          const menu = this.buildApplicationMenu(this.currentMenuItems);
          Menu.setApplicationMenu(menu);
          this.logger.debug(`Removed menu item with ID: ${id}`);
        }

        return removed;
      } catch (error: unknown) {
        this.logger.error(`App menu removeItem error: ${error instanceof Error ? error.message : String(error)}`);
        return false;
      }
    });

    // Handle finding a menu item
    ipcMain.handle('appMenu:findItem', async (_, id: string) => {
      try {
        if (!id || typeof id !== 'string') {
          throw new Error('Item ID must be a non-empty string');
        }

        const findInItems = (items: AppMenuItem[]): AppMenuItem | null => {
          for (const item of items) {
            if (item.id === id) {
              return { ...item }; // Return a copy
            }
            if (item.submenu) {
              const found = findInItems(item.submenu);
              if (found) return found;
            }
          }
          return null;
        };

        return findInItems(this.currentMenuItems);
      } catch (error: unknown) {
        this.logger.error(`App menu findItem error: ${error instanceof Error ? error.message : String(error)}`);
        return null;
      }
    });

    // Handle updating a menu item
    ipcMain.handle('appMenu:updateItem', async (_, id: string, updates: Partial<AppMenuItem>) => {
      try {
        if (!id || typeof id !== 'string') {
          throw new Error('Item ID must be a non-empty string');
        }

        if (!updates || typeof updates !== 'object') {
          throw new Error('Updates must be an object');
        }

        const updateInItems = (items: AppMenuItem[]): boolean => {
          for (const item of items) {
            if (item.id === id) {
              Object.assign(item, updates);
              return true;
            }
            if (item.submenu && updateInItems(item.submenu)) {
              return true;
            }
          }
          return false;
        };

        const updated = updateInItems(this.currentMenuItems);

        if (updated) {
          // Rebuild and apply the menu
          const menu = this.buildApplicationMenu(this.currentMenuItems);
          Menu.setApplicationMenu(menu);
          this.logger.debug(`Updated menu item with ID: ${id}`);
        }

        return updated;
      } catch (error: unknown) {
        this.logger.error(`App menu updateItem error: ${error instanceof Error ? error.message : String(error)}`);
        return false;
      }
    });

    // Handle clearing all menu items
    ipcMain.handle('appMenu:clear', async () => {
      try {
        this.currentMenuItems = [];

        // Rebuild and apply the menu
        const menu = this.buildApplicationMenu(this.currentMenuItems);
        Menu.setApplicationMenu(menu);

        this.logger.debug('Cleared all menu items');
        return true;
      } catch (error: unknown) {
        this.logger.error(`App menu clear error: ${error instanceof Error ? error.message : String(error)}`);
        return false;
      }
    });

    // Handle getting menu item count
    ipcMain.handle('appMenu:getItemCount', async () => {
      try {
        const countItems = (items: AppMenuItem[]): number => {
          let count = 0;
          for (const item of items) {
            count++;
            if (item.submenu) {
              count += countItems(item.submenu);
            }
          }
          return count;
        };

        return countItems(this.currentMenuItems);
      } catch (error: unknown) {
        this.logger.error(`App menu getItemCount error: ${error instanceof Error ? error.message : String(error)}`);
        return 0;
      }
    });
  }

  private buildApplicationMenu(items: AppMenuItem[]): Menu {
    this.logger.debug(`Building application menu with items: ${items.length}`);

    const isMac = process.platform === 'darwin';
    const template: Electron.MenuItemConstructorOptions[] = [
      // macOS app menu
      ...(isMac
        ? [
            {
              label: app.getName(),
              submenu: [
                { role: 'about' as const },
                { type: 'separator' as const },
                { role: 'services' as const },
                { type: 'separator' as const },
                { role: 'hide' as const },
                { role: 'hideOthers' as const },
                { role: 'unhide' as const },
                { type: 'separator' as const },
                { role: 'quit' as const },
              ],
            },
          ]
        : []),

      // File menu
      ...(items.find((item) => item.id === 'file')
        ? [
            {
              label: 'File',
              submenu: this.createSpecialMenuItems(items.find((item) => item.id === 'file')),
            },
          ]
        : []),

      // Modules menu
      ...(items.find((item) => item.id === 'modules')
        ? [
            {
              label: 'Modules',
              submenu: this.createSpecialMenuItems(items.find((item) => item.id === 'modules')),
            },
          ]
        : []),

      // Application menu items (excluding Help, File, and Modules menus)
      ...this.createMenuTemplate(
        items.filter((item) => item.id !== 'help' && item.id !== 'file' && item.id !== 'modules'),
      ),

      // Standard View menu
      {
        label: 'View',
        submenu: [
          { role: 'reload' as const },
          { role: 'forceReload' as const },
          { type: 'separator' as const },
          { role: 'resetZoom' as const },
          { role: 'zoomIn' as const },
          { role: 'zoomOut' as const },
          { type: 'separator' as const },
          { role: 'togglefullscreen' as const },
        ],
      },

      // Window menu
      {
        label: 'Window',
        submenu: [
          { role: 'minimize' as const },
          { role: 'zoom' as const },
          ...(isMac ? [{ type: 'separator' as const }, { role: 'front' as const }] : [{ role: 'close' as const }]),
        ],
      },

      // Help menu
      {
        label: 'Help',
        submenu: this.createHelpMenuItems(items.find((item) => item.id === 'help')),
      },
    ];

    return Menu.buildFromTemplate(template);
  }

  private createMenuTemplate(items: AppMenuItem[]): Electron.MenuItemConstructorOptions[] {
    return items.filter((item) => item.visible !== false).map((item) => this.createMenuItem(item));
  }

  private createMenuItem(item: AppMenuItem): Electron.MenuItemConstructorOptions {
    if (item.type === 'separator') {
      return { type: 'separator' };
    }

    const menuItem: Electron.MenuItemConstructorOptions = {
      id: item.id,
      label: item.label || '',
      enabled: item.enabled !== false,
      visible: item.visible !== false,
      type: item.type || 'normal',
      accelerator: item.accelerator,
    };

    // Handle role-based menu items
    if (item.role) {
      menuItem.role = item.role as Electron.MenuItemConstructorOptions['role'];
    }

    // Handle checkbox and radio types
    if (item.type === 'checkbox' || item.type === 'radio') {
      menuItem.checked = item.checked || false;
    }

    // Handle submenu items (both submenu and items properties)
    const submenuItems = item.submenu || item.items;
    const hasSubmenu = submenuItems && submenuItems.length > 0;
    if (hasSubmenu) {
      menuItem.submenu = this.createMenuTemplate(submenuItems);
    }

    // Handle click events for non-submenu items
    if (item.id && !hasSubmenu) {
      menuItem.click = () => {
        this.logger.trace(`Native menu item clicked: ${item.id}`);
        this.handleMenuItemClick(item);
      };
    }

    return menuItem;
  }

  private handleMenuItemClick(item: AppMenuItem): void {
    this.logger.trace(`Menu item click handler called for: ${item.id || item.label}`);

    const win = BrowserWindow.getAllWindows()[0];
    if (!win || win.isDestroyed()) {
      this.logger.warn('No main window available for menu item click');
      return;
    }

    const clickData = {
      id: item.id,
      routerLink: item.routerLink,
      command: item.command,
      item: item,
    };

    // Send the menu item data to renderer for handling
    win.webContents.send('appMenu:itemClicked', clickData);
  }

  private createSpecialMenuItems(menuItem?: AppMenuItem): Electron.MenuItemConstructorOptions[] {
    if (!menuItem || !menuItem.submenu || menuItem.submenu.length === 0) {
      return [];
    }

    return menuItem.submenu.map((item) => ({
      label: item.label || '',
      click: () => {
        const win = BrowserWindow.getAllWindows()[0];
        if (win && !win.isDestroyed()) {
          win.webContents.send('appMenu:itemClicked', {
            id: item.id,
            command: item.command,
            routerLink: item.routerLink,
            item: item,
          });
        }
      },
    }));
  }

  private createHelpMenuItems(helpMenuItem?: AppMenuItem): Electron.MenuItemConstructorOptions[] {
    if (!helpMenuItem || !helpMenuItem.submenu || helpMenuItem.submenu.length === 0) {
      return [
        {
          label: 'About',
          click: () => {
            const win = BrowserWindow.getAllWindows()[0];
            if (win && !win.isDestroyed()) {
              win.webContents.send('appMenu:itemClicked', {
                id: 'help-about',
                command: 'help-about',
              });
            }
          },
        },
      ];
    }

    return helpMenuItem.submenu.map((item) => ({
      label: item.label || '',
      click: () => {
        const win = BrowserWindow.getAllWindows()[0];
        if (win && !win.isDestroyed()) {
          win.webContents.send('appMenu:itemClicked', {
            id: item.id,
            command: item.command,
            item: item,
          });
        }
      },
    }));
  }
}

export function createAppMenuModule() {
  return new AppMenuModule();
}
