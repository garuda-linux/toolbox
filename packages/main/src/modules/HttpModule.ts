import type { AppModule } from '../AppModule.js';
import type { ModuleContext } from '../ModuleContext.js';
import { ipcMain } from 'electron';
import { Logger } from '../logging/logging.js';

export interface HttpRequestConfig {
  headers?: Record<string, string>;
  timeout?: number;
}

export interface HttpResponse<T = any> {
  data: T;
  status: number;
  statusText: string;
  headers: Record<string, string>;
}

class HttpModule implements AppModule {
  private readonly logger = Logger.getInstance();

  enable({ app: _app }: ModuleContext): void {
    this.setupHttpHandlers();
  }

  private setupHttpHandlers(): void {
    // HTTP GET handler
    ipcMain.handle('http:get', async (_, url: string, config?: HttpRequestConfig): Promise<HttpResponse> => {
      try {
        if (!url || typeof url !== 'string') {
          throw new Error('URL is required and must be a string');
        }

        this.logger.debug(`HTTP GET request to: ${url}`);

        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), config?.timeout || 30000);

        const response = await fetch(url, {
          method: 'GET',
          headers: {
            'Content-Type': 'application/json',
            ...config?.headers,
          },
          signal: controller.signal,
        });

        clearTimeout(timeoutId);

        // Convert Headers object to plain object
        const headers: Record<string, string> = {};
        response.headers.forEach((value, key) => {
          headers[key] = value;
        });

        const data = await response.json();

        const result: HttpResponse = {
          data,
          status: response.status,
          statusText: response.statusText,
          headers,
        };

        this.logger.debug(`HTTP GET response: ${response.status} ${response.statusText}`);
        return result;
      } catch (error: any) {
        this.logger.error(`HTTP GET error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`HTTP GET request failed: ${error instanceof Error ? error.message : error}`, { cause: error });
      }
    });

    // HTTP POST handler
    ipcMain.handle(
      'http:post',
      async (_, url: string, body?: any, config?: HttpRequestConfig): Promise<HttpResponse> => {
        try {
          if (!url || typeof url !== 'string') {
            throw new Error('URL is required and must be a string');
          }

          this.logger.debug(`HTTP POST request to: ${url}`);

          const controller = new AbortController();
          const timeoutId = setTimeout(() => controller.abort(), config?.timeout || 30000);

          const response = await fetch(url, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              ...config?.headers,
            },
            body: body ? JSON.stringify(body) : undefined,
            signal: controller.signal,
          });

          clearTimeout(timeoutId);

          // Convert Headers object to plain object
          const headers: Record<string, string> = {};
          response.headers.forEach((value, key) => {
            headers[key] = value;
          });

          const data = await response.json();

          const result: HttpResponse = {
            data,
            status: response.status,
            statusText: response.statusText,
            headers,
          };

          this.logger.debug(`HTTP POST response: ${response.status} ${response.statusText}`);
          return result;
        } catch (error: any) {
          this.logger.error(`HTTP POST error: ${error instanceof Error ? error.message : String(error)}`);
          throw new Error(`HTTP POST request failed: ${error instanceof Error ? error.message : error}`, {
            cause: error,
          });
        }
      },
    );

    // HTTP PUT handler
    ipcMain.handle(
      'http:put',
      async (_, url: string, body?: any, config?: HttpRequestConfig): Promise<HttpResponse> => {
        try {
          if (!url || typeof url !== 'string') {
            throw new Error('URL is required and must be a string');
          }

          this.logger.debug(`HTTP PUT request to: ${url}`);

          const controller = new AbortController();
          const timeoutId = setTimeout(() => controller.abort(), config?.timeout || 30000);

          const response = await fetch(url, {
            method: 'PUT',
            headers: {
              'Content-Type': 'application/json',
              ...config?.headers,
            },
            body: body ? JSON.stringify(body) : undefined,
            signal: controller.signal,
          });

          clearTimeout(timeoutId);

          // Convert Headers object to plain object
          const headers: Record<string, string> = {};
          response.headers.forEach((value, key) => {
            headers[key] = value;
          });

          const data = await response.json();

          const result: HttpResponse = {
            data,
            status: response.status,
            statusText: response.statusText,
            headers,
          };

          this.logger.debug(`HTTP PUT response: ${response.status} ${response.statusText}`);
          return result;
        } catch (error: any) {
          this.logger.error(`HTTP PUT error: ${error instanceof Error ? error.message : String(error)}`);
          throw new Error(`HTTP PUT request failed: ${error instanceof Error ? error.message : error}`, {
            cause: error,
          });
        }
      },
    );

    // HTTP DELETE handler
    ipcMain.handle('http:delete', async (_, url: string, config?: HttpRequestConfig): Promise<HttpResponse> => {
      try {
        if (!url || typeof url !== 'string') {
          throw new Error('URL is required and must be a string');
        }

        this.logger.debug(`HTTP DELETE request to: ${url}`);

        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), config?.timeout || 30000);

        const response = await fetch(url, {
          method: 'DELETE',
          headers: {
            'Content-Type': 'application/json',
            ...config?.headers,
          },
          signal: controller.signal,
        });

        clearTimeout(timeoutId);

        // Convert Headers object to plain object
        const headers: Record<string, string> = {};
        response.headers.forEach((value, key) => {
          headers[key] = value;
        });

        const data = await response.json();

        const result: HttpResponse = {
          data,
          status: response.status,
          statusText: response.statusText,
          headers,
        };

        this.logger.debug(`HTTP DELETE response: ${response.status} ${response.statusText}`);
        return result;
      } catch (error: any) {
        this.logger.error(`HTTP DELETE error: ${error instanceof Error ? error.message : String(error)}`);
        throw new Error(`HTTP DELETE request failed: ${error instanceof Error ? error.message : error}`, {
          cause: error,
        });
      }
    });
  }
}

export function createHttpModule() {
  return new HttpModule();
}
