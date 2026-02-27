import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { BrowserWindow, Size, screen, shell } from 'electron';
import type { AppInitConfig } from '../AppInitConfig.js';
import { Logger } from '../logging/logging.js';
import ElectronStore from 'electron-store';
import Rectangle = Electron.Rectangle;

class WindowManager implements AppModule {
  readonly #preload: { path: string };
  readonly #renderer: { path: string } | URL;
  readonly #openDevTools;
  readonly #isDevelopment;
  readonly store = new ElectronStore({
    encryptionKey: 'non-security-by-obscurity',
  });

  private readonly logger = Logger.getInstance();
  private readonly rendererLogger = Logger.child('Renderer');

  constructor({
    initConfig,
    openDevTools = false,
    isDevelopment = false,
  }: {
    initConfig: AppInitConfig;
    openDevTools?: boolean;
    isDevelopment?: boolean;
  }) {
    this.#preload = initConfig.preload;
    this.#renderer = initConfig.renderer;
    this.#openDevTools = openDevTools;
    this.#isDevelopment = isDevelopment;
  }

  async enable({ app }: ModuleContext): Promise<void> {
    await app.whenReady();

    // Show window immediately with splash screen
    await this.restoreOrCreateWindow(true);
    app.on('second-instance', () => this.restoreOrCreateWindow(true));
    app.on('activate', () => this.restoreOrCreateWindow(true));
  }

  async createWindow(): Promise<BrowserWindow> {
    const windowSize: Size = screen.getPrimaryDisplay().workAreaSize;
    const size: Size = screen.getPrimaryDisplay().workAreaSize;
    this.logger.debug(`Get display with size: ${size.width}x${size.height}`);

    // Define reasonable window dimensions
    const minWidth = windowSize.width > 2000 ? 1920 : 1280;
    const minHeight = windowSize.height > 1300 ? 1080 : 800;

    // Use reasonable size that fits the screen
    const windowWidth = Math.min(minWidth, windowSize.width * 0.9);
    const windowHeight = Math.min(minHeight, windowSize.height * 0.9);

    this.logger.debug(`Window size will be: ${windowWidth}x${windowHeight}`);

    // Center the window using actual window dimensions
    const x = Math.round((windowSize.width - windowWidth) / 2);
    const y = Math.round((windowSize.height - windowHeight) / 2);

    // Create the browser window with secure defaults
    const browserWindow = new BrowserWindow({
      x,
      y,
      width: minWidth,
      height: minHeight,
      minHeight: 500,
      minWidth: 700,
      show: true,
      frame: true,
      title: 'Garuda Toolbox',
      backgroundColor: '#1e1e2e',
      webPreferences: {
        nodeIntegration: false,
        contextIsolation: true,
        allowRunningInsecureContent: false,
        preload: this.#preload.path,
        webSecurity: true,
        sandbox: false,
        webviewTag: false,
        experimentalFeatures: false,
        plugins: false,
        webgl: false,
        accessibleTitle: 'Garuda Toolbox',
        navigateOnDragDrop: false,
        autoplayPolicy: 'user-gesture-required',
        safeDialogs: true,
        safeDialogsMessage: 'This app is preventing additional dialogs',
      },
    });

    const savedBounds = this.store.get('bounds') as Rectangle;
    if (savedBounds !== undefined) {
      this.logger.debug(`Restoring window bounds: ${JSON.stringify(savedBounds)}`);
      const screenArea: Rectangle = screen.getDisplayMatching(savedBounds).workArea;
      if (
        savedBounds.x > screenArea.x + screenArea.width ||
        savedBounds.x < screenArea.x ||
        savedBounds.y < screenArea.y ||
        savedBounds.y > screenArea.y + screenArea.height
      ) {
        // Reset window into existing screenarea
        browserWindow.setBounds({ x: 0, y: 0, width: size.width, height: size.height });
      } else {
        browserWindow.setBounds(savedBounds);
      }
    }

    // Set title explicitly to avoid issues with default title
    browserWindow.setTitle('Garuda Toolbox');

    // Some error handling for the renderer process
    browserWindow.once('ready-to-show', () => {
      browserWindow.webContents.on('unresponsive', () => {
        this.logger.warn('Renderer process became unresponsive');
      });
      browserWindow.webContents.on('responsive', () => {
        this.logger.info('Renderer process became responsive again');
      });
    });

    // Handle navigation security
    browserWindow.webContents.on('will-navigate', (event, navigationUrl) => {
      const parsedUrl = new URL(navigationUrl);

      // Allow devtools URLs
      if (navigationUrl.startsWith('devtools://') || navigationUrl.startsWith('chrome-devtools://')) {
        return;
      }

      if (parsedUrl.origin !== 'http://localhost:5173' && parsedUrl.protocol !== 'file:') {
        event.preventDefault();
        this.logger.warn(`Blocked navigation to: ${navigationUrl}`);
      }
    });

    // Handle new window creation security
    browserWindow.webContents.setWindowOpenHandler(({ url }) => {
      // Allow devtools
      if (url.startsWith('devtools://') || url.startsWith('chrome-devtools://')) {
        return { action: 'allow' };
      }

      // Allow external URLs to open in default browser
      if (url.startsWith('http://') || url.startsWith('https://')) {
        shell.openExternal(url);
      }
      return { action: 'deny' };
    });

    browserWindow.on('closed', () => {
      // Window cleanup will be handled by the framework
    });

    browserWindow.on('close', () => {
      this.logger.debug('Window is closing, saving bounds...');
      this.store.set('bounds', browserWindow.getBounds());
    });

    browserWindow.on('page-title-updated', (e) => {
      e.preventDefault();
    });

    // Handle uncaught exceptions in renderer
    browserWindow.webContents.on('render-process-gone', (event, details) => {
      this.logger.error(`Renderer process gone: ${JSON.stringify(details, null, 2)}`);
    });

    // Handle console messages from renderer for better debugging
    browserWindow.webContents.on('console-message', (event) => {
      switch (event.level) {
        case 'debug':
          this.rendererLogger.debug(`${event.message}`);
          break;
        case 'info':
          this.rendererLogger.info(`${event.message}`);
          break;
        case 'warning':
          this.rendererLogger.warn(`${event.message}`);
          break;
        case 'error':
          this.rendererLogger.error(`${event.message}`);
          break;
      }
    });

    if (this.#renderer instanceof URL) {
      await browserWindow.loadURL(this.#renderer.href);
    } else {
      await browserWindow.loadFile(this.#renderer.path);
    }

    // Only open dev tools in development and handle errors
    if (this.#isDevelopment && this.#openDevTools) {
      browserWindow.webContents.once('dom-ready', () => {
        try {
          browserWindow.webContents.openDevTools({ mode: 'detach' });
        } catch (error) {
          this.logger.warn(`Could not open DevTools: ${error instanceof Error ? error.message : String(error)}`);
        }
      });
    }

    return browserWindow;
  }

  async restoreOrCreateWindow(show = false) {
    let window = BrowserWindow.getAllWindows().find((w) => !w.isDestroyed());
    window ??= await this.createWindow();

    if (!show) {
      return window;
    }

    if (window.isMinimized()) {
      window.restore();
    }

    window?.show();

    if (this.#openDevTools) {
      window?.webContents.openDevTools();
    }

    window.focus();

    return window;
  }
}

export function createWindowManagerModule(...args: ConstructorParameters<typeof WindowManager>) {
  return new WindowManager(...args);
}
