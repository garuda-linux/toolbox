// Cache for loaded functions to avoid repeated lookups
import type { ChildProcess } from '../types/shell';

const functionCache = new Map<string, any>();

// Check if preload functions are ready
function isPreloadReady(): boolean {
  const testFunctionName = btoa('logInfo'); // Test with a known function
  return typeof (window as any)[testFunctionName] === 'function';
}

// Wait for preload to be ready
function waitForPreload(timeout = 5000): Promise<void> {
  return new Promise((resolve, reject) => {
    if (isPreloadReady()) {
      resolve();
      return;
    }

    const startTime = Date.now();
    const checkInterval = setInterval(() => {
      if (isPreloadReady()) {
        clearInterval(checkInterval);
        resolve();
      } else if (Date.now() - startTime > timeout) {
        clearInterval(checkInterval);
        reject(new Error(`Preload functions not available after ${timeout}ms`));
      }
    }, 10);
  });
}

// Helper function to get base64 encoded function from window with caching
function getExposedFunction<T extends (...args: any[]) => any>(name: string): T {
  // Check cache first
  if (functionCache.has(name)) {
    return functionCache.get(name) as T;
  }

  const encodedName = btoa(name);
  const func = (window as any)[encodedName] as T;

  if (!func) {
    console.error(`Function '${name}' not found. Preload ready: ${isPreloadReady()}`);
    throw new Error(
      `Exposed function '${name}' not found. Make sure it's exported from preload/index.ts and that the preload script has finished loading.`,
    );
  }

  // Cache the function
  functionCache.set(name, func);
  return func;
}

// Create a deferred function wrapper that only loads the actual function when called
function createDeferredFunction<T extends (...args: any[]) => any>(name: string): T {
  return ((...args: any[]) => {
    try {
      const func = getExposedFunction<T>(name);
      return func(...args);
    } catch (error) {
      console.error(`Failed to call function '${name}':`, error);
      throw error;
    }
  }) as T;
}

// Create an async deferred function wrapper for async functions
function createAsyncDeferredFunction<T extends (...args: any[]) => Promise<any>>(name: string): T {
  return (async (...args: any[]) => {
    try {
      // Wait for preload to be ready before attempting to get the function
      await waitForPreload();
      const func = getExposedFunction<T>(name);
      return await func(...args);
    } catch (error) {
      console.error(`Failed to call async function '${name}':`, error);
      throw error;
    }
  }) as T;
}

// Export all the exposed functions using deferred loading
// Shell operations
export const shellSpawnStreaming =
  createDeferredFunction<
    (command: string, args?: string[], options?: Record<string, unknown>) => { processId: string; pid: number }
  >('shellSpawnStreaming');
export const shellWriteStdin = createDeferredFunction<(processId: string, data: string) => boolean>('shellWriteStdin');
export const shellKillProcess =
  createDeferredFunction<(processId: string, signal?: string) => boolean>('shellKillProcess');
export const open = createAsyncDeferredFunction<(url: string) => Promise<boolean>>('open');
export const execute =
  createAsyncDeferredFunction<
    (command: string, args?: string[], options?: Record<string, unknown>) => Promise<ChildProcess<string>>
  >('execute');

// Filesystem operations
export const exists =
  createAsyncDeferredFunction<(filePath: string, handleAccessDeniedAsExists?: boolean) => Promise<boolean>>('exists');
export const readTextFile = createAsyncDeferredFunction<(filePath: string) => Promise<string>>('readTextFile');
export const writeTextFile =
  createAsyncDeferredFunction<(filePath: string, contents: string) => Promise<boolean>>('writeTextFile');
export const createDirectory = createAsyncDeferredFunction<(dirPath: string) => Promise<boolean>>('createDirectory');
export const removeFile = createAsyncDeferredFunction<(filePath: string) => Promise<boolean>>('removeFile');

