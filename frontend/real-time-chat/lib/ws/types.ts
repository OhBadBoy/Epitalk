import { z } from "zod";

// ─── Primitive types ──────────────────────────────────────────────────────────

export type PresenceStatus = "online" | "idle" | "dnd" | "offline";

export type ConnectionState =
  | "idle"
  | "connecting"
  | "connected"
  | "backoff"
  | "degraded"
  | "auth_invalid";

export interface WsMessage {
  id: string;
  channel_id: string;
  author_id: string;
  username?: string;
  content: string;
  created_at: string;
  reply_to?: string;
  attachment_url?: string | null;
  reactions?: Array<{
    emoji: string;
    user_id: string;
    username?: string;
    created_at?: string;
  }>;
  edited_at?: string;
  pinned_by?: string | null;
  pinned_at?: string | null;
}

export interface GifPayload {
  id: string;
  url: string;
  preview?: string;
  provider?: string;
}

// ─── Client → Server events ───────────────────────────────────────────────────

export type ClientEvent =
  | { type: "MessageSend"; payload: { channel_id: string; content: string; reply_to?: string; attachment_url?: string } }
  | { type: "MessageEdit"; payload: { channel_id: string; message_id: string; content: string } }
  | { type: "MessageDelete"; payload: { channel_id: string; message_id: string } }
  | { type: "MessageSendGif"; payload: { channel_id: string; gif: GifPayload; caption?: string | null } }
  | { type: "JoinChannel"; payload: { channel_id: string } }
  | { type: "LeaveChannel"; payload: { channel_id: string } }
  | { type: "TypingStart"; payload: { channel_id: string } }
  | { type: "TypingStop"; payload: { channel_id: string } }
  | { type: "DmSend"; payload: { recipient_id: string; content: string; reply_to?: string; attachment_url?: string } }
  | { type: "DmEdit"; payload: { conversation_id: string; message_id: string; content: string } }
  | { type: "DmDelete"; payload: { conversation_id: string; message_id: string } }
  | { type: "DmSendGif"; payload: { recipient_id: string; gif: GifPayload; caption?: string | null } }
  | { type: "JoinDm"; payload: { peer_id: string } }
  | { type: "LeaveDm"; payload: { peer_id: string } }
  | { type: "PresenceSet"; payload: { status: PresenceStatus } }
  | { type: "Ping" };

// ─── Server → Client Zod schemas ─────────────────────────────────────────────

const ReactionSchema = z.object({
  emoji: z.string(),
  user_id: z.string(),
  username: z.string().optional(),
});

export const MessageNewSchema = z.object({
  id: z.string(),
  channel_id: z.string(),
  author_id: z.string(),
  username: z.string().optional(),
  content: z.string(),
  created_at: z.string(),
  reply_to: z.string().nullish(),
  attachment_url: z.string().nullable().optional(),
  reactions: z.array(ReactionSchema).optional(),
});

export const MessageEditedSchema = z.object({
  id: z.string(),
  channel_id: z.string(),
  author_id: z.string(),
  username: z.string().optional(),
  content: z.string(),
  edited_at: z.string(),
});

export const MessageDeletedSchema = z.object({
  id: z.string(),
  channel_id: z.string(),
});

export const MessagePinnedSchema = z.object({
  message_id: z.string(),
  channel_id: z.string(),
  pinned_by: z.string(),
  pinned_at: z.string(),
});

export const MessageUnpinnedSchema = z.object({
  message_id: z.string(),
  channel_id: z.string(),
});

export const ReactionAddedSchema = z.object({
  message_id: z.string(),
  emoji: z.string(),
  user_id: z.string(),
  username: z.string().optional(),
});

export const ReactionRemovedSchema = z.object({
  message_id: z.string(),
  emoji: z.string(),
  user_id: z.string(),
});

export const UserOnlineSchema = z.object({
  user_id: z.string(),
  // Backend may send the preserved status (idle/dnd) on reconnect
  status: z.enum(["online", "idle", "dnd", "offline"]).optional().default("online"),
});
export const UserOfflineSchema = z.object({
  user_id: z.string(),
  status: z.enum(["online", "idle", "dnd", "offline"]).optional().default("offline"),
});

export const PresenceUpdatedSchema = z.object({
  user_id: z.string(),
  status: z.enum(["online", "idle", "dnd", "offline"]),
  last_activity: z.string().optional(),
});

const PresenceUserSchema = z.object({
  user_id: z.string(),
  status: z.string(),
});

export const PresenceSyncSchema = z.object({
  users: z.array(PresenceUserSchema),
});

export const TypingStartSchema = z.object({
  user_id: z.string(),
  username: z.string(),
  channel_id: z.string(),
});

export const TypingStopSchema = z.object({
  user_id: z.string(),
  username: z.string(),
  channel_id: z.string(),
});

export const UserJoinedSchema = z.object({
  user_id: z.string(),
  channel_id: z.string(),
});

export const UserLeftSchema = z.object({
  user_id: z.string(),
  channel_id: z.string(),
});

export const DmNewSchema = z.object({
  id: z.string(),
  conversation_id: z.string(),
  author_id: z.string(),
  username: z.string(),
  content: z.string(),
  created_at: z.string(),
  reply_to: z.string().nullish(),
  attachment_url: z.string().nullable().optional(),
});

export const DmEditedSchema = z.object({
  id: z.string(),
  conversation_id: z.string(),
  author_id: z.string(),
  username: z.string(),
  content: z.string(),
  edited_at: z.string(),
});

export const DmDeletedSchema = z.object({
  id: z.string(),
  conversation_id: z.string(),
});

export const ServerErrorSchema = z.object({
  code: z.string(),
  message: z.string(),
});

// ─── Discriminated union for all server → client events ──────────────────────

export const ServerEventSchema = z.discriminatedUnion("type", [
  z.object({ type: z.literal("MessageNew"), payload: MessageNewSchema }),
  z.object({ type: z.literal("MessageEdited"), payload: MessageEditedSchema }),
  z.object({ type: z.literal("MessageDeleted"), payload: MessageDeletedSchema }),
  z.object({ type: z.literal("MessagePinned"), payload: MessagePinnedSchema }),
  z.object({ type: z.literal("MessageUnpinned"), payload: MessageUnpinnedSchema }),
  z.object({ type: z.literal("ReactionAdded"), payload: ReactionAddedSchema }),
  z.object({ type: z.literal("ReactionRemoved"), payload: ReactionRemovedSchema }),
  z.object({ type: z.literal("UserOnline"), payload: UserOnlineSchema }),
  z.object({ type: z.literal("UserOffline"), payload: UserOfflineSchema }),
  z.object({ type: z.literal("PresenceUpdated"), payload: PresenceUpdatedSchema }),
  z.object({ type: z.literal("PresenceSync"), payload: PresenceSyncSchema }),
  z.object({ type: z.literal("TypingStart"), payload: TypingStartSchema }),
  z.object({ type: z.literal("TypingStop"), payload: TypingStopSchema }),
  z.object({ type: z.literal("UserJoined"), payload: UserJoinedSchema }),
  z.object({ type: z.literal("UserLeft"), payload: UserLeftSchema }),
  z.object({ type: z.literal("DmNew"), payload: DmNewSchema }),
  z.object({ type: z.literal("DmEdited"), payload: DmEditedSchema }),
  z.object({ type: z.literal("DmDeleted"), payload: DmDeletedSchema }),
  z.object({ type: z.literal("Error"), payload: ServerErrorSchema }),
  z.object({ type: z.literal("Pong") }),
]);

export type ServerEvent = z.infer<typeof ServerEventSchema>;
