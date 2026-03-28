import { Injectable } from '@angular/core';
import {
  osPlatform,
  osArch,
  osVersion,
  osLocale,
  osHostname,
  osHomedir,
  osTmpdir,
  osArgv,
} from './electron-api-utils.js';

@Injectable({
  providedIn: 'root',
})
export class ElectronOsService {
  async platform(): Promise<string> {
    return osPlatform();
  }

  async arch(): Promise<string> {
    return osArch();
  }

  async version(): Promise<string> {
    return osVersion();
  }

  async locale(): Promise<string> {
    return osLocale();
  }

  async hostname(): Promise<string> {
    return osHostname();
  }

  async homedir(): Promise<string> {
    return osHomedir();
  }

  async tmpdir(): Promise<string> {
    return osTmpdir();
  }

  async argv(): Promise<string[]> {
    return osArgv();
  }
}

// Standalone functions for direct use
export async function platform(): Promise<string> {
  return osPlatform();
}

export async function arch(): Promise<string> {
  return osArch();
}

export async function version(): Promise<string> {
  return osVersion();
}

export async function locale(): Promise<string> {
  return osLocale();
}

export async function hostname(): Promise<string> {
  return osHostname();
}

export async function homedir(): Promise<string> {
  return osHomedir();
}

export async function tmpdir(): Promise<string> {
  return osTmpdir();
}

export async function argv(): Promise<string[]> {
  return osArgv();
}
