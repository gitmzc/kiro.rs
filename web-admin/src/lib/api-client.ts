// API Client for Kiro Admin API
// Handles authentication, error handling, and request formatting

const API_BASE = '/api/admin';

interface ApiError {
  type: string;
  message: string;
}

interface ApiErrorResponse {
  error: ApiError;
}

export class ApiClientError extends Error {
  type: string;
  status?: number;

  constructor(
    message: string,
    type: string,
    status?: number
  ) {
    super(message);
    this.name = 'ApiClientError';
    this.type = type;
    this.status = status;
  }
}

class ApiClient {
  private adminApiKey: string | null = null;

  setAdminApiKey(key: string) {
    this.adminApiKey = key;
  }

  private getHeaders(): HeadersInit {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
    };

    if (this.adminApiKey) {
      headers['x-api-key'] = this.adminApiKey;
    }

    return headers;
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
      // Handle 401 Unauthorized - clear token and reload
      if (response.status === 401) {
        localStorage.removeItem("adminApiKey");
        localStorage.removeItem("chatApiKey");
        window.location.reload();
        throw new ApiClientError("认证失败，请重新登录", "authentication_error", 401);
      }

      let errorMessage = `HTTP ${response.status}: ${response.statusText}`;
      let errorType = 'api_error';

      try {
        const errorData: ApiErrorResponse = await response.json();
        if (errorData.error) {
          errorMessage = errorData.error.message;
          errorType = errorData.error.type;
        }
      } catch {
        // If JSON parsing fails, use default error message
      }

      throw new ApiClientError(errorMessage, errorType, response.status);
    }

    return response.json();
  }

  async get<T>(path: string, params?: Record<string, string | number>): Promise<T> {
    const url = new URL(`${API_BASE}${path}`, window.location.origin);
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        url.searchParams.append(key, String(value));
      });
    }

    const response = await fetch(url.toString(), {
      method: 'GET',
      headers: this.getHeaders(),
    });

    return this.handleResponse<T>(response);
  }

  async post<T>(path: string, body?: unknown): Promise<T> {
    const response = await fetch(`${API_BASE}${path}`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: body ? JSON.stringify(body) : undefined,
    });

    return this.handleResponse<T>(response);
  }

  async put<T>(path: string, body?: unknown): Promise<T> {
    const response = await fetch(`${API_BASE}${path}`, {
      method: 'PUT',
      headers: this.getHeaders(),
      body: body ? JSON.stringify(body) : undefined,
    });

    return this.handleResponse<T>(response);
  }

  async delete<T>(path: string): Promise<T> {
    const response = await fetch(`${API_BASE}${path}`, {
      method: 'DELETE',
      headers: this.getHeaders(),
    });

    return this.handleResponse<T>(response);
  }

  async postMultipart<T>(path: string, formData: FormData): Promise<T> {
    const headers: HeadersInit = {};
    if (this.adminApiKey) {
      headers['x-api-key'] = this.adminApiKey;
    }

    const response = await fetch(`${API_BASE}${path}`, {
      method: 'POST',
      headers,
      body: formData,
    });

    return this.handleResponse<T>(response);
  }
}

export const apiClient = new ApiClient();

// API Response Types
export interface StatsSummaryResponse {
  range: {
    start: string;
    end: string;
  };
  requests: {
    total: number;
    success: number;
    failed: number;
    errorRate: number;
  };
  tokens: {
    input: number;
    output: number;
    total: number;
  };
  latencyMs: {
    avg: number;
    p95: number;
  };
}

export interface HealthResponse {
  status: string;
  db: string;
  now: string;
  uptimeSeconds: number;
}

export interface TimeseriesPoint {
  ts: string;
  requests: number;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  avgLatencyMs: number;
}

export interface StatsTimeseriesResponse {
  intervalMinutes: number;
  points: TimeseriesPoint[];
}

export interface RequestItem {
  ts: string;
  method: string;
  path: string;
  status: number;
  durationMs: number;
  inputTokens?: number;
  outputTokens?: number;
  totalTokens: number;
  model: string;
  stream: boolean;
}

export interface StatsRequestsResponse {
  items: RequestItem[];
}

export interface CredentialStatus {
  index: number;
  priority: number;
  disabled: boolean;
  failureCount: number;
  isCurrent: boolean;
  expiresAt: string;
  authMethod: string;
  hasProfileArn: boolean;
}

export interface CredentialsStatusResponse {
  total: number;
  available: number;
  currentIndex: number;
  credentials: CredentialStatus[];
}

export interface SuccessResponse {
  success: boolean;
  message: string;
}

export interface BalanceResponse {
  index: number;
  subscriptionTitle: string;
  currentUsage: number;
  usageLimit: number;
  remaining: number;
  usagePercentage: number;
  nextResetAt: number;
}

export interface ConfigView {
  host: string;
  port: number;
  region: string;
  kiroVersion: string;
  systemVersion: string;
  nodeVersion: string;
  countTokensApiUrl: string | null;
  countTokensAuthType: string;
  proxyUrl: string | null;
  adminApiKey: string;
  apiKey: string;
  proxyUsername: string | null;
  proxyPassword: string | null;
  countTokensApiKey: string | null;
  thinkingBudgetTokens: number;
  modelMapping: Record<string, string>;
}

export interface ConfigPatch {
  thinkingBudgetTokens?: number;
  modelMapping?: Record<string, string>;
}

export interface ApiKeyItem {
  id: string;
  name: string;
  keyPreview: string;
  enabled: boolean;
  createdAt: number;
}

export interface ApiKeysResponse {
  apiKeys: ApiKeyItem[];
}

export interface CreateApiKeyRequest {
  name: string;
}

export interface CreateApiKeyResponse {
  id: string;
  key: string;
  name: string;
  createdAt: number;
}

export interface UpdateApiKeyRequest {
  name?: string;
  enabled?: boolean;
}

export interface ChangePasswordRequest {
  oldPassword: string;
  newPassword: string;
}
