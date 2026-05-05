import { create } from "zustand";
import type { WsMessage } from "@/lib/ws/types";

type MessageState = {
  /** channelId → ordered messages */
  messages: Record<string, WsMessage[]>;
  /** channelId → oldest message id (cursor for infinite scroll) */
  cursors: Record<string, string | null>;
  currentChannelId: string | null;

  setMessages(channelId: string, msgs: WsMessage[]): void;
  prependMessages(channelId: string, older: WsMessage[]): void;
  setCursor(channelId: string, cursor: string | null): void;
  addMessage(channelId: string, msg: WsMessage): void;
  updateMessage(channelId: string, id: string, patch: Partial<WsMessage>): void;
  removeMessage(channelId: string, id: string): void;
  setCurrentChannel(channelId: string | null): void;
  clearChannel(channelId: string): void;
  getMessages(channelId: string): WsMessage[];
};

export const useMessageStore = create<MessageState>((set, get) => ({
  messages: {},
  cursors: {},
  currentChannelId: null,

  setMessages(channelId, incoming) {
    set((s) => {
      const existing = s.messages[channelId] ?? [];
      const byId = new Map<string, WsMessage>();
      for (const m of existing) byId.set(m.id, m);
      // Incoming wins on conflict (edits/pins from REST history load)
      for (const m of incoming) byId.set(m.id, m);
      const merged = Array.from(byId.values()).sort((a, b) =>
        a.created_at.localeCompare(b.created_at),
      );
      return { messages: { ...s.messages, [channelId]: merged } };
    });
  },

  prependMessages(channelId, older) {
    set((s) => {
      const existing = s.messages[channelId] ?? [];
      const existingIds = new Set(existing.map((m) => m.id));
      const newMsgs = older.filter((m) => !existingIds.has(m.id));
      if (newMsgs.length === 0) return s;
      return {
        messages: { ...s.messages, [channelId]: [...newMsgs, ...existing] },
      };
    });
  },

  setCursor(channelId, cursor) {
    set((s) => ({ cursors: { ...s.cursors, [channelId]: cursor } }));
  },

  addMessage(channelId, msg) {
    set((s) => {
      const existing = s.messages[channelId] ?? [];
      if (existing.some((m) => m.id === msg.id)) return s;
      return {
        messages: { ...s.messages, [channelId]: [...existing, msg] },
      };
    });
  },

  updateMessage(channelId, id, patch) {
    set((s) => {
      const msgs = s.messages[channelId] ?? [];
      return {
        messages: {
          ...s.messages,
          [channelId]: msgs.map((m) => (m.id === id ? { ...m, ...patch } : m)),
        },
      };
    });
  },

  removeMessage(channelId, id) {
    set((s) => ({
      messages: {
        ...s.messages,
        [channelId]: (s.messages[channelId] ?? []).filter((m) => m.id !== id),
      },
    }));
  },

  setCurrentChannel(channelId) {
    set({ currentChannelId: channelId });
  },

  clearChannel(channelId) {
    set((s) => {
      const msgs = { ...s.messages };
      const curs = { ...s.cursors };
      delete msgs[channelId];
      delete curs[channelId];
      return { messages: msgs, cursors: curs };
    });
  },

  getMessages(channelId) {
    return get().messages[channelId] ?? [];
  },
}));
