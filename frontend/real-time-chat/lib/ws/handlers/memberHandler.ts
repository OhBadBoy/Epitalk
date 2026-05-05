import type { z } from "zod";
import type { UserJoinedSchema, UserLeftSchema } from "@/lib/ws/types";

type UserJoined = z.infer<typeof UserJoinedSchema>;
type UserLeft = z.infer<typeof UserLeftSchema>;

/**
 * Handles channel membership presence events.
 * Member list is fetched via REST on channel join, so these events
 * are currently informational only.
 */
export const memberHandler = {
  onUserJoined(payload: UserJoined): void {
    console.debug("[MemberHandler] joined", payload.user_id, payload.channel_id);
  },

  onUserLeft(payload: UserLeft): void {
    console.debug("[MemberHandler] left", payload.user_id, payload.channel_id);
  },
};
