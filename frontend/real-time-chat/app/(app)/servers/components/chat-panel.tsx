"use client";

import { useEffect, useLayoutEffect, useMemo, useRef, useState, useCallback } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import {
  Smile,
  Gift,
  Sticker,
  Send,
  Loader2,
  CornerUpLeft,
  Edit3,
  Trash2,
  X,
  Search,
  Pin,
  PinOff,
  Paperclip,
  FileText,
  Clock3,
} from "lucide-react";
import { useServerStore } from "@/store/server.store";
import { useChannelStore } from "@/store/channel.store";
import { useWebSocketStore } from "@/store/websocket.store";
import { useAuthStore } from "@/store/auth.store";
import { useMemberStore } from "@/store/member.store";
import { useMessageStore } from "@/store/message.store";
import { messagesApi } from "@/lib/api";
import type { Message } from "@/lib/api/schemas/messages.schema";
import {
  formatScheduledAt,
  getNextWeeklyOccurrenceLocal,
  toBackendUtcSchedule,
} from "@/lib/schedule";
import type { WsMessage } from "@/lib/ws/types";
import { useLanguage } from "@/components/language-provider";
import { useAppearanceStore } from "@/store/appearance.store";

const FONT_SIZE_MAP = { sm: "14px", base: "16px", lg: "18px", xl: "20px" } as const;

type ScheduledPreview = {
  local_id: string;
  channel_id: string;
  content: string;
  scheduled_for: string;
};

