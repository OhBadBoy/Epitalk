import { ApiError, parseApiError } from "./errors";

const DEFAULT_REQUEST_TIMEOUT_MS = 20_000;
const RETRY_TIMEOUT_MS = 45_000;
const NON_REFRESHABLE_PATHS = ["/auth/login", "/auth/register", "/auth/refresh"];
let refreshInFlight: Promise<string | null> | null = null;

function isEnglishPreferred(): boolean {
  if (typeof window === "undefined") return false;
  return window.localStorage.getItem("epitalk_language") === "en";
}

export class FetchClient {
  constructor(private baseUrl: string = "") {}

  getBaseUrl(): string {
    return this.baseUrl;
  }

  private getToken(): string | null {
    if (typeof window === "undefined") return null;
    return localStorage.getItem("token");
  }

  private setToken(token: string | null): void {
    if (typeof window === "undefined") return;
    if (token) {
      localStorage.setItem("token", token);
    } else {
      localStorage.removeItem("token");
    }
  }

  private syncPersistedAuth(token: string | null, user?: unknown): void {
    if (typeof window === "undefined") return;

    if (!token) {
      localStorage.removeItem("auth-storage");
      return;
    }

    const raw = localStorage.getItem("auth-storage");
    let persisted: Record<string, unknown> = {};
    if (raw) {
      try {
        persisted = JSON.parse(raw) as Record<string, unknown>;
      } catch {
        persisted = {};
      }
    }

    const currentState =
      persisted && typeof persisted === "object" && "state" in persisted
        ? ((persisted.state as Record<string, unknown>) ?? {})
        : {};

    const nextState: Record<string, unknown> = {
      ...currentState,
      token,
      isAuthenticated: true,
    };

    if (user && typeof user === "object") {
      nextState.user = user;
    }

    const nextPersisted = {
      ...(persisted ?? {}),
      state: nextState,
    };

    localStorage.setItem("auth-storage", JSON.stringify(nextPersisted));
  }

  private async tryRefreshToken(currentToken: string): Promise<string | null> {
    if (refreshInFlight) {
      return refreshInFlight;
    }

    const refreshHeaders: Record<string, string> = {
      "Content-Type": "application/json",
      Authorization: `Bearer ${currentToken}`,
    };

    refreshInFlight = (async () => {
      try {
        const res = await fetch(`${this.baseUrl}/auth/refresh`, {
          method: "POST",
          headers: refreshHeaders,
        });

        if (!res.ok) return null;

        const payload = (await res.json()) as {
          token?: string;
          user?: unknown;
        };

        if (!payload?.token) return null;

        this.setToken(payload.token);
        this.syncPersistedAuth(payload.token, payload.user);
        return payload.token;
      } catch {
        return null;
      } finally {
        refreshInFlight = null;
      }
    })();

    return refreshInFlight;
  }

  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
    retryOnAuthFailure: boolean = true,
    retryOnTimeout: boolean = true,
  ): Promise<T> {
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };

    // Ajouter le token JWT si disponible
    const token = this.getToken();
    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(
      () => controller.abort(),
      retryOnTimeout ? DEFAULT_REQUEST_TIMEOUT_MS : RETRY_TIMEOUT_MS,
    );

    let res: Response;
    try {
      res = await fetch(`${this.baseUrl}${path}`, {
        method,
        headers,
        signal: controller.signal,
        body: body === undefined ? undefined : JSON.stringify(body),
      });
    } catch (error) {
      if (error instanceof DOMException && error.name === "AbortError") {
        if (retryOnTimeout) {
          return this.request<T>(method, path, body, retryOnAuthFailure, false);
        }
        throw new ApiError(
          0,
          isEnglishPreferred()
            ? "The server is taking too long to respond. Check that it is running and try again."
            : "Le serveur met trop de temps a repondre. Verifiez qu'il est demarre et reessayez.",
        );
      }
      throw new ApiError(
        0,
        isEnglishPreferred()
          ? "Unable to reach the server. Please check your connection."
          : "Impossible de contacter le serveur. Verifiez votre connexion.",
      );
    } finally {
      clearTimeout(timeoutId);
    }

    if (
      res.status === 401 &&
      retryOnAuthFailure &&
      token &&
      !NON_REFRESHABLE_PATHS.some((p) => path.startsWith(p))
    ) {
      const refreshedToken = await this.tryRefreshToken(token);
      if (refreshedToken) {
        return this.request<T>(method, path, body, false, retryOnTimeout);
      }

      this.setToken(null);
      this.syncPersistedAuth(null);
    }

    if (!res.ok) {
      let payload: unknown = null;
      try {
        payload = await res.json();
      } catch {
        const text = await res.text().catch(() => "");
        payload = text || null;
      }
      throw parseApiError(res.status, payload);
    }

    if (res.status === 204) return undefined as T;

    return (await res.json()) as T;
  }

  get<T>(path: string) {
    return this.request<T>("GET", path);
  }

  post<T>(path: string, body?: unknown) {
    return this.request<T>("POST", path, body);
  }

  put<T>(path: string, body?: unknown) {
    return this.request<T>("PUT", path, body);
  }

  patch<T>(path: string, body?: unknown) {
    return this.request<T>("PATCH", path, body);
  }

  delete<T>(path: string) {
    return this.request<T>("DELETE", path);
  }

  async postFile<T>(path: string, formData: FormData): Promise<T> {
    const headers: Record<string, string> = {};
    const token = this.getToken();
    if (token) headers["Authorization"] = `Bearer ${token}`;

    const res = await fetch(`${this.baseUrl}${path}`, {
      method: "POST",
      headers,
      body: formData,
    });

    if (!res.ok) {
      let payload: unknown = null;
      try { payload = await res.json(); } catch {
        const text = await res.text().catch(() => "");
        payload = text || null;
      }
      throw parseApiError(res.status, payload);
    }

    return (await res.json()) as T;
  }
}
