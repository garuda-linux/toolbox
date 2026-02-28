import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { app, session, shell } from 'electron';
import { Logger } from '../logging/logging.js';

class EnhancedSecurityModule implements AppModule {
  private readonly isDevelopment: boolean;
  private readonly logger = Logger.getInstance();

  constructor(isDevelopment = false) {
    this.isDevelopment = isDevelopment;
  }

  async enable({ app: _app }: ModuleContext): Promise<void> {
    await app.whenReady();
    this.setupSecurityHeaders();
    this.setupRequestFiltering();
    this.setupWebContentsProtection();
    this.setupAppProtection();
  }

  private setupSecurityHeaders(): void {
    // Set comprehensive security headers for production
    if (!this.isDevelopment) {
      session.defaultSession.webRequest.onHeadersReceived((details, callback) => {
        callback({
          responseHeaders: {
            ...details.responseHeaders,
            'Content-Security-Policy': [
              "default-src 'self'; " +
                "script-src 'self' 'unsafe-inline' 'unsafe-eval'; " +
                "style-src 'self' 'unsafe-inline'; " +
                "img-src 'self' data: blob: app-icon:; " +
                "font-src 'self' data:; " +
                "connect-src 'self';" +
                "object-src 'none'; " +
                "frame-src 'none'; " +
                "base-uri 'self'; " +
                "form-action 'self';",
            ],
            'X-Content-Type-Options': ['nosniff'],
            'X-Frame-Options': ['DENY'],
            'X-XSS-Protection': ['1; mode=block'],
            'Referrer-Policy': ['strict-origin-when-cross-origin'],
            'Permissions-Policy': ['geolocation=(), microphone=(), camera=(), fullscreen=(self)'],
            'Strict-Transport-Security': ['max-age=31536000; includeSubDomains'],
            'X-Download-Options': ['noopen'],
            'X-Permitted-Cross-Domain-Policies': ['none'],
          },
        });
      });
    }
  }

  private setupRequestFiltering(): void {
    // Block external requests that aren't explicitly allowed (only in production)
    if (!this.isDevelopment) {
      session.defaultSession.webRequest.onBeforeRequest((details, callback) => {
        try {
          const url = new URL(details.url);
          const allowedDomains = ['localhost', '127.0.0.1'];
          const allowedProtocols = ['file:', 'data:', 'blob:', 'chrome-extension:', 'app-icon:'];

          if (allowedProtocols.includes(url.protocol) || allowedDomains.includes(url.hostname)) {
            callback({ cancel: false });
          } else {
            this.logger.warn(`Blocked external request to: ${details.url}`);
            callback({ cancel: true });
          }
        } catch (error: any) {
          this.logger.warn(
            `Invalid URL blocked: ${details.url} - ${error instanceof Error ? error.message : String(error)}`,
          );
          callback({ cancel: true });
        }
      });
    }

    // Block potentially dangerous file downloads
    session.defaultSession.on('will-download', (event, item, _webContents) => {
      const filename = item.getFilename();
      const dangerousExtensions = [
        '.exe',
        '.bat',
        '.cmd',
        '.com',
        '.scr',
        '.pif',
        '.vbs',
        '.js',
        '.jar',
        '.app',
        '.deb',
        '.rpm',
        '.dmg',
        '.pkg',
        '.msi',
        '.ps1',
        '.sh',
      ];

      const isDangerous = dangerousExtensions.some((ext) => filename.toLowerCase().endsWith(ext));

      if (isDangerous && !this.isDevelopment) {
        this.logger.warn(`Blocked potentially dangerous download: ${filename}`);
        event.preventDefault();
      }
    });
  }