// Store operations
export const get = createAsyncDeferredFunction<(key: string) => Promise<unknown>>('get');
export const set = createAsyncDeferredFunction<(key: string, value: unknown) => Promise<boolean>>('set');
export const deleteKey = createAsyncDeferredFunction<(key: string) => Promise<boolean>>('deleteKey');
export const clear = createAsyncDeferredFunction<() => Promise<boolean>>('clear');
export const has = createAsyncDeferredFunction<(key: string) => Promise<boolean>>('has');

// Path operations
export const appConfigDir = createAsyncDeferredFunction<() => Promise<string>>('appConfigDir');
export const appDataDir = createAsyncDeferredFunction<() => Promise<string>>('appDataDir');
export const appLocalDataDir = createAsyncDeferredFunction<() => Promise<string>>('appLocalDataDir');
export const appCacheDir = createAsyncDeferredFunction<() => Promise<string>>('appCacheDir');
export const pathResolve = createAsyncDeferredFunction<(...paths: string[]) => Promise<string>>('pathResolve');
export const pathJoin = createAsyncDeferredFunction<(...paths: string[]) => Promise<string>>('pathJoin');
export const resolveResource =
  createAsyncDeferredFunction<(resourcePath: string) => Promise<string>>('resolveResource');

// OS operations
export const osPlatform = createDeferredFunction<() => string>('osPlatform');
export const osArch = createDeferredFunction<() => string>('osArch');
export const osVersion = createDeferredFunction<() => string>('osVersion');
export const osLocale = createDeferredFunction<() => string>('osLocale');
export const osHostname = createDeferredFunction<() => string>('osHostname');
export const osHomedir = createDeferredFunction<() => string>('osHomedir');
export const osTmpdir = createDeferredFunction<() => string>('osTmpdir');
export const osArgv = createAsyncDeferredFunction<() => Promise<string[]>>('osArgv');

// Notification operations
export const notificationIsPermissionGranted = createDeferredFunction<() => boolean>('notificationIsPermissionGranted');
export const notificationRequestPermission = createAsyncDeferredFunction<() => Promise<boolean>>(
  'notificationRequestPermission',
);
export const notificationSend =
  createDeferredFunction<(options: { title: string; body?: string; icon?: string }) => boolean>('notificationSend');
export const notificationSendWithActions =
  createDeferredFunction<
    (options: { title: string; body?: string; icon?: string; actions?: { type: string; text: string }[] }) => boolean
  >('notificationSendWithActions');

// Window operations
export const windowClose = createDeferredFunction<() => boolean>('windowClose');
export const windowRequestClose = createDeferredFunction<() => boolean>('windowRequestClose');
export const windowRelaunch = createDeferredFunction<() => boolean>('windowRelaunch');
export const windowMinimize = createDeferredFunction<() => boolean>('windowMinimize');
export const windowMaximize = createDeferredFunction<() => boolean>('windowMaximize');
export const windowHide = createDeferredFunction<() => boolean>('windowHide');
export const windowShow = createDeferredFunction<() => boolean>('windowShow');
export const windowFocus = createDeferredFunction<() => boolean>('windowFocus');
export const windowIsMaximized = createDeferredFunction<() => boolean>('windowIsMaximized');
export const windowIsMinimized = createDeferredFunction<() => boolean>('windowIsMinimized');
export const windowIsVisible = createDeferredFunction<() => boolean>('windowIsVisible');
export const windowSetTitle = createDeferredFunction<(title: string) => boolean>('windowSetTitle');
export const windowGetTitle = createDeferredFunction<() => string>('windowGetTitle');
export const windowSetSize = createDeferredFunction<(width: number, height: number) => boolean>('windowSetSize');
export const windowGetSize = createDeferredFunction<() => { width: number; height: number }>('windowGetSize');
export const windowSetPosition = createDeferredFunction<(x: number, y: number) => boolean>('windowSetPosition');
export const windowGetPosition = createDeferredFunction<() => { x: number; y: number }>('windowGetPosition');

