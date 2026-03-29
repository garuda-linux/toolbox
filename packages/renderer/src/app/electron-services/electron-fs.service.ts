import { Injectable } from '@angular/core';
import { Logger } from '../logging/logging';
import { createDirectory, exists, readTextFile, removeFile, writeTextFile } from './electron-api-utils.js';

@Injectable({
  providedIn: 'root',
})
export class ElectronFsService {
  private readonly logger = Logger.getInstance();

  async exists(filePath: string, handleAccessDeniedAsExists?: boolean): Promise<boolean> {
    return await exists(filePath, handleAccessDeniedAsExists);
  }

  async readTextFile(filePath: string): Promise<string> {
    try {
      const content = await readTextFile(filePath);

      // Validate content is not null/undefined
      if (content === null || content === undefined) {
        throw new Error(`File content is null or undefined: ${filePath}`);
      }

      return content;
    } catch (error) {
      // Re-throw with more context
      if (error instanceof Error) {
        throw new Error(`Failed to read file ${filePath}: ${error.message}`, { cause: error });
      }
      throw new Error(`Failed to read file ${filePath}: ${error}`, { cause: error });
    }
  }

  async readJsonFile<T = unknown>(filePath: string): Promise<T> {
    try {
      const content = await this.readTextFile(filePath);

      // Validate JSON content before parsing
      const trimmedContent = content.trim();
      if (!trimmedContent) {
        throw new Error(`File is empty: ${filePath}`);
      }

      // Check if content looks like JSON
      if (!trimmedContent.startsWith('{') && !trimmedContent.startsWith('[')) {
        throw new Error(`File does not contain valid JSON format: ${filePath}`);
      }

      return JSON.parse(content) as T;
    } catch (error) {
      if (error instanceof SyntaxError) {
        throw new Error(`Invalid JSON in file ${filePath}: ${error.message}`, { cause: error });
      }
      throw error;
    }
  }

  async safeReadJsonFile<T = unknown>(filePath: string, defaultValue: T): Promise<T> {
    try {
      return await this.readJsonFile<T>(filePath);
    } catch (error) {
      this.logger.warn(`Could not read JSON file ${filePath}, using default value: ${error}`);
      return defaultValue;
    }
  }

  async writeTextFile(filePath: string, contents: string): Promise<boolean> {
    return await writeTextFile(filePath, contents);
  }

  async createDirectory(dirPath: string): Promise<boolean> {
    return await createDirectory(dirPath);
  }

  async removeFile(filePath: string): Promise<boolean> {
    return await removeFile(filePath);
  }
}
