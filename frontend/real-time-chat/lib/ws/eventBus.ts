import { wsManager } from "./manager";
import { messageHandler } from "./handlers/messageHandler";
import { presenceHandler } from "./handlers/presenceHandler";
import { typingHandler } from "./handlers/typingHandler";
import { dmHandler } from "./handlers/dmHandler";
import { memberHandler } from "./handlers/memberHandler";
import type { ServerEvent } from "./types";

// ─── Dispatch table ───────────────────────────────────────────────────────────
// Maps each server event type to its domain handler.
// "Error" and "Pong" are handled directly in websocket.store, not here.

type AnyPayload = Parameters<typeof messageHandler.onMessageNew>[0] &
  Parameters<typeof presenceHandler.onUserOnline>[0];

const DISPATCH: Partial<Record<ServerEvent["type"], (payload: AnyPayload) => void>> = {
  MessageNew: (p) => messageHandler.onMessageNew(p as never),
  MessageEdited: (p) => messageHandler.onMessageEdited(p as never),
  MessageDeleted: (p) => messageHandler.onMessageDeleted(p as never),
  MessagePinned: (p) => messageHandler.onMessagePinned(p as never),
  MessageUnpinned: (p) => messageHandler.onMessageUnpinned(p as never),
  ReactionAdded: (p) => messageHandler.onReactionAdded(p as never),
  ReactionRemoved: (p) => messageHandler.onReactionRemoved(p as never),

  UserOnline: (p) => presenceHandler.onUserOnline(p as never),
  UserOffline: (p) => presenceHandler.onUserOffline(p as never),
  PresenceUpdated: (p) => presenceHandler.onPresenceUpdated(p as never),
  PresenceSync: (p) => presenceHandler.onPresenceSync(p as never),

  TypingStart: (p) => typingHandler.onTypingStart(p as never),
  TypingStop: (p) => typingHandler.onTypingStop(p as never),

  DmNew: (p) => dmHandler.onDmNew(p as never),
  DmEdited: (p) => dmHandler.onDmEdited(p as never),
  DmDeleted: (p) => dmHandler.onDmDeleted(p as never),

  UserJoined: (p) => memberHandler.onUserJoined(p as never),
  UserLeft: (p) => memberHandler.onUserLeft(p as never),
};

// ─── Init (idempotent) ────────────────────────────────────────────────────────

let _initialized = false;

/**
 * Wire the EventBus to the WebSocketManager.
 * Safe to call multiple times — only registers once.
 */
export function initEventBus(): void {
  if (_initialized) return;
  _initialized = true;

  wsManager.onMessage((event) => {
    // Pong and Error are handled at the store/connection level
    if (event.type === "Pong" || event.type === "Error") return;

    const handler = DISPATCH[event.type];
    if (!handler) return;

    if ("payload" in event) {
      try {
        handler(event.payload as AnyPayload);
      } catch (err) {
        console.error(`[EventBus] Handler error for "${event.type}":`, err);
      }
    }
  });
}
