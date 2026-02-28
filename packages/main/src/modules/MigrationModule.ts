import { app } from 'electron';
import { existsSync, renameSync, readdirSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { Logger } from '../logging/logging.js';

export function migrateConfig() {
  const logger = Logger.getInstance();

  try {
    const configDir = app.getPath('appData');
    const currentPath = join(configDir, 'garuda-toolbox');
    const oldPath = join(configDir, 'garuda-rani');

    logger.debug(`Checking for migration: oldPath=${oldPath}, currentPath=${currentPath}`);

    if (existsSync(oldPath)) {
      const isCurrentEmpty = !existsSync(currentPath) || readdirSync(currentPath).length === 0;
      if (isCurrentEmpty) {
        logger.info(`Migrating config from ${oldPath} to ${currentPath}`);
        if (existsSync(currentPath)) {
          rmSync(currentPath, { recursive: true, force: true });
        }

        renameSync(oldPath, currentPath);
        logger.info('Migration successful');
      } else {
        logger.warn(
          `Both ${oldPath} and ${currentPath} exist and the new one is not empty. Skipping migration to avoid overwriting.`,
        );
      }
    }
  } catch (error) {
    logger.error(`Failed to migrate config: ${error instanceof Error ? error.message : String(error)}`);
  }
}
