import type { FetchClient } from "./fetchClient";
import type { Channel } from "./schemas/channels.schema";

export function createChannelsApi(client: FetchClient) {
  return {
    listByServer: (serverId: string) =>
      client.get<Channel[]>(`/servers/${serverId}/channels`),

    get: (serverId: string, channelId: string) =>
      client.get<Channel>(`/servers/${serverId}/channels/${channelId}`),

    create: (serverId: string, name: string) =>
      client.post<Channel>(`/servers/${serverId}/channels`, { name }),

    delete: (serverId: string, channelId: string) =>
      client.delete<Channel>(`/servers/${serverId}/channels/${channelId}`),
  };
}
