import { z } from "zod";

// Channel schema - matches backend ChannelResponse
// Backend uses snake_case and returns kind as "Text" (PascalCase)
export const ChannelSchema = z.object({
  id: z.string(),
  server_id: z.string(),
  name: z.string(),
  kind: z.enum(["Text"]), // Backend returns PascalCase ChannelKind
  created_at: z.string(),
});

export type Channel = z.infer<typeof ChannelSchema>;

export const ChannelListSchema = z.array(ChannelSchema);

export const CreateChannelInputSchema = z.object({
  name: z.string().min(1, "Nom requis"),
  kind: z.enum(["Text"]).optional().default("Text"),
});

export type CreateChannelInput = z.infer<typeof CreateChannelInputSchema>;
