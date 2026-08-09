import { Service } from '@angular/core';
import {
  isPermissionGranted,
  type NotificationOptions,
  requestPermission,
  sendNotification,
} from '../../electron-services';
import { Logger } from '../../logging/logging';

@Service()
export class NotificationService {
  private permissionGranted = false;
  private readonly logger = Logger.getInstance();

  constructor() {
    void this.init();
  }

  async init(): Promise<void> {
    const permissionGranted: boolean = await isPermissionGranted();
    if (!permissionGranted) {
      this.logger.info('Requesting notification permission');
      this.permissionGranted = await requestPermission();
    } else {
      this.logger.debug('Notification permission already granted');
      this.permissionGranted = true;
    }
  }

  /**
   * Send a notification to the user.
   * @param options The options for the notification.
   */
  async sendNotification(options: NotificationOptions): Promise<void> {
    if (!this.permissionGranted) {
      this.logger.warn('Notification permission not granted');
      return;
    }

    await sendNotification(options);
  }
}
