import { useMessageStore } from "@/store/message.store";
import { useNotificationStore } from "@/store/notifications.store";
import { useAuthStore } from "@/store/auth.store";
import { useChannelStore } from "@/store/channel.store";
import { useServerStore } from "@/store/server.store";
import type { z } from "zod";
import type {
  MessageNewSchema,
  MessageEditedSchema,
  MessageDeletedSchema,
  MessagePinnedSchema,
  MessageUnpinnedSchema,
  ReactionAddedSchema,
  ReactionRemovedSchema,
} from "@/lib/ws/types";

type MessageNew = z.infer<typeof MessageNewSchema>;
type MessageEdited = z.infer<typeof MessageEditedSchema>;
type MessageDeleted = z.infer<typeof MessageDeletedSchema>;
type MessagePinned = z.infer<typeof MessagePinnedSchema>;
type MessageUnpinned = z.infer<typeof MessageUnpinnedSchema>;
type ReactionAdded = z.infer<typeof ReactionAddedSchema>;
type ReactionRemoved = z.infer<typeof ReactionRemovedSchema>;

export const messageHandler = {
  onMessageNew(payload: MessageNew): void {
    useMessageStore.getState().addMessage(payload.channel_id, {
      id: payload.id,
      channel_id: payload.channel_id,
      author_id: payload.author_id,
      username: payload.username,
      content: payload.content,
      created_at: payload.created_at,
      reply_to: payload.reply_to ?? undefined,
      attachment_url: payload.attachment_url,
      reactions: payload.reactions,
    });

    // Trigger notification for new server message
    const currentUser = useAuthStore.getState().user;
    if (payload.author_id !== currentUser?.id) {
      // Detect if this is a historical message (loaded from archive during JoinChannel)
      // Messages older than 5 seconds are considered historical
      const messageTimestamp = new Date(payload.created_at).getTime();
      const now = Date.now();
      const messageAgeMs = now - messageTimestamp;
      const isHistorical = messageAgeMs > 5000; // 5 seconds threshold

      const channelStore = useChannelStore.getState();
      const serverStore = useServerStore.getState();
      
      const channel = channelStore.channels.find((c) => c.id === payload.channel_id);
      const server = channel ? serverStore.servers.find((s) => s.id === channel.server_id) : null;
      
      const channelName = channel?.name || "channel";
      const serverName = server?.name || "Serveur";
      
      useNotificationStore.getState().showNotification({
        type: "server_message",
        title: `${payload.username} in ${serverName}`,
        message: `#${channelName}: ${payload.content.substring(0, 100)}`,
        isHistorical,
        data: {
          channelId: payload.channel_id,
          channelName,
          serverId: server?.id,
          serverName,
        },
      });
    }
  },

  onMessageEdited(payload: MessageEdited): void {
    useMessageStore
      .getState()
      .updateMessage(payload.channel_id, payload.id, {
        content: payload.content,
        edited_at: payload.edited_at,
      });
  },

  onMessageDeleted(payload: MessageDeleted): void {
    useMessageStore.getState().removeMessage(payload.channel_id, payload.id);
  },

  onMessagePinned(payload: MessagePinned): void {
    useMessageStore
      .getState()
      .updateMessage(payload.channel_id, payload.message_id, {
        pinned_by: payload.pinned_by,
        pinned_at: payload.pinned_at,
      });
  },

  onMessageUnpinned(payload: MessageUnpinned): void {
    useMessageStore
      .getState()
      .updateMessage(payload.channel_id, payload.message_id, {
        pinned_by: null,
        pinned_at: null,
      });
  },

  onReactionAdded(payload: ReactionAdded): void {
    const { messages } = useMessageStore.getState();
    for (const [channelId, msgs] of Object.entries(messages)) {
      const msg = msgs.find((m) => m.id === payload.message_id);
      if (!msg) continue;
      const existing = msg.reactions ?? [];
      const already = existing.some(
        (r) => r.user_id === payload.user_id && r.emoji === payload.emoji,
      );
      if (already) return;
      useMessageStore.getState().updateMessage(channelId, payload.message_id, {
        reactions: [
          ...existing,
          { emoji: payload.emoji, user_id: payload.user_id, username: payload.username },
        ],
      });
      return;
    }
  },

  onReactionRemoved(payload: ReactionRemoved): void {
    const { messages } = useMessageStore.getState();
    for (const [channelId, msgs] of Object.entries(messages)) {
      const msg = msgs.find((m) => m.id === payload.message_id);
      if (!msg) continue;
      useMessageStore.getState().updateMessage(channelId, payload.message_id, {
        reactions: (msg.reactions ?? []).filter(
          (r) => !(r.user_id === payload.user_id && r.emoji === payload.emoji),
        ),
      });
      return;
    }
  },
};
