"use client";

import { useEffect } from "react";
import { useDmStore } from "@/store/dm.store";
import { usePresenceStore } from "@/store/presence.store";
import { MessageSquare } from "lucide-react";

export function DmSidebar() {
  const conversations = useDmStore((s) => s.conversations);
  const activePeerId = useDmStore((s) => s.activePeerId);
  const setActivePeer = useDmStore((s) => s.setActivePeer);
  const fetchConversations = useDmStore((s) => s.fetchConversations);
  const loading = useDmStore((s) => s.loading);

  const presence = usePresenceStore((s) => s.presence);

  useEffect(() => {
    fetchConversations();
  }, [fetchConversations]);

  return (
    <aside className="h-full flex flex-col bg-[var(--card)] border-r border-[var(--border)]">
      {/* Header */}
      <div className="h-12 px-4 flex items-center border-b border-[var(--border)] shrink-0">
        <MessageSquare className="w-5 h-5 text-[#023BFC] mr-2" />
        <h2 className="font-bold text-sm text-[var(--foreground)]">
          Messages privés
        </h2>
      </div>

      {/* Conversation list */}
      <div className="flex-1 overflow-y-auto py-2">
        {loading && conversations.length === 0 && (
          <div className="px-4 py-8 text-sm text-[var(--muted-foreground)] text-center">
            Chargement...
          </div>
        )}

        {!loading && conversations.length === 0 && (
          <div className="px-4 py-8 text-sm text-[var(--muted-foreground)] text-center">
            Aucune conversation privée.
            <br />
            <span className="text-xs">
              Envoie un message à un membre depuis un serveur pour commencer.
            </span>
          </div>
        )}

        {conversations.map((conv) => {
          const active = conv.peer_id === activePeerId;
          const status = presence?.[conv.peer_id]?.status ?? "offline";
          const isOnline = status === "online";

          return (
            <button
              key={conv.conversation_id}
              onClick={() => setActivePeer(conv.peer_id)}
              className={[
                "w-full px-3 py-2.5 flex items-center gap-3 transition-all duration-200",
                active
                  ? "bg-[#023BFC]/10 border-l-2 border-[#023BFC]"
                  : "hover:bg-[var(--surface)] border-l-2 border-transparent",
              ].join(" ")}
            >
              {/* Avatar */}
              <div className="relative shrink-0">
                <div
                  className={[
                    "w-10 h-10 rounded-full flex items-center justify-center text-xs font-semibold",
                    active
                      ? "bg-gradient-to-br from-[#023BFC] to-[#3D6AFF] text-white"
                      : "bg-zinc-200 dark:bg-zinc-700 text-zinc-700 dark:text-zinc-100",
                  ].join(" ")}
                >
                  {conv.peer_username.slice(0, 2).toUpperCase()}
                </div>
                {/* Presence indicator */}
                <div
                  className={[
                    "absolute -bottom-0.5 -right-0.5 w-3.5 h-3.5 border-2 border-white dark:border-[#2b2d31] rounded-full",
                    status === "online"
                      ? "bg-emerald-500"
                      : status === "idle"
                        ? "bg-red-500"
                        : status === "dnd"
                          ? "bg-amber-500"
                          : "bg-gray-400",
                  ].join(" ")}
                  title={status}
                />
              </div>

              <div className="flex-1 min-w-0 text-left">
                <div
                  className={[
                    "text-sm font-semibold truncate",
                    active
                      ? "text-[#023BFC]"
                      : "text-zinc-800 dark:text-zinc-100",
                  ].join(" ")}
                >
                  {conv.peer_username}
                </div>
                <div className="text-xs text-[var(--muted-foreground)] truncate max-w-[160px]">
                  {(() => {
                    try {
                      const p = JSON.parse(conv.last_message);
                      if (p?.type === "gif") return "🖼 GIF";
                    } catch { /* not JSON */ }
                    return conv.last_message.length > 40
                      ? conv.last_message.slice(0, 40) + "..."
                      : conv.last_message;
                  })()}
                </div>
              </div>

              {/* Time */}
              {conv.last_message_at && (
                <div className="text-[10px] text-[var(--muted-foreground)] shrink-0">
                  {formatTime(conv.last_message_at)}
                </div>
              )}
            </button>
          );
        })}
      </div>
    </aside>
  );
}

function formatTime(dateStr: string): string {
  try {
    const d = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
      return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    }
    if (diffDays === 1) return "Hier";
    if (diffDays < 7) return d.toLocaleDateString([], { weekday: "short" });
    return d.toLocaleDateString([], { day: "2-digit", month: "2-digit" });
  } catch {
    return "";
  }
}
