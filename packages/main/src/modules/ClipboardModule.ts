import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { clipboard, ipcMain, nativeImage } from 'electron';
import { Logger } from '../logging/logging.js';

class ClipboardModule implements AppModule {
  private readonly logger = Logger.getInstance();

  enable({ app: _app }: ModuleContext): void {
    this.setupClipboardHandlers();
  }

  private setupClipboardHandlers(): void {
    // Basic clipboard operations
    ipcMain.handle('clipboard:writeText', async (_, text: string) => {
      try {
        if (typeof text !== 'string') {
          throw new Error('Text must be a string');
        }
        clipboard.writeText(text);
        return true;
      } catch (error: any) {
        this.logger.error(`Clipboard writeText error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to write text to clipboard: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('clipboard:readText', async () => {
      try {
        return clipboard.readText();
      } catch (error: any) {
        this.logger.error(`Clipboard readText error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to read text from clipboard: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('clipboard:clear', async () => {
      try {
        clipboard.clear();
        return true;
      } catch (error: any) {
        this.logger.error(`Clipboard clear error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to clear clipboard: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    // HTML operations
    ipcMain.handle('clipboard:writeHTML', async (_, markup: string, _text?: string) => {
      try {
        if (typeof markup !== 'string') {
          throw new Error('Markup must be a string');
        }
        clipboard.writeHTML(markup);
        return true;
      } catch (error: any) {
        this.logger.error(`Clipboard writeHTML error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to write HTML to clipboard: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('clipboard:readHTML', async () => {
      try {
        return clipboard.readHTML();
      } catch (error: any) {
        this.logger.error(`Clipboard readHTML error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to read HTML from clipboard: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    // RTF operations
    ipcMain.handle('clipboard:writeRTF', async (_, text: string) => {
      try {
        if (typeof text !== 'string') {
          throw new Error('RTF text must be a string');
        }
        clipboard.writeRTF(text);
        return true;
      } catch (error: any) {
        this.logger.error(`Clipboard writeRTF error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to write RTF to clipboard: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('clipboard:readRTF', async () => {
      try {
        return clipboard.readRTF();
      } catch (error: any) {
        this.logger.error(`Clipboard readRTF error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to read RTF from clipboard: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    // Image operations
    ipcMain.handle('clipboard:writeImage', async (_, dataURL: string) => {
      try {
        if (typeof dataURL !== 'string') {
          throw new Error('Image data URL must be a string');
        }

        const image = nativeImage.createFromDataURL(dataURL);
        if (image.isEmpty()) {
          throw new Error('Invalid image data');
        }

        clipboard.writeImage(image);
        return true;
      } catch (error: any) {
        this.logger.error(`Clipboard writeImage error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to write image to clipboard: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('clipboard:readImage', async () => {
      try {
        const image = clipboard.readImage();
        if (image.isEmpty()) {
          return null;
        }
        return image.toDataURL();
      } catch (error: any) {
        this.logger.error(`Clipboard readImage error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to read image from clipboard: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    // Bookmark operations (macOS only)
    ipcMain.handle('clipboard:writeBookmark', async (_, title: string, url: string) => {
      try {
        if (process.platform !== 'darwin') {
          this.logger.warn('Bookmark operations are only supported on macOS');
          return false;
        }

        if (typeof title !== 'string' || typeof url !== 'string') {
          throw new Error('Title and URL must be strings');
        }

        clipboard.writeBookmark(title, url);
        return true;
      } catch (error: any) {
        this.logger.error(`Clipboard writeBookmark error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to write bookmark to clipboard: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('clipboard:readBookmark', async () => {
      try {
        if (process.platform !== 'darwin') {
          this.logger.warn('Bookmark operations are only supported on macOS');
          return null;
        }

        return clipboard.readBookmark();
      } catch (error: any) {
        this.logger.error(`Clipboard readBookmark error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to read bookmark from clipboard: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    // Advanced operations
    ipcMain.handle('clipboard:availableFormats', async () => {
      try {
        return clipboard.availableFormats();
      } catch (error: any) {
        this.logger.error(
          `Clipboard availableFormats error: ${error instanceof Error ? error.message : String(error)}`,
        );
        throw new Error(
          `Failed to get available clipboard formats: ${error instanceof Error ? error.message : error}`,
          { cause: error },
        );
      }
    });

    ipcMain.handle('clipboard:has', async (_, format: string) => {
      try {
        if (typeof format !== 'string') {
          throw new Error('Format must be a string');
        }
        return clipboard.has(format);
      } catch (error: any) {
        this.logger.error(`Clipboard has error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to check clipboard format: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('clipboard:read', async (_, format: string) => {
      try {
        if (typeof format !== 'string') {
          throw new Error('Format must be a string');
        }
        return clipboard.read(format);
      } catch (error: any) {
        this.logger.error(`Clipboard read error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to read clipboard format: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    // Utility functions
    ipcMain.handle('clipboard:isEmpty', async () => {
      try {
        const formats = clipboard.availableFormats();
        return formats.length === 0;
      } catch (error: any) {
        this.logger.error(`Clipboard isEmpty error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to check if clipboard is empty: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('clipboard:hasText', async () => {
      try {
        const text = clipboard.readText();
        return text.length > 0;
      } catch (error: any) {
        this.logger.error(`Clipboard hasText error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to check if clipboard has text: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });

    ipcMain.handle('clipboard:hasImage', async () => {
      try {
        const image = clipboard.readImage();
        return !image.isEmpty();
      } catch (error: any) {
        this.logger.error(`Clipboard hasImage error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`Failed to check if clipboard has image: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });
  }
}

export function createClipboardModule() {
  return new ClipboardModule();
}