export function ChatPanel() {
  const activeServerId = useServerStore((s) => s.activeServerId);
  const setActiveServer = useServerStore((s) => s.setActiveServer);

  const channels = useChannelStore((s) => s.channels);
  const activeChannelId = useChannelStore((s) => s.activeChannelId);
  const setActiveChannel = useChannelStore((s) => s.setActiveChannel);

  const user = useAuthStore((s) => s.user);
  const token = useAuthStore((s) => s.token);

  const members = useMemberStore((s) => s.members);
  const fontSize = useAppearanceStore((s) => s.fontSize);
  const chatFontSize = FONT_SIZE_MAP[fontSize] || "16px";
  const { language } = useLanguage();
  const isEnglish = language === "en";

  // WebSocket store
  const isConnected = useWebSocketStore((s) => s.isConnected);
  const connect = useWebSocketStore((s) => s.connect);
  const sendMessage = useWebSocketStore((s) => s.sendMessage);
  const sendScheduledMessage = useWebSocketStore((s) => s.sendScheduledMessage);
  const editMessage = useWebSocketStore((s) => s.editMessage);
  const deleteMessage = useWebSocketStore((s) => s.deleteMessage);
  const joinChannel = useWebSocketStore((s) => s.joinChannel);
  const socket = useWebSocketStore((s) => s.socket);
  const startTyping = useWebSocketStore((s) => s.startTyping);
  const stopTyping = useWebSocketStore((s) => s.stopTyping);
  const typingUsers = useWebSocketStore((s) => s.typingUsers);

  // Message store (source of truth)
  const messagesByChannel = useMessageStore((s) => s.messages);
  const setMsgs = useMessageStore((s) => s.setMessages);
  const prependMessages = useMessageStore((s) => s.prependMessages);
  const setCursor = useMessageStore((s) => s.setCursor);
  const clearChannel = useMessageStore((s) => s.clearChannel);

  const activeChannelName = useMemo(() => {
    return channels.find((c) => c.id === activeChannelId)?.name ?? null;
  }, [channels, activeChannelId]);

  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [value, setValue] = useState("");
  const [attachmentFile, setAttachmentFile] = useState<File | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  // Search UI state
  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<
    {
      id: string;
      server_id: string;
      channel_id: string;
      author_id: string;
      username: string;
      content: string;
      created_at: string;
    }[]
  >([]);
  const [searchLoading, setSearchLoading] = useState(false);
  const [openGifLightbox, setOpenGifLightbox] = useState<string | null>(null);
  const [replyTo, setReplyTo] = useState<string | null>(null);
  const [replyToUsername, setReplyToUsername] = useState<string | null>(null);
  const [editingMessageId, setEditingMessageId] = useState<string | null>(null);
  const [isEditing, setIsEditing] = useState(false);

  // Pinned messages panel state
  const [pinnedOpen, setPinnedOpen] = useState(false);
  const [pinnedMessages, setPinnedMessages] = useState<Message[]>([]);
  const [pinnedLoading, setPinnedLoading] = useState(false);

  const loadPinnedMessages = async () => {
    if (!activeServerId || !activeChannelId) return;
    setPinnedLoading(true);
    try {
      const data = await messagesApi.listPinned(activeServerId, activeChannelId);
      setPinnedMessages(data);
    } catch (err) {
      console.error("Failed to load pinned messages:", err);
    } finally {
      setPinnedLoading(false);
    }
  };

  const handlePin = async (messageId: string) => {
    if (!activeServerId || !activeChannelId) return;
    try {
      await messagesApi.pin(activeServerId, activeChannelId, messageId);
    } catch (err) {
      console.error("Failed to pin message:", err);
    }
  };

  const handleUnpin = async (messageId: string) => {
    if (!activeServerId || !activeChannelId) return;
    try {
      await messagesApi.unpin(activeServerId, activeChannelId, messageId);
    } catch (err) {
      console.error("Failed to unpin message:", err);
    }
  };

  // GIF picker state
  const [openGifPicker, setOpenGifPicker] = useState<"input" | null>(null);
  const [gifQuery, setGifQuery] = useState("");
  const [gifResults, setGifResults] = useState<
    { id: string; url: string; preview?: string; provider?: string }[]
  >([]);
  const [gifLoading, setGifLoading] = useState(false);

  // Emoji reaction picker state
  const [emojiPickerMsgId, setEmojiPickerMsgId] = useState<string | null>(null);
  const [showInputEmojis, setShowInputEmojis] = useState(false);
  const [scheduleOpen, setScheduleOpen] = useState(false);
  const [scheduleDay, setScheduleDay] = useState("1");
  const [scheduleHour, setScheduleHour] = useState("9");
  const [scheduleMinute, setScheduleMinute] = useState("0");
  const [scheduledByChannel, setScheduledByChannel] = useState<
    Record<string, ScheduledPreview[]>
  >({});
  const QUICK_EMOJIS = ["👍", "❤️", "😂", "😮", "😢", "🔥", "🎉", "👀"];

  const bottomRef = useRef<HTMLDivElement | null>(null);
  const scrollContainerRef = useRef<HTMLDivElement | null>(null);
  // distance from bottom saved before prepend, restored via useLayoutEffect
  const scrollAnchorRef = useRef<number | null>(null);
  const prevChannelRef = useRef<string | null>(null);

  const [hasMore, setHasMore] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);

  // Jump to a message by id (switch server/channel if needed)
  const jumpToMessage = async (
    messageId: string,
    serverId: string,
    channelId: string,
  ) => {
    try {
      // If server or channel differs, set them so the UI switches
      if (serverId && serverId !== activeServerId) {
        setActiveServer(serverId);
      }
      if (channelId && channelId !== activeChannelId) {
        setActiveChannel(channelId);
      }

      // Wait for the DOM element to appear (messages loaded via WS)
      const selector = `#msg-${messageId}`;
      const start = Date.now();
      const timeout = 3000; // ms
      while (Date.now() - start < timeout) {
        const el = document.querySelector(selector) as HTMLElement | null;
        if (el) {
          el.scrollIntoView({ behavior: "smooth", block: "center" });
          // briefly highlight
          const original = el.style.boxShadow;
          el.style.boxShadow = "0 0 0 3px rgba(99,102,241,0.25)";
          setTimeout(() => {
            el.style.boxShadow = original;
          }, 1600);
          return;
        }
        // wait a bit
        await new Promise((r) => setTimeout(r, 150));
      }
      // If not found, just close search
    } catch (err) {
      console.error("jumpToMessage failed", err);
    }
  };

  const canLoad = !!activeServerId && !!activeChannelId;

  // Get messages for current channel
  const messages = useMemo(() => {
    if (!activeChannelId) return [];
    return messagesByChannel[activeChannelId] ?? [];
  }, [activeChannelId, messagesByChannel]);

  const dayOptions = useMemo(
    () => [
      { value: "1", label: isEnglish ? "Monday" : "Lundi" },
      { value: "2", label: isEnglish ? "Tuesday" : "Mardi" },
      { value: "3", label: isEnglish ? "Wednesday" : "Mercredi" },
      { value: "4", label: isEnglish ? "Thursday" : "Jeudi" },
      { value: "5", label: isEnglish ? "Friday" : "Vendredi" },
      { value: "6", label: isEnglish ? "Saturday" : "Samedi" },
      { value: "7", label: isEnglish ? "Sunday" : "Dimanche" },
    ],
    [isEnglish],
  );

  const scheduledMessages = useMemo(() => {
    if (!activeChannelId) return [];
    return scheduledByChannel[activeChannelId] || [];
  }, [activeChannelId, scheduledByChannel]);

  const schedulePreviewLabel = useMemo(() => {
    const now = new Date();
    const day = Number(scheduleDay);
    const hour = Number(scheduleHour);
    const minute = Number(scheduleMinute);
    const next = getNextWeeklyOccurrenceLocal(now, day, hour, minute);
    if (!next) {
      return isEnglish ? "Invalid date/time" : "Date/heure invalide";
    }
    return formatScheduledAt(next, isEnglish ? "en" : "fr");
  }, [isEnglish, scheduleDay, scheduleHour, scheduleMinute]);

  // Debug: log chat state when messages / channel change
  useEffect(() => {
    console.log("📚 ChatPanel state", {
      activeServerId,
      activeChannelId,
      isConnected,
      messageCount: messages.length,
    });
  }, [activeServerId, activeChannelId, isConnected, messages.length]);

  // Connect WebSocket on mount
  useEffect(() => {
    if (token && !isConnected) {
      connect(token);
    }
  }, [token, isConnected, connect]);

  // Re-join channel whenever the WebSocket (re)connects — backend requires
  // an explicit JoinChannel even after a page refresh.
  useEffect(() => {
    if (!isConnected || !activeChannelId) return;
    joinChannel(activeChannelId);
  }, [isConnected, activeChannelId, joinChannel]);

  // On channel change: purge old cache, fetch if empty, join WS
  useEffect(() => {
    if (!activeChannelId || !isConnected || !activeServerId) return;

    const prevChannel = prevChannelRef.current;
    if (prevChannel && prevChannel !== activeChannelId) {
      clearChannel(prevChannel);
    }
    prevChannelRef.current = activeChannelId;

    setHasMore(true);
    setLoadingMore(false);

    const cached = useMessageStore.getState().getMessages(activeChannelId);
    if (cached.length === 0) {
      void (async () => {
        try {
          const data = await messagesApi.list(activeServerId, activeChannelId);
          setMsgs(activeChannelId, data as WsMessage[]);
          if (data.length > 0) setCursor(activeChannelId, data[0].id);
          setHasMore(data.length >= 50);
        } catch (err) {
          console.warn("Failed to load initial messages:", err);
        } finally {
          joinChannel(activeChannelId);
        }
      })();
    } else {
      joinChannel(activeChannelId);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeChannelId, isConnected, activeServerId]);

  // Scroll to bottom only for new WS messages (not prepend)
  useEffect(() => {
    if (scrollAnchorRef.current !== null) return; // restoring after prepend
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // Restore scroll position after prepend (useLayoutEffect fires before paint)
  useLayoutEffect(() => {
    if (scrollAnchorRef.current === null) return;
    const el = scrollContainerRef.current;
    if (el) el.scrollTop = el.scrollHeight - scrollAnchorRef.current;
    scrollAnchorRef.current = null;
  }, [messages]);

  // Load older messages (infinite scroll up)
  const loadOlderMessages = useCallback(async () => {
    if (!activeServerId || !activeChannelId || !hasMore || loadingMore) return;
    const msgs = useMessageStore.getState().getMessages(activeChannelId);
    if (!msgs.length) return;

    const el = scrollContainerRef.current;
    if (el) scrollAnchorRef.current = el.scrollHeight - el.scrollTop;

    setLoadingMore(true);
    try {
      const data = await messagesApi.list(activeServerId, activeChannelId, {
        before: msgs[0].id,
      });
      if (data.length > 0) {
        prependMessages(activeChannelId, data as WsMessage[]);
        setCursor(activeChannelId, data[0].id);
      }
      if (data.length < 50) setHasMore(false);
    } catch (err) {
      console.warn("Failed to load older messages:", err);
    } finally {
      setLoadingMore(false);
    }
  }, [activeServerId, activeChannelId, hasMore, loadingMore, prependMessages, setCursor]);

  const handleScroll = useCallback(
    (e: React.UIEvent<HTMLDivElement>) => {
      if (e.currentTarget.scrollTop < 150 && hasMore && !loadingMore) {
        void loadOlderMessages();
      }
    },
    [hasMore, loadingMore, loadOlderMessages],
  );

  // Get username from message, members or fallback to author_id
  const getUsernameById = useCallback(
    (authorId: string, msgUsername?: string): string => {
      // Check if username is provided in the message itself
      if (msgUsername) return msgUsername;
      // Check if it's the current user
      if (user && user.id === authorId) {
        return user.username;
      }
      // Check members
      const member = members.find((m) => m.user_id === authorId);
      if (member) {
        return member.username;
      }
      // Fallback: use first part of UUID
      return authorId.slice(0, 8);
    },
    [user, members],
  );

  const parseGifContent = useCallback((content: string) => {
    try {
      const parsed = JSON.parse(content);
      if (
        parsed &&
        parsed.type === "gif" &&
        parsed.gif &&
        typeof parsed.gif.url === "string"
      ) {
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

  const currentMemberRole = members.find((m) => m.user_id === user?.id)?.role;
  const canModerate =
    currentMemberRole === "Owner" ||
    currentMemberRole === "Admin" ||
    currentMemberRole === "Moderator";

  const onSend = async () => {
    console.log("🔵 [onSend] Called:", { activeChannelId, canLoad, activeServerId });
    if (!activeChannelId || !canLoad) {
      console.error("❌ [onSend] Blocked - activeChannelId:", activeChannelId, "canLoad:", canLoad);
      return;
    }

    const content = value.trim();
    if (!content && !attachmentFile) {
      console.log("⚠️ [onSend] Empty content and no attachment");
      return;
    }

    if (!isConnected) {
      setError(isEnglish ? "Not connected to the server" : "Non connecté au serveur");
      return;
    }

    setSending(true);
    setError(null);

    try {
      let attachmentUrl: string | undefined;
      if (attachmentFile && activeServerId) {
        const result = await messagesApi.uploadAttachment(activeServerId, activeChannelId, attachmentFile);
        attachmentUrl = result.url;
      }

      if (isEditing && editingMessageId) {
        editMessage(activeChannelId, editingMessageId, content);
        setEditingMessageId(null);
        setIsEditing(false);
      } else if (scheduleOpen) {
        const day = Number(scheduleDay);
        const hour = Number(scheduleHour);
        const minute = Number(scheduleMinute);

        if (!(day >= 1 && day <= 7 && hour >= 0 && hour <= 23 && minute >= 0 && minute <= 59)) {
          setError(
            isEnglish
              ? "Invalid schedule values (day 1-7, hour 0-23, minute 0-59)."
              : "Valeurs de planification invalides (jour 1-7, heure 0-23, minute 0-59).",
          );
          return;
        }

        const nextAt = getNextWeeklyOccurrenceLocal(new Date(), day, hour, minute);
        if (!nextAt) {
          setError(
            isEnglish
              ? "Unable to compute next schedule slot."
              : "Impossible de calculer le prochain envoi programme.",
          );
          return;
        }

        const backendUtcSchedule = toBackendUtcSchedule(nextAt);

        sendScheduledMessage(
          activeChannelId,
          content,
          backendUtcSchedule.dayOfWeek,
          backendUtcSchedule.hour,
          backendUtcSchedule.minute,
          replyTo || undefined,
        );
        setScheduledByChannel((prev) => {
          const current = prev[activeChannelId] || [];
          return {
            ...prev,
            [activeChannelId]: [
              ...current,
              {
                local_id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
                channel_id: activeChannelId,
                content,
                scheduled_for: nextAt.toISOString(),
              },
            ],
          };
        });
        setReplyTo(null);
        setReplyToUsername(null);
        setScheduleOpen(false);
      } else {
        console.log("🔵 [onSend] About to call sendMessage:", { activeChannelId, content: content.substring(0, 50), replyTo, attachmentUrl });
        sendMessage(activeChannelId, content, replyTo || undefined, attachmentUrl);
        setReplyTo(null);
        setReplyToUsername(null);
        setAttachmentFile(null);
      }
      setValue("");
      if (activeChannelId) stopTyping(activeChannelId);
    } catch (e) {
      setError(e instanceof Error ? e.message : isEnglish ? "Error sending message" : "Erreur envoi message");
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

  // Typing indicator: track when user is typing
  const typingTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const newValue = e.target.value;
      setValue(newValue);

      if (!activeChannelId || !isConnected) return;

      if (newValue.trim()) {
        startTyping(activeChannelId);
        // Clear previous timeout
        if (typingTimeoutRef.current) clearTimeout(typingTimeoutRef.current);
        // Stop typing after 2 seconds of inactivity
        typingTimeoutRef.current = setTimeout(() => {
          if (activeChannelId) stopTyping(activeChannelId);
        }, 2000);
      } else {
        stopTyping(activeChannelId);
        if (typingTimeoutRef.current) clearTimeout(typingTimeoutRef.current);
      }
    },
    [activeChannelId, isConnected, startTyping, stopTyping],
  );

  // Get typing users for current channel (exclude self)
  const currentTypingUsers = useMemo(() => {
    if (!activeChannelId) return [];
    const users = typingUsers[activeChannelId] || [];
    return users.filter((uid) => uid !== user?.id);
  }, [activeChannelId, typingUsers, user?.id]);

  const typingDisplay = useMemo(() => {
    if (currentTypingUsers.length === 0) return null;
    const names = currentTypingUsers.map((uid) => getUsernameById(uid));
    if (names.length === 1)
      return isEnglish
        ? `${names[0]} is typing...`
        : `${names[0]} est en train d'écrire...`;
    if (names.length === 2)
      return isEnglish
        ? `${names[0]} and ${names[1]} are typing...`
        : `${names[0]} et ${names[1]} écrivent...`;
    return isEnglish
      ? `${names.length} people are typing...`
      : `${names.length} personnes écrivent...`;
  }, [currentTypingUsers, getUsernameById, isEnglish]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      onSend();
    }
  };

  // Autocomplete/debounced GIF search when picker is open
  useEffect(() => {
    if (openGifPicker !== "input") return;

    const query =
      gifQuery && gifQuery.trim() !== "" ? gifQuery.trim() : "trending";
    const controller = new AbortController();
    setGifLoading(true);

    const timer = setTimeout(async () => {
      try {
        const res = await fetch(
          `/api/gifs/search?q=${encodeURIComponent(query)}&limit=24`,
          { signal: controller.signal },
        );
        if (!res.ok) {
          setGifResults([]);
          setGifLoading(false);
          return;
        }
        const json = await res.json();
        const results: {
          id: string;
          url: string;
          preview?: string;
          provider?: string;
        }[] = [];
        if (json.results) {
          for (const it of json.results) {
            const id = it.id || "";
            const url = it.url || "";
            const preview = it.preview || undefined;
            const provider = it.provider || undefined;
            if (url)
              results.push({ id: id.toString(), url, preview, provider });
          }
        } else if (json.data) {
          for (const it of json.data) {
            const id = it.id || "";
            const url =
              it.images?.original?.url || it.images?.fixed_width?.url || "";
            const preview =
              it.images?.preview_gif?.url ||
              it.images?.fixed_width_small_still?.url;
            if (url)
              results.push({
                id: id.toString(),
                url,
                preview,
                provider: "giphy",
              });
          }
        }
        setGifResults(results);
      } catch (err: unknown) {
        // Ignore AbortError (fetch aborted on quick typing / picker close)
        if (!(err instanceof DOMException) || err.name !== "AbortError") {
          console.error("GIF search failed", err);
        }
        setGifResults([]);
      } finally {
        setGifLoading(false);
      }
    }, 300);

    return () => {
      clearTimeout(timer);
      controller.abort();
    };
  }, [gifQuery, openGifPicker]);

  return (
    <div className="h-full border border-[var(--border)] min-w-0 flex flex-col bg-[var(--card)] shadow-lg overflow-hidden">
      {/* --- HEADER --- */}
      <div className="h-12 px-4 flex items-center border-b shadow-sm shrink-0">
        <span className="text-[var(--muted-foreground)] mr-2 text-2xl">#</span>
        <h2 className="font-bold text-md text-[var(--foreground)]">
          {activeChannelName ?? (isEnglish ? "no-channel" : "aucun-channel")}
        </h2>
        <div className="ml-auto flex items-center gap-1 relative">
          <button
            title={isEnglish ? "Pinned messages" : "Messages épinglés"}
            onClick={() => {
              setPinnedOpen((s) => !s);
              if (!pinnedOpen) loadPinnedMessages();
            }}
            className={`h-8 w-8 rounded-md flex items-center justify-center transition ${pinnedOpen ? "text-indigo-600 bg-indigo-50 dark:bg-indigo-900/30" : "text-zinc-500 hover:bg-zinc-100 dark:hover:bg-zinc-800"}`}
          >
            <Pin className="h-4 w-4" />
          </button>

          {pinnedOpen && (
            <div className="absolute right-0 top-10 z-40 bg-white dark:bg-zinc-900 border rounded-md shadow-lg p-3 w-[min(90vw,28rem)]">
              <div className="flex items-center justify-between mb-2">
                <span className="font-semibold text-sm">
                  {isEnglish ? "Pinned messages" : "Messages épinglés"}
                </span>
                <button onClick={() => setPinnedOpen(false)} className="text-zinc-400 hover:text-zinc-600">
                  <X className="h-4 w-4" />
                </button>
              </div>
              {pinnedLoading ? (
                <div className="flex items-center gap-2 text-sm text-zinc-500 py-2">
                  <Loader2 className="h-4 w-4 animate-spin" /> Chargement...
                </div>
              ) : pinnedMessages.length === 0 ? (
                <div className="text-sm text-zinc-500">
                  {isEnglish ? "No pinned messages." : "Aucun message épinglé."}
                </div>
              ) : (
                <ul className="flex flex-col gap-2 max-h-72 overflow-auto">
                  {pinnedMessages.map((m) => (
                    <li
                      key={m.id}
                      className="p-2 border rounded hover:bg-zinc-50 dark:hover:bg-zinc-800 cursor-pointer"
                      onClick={async () => {
                        setPinnedOpen(false);
                        await jumpToMessage(m.id, activeServerId ?? "", m.channel_id);
                      }}
                    >
                      <div className="text-xs text-zinc-500 mb-0.5">
                        {new Date(m.created_at).toLocaleString()}
                      </div>
                      <div className="text-sm text-zinc-700 dark:text-zinc-200 truncate">
                        {m.content}
                      </div>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}

          <button
            title="Rechercher"
            onClick={() => setSearchOpen((s) => !s)}
            className="h-8 w-8 rounded-md flex items-center justify-center text-zinc-500 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition"
          >
            <Search className="h-4 w-4" />
          </button>

          {searchOpen && (
            <div className="absolute right-0 top-10 z-40 bg-white dark:bg-zinc-900 border rounded-md shadow-lg p-3 w-[min(90vw,28rem)]">
              <div className="flex gap-2">
                <input
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="flex-1 px-2 py-1 border rounded bg-zinc-50 dark:bg-zinc-800"
                  placeholder="Rechercher dans ce channel"
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      void (async () => {
                        if (!activeServerId || !activeChannelId) return;
                        setSearchLoading(true);
                        try {
                          const url = `/api/servers/${activeServerId}/channels/${activeChannelId}/messages/search?q=${encodeURIComponent(
                            searchQuery || "",
                          )}&per_page=12`;
                          // Attach Authorization header from localStorage (client-side)
                          const headers: Record<string, string> = {
                            "Content-Type": "application/json",
                          };
                          try {
                            const localToken =
                              typeof window !== "undefined"
                                ? localStorage.getItem("token")
                                : null;
                            if (localToken)
                              headers["Authorization"] = `Bearer ${localToken}`;
                          } catch {
                            /* ignore */
                          }
                          const res = await fetch(url, { headers });
                          if (!res.ok) {
                            setSearchResults([]);
                            return;
                          }
                          const json = await res.json();
                          setSearchResults(json || []);
                        } catch (err) {
                          console.error("Search failed", err);
                          setSearchResults([]);
                        } finally {
                          setSearchLoading(false);
                        }
                      })();
                    }
                  }}
                />
                <button
                  onClick={async () => {
                    if (!activeServerId || !activeChannelId) return;
                    setSearchLoading(true);
                    try {
                      const url = `/api/servers/${activeServerId}/channels/${activeChannelId}/messages/search?q=${encodeURIComponent(
                        searchQuery || "",
                      )}&per_page=12`;
                      const headers: Record<string, string> = {
                        "Content-Type": "application/json",
                      };
                      try {
                        const localToken =
                          typeof window !== "undefined"
                            ? localStorage.getItem("token")
                            : null;
                        if (localToken)
                          headers["Authorization"] = `Bearer ${localToken}`;
                      } catch {
                        /* ignore */
                      }
                      const res = await fetch(url, { headers });
                      if (!res.ok) {
                        setSearchResults([]);
                        return;
                      }
                      const json = await res.json();
                      setSearchResults(json || []);
                    } catch (err) {
                      console.error("Search failed", err);
                      setSearchResults([]);
                    } finally {
                      setSearchLoading(false);
                    }
                  }}
                  className="px-2 py-1 bg-indigo-600 text-white rounded"
                >
                  {searchLoading ? "…" : "Go"}
                </button>
              </div>

              <div className="mt-2 max-h-64 overflow-auto">
                {searchResults.length === 0 ? (
                  <div className="text-sm text-zinc-500">Aucun résultat</div>
                ) : (
                  <ul className="flex flex-col gap-2">
                    {searchResults.map((r) => (
                      <li
                        key={r.id}
                        className="p-2 border rounded hover:bg-zinc-50 dark:hover:bg-zinc-800 cursor-pointer"
                        onClick={async () => {
                          // Jump to the message in the conversation
                          setSearchOpen(false);
                          setSearchResults([]);
                          await jumpToMessage(r.id, r.server_id, r.channel_id);
                        }}
                      >
                        <div className="text-xs text-zinc-500">
                          {new Date(r.created_at).toLocaleString()} —{" "}
                          {r.username}
                        </div>
                        <div className="text-sm text-zinc-700 dark:text-zinc-200 truncate">
                          {r.content}
                        </div>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            </div>
          )}
        </div>
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
        {!canLoad ? (
          <div className="px-4 text-sm text-muted-foreground">
            {isEnglish
              ? "Select a server and a channel."
              : "Sélectionne un serveur et un channel."}
          </div>
        ) : messages.length === 0 && !isConnected ? (
          <div className="px-4 text-sm text-muted-foreground flex items-center gap-2">
            <Loader2 className="h-4 w-4 animate-spin" />
            {isEnglish ? "Connecting to server..." : "Connexion au serveur..."}
          </div>
        ) : messages.length === 0 ? (
          <div className="px-4 text-sm text-muted-foreground">
            {isEnglish
              ? "No messages in this channel yet. Be the first to write!"
              : "Aucun message dans ce channel. Soyez le premier à écrire !"}
          </div>
        ) : (
          <div className="flex flex-col mt-auto">
            {messages.map((msg) => {
              const isAuthor = user?.id === msg.author_id;
              const canDelete = isAuthor || canModerate;
              const messageUsername = getUsernameById(
                msg.author_id,
                msg.username,
              );
              return (
                <div
                  key={msg.id}
                  id={`msg-${msg.id}`}
                  className="group relative flex items-start px-4 py-2 hover:bg-black/5 dark:hover:bg-white/5 transition w-full"
                >
                  {/* Action buttons — top-right, visible on hover */}
                  <div className="absolute -top-3 right-4 hidden group-hover:flex items-center gap-0.5 bg-[var(--card)] border border-[var(--border)] rounded-md shadow-sm px-1 py-0.5 z-10">
                    <button
                      className="h-7 w-7 flex items-center justify-center text-[var(--muted-foreground)] hover:text-indigo-600 hover:bg-[var(--surface)] rounded transition"
                      onClick={() => {
                        setReplyTo(msg.id);
                        setReplyToUsername(messageUsername);
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
                          setValue(msg.content);
                        }}
                        title="Modifier"
                      >
                        <Edit3 className="h-4 w-4" />
                      </button>
                    )}
                    {canDelete && (
                      <button
                        className="h-7 w-7 flex items-center justify-center text-zinc-500 hover:text-red-600 hover:bg-zinc-100 dark:hover:bg-zinc-700 rounded transition"
                        onClick={() =>
                          deleteMessage(activeChannelId || "", msg.id)
                        }
                        title="Supprimer"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    )}
                    {canModerate && (
                      <button
                        className="h-7 w-7 flex items-center justify-center text-zinc-500 hover:text-indigo-600 hover:bg-zinc-100 dark:hover:bg-zinc-700 rounded transition"
                        onClick={() =>
                          msg.pinned_at ? handleUnpin(msg.id) : handlePin(msg.id)
                        }
                        title={msg.pinned_at ? (isEnglish ? "Unpin" : "Désépingler") : (isEnglish ? "Pin" : "Épingler")}
                      >
                        {msg.pinned_at ? <PinOff className="h-4 w-4" /> : <Pin className="h-4 w-4" />}
                      </button>
                    )}
                    <button
                      className="h-7 w-7 flex items-center justify-center text-zinc-500 hover:text-amber-500 hover:bg-zinc-100 dark:hover:bg-zinc-700 rounded transition"
                      onClick={() =>
                        setEmojiPickerMsgId(
                          emojiPickerMsgId === msg.id ? null : msg.id,
                        )
                      }
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
                            if (
                              socket &&
                              socket.readyState === WebSocket.OPEN
                            ) {
                              socket.send(
                                JSON.stringify({
                                  type: "ReactionAdd",
                                  payload: { message_id: msg.id, emoji },
                                }),
                              );
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
                      {msg.pinned_at && (
                        <span className="inline-flex items-center gap-0.5 text-xs text-amber-600 dark:text-amber-400 font-medium">
                          <Pin className="h-3 w-3" />
                          {isEnglish ? "pinned" : "épinglé"}
                        </span>
                      )}
                    </div>

                    {msg.reply_to &&
                      (() => {
                        const original = messages.find(
                          (m) => m.id === msg.reply_to,
                        );
                        return (
                          <div className="text-xs text-zinc-500 italic mb-1 bg-zinc-100 dark:bg-zinc-800 p-2 rounded-md max-h-16 overflow-hidden">
                            Réponse à{" "}
                            {original
                              ? getUsernameById(
                                  original.author_id,
                                  original.username,
                                )
                              : msg.reply_to.slice(0, 8)}
                            :
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
                            {/* GIF preview, constrained size; click to open full */}
                            {/* eslint-disable-next-line @next/next/no-img-element */}
                            <img
                              src={gifMessage.gif.url}
                              alt={gifMessage.caption || "GIF"}
                              className="max-h-90 max-w-[70%] rounded-md object-contain cursor-pointer"
                              onClick={() =>
                                setOpenGifLightbox(gifMessage.gif.url)
                              }
                            />
                            {gifMessage.caption && (
                              <p className="mt-1 text-sm text-[var(--muted-foreground)]">
                                {gifMessage.caption}
                              </p>
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
                          msg.reactions.reduce<
                            Record<
                              string,
                              {
                                count: number;
                                users: string[];
                                userIds: string[];
                              }
                            >
                          >((acc, r) => {
                            if (!acc[r.emoji])
                              acc[r.emoji] = {
                                count: 0,
                                users: [],
                                userIds: [],
                              };
                            acc[r.emoji].count++;
                            acc[r.emoji].users.push(
                              r.username || r.user_id.slice(0, 6),
                            );
                            acc[r.emoji].userIds.push(r.user_id);
                            return acc;
                          }, {}),
                        ).map(([emoji, data]) => {
                          const hasReacted = data.userIds.includes(
                            user?.id || "",
                          );
                          return (
                            <button
                              key={emoji}
                              className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs border transition ${
                                hasReacted
                                  ? "bg-indigo-100 border-indigo-300 text-indigo-700 dark:bg-indigo-900/40 dark:border-indigo-600 dark:text-indigo-300"
                                  : "bg-[var(--surface)] border-[var(--border)] text-[var(--muted-foreground)] hover:bg-[var(--muted)]"
                              }`}
                              title={data.users.join(", ")}
                              onClick={() => {
                                if (
                                  socket &&
                                  socket.readyState === WebSocket.OPEN
                                ) {
                                  socket.send(
                                    JSON.stringify({
                                      type: "ReactionAdd",
                                      payload: { message_id: msg.id, emoji },
                                    }),
                                  );
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

      {/* --- INPUT --- */}
      <div className="p-4 mb-2 shrink-0">
        {((replyTo && !isEditing) || isEditing) && (
          <div className="mb-2 rounded-md border border-indigo-200 bg-indigo-50 p-2 text-xs text-indigo-700 dark:border-indigo-700 dark:bg-indigo-950/30 dark:text-indigo-200 flex items-center justify-between">
            <div>
              {isEditing
                ? "Modification du message en cours"
                : "Réponse en cours"}
              {replyTo && !isEditing && replyToUsername
                ? `: ${replyToUsername}`
                : ""}
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
                setError(isEnglish ? "File must be under 10 MB" : "Le fichier doit faire moins de 10 Mo");
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
              title={isEnglish ? "Attach a file" : "Joindre un fichier"}
            >
              <Paperclip className="text-white h-3.5 w-3.5" />
            </button>

            <Input
              value={value}
              onChange={handleInputChange}
              onKeyDown={handleKeyDown}
              disabled={!canLoad || sending}
              className="px-14 pr-32 py-6 bg-[var(--muted)] border-none border-0 focus-visible:ring-0 focus-visible:ring-offset-0 text-[var(--foreground)] placeholder:text-[var(--muted-foreground)]"
              placeholder={
                !canLoad
                  ? isEnglish
                    ? "Select a channel..."
                    : "Sélectionne un channel..."
                  : isEnglish
                    ? `Send a message in #${activeChannelName ?? ""}`
                    : `Envoyer un message dans #${activeChannelName ?? ""}`
              }
            />

            <div className="absolute right-4 top-1/2 -translate-y-1/2 flex items-center gap-x-3">
              <button
                type="button"
                onClick={() => setScheduleOpen((v) => !v)}
                title={isEnglish ? "Schedule weekly message" : "Programmer un message hebdomadaire"}
                className={`h-7 px-2 rounded-full border text-[11px] font-semibold transition flex items-center gap-1 ${
                  scheduleOpen
                    ? "border-amber-300 text-amber-700 bg-amber-50 dark:border-amber-700 dark:text-amber-300 dark:bg-amber-950/40"
                    : "border-[var(--border)] text-[var(--muted-foreground)] hover:text-[var(--foreground)]"
                }`}
              >
                <Clock3 className="h-3.5 w-3.5" />
                {isEnglish ? "Later" : "Plus tard"}
              </button>

              <span title="Fonction à venir">
                <Gift className="h-5 w-5 text-[var(--muted-foreground)] hover:text-[var(--muted-foreground)] transition cursor-not-allowed" />
              </span>

              <button
                type="button"
                onClick={() =>
                  setOpenGifPicker(openGifPicker === "input" ? null : "input")
                }
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
            disabled={!canLoad || sending || !isConnected || (!value.trim() && !attachmentFile)}
            className="h-12 px-4 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            title={
              scheduleOpen
                ? isEnglish
                  ? "Schedule the message"
                  : "Programmer le message"
                : isEnglish
                  ? "Send message"
                  : "Envoyer le message"
            }
          >
            {sending ? (
              <Loader2 className="h-5 w-5 animate-spin" />
            ) : (
              <Send className="h-5 w-5" />
            )}
          </Button>
        </div>

        {scheduleOpen && (
          <div className="mt-2 rounded-xl border border-amber-200/70 dark:border-amber-900/70 bg-amber-50/60 dark:bg-amber-950/20 p-3 flex flex-wrap items-end gap-3 text-xs">
            <div className="flex flex-col gap-1">
              <label>{isEnglish ? "Day" : "Jour"}</label>
              <select
                value={scheduleDay}
                onChange={(e) => setScheduleDay(e.target.value)}
                className="w-44 rounded border border-[var(--border)] bg-[var(--card)] px-2 py-1"
              >
                {dayOptions.map((day) => (
                  <option key={day.value} value={day.value}>
                    {day.label}
                  </option>
                ))}
              </select>
            </div>
            <div className="flex flex-col gap-1">
              <label>{isEnglish ? "Hour" : "Heure"}</label>
              <select
                value={scheduleHour}
                onChange={(e) => setScheduleHour(e.target.value)}
                className="w-20 rounded border border-[var(--border)] bg-[var(--card)] px-2 py-1"
              >
                {Array.from({ length: 24 }, (_, hour) => (
                  <option key={hour} value={String(hour)}>
                    {hour.toString().padStart(2, "0")}
                  </option>
                ))}
              </select>
            </div>
            <div className="flex flex-col gap-1">
              <label>{isEnglish ? "Minute" : "Minute"}</label>
              <select
                value={scheduleMinute}
                onChange={(e) => setScheduleMinute(e.target.value)}
                className="w-20 rounded border border-[var(--border)] bg-[var(--card)] px-2 py-1"
              >
                {Array.from({ length: 60 }, (_, minute) => (
                  <option key={minute} value={String(minute)}>
                    {minute.toString().padStart(2, "0")}
                  </option>
                ))}
              </select>
            </div>
            <div className="text-[var(--muted-foreground)] flex-1 min-w-52">
              {isEnglish
                ? "The receiver will only see it when the scheduled time arrives."
                : "Le destinataire le verra seulement a l'heure programmee."}
              <div className="mt-1 text-[var(--foreground)] font-medium">
                {isEnglish ? "Preview (local time):" : "Apercu (heure locale) :"} {schedulePreviewLabel}
              </div>
            </div>
          </div>
        )}

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
                      `/api/gifs/search?q=${encodeURIComponent(
                        gifQuery || "trending",
                      )}&limit=24`,
                    );
                    if (!res.ok) {
                      setGifResults([]);
                      setGifLoading(false);
                      return;
                    }
                    const json = await res.json();
                    const results: {
                      id: string;
                      url: string;
                      preview?: string;
                      provider?: string;
                    }[] = [];
                    if (json.results) {
                      for (const it of json.results) {
                        const id = it.id || "";
                        const url = it.url || "";
                        const preview = it.preview || undefined;
                        const provider = it.provider || undefined;
                        if (url)
                          results.push({
                            id: id.toString(),
                            url,
                            preview,
                            provider,
                          });
                      }
                    } else if (json.data) {
                      for (const it of json.data) {
                        const id = it.id || "";
                        const url =
                          it.images?.original?.url ||
                          it.images?.fixed_width?.url ||
                          "";
                        const preview =
                          it.images?.preview_gif?.url ||
                          it.images?.fixed_width_small_still?.url;
                        if (url)
                          results.push({
                            id: id.toString(),
                            url,
                            preview,
                            provider: "giphy",
                          });
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
                {gifLoading ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  "Rechercher"
                )}
              </button>
              <button
                onClick={() => {
                  setOpenGifPicker(null);
                  setGifResults([]);
                  setGifQuery("");
                }}
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
                      if (
                        socket &&
                        socket.readyState === WebSocket.OPEN &&
                        activeChannelId
                      ) {
                        const ev = {
                          type: "MessageSendGif",
                          payload: {
                            channel_id: activeChannelId,
                            gif: {
                              id: g.id,
                              url: g.url,
                              preview: g.preview,
                              provider: g.provider,
                            },
                            caption: null,
                          },
                        };
                        socket.send(JSON.stringify(ev));
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

        {/* Typing indicator */}
        {typingDisplay && (
          <div className="mt-1 px-1 text-xs text-indigo-500 flex items-center gap-1 animate-pulse">
            <span className="flex gap-0.5">
              <span
                className="w-1.5 h-1.5 bg-indigo-400 rounded-full animate-bounce"
                style={{ animationDelay: "0ms" }}
              />
              <span
                className="w-1.5 h-1.5 bg-indigo-400 rounded-full animate-bounce"
                style={{ animationDelay: "150ms" }}
              />
              <span
                className="w-1.5 h-1.5 bg-indigo-400 rounded-full animate-bounce"
                style={{ animationDelay: "300ms" }}
              />
            </span>
            {typingDisplay}
          </div>
        )}

        {/* Connection status */}
        {!isConnected && canLoad && (
          <div className="mt-2 text-xs text-amber-600 flex items-center gap-1">
            <Loader2 className="h-3 w-3 animate-spin" />
            Reconnexion en cours...
          </div>
        )}
      </div>
    </div>
  );
}
