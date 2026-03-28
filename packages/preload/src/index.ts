// Import crypto utilities
import { sha256sum } from './node-crypto.js';

// Import version information
import { versions } from './versions.js';

// Import shell operations
import { shellSpawnStreaming, shellWriteStdin, shellKillProcess, open, execute } from './shell.js';

// Import filesystem operations
import { exists, readTextFile, writeTextFile, createDirectory, removeFile } from './filesystem.js';

// Import store operations
import { get, set, deleteKey, clear, has } from './store.js';

// Import path operations
import {
  appConfigDir,
  appDataDir,
  appLocalDataDir,
  appCacheDir,
  pathResolve,
  pathJoin,
  resolveResource,
} from './path.js';

// Import OS operations
import {
  getPlatform as osPlatform,
  getArch as osArch,
  getVersion as osVersion,
  getLocale as osLocale,
  getHostname as osHostname,
  getHomedir as osHomedir,
  getTmpdir as osTmpdir,
  getArgv as osArgv,
} from './os.js';

// Import notification operations
import {
  isPermissionGranted as notificationIsPermissionGranted,
  requestPermission as notificationRequestPermission,
  send as notificationSend,
  sendWithActions as notificationSendWithActions,
} from './notification.js';

// Import window operations
import {
  close as windowClose,
  requestClose as windowRequestClose,
  relaunch as windowRelaunch,
  minimize as windowMinimize,
  maximize as windowMaximize,
  hide as windowHide,
  show as windowShow,
  focus as windowFocus,
  isMaximized as windowIsMaximized,
  isMinimized as windowIsMinimized,
  isVisible as windowIsVisible,
  setTitle as windowSetTitle,
  getTitle as windowGetTitle,
  setSize as windowSetSize,
  getSize as windowGetSize,
  setPosition as windowSetPosition,
  getPosition as windowGetPosition,
} from './window.js';

// Import logging operations
import {
  trace as logTrace,
  debug as logDebug,
  info as logInfo,
  warn as logWarn,
  error as logError,
} from './logging.js';

// Import dialog operations
import {
  open as dialogOpen,
  save as dialogSave,
  message as dialogMessage,
  error as dialogError,
  confirm as dialogConfirm,
  warning as dialogWarning,
  info as dialogInfo,
} from './dialog.js';

// Import clipboard operations
import {
  writeText as clipboardWriteText,
  readText as clipboardReadText,
  clear as clipboardClear,
  writeHTML as clipboardWriteHTML,
  readHTML as clipboardReadHTML,
  writeRTF as clipboardWriteRTF,
  readRTF as clipboardReadRTF,
  writeImage as clipboardWriteImage,
  readImage as clipboardReadImage,
  writeBookmark as clipboardWriteBookmark,
  readBookmark as clipboardReadBookmark,
  availableFormats as clipboardAvailableFormats,
  has as clipboardHas,
  read as clipboardRead,
  isEmpty as clipboardIsEmpty,
  hasText as clipboardHasText,
  hasImage as clipboardHasImage,
} from './clipboard.js';

// Import context menu operations
import { show as contextMenuShow } from './context-menu.js';

// Import app menu operations
import {
  update as appMenuUpdate,
  getItems as appMenuGetItems,
  addItem as appMenuAddItem,
  removeItem as appMenuRemoveItem,
  findItem as appMenuFindItem,
  updateItem as appMenuUpdateItem,
  clear as appMenuClear,
  getItemCount as appMenuGetItemCount,
} from './app-menu.js';

// Import config operations
import { notifyChange as configNotifyChange } from './config.js';

// Import event operations
import { on as eventsOn, off as eventsOff, once as eventsOnce, emit as eventsEmit } from './events.js';

// Import HTTP operations
import { httpGet, httpPost, httpPut, httpDelete } from './http.js';

// Import Home Config operations
import { writeHomeConfig, readHomeConfig, homeConfigExists, removeHomeConfig } from './home-config.js';

// Export all functions with their original names
export {
  // Crypto
  sha256sum,

  // Versions
  versions,

  // Shell
  shellSpawnStreaming,
  shellWriteStdin,
  shellKillProcess,
  open,
  execute,

  // Filesystem
  exists,
  readTextFile,
  writeTextFile,
  createDirectory,
  removeFile,

  // Store
  get,
  set,
  deleteKey,
  clear,
  has,

  // Path
  appConfigDir,
  appDataDir,
  appLocalDataDir,
  appCacheDir,
  pathResolve,
  pathJoin,
  resolveResource,

  // OS Operations
  osPlatform,
  osArch,
  osVersion,
  osLocale,
  osHostname,
  osHomedir,
  osTmpdir,
  osArgv,

  // Notification Operations
  notificationIsPermissionGranted,
  notificationRequestPermission,
  notificationSend,
  notificationSendWithActions,

  // Window Operations
  windowClose,
  windowRequestClose,
  windowRelaunch,
  windowMinimize,
  windowMaximize,
  windowHide,
  windowShow,
  windowFocus,
  windowIsMaximized,
  windowIsMinimized,
  windowIsVisible,
  windowSetTitle,
  windowGetTitle,
  windowSetSize,
  windowGetSize,
  windowSetPosition,
  windowGetPosition,

  // Logging Operations
  logTrace,
  logDebug,
  logInfo,
  logWarn,
  logError,

  // Dialog Operations
  dialogOpen,
  dialogSave,
  dialogMessage,
  dialogError,
  dialogConfirm,
  dialogWarning,
  dialogInfo,

  // Clipboard Operations
  clipboardWriteText,
  clipboardReadText,
  clipboardClear,
  clipboardWriteHTML,
  clipboardReadHTML,
  clipboardWriteRTF,
  clipboardReadRTF,
  clipboardWriteImage,
  clipboardReadImage,
  clipboardWriteBookmark,
  clipboardReadBookmark,
  clipboardAvailableFormats,
  clipboardHas,
  clipboardRead,
  clipboardIsEmpty,
  clipboardHasText,
  clipboardHasImage,

  // Context Menu Operations
  contextMenuShow,

  // App Menu Operations
  appMenuUpdate,
  appMenuGetItems,
  appMenuAddItem,
  appMenuRemoveItem,
  appMenuFindItem,
  appMenuUpdateItem,
  appMenuClear,
  appMenuGetItemCount,

  // Config Operations
  configNotifyChange,

  // Event Operations
  eventsOn,
  eventsOff,
  eventsOnce,
  eventsEmit,

  // HTTP Operations
  httpGet,
  httpPost,
  httpPut,
  httpDelete,

  // Home Config Operations
  writeHomeConfig,
  readHomeConfig,
  homeConfigExists,
  removeHomeConfig,
};