// Logging operations
export const logTrace = createDeferredFunction<(msg: string, obj?: any) => boolean>('logTrace');
export const logDebug = createDeferredFunction<(msg: string, obj?: any) => boolean>('logDebug');
export const logInfo = createDeferredFunction<(msg: string, obj?: any) => boolean>('logInfo');
export const logWarn = createDeferredFunction<(msg: string, obj?: any) => boolean>('logWarn');
export const logError = createDeferredFunction<(msg: string, obj?: any) => boolean>('logError');

// Dialog operations
export const dialogOpen =
  createAsyncDeferredFunction<(options: Record<string, unknown>) => Promise<string[]>>('dialogOpen');
export const dialogSave =
  createAsyncDeferredFunction<(options: Record<string, unknown>) => Promise<string>>('dialogSave');
export const dialogMessage =
  createAsyncDeferredFunction<(options: Record<string, unknown>) => Promise<number>>('dialogMessage');
export const dialogError =
  createAsyncDeferredFunction<(title: string, content: string) => Promise<boolean>>('dialogError');
export const dialogCertificate =
  createAsyncDeferredFunction<(options: Record<string, unknown>) => Promise<boolean>>('dialogCertificate');
export const dialogConfirm =
  createAsyncDeferredFunction<(message: string, title?: string, detail?: string) => Promise<boolean>>('dialogConfirm');
export const dialogWarning =
  createAsyncDeferredFunction<(message: string, title?: string, detail?: string) => Promise<boolean>>('dialogWarning');
export const dialogInfo =
  createAsyncDeferredFunction<(message: string, title?: string, detail?: string) => Promise<boolean>>('dialogInfo');

// Clipboard operations
export const clipboardWriteText = createAsyncDeferredFunction<(text: string) => Promise<boolean>>('clipboardWriteText');
export const clipboardReadText = createAsyncDeferredFunction<() => Promise<string>>('clipboardReadText');
export const clipboardClear = createAsyncDeferredFunction<() => Promise<boolean>>('clipboardClear');
export const clipboardWriteHTML =
  createAsyncDeferredFunction<(markup: string, text?: string) => Promise<boolean>>('clipboardWriteHTML');
export const clipboardReadHTML = createAsyncDeferredFunction<() => Promise<string>>('clipboardReadHTML');
export const clipboardWriteRTF = createAsyncDeferredFunction<(text: string) => Promise<boolean>>('clipboardWriteRTF');
export const clipboardReadRTF = createAsyncDeferredFunction<() => Promise<string>>('clipboardReadRTF');
export const clipboardWriteImage =
  createAsyncDeferredFunction<(dataURL: string) => Promise<boolean>>('clipboardWriteImage');
export const clipboardReadImage = createAsyncDeferredFunction<() => Promise<string>>('clipboardReadImage');
export const clipboardWriteBookmark =
  createAsyncDeferredFunction<(title: string, url: string) => Promise<boolean>>('clipboardWriteBookmark');
export const clipboardReadBookmark =
  createAsyncDeferredFunction<() => Promise<{ title: string; url: string }>>('clipboardReadBookmark');
export const clipboardAvailableFormats =
  createAsyncDeferredFunction<() => Promise<string[]>>('clipboardAvailableFormats');
export const clipboardHas = createAsyncDeferredFunction<(format: string) => Promise<boolean>>('clipboardHas');
export const clipboardRead = createAsyncDeferredFunction<(format: string) => Promise<string>>('clipboardRead');
export const clipboardIsEmpty = createAsyncDeferredFunction<() => Promise<boolean>>('clipboardIsEmpty');
export const clipboardHasText = createAsyncDeferredFunction<() => Promise<boolean>>('clipboardHasText');
export const clipboardHasImage = createAsyncDeferredFunction<() => Promise<boolean>>('clipboardHasImage');

