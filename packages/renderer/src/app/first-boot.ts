import { Logger } from './logging/logging';
import { getConfigStore } from './components/config/store';
import {
  type Store,
  type CommandResult,
  ElectronShellService,
  ElectronFsService,
  ElectronAppService,
  ElectronWindowService,
} from './electron-services';

const logger = Logger.getInstance();

/**
 * Check if the system is on first boot and act accordingly, showing the setup assistant if required.
 * Sets the firstBoot flag to false after the first boot is detected.
 */
export async function checkFirstBoot(): Promise<boolean> {
  const store: Store = await getConfigStore('checkFirstBoot');
  const firstBoot: boolean | undefined = await store.get<boolean>('firstBoot');
  const shellService = new ElectronShellService();
  const fsService = new ElectronFsService();
  const appService = new ElectronAppService();
  const windowService = new ElectronWindowService();

  // Check if we've been through this before
  if (firstBoot === false) return false;

  try {
    const result: CommandResult = await new shellService.Command('last')
      .args(['reboot', '-n', '2', '--time-format', 'notime'])
      .execute();

    // If row count of the result is 1, then it's the first boot, otherwise it's not
    if (result.stdout.split('\n').length !== 2) {
      logger.info('Not first boot');
      await store.set('firstBoot', false);
      return false;
    }

    if (await fsService.exists('/usr/bin/setup-assistant')) {
      logger.info('Setup assistant exists, running');
      await windowService.hide();
      await runSetupAssistant();

      // Set first boot flag to false
      await store.set('firstBoot', false);

      // And finally relaunch
      await appService.relaunch();
      return true;
    }

    logger.debug('Setup assistant does not exist');
    return false;
  } catch (error) {
    logger.error(`Error checking first boot: ${error}`);
    return false;
  }
}

/**
 * Run the setup assistant and wait until it's done.
 */
async function runSetupAssistant(): Promise<void> {
  const shellService = new ElectronShellService();

  try {
    const result: CommandResult = await new shellService.Command('sh')
      .args(['-c', 'VERSION=4 setup-assistant'])
      .execute();

    if (result.code === 0) {
      logger.info('Setup assistant completed successfully');
    } else {
      logger.warn(`Setup assistant exited with code ${result.code}`);
    }
  } catch (error) {
    logger.error(`Error running setup assistant: ${error}`);
  }
}
