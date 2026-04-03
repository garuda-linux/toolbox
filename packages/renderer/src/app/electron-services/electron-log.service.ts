import { Injectable } from '@angular/core';
import { get, logDebug, logError, logInfo, logTrace, logWarn } from './electron-api-utils.js';

export enum LogLevel {
  Trace = 0,
  Debug = 1,
  Info = 2,
  Warn = 3,
  Error = 4,
}

@Injectable({
  providedIn: 'root',
})
export class ElectronLogService {
  private currentLogLevel: LogLevel = LogLevel.Info;

  constructor() {
    void this.initialize();
  }

  private async initialize() {
    try {
      const level = (await get('logLevel')) as number;
      if (level !== undefined) {
        this.currentLogLevel = level as LogLevel;
      }
    } catch {
      // Ignore
    }
  }

  setLogLevel(level: LogLevel) {
    this.currentLogLevel = level;
  }

  async trace(message: string): Promise<void> {
    if (this.currentLogLevel <= LogLevel.Trace) {
      logTrace(message);
    }
  }

  async debug(message: string): Promise<void> {
    if (this.currentLogLevel <= LogLevel.Debug) {
      logDebug(message);
    }
  }

  async info(message: string): Promise<void> {
    if (this.currentLogLevel <= LogLevel.Info) {
      logInfo(message);
    }
  }

  async warn(message: string): Promise<void> {
    if (this.currentLogLevel <= LogLevel.Warn) {
      logWarn(message);
    }
  }

  async error(message: string): Promise<void> {
    if (this.currentLogLevel <= LogLevel.Error) {
      logError(message);
    }
  }
}

let globalLogLevel = LogLevel.Info;
void (async () => {
  try {
    const level = (await get('logLevel')) as number;
    if (level !== undefined) globalLogLevel = level as LogLevel;
  } catch {
    /* ignore */
  }
})();

export async function trace(message: string): Promise<void> {
  if (globalLogLevel <= LogLevel.Trace) logTrace(message);
}

export async function debug(message: string): Promise<void> {
  if (globalLogLevel <= LogLevel.Debug) logDebug(message);
}

export async function info(message: string): Promise<void> {
  if (globalLogLevel <= LogLevel.Info) logInfo(message);
}

export async function warn(message: string): Promise<void> {
  if (globalLogLevel <= LogLevel.Warn) logWarn(message);
}

export async function error(message: string): Promise<void> {
  if (globalLogLevel <= LogLevel.Error) logError(message);
}
