import { Service } from '@angular/core';
import type { AppMenuItem } from './electron-types';
import { Logger } from '../logging/logging';
import { appMenuGetItems, appMenuUpdate, eventsOn } from './electron-api-utils.js';

@Service()
export class ElectronAppMenuService {
  private readonly logger = Logger.getInstance();
  private readonly menuClickHandlers = new Map<string, () => undefined | Promise<void>>();

  constructor() {
    this.setupEventListeners();
  }

  /**
   * Updates the application menu with the provided menu items
   * @param items Array of menu items to set as the application menu
   * @returns Promise that resolves to true if successful
   */
  async updateAppMenu(items: AppMenuItem[]): Promise<boolean> {
    try {
      // Register click handlers for items with commands
      this.registerMenuItemHandlers(items);

      const result = await appMenuUpdate(items);
      this.logger.trace(`App menu update result from main process: ${result}`);
      return result;
    } catch (error) {
      this.logger.error(
        `Failed to update application menu:  ${error instanceof Error ? error.message : String(error)}`,
      );
      return false;
    }
  }

  /**
   * Gets the current application menu items
   * @returns Promise that resolves to the current menu items
   */
  async getAppMenuItems(): Promise<AppMenuItem[]> {
    try {
      return appMenuGetItems();
    } catch (error) {
      this.logger.error(
        `Failed to get application menu items: ${error instanceof Error ? error.message : String(error)}`,
      );
      return [];
    }
  }

  /**
   * Registers a click handler for a specific menu item
   * @param id Menu item ID
   * @param handler Function to execute when the menu item is clicked
   */
  registerMenuItemHandler(id: string, handler: () => undefined | Promise<void>): void {
    this.menuClickHandlers.set(id, handler);
    this.logger.trace(`Registered menu item handler for: ${id}`);
  }

  /**
   * Unregisters a click handler for a specific menu item
   * @param id Menu item ID
   */
  unregisterMenuItemHandler(id: string): void {
    this.menuClickHandlers.delete(id);
    this.logger.trace(`Unregistered menu item handler for: ${id}`);
  }

  /**
   * Creates a menu item with standardized properties
   * @param options Menu item configuration
   * @returns Configured AppMenuItem
   */
  createMenuItem(options: {
    id: string;
    label: string;
    icon?: string;
    enabled?: boolean;
    visible?: boolean;
    type?: 'normal' | 'separator' | 'submenu' | 'checkbox' | 'radio';
    checked?: boolean;
    accelerator?: string;
    role?: string;
    routerLink?: string;
    onClick?: () => undefined | Promise<void>;
    submenu?: AppMenuItem[];
    items?: AppMenuItem[];
  }): AppMenuItem {
    const { onClick, ...menuItem } = options;

    if (onClick) {
      this.registerMenuItemHandler(options.id, onClick);
    }

    return menuItem;
  }

  /**
   * Registers handlers for menu items that have click actions
   * @param items Menu items to register handlers for
   * @private
   */
  private registerMenuItemHandlers(items: AppMenuItem[]): void {
    for (const item of items) {
      if (item.submenu) {
        this.registerMenuItemHandlers(item.submenu);
      }
      if (item.items) {
        this.registerMenuItemHandlers(item.items);
      }
    }
  }

  /**
   * Sets up event listeners for menu item clicks from main process
   * @private
   */
  private setupEventListeners(): void {
    this.logger.trace('Setting up app menu event listeners');

    eventsOn('appMenu:itemClicked', (...args: unknown[]) => {
      this.logger.debug(`App menu item clicked event received: ${args}`);
      const data = args[0] as any;
      this.logger.trace(`App menu service received click event: ${JSON.stringify(data)}`);

      try {
        // Handle router navigation
        if (data.routerLink) {
          // This would be handled by the component using this service
          // since the service doesn't have direct access to the router
          this.logger.trace(`Menu item has router link: ${data.routerLink}`);
        }

        // Handle command execution
        if (data.command) {
          this.logger.trace(`Menu item has command: ${data.command}`);
          // Commands would be handled by the component
        }

        // Handle registered click handlers
        if (data.id && this.menuClickHandlers.has(data.id)) {
          this.logger.trace(`Executing registered handler for: ${data.id}`);
          const handler = this.menuClickHandlers.get(data.id);
          if (handler) {
            Promise.resolve(handler()).catch((error) => {
              this.logger.error(
                `Error executing menu item handler for ${data.id}: ${error instanceof Error ? error.message : String(error)}`,
              );
            });
          }
        } else if (data.id) {
          this.logger.warn(`No registered handler found for menu item: ${data.id}`);
        }
      } catch (error) {
        this.logger.error(`Error handling menu item click: ${error instanceof Error ? error.message : String(error)}`);
      }
    });

    this.logger.debug('App menu event listeners setup complete');
  }
}
