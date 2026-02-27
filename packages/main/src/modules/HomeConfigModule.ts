import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { ipcMain } from 'electron';
import { homedir } from 'node:os';
import { mkdir, writeFile, readFile, access, constants, unlink } from 'node:fs/promises';
import { join } from 'node:path';
import { Logger } from '../logging/logging.js';

class HomeConfigModule implements AppModule {
  private readonly logger = Logger.getInstance();

  enable(_moduleContext: ModuleContext): void {
    this.setupHomeConfigHandlers();
  }

  private getHomeDir(): string {
    return homedir();
  }

  private async ensureDir(dirPath: string): Promise<void> {
    try {
      await mkdir(dirPath, { recursive: true });
    } catch (error) {
      this.logger.error(
        `Failed to create directory ${dirPath}: ${error instanceof Error ? error.message : String(error)}`,
      );
      throw error;
    }
  }

  private async fileExists(filePath: string): Promise<boolean> {
    try {
      await access(filePath, constants.F_OK);
      return true;
    } catch {
      return false;
    }
  }

  private setupHomeConfigHandlers(): void {
    ipcMain.handle('homeConfig:write', async (_event, relativePath: string, content: string, createDir = true) => {
      try {
        const homeDir = this.getHomeDir();
        const fullPath = join(homeDir, relativePath);

        if (createDir) {
          const dirPath = join(fullPath, '..');
          await this.ensureDir(dirPath);
        }

        await writeFile(fullPath, content, { encoding: 'utf8' });
        this.logger.debug(`Wrote config file: ${fullPath}`);
        return { success: true, path: fullPath };
      } catch (error) {
        this.logger.error(`Failed to write config file: ${error instanceof Error ? error.message : String(error)}`);
        return { success: false, error: error instanceof Error ? error.message : String(error) };
      }
    });

    ipcMain.handle('homeConfig:read', async (_event, relativePath: string) => {
      try {
        const homeDir = this.getHomeDir();
        const fullPath = join(homeDir, relativePath);

        const exists = await this.fileExists(fullPath);
        if (!exists) {
          return { success: false, error: 'File does not exist', exists: false };
        }

        const content = await readFile(fullPath, { encoding: 'utf8' });
        return { success: true, content, path: fullPath, exists: true };
      } catch (error) {
        this.logger.error(`Failed to read config file: ${error instanceof Error ? error.message : String(error)}`);
        return { success: false, error: error instanceof Error ? error.message : String(error) };
      }
    });

    ipcMain.handle('homeConfig:exists', async (_event, relativePath: string) => {
      try {
        const homeDir = this.getHomeDir();
        const fullPath = join(homeDir, relativePath);
        const exists = await this.fileExists(fullPath);
        return { exists, path: fullPath };
      } catch (error) {
        this.logger.error(`Failed to check config file: ${error instanceof Error ? error.message : String(error)}`);
        return { exists: false, error: error instanceof Error ? error.message : String(error) };
      }
    });

    ipcMain.handle('homeConfig:remove', async (_event, relativePath: string) => {
      try {
        const homeDir = this.getHomeDir();
        const fullPath = join(homeDir, relativePath);

        const exists = await this.fileExists(fullPath);
        if (!exists) {
          return { success: true };
        }

        await unlink(fullPath);
        this.logger.debug(`Removed config file: ${fullPath}`);
        return { success: true };
      } catch (error) {
        this.logger.error(`Failed to remove config file: ${error instanceof Error ? error.message : String(error)}`);
        return { success: false, error: error instanceof Error ? error.message : String(error) };
      }
    });
  }
}

export function createHomeConfigModule() {
  return new HomeConfigModule();
}
