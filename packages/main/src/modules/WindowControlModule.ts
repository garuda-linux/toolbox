import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { ipcMain, BrowserWindow, App } from 'electron';
import { Logger } from '../logging/logging.js';
import { execve } from 'node:process';

class WindowControlModule implements AppModule {
  private readonly logger = Logger.getInstance();

  enable({ app }: ModuleContext): void {
    this.setupWindowControlHandlers(app);
  }

  private setupWindowControlHandlers(app: App): void {
    // Get the main window instance
    const getMainWindow = (): BrowserWindow | null => {
      const windows = BrowserWindow.getAllWindows();
      return windows.length > 0 ? windows[0] : null;
    };

    // Window Operations
    ipcMain.handle('window:close', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          // Force close the window without confirmation
          win.destroy();
          return true;
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window close error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to close window: ${error instanceof Error ? error.message : error}`, { cause: error });
      }
    });

    // Add a new handler for graceful close with confirmation
    ipcMain.handle('window:requestClose', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          // This will trigger the beforeunload event in the renderer
          // which handles the confirmation logic
          win.close();
          return true;
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window request close error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to request window close: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('window:minimize', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          win.minimize();
          return true;
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window minimize error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to minimize window: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('window:maximize', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          if (win.isMaximized()) {
            win.unmaximize();
          } else {
            win.maximize();
          }
          return true;
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window maximize error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to maximize window: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('window:hide', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          win.hide();
          return true;
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window hide error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to hide window: ${error instanceof Error ? error.message : error}`, { cause: error });
      }
    });

    ipcMain.handle('window:show', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          win.show();
          return true;
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window show error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to show window: ${error instanceof Error ? error.message : error}`, { cause: error });
      }
    });

    ipcMain.handle('window:focus', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          win.focus();
          return true;
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window focus error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to focus window: ${error instanceof Error ? error.message : error}`, { cause: error });
      }
    });

    ipcMain.handle('window:isMaximized', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          return win.isMaximized();
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window isMaximized error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to check if window is maximized: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('window:isMinimized', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          return win.isMinimized();
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window isMinimized error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to check if window is minimized: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('window:isVisible', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          return win.isVisible();
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window isVisible error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to check if window is visible: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('window:setTitle', async (_, title: string) => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          win.setTitle(title);
          return true;
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window setTitle error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to set window title: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('window:getTitle', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          return win.getTitle();
        }
        return '';
      } catch (error: any) {
        this.logger.error(`Window getTitle error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to get window title: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('window:setSize', async (_, width: number, height: number) => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          win.setSize(width, height);
          return true;
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window setSize error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to set window size: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('window:getSize', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          return win.getSize();
        }
        return [0, 0];
      } catch (error: any) {
        this.logger.error(`Window getSize error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to get window size: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('window:setPosition', async (_, x: number, y: number) => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          win.setPosition(x, y);
          return true;
        }
        return false;
      } catch (error: any) {
        this.logger.error(`Window setPosition error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to set window position: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('window:getPosition', async () => {
      try {
        const win = getMainWindow();
        if (win && !win.isDestroyed()) {
          return win.getPosition();
        }
        return [0, 0];
      } catch (error: any) {
        this.logger.error(`Window getPosition error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to get window position: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('window:relaunch', async () => {
      try {
        app.on('quit', () => {
          // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
          execve!(process.execPath, process.argv, process.env);
        });
        app.exit();
        return true;
      } catch (error: any) {
        this.logger.error(`Window relaunch error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to relaunch app: ${error instanceof Error ? error.message : error}`, { cause: error });
      }
    });
  }
}

export function createWindowControlModule() {
  return new WindowControlModule();
}
