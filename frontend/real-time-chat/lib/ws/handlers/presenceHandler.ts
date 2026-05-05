import { usePresenceStore } from "@/store/presence.store";
import type { PresenceStatus } from "@/lib/ws/types";
import type { z } from "zod";
import type {
  UserOnlineSchema,
  UserOfflineSchema,
  PresenceUpdatedSchema,
  PresenceSyncSchema,
} from "@/lib/ws/types";

type UserOnline = z.infer<typeof UserOnlineSchema>;
type UserOffline = z.infer<typeof UserOfflineSchema>;
type PresenceUpdated = z.infer<typeof PresenceUpdatedSchema>;
type PresenceSync = z.infer<typeof PresenceSyncSchema>;

export const presenceHandler = {
  onUserOnline(payload: UserOnline): void {
    const status = payload.status ?? "online";
    console.debug(`Presence event: UserOnline ${payload.user_id} ${status}`);
    usePresenceStore.getState().setUserPresence(payload.user_id, status);
  },

  onUserOffline(payload: UserOffline): void {
    console.debug(`Presence event: UserOffline ${payload.user_id} offline`);
    usePresenceStore.getState().setUserOffline(payload.user_id);
  },

  onPresenceUpdated(payload: PresenceUpdated): void {
    console.debug(`Presence event: PresenceUpdated ${payload.user_id} ${payload.status}`);
    usePresenceStore
      .getState()
      .setUserPresence(payload.user_id, payload.status, payload.last_activity);
  },

  onPresenceSync(payload: PresenceSync): void {
    console.debug(`Presence event: PresenceSync ${payload.users.length} users`);
    const store = usePresenceStore.getState();
    for (const u of payload.users) {
      if (u.status === "offline") {
        store.setUserOffline(u.user_id);
      } else {
        store.setUserPresence(u.user_id, u.status as PresenceStatus);
      }
    }
  },
};
