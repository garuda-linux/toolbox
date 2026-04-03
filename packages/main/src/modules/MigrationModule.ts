import { app } from 'electron';
import { rename, readdir, rm, readFile, writeFile, access } from 'node:fs/promises';
import { join } from 'node:path';
import { Logger } from '../logging/logging.js';

async function exists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

export async function migrateConfig() {
  const logger = Logger.getInstance();

  try {
    // Migrate app data directory
    const configDir = app.getPath('appData');
    const currentPath = join(configDir, 'garuda-toolbox');
    const oldPath = join(configDir, 'garuda-rani');

    logger.debug(`Checking for migration: oldPath=${oldPath}, currentPath=${currentPath}`);

    if (await exists(oldPath)) {
      const isCurrentEmpty = !(await exists(currentPath)) || (await readdir(currentPath)).length === 0;
      if (isCurrentEmpty) {
        logger.info(`Migrating config from ${oldPath} to ${currentPath}`);
        if (await exists(currentPath)) {
          await rm(currentPath, { recursive: true, force: true });
        }

        await rename(oldPath, currentPath);
        logger.info('Migration successful');
      } else {
        logger.warn(
          `Both ${oldPath} and ${currentPath} exist and the new one is not empty. Skipping migration to avoid overwriting.`,
        );
      }
    }

    // Migrate Plasma applet config
    const plasmaConfigPath = join(app.getPath('home'), '.config', 'plasma-org.kde.plasma.desktop-appletsrc');
    if (await exists(plasmaConfigPath)) {
      const content = await readFile(plasmaConfigPath, 'utf8');
      if (content.includes('applications:garuda-rani.desktop')) {
        logger.info(`Migrating Plasma applet config to garuda-toolbox.desktop in ${plasmaConfigPath}`);
        const newContent = content.replaceAll(
          'applications:garuda-rani.desktop',
          'applications:garuda-toolbox.desktop',
        );
        await writeFile(plasmaConfigPath, newContent, 'utf8');
        logger.info('Plasma applet migration successful');
      }
    }
  } catch (error) {
    logger.error(`Failed to migrate config: ${error instanceof Error ? error.message : String(error)}`);
  }
}
