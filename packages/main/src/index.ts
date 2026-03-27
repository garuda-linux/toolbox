import type { AppInitConfig } from './AppInitConfig.js';
import { createModuleRunner, type ModuleRunner } from './ModuleRunner.js';
import { disallowMultipleAppInstance } from './modules/SingleInstanceApp.js';
import { createWindowManagerModule } from './modules/WindowManager.js';
import { allowInternalOrigins } from './modules/BlockNotAllowdOrigins.js';
import { allowExternalUrls } from './modules/ExternalUrls.js';
import { createConfigModule } from './modules/ConfigModule.js';
import { createOSModule } from './modules/OSModule.js';
import { createNotificationModule } from './modules/NotificationModule.js';
import { createWindowControlModule } from './modules/WindowControlModule.js';
import { createLoggingModule } from './modules/LoggingModule.js';
import { createDialogModule } from './modules/DialogModule.js';
import { createClipboardModule } from './modules/ClipboardModule.js';
import { createContextMenuModule } from './modules/ContextMenuModule.js';
import { createAppMenuModule } from './modules/AppMenuModule.js';
import { createHttpModule } from './modules/HttpModule.js';
import { createEnhancedSecurityModule } from './modules/EnhancedSecurityModule.js';
import { createHomeConfigModule } from './modules/HomeConfigModule.js';
import { createAppIconModule } from './modules/AppIconModule.js';
import { createShellModule } from './modules/ShellModule.js';
import { app, protocol } from 'electron';
import { Logger } from './logging/logging.js';
import { migrateConfig } from './modules/MigrationModule.js';

// GPU acceleration flags — must be set before app.whenReady()
// Electron's GPU process spawns as a zygote child that doesn't inherit
// --ozone-platform=wayland or DRM features (electron/electron#50455).
// --no-zygote disables the zygote process pool, forcing each child process
// to be spawned fresh with the full command line including GPU flags.
// This trades startup speed for correct GPU flag propagation.
app.commandLine.appendSwitch('ignore-gpu-blocklist');
app.commandLine.appendSwitch('enable-gpu-rasterization');
app.commandLine.appendSwitch('enable-zero-copy');
app.commandLine.appendSwitch('no-zygote');
app.commandLine.appendSwitch(
  'enable-features',
  'AcceleratedVideoDecodeLinuxGL,AcceleratedVideoDecodeLinuxZeroCopyGL,' +
    'VaapiVideoDecoder,VaapiVideoEncoder,VaapiOnNvidiaGPUs,' +
    'AcceleratedVideoEncoder,CanvasOopRasterization,',
);

protocol.registerSchemesAsPrivileged([
  {
    scheme: 'app-icon',
    privileges: {
      secure: true,
      standard: true,
      supportFetchAPI: true,
      corsEnabled: true,
      stream: true,
    },
  },
]);

export async function initApp(initConfig: AppInitConfig) {
  const logger = Logger.getInstance();

  if (process.getuid && process.getuid() === 0) {
    logger.error('The application should not be run as root. Exiting.');
    app.quit();
    return;
  }

  migrateConfig();

  const isDevelopment = import.meta.env.DEV;

  // Shut the fuck up, thank you.
  process.env.ELECTRON_NO_ATTACH_CONSOLE = '1';

  try {
    // First priority: Show window immediately with minimal setup
    const moduleRunner: ModuleRunner = createModuleRunner()
      // Core modules for window creation
      .init(disallowMultipleAppInstance())
      .init(createEnhancedSecurityModule(isDevelopment))
      .init(createAppIconModule())

      // Security modules
      .init(allowInternalOrigins(new Set(initConfig.renderer instanceof URL ? [initConfig.renderer.origin] : [])))
      .init(allowExternalUrls(new Set(initConfig.renderer instanceof URL ? ['garudalinux.org'] : [])))

      // Window manager — show window ASAP
      .init(
        createWindowManagerModule({
          initConfig,
          openDevTools: isDevelopment,
          isDevelopment,
        }),
      );

    await moduleRunner;

    // Register other IPC handlers in background after window is shown
    await registerBackgroundIPCHandlers(app, logger);
  } catch (error) {
    logger.error(`Failed to initialize app: ${error instanceof Error ? error.message : String(error)}`);
    throw error;
  }
}

async function registerBackgroundIPCHandlers(app: Electron.App, logger: Logger) {
  try {
    // Create all IPC modules
    const loggingModule = createLoggingModule();
    const configModule = createConfigModule();
    const osModule = createOSModule();
    const notificationModule = createNotificationModule();
    const windowControlModule = createWindowControlModule();
    const dialogModule = createDialogModule();
    const clipboardModule = createClipboardModule();
    const contextMenuModule = createContextMenuModule();
    const appMenuModule = createAppMenuModule();
    const httpModule = createHttpModule();
    const homeConfigModule = createHomeConfigModule();
    const shellModule = createShellModule();

    // Create module context
    const moduleContext = { app };

    // Enable IPC modules
    loggingModule.enable(moduleContext);
    configModule.enable(moduleContext);
    osModule.enable(moduleContext);
    notificationModule.enable(moduleContext);
    windowControlModule.enable(moduleContext);
    dialogModule.enable(moduleContext);
    clipboardModule.enable(moduleContext);
    contextMenuModule.enable(moduleContext);
    appMenuModule.enable(moduleContext);
    httpModule.enable(moduleContext);
    homeConfigModule.enable(moduleContext);
    shellModule.enable(moduleContext);

    logger.debug('Background IPC handlers registered successfully');
  } catch (error) {
    logger.error(
      `Failed to register background IPC handlers: ${error instanceof Error ? error.message : String(error)}`,
    );
  }
}