  private setupWebContentsProtection(): void {
    // Prevent new window creation with unsafe features
    app.on('web-contents-created', (event, contents) => {
      // Handle window opening
      contents.setWindowOpenHandler(({ url }) => {
        this.logger.warn(`Blocked new window creation: ${url}`);
        // Allow external URLs to open in default browser instead
        if (url.startsWith('http://') || url.startsWith('https://')) {
          shell.openExternal(url);
        }
        return { action: 'deny' };
      });

      // Prevent webview attachment
      contents.on('will-attach-webview', (event) => {
        event.preventDefault();
        this.logger.warn('Blocked webview attachment');
      });

      // Handle navigation security
      contents.on('will-navigate', (event, navigationUrl) => {
        try {
          const parsedUrl = new URL(navigationUrl);
          const allowedOrigins = ['http://localhost:5173'];
          const isFileProtocol = navigationUrl.startsWith('file://');
          const isDevTools = navigationUrl.includes('devtools');

          if (!allowedOrigins.includes(parsedUrl.origin) && !isFileProtocol && !isDevTools) {
            this.logger.warn(`Blocked navigation to: ${navigationUrl}`);
            event.preventDefault();
          }
        } catch (error: any) {
          this.logger.warn(
            `Invalid navigation URL blocked: ${navigationUrl} - ${error instanceof Error ? error.message : String(error)}`,
          );
          event.preventDefault();
        }
      });

      // Handle frame navigation
      contents.on('will-navigate', (event, navigationUrl) => {
        try {
          const parsedUrl = new URL(navigationUrl);
          const allowedOrigins = ['http://localhost:5173'];
          const isFileProtocol = navigationUrl.startsWith('file://');

          if (!allowedOrigins.includes(parsedUrl.origin) && !isFileProtocol) {
            this.logger.warn(`Blocked frame navigation to: ${navigationUrl}`);
            event.preventDefault();
          }
        } catch (error: any) {
          this.logger.warn(
            `Invalid frame navigation URL blocked: ${navigationUrl} - ${error instanceof Error ? error.message : String(error)}`,
          );
          event.preventDefault();
        }
      });

      // Prevent loading of remote content in frames
      contents.on('did-create-window', (window) => {
        window.webContents.on('will-navigate', (event, url) => {
          if (!url.startsWith('file://') && !url.startsWith('http://localhost')) {
            event.preventDefault();
            this.logger.warn(`Blocked remote navigation in child window: ${url}`);
          }
        });
      });

      // Handle certificate errors
      contents.on('certificate-error', (event, url, error, certificate, callback) => {
        if (this.isDevelopment) {
          this.logger.warn(`Certificate error for ${url}: ${error}`);
          callback(true);
        } else {
          this.logger.error(`Certificate error for ${url}: ${error}`);
          callback(false);
        }
      });

      // Monitor console messages for security issues
      contents.on('console-message', (event) => {
        // Check for potential security issues in console messages
        const securityKeywords = ['XSS', 'injection', 'eval', 'unsafe'];
        if (securityKeywords.some((keyword) => event.message.toLowerCase().includes(keyword))) {
          this.logger.warn(`Potential security issue detected in renderer: ${event.message}`);
        }
      });
    });
  }

  private setupAppProtection(): void {
    app.setAppUserModelId('org.garudalinux.toolbox');

    // Handle app certificate verification
    app.on('certificate-error', (event, webContents, url, error, certificate, callback) => {
      if (this.isDevelopment) {
        this.logger.warn(`App certificate error for ${url}: ${error}`);
        callback(true);
      } else {
        this.logger.error(`App certificate error for ${url}: ${error}`);
        callback(false);
      }
    });

    // Handle select client certificate
    app.on('select-client-certificate', (event, webContents, url, list, callback) => {
      event.preventDefault();
      this.logger.warn(`Client certificate request for ${url}`);

      // Don't provide any client certificate
      callback();
    });

    // Handle login requests
    app.on('login', (event, webContents, details, authInfo, callback) => {
      event.preventDefault();
      this.logger.warn(`Login request for ${details.url}`);

      // Don't provide credentials
      callback();
    });

    // Monitor for suspicious activity
    app.on('accessibility-support-changed', (event, accessibilitySupportEnabled) => {
      this.logger.debug(`Accessibility support changed: ${accessibilitySupportEnabled}`);
    });

    // Handle GPU info update
    app.on('gpu-info-update', () => {
      this.logger.debug('GPU info updated');
    });

    // Handle renderer process crashed
    app.on('render-process-gone', (event, webContents, details) => {
      this.logger.error(`Renderer process gone: ${JSON.stringify(details, null, 2)}`);
    });

    // Handle child process gone
    app.on('child-process-gone', (event, details) => {
      this.logger.error(`Child process gone: ${JSON.stringify(details, null, 2)}`);
    });
  }

  // Method to update allowed origins dynamically
  updateAllowedOrigins(origins: string[]): void {
    // This could be used to update CSP or navigation rules dynamically
    this.logger.info(`Updating allowed origins: ${JSON.stringify(origins, null, 2)}`);
  }

  // Method to add temporary security exceptions (for development)
  addSecurityException(type: string, value: string): void {
    if (this.isDevelopment) {
      this.logger.warn(`Adding security exception - ${type}: ${value}`);
    } else {
      this.logger.error('Security exceptions are not allowed in production');
    }
  }
}

export function createEnhancedSecurityModule(isDevelopment = false) {
  return new EnhancedSecurityModule(isDevelopment);
}
