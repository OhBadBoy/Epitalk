import { z } from "zod";

// Server schema - matches backend ServerResponse
export const ServerSchema = z.object({
  id: z.string(),
  name: z.string(),
  owner_id: z.string(),
  created_at: z.string(),
  member_count: z.number().optional(),
});

export type Server = z.infer<typeof ServerSchema>;

// Channel schema (re-exported from channels.schema for convenience)
// Matches backend ChannelResponse - uses snake_case and PascalCase kind
export const ChannelSchema = z.object({
  id: z.string(),
  server_id: z.string(),
  name: z.string(),
  kind: z.enum(["Text"]), // Backend returns PascalCase ChannelKind
  created_at: z.string(),
});

export type Channel = z.infer<typeof ChannelSchema>;

// Member schema (membership with user info)
// Note: Backend returns user_id as the identifier (no separate 'id' field)
// Backend uses PascalCase for role values: "Owner", "Admin", "Moderator", "Member"
export const MemberSchema = z.object({
  user_id: z.string(),
  username: z.string(),
  role: z.enum(["Owner", "Admin", "Moderator", "Member"]),
  joined_at: z.string(),
  avatar_url: z.string().nullable().optional(),
  status: z.string().nullable().optional(),
});

// Alias for convenience
export type MemberId = string;

export type Member = z.infer<typeof MemberSchema>;

// Ban schema - matches backend BanResponse
export const BanSchema = z.object({
  id: z.string(),
  user_id: z.string(),
  username: z.string(),
  banned_by: z.string(),
  reason: z.string().nullable(),
  expires_at: z.string().nullable(),
  created_at: z.string(),
});

export type Ban = z.infer<typeof BanSchema>;

// Request body for ban endpoint
export const BanMemberRequestSchema = z.object({
  reason: z.string().nullable().optional(),
  expires_at: z.string().nullable().optional(),
});

export type BanMemberRequest = z.infer<typeof BanMemberRequestSchema>;

// Invite schema - matches backend InviteResponse
export const InviteSchema = z.object({
  code: z.string(),
  server_id: z.string(),
  created_by: z.string(),
  expires_at: z.string().nullable(),
  max_uses: z.number().nullable(),
  uses: z.number(),
  is_valid: z.boolean().optional(), // Backend includes this in response
});

export type Invite = z.infer<typeof InviteSchema>;
