use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::repositories::ChannelRepository;
use crate::repositories::MembershipRepository;
use crate::repositories::UserRepository;
use crate::services::message_service::MessageService;
use crate::services::presence_service::PresenceService;
use crate::services::scheduled_message_service::next_weekly_occurrence;
use crate::services::typing_service::TypingService;
use crate::ws::hub::{ConnId, Hub};
use mongodb::bson::oid::ObjectId;
use crate::ws::protocol::{
    validate_channel_id, validate_content, validate_schedule, ClientEvent, ServerEvent, MAX_CONTENT_LEN,
    MAX_FRAME_BYTES, TYPING_THROTTLE_MS,
};
use sqlx::PgPool;
use uuid::Uuid;

const HEARTBEAT_TIMEOUT_SECS: i64 = 120;

// ─────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────

/// Build a deterministic DM conversation id from two user ids.
/// Format: `dm:{min}:{max}` where min/max are lexicographic.
fn dm_conversation_id(user_a: &str, user_b: &str) -> String {
    if user_a < user_b {
        format!("dm:{}:{}", user_a, user_b)
    } else {
        format!("dm:{}:{}", user_b, user_a)
    }
}

/// Serialise a `ServerEvent::Error` and send it on the connection channel.
/// Errors during send are silently ignored (the client may have disconnected).
fn send_error(hub: &Hub, conn_id: &ConnId, code: &str, message: &str) {
    let event = ServerEvent::Error {
        code: code.to_owned(),
        message: message.to_owned(),
    };
    if let Some(tx) = hub.sockets.get(conn_id) {
        if let Ok(json) = serde_json::to_string(&event) {
            let _ = tx.send(Message::Text(json));
        }
    }
}

/// Resolve `server_id` for a given `channel_id` by querying Postgres.
/// Results are cached in `cache` to avoid repeated queries for the same channel.
/// Returns `None` (and sends an error to the client) if the channel doesn't exist.
async fn resolve_server_id(
    channel_id: &str,
    pool: &PgPool,
    cache: &mut HashMap<String, String>,
    hub: &Hub,
    conn_id: &ConnId,
) -> Option<String> {
    // Check cache first
    if let Some(sid) = cache.get(channel_id) {
        return Some(sid.clone());
    }

    // Parse channel_id as UUID
    let channel_uuid = match Uuid::parse_str(channel_id) {
        Ok(u) => u,
        Err(_) => {
            send_error(
                hub,
                conn_id,
                "INVALID_CHANNEL_ID",
                "channel_id is not a valid UUID",
            );
            return None;
        },
    };

    // Look up channel in Postgres
    match ChannelRepository::find_by_id(pool, channel_uuid).await {
        Ok(Some(channel)) => {
            let sid = channel.server_id.to_string();
            cache.insert(channel_id.to_owned(), sid.clone());
            Some(sid)
        },
        Ok(None) => {
            send_error(hub, conn_id, "CHANNEL_NOT_FOUND", "channel does not exist");
            None
        },
        Err(e) => {
            tracing::error!(channel = %channel_id, "Failed to look up channel: {e}");
            send_error(hub, conn_id, "INTERNAL_ERROR", "failed to look up channel");
            None
        },
    }
}

