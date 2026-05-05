import type { FetchClient } from "./fetchClient";
import { MessageListSchema, MessageSchema, type Message } from "./schemas/messages.schema";

export function createMessagesApi(client: FetchClient) {
  return {
    list: async (
      serverId: string,
      channelId: string,
      params?: { before?: string; limit?: number },
    ): Promise<Message[]> => {
      const qs = new URLSearchParams();
      if (params?.before) qs.set("before", params.before);
      if (params?.limit) qs.set("per_page", String(params.limit));
      const query = qs.toString() ? `?${qs.toString()}` : "";
      const data = await client.get<Message[]>(
        `/servers/${serverId}/channels/${channelId}/messages${query}`,
      );
      return MessageListSchema.parse(data);
    },

    search: async (
      serverId: string,
      channelId: string,
      query: string,
      page = 1,
      perPage = 50
    ): Promise<Message[]> => {
      const q = encodeURIComponent(query.trim());
      const data = await client.get<Message[]>(
        `/servers/${serverId}/channels/${channelId}/messages/search?q=${q}&page=${page}&per_page=${perPage}`
      );
      return MessageListSchema.parse(data);
    },

    listPinned: async (
      serverId: string,
      channelId: string,
      page = 1,
      perPage = 50
    ): Promise<Message[]> => {
      const data = await client.get<Message[]>(
        `/servers/${serverId}/channels/${channelId}/messages/pinned?page=${page}&per_page=${perPage}`
      );
      return MessageListSchema.parse(data);
    },

    send: async (
      serverId: string,
      channelId: string,
      content: string
    ): Promise<Message> => {
      const data = await client.post<Message>(
        `/servers/${serverId}/channels/${channelId}/messages`,
        { content }
      );
      return MessageSchema.parse(data);
    },

    edit: async (
      serverId: string,
      channelId: string,
      messageId: string,
      content: string
    ): Promise<Message> => {
      const data = await client.patch<Message>(
        `/servers/${serverId}/channels/${channelId}/messages/${messageId}`,
        { content }
      );
      return MessageSchema.parse(data);
    },

    pin: async (serverId: string, channelId: string, messageId: string): Promise<void> => {
      await client.post<{ pinned: boolean }>(
        `/servers/${serverId}/channels/${channelId}/messages/${messageId}/pin`
      );
    },

    unpin: async (serverId: string, channelId: string, messageId: string): Promise<void> => {
      await client.delete<{ unpinned: boolean }>(
        `/servers/${serverId}/channels/${channelId}/messages/${messageId}/pin`
      );
    },

    uploadAttachment: async (
      serverId: string,
      channelId: string,
      file: File,
    ): Promise<{ url: string }> => {
      const formData = new FormData();
      formData.append("file", file);
      return client.postFile<{ url: string }>(
        `/servers/${serverId}/channels/${channelId}/upload`,
        formData,
      );
    },
  };
}
