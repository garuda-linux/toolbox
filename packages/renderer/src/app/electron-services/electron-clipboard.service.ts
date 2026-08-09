import { Service } from '@angular/core';
import {
  clipboardAvailableFormats,
  clipboardClear,
  clipboardHas,
  clipboardHasImage,
  clipboardHasText,
  clipboardIsEmpty,
  clipboardRead,
  clipboardReadBookmark,
  clipboardReadHTML,
  clipboardReadImage,
  clipboardReadRTF,
  clipboardReadText,
  clipboardWriteBookmark,
  clipboardWriteHTML,
  clipboardWriteImage,
  clipboardWriteRTF,
  clipboardWriteText,
} from './electron-api-utils.js';

@Service()
export class ElectronClipboardService {
  async writeText(text: string): Promise<boolean> {
    return await clipboardWriteText(text);
  }

  async readText(): Promise<string> {
    return await clipboardReadText();
  }

  async clear(): Promise<boolean> {
    return await clipboardClear();
  }

  async writeHTML(markup: string, text?: string): Promise<boolean> {
    return await clipboardWriteHTML(markup, text);
  }

  async readHTML(): Promise<string> {
    return await clipboardReadHTML();
  }

  async writeRTF(text: string): Promise<boolean> {
    return await clipboardWriteRTF(text);
  }

  async readRTF(): Promise<string> {
    return await clipboardReadRTF();
  }

  async writeImage(dataURL: string): Promise<boolean> {
    return await clipboardWriteImage(dataURL);
  }

  async readImage(): Promise<string> {
    return await clipboardReadImage();
  }

  async writeBookmark(title: string, url: string): Promise<boolean> {
    return await clipboardWriteBookmark(title, url);
  }

  async readBookmark(): Promise<{ title: string; url: string }> {
    return await clipboardReadBookmark();
  }

  async availableFormats(): Promise<string[]> {
    return await clipboardAvailableFormats();
  }

  async has(format: string): Promise<boolean> {
    return await clipboardHas(format);
  }

  async read(format: string): Promise<string> {
    return await clipboardRead(format);
  }

  async isEmpty(): Promise<boolean> {
    return await clipboardIsEmpty();
  }

  async hasText(): Promise<boolean> {
    return await clipboardHasText();
  }

  async hasImage(): Promise<boolean> {
    return await clipboardHasImage();
  }
}

// Standalone functions for direct use
export async function writeText(text: string): Promise<boolean> {
  return await clipboardWriteText(text);
}

export async function readText(): Promise<string> {
  return await clipboardReadText();
}

export async function clear(): Promise<boolean> {
  return await clipboardClear();
}

export async function writeHTML(markup: string, text?: string): Promise<boolean> {
  return await clipboardWriteHTML(markup, text);
}

export async function readHTML(): Promise<string> {
  return await clipboardReadHTML();
}

export async function writeRTF(text: string): Promise<boolean> {
  return await clipboardWriteRTF(text);
}

export async function readRTF(): Promise<string> {
  return await clipboardReadRTF();
}

export async function writeImage(dataURL: string): Promise<boolean> {
  return await clipboardWriteImage(dataURL);
}

export async function readImage(): Promise<string> {
  return await clipboardReadImage();
}

export async function writeBookmark(title: string, url: string): Promise<boolean> {
  return await clipboardWriteBookmark(title, url);
}

export async function readBookmark(): Promise<{ title: string; url: string }> {
  return await clipboardReadBookmark();
}

export async function availableFormats(): Promise<string[]> {
  return await clipboardAvailableFormats();
}

export async function has(format: string): Promise<boolean> {
  return await clipboardHas(format);
}

export async function read(format: string): Promise<string> {
  return await clipboardRead(format);
}

export async function isEmpty(): Promise<boolean> {
  return await clipboardIsEmpty();
}

export async function hasText(): Promise<boolean> {
  return await clipboardHasText();
}

export async function hasImage(): Promise<boolean> {
  return await clipboardHasImage();
}
