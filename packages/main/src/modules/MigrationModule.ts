import { app } from 'electron';
import { rename, readdir, rm, readFile, writeFile, access, unlink } from 'node:fs/promises';
import { join } from 'node:path';
import { exec } from 'node:child_process';
import { promisify } from 'node:util';
import { Logger } from '../logging/logging.js';

const execAsync = promisify(exec);
const CURRENT_MIGRATION_VERSION = 2;

async function exists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

async function getMigrationVersion(configDir: string): Promise<number> {
  const versionFile = join(configDir, '.migration-version');
  try {
    const content = await readFile(versionFile, 'utf8');
    return parseInt(content.trim(), 10) || 0;
  } catch {
    return 0;
  }
}

async function setMigrationVersion(configDir: string, version: number): Promise<void> {
  const versionFile = join(configDir, '.migration-version');
  await writeFile(versionFile, version.toString(), 'utf8');
}

export async function migrateConfig() {
  const logger = Logger.getInstance();
  const configDir = app.getPath('appData');
  const toolboxConfigDir = join(configDir, 'garuda-toolbox');

  const currentVersion = await getMigrationVersion(toolboxConfigDir);
  if (currentVersion >= CURRENT_MIGRATION_VERSION) {
    logger.debug(`Migration version ${currentVersion} is up to date`);
    return;
  }

  logger.info(`Running migrations from version ${currentVersion} to ${CURRENT_MIGRATION_VERSION}`);

  try {
    if (currentVersion < 1) {
      const oldPath = join(configDir, 'garuda-rani');
      if (await exists(oldPath)) {
        const isCurrentEmpty = !(await exists(toolboxConfigDir)) || (await readdir(toolboxConfigDir)).length === 0;
        if (isCurrentEmpty) {
          logger.info(`Migrating config from ${oldPath} to ${toolboxConfigDir}`);
          if (await exists(toolboxConfigDir)) {
            await rm(toolboxConfigDir, { recursive: true, force: true });
          }
          await rename(oldPath, toolboxConfigDir);
        } else {
          logger.warn('Both old and new config directories exist - skipping migration');
        }
      }

      const plasmaConfigPath = join(app.getPath('home'), '.config', 'plasma-org.kde.plasma.desktop-appletsrc');
      if (await exists(plasmaConfigPath)) {
        const content = await readFile(plasmaConfigPath, 'utf8');
        if (content.includes('applications:garuda-rani.desktop')) {
          const newContent = content.replaceAll(
            'applications:garuda-rani.desktop',
            'applications:garuda-toolbox.desktop',
          );
          await writeFile(plasmaConfigPath, newContent, 'utf8');
          logger.info('Migrated Plasma applet config');
        }
      }

      await setMigrationVersion(toolboxConfigDir, 1);
    }

    if (currentVersion < 2) {
      const autostartDir = join(app.getPath('home'), '.config', 'autostart');
      const oldAutostartFiles = [
        join(autostartDir, 'org.garudalinux.rani.desktop'),
        join(autostartDir, 'garuda-rani.desktop'),
      ];
      const newAutostartFile = join(autostartDir, 'garuda-toolbox.desktop');

      let hasOldAutostart = false;
      for (const oldFile of oldAutostartFiles) {
        if (await exists(oldFile)) {
          hasOldAutostart = true;
          try {
            await unlink(oldFile);
            logger.info(`Removed old autostart file: ${oldFile}`);
          } catch (error) {
            logger.warn(`Failed to remove old autostart file ${oldFile}: ${error}`);
          }
        }
      }

      if (hasOldAutostart && !(await exists(newAutostartFile))) {
        try {
          await execAsync(`mkdir -p "${autostartDir}"`);
          await execAsync(`ln -sf /usr/share/applications/garuda-toolbox.desktop "${newAutostartFile}"`);
          logger.info('Migrated autostart to garuda-toolbox.desktop');
        } catch (error) {
          logger.error(`Failed to create new autostart symlink: ${error}`);
        }
      }

      await setMigrationVersion(toolboxConfigDir, 2);
    }
  } catch (error) {
    logger.error(`Failed to migrate config: ${error instanceof Error ? error.message : String(error)}`);
  }
}
