import { error, warn, trace } from './logging.js';
import { ipcRenderer } from 'electron';

// Event listener storage
type EventCallback = (...args: unknown[]) => void;
const eventListeners = new Map<string, Set<EventCallback>>();

// Valid event channels
const validChannels = [
  'window-focus',
  'window-blur',
  'window-maximize',
  'window-unmaximize',
  'window-minimize',
  'window-restore',
  'app-update',
  'system-theme-changed',
  'shell:stdout',
  'shell:stderr',
  'shell:close',
  'shell:error',
  'contextMenu:itemClicked',
  'appMenu:itemClicked',
];

// Browser event mappings
const browserEventMappings: Record<string, string> = {
  'window-focus': 'focus',
  'window-blur': 'blur',
  'system-theme-changed': 'change', // for media query changes
};

// Track browser event listeners to avoid duplicates
const browserListeners = new Map<string, EventListener>();

export function on(channel: string, callback: EventCallback): (() => void) | undefined {
  try {
    if (!channel || typeof channel !== 'string') {
      error('Event channel must be a non-empty string');
      return undefined;
    }

    if (typeof callback !== 'function') {
      error('Event callback must be a function');
      return undefined;
    }

    if (!validChannels.includes(channel)) {
      warn(`Invalid event channel: ${channel}`);
      return undefined;
    }

    // Initialize listener set if not exists
    if (!eventListeners.has(channel)) {
      eventListeners.set(channel, new Set());
    }

    const listeners = eventListeners.get(channel);
    if (!listeners) {
      return undefined;
    }
    listeners.add(callback);

    // Set up browser event listener if needed
    setupBrowserEventListener(channel);

    trace(`Event listener added for channel: ${channel}`);

    // Return unsubscribe function
    return () => {
      listeners.delete(callback);
      if (listeners.size === 0) {
        eventListeners.delete(channel);
        cleanupBrowserEventListener(channel);
      }
    };
  } catch (err) {
    error(`Events on error: ${err instanceof Error ? err.message : String(err)}`);
    return undefined;
  }
}

export function off(channel: string, listener: EventCallback): boolean {
  try {
    if (!channel || typeof channel !== 'string') {
      error('Event channel must be a non-empty string');
      return false;
    }

    if (typeof listener !== 'function') {
      error('Event listener must be a function');
      return false;
    }

    const listeners = eventListeners.get(channel);
    if (!listeners) {
      return false;
    }

    const removed = listeners.delete(listener);

    if (listeners.size === 0) {
      eventListeners.delete(channel);
      cleanupBrowserEventListener(channel);
    }

    if (removed) {
      trace(`Event listener removed for channel: ${channel}`);
    }

    return removed;
  } catch (err) {
    error(`Events off error: ${err instanceof Error ? err.message : String(err)}`);
    return false;
  }
}

export function once(channel: string, listener: EventCallback): boolean {
  try {
    if (!channel || typeof channel !== 'string') {
      error('Event channel must be a non-empty string');
      return false;
    }

    if (typeof listener !== 'function') {
      error('Event listener must be a function');
      return false;
    }

    if (!validChannels.includes(channel)) {
      warn(`Invalid event channel: ${channel}`);
      return false;
    }

    // Create a wrapper that removes itself after first call
    const onceWrapper = (...args: unknown[]) => {
      off(channel, onceWrapper);
      listener(...args);
    };

    const unsubscribe = on(channel, onceWrapper);
    return unsubscribe !== undefined;
  } catch (err) {
    error(`Events once error: ${err instanceof Error ? err.message : String(err)}`);
    return false;
  }
}

export function emit(channel: string, ...args: unknown[]): boolean {
  try {
    if (!channel || typeof channel !== 'string') {
      error('Event channel must be a non-empty string');
      return false;
    }

    const listeners = eventListeners.get(channel);
    if (!listeners || listeners.size === 0) {
      return false;
    }

    // Call all listeners
    for (const listener of listeners) {
      try {
        listener(...args);
      } catch (listenerError) {
        error(
          `Event listener error: ${listenerError instanceof Error ? listenerError.message : String(listenerError)}`,
        );
      }
    }

    trace(`Event emitted for channel: ${channel} with ${args.length} arguments`);
    return true;
  } catch (err) {
    error(`Events emit error: ${err instanceof Error ? err.message : String(err)}`);
    return false;
  }
}

