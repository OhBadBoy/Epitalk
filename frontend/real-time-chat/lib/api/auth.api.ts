import { FetchClient } from "./fetchClient";
import {
  LoginSchema,
  type LoginInput,
  RegisterSchema,
  type RegisterInput,
  UserSchema,
  type User,
  AuthResponseSchema,
  type AuthResponse,
} from "./schemas/auth.schema";

export interface UpdateProfileInput {
  username?: string;
  bio?: string;
  banner_color_1?: string;
  banner_color_2?: string;
  status?: "ONLINE" | "IDLE" | "DND" | "OFFLINE";
  theme?: string;
}

export class AuthAPI {
  private client: FetchClient;

  constructor(client: FetchClient) {
    this.client = client;
  }

  async register(data: RegisterInput): Promise<AuthResponse> {
    const validData = RegisterSchema.parse(data);
    const response = await this.client.post<AuthResponse>("/auth/register", validData);
    return AuthResponseSchema.parse(response);
  }

  async login(data: LoginInput): Promise<AuthResponse> {
    const validData = LoginSchema.parse(data);
    const response = await this.client.post<AuthResponse>("/auth/login", validData);
    return AuthResponseSchema.parse(response);
  }

  async me(): Promise<User> {
    const response = await this.client.get<User>("/auth/me");
    return UserSchema.parse(response);
  }

  async updateProfile(data: UpdateProfileInput): Promise<User> {
    const response = await this.client.patch<User>("/auth/me", data);
    return UserSchema.parse(response);
  }

  async uploadAvatar(file: File): Promise<User> {
    const formData = new FormData();
    formData.append("avatar", file);

    const token = typeof window !== "undefined" ? localStorage.getItem("token") : null;
    const baseUrl = this.client.getBaseUrl();

    const res = await fetch(`${baseUrl}/auth/me/avatar`, {
      method: "POST",
      headers: token ? { Authorization: `Bearer ${token}` } : {},
      body: formData,
    });

    if (!res.ok) {
      const payload = await res.json().catch(() => null);
      throw new Error(payload?.error || "Upload failed");
    }

    const response = await res.json();
    return UserSchema.parse(response);
  }

  async setAvatarFromUrl(url: string): Promise<User> {
    const response = await this.client.post<User>("/auth/me/avatar-url", { url });
    return UserSchema.parse(response);
  }

  async changeEmail(newEmail: string, password: string): Promise<User> {
    const response = await this.client.post<User>("/auth/me/email", {
      new_email: newEmail,
      password,
    });
    return UserSchema.parse(response);
  }

  async changePassword(currentPassword: string, newPassword: string): Promise<User> {
    const response = await this.client.post<User>("/auth/me/password", {
      current_password: currentPassword,
      new_password: newPassword,
    });
    return UserSchema.parse(response);
  }

  async refresh(): Promise<AuthResponse> {
    const response = await this.client.post<AuthResponse>("/auth/refresh");
    return AuthResponseSchema.parse(response);
  }

  async logout(): Promise<void> {
    // Logout cote client - supprimer le token du localStorage
    if (typeof window !== "undefined") {
      localStorage.removeItem("token");
      localStorage.removeItem("auth-storage");
    }
  }
}
