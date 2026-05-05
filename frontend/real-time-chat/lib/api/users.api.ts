import { FetchClient } from "./fetchClient";

export class UsersAPI {
  private client: FetchClient;

  constructor(client: FetchClient) {
    this.client = client;
  }

  // Patch user status: { status: 'online' | 'dnd' | 'offline' }
  async setStatus(userId: string, status: "online" | "dnd" | "offline") {
    return this.client.patch<{ id: string; status: string }>(
      `/users/${userId}/status`,
      { status },
    );
  }
}
