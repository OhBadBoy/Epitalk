import { create } from "zustand";

// Module-level timer map: key = `${channelId}::${username}`
const _timers = new Map<string, ReturnType<typeof setTimeout>>();

function _key(channelId: string, username: string): string {
  return `${channelId}::${username}`;
}

function _clearTimer(channelId: string, username: string): void {
  const t = _timers.get(_key(channelId, username));
  if (t) {
    clearTimeout(t);
    _timers.delete(_key(channelId, username));
  }
}

const AUTO_CLEAR_MS = 3_000;

type TypingState = {
  /** channelId → array of usernames currently typing */
  typingUsers: Record<string, string[]>;

  /** Start typing for a user. Resets the 3-second auto-clear timer. */
  addTypingUser(channelId: string, username: string): void;
  /** Stop typing for a user immediately. */
  removeTypingUser(channelId: string, username: string): void;
  /** Clear all typing users for a channel (e.g. on channel leave). */
  clearChannel(channelId: string): void;
};

export const useTypingStore = create<TypingState>((set, get) => ({
  typingUsers: {},

  addTypingUser(channelId, username) {
    _clearTimer(channelId, username);

    set((s) => {
      const current = s.typingUsers[channelId] ?? [];
      if (current.includes(username)) return s;
      return {
        typingUsers: { ...s.typingUsers, [channelId]: [...current, username] },
      };
    });

    // Auto-clear if no TypingStop arrives within 3s
    const timer = setTimeout(() => {
      get().removeTypingUser(channelId, username);
    }, AUTO_CLEAR_MS);
    _timers.set(_key(channelId, username), timer);
  },

  removeTypingUser(channelId, username) {
    _clearTimer(channelId, username);
    set((s) => ({
      typingUsers: {
        ...s.typingUsers,
        [channelId]: (s.typingUsers[channelId] ?? []).filter((u) => u !== username),
      },
    }));
  },

  clearChannel(channelId) {
    const users = useTypingStore.getState().typingUsers[channelId] ?? [];
    for (const u of users) _clearTimer(channelId, u);
    set((s) => {
      const t = { ...s.typingUsers };
      delete t[channelId];
      return { typingUsers: t };
    });
  },
}));
