import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { ipcMain } from 'electron';
import { homedir, hostname, release, tmpdir } from 'node:os';
import { Logger } from '../logging/logging.js';

class OSModule implements AppModule {
  private readonly logger = Logger.getInstance();

  enable({ app }: ModuleContext): void {
    this.setupOSHandlers(app);
  }

  private setupOSHandlers(app: Electron.App): void {
    ipcMain.handle('os:locale', async () => {
      try {
        return app.getLocale();
      } catch (error: unknown) {
        this.logger.error(`OS locale error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to get locale: ${error instanceof Error ? error.message : error}`, { cause: error });
      }
    });

    ipcMain.handle('os:argv', async () => {
      try {
        return process.argv;
      } catch (error: unknown) {
        this.logger.error(`OS argv error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to get argv: ${error instanceof Error ? error.message : error}`, { cause: error });
      }
    });
  }
}

export function createOSModule() {
  return new OSModule();
}