// Context menu operations
export const contextMenuShow =
  createDeferredFunction<(items: any[], x?: number, y?: number) => boolean>('contextMenuShow');

// App menu operations
export const appMenuUpdate = createAsyncDeferredFunction<(items: any[]) => Promise<boolean>>('appMenuUpdate');
export const appMenuGetItems = createAsyncDeferredFunction<() => Promise<any[]>>('appMenuGetItems');
export const appMenuAddItem =
  createAsyncDeferredFunction<(item: any, position?: number) => Promise<boolean>>('appMenuAddItem');
export const appMenuRemoveItem = createAsyncDeferredFunction<(id: string) => Promise<boolean>>('appMenuRemoveItem');
export const appMenuFindItem = createAsyncDeferredFunction<(id: string) => Promise<any | null>>('appMenuFindItem');
export const appMenuUpdateItem =
  createAsyncDeferredFunction<(id: string, updates: any) => Promise<boolean>>('appMenuUpdateItem');
export const appMenuClear = createAsyncDeferredFunction<() => Promise<boolean>>('appMenuClear');
export const appMenuGetItemCount = createAsyncDeferredFunction<() => Promise<number>>('appMenuGetItemCount');

// Config operations
export const configNotifyChange =
  createDeferredFunction<(key: string, value: unknown) => boolean>('configNotifyChange');

// Event operations
export const eventsOn =
  createDeferredFunction<(channel: string, callback: (...args: unknown[]) => void) => (() => void) | undefined>(
    'eventsOn',
  );
export const eventsOff =
  createDeferredFunction<(channel: string, listener: (...args: unknown[]) => void) => boolean>('eventsOff');
export const eventsOnce =
  createDeferredFunction<(channel: string, listener: (...args: unknown[]) => void) => boolean>('eventsOnce');

// HTTP operations
export const httpGet = createAsyncDeferredFunction<
  (
    url: string,
    config?: { headers?: Record<string, string>; timeout?: number },
  ) => Promise<{
    data: unknown;
    status: number;
    statusText: string;
    headers: Record<string, string>;
  }>
>('httpGet');

export const httpPost = createAsyncDeferredFunction<
  (
    url: string,
    body?: unknown,
    config?: { headers?: Record<string, string>; timeout?: number },
  ) => Promise<{
    data: unknown;
    status: number;
    statusText: string;
    headers: Record<string, string>;
  }>
>('httpPost');

export const httpPut = createAsyncDeferredFunction<
  (
    url: string,
    body?: unknown,
    config?: { headers?: Record<string, string>; timeout?: number },
  ) => Promise<{
    data: unknown;
    status: number;
    statusText: string;
    headers: Record<string, string>;
  }>
>('httpPut');

export const httpDelete = createAsyncDeferredFunction<
  (
    url: string,
    config?: { headers?: Record<string, string>; timeout?: number },
  ) => Promise<{
    data: unknown;
    status: number;
    statusText: string;
    headers: Record<string, string>;
  }>
>('httpDelete');

// Home Config operations
export const writeHomeConfig =
  createAsyncDeferredFunction<
    (
      relativePath: string,
      content: string,
      createDir?: boolean,
    ) => Promise<{ success: boolean; path?: string; error?: string }>
  >('writeHomeConfig');
export const readHomeConfig = createAsyncDeferredFunction<
  (relativePath: string) => Promise<{
    success: boolean;
    content?: string;
    path?: string;
    exists?: boolean;
    error?: string;
  }>
>('readHomeConfig');
export const homeConfigExists =
  createAsyncDeferredFunction<(relativePath: string) => Promise<{ exists: boolean; path?: string; error?: string }>>(
    'homeConfigExists',
  );
export const removeHomeConfig =
  createAsyncDeferredFunction<(relativePath: string) => Promise<{ success: boolean; error?: string }>>(
    'removeHomeConfig',
  );

// Export utility functions for checking preload status
export { isPreloadReady, waitForPreload };
