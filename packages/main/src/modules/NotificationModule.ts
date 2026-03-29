import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { ipcMain, Notification } from 'electron';
import { Logger } from '../logging/logging.js';

class NotificationModule implements AppModule {
  private readonly logger = Logger.getInstance();

  enable({ app: _app }: ModuleContext): void {
    this.setupNotificationHandlers();
  }

  private setupNotificationHandlers(): void {
    // Notification Operations
    ipcMain.handle('notification:isPermissionGranted', async () => {
      try {
        return true;
      } catch (error: any) {
        this.logger.error(
          `Notification permission check error: ${error instanceof Error ? error.message : String(error)}`,
        );
        return false;
      }
    });

    ipcMain.handle('notification:requestPermission', async () => {
      try {
        return 'granted';
      } catch (error: any) {
        this.logger.error(
          `Notification permission request error: ${error instanceof Error ? error.message : String(error)}`,
        );
        return 'denied';
      }
    });

    ipcMain.handle('notification:send', async (_, options: { title: string; body?: string; icon?: string }) => {
      try {
        if (!Notification.isSupported()) {
          this.logger.warn('Notifications are not supported on this system');
          return false;
        }

        if (!options.title) {
          throw new Error('Notification title is required');
        }

        const notification = new Notification({
          title: options.title,
          body: options.body,
          icon: options.icon,
          silent: false,
        });

        notification.show();

        // Handle notification events
        notification.on('click', () => {
          this.logger.debug('Notification clicked');
        });

        notification.on('close', () => {
          this.logger.debug('Notification closed');
        });

        notification.on('failed', (error: any) => {
          this.logger.error(`Notification failed: ${error instanceof Error ? error.message : String(error)}`);
        });

        return true;
      } catch (error: any) {
        this.logger.error(`Notification send error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to send notification: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle(
      'notification:sendWithActions',
      async (
        _,
        options: {
          title: string;
          body?: string;
          icon?: string;
          actions?: { type: string; text: string }[];
        },
      ) => {
        try {
          if (!Notification.isSupported()) {
            this.logger.warn('Notifications are not supported on this system');
            return false;
          }

          if (!options.title) {
            throw new Error('Notification title is required');
          }

          const notification = new Notification({
            title: options.title,
            body: options.body,
            icon: options.icon,
            silent: false,
          });

          notification.show();

          // Return a promise that resolves when the notification is interacted with
          return new Promise((resolve) => {
            notification.on('click', () => {
              resolve({ action: 'click' });
            });

            notification.on('action', (event, index) => {
              resolve({ action: 'action', index });
            });

            notification.on('close', () => {
              resolve({ action: 'close' });
            });

            notification.on('failed', (error: any) => {
              this.logger.error(`Notification failed: ${error instanceof Error ? error.message : String(error)}`);
              resolve({ action: 'failed', error: 'Notification failed' });
            });
          });
        } catch (error: any) {
          this.logger.error(
            `Notification with actions send error: ${error instanceof Error ? error.message : String(error)}`,
          );
          throw new Error(
            `Failed to send notification with actions: ${error instanceof Error ? error.message : error}`,
            { cause: error },
          );
        }
      },
    );
  }
}

export function createNotificationModule() {
  return new NotificationModule();
}
