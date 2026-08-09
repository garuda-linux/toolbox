import { Service } from '@angular/core';
import {
  notificationIsPermissionGranted,
  notificationRequestPermission,
  notificationSend,
  notificationSendWithActions,
} from './electron-api-utils.js';

export interface NotificationOptions {
  title: string;
  body?: string;
  icon?: string;
  actions?: { type: string; text: string }[];
}

@Service()
export class ElectronNotificationService {
  async isPermissionGranted(): Promise<boolean> {
    return notificationIsPermissionGranted();
  }

  async requestPermission(): Promise<boolean> {
    return await notificationRequestPermission();
  }

  async sendNotification(options: NotificationOptions): Promise<boolean> {
    if (options.actions && options.actions.length > 0) {
      return notificationSendWithActions(options);
    }
    return notificationSend(options);
  }

  async sendNotificationWithActions(options: NotificationOptions): Promise<boolean> {
    return notificationSendWithActions(options);
  }
}

// Standalone functions for direct use
export async function isPermissionGranted(): Promise<boolean> {
  return notificationIsPermissionGranted();
}

export async function requestPermission(): Promise<boolean> {
  return await notificationRequestPermission();
}

export async function sendNotification(options: NotificationOptions): Promise<boolean> {
  if (options.actions && options.actions.length > 0) {
    return notificationSendWithActions(options);
  }
  return notificationSend(options);
}

export async function sendNotificationWithActions(options: NotificationOptions): Promise<boolean> {
  return notificationSendWithActions(options);
}
