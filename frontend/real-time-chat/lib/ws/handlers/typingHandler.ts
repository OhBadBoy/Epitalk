import { useTypingStore } from "@/store/typing.store";
import type { z } from "zod";
import type { TypingStartSchema, TypingStopSchema } from "@/lib/ws/types";

type TypingStart = z.infer<typeof TypingStartSchema>;
type TypingStop = z.infer<typeof TypingStopSchema>;

export const typingHandler = {
  onTypingStart(payload: TypingStart): void {
    useTypingStore.getState().addTypingUser(payload.channel_id, payload.username);
  },

  onTypingStop(payload: TypingStop): void {
    useTypingStore.getState().removeTypingUser(payload.channel_id, payload.username);
  },
};