export function removeAllListeners(channel?: string): boolean {
  try {
    if (channel) {
      if (typeof channel !== 'string') {
        error('Event channel must be a string');
        return false;
      }

      eventListeners.delete(channel);
      cleanupBrowserEventListener(channel);
      trace(`All listeners removed for channel: ${channel}`);
    } else {
      // Remove all listeners for all channels
      const channels = Array.from(eventListeners.keys());
      eventListeners.clear();

      // Cleanup all browser event listeners
      for (const ch of channels) {
        cleanupBrowserEventListener(ch);
      }

      trace('All event listeners removed');
    }

    return true;
  } catch (err) {
    error(`Events removeAllListeners error: ${err instanceof Error ? err.message : String(err)}`);
    return false;
  }
}

export function listenerCount(channel: string): number {
  try {
    if (!channel || typeof channel !== 'string') {
      error('Event channel must be a non-empty string');
      return 0;
    }

    const listeners = eventListeners.get(channel);
    return listeners ? listeners.size : 0;
  } catch (err) {
    error(`Events listenerCount error: ${err instanceof Error ? err.message : String(err)}`);
    return 0;
  }
}

export function eventNames(): string[] {
  try {
    return Array.from(eventListeners.keys());
  } catch (err) {
    error(`Events eventNames error: ${err instanceof Error ? err.message : String(err)}`);
    return [];
  }
}

function setupBrowserEventListener(channel: string): void {
  try {
    const browserEvent = browserEventMappings[channel];
    if (!browserEvent || browserListeners.has(channel)) {
      return;
    }

    let target: EventTarget;
    let eventListener: EventListener;

    switch (channel) {
      case 'window-focus':
      case 'window-blur':
        target = window;
        eventListener = () => emit(channel);
        break;

      case 'system-theme-changed': {
        // Listen for prefers-color-scheme changes
        const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        eventListener = () => emit(channel, mediaQuery.matches ? 'dark' : 'light');
        target = mediaQuery as EventTarget; // MediaQueryList implements EventTarget
        break;
      }

      default:
        return;
    }

    target.addEventListener(browserEvent, eventListener);
    browserListeners.set(channel, eventListener);

    trace(`Browser event listener set up for channel: ${channel}`);
  } catch (err) {
    error(`Setup browser event listener error: ${err instanceof Error ? err.message : String(err)}`);
  }
}

function cleanupBrowserEventListener(channel: string): void {
  try {
    const browserEvent = browserEventMappings[channel];
    const listener = browserListeners.get(channel);

    if (!browserEvent || !listener) {
      return;
    }

    let target: EventTarget;

    switch (channel) {
      case 'window-focus':
      case 'window-blur':
        target = window;
        break;

      case 'system-theme-changed':
        target = window.matchMedia('(prefers-color-scheme: dark)') as EventTarget;
        break;

      default:
        return;
    }

    target.removeEventListener(browserEvent, listener);
    browserListeners.delete(channel);

    trace(`Browser event listener cleaned up for channel: ${channel}`);
  } catch (err) {
    error(`Cleanup browser event listener error: ${err instanceof Error ? err.message : String(err)}`);
  }
}

// Initialize system theme change listener if supported
if (typeof window !== 'undefined' && window.matchMedia) {
  try {
    // Initial setup will be done when first listener is added
    window.matchMedia('(prefers-color-scheme: dark)');
  } catch (err) {
    warn(`Failed to initialize media query listener: ${err instanceof Error ? err.message : String(err)}`);
  }
}

// Set up IPC listener for app menu clicks - forward to events system
ipcRenderer.on('appMenu:itemClicked', (_, clickData: any) => {
  emit('appMenu:itemClicked', clickData);
});

ipcRenderer.on('events:emit', (_, channel: string, ...args: unknown[]) => {
  emit(channel, ...args);
});

// Clean up on page unload
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    removeAllListeners();
  });
}
