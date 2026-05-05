import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { DmConversation } from "@/lib/api/dm.api";
import { dmApi } from "@/lib/api";
import type { WsMessage } from "@/lib/ws/types";

type DmState = {
  conversations: DmConversation[];
  activePeerId: string | null;
  loading: boolean;
  /** conversationId → ordered DM messages */
  dmMessages: Record<string, WsMessage[]>;
  /** conversationId → oldest message id cursor */
  dmCursors: Record<string, string | null>;

  setConversations(conversations: DmConversation[]): void;
  setActivePeer(peerId: string | null): void;
  fetchConversations(): Promise<void>;
  addOrUpdateConversation(conv: DmConversation): void;

  setDmMessages(conversationId: string, msgs: WsMessage[]): void;
  prependDmMessages(conversationId: string, older: WsMessage[]): void;
  setDmCursor(conversationId: string, cursor: string | null): void;
  clearDmMessages(conversationId: string): void;

  addDmMessage(conversationId: string, msg: WsMessage): void;
  updateDmMessage(conversationId: string, id: string, patch: Partial<WsMessage>): void;
  removeDmMessage(conversationId: string, id: string): void;
  getDmMessages(conversationId: string): WsMessage[];
};

export const useDmStore = create<DmState>((set, get) => ({
  conversations: [],
  activePeerId: null,
  loading: false,
  dmMessages: {},
  dmCursors: {},

  setConversations(conversations) {
    set({ conversations });
  },

  setActivePeer(peerId) {
    set({ activePeerId: peerId });
  },

  async fetchConversations() {
    set({ loading: true });
    try {
      const data = await dmApi.listConversations();
      set({ conversations: data });
    } catch (e) {
      console.error("Failed to fetch DM conversations:", e);
    } finally {
      set({ loading: false });
    }
  },

  addOrUpdateConversation(conv) {
    set((s) => {
      const idx = s.conversations.findIndex(
        (c) => c.conversation_id === conv.conversation_id,
      );
      if (idx >= 0) {
        const updated = [...s.conversations];
        updated[idx] = conv;
        updated.sort((a, b) =>
          b.last_message_at.localeCompare(a.last_message_at),
        );
        return { conversations: updated };
      }
      return { conversations: [conv, ...s.conversations] };
    });
  },

  setDmMessages(conversationId, incoming) {
    set((s) => {
      const existing = s.dmMessages[conversationId] ?? [];
      const byId = new Map<string, WsMessage>();
      for (const m of existing) byId.set(m.id, m);
      for (const m of incoming) byId.set(m.id, m);
      const merged = Array.from(byId.values()).sort((a, b) =>
        a.created_at.localeCompare(b.created_at),
      );
      return { dmMessages: { ...s.dmMessages, [conversationId]: merged } };
    });
  },

  prependDmMessages(conversationId, older) {
    set((s) => {
      const existing = s.dmMessages[conversationId] ?? [];
      const existingIds = new Set(existing.map((m) => m.id));
      const newMsgs = older.filter((m) => !existingIds.has(m.id));
      if (newMsgs.length === 0) return s;
      return {
        dmMessages: {
          ...s.dmMessages,
          [conversationId]: [...newMsgs, ...existing],
        },
      };
    });
  },

  setDmCursor(conversationId, cursor) {
    set((s) => ({ dmCursors: { ...s.dmCursors, [conversationId]: cursor } }));
  },

  clearDmMessages(conversationId) {
    set((s) => {
      const msgs = { ...s.dmMessages };
      const curs = { ...s.dmCursors };
      delete msgs[conversationId];
      delete curs[conversationId];
      return { dmMessages: msgs, dmCursors: curs };
    });
  },

  addDmMessage(conversationId, msg) {
    set((s) => {
      const existing = s.dmMessages[conversationId] ?? [];
      if (existing.some((m) => m.id === msg.id)) return s;
      return {
        dmMessages: {
          ...s.dmMessages,
          [conversationId]: [...existing, msg],
        },
      };
    });
  },

  updateDmMessage(conversationId, id, patch) {
    set((s) => {
      const msgs = s.dmMessages[conversationId] ?? [];
      return {
        dmMessages: {
          ...s.dmMessages,
          [conversationId]: msgs.map((m) => (m.id === id ? { ...m, ...patch } : m)),
        },
      };
    });
  },

  removeDmMessage(conversationId, id) {
    set((s) => ({
      dmMessages: {
        ...s.dmMessages,
        [conversationId]: (s.dmMessages[conversationId] ?? []).filter(
          (m) => m.id !== id,
        ),
      },
    }));
  },

  getDmMessages(conversationId) {
    return get().dmMessages[conversationId] ?? [];
  },
}));
