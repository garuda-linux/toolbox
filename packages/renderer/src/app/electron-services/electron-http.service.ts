import { Injectable } from '@angular/core';
import { Logger } from '../logging/logging';
import { httpGet, httpPost, httpPut, httpDelete } from './electron-api-utils.js';

export interface HttpRequestConfig {
  headers?: Record<string, string>;
  timeout?: number;
}

export interface HttpResponse<T = unknown> {
  data: T;
  status: number;
  statusText: string;
  headers: Record<string, string>;
}

@Injectable({
  providedIn: 'root',
})
export class ElectronHttpService {
  private readonly logger = Logger.getInstance();

  /**
   * Send a GET request to the specified URL.
   * @param url - The URL to send the GET request to.
   * @param config - Optional configuration for the request.
   * @returns A promise that resolves to the response data.
   */
  async get<T = unknown>(url: string, config?: HttpRequestConfig): Promise<HttpResponse<T>> {
    try {
      this.logger.debug(`HTTP GET: ${url}`);
      const response = await httpGet(url, config);
      this.logger.debug(`HTTP GET response: ${response.status} ${response.statusText}`);
      return response as HttpResponse<T>;
    } catch (error) {
      this.logger.error(`HTTP GET error for ${url}: ${error instanceof Error ? error.message : String(error)}`);
      throw new Error(`GET request failed: ${error instanceof Error ? error.message : String(error)}`, {
        cause: error,
      });
    }
  }

  /**
   * Send a POST request to the specified URL.
   * @param url - The URL to send the POST request to.
   * @param body - The request body.
   * @param config - Optional configuration for the request.
   * @returns A promise that resolves to the response data.
   */
  async post<T = unknown, B = unknown>(url: string, body?: B, config?: HttpRequestConfig): Promise<HttpResponse<T>> {
    try {
      this.logger.debug(`HTTP POST: ${url}`);
      const response = await httpPost(url, body, config);
      this.logger.debug(`HTTP POST response: ${response.status} ${response.statusText}`);
      return response as HttpResponse<T>;
    } catch (error) {
      this.logger.error(`HTTP POST error for ${url}: ${error instanceof Error ? error.message : String(error)}`);
      throw new Error(`POST request failed: ${error instanceof Error ? error.message : String(error)}`, {
        cause: error,
      });
    }
  }

  /**
   * Sends a PUT request to the specified URL.
   * @param url The URL to send the PUT request to.
   * @param body The request body.
   * @param config Optional configuration for the PUT request.
   * @returns A promise that resolves to the response from the server.
   */
  async put<T = unknown, B = unknown>(url: string, body?: B, config?: HttpRequestConfig): Promise<HttpResponse<T>> {
    try {
      this.logger.debug(`HTTP PUT: ${url}`);
      const response = await httpPut(url, body, config);
      this.logger.debug(`HTTP PUT response: ${response.status} ${response.statusText}`);
      return response as HttpResponse<T>;
    } catch (error) {
      this.logger.error(`HTTP PUT error for ${url}: ${error instanceof Error ? error.message : String(error)}`);
      throw new Error(`PUT request failed: ${error instanceof Error ? error.message : String(error)}`, {
        cause: error,
      });
    }
  }

  /**
   * Sends a DELETE request to the specified URL.
   * @param url The URL to send the DELETE request to.
   * @param config Optional configuration for the DELETE request.
   * @returns A promise that resolves to the response from the server.
   */
  async delete<T = unknown>(url: string, config?: HttpRequestConfig): Promise<HttpResponse<T>> {
    try {
      this.logger.debug(`HTTP DELETE: ${url}`);
      const response = await httpDelete(url, config);
      this.logger.debug(`HTTP DELETE response: ${response.status} ${response.statusText}`);
      return response as HttpResponse<T>;
    } catch (error) {
      this.logger.error(`HTTP DELETE error for ${url}: ${error instanceof Error ? error.message : String(error)}`);
      throw new Error(`DELETE request failed: ${error instanceof Error ? error.message : String(error)}`, {
        cause: error,
      });
    }
  }

  /**
   * Utility method to check if response is successful
   * @param response The response to check.
   * @returns True if the response is successful, false otherwise.
   */
  isSuccessResponse<T>(response: HttpResponse<T>): boolean {
    return response.status >= 200 && response.status < 300;
  }

  /**
   * Utility method to extract data from successful response
   * @param response The response to extract data from.
   * @returns The data from the response.
   */
  extractData<T>(response: HttpResponse<T>): T {
    if (!this.isSuccessResponse(response)) {
      throw new Error(`HTTP request failed with status ${response.status}: ${response.statusText}`);
    }
    return response.data;
  }
}
