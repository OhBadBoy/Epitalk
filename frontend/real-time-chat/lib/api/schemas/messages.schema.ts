import { z } from "zod";

const ReactionSchema = z.object({
  emoji: z.string(),
  user_id: z.string(),
  username: z.string().optional(),
});

// Message schema - matches backend MessageResponse (channels.rs)
export const MessageSchema = z.object({
  id: z.string(),
  channel_id: z.string(),
  server_id: z.string().optional(),
  author_id: z.string(),
  username: z.string().optional(),
  content: z.string(),
  gif: z
    .object({
      id: z.string(),
      url: z.string(),
      preview: z.string().optional(),
      provider: z.string().optional(),
    })
    .optional(),
  created_at: z.string(),
  deleted_at: z.string().nullable().optional(),
  reply_to: z.string().nullish(),
  attachment_url: z.string().nullable().optional(),
  reactions: z.array(ReactionSchema).optional(),
  pinned_by: z.string().nullable().optional(),
  pinned_at: z.string().nullable().optional(),
});

export const MessageListSchema = z.array(MessageSchema);

export type Message = z.infer<typeof MessageSchema>;