/// Resolve username for a given user_id by querying Postgres.
/// Results are cached in `cache` to avoid repeated queries.
async fn resolve_username(
    user_id: &str,
    pool: &PgPool,
    cache: &mut HashMap<String, String>,
) -> String {
    if let Some(name) = cache.get(user_id) {
        return name.clone();
    }
    if let Ok(uuid) = Uuid::parse_str(user_id) {
        if let Ok(Some(user)) = UserRepository::find_by_id(pool, uuid).await {
            cache.insert(user_id.to_owned(), user.username.clone());
            return user.username;
        }
    }
    user_id.chars().take(8).collect()
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_connection(
    socket: WebSocket,
    hub: Arc<Hub>,
    message_service: Arc<MessageService>,
    presence: Arc<PresenceService>,
    typing_service: Arc<TypingService>,
    pg_pool: PgPool,
    user_id: String,
    conn_id: ConnId,
) {
    // Canal interne pour envoyer des messages au client
    let (tx, rx) = mpsc::unbounded_channel();
    // Register connection in hub so broadcast messages reach this socket
    hub.register_connection(&user_id, conn_id, tx);

    // Register connection in presence service (stores conn id + last activity)
    // and broadcast presence so other connected clients become aware.
    // We do this here (after registering the connection) to avoid a race where
    // broadcasts happen before the new socket is registered and thus the
    // event is not received by clients already connected.
    let conn_id_str = conn_id.to_string();

    // Read the persisted status before overwriting it so we can preserve
    // IDLE / DND across a page refresh (only force ONLINE for OFFLINE users).
    let effective_status_str = if let Ok(uid) = Uuid::parse_str(&user_id) {
        match UserRepository::find_by_id(&pg_pool, uid).await {
            Ok(Some(user)) => match user.status {
                crate::models::user::UserStatus::Idle => "idle",
                crate::models::user::UserStatus::Dnd  => "dnd",
                _                                     => "online",
            },
            _ => "online",
        }
    } else {
        "online"
    };

    presence.add_connection(&user_id, &conn_id_str);

    // add_connection always sets Online in memory; restore IDLE/DND if needed.
    match effective_status_str {
        "idle" => { presence.set_status(&user_id, crate::services::presence_service::PresenceStatus::Idle); }
        "dnd"  => { presence.set_status(&user_id, crate::services::presence_service::PresenceStatus::Dnd); }
        _      => {}
    }

    // Only write ONLINE to PG when the user was previously OFFLINE.
    // IDLE / DND are left untouched so they survive the reconnect.
    if effective_status_str == "online" {
        if let Ok(uid) = Uuid::parse_str(&user_id) {
            let pool_pg = pg_pool.clone();
            tokio::spawn(async move {
                let _ = UserRepository::update_status(
                    &pool_pg,
                    uid,
                    &crate::models::user::UserStatus::Online,
                )
                .await;
            });
        }
    }

    // Always broadcast the effective status so late-joining clients stay in sync.
    hub.broadcast_all(crate::ws::protocol::ServerEvent::UserOnline {
        user_id: user_id.clone(),
        status: effective_status_str.to_string(),
    })
    .await;
    hub.broadcast_all(crate::ws::protocol::ServerEvent::PresenceUpdated {
        user_id: user_id.clone(),
        status: effective_status_str.to_string(),
        last_activity: chrono::Utc::now().to_rfc3339(),
    })
    .await;

    // Send a presence snapshot to the newly connected client so existing users
    // appear online immediately without waiting for new events.
    if let Some(tx) = hub.sockets.get(&conn_id) {
        for (uid, status, last_activity) in presence.snapshot() {
            let status_str = match status {
                crate::services::presence_service::PresenceStatus::Online => "online",
                crate::services::presence_service::PresenceStatus::Idle => "idle",
                crate::services::presence_service::PresenceStatus::Dnd => "dnd",
                crate::services::presence_service::PresenceStatus::Offline => "offline",
            };
            let event = crate::ws::protocol::ServerEvent::PresenceUpdated {
                user_id: uid,
                status: status_str.to_string(),
                last_activity: last_activity.to_rfc3339(),
            };
            if let Ok(json) = serde_json::to_string(&event) {
                let _ = tx.send(Message::Text(json));
            }
        }
    }

    let mut rx = UnboundedReceiverStream::new(rx);
    let (mut sender, mut receiver) = socket.split();

    // ---------------------------------------------------------
    // TASK SEND : hub → client WebSocket
    // ---------------------------------------------------------
    tokio::spawn(async move {
        while let Some(msg) = rx.next().await {
            let _ = sender.send(msg).await;
        }
    });

    // Rejoindre la room globale par défaut
    let global_room = "global".to_string();
    hub.join_room(&global_room, conn_id);

    // Note: "global" is a virtual room with no Postgres channel/server.
    // No history is loaded for it — real history comes from JoinChannel events.

    let hub_recv = hub.clone();
    let message_service_recv = message_service.clone();
    let presence_recv = presence.clone();
    let typing_recv = typing_service.clone();
    let user_id_recv = user_id.clone();
    let pg_pool_recv = pg_pool.clone();

    // ---------------------------------------------------------
    // TASK CLEANUP : kill stale connections (no message > 120 s)
    // ---------------------------------------------------------
    let hub_cleanup = hub.clone();
    let cleanup_conn_id = conn_id;
    let cleanup_handle = tokio::spawn(async move {
        let mut ticker =
            tokio::time::interval(std::time::Duration::from_secs(30));
        ticker.tick().await; // skip the immediate first tick
        loop {
            ticker.tick().await;
            match hub_cleanup.heartbeats.get(&cleanup_conn_id) {
                None => break, // connection already unregistered
                Some(last_beat) => {
                    let elapsed =
                        (chrono::Utc::now() - *last_beat).num_seconds();
                    if elapsed > HEARTBEAT_TIMEOUT_SECS {
                        tracing::warn!(
                            conn = %cleanup_conn_id,
                            elapsed_secs = elapsed,
                            "Cleaning up: last_ping > {}s ago",
                            HEARTBEAT_TIMEOUT_SECS
                        );
                        if let Some(tx) =
                            hub_cleanup.sockets.get(&cleanup_conn_id)
                        {
                            let _ = tx.send(Message::Close(None));
                        }
                        break;
                    }
                }
            }
        }
    });

    // ---------------------------------------------------------
    // TASK RECEIVE : client → hub / services
    // ---------------------------------------------------------
    let recv_task = tokio::spawn(async move {
        // Per-channel typing throttle: channel_id → last TypingStart instant
        let mut typing_timestamps: HashMap<String, Instant> = HashMap::new();
        let throttle = std::time::Duration::from_millis(TYPING_THROTTLE_MS);

        // Cache: channel_id (string) → server_id (string) to avoid repeated PG lookups
        let mut channel_server_cache: HashMap<String, String> = HashMap::new();

        // Cache: user_id (string) → username to avoid repeated PG lookups
        let mut username_cache: HashMap<String, String> = HashMap::new();

        let current_user_uuid = match Uuid::parse_str(&user_id_recv) {
            Ok(uid) => uid,
            Err(_) => {
                send_error(&hub_recv, &conn_id, "INVALID_USER_ID", "invalid user identifier");
                if let Some(tx) = hub_recv.sockets.get(&conn_id) {
                    let _ = tx.send(Message::Close(None));
                }
                return;
            }
        };

        while let Some(Ok(msg)) = receiver.next().await {
            let text = match msg {
                Message::Text(t) => t,
                Message::Close(_) => break,
                // Ignore binary / ping / pong frames
                _ => continue,
            };

            // ── Guard 1: frame size ─────────────────────────────
            if text.len() > MAX_FRAME_BYTES {
                tracing::warn!(
                    conn = %conn_id,
                    bytes = text.len(),
                    "Rejected oversized WS frame"
                );
                send_error(
                    &hub_recv,
                    &conn_id,
                    "PAYLOAD_TOO_LARGE",
                    &format!("frame exceeds {} bytes", MAX_FRAME_BYTES),
                );
                continue;
            }

            // ── Guard 2: JSON parse ─────────────────────────────
            let event = match serde_json::from_str::<ClientEvent>(&text) {
                Ok(e) => e,
                Err(e) => {
                    tracing::debug!(conn = %conn_id, "Invalid WS payload: {e}");
                    send_error(
                        &hub_recv,
                        &conn_id,
                        "INVALID_PAYLOAD",
                        "could not parse event (check type / payload fields)",
                    );
                    continue;
                },
            };

            // Any valid client event (MessageSend, JoinChannel, TypingStart, …)
            // resets the heartbeat timer, not just Ping.
            hub_recv.heartbeat(&conn_id);

            match event {
                ClientEvent::MessageSend {
                    channel_id,
                    content,
                    reply_to,
                    attachment_url,
                } => {
                    if let Err(reason) = validate_channel_id(&channel_id) {
                        send_error(&hub_recv, &conn_id, "INVALID_CHANNEL_ID", reason);
                        continue;
                    }
                    if attachment_url.is_none() {
                        if let Err(reason) = validate_content(&content) {
                            send_error(&hub_recv, &conn_id, "INVALID_CONTENT", reason);
                            continue;
                        }
                    } else if content.len() > MAX_CONTENT_LEN {
                        send_error(&hub_recv, &conn_id, "INVALID_CONTENT", "message content exceeds maximum length");
                        continue;
                    }

                    let server_id = match resolve_server_id(
                        &channel_id,
                        &pg_pool_recv,
                        &mut channel_server_cache,
                        &hub_recv,
                        &conn_id,
                    )
                    .await
                    {
                        Some(sid) => sid,
                        None => continue,
                    };

                    let server_uuid = match Uuid::parse_str(&server_id) {
                        Ok(id) => id,
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INTERNAL_ERROR", "invalid server mapping");
                            continue;
                        }
                    };

                    if MembershipRepository::is_banned(&pg_pool_recv, server_uuid, current_user_uuid)
                        .await
                        .unwrap_or(true)
                    {
                        send_error(&hub_recv, &conn_id, "BANNED", "You are banned from this server");
                        if let Some(tx) = hub_recv.sockets.get(&conn_id) {
                            let _ = tx.send(Message::Close(None));
                        }
                        break;
                    }

                    if !MembershipRepository::is_member(&pg_pool_recv, current_user_uuid, server_uuid)
                        .await
                        .unwrap_or(false)
                    {
                        send_error(&hub_recv, &conn_id, "NOT_MEMBER", "You are not a member of this server");
                        continue;
                    }

                    tracing::info!(
                        "📩 MessageSend: server={}, channel={}, len={}",
                        server_id,
                        channel_id,
                        content.len()
                    );
                    let reply_to_oid = if let Some(ref reply_id) = reply_to {
                        match ObjectId::parse_str(reply_id) {
                            Ok(oid) => Some(oid),
                            Err(_) => {
                                send_error(&hub_recv, &conn_id, "INVALID_REPLY_TO", "reply_to must be a valid message id");
                                continue;
                            }
                        }
                    } else {
                        None
                    };
                    let created_at = chrono::Utc::now().to_rfc3339();

                    let id = match message_service_recv
                        .create_message(
                            channel_id.clone(),
                            user_id_recv.clone(),
                            content.clone(),
                            created_at.clone(),
                            reply_to_oid,
                            attachment_url.clone(),
                        )
                        .await
                    {
                        Ok(id) => id,
                        Err(e) => {
                            tracing::error!(
                                channel = %channel_id,
                                user = %user_id_recv,
                                "Failed to persist message in MongoDB: {e}"
                            );
                            send_error(
                                &hub_recv,
                                &conn_id,
                                "MESSAGE_PERSIST_FAILED",
                                "failed to persist message",
                            );
                            continue;
                        },
                    };

                    tracing::info!("💾 Message saved to MongoDB with id={}", id.to_hex());

                    let username = resolve_username(
                        &user_id_recv,
                        &pg_pool_recv,
                        &mut username_cache,
                    )
                    .await;

                    let event = ServerEvent::MessageNew {
                        id: id.to_hex(),
                        channel_id: channel_id.clone(),
                        author_id: user_id_recv.clone(),
                        username: username.clone(),
                        content: content.clone(),
                        created_at: created_at.clone(),
                        reply_to: reply_to.clone(),
                        attachment_url: attachment_url.clone(),
                    };

                    hub_recv.broadcast_room(&channel_id, event).await;
                    // If sender is not in the room, send directly so UI updates.
                    let in_room = hub_recv
                        .rooms
                        .get(&channel_id)
                        .map(|r| r.contains(&conn_id))
                        .unwrap_or(false);
                    if !in_room {
                        if let Some(tx) = hub_recv.sockets.get(&conn_id) {
                            if let Ok(json) = serde_json::to_string(&ServerEvent::MessageNew {
                                id: id.to_hex(),
                                channel_id: channel_id.clone(),
                                author_id: user_id_recv.clone(),
                                username: username.clone(),
                                content: content.clone(),
                                created_at: created_at.clone(),
                                reply_to: reply_to.clone(),
                                attachment_url: attachment_url.clone(),
                            }) {
                                let _ = tx.send(Message::Text(json));
                            }
                        }
                    }
                },
                ClientEvent::MessageSchedule {
                    channel_id,
                    content,
                    day_of_week,
                    hour,
                    minute,
                    reply_to,
                } => {
                    if let Err(reason) = validate_channel_id(&channel_id) {
                        send_error(&hub_recv, &conn_id, "INVALID_CHANNEL_ID", reason);
                        continue;
                    }
                    if let Err(reason) = validate_content(&content) {
                        send_error(&hub_recv, &conn_id, "INVALID_CONTENT", reason);
                        continue;
                    }
                    if let Err(reason) = validate_schedule(day_of_week, hour, minute) {
                        send_error(&hub_recv, &conn_id, "INVALID_SCHEDULE", reason);
                        continue;
                    }

                    let reply_to_oid = if let Some(ref reply_id) = reply_to {
                        match ObjectId::parse_str(reply_id) {
                            Ok(oid) => Some(oid),
                            Err(_) => {
                                send_error(
                                    &hub_recv,
                                    &conn_id,
                                    "INVALID_REPLY_TO",
                                    "reply_to must be a valid message id",
                                );
                                continue;
                            }
                        }
                    } else {
                        None
                    };

                    let server_id = match resolve_server_id(
                        &channel_id,
                        &pg_pool_recv,
                        &mut channel_server_cache,
                        &hub_recv,
                        &conn_id,
                    )
                    .await
                    {
                        Some(sid) => sid,
                        None => continue,
                    };

                    let server_uuid = match Uuid::parse_str(&server_id) {
                        Ok(id) => id,
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INTERNAL_ERROR", "invalid server mapping");
                            continue;
                        }
                    };

                    if MembershipRepository::is_banned(&pg_pool_recv, server_uuid, current_user_uuid)
                        .await
                        .unwrap_or(true)
                    {
                        send_error(&hub_recv, &conn_id, "BANNED", "You are banned from this server");
                        continue;
                    }

                    if !MembershipRepository::is_member(&pg_pool_recv, current_user_uuid, server_uuid)
                        .await
                        .unwrap_or(false)
                    {
                        send_error(&hub_recv, &conn_id, "NOT_MEMBER", "You are not a member of this server");
                        continue;
                    }

                    let now = chrono::Utc::now();
                    let scheduled_for = match next_weekly_occurrence(now, day_of_week, hour, minute) {
                        Some(at) => at,
                        None => {
                            send_error(&hub_recv, &conn_id, "INVALID_SCHEDULE", "invalid schedule values");
                            continue;
                        }
                    };

                    let ack = ServerEvent::MessageScheduled {
                        channel_id: channel_id.clone(),
                        day_of_week,
                        hour,
                        minute,
                        scheduled_for: scheduled_for.to_rfc3339(),
                    };
                    if let Some(tx) = hub_recv.sockets.get(&conn_id) {
                        if let Ok(json) = serde_json::to_string(&ack) {
                            let _ = tx.send(Message::Text(json));
                        }
                    }

                    let hub_scheduled = hub_recv.clone();
                    let message_service_scheduled = message_service_recv.clone();
                    let pg_pool_scheduled = pg_pool_recv.clone();
                    let channel_id_scheduled = channel_id.clone();
                    let user_id_scheduled = user_id_recv.clone();
                    let content_scheduled = content.clone();
                    let reply_to_hex = reply_to.clone();
                    let reply_to_oid_scheduled = reply_to_oid;

                    tokio::spawn(async move {
                        let wait = scheduled_for
                            .signed_duration_since(chrono::Utc::now())
                            .to_std()
                            .unwrap_or_else(|_| std::time::Duration::from_secs(0));
                        tokio::time::sleep(wait).await;

                        let created_at = chrono::Utc::now().to_rfc3339();
                        let id = match message_service_scheduled
                            .create_message(
                                channel_id_scheduled.clone(),
                                user_id_scheduled.clone(),
                                content_scheduled.clone(),
                                created_at.clone(),
                                reply_to_oid_scheduled,
                                None,
                            )
                            .await
                        {
                            Ok(id) => id,
                            Err(e) => {
                                tracing::error!(
                                    channel = %channel_id_scheduled,
                                    user = %user_id_scheduled,
                                    "Failed to persist scheduled message in MongoDB: {e}"
                                );
                                return;
                            }
                        };

                        let username = match Uuid::parse_str(&user_id_scheduled) {
                            Ok(uid) => UserRepository::find_by_id(&pg_pool_scheduled, uid)
                                .await
                                .ok()
                                .flatten()
                                .map(|u| u.username)
                                .unwrap_or_else(|| user_id_scheduled.chars().take(8).collect()),
                            Err(_) => user_id_scheduled.chars().take(8).collect(),
                        };

                        let event = ServerEvent::MessageNew {
                            id: id.to_hex(),
                            channel_id: channel_id_scheduled.clone(),
                            author_id: user_id_scheduled.clone(),
                            username,
                            content: content_scheduled.clone(),
                            created_at,
                            reply_to: reply_to_hex,
                            attachment_url: None,
                        };

                        hub_scheduled.broadcast_room(&channel_id_scheduled, event).await;
                    });
                },
                ClientEvent::MessageEdit {
                    channel_id,
                    message_id,
                    content,
                } => {
                    if let Err(reason) = validate_channel_id(&channel_id) {
                        send_error(&hub_recv, &conn_id, "INVALID_CHANNEL_ID", reason);
                        continue;
                    }
                    if let Err(reason) = validate_content(&content) {
                        send_error(&hub_recv, &conn_id, "INVALID_CONTENT", reason);
                        continue;
                    }
                    let msg_oid = match ObjectId::parse_str(&message_id) {
                        Ok(id) => id,
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INVALID_MESSAGE_ID", "message_id must be a valid ObjectId");
                            continue;
                        }
                    };

                    let _server_id = match resolve_server_id(
                        &channel_id,
                        &pg_pool_recv,
                        &mut channel_server_cache,
                        &hub_recv,
                        &conn_id,
                    )
                    .await
                    {
                        Some(sid) => sid,
                        None => continue,
                    };

                    let existing = match message_service_recv.get_message_by_id(&msg_oid).await {
                        Some(m) => m,
                        None => {
                            send_error(&hub_recv, &conn_id, "MESSAGE_NOT_FOUND", "message not found");
                            continue;
                        }
                    };

                    if existing.channel_id != channel_id {
                        send_error(&hub_recv, &conn_id, "CHANNEL_MISMATCH", "message does not belong to this channel");
                        continue;
                    }

                    if existing.author_id != user_id_recv {
                        send_error(&hub_recv, &conn_id, "FORBIDDEN", "only the message author can edit their message");
                        continue;
                    }

                    let edited_at = chrono::Utc::now().to_rfc3339();
                    if let Err(e) = message_service_recv
                        .edit_message(msg_oid, &channel_id, &user_id_recv, &content)
                        .await
                    {
                        tracing::error!("failed to edit message: {e}");
                        send_error(&hub_recv, &conn_id, "INTERNAL_ERROR", "failed to edit message");
                        continue;
                    }

                    let username = resolve_username(
                        &existing.author_id,
                        &pg_pool_recv,
                        &mut username_cache,
                    )
                    .await;

                    let event = ServerEvent::MessageEdited {
                        id: message_id.clone(),
                        channel_id: channel_id.clone(),
                        author_id: existing.author_id.clone(),
                        username,
                        content: content.clone(),
                        edited_at,
                    };
                    hub_recv.broadcast_room(&channel_id, event).await;
                },
                ClientEvent::MessageDelete {
                    channel_id,
                    message_id,
                } => {
                    if let Err(reason) = validate_channel_id(&channel_id) {
                        send_error(&hub_recv, &conn_id, "INVALID_CHANNEL_ID", reason);
                        continue;
                    }
                    let msg_oid = match ObjectId::parse_str(&message_id) {
                        Ok(id) => id,
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INVALID_MESSAGE_ID", "message_id must be a valid ObjectId");
                            continue;
                        }
                    };

                    let server_id = match resolve_server_id(
                        &channel_id,
                        &pg_pool_recv,
                        &mut channel_server_cache,
                        &hub_recv,
                        &conn_id,
                    )
                    .await
                    {
                        Some(sid) => sid,
                        None => continue,
                    };

                    let existing = match message_service_recv.get_message_by_id(&msg_oid).await {
                        Some(m) => m,
                        None => {
                            send_error(&hub_recv, &conn_id, "MESSAGE_NOT_FOUND", "message not found");
                            continue;
                        }
                    };

                    if existing.channel_id != channel_id {
                        send_error(&hub_recv, &conn_id, "CHANNEL_MISMATCH", "message does not belong to this channel");
                        continue;
                    }

                    let member_role = match Uuid::parse_str(&user_id_recv)
                        .ok()
                        .and_then(|user_uuid| Uuid::parse_str(&server_id).ok().map(|server_uuid| (user_uuid, server_uuid)))
                    {
                        Some((user_uuid, server_uuid)) => {
                            MembershipRepository::get_role(&pg_pool_recv, user_uuid, server_uuid)
                                .await
                                .unwrap_or(None)
                        }
                        None => None,
                    };

                    let is_author = existing.author_id == user_id_recv;
                    let can_delete = is_author
                        || member_role
                            .map(|r| r.can_delete_others_messages())
                            .unwrap_or(false);

                    if !can_delete {
                        send_error(&hub_recv, &conn_id, "FORBIDDEN", "not allowed to delete this message");
                        continue;
                    }

                    if let Err(e) = message_service_recv.delete_message(&msg_oid).await {
                        tracing::error!("failed to delete message: {e}");
                        send_error(&hub_recv, &conn_id, "INTERNAL_ERROR", "failed to delete message");
                        continue;
                    }

                    let event = ServerEvent::MessageDeleted {
                        id: message_id.clone(),
                        channel_id: channel_id.clone(),
                    };
                    hub_recv.broadcast_room(&channel_id, event).await;
                },
                ClientEvent::JoinChannel { channel_id } => {
                    if let Err(reason) = validate_channel_id(&channel_id) {
                        send_error(&hub_recv, &conn_id, "INVALID_CHANNEL_ID", reason);
                        continue;
                    }

                    // ── Resolve server_id from channel ──────────
                    let server_id = match resolve_server_id(
                        &channel_id,
                        &pg_pool_recv,
                        &mut channel_server_cache,
                        &hub_recv,
                        &conn_id,
                    )
                    .await
                    {
                        Some(sid) => sid,
                        None => continue,
                    };

                    let server_uuid = match Uuid::parse_str(&server_id) {
                        Ok(id) => id,
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INTERNAL_ERROR", "invalid server mapping");
                            continue;
                        }
                    };

                    if MembershipRepository::is_banned(&pg_pool_recv, server_uuid, current_user_uuid)
                        .await
                        .unwrap_or(true)
                    {
                        send_error(&hub_recv, &conn_id, "BANNED", "You are banned from this server");
                        if let Some(tx) = hub_recv.sockets.get(&conn_id) {
                            let _ = tx.send(Message::Close(None));
                        }
                        break;
                    }

                    if !MembershipRepository::is_member(&pg_pool_recv, current_user_uuid, server_uuid)
                        .await
                        .unwrap_or(false)
                    {
                        send_error(&hub_recv, &conn_id, "NOT_MEMBER", "You are not a member of this server");
                        continue;
                    }

                    tracing::info!(
                        "🚪 JoinChannel: server={}, channel={}",
                        server_id,
                        channel_id
                    );
                    hub_recv.join_room(&channel_id, conn_id);

                    if let Ok(history) = message_service_recv
                        .get_history(&channel_id, 1, 50)
                        .await
                    {
                        tracing::info!(
                            "📜 Sending {} history messages for channel {}",
                            history.len(),
                            channel_id
                        );
                        if let Some(tx) = hub_recv.sockets.get(&conn_id) {
                            for msg in history {
                                let msg_id = msg.id.map(|oid| oid.to_hex()).unwrap_or_default();
                                let username = resolve_username(
                                    &msg.author_id,
                                    &pg_pool_recv,
                                    &mut username_cache,
                                )
                                .await;
                                let event = ServerEvent::MessageNew {
                                    id: msg_id,
                                    channel_id: msg.channel_id.clone(),
                                    author_id: msg.author_id.clone(),
                                    username,
                                    content: msg.content.clone(),
                                    created_at: msg.created_at.clone(),
                                    reply_to: msg.reply_to.map(|oid| oid.to_hex()),
                                    attachment_url: msg.attachment_url.clone(),
                                };
                                if let Ok(json) = serde_json::to_string(&event) {
                                    let _ = tx.send(Message::Text(json));
                                }
                            }
                        }
                    }

                    // Send a presence snapshot so the joining client immediately
                    // knows who is online without waiting for future events.
                    let presence_users: Vec<crate::ws::protocol::PresenceUser> = presence_recv
                        .snapshot()
                        .into_iter()
                        .map(|(uid, status, _)| {
                            let status_str = match status {
                                crate::services::presence_service::PresenceStatus::Online => "online",
                                crate::services::presence_service::PresenceStatus::Idle => "idle",
                                crate::services::presence_service::PresenceStatus::Dnd => "dnd",
                                crate::services::presence_service::PresenceStatus::Offline => "offline",
                            };
                            crate::ws::protocol::PresenceUser {
                                user_id: uid,
                                status: status_str.to_string(),
                            }
                        })
                        .collect();
                    if let Some(tx) = hub_recv.sockets.get(&conn_id) {
                        let sync_event = ServerEvent::PresenceSync { users: presence_users };
                        if let Ok(json) = serde_json::to_string(&sync_event) {
                            let _ = tx.send(Message::Text(json));
                        }
                    }

                    let event = ServerEvent::UserJoined {
                        user_id: user_id_recv.clone(),
                        channel_id: channel_id.clone(),
                    };
                    hub_recv.broadcast_room(&channel_id, event).await;
                },
                ClientEvent::LeaveChannel { channel_id } => {
                    if let Err(reason) = validate_channel_id(&channel_id) {
                        send_error(&hub_recv, &conn_id, "INVALID_CHANNEL_ID", reason);
                        continue;
                    }

                    hub_recv.leave_room(&channel_id, &conn_id);

                    let event = ServerEvent::UserLeft {
                        user_id: user_id_recv.clone(),
                        channel_id: channel_id.clone(),
                    };
                    hub_recv.broadcast_room(&channel_id, event).await;
                },
                ClientEvent::TypingStart { channel_id } => {
                    if let Err(reason) = validate_channel_id(&channel_id) {
                        send_error(&hub_recv, &conn_id, "INVALID_CHANNEL_ID", reason);
                        continue;
                    }

                    let now = Instant::now();
                    if let Some(last) = typing_timestamps.get(&channel_id) {
                        if now.duration_since(*last) < throttle {
                            // Silently drop — not an error, just throttled
                            continue;
                        }
                    }
                    typing_timestamps.insert(channel_id.clone(), now);

                    // Track in TypingService (shared state)
                    typing_recv.start_typing(&channel_id, &user_id_recv);

                    let username = resolve_username(
                        &user_id_recv,
                        &pg_pool_recv,
                        &mut username_cache,
                    )
                    .await;

                    let event = ServerEvent::TypingStart {
                        user_id: user_id_recv.clone(),
                        username,
                        channel_id: channel_id.clone(),
                    };
                    hub_recv.broadcast_room(&channel_id, event).await;
                },
                ClientEvent::TypingStop { channel_id } => {
                    if let Err(reason) = validate_channel_id(&channel_id) {
                        send_error(&hub_recv, &conn_id, "INVALID_CHANNEL_ID", reason);
                        continue;
                    }

                    // Clear throttle entry so next TypingStart fires immediately
                    typing_timestamps.remove(&channel_id);

                    // Remove from TypingService (shared state)
                    typing_recv.stop_typing(&channel_id, &user_id_recv);

                    let username = resolve_username(
                        &user_id_recv,
                        &pg_pool_recv,
                        &mut username_cache,
                    )
                    .await;

                    let event = ServerEvent::TypingStop {
                        user_id: user_id_recv.clone(),
                        username,
                        channel_id: channel_id.clone(),
                    };
                    hub_recv.broadcast_room(&channel_id, event).await;
                },
                ClientEvent::ReactionAdd { message_id, emoji } => {
                    let message_oid = match ObjectId::parse_str(&message_id) {
                        Ok(id) => id,
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INVALID_MESSAGE_ID", "message_id must be a valid ObjectId");
                            continue;
                        }
                    };

                    // Resolve username for the reacting user
                    let username = resolve_username(
                        &user_id_recv,
                        &pg_pool_recv,
                        &mut username_cache,
                    )
                    .await;

                    match message_service_recv
                        .add_reaction(&message_oid, &emoji, &user_id_recv, Some(&username))
                        .await
                    {
                        Ok((Some(channel_id), was_added)) => {
                            if was_added {
                                let event = ServerEvent::ReactionAdded {
                                    message_id: message_id.clone(),
                                    emoji: emoji.clone(),
                                    user_id: user_id_recv.clone(),
                                    username: Some(username),
                                };
                                hub_recv.broadcast_room(&channel_id, event).await;
                            } else {
                                let event = ServerEvent::ReactionRemoved {
                                    message_id: message_id.clone(),
                                    emoji: emoji.clone(),
                                    user_id: user_id_recv.clone(),
                                };
                                hub_recv.broadcast_room(&channel_id, event).await;
                            }
                        }
                        Ok((None, _)) => {
                            send_error(&hub_recv, &conn_id, "MESSAGE_NOT_FOUND", "message not found");
                        }
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INVALID_MESSAGE_ID", "invalid message id or failed to add/remove reaction");
                        }
                    }
                },
                ClientEvent::ReactionRemove { message_id, emoji } => {
                    let message_oid = match ObjectId::parse_str(&message_id) {
                        Ok(id) => id,
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INVALID_MESSAGE_ID", "message_id must be a valid ObjectId");
                            continue;
                        }
                    };

                    match message_service_recv
                        .remove_reaction(&message_oid, &emoji, &user_id_recv)
                        .await
                    {
                        Ok(Some(channel_id)) => {
                            let event = ServerEvent::ReactionRemoved {
                                message_id: message_id.clone(),
                                emoji: emoji.clone(),
                                user_id: user_id_recv.clone(),
                            };
                            hub_recv.broadcast_room(&channel_id, event).await;
                        }
                        Ok(None) => {
                            send_error(&hub_recv, &conn_id, "MESSAGE_NOT_FOUND", "message not found");
                        }
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INVALID_MESSAGE_ID", "invalid message id or failed to remove reaction");
                        }
                    }
                },
                ClientEvent::Ping => {
                    hub_recv.heartbeat(&conn_id);
                    // Refresh presence timestamp; if status changed (idle -> online) broadcast update
                    let changed = presence_recv.refresh_activity(&user_id_recv);
                    if changed {
                        let event = crate::ws::protocol::ServerEvent::PresenceUpdated {
                            user_id: user_id_recv.clone(),
                            status: "online".to_string(),
                            last_activity: chrono::Utc::now().to_rfc3339(),
                        };
                        hub_recv.broadcast_all(event).await;
                    }
                    if let Some(tx) = hub_recv.sockets.get(&conn_id) {
                        let event = ServerEvent::Pong;
                        if let Ok(json) = serde_json::to_string(&event) {
                            let _ = tx.send(Message::Text(json));
                        }
                    }
                },
                ClientEvent::PresenceSet { status } => {
                    // Map string -> PresenceStatus
                    let st = match status.as_str() {
                        "online" => crate::services::presence_service::PresenceStatus::Online,
                        "idle" => crate::services::presence_service::PresenceStatus::Idle,
                        "dnd" => crate::services::presence_service::PresenceStatus::Dnd,
                        "offline" => crate::services::presence_service::PresenceStatus::Offline,
                        other => {
                            send_error(&hub_recv, &conn_id, "INVALID_PRESENCE", &format!("unknown status: {}", other));
                            continue;
                        }
                    };

                    let changed = presence_recv.set_status(&user_id_recv, st);
                    if changed {
                        let event = crate::ws::protocol::ServerEvent::PresenceUpdated {
                            user_id: user_id_recv.clone(),
                            status: status.clone(),
                            last_activity: chrono::Utc::now().to_rfc3339(),
                        };
                        hub_recv.broadcast_all(event).await;
                    }
                },
                ClientEvent::MessageSendGif { channel_id, gif, caption } => {
                    if let Err(reason) = validate_channel_id(&channel_id) {
                        send_error(&hub_recv, &conn_id, "INVALID_CHANNEL_ID", reason);
                        continue;
                    }

                    if gif.id.trim().is_empty() || gif.url.trim().is_empty() {
                        send_error(&hub_recv, &conn_id, "INVALID_GIF", "gif id and url are required");
                        continue;
                    }

                    let _server_id = match resolve_server_id(
                        &channel_id,
                        &pg_pool_recv,
                        &mut channel_server_cache,
                        &hub_recv,
                        &conn_id,
                    )
                    .await
                    {
                        Some(sid) => sid,
                        None => continue,
                    };

                    let caption_text = caption.unwrap_or_default();
                    let content = serde_json::json!({
                        "type": "gif",
                        "gif": {
                            "id": gif.id,
                            "url": gif.url,
                            "preview": gif.preview,
                            "provider": gif.provider,
                        },
                        "caption": caption_text,
                    })
                    .to_string();

                    let created_at = chrono::Utc::now().to_rfc3339();

                    let id = match message_service_recv
                        .create_message(
                            channel_id.clone(),
                            user_id_recv.clone(),
                            content.clone(),
                            created_at.clone(),
                            None,
                            None,
                        )
                        .await
                    {
                        Ok(id) => id,
                        Err(e) => {
                            tracing::error!(
                                channel = %channel_id,
                                user = %user_id_recv,
                                "Failed to persist GIF message in MongoDB: {e}"
                            );
                            send_error(
                                &hub_recv,
                                &conn_id,
                                "MESSAGE_PERSIST_FAILED",
                                "failed to persist message",
                            );
                            continue;
                        }
                    };

                    tracing::info!("💾 GIF message saved to MongoDB with id={}", id.to_hex());

                    let username = resolve_username(
                        &user_id_recv,
                        &pg_pool_recv,
                        &mut username_cache,
                    )
                    .await;

                    let event = ServerEvent::MessageNew {
                        id: id.to_hex(),
                        channel_id: channel_id.clone(),
                        author_id: user_id_recv.clone(),
                        username: username.clone(),
                        content: content.clone(),
                        created_at: created_at.clone(),
                        reply_to: None,
                        attachment_url: None,
                    };
                    hub_recv.broadcast_room(&channel_id, event).await;
                    let in_room = hub_recv
                        .rooms
                        .get(&channel_id)
                        .map(|r| r.contains(&conn_id))
                        .unwrap_or(false);
                    if !in_room {
                        if let Some(tx) = hub_recv.sockets.get(&conn_id) {
                            if let Ok(json) = serde_json::to_string(&ServerEvent::MessageNew {
                                id: id.to_hex(),
                                channel_id: channel_id.clone(),
                                author_id: user_id_recv.clone(),
                                username: username.clone(),
                                content: content.clone(),
                                created_at: created_at.clone(),
                                reply_to: None,
                                attachment_url: None,
                            }) {
                                let _ = tx.send(Message::Text(json));
                            }
                        }
                    }
                },
                // ─────────────────────────────────────────────
                // DIRECT MESSAGES
                // ─────────────────────────────────────────────
                ClientEvent::DmSend { recipient_id, content, reply_to, attachment_url } => {
                    // Validate recipient UUID
                    let recipient_uuid = match Uuid::parse_str(&recipient_id) {
                        Ok(u) => u,
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INVALID_RECIPIENT", "recipient_id is not a valid UUID");
                            continue;
                        }
                    };
                    if attachment_url.is_none() {
                        if let Err(reason) = validate_content(&content) {
                            send_error(&hub_recv, &conn_id, "INVALID_CONTENT", reason);
                            continue;
                        }
                    } else if content.len() > MAX_CONTENT_LEN {
                        send_error(&hub_recv, &conn_id, "INVALID_CONTENT", "message content exceeds maximum length");
                        continue;
                    }
                    // Prevent sending DM to self
                    if recipient_id == user_id_recv {
                        send_error(&hub_recv, &conn_id, "INVALID_RECIPIENT", "cannot send DM to yourself");
                        continue;
                    }
                    // Check recipient exists
                    if UserRepository::find_by_id(&pg_pool_recv, recipient_uuid).await.ok().flatten().is_none() {
                        send_error(&hub_recv, &conn_id, "RECIPIENT_NOT_FOUND", "recipient user does not exist");
                        continue;
                    }

                    let conversation_id = dm_conversation_id(&user_id_recv, &recipient_id);

                    let reply_to_oid = if let Some(ref reply_id) = reply_to {
                        match ObjectId::parse_str(reply_id) {
                            Ok(oid) => Some(oid),
                            Err(_) => {
                                send_error(&hub_recv, &conn_id, "INVALID_REPLY_TO", "reply_to must be a valid message id");
                                continue;
                            }
                        }
                    } else {
                        None
                    };

                    let created_at = chrono::Utc::now().to_rfc3339();
                    let id = match message_service_recv
                        .create_message(
                            conversation_id.clone(),
                            user_id_recv.clone(),
                            content.clone(),
                            created_at.clone(),
                            reply_to_oid,
                            attachment_url.clone(),
                        )
                        .await
                    {
                        Ok(id) => id,
                        Err(e) => {
                            tracing::error!(
                                conversation = %conversation_id,
                                user = %user_id_recv,
                                recipient = %recipient_id,
                                "Failed to persist DM in MongoDB: {e}"
                            );
                            send_error(
                                &hub_recv,
                                &conn_id,
                                "MESSAGE_PERSIST_FAILED",
                                "failed to persist message",
                            );
                            continue;
                        }
                    };

                    tracing::info!("💬 DM saved: id={}, conv={}", id.to_hex(), conversation_id);

                    let username = resolve_username(&user_id_recv, &pg_pool_recv, &mut username_cache).await;

                    let event = ServerEvent::DmNew {
                        id: id.to_hex(),
                        conversation_id: conversation_id.clone(),
                        author_id: user_id_recv.clone(),
                        username,
                        content,
                        created_at,
                        reply_to,
                        attachment_url,
                    };

                    // Broadcast to the DM room (users currently viewing)
                    hub_recv.broadcast_room(&conversation_id, event.clone()).await;

                    // Also send to ALL connections of the recipient (notification)
                    if let Some(conns) = hub_recv.connections.get(&recipient_id) {
                        if let Ok(json) = serde_json::to_string(&event) {
                            for rid in conns.iter() {
                                // Skip if already in the DM room (avoid duplicate)
                                let in_room = hub_recv.rooms.get(&conversation_id)
                                    .map(|r| r.contains(&*rid))
                                    .unwrap_or(false);
                                if !in_room {
                                    if let Some(tx) = hub_recv.sockets.get(&*rid) {
                                        let _ = tx.send(Message::Text(json.clone()));
                                    }
                                }
                            }
                        }
                    }
                },
                ClientEvent::JoinDm { peer_id } => {
                    let peer_uuid = match Uuid::parse_str(&peer_id) {
                        Ok(u) => u,
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INVALID_PEER_ID", "peer_id is not a valid UUID");
                            continue;
                        }
                    };
                    if UserRepository::find_by_id(&pg_pool_recv, peer_uuid).await.ok().flatten().is_none() {
                        send_error(&hub_recv, &conn_id, "PEER_NOT_FOUND", "peer user does not exist");
                        continue;
                    }

                    let conversation_id = dm_conversation_id(&user_id_recv, &peer_id);
                    tracing::info!("🚪 JoinDm: conv={}", conversation_id);
                    hub_recv.join_room(&conversation_id, conn_id);

                    // Send DM history
                    if let Ok(history) = message_service_recv
                        .get_history(&conversation_id, 1, 50)
                        .await
                    {
                        tracing::info!("📜 Sending {} DM history messages for {}", history.len(), conversation_id);
                        if let Some(tx) = hub_recv.sockets.get(&conn_id) {
                            for msg in history {
                                let msg_id = msg.id.map(|oid| oid.to_hex()).unwrap_or_default();
                                let username = resolve_username(&msg.author_id, &pg_pool_recv, &mut username_cache).await;
                                let event = ServerEvent::DmNew {
                                    id: msg_id,
                                    conversation_id: msg.channel_id.clone(),
                                    author_id: msg.author_id.clone(),
                                    username,
                                    content: msg.content.clone(),
                                    created_at: msg.created_at.clone(),
                                    reply_to: msg.reply_to.map(|oid| oid.to_hex()),
                                    attachment_url: msg.attachment_url.clone(),
                                };
                                if let Ok(json) = serde_json::to_string(&event) {
                                    let _ = tx.send(Message::Text(json));
                                }
                            }
                        }
                    }
                },
                ClientEvent::LeaveDm { peer_id } => {
                    let conversation_id = dm_conversation_id(&user_id_recv, &peer_id);
                    hub_recv.leave_room(&conversation_id, &conn_id);
                },
                ClientEvent::DmEdit { conversation_id, message_id, content } => {
                    if let Err(reason) = validate_content(&content) {
                        send_error(&hub_recv, &conn_id, "INVALID_CONTENT", reason);
                        continue;
                    }
                    // Verify the user is part of this DM conversation
                    if !conversation_id.starts_with("dm:") || !conversation_id.contains(&user_id_recv) {
                        send_error(&hub_recv, &conn_id, "FORBIDDEN", "not part of this DM conversation");
                        continue;
                    }
                    let msg_oid = match ObjectId::parse_str(&message_id) {
                        Ok(id) => id,
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INVALID_MESSAGE_ID", "message_id must be a valid ObjectId");
                            continue;
                        }
                    };
                    let existing = match message_service_recv.get_message_by_id(&msg_oid).await {
                        Some(m) => m,
                        None => {
                            send_error(&hub_recv, &conn_id, "MESSAGE_NOT_FOUND", "message not found");
                            continue;
                        }
                    };
                    if existing.author_id != user_id_recv {
                        send_error(&hub_recv, &conn_id, "FORBIDDEN", "only the author can edit their DM");
                        continue;
                    }
                    let edited_at = chrono::Utc::now().to_rfc3339();
                    if let Err(e) = message_service_recv
                        .edit_message(msg_oid, &conversation_id, &user_id_recv, &content)
                        .await
                    {
                        tracing::error!("failed to edit DM: {e}");
                        send_error(&hub_recv, &conn_id, "INTERNAL_ERROR", "failed to edit DM");
                        continue;
                    }
                    let username = resolve_username(&user_id_recv, &pg_pool_recv, &mut username_cache).await;
                    let event = ServerEvent::DmEdited {
                        id: message_id,
                        conversation_id: conversation_id.clone(),
                        author_id: user_id_recv.clone(),
                        username,
                        content,
                        edited_at,
                    };
                    hub_recv.broadcast_room(&conversation_id, event).await;
                },
                ClientEvent::DmDelete { conversation_id, message_id } => {
                    if !conversation_id.starts_with("dm:") || !conversation_id.contains(&user_id_recv) {
                        send_error(&hub_recv, &conn_id, "FORBIDDEN", "not part of this DM conversation");
                        continue;
                    }
                    let msg_oid = match ObjectId::parse_str(&message_id) {
                        Ok(id) => id,
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INVALID_MESSAGE_ID", "message_id must be a valid ObjectId");
                            continue;
                        }
                    };
                    let existing = match message_service_recv.get_message_by_id(&msg_oid).await {
                        Some(m) => m,
                        None => {
                            send_error(&hub_recv, &conn_id, "MESSAGE_NOT_FOUND", "message not found");
                            continue;
                        }
                    };
                    if existing.author_id != user_id_recv {
                        send_error(&hub_recv, &conn_id, "FORBIDDEN", "only the author can delete their DM");
                        continue;
                    }
                    if let Err(e) = message_service_recv.delete_message(&msg_oid).await {
                        tracing::error!("failed to delete DM: {e}");
                        send_error(&hub_recv, &conn_id, "INTERNAL_ERROR", "failed to delete DM");
                        continue;
                    }
                    let event = ServerEvent::DmDeleted {
                        id: message_id,
                        conversation_id: conversation_id.clone(),
                    };
                    hub_recv.broadcast_room(&conversation_id, event).await;
                },
                ClientEvent::DmSendGif { recipient_id, gif, caption } => {
                    let recipient_uuid = match Uuid::parse_str(&recipient_id) {
                        Ok(u) => u,
                        Err(_) => {
                            send_error(&hub_recv, &conn_id, "INVALID_RECIPIENT", "recipient_id is not a valid UUID");
                            continue;
                        }
                    };
                    if recipient_id == user_id_recv {
                        send_error(&hub_recv, &conn_id, "INVALID_RECIPIENT", "cannot send DM to yourself");
                        continue;
                    }
                    if gif.id.trim().is_empty() || gif.url.trim().is_empty() {
                        send_error(&hub_recv, &conn_id, "INVALID_GIF", "gif id and url are required");
                        continue;
                    }
                    if UserRepository::find_by_id(&pg_pool_recv, recipient_uuid).await.ok().flatten().is_none() {
                        send_error(&hub_recv, &conn_id, "RECIPIENT_NOT_FOUND", "recipient user does not exist");
                        continue;
                    }
                    let conversation_id = dm_conversation_id(&user_id_recv, &recipient_id);
                    let caption_text = caption.unwrap_or_default();
                    let content = serde_json::json!({
                        "type": "gif",
                        "gif": {
                            "id": gif.id,
                            "url": gif.url,
                            "preview": gif.preview,
                            "provider": gif.provider,
                        },
                        "caption": caption_text,
                    })
                    .to_string();
                    let created_at = chrono::Utc::now().to_rfc3339();
                    let id = match message_service_recv
                        .create_message(
                            conversation_id.clone(),
                            user_id_recv.clone(),
                            content.clone(),
                            created_at.clone(),
                            None,
                            None,
                        )
                        .await
                    {
                        Ok(id) => id,
                        Err(e) => {
                            tracing::error!(
                                conversation = %conversation_id,
                                user = %user_id_recv,
                                recipient = %recipient_id,
                                "Failed to persist DM GIF in MongoDB: {e}"
                            );
                            send_error(
                                &hub_recv,
                                &conn_id,
                                "MESSAGE_PERSIST_FAILED",
                                "failed to persist message",
                            );
                            continue;
                        }
                    };
                    tracing::info!("💾 DM GIF saved: id={}, conv={}", id.to_hex(), conversation_id);
                    let username = resolve_username(&user_id_recv, &pg_pool_recv, &mut username_cache).await;
                    let event = ServerEvent::DmNew {
                        id: id.to_hex(),
                        conversation_id: conversation_id.clone(),
                        author_id: user_id_recv.clone(),
                        username,
                        content,
                        created_at,
                        reply_to: None,
                        attachment_url: None,
                    };
                    hub_recv.broadcast_room(&conversation_id, event.clone()).await;
                    // Notify recipient connections not in the room
                    if let Some(conns) = hub_recv.connections.get(&recipient_id) {
                        if let Ok(json) = serde_json::to_string(&event) {
                            for rid in conns.iter() {
                                let in_room = hub_recv.rooms.get(&conversation_id)
                                    .map(|r| r.contains(&*rid))
                                    .unwrap_or(false);
                                if !in_room {
                                    if let Some(tx) = hub_recv.sockets.get(&*rid) {
                                        let _ = tx.send(Message::Text(json.clone()));
                                    }
                                }
                            }
                        }
                    }
                },
                            }
        }
    });

    // Attendre la fin de la task
    let _ = recv_task.await;
    cleanup_handle.abort(); // connexion fermée normalement, plus besoin du watcher

    // Nettoyage connexion
    // Log connection cleanup for debugging resets and presence transitions
    let conn_count_before = hub
        .connections
        .get(&user_id)
        .map(|s| s.len())
        .unwrap_or(0);
    tracing::info!(user = %user_id, conn = %conn_id, before = conn_count_before, "Cleaning up connection");

    hub.unregister_connection(&user_id, &conn_id);

    let conn_count_after = hub
        .connections
        .get(&user_id)
        .map(|s| s.len())
        .unwrap_or(0);
    tracing::info!(user = %user_id, conn = %conn_id, after = conn_count_after, "Connection unregistered");

    // Clean up typing state for this user
    typing_service.cleanup();

    // Retirer la connexion de la présence. Si plus aucune connexion => offline.
    if let Some(new_status) = presence.remove_connection(&user_id, &conn_id.to_string()) {
        tracing::info!(user = %user_id, new_status = ?new_status, "Presence changed after removing connection");
        match new_status {
            crate::services::presence_service::PresenceStatus::Offline => {
                // Persist OFFLINE in PG (fire-and-forget).
                if let Ok(uid) = Uuid::parse_str(&user_id) {
                    let pool_pg = pg_pool.clone();
                    tokio::spawn(async move {
                        let _ = UserRepository::update_status(
                            &pool_pg,
                            uid,
                            &crate::models::user::UserStatus::Offline,
                        )
                        .await;
                    });
                }

                hub.broadcast_all(ServerEvent::UserOffline {
                    user_id: user_id.clone(),
                    status: "offline".to_string(),
                })
                .await;
                hub.broadcast_all(crate::ws::protocol::ServerEvent::PresenceUpdated {
                    user_id: user_id.clone(),
                    status: "offline".to_string(),
                    last_activity: chrono::Utc::now().to_rfc3339(),
                })
                .await;
            }
            _ => {}
        }
    } else {
        tracing::info!(user = %user_id, "Connection removed but user still has other connections");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn send_error_sends_error_event_on_socket() {
        let hub = Hub::new();
        let (tx, mut rx) = unbounded_channel();
        let conn_id = Uuid::new_v4();
        hub.sockets.insert(conn_id, tx);

        send_error(&hub, &conn_id, "TEST_CODE", "Something went wrong");

        let msg = rx.try_recv().expect("expected a message");
        match msg {
            Message::Text(text) => {
                let v: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(v["type"], "Error");
                assert_eq!(v["payload"]["code"], "TEST_CODE");
                assert_eq!(v["payload"]["message"], "Something went wrong");
            }
            other => panic!("unexpected message: {:?}", other),
        }
    }

    #[tokio::test]
    async fn resolve_username_falls_back_to_prefix_for_invalid_uuid() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://epitalk:Epitalk94!@localhost:5432/epitalk")
            .expect("lazy pool");
        let mut cache = HashMap::new();

        let user_id = "not-a-uuid-but-long";
        let name = resolve_username(user_id, &pool, &mut cache).await;

        assert_eq!(name, user_id.chars().take(8).collect::<String>());
    }

    #[tokio::test]
    async fn resolve_server_id_invalid_uuid_sends_error_and_returns_none() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://epitalk:Epitalk94!@localhost:5432/epitalk")
            .expect("lazy pool");
        let hub = Hub::new();
        let (tx, mut rx) = unbounded_channel();
        let conn_id = Uuid::new_v4();
        hub.sockets.insert(conn_id, tx);
        let mut cache = HashMap::new();

        let result = resolve_server_id("not-a-uuid", &pool, &mut cache, &hub, &conn_id).await;
        assert!(result.is_none());

        let msg = rx.try_recv().expect("expected an error message");
        match msg {
            Message::Text(text) => {
                let v: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(v["type"], "Error");
                assert_eq!(v["payload"]["code"], "INVALID_CHANNEL_ID");
            }
            other => panic!("unexpected message: {:?}", other),
        }
    }
}
