import { create } from "zustand";
import type { PresenceStatus } from "@/lib/ws/types";

type PresenceRecord = {
  status: PresenceStatus;
  last_activity?: string;
};

type PresenceState = {
  /** userId → presence record — single source of truth */
  presence: Record<string, PresenceRecord>;

  setUserPresence(userId: string, status: PresenceStatus, last_activity?: string): void;
  setUserOffline(userId: string): void;
  reset(): void;
};

export const usePresenceStore = create<PresenceState>((set) => ({
  presence: {},

  setUserPresence(userId, status, last_activity) {
    console.debug(`[PresenceStore] setUserPresence: ${userId} → ${status}`);
    set((s) => ({
      presence: {
        ...s.presence,
        [userId]: {
          status,
          last_activity: last_activity ?? new Date().toISOString(),
        },
      },
    }));
  },

  setUserOffline(userId) {
    set((s) => ({
      presence: {
        ...s.presence,
        [userId]: { status: "offline", last_activity: new Date().toISOString() },
      },
    }));
  },

  reset() {
    set({ presence: {} });
  },
}));
