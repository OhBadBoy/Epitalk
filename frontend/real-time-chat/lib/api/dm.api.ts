import type { FetchClient } from "./fetchClient";

export interface DmConversation {
  conversation_id: string;
  peer_id: string;
  peer_username: string;
  last_message: string;
  last_message_at: string;
}

export interface DmMessageItem {
  id: string;
  conversation_id: string;
  author_id: string;
  username: string;
  content: string;
  created_at: string;
  reply_to?: string;
  attachment_url?: string | null;
}

export function createDmApi(client: FetchClient) {
  return {
    listConversations: () =>
      client.get<DmConversation[]>("/dm/conversations"),

    listMessages: async (
      peerId: string,
      params?: { before?: string; limit?: number },
    ): Promise<DmMessageItem[]> => {
      const qs = new URLSearchParams();
      if (params?.before) qs.set("before", params.before);
      if (params?.limit) qs.set("per_page", String(params.limit));
      const query = qs.toString() ? `?${qs.toString()}` : "";
      return client.get<DmMessageItem[]>(`/dm/conversations/${peerId}/messages${query}`);
    },

    uploadAttachment: async (file: File): Promise<{ url: string }> => {
      const formData = new FormData();
      formData.append("file", file);
      return client.postFile<{ url: string }>("/dm/upload", formData);
    },
  };
}
