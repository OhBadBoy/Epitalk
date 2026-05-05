import { useDmStore } from "@/store/dm.store";
import { useAuthStore } from "@/store/auth.store";
import { useNotificationStore } from "@/store/notifications.store";
import type { z } from "zod";
import type { DmNewSchema, DmEditedSchema, DmDeletedSchema } from "@/lib/ws/types";

type DmNew = z.infer<typeof DmNewSchema>;
type DmEdited = z.infer<typeof DmEditedSchema>;
type DmDeleted = z.infer<typeof DmDeletedSchema>;

function getPeerId(conversationId: string, myId: string): string | null {
  if (!conversationId.startsWith("dm:")) return null;
  const parts = conversationId.slice(3).split(":");
  if (parts.length !== 2) return null;
  return parts[0] === myId ? parts[1] : parts[0];
}

function formatLastMessage(content: string): string {
  try {
    const p = JSON.parse(content) as { type?: string };
    if (p?.type === "gif") return "GIF";
  } catch {
    // not JSON
  }
  return content;
}

export const dmHandler = {
  onDmNew(payload: DmNew): void {
    const { id, conversation_id, author_id, username, content, created_at, reply_to, attachment_url } =
      payload;

    useDmStore.getState().addDmMessage(conversation_id, {
      id,
      channel_id: conversation_id,
      author_id,
      username,
      content,
      created_at,
      reply_to: reply_to ?? undefined,
      attachment_url,
    });

    const authUser = useAuthStore.getState().user;
    if (!authUser) return;

    const peerId = getPeerId(conversation_id, authUser.id);
    if (!peerId) return;

    const existing = useDmStore
      .getState()
      .conversations.find((c) => c.peer_id === peerId);

    const peerUsername =
      author_id !== authUser.id
        ? username
        : (existing?.peer_username ?? peerId.slice(0, 8));

    useDmStore.getState().addOrUpdateConversation({
      conversation_id,
      peer_id: peerId,
      peer_username: peerUsername,
      last_message: formatLastMessage(content),
      last_message_at: created_at,
    });

    // Trigger notification for incoming DM (not own messages)
    if (author_id !== authUser.id) {
      // Detect if this is a historical message (loaded from archive during JoinDm)
      // Messages older than 5 seconds are considered historical
      const messageTimestamp = new Date(created_at).getTime();
      const now = Date.now();
      const messageAgeMs = now - messageTimestamp;
      const isHistorical = messageAgeMs > 5000; // 5 seconds threshold

      useNotificationStore.getState().showNotification({
        type: "dm",
        title: `Message de ${peerUsername}`,
        message: content.substring(0, 100),
        userId: peerId,
        isHistorical,
        data: {
          peerId,
          peerUsername,
        },
      });
    }
  },

  onDmEdited(payload: DmEdited): void {
    useDmStore
      .getState()
      .updateDmMessage(payload.conversation_id, payload.id, {
        content: payload.content,
        edited_at: payload.edited_at,
      });
  },

  onDmDeleted(payload: DmDeleted): void {
    useDmStore.getState().removeDmMessage(payload.conversation_id, payload.id);
  },
};
