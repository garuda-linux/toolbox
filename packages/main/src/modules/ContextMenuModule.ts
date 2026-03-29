import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { ipcMain, BrowserWindow, Menu } from 'electron';
import { Logger } from '../logging/logging.js';

interface ContextMenuItem {
  id?: string;
  label?: string;
  icon?: string;
  enabled?: boolean;
  visible?: boolean;
  type?: 'normal' | 'separator' | 'submenu' | 'checkbox' | 'radio';
  checked?: boolean;
  accelerator?: string;
  submenu?: ContextMenuItem[];
}

class ContextMenuModule implements AppModule {
  private readonly logger = Logger.getInstance();

  enable({ app }: ModuleContext): void {
    this.setupContextMenuHandlers();
  }

  private setupContextMenuHandlers(): void {
    const getMainWindow = (): BrowserWindow | null => {
      const windows = BrowserWindow.getAllWindows();
      return windows.length > 0 ? windows[0] : null;
    };

    ipcMain.handle('contextMenu:show', async (_, items: ContextMenuItem[], x?: number, y?: number) => {
      try {
        const win = getMainWindow();
        if (!win || win.isDestroyed()) {
          return false;
        }

        const menu = this.buildMenu(items, win);

        if (x !== undefined && y !== undefined) {
          menu.popup({
            window: win,
            x: Math.round(x),
            y: Math.round(y),
          });
        } else {
          menu.popup({ window: win });
        }

        return true;
      } catch (error: any) {
        this.logger.error(`Context menu show error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to show context menu: ${error instanceof Error ? error.message : error}`, { cause: error });
      }
    });
  }

  private buildMenu(items: ContextMenuItem[], window: BrowserWindow): Menu {
    const menuItems = items.map((item) => this.createMenuItem(item, window));
    return Menu.buildFromTemplate(menuItems);
  }

  private createMenuItem(item: ContextMenuItem, window: BrowserWindow): Electron.MenuItemConstructorOptions {
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

    if (item.type === 'checkbox' || item.type === 'radio') {
      menuItem.checked = item.checked || false;
    }

    if (item.submenu && item.submenu.length > 0) {
      menuItem.submenu = item.submenu.map((subItem) => this.createMenuItem(subItem, window));
    }

    if (item.id && !item.submenu) {
      menuItem.click = () => {
        window.webContents.send('contextMenu:itemClicked', item.id);
      };
    }

    return menuItem;
  }
}

export function createContextMenuModule() {
  return new ContextMenuModule();
}
