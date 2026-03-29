// Sourced from: https://github.com/pixelfactoryio/privatebin-cli
import { ElectronHttpService } from '../../electron-services';

export interface ApiConfig {
  baseURL: string;
  headers?: Record<string, string>;
}

export interface ApiResponse<T> {
  data: T;
  status: number;
  statusText: string;
}

export class Api {
  private baseURL: string;
  private defaultHeaders: Record<string, string>;

  constructor(
    private httpService: ElectronHttpService,
    config?: ApiConfig,
  ) {
    this.baseURL = config?.baseURL || '';
    this.defaultHeaders = config?.headers || {};
  }

  // Configure the API with base URL and default headers
  configure(config: ApiConfig): void {
    this.baseURL = config.baseURL;
    this.defaultHeaders = config.headers || {};
  }

  async get<T, R = ApiResponse<T>>(url: string, config?: { headers?: Record<string, string> }): Promise<R> {
    try {
      const response = await this.httpService.get<T>(this.baseURL + url, {
        headers: {
          ...this.defaultHeaders,
          ...config?.headers,
        },
      });

      return {
        data: response.data,
        status: response.status,
        statusText: response.statusText,
      } as R;
    } catch (error) {
      throw new Error(`GET request failed: ${error instanceof Error ? error.message : String(error)}`, {
        cause: error,
      });
    }
  }

  public async post<T, B, R = ApiResponse<T>>(
    url: string,
    data?: B,
    config?: { headers?: Record<string, string> },
  ): Promise<R> {
    try {
      const response = await this.httpService.post<T, B>(this.baseURL + url, data, {
        headers: {
          ...this.defaultHeaders,
          ...config?.headers,
        },
      });

      return {
        data: response.data,
        status: response.status,
        statusText: response.statusText,
      } as R;
    } catch (error) {
      throw new Error(`POST request failed: ${error instanceof Error ? error.message : String(error)}`, {
        cause: error,
      });
    }
  }

  public async put<T, B, R = ApiResponse<T>>(
    url: string,
    data?: B,
    config?: { headers?: Record<string, string> },
  ): Promise<R> {
    try {
      const response = await this.httpService.put<T, B>(this.baseURL + url, data, {
        headers: {
          ...this.defaultHeaders,
          ...config?.headers,
        },
      });

      return {
        data: response.data,
        status: response.status,
        statusText: response.statusText,
      } as R;
    } catch (error) {
      throw new Error(`PUT request failed: ${error instanceof Error ? error.message : String(error)}`, {
        cause: error,
      });
    }
  }

  public async delete<T, R = ApiResponse<T>>(url: string, config?: { headers?: Record<string, string> }): Promise<R> {
    try {
      const response = await this.httpService.delete<T>(this.baseURL + url, {
        headers: {
          ...this.defaultHeaders,
          ...config?.headers,
        },
      });

      return {
        data: response.data,
        status: response.status,
        statusText: response.statusText,
      } as R;
    } catch (error) {
      throw new Error(`DELETE request failed: ${error instanceof Error ? error.message : String(error)}`, {
        cause: error,
      });
    }
  }

  public success<T>(response: ApiResponse<T>): T {
    return response.data;
  }

  public isSuccess<T>(response: ApiResponse<T>): boolean {
    return response.status >= 200 && response.status < 300;
  }
}
