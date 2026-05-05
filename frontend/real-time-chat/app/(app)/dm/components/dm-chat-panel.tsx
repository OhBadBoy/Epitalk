"use client";

import { useEffect, useLayoutEffect, useMemo, useRef, useState, useCallback } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Paperclip, FileText, Smile, Gift, Sticker, Send, Loader2, CornerUpLeft, Edit3, Trash2, X, ArrowLeft } from "lucide-react";
import { useWebSocketStore, type WsMessage } from "@/store/websocket.store";
import { useAuthStore } from "@/store/auth.store";
import { useDmStore } from "@/store/dm.store";
import { useAppearanceStore } from "@/store/appearance.store";
import { useNotificationClick } from "@/hooks/use-notification-click";
import { dmApi } from "@/lib/api";

const FONT_SIZE_MAP = { sm: "14px", base: "16px", lg: "18px", xl: "20px" } as const;

function dmConversationId(a: string, b: string): string {
  return a < b ? `dm:${a}:${b}` : `dm:${b}:${a}`;
}

export function DmChatPanel() {
  const user = useAuthStore((s) => s.user);
  const token = useAuthStore((s) => s.token);

  const activePeerId = useDmStore((s) => s.activePeerId);
  const setActivePeer = useDmStore((s) => s.setActivePeer);
  const conversations = useDmStore((s) => s.conversations);

  const isConnected = useWebSocketStore((s) => s.isConnected);
  const connect = useWebSocketStore((s) => s.connect);
  const sendDm = useWebSocketStore((s) => s.sendDm);
  const editDm = useWebSocketStore((s) => s.editDm);
  const deleteDm = useWebSocketStore((s) => s.deleteDm);
  const sendDmGif = useWebSocketStore((s) => s.sendDmGif);
  const joinDm = useWebSocketStore((s) => s.joinDm);
  const leaveDm = useWebSocketStore((s) => s.leaveDm);
  const socket = useWebSocketStore((s) => s.socket);

  // DM message store (source of truth)
  const dmMessagesByConv = useDmStore((s) => s.dmMessages);
  const setDmMsgs = useDmStore((s) => s.setDmMessages);
  const prependDmMsgs = useDmStore((s) => s.prependDmMessages);
  const setDmCursor = useDmStore((s) => s.setDmCursor);
  const clearDmMessages = useDmStore((s) => s.clearDmMessages);
  const fontSize = useAppearanceStore((s) => s.fontSize);
  const chatFontSize = FONT_SIZE_MAP[fontSize] || "16px";

  const QUICK_EMOJIS = ["👍", "❤️", "😂", "😮", "😢", "🔥", "🎉", "👀"];

  const conversationId = useMemo(() => {
    if (!user || !activePeerId) return null;
    return dmConversationId(user.id, activePeerId);
  }, [user, activePeerId]);

  const peerUsername = useMemo(() => {
    if (!activePeerId) return null;
    const conv = conversations.find((c) => c.peer_id === activePeerId);
    return conv?.peer_username ?? activePeerId.slice(0, 8);
  }, [activePeerId, conversations]);

  const messages: WsMessage[] = useMemo(() => {
    if (!conversationId) return [];
    const list = dmMessagesByConv[conversationId] ?? [];
    return [...list].sort((a, b) => a.created_at.localeCompare(b.created_at));
  }, [conversationId, dmMessagesByConv]);

  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [value, setValue] = useState("");
  const [attachmentFile, setAttachmentFile] = useState<File | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const [replyTo, setReplyTo] = useState<string | null>(null);
  const [replyToUsername, setReplyToUsername] = useState<string | null>(null);
  const [editingMessageId, setEditingMessageId] = useState<string | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [emojiPickerMsgId, setEmojiPickerMsgId] = useState<string | null>(null);
  const [showInputEmojis, setShowInputEmojis] = useState(false);

  // GIF lightbox state
  const [openGifLightbox, setOpenGifLightbox] = useState<string | null>(null);

  // GIF picker state
  const [openGifPicker, setOpenGifPicker] = useState<"input" | null>(null);
  const [gifQuery, setGifQuery] = useState("");
  const [gifResults, setGifResults] = useState<{ id: string; url: string; preview?: string; provider?: string }[]>([]);
  const [gifLoading, setGifLoading] = useState(false);

  const bottomRef = useRef<HTMLDivElement | null>(null);
  const scrollContainerRef = useRef<HTMLDivElement | null>(null);
  const scrollAnchorRef = useRef<number | null>(null);
  const prevPeerRef = useRef<string | null>(null);

  // Handle notification clicks
  useNotificationClick();
  const [hasMore, setHasMore] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);

  // Connect WebSocket
  useEffect(() => {
    if (token && !isConnected) {
      connect(token);
    }
  }, [token, isConnected, connect]);

  // Re-join DM conversation whenever the WebSocket (re)connects — backend
  // requires an explicit JoinDm even after a page refresh.
  useEffect(() => {
    if (!isConnected || !activePeerId) return;
    joinDm(activePeerId);
  }, [isConnected, activePeerId, joinDm]);

  // On peer change: leave old, purge cache, fetch if empty, join new
  useEffect(() => {
    const prevPeer = prevPeerRef.current;
    if (prevPeer && prevPeer !== activePeerId) {
      leaveDm(prevPeer);
      if (user) {
        const oldConvId = dmConversationId(user.id, prevPeer);
        clearDmMessages(oldConvId);
      }
    }
    prevPeerRef.current = activePeerId;

    if (!activePeerId || !isConnected || !user) return;

    const convId = dmConversationId(user.id, activePeerId);
    setHasMore(true);
    setLoadingMore(false);

    const cached = useDmStore.getState().getDmMessages(convId);
    if (cached.length === 0) {
      void (async () => {
        try {
          const data = await dmApi.listMessages(activePeerId);
          const wsMsgs: WsMessage[] = data.map((m) => ({
            id: m.id,
            channel_id: m.conversation_id,
            author_id: m.author_id,
            username: m.username,
            content: m.content,
            created_at: m.created_at,
            reply_to: m.reply_to,
            attachment_url: m.attachment_url,
          }));
          setDmMsgs(convId, wsMsgs);
          if (wsMsgs.length > 0) setDmCursor(convId, wsMsgs[0].id);
          setHasMore(data.length >= 50);
        } catch (err) {
          console.warn("Failed to load DM history:", err);
        } finally {
          joinDm(activePeerId);
        }
      })();
    } else {
      joinDm(activePeerId);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activePeerId, isConnected]);

  // Scroll to bottom for new messages only
  useEffect(() => {
    if (scrollAnchorRef.current !== null) return;
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Restore scroll position after prepend
  useLayoutEffect(() => {
    if (scrollAnchorRef.current === null) return;
    const el = scrollContainerRef.current;
    if (el) el.scrollTop = el.scrollHeight - scrollAnchorRef.current;
    scrollAnchorRef.current = null;
  }, [messages]);

  // Load older DM messages (infinite scroll up)
  const loadOlderDmMessages = useCallback(async () => {
    if (!activePeerId || !user || !hasMore || loadingMore) return;
    const convId = dmConversationId(user.id, activePeerId);
    const msgs = useDmStore.getState().getDmMessages(convId);
    if (!msgs.length) return;

    const el = scrollContainerRef.current;
    if (el) scrollAnchorRef.current = el.scrollHeight - el.scrollTop;

    setLoadingMore(true);
    try {
      const data = await dmApi.listMessages(activePeerId, { before: msgs[0].id });
      if (data.length > 0) {
        const wsMsgs: WsMessage[] = data.map((m) => ({
          id: m.id,
          channel_id: m.conversation_id,
          author_id: m.author_id,
          username: m.username,
          content: m.content,
          created_at: m.created_at,
          reply_to: m.reply_to,
        }));
        prependDmMsgs(convId, wsMsgs);
        setDmCursor(convId, wsMsgs[0].id);
      }
      if (data.length < 50) setHasMore(false);
    } catch (err) {
      console.warn("Failed to load older DM messages:", err);
    } finally {
      setLoadingMore(false);
    }
  }, [activePeerId, user, hasMore, loadingMore, prependDmMsgs, setDmCursor]);

  const handleScroll = useCallback(
    (e: React.UIEvent<HTMLDivElement>) => {
      if (e.currentTarget.scrollTop < 150 && hasMore && !loadingMore) {
        void loadOlderDmMessages();
      }
    },
    [hasMore, loadingMore, loadOlderDmMessages],
  );

  // GIF search effect
  useEffect(() => {
    if (openGifPicker !== "input") return;
    const query = gifQuery && gifQuery.trim() !== "" ? gifQuery.trim() : "trending";
    const controller = new AbortController();
    setGifLoading(true);

    const timer = setTimeout(async () => {
      try {
        const res = await fetch(
          `/api/gifs/search?q=${encodeURIComponent(query)}&limit=24`,
          { signal: controller.signal },
        );
        if (!res.ok) { setGifResults([]); setGifLoading(false); return; }
        const json = await res.json();
        const results: { id: string; url: string; preview?: string; provider?: string }[] = [];
        if (json.results) {
          for (const it of json.results) {
            const id = it.id || "";
            const url = it.url || "";
            const preview = it.preview || undefined;
            const provider = it.provider || undefined;
            if (url) results.push({ id: id.toString(), url, preview, provider });
          }
        } else if (json.data) {
          for (const it of json.data) {
            const id = it.id || "";
            const url = it.images?.original?.url || it.images?.fixed_width?.url || "";
            const preview = it.images?.preview_gif?.url || it.images?.fixed_width_small_still?.url;
            if (url) results.push({ id: id.toString(), url, preview, provider: "giphy" });
          }
        }
        setGifResults(results);
      } catch (err: unknown) {
        if (!(err instanceof DOMException) || err.name !== "AbortError") {
          console.error("GIF search failed", err);
        }
        setGifResults([]);
      } finally {
        setGifLoading(false);
      }
    }, 300);

    return () => { clearTimeout(timer); controller.abort(); };
  }, [gifQuery, openGifPicker]);

  const getUsernameById = useCallback(
    (authorId: string, msgUsername?: string): string => {
      if (msgUsername) return msgUsername;
      if (user && user.id === authorId) return user.username;
      if (activePeerId === authorId && peerUsername) return peerUsername;
      return authorId.slice(0, 8);
    },
    [user, activePeerId, peerUsername],
  );

  const parseGifContent = useCallback((content: string) => {
    try {
      const parsed = JSON.parse(content);
      if (parsed?.type === "gif" && parsed?.gif?.url) {
        return parsed as {
          type: "gif";
          gif: { id: string; url: string; preview?: string; provider?: string };
          caption?: string;
        };
      }
    } catch {
      // Not JSON
    }
    return null;
  }, []);

  const onSend = async () => {
    if (!activePeerId || !conversationId) return;
    const content = value.trim();
    if (!content && !attachmentFile) return;
    if (!isConnected) {
      setError("Non connecté au serveur");
      return;
    }

    setSending(true);
    setError(null);

    try {
      let attachmentUrl: string | undefined;
      if (attachmentFile) {
        const result = await dmApi.uploadAttachment(attachmentFile);
        attachmentUrl = result.url;
      }

      if (isEditing && editingMessageId) {
        editDm(conversationId, editingMessageId, content);
        setEditingMessageId(null);
        setIsEditing(false);
      } else {
        sendDm(activePeerId, content, replyTo || undefined, attachmentUrl);
        setReplyTo(null);
        setReplyToUsername(null);
        setAttachmentFile(null);
      }
      setValue("");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Erreur envoi message");
    } finally {
      setSending(false);
    }
  };

  const cancelPendingAction = () => {
    setIsEditing(false);
    setEditingMessageId(null);
    setReplyTo(null);
    setValue("");
    setAttachmentFile(null);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      onSend();
    }
  };

  if (!activePeerId) {
    return (
      <div className="h-full border border-[var(--border)] min-w-0 flex flex-col bg-[var(--card)] shadow-lg overflow-hidden items-center justify-center">
        <div className="text-center text-[var(--muted-foreground)]">
          <div className="w-16 h-16 mx-auto mb-4 rounded-full bg-[var(--surface)] flex items-center justify-center">
            <Send className="w-8 h-8 text-[var(--muted-foreground)]" />
          </div>
          <p className="text-lg font-semibold text-[var(--muted-foreground)] mb-1">Messages privés</p>
          <p className="text-sm">Sélectionne une conversation pour commencer.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full border border-[var(--border)] min-w-0 flex flex-col bg-[var(--card)] shadow-lg overflow-hidden">
      {/* --- HEADER --- */}
      <div className="h-12 px-4 flex items-center border-b shadow-sm shrink-0">
        <button
          className="md:hidden mr-3 text-[var(--muted-foreground)] hover:text-[var(--foreground)] transition"
          onClick={() => setActivePeer(null)}
          title="Retour"
        >
          <ArrowLeft className="h-5 w-5" />
        </button>
        <div className="w-7 h-7 rounded-full bg-gradient-to-br from-[#023BFC] to-[#3D6AFF] flex items-center justify-center text-[10px] font-semibold text-white mr-3">
          {(peerUsername ?? "?").slice(0, 2).toUpperCase()}
        </div>
        <h2 className="font-bold text-md text-[var(--foreground)]">
          {peerUsername}
        </h2>
      </div>

      {/* --- MESSAGES --- */}
      <div
        ref={scrollContainerRef}
        className="flex-1 overflow-y-auto flex flex-col py-4"
        onScroll={handleScroll}
      >
        {loadingMore && (
          <div className="flex justify-center py-2">
            <Loader2 className="h-4 w-4 animate-spin text-[var(--muted-foreground)]" />
          </div>
        )}
        {messages.length === 0 && !isConnected ? (
          <div className="px-4 text-sm text-muted-foreground flex items-center gap-2">
            <Loader2 className="h-4 w-4 animate-spin" />
            Connexion au serveur...
          </div>
        ) : messages.length === 0 ? (
          <div className="px-4 text-sm text-muted-foreground">
            Aucun message. Écris le premier message à {peerUsername} !
          </div>
        ) : (
          <div className="flex flex-col mt-auto">
            {messages.map((msg) => {
              const isAuthor = user?.id === msg.author_id;
              const messageUsername = getUsernameById(msg.author_id, msg.username);
              return (
                <div
                  key={msg.id}
                  className="group relative flex items-start px-4 py-2 hover:bg-black/5 transition w-full"
                >
                  {/* Action buttons — top-right, visible on hover */}
                  <div className="absolute -top-3 right-4 hidden group-hover:flex items-center gap-0.5 bg-[var(--card)] border border-[var(--border)] rounded-md shadow-sm px-1 py-0.5 z-10">
                    <button
                      className="h-7 w-7 flex items-center justify-center text-[var(--muted-foreground)] hover:text-indigo-600 hover:bg-[var(--surface)] rounded transition"
                      onClick={() => {
                        setReplyTo(msg.id);
                        setReplyToUsername(messageUsername);
                        setIsEditing(false);
                        setEditingMessageId(null);
                        setValue("");
                      }}
                      title="Répondre"
                    >
                      <CornerUpLeft className="h-4 w-4" />
                    </button>
                    {isAuthor && (
                      <button
                        className="h-7 w-7 flex items-center justify-center text-[var(--muted-foreground)] hover:text-emerald-600 hover:bg-[var(--surface)] rounded transition"
                        onClick={() => {
                          setIsEditing(true);
                          setEditingMessageId(msg.id);
                          setReplyTo(null);
                          setReplyToUsername(null);
                          setValue(msg.content);
                        }}
                        title="Modifier"
                      >
                        <Edit3 className="h-4 w-4" />
                      </button>
                    )}
                    {isAuthor && conversationId && (
                      <button
                        className="h-7 w-7 flex items-center justify-center text-[var(--muted-foreground)] hover:text-red-600 hover:bg-[var(--surface)] rounded transition"
                        onClick={() => deleteDm(conversationId, msg.id)}
                        title="Supprimer"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    )}
                    <button
                      className="h-7 w-7 flex items-center justify-center text-[var(--muted-foreground)] hover:text-amber-500 hover:bg-[var(--surface)] rounded transition"
                      onClick={() => setEmojiPickerMsgId(emojiPickerMsgId === msg.id ? null : msg.id)}
                      title="Réaction"
                    >
                      <Smile className="h-4 w-4" />
                    </button>
                  </div>

                  {/* Quick emoji picker */}
                  {emojiPickerMsgId === msg.id && (
                    <div className="absolute -top-3 right-4 z-20 flex items-center gap-1 bg-[var(--card)] border border-[var(--border)] rounded-lg shadow-lg px-2 py-1">
                      {QUICK_EMOJIS.map((emoji) => (
                        <button
                          key={emoji}
                          className="text-lg hover:scale-125 transition-transform p-0.5"
                          onClick={() => {
                            if (socket && socket.readyState === WebSocket.OPEN) {
                              socket.send(JSON.stringify({
                                type: "ReactionAdd",
                                payload: { message_id: msg.id, emoji },
                              }));
                            }
                            setEmojiPickerMsgId(null);
                          }}
                        >
                          {emoji}
                        </button>
                      ))}
                      <button
                        className="ml-1 text-[var(--muted-foreground)] hover:text-[var(--muted-foreground)] text-xs"
                        onClick={() => setEmojiPickerMsgId(null)}
                      >
                        <X className="h-3.5 w-3.5" />
                      </button>
                    </div>
                  )}

                  <div className="mr-4">
                    <div className="w-10 h-10 rounded-full bg-[var(--muted)] flex items-center justify-center text-xs font-semibold text-[var(--foreground)]">
                      {messageUsername.slice(0, 2).toUpperCase()}
                    </div>
                  </div>

                  <div className="flex flex-col w-full">
                    <div className="flex items-center gap-x-2">
                      <span className="font-semibold text-sm text-[var(--foreground)]">
                        {messageUsername}
                      </span>
                      <span className="text-xs text-[var(--muted-foreground)]">
                        {new Date(msg.created_at).toLocaleString()}
                      </span>
                      {msg.edited_at && (
                        <span className="text-xs text-indigo-500">(édité)</span>
                      )}
                    </div>

                    {msg.reply_to && (() => {
                      const original = messages.find((m) => m.id === msg.reply_to);
                      return (
                        <div className="text-xs text-[var(--muted-foreground)] italic mb-1 bg-[var(--surface)] p-2 rounded-md max-h-16 overflow-hidden">
                          Réponse à {original ? getUsernameById(original.author_id, original.username) : msg.reply_to.slice(0, 8)}:
                          <div className="truncate max-w-full">
                            {original
                              ? original.content.startsWith('{"type":"gif"')
                                ? "🖼 GIF"
                                : original.content
                              : "message introuvable"}
                          </div>
                        </div>
                      );
                    })()}

                    {(() => {
                      const gifMessage = parseGifContent(msg.content);
                      if (gifMessage) {
                        return (
                          <div className="mt-2">
                            {/* eslint-disable-next-line @next/next/no-img-element */}
                            <img
                              src={gifMessage.gif.url}
                              alt={gifMessage.caption || "GIF"}
                              className="max-h-90 max-w-[70%] rounded-md object-contain cursor-pointer"
                              onClick={() => setOpenGifLightbox(gifMessage.gif.url)}
                            />
                            {gifMessage.caption && (
                              <p className="mt-1 text-sm text-[var(--muted-foreground)]">{gifMessage.caption}</p>
                            )}
                          </div>
                        );
                      }
                      return (
                        <>
                          {msg.content && (
                            <p className="text-[var(--muted-foreground)] whitespace-pre-wrap" style={{ fontSize: chatFontSize }}>
                              {msg.content}
                            </p>
                          )}
                          {msg.attachment_url && (() => {
                            const raw = msg.attachment_url!;
                            const base = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3001";
                            const url = raw.startsWith("http") ? raw : `${base}${raw}`;
                            const isImage = /\.(jpg|jpeg|png|gif|webp)(\?.*)?$/i.test(raw);
                            if (isImage) {
                              return (
                                // eslint-disable-next-line @next/next/no-img-element
                                <img
                                  src={url}
                                  alt="attachment"
                                  className="mt-2 max-w-xs max-h-64 rounded-md object-contain cursor-pointer"
                                  onClick={() => setOpenGifLightbox(url)}
                                />
                              );
                            }
                            const filename = raw.split("/").pop() ?? raw;
                            const displayName = filename.length > 30 ? `${filename.slice(0, 30)}...` : filename;
                            return (
                              <a
                                href={url}
                                target="_blank"
                                rel="noopener noreferrer"
                                download
                                className="mt-2 inline-flex items-center gap-2 text-sm text-indigo-600 hover:text-indigo-700 underline"
                              >
                                <FileText className="h-4 w-4 shrink-0" />
                                {displayName}
                              </a>
                            );
                          })()}
                        </>
                      );
                    })()}

                    {/* Reactions display */}
                    {msg.reactions && msg.reactions.length > 0 && (
                      <div className="mt-1.5 flex flex-wrap gap-1">
                        {Object.entries(
                          msg.reactions.reduce<Record<string, { count: number; users: string[]; userIds: string[] }>>((acc, r) => {
                            if (!acc[r.emoji]) acc[r.emoji] = { count: 0, users: [], userIds: [] };
                            acc[r.emoji].count++;
                            acc[r.emoji].users.push(r.username || r.user_id.slice(0, 6));
                            acc[r.emoji].userIds.push(r.user_id);
                            return acc;
                          }, {})
                        ).map(([emoji, data]) => {
                          const hasReacted = data.userIds.includes(user?.id || "");
                          return (
                            <button
                              key={emoji}
                              className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs border transition ${
                                hasReacted
                                  ? "bg-indigo-100 border-indigo-300 text-indigo-700"
                                  : "bg-[var(--surface)] border-[var(--border)] text-[var(--muted-foreground)] hover:bg-[var(--muted)]"
                              }`}
                              title={data.users.join(", ")}
                              onClick={() => {
                                if (socket && socket.readyState === WebSocket.OPEN) {
                                  socket.send(JSON.stringify({
                                    type: "ReactionAdd",
                                    payload: { message_id: msg.id, emoji },
                                  }));
                                }
                              }}
                            >
                              <span>{emoji}</span>
                              <span>{data.count}</span>
                            </button>
                          );
                        })}
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
            <div ref={bottomRef} />

            {/* GIF lightbox */}
            {openGifLightbox && (
              <div
                className="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4"
                onClick={() => setOpenGifLightbox(null)}
              >
                {/* eslint-disable-next-line @next/next/no-img-element */}
                <img
                  src={openGifLightbox}
                  alt="GIF full"
                  className="max-h-[90vh] max-w-[90vw] object-contain rounded-md shadow-lg"
                  onClick={(e) => e.stopPropagation()}
                />
              </div>
            )}
          </div>
        )}

        {!isConnected && messages.length > 0 && (
          <div className="px-4 py-1 text-xs text-[var(--muted-foreground)] flex items-center gap-2">
            <Loader2 className="h-3 w-3 animate-spin" />
            Reconnexion en cours...
          </div>
        )}

        {error && <div className="px-4 mt-2 text-xs text-red-500">{error}</div>}
      </div>
      <div className="p-4 mb-2 shrink-0">
        {((replyTo && !isEditing) || isEditing) && (
          <div className="mb-2 rounded-md border border-indigo-200 bg-indigo-50 p-2 text-xs text-indigo-700 flex items-center justify-between">
            <div>
              {isEditing ? "Modification du message en cours" : "Réponse en cours"}
              {replyTo && !isEditing && replyToUsername ? `: ${replyToUsername}` : ""}
            </div>
            <button
              className="text-indigo-700 underline text-xs flex items-center gap-1"
              onClick={cancelPendingAction}
            >
              <X className="h-3 w-3" /> Annuler
            </button>
          </div>
        )}

        {/* Attachment preview */}
        {attachmentFile && (
          <div className="mb-2 flex items-center gap-3 px-3 py-2 rounded-lg border border-[var(--border)] bg-[var(--surface)] text-sm">
            {attachmentFile.type.startsWith("image/") ? (
              // eslint-disable-next-line @next/next/no-img-element
              <img
                src={URL.createObjectURL(attachmentFile)}
                alt="preview"
                className="h-16 w-16 rounded object-cover shrink-0"
              />
            ) : (
              <FileText className="h-8 w-8 text-indigo-500 shrink-0" />
            )}
            <div className="flex-1 min-w-0">
              <p className="truncate font-medium text-[var(--foreground)]">{attachmentFile.name}</p>
              <p className="text-xs text-[var(--muted-foreground)]">
                {attachmentFile.size < 1024 * 1024
                  ? `${(attachmentFile.size / 1024).toFixed(1)} KB`
                  : `${(attachmentFile.size / (1024 * 1024)).toFixed(1)} MB`}
              </p>
            </div>
            <button
              type="button"
              onClick={() => setAttachmentFile(null)}
              className="text-[var(--muted-foreground)] hover:text-red-500 transition shrink-0"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        )}

        <input
          ref={fileInputRef}
          type="file"
          className="hidden"
          accept="image/jpeg,image/png,image/gif,image/webp,application/pdf"
          onChange={(e) => {
            const file = e.target.files?.[0];
            if (file) {
              if (file.size > 10 * 1024 * 1024) {
                setError("Le fichier doit faire moins de 10 Mo");
                return;
              }
              setAttachmentFile(file);
            }
            e.target.value = "";
          }}
        />

        <div className="relative flex items-center gap-2">
          <div className="relative flex-1">
            <button
              type="button"
              className="absolute left-4 top-1/2 -translate-y-1/2 h-6 w-6 bg-[var(--muted-foreground)] hover:bg-[var(--muted-foreground)] transition rounded-full p-1 flex items-center justify-center text-white"
              onClick={() => fileInputRef.current?.click()}
              title="Joindre un fichier"
            >
              <Paperclip className="text-white h-3.5 w-3.5" />
            </button>

            <Input
              value={value}
              onChange={(e) => setValue(e.target.value)}
              onKeyDown={handleKeyDown}
              disabled={!activePeerId || sending}
              className="px-14 pr-32 py-6 bg-[var(--muted)]/90 border-none border-0 focus-visible:ring-0 focus-visible:ring-offset-0 text-[var(--muted-foreground)] placeholder:text-[var(--muted-foreground)]"
              placeholder={`Envoyer un message à ${peerUsername ?? ""}`}
            />

            <div className="absolute right-4 top-1/2 -translate-y-1/2 flex items-center gap-x-3">
              <span title="Fonction à venir">
                <Gift className="h-5 w-5 text-[var(--muted-foreground)] hover:text-[var(--muted-foreground)] transition cursor-not-allowed" />
              </span>

              <button
                type="button"
                onClick={() => setOpenGifPicker(openGifPicker === "input" ? null : "input")}
                title="GIF"
                className="h-6 w-6 flex items-center justify-center"
              >
                <Sticker className="h-5 w-5 text-[var(--muted-foreground)] hover:text-[var(--muted-foreground)] transition cursor-pointer" />
              </button>

              <button
                type="button"
                onClick={() => setShowInputEmojis(!showInputEmojis)}
                title="Emojis"
                className="h-6 w-6 flex items-center justify-center"
              >
                <Smile className="h-5 w-5 text-[var(--muted-foreground)] hover:text-[var(--muted-foreground)] transition cursor-pointer" />
              </button>
              {showInputEmojis && (
                <div className="absolute bottom-10 right-0 z-50 bg-[var(--card)] border border-[var(--border)] rounded-lg shadow-lg px-2 py-2 flex flex-wrap gap-1 w-48">
                  {QUICK_EMOJIS.map((emoji) => (
                    <button
                      key={emoji}
                      className="text-xl hover:scale-125 transition-transform p-1 rounded hover:bg-[var(--surface)]"
                      onClick={() => {
                        setValue((prev) => prev + emoji);
                        setShowInputEmojis(false);
                      }}
                    >
                      {emoji}
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Bouton Envoyer */}
          <Button
            onClick={onSend}
            disabled={!activePeerId || sending || !isConnected || (!value.trim() && !attachmentFile)}
            className="h-12 px-4 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            title="Envoyer le message"
          >
            {sending ? (
              <Loader2 className="h-5 w-5 animate-spin" />
            ) : (
              <Send className="h-5 w-5" />
            )}
          </Button>
        </div>

        {/* GIF picker anchored to input bar */}
        {openGifPicker === "input" && (
          <div className="absolute left-4 right-4 bottom-20 z-50 bg-[var(--card)] border rounded-lg shadow-md p-3 w-[min(90vw,40rem)]">
            <div className="flex gap-2 mb-2">
              <input
                value={gifQuery}
                onChange={(e) => setGifQuery(e.target.value)}
                placeholder="Recherche de GIFs"
                className="flex-1 px-2 py-1 border rounded bg-[var(--surface)]"
              />
              <button
                onClick={async () => {
                  setGifLoading(true);
                  try {
                    const res = await fetch(
                      `/api/gifs/search?q=${encodeURIComponent(gifQuery || "trending")}&limit=24`,
                    );
                    if (!res.ok) { setGifResults([]); setGifLoading(false); return; }
                    const json = await res.json();
                    const results: { id: string; url: string; preview?: string; provider?: string }[] = [];
                    if (json.results) {
                      for (const it of json.results) {
                        const id = it.id || "";
                        const url = it.url || "";
                        const preview = it.preview || undefined;
                        const provider = it.provider || undefined;
                        if (url) results.push({ id: id.toString(), url, preview, provider });
                      }
                    } else if (json.data) {
                      for (const it of json.data) {
                        const id = it.id || "";
                        const url = it.images?.original?.url || it.images?.fixed_width?.url || "";
                        const preview = it.images?.preview_gif?.url || it.images?.fixed_width_small_still?.url;
                        if (url) results.push({ id: id.toString(), url, preview, provider: "giphy" });
                      }
                    }
                    setGifResults(results);
                  } catch (err) {
                    console.error("GIF search failed", err);
                    setGifResults([]);
                  } finally {
                    setGifLoading(false);
                  }
                }}
                className="px-2 py-1 bg-indigo-600 text-white rounded"
                disabled={gifLoading}
              >
                {gifLoading ? <Loader2 className="h-4 w-4 animate-spin" /> : "Rechercher"}
              </button>
              <button
                onClick={() => { setOpenGifPicker(null); setGifResults([]); setGifQuery(""); }}
                className="px-2 py-1 bg-[var(--muted)] rounded"
              >
                Fermer
              </button>
            </div>
            <div className="grid grid-cols-6 gap-2 max-h-64 overflow-auto">
              {gifResults.map((g) => (
                // eslint-disable-next-line @next/next/no-img-element
                <img
                  key={g.id}
                  src={g.preview || g.url}
                  alt="gif"
                  className="h-20 w-full object-cover rounded cursor-pointer"
                  onClick={() => {
                    try {
                      if (activePeerId) {
                        sendDmGif(activePeerId, {
                          id: g.id,
                          url: g.url,
                          preview: g.preview,
                          provider: g.provider,
                        }, null);
                      }
                    } catch (e) {
                      console.error("Failed to send gif", e);
                      setError("Erreur lors de l'envoi du GIF");
                    } finally {
                      setOpenGifPicker(null);
                      setGifResults([]);
                      setGifQuery("");
                    }
                  }}
                />
              ))}
            </div>
          </div>
        )}

        {/* Connection status */}
        {!isConnected && (
          <div className="mt-2 text-xs text-amber-600 flex items-center gap-1">
            <Loader2 className="h-3 w-3 animate-spin" />
            Reconnexion en cours...
          </div>
        )}
      </div>
    </div>
  );
}
