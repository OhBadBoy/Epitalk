//! Channel routes
//!
//! Routes (nested under /servers/:server_id/channels):
//! - GET    /                          - List server channels
//! - POST   /                          - Create a channel (ADMIN+)
//! - GET    /:channel_id               - Get channel details
//! - PATCH  /:channel_id               - Update channel (ADMIN+)
//! - DELETE /:channel_id               - Delete channel (ADMIN+)
//! - GET    /:channel_id/messages      - Get message history
//! - GET    /:channel_id/messages/search?q=... - Search messages in channel
//! - GET    /:channel_id/messages/pinned - List pinned messages
//! - PATCH  /:channel_id/messages/:message_id - Edit own message
//! - POST   /:channel_id/messages/:message_id/pin - Pin message (MOD+)
//! - DELETE /:channel_id/messages/:message_id/pin - Unpin message (MOD+)

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::RequireAuth;
use crate::error::{AppError, AppResult};
use crate::models::{ChannelResponse, CreateChannelRequest, UpdateChannelRequest};
use crate::repositories::{ChannelRepository, MembershipRepository, UserRepository};
use crate::state::AppState;
use crate::ws::protocol::ServerEvent;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_channels).post(create_channel))
        .route(
            "/:channel_id",
            get(get_channel)
                .patch(update_channel)
                .delete(delete_channel),
        )
        .route("/:channel_id/messages", get(get_messages))
        .route("/:channel_id/messages/search", get(search_messages))
        .route("/:channel_id/messages/pinned", get(list_pinned_messages))
        .route("/:channel_id/messages/:message_id/pin", post(pin_message_handler).delete(unpin_message_handler))
        .route("/:channel_id/upload", post(upload_attachment).layer(DefaultBodyLimit::max(10 * 1024 * 1024)))
}

/// Path parameters for nested routes
#[derive(serde::Deserialize)]
pub struct ServerPath {
    pub server_id: Uuid,
}

#[derive(serde::Deserialize)]
pub struct ChannelPath {
    pub server_id: Uuid,
    pub channel_id: Uuid,
}

#[derive(serde::Deserialize)]
pub struct MessagePath {
    pub server_id: Uuid,
    pub channel_id: Uuid,
    pub message_id: String,
}

/// List all channels in a server
async fn list_channels(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ServerPath>,
) -> AppResult<Json<Vec<ChannelResponse>>> {
    let user_id = auth.user_id;

    // Check membership
    if !MembershipRepository::is_member(&state.db, user_id, params.server_id).await? {
        return Err(AppError::Forbidden(
            "Not a member of this server".to_string(),
        ));
    }

    let channels = ChannelRepository::find_by_server(&state.db, params.server_id).await?;
    let responses: Vec<ChannelResponse> = channels.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// Create a new channel (ADMIN+ only)
async fn create_channel(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ServerPath>,
    Json(payload): Json<CreateChannelRequest>,
) -> AppResult<Json<ChannelResponse>> {
    let user_id = auth.user_id;

    // Check role
    let role = MembershipRepository::get_role(&state.db, user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if !role.can_manage_channels() {
        return Err(AppError::Forbidden(
            "Insufficient permissions to create channels".to_string(),
        ));
    }

    let channel =
        ChannelRepository::create(&state.db, params.server_id, &payload.name, payload.kind).await?;

    Ok(Json(channel.into()))
}

/// Get channel details
async fn get_channel(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ChannelPath>,
) -> AppResult<Json<ChannelResponse>> {
    let user_id = auth.user_id;

    // Check membership
    if !MembershipRepository::is_member(&state.db, user_id, params.server_id).await? {
        return Err(AppError::Forbidden(
            "Not a member of this server".to_string(),
        ));
    }

    let channel = ChannelRepository::find_by_id(&state.db, params.channel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;

    // Verify channel belongs to server
    if channel.server_id != params.server_id {
        return Err(AppError::NotFound(
            "Channel not found in this server".to_string(),
        ));
    }

    Ok(Json(channel.into()))
}

/// Update channel (ADMIN+ only)
async fn update_channel(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ChannelPath>,
    Json(payload): Json<UpdateChannelRequest>,
) -> AppResult<Json<ChannelResponse>> {
    let user_id = auth.user_id;

    // Check role
    let role = MembershipRepository::get_role(&state.db, user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if !role.can_manage_channels() {
        return Err(AppError::Forbidden(
            "Insufficient permissions to update channels".to_string(),
        ));
    }

    // Verify channel belongs to server
    let existing = ChannelRepository::find_by_id(&state.db, params.channel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;

    if existing.server_id != params.server_id {
        return Err(AppError::NotFound(
            "Channel not found in this server".to_string(),
        ));
    }

    let channel = ChannelRepository::update(&state.db, params.channel_id, &payload.name).await?;

    Ok(Json(channel.into()))
}

/// Delete channel (ADMIN+ only)
async fn delete_channel(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ChannelPath>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = auth.user_id;

    // Check role
    let role = MembershipRepository::get_role(&state.db, user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if !role.can_manage_channels() {
        return Err(AppError::Forbidden(
            "Insufficient permissions to delete channels".to_string(),
        ));
    }

    // Verify channel belongs to server
    let existing = ChannelRepository::find_by_id(&state.db, params.channel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;

    if existing.server_id != params.server_id {
        return Err(AppError::NotFound(
            "Channel not found in this server".to_string(),
        ));
    }

    ChannelRepository::delete(&state.db, params.channel_id).await?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

// ---------------------------------------------------------
// MESSAGE HISTORY
// ---------------------------------------------------------

#[derive(Deserialize)]
pub struct MessagesQuery {
    pub before: Option<String>,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    1
}
fn default_per_page() -> u64 {
    50
}

const MAX_SEARCH_QUERY_LEN: usize = 200;

fn normalize_search_query(input: &str) -> AppResult<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(
            "Search query must not be empty".to_string(),
        ));
    }
    if trimmed.len() > MAX_SEARCH_QUERY_LEN {
        return Err(AppError::BadRequest(
            "Search query is too long".to_string(),
        ));
    }
    Ok(trimmed.to_string())
}

#[derive(Serialize)]
pub struct ReactionResponse {
    pub emoji: String,
    pub user_id: String,
    pub username: Option<String>,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub server_id: String,
    pub channel_id: String,
    pub author_id: String,
    pub username: String,
    pub content: String,
    pub created_at: String,
    pub reply_to: Option<String>,
    pub attachment_url: Option<String>,
    pub reactions: Vec<ReactionResponse>,
    pub pinned_by: Option<String>,
    pub pinned_at: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

/// Get message history for a channel
async fn get_messages(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ChannelPath>,
    Query(query): Query<MessagesQuery>,
) -> AppResult<Json<Vec<MessageResponse>>> {
    let user_id = auth.user_id;

    // Check membership
    if !MembershipRepository::is_member(&state.db, user_id, params.server_id).await? {
        return Err(AppError::Forbidden(
            "Not a member of this server".to_string(),
        ));
    }

    // Verify channel belongs to server
    let channel = ChannelRepository::find_by_id(&state.db, params.channel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;

    if channel.server_id != params.server_id {
        return Err(AppError::NotFound(
            "Channel not found in this server".to_string(),
        ));
    }

    // Get messages from MongoDB (cursor-based if `before` is provided, otherwise page 1)
    let messages = if let Some(ref before_hex) = query.before {
        let before_oid = ObjectId::parse_str(before_hex)
            .map_err(|_| AppError::BadRequest("invalid before cursor".to_string()))?;
        state
            .message_service
            .get_history_before(&params.channel_id.to_string(), &before_oid, query.per_page)
            .await
            .map_err(|_| {
                tracing::error!(channel_id = %params.channel_id, "Failed to fetch messages before cursor");
                AppError::Internal("Failed to fetch messages".to_string())
            })?
    } else {
        state
            .message_service
            .get_history(&params.channel_id.to_string(), 1, query.per_page)
            .await
            .map_err(|_| {
                tracing::error!(channel_id = %params.channel_id, "Failed to fetch messages");
                AppError::Internal("Failed to fetch messages".to_string())
            })?
    };

    let responses: Vec<MessageResponse> = {
        let mut result = Vec::new();
        for m in messages {
            let username = if let Ok(uuid) = uuid::Uuid::parse_str(&m.author_id) {
                UserRepository::find_by_id(&state.db, uuid)
                    .await
                    .ok()
                    .flatten()
                    .map(|u| u.username)
                    .unwrap_or_else(|| m.author_id.chars().take(8).collect())
            } else {
                m.author_id.chars().take(8).collect()
            };
            let reactions = m.reactions.unwrap_or_default().into_iter().map(|r| ReactionResponse {
                emoji: r.emoji,
                user_id: r.user_id,
                username: r.username,
            }).collect();
            result.push(MessageResponse {
                id: m.id.map(|oid| oid.to_hex()).unwrap_or_default(),
                server_id: params.server_id.to_string(),
                channel_id: m.channel_id,
                author_id: m.author_id,
                username,
                content: m.content,
                created_at: m.created_at,
                reply_to: m.reply_to.map(|oid| oid.to_hex()),
                attachment_url: m.attachment_url,
                reactions,
                pinned_by: m.pinned_by,
                pinned_at: m.pinned_at,
            });
        }
        result
    };

    Ok(Json(responses))
}

/// Search messages in a channel (simple content search)
async fn search_messages(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ChannelPath>,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Vec<MessageResponse>>> {
    let user_id = auth.user_id;

    let normalized_query = normalize_search_query(&query.q)?;

    // Check membership
    if !MembershipRepository::is_member(&state.db, user_id, params.server_id).await? {
        return Err(AppError::Forbidden(
            "Not a member of this server".to_string(),
        ));
    }

    // Verify channel belongs to server
    let channel = ChannelRepository::find_by_id(&state.db, params.channel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;

    if channel.server_id != params.server_id {
        return Err(AppError::NotFound(
            "Channel not found in this server".to_string(),
        ));
    }

    // Use message service to search
    let messages = state
        .message_service
        .search_messages(&params.channel_id.to_string(), &normalized_query, query.page, query.per_page)
        .await
        .map_err(|_| {
            tracing::error!(channel_id = %params.channel_id, "Failed to search messages");
            AppError::Internal("Failed to search messages".to_string())
        })?;

    let responses: Vec<MessageResponse> = {
        let mut result = Vec::new();
        for m in messages {
            let username = if let Ok(uuid) = uuid::Uuid::parse_str(&m.author_id) {
                UserRepository::find_by_id(&state.db, uuid)
                    .await
                    .ok()
                    .flatten()
                    .map(|u| u.username)
                    .unwrap_or_else(|| m.author_id.chars().take(8).collect())
            } else {
                m.author_id.chars().take(8).collect()
            };
            let reactions = m.reactions.unwrap_or_default().into_iter().map(|r| ReactionResponse {
                emoji: r.emoji,
                user_id: r.user_id,
                username: r.username,
            }).collect();
            result.push(MessageResponse {
                id: m.id.map(|oid| oid.to_hex()).unwrap_or_default(),
                server_id: params.server_id.to_string(),
                channel_id: m.channel_id,
                author_id: m.author_id,
                username,
                content: m.content,
                created_at: m.created_at,
                reply_to: m.reply_to.map(|oid| oid.to_hex()),
                attachment_url: m.attachment_url,
                reactions,
                pinned_by: m.pinned_by,
                pinned_at: m.pinned_at,
            });
        }
        result
    };

    Ok(Json(responses))
}

// ---------------------------------------------------------
// PINNED MESSAGES
// ---------------------------------------------------------

#[derive(Serialize)]
pub struct PinnedMessageResponse {
    pub id: String,
    pub server_id: String,
    pub channel_id: String,
    pub author_id: String,
    pub username: String,
    pub content: String,
    pub created_at: String,
    pub pinned_by: Option<String>,
    pub pinned_at: Option<String>,
}

async fn list_pinned_messages(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ChannelPath>,
    Query(query): Query<PaginationQuery>,
) -> AppResult<Json<Vec<PinnedMessageResponse>>> {
    if !MembershipRepository::is_member(&state.db, auth.user_id, params.server_id).await? {
        return Err(AppError::Forbidden("Not a member of this server".to_string()));
    }

    let channel = ChannelRepository::find_by_id(&state.db, params.channel_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Channel not found".to_string()))?;
    if channel.server_id != params.server_id {
        return Err(AppError::NotFound("Channel not found in this server".to_string()));
    }

    let messages = state.message_service
        .list_pinned(&params.channel_id.to_string(), query.page, query.per_page)
        .await;

    let mut result = Vec::new();
    for m in messages {
        let username = if let Ok(uuid) = Uuid::parse_str(&m.author_id) {
            UserRepository::find_by_id(&state.db, uuid)
                .await.ok().flatten()
                .map(|u| u.username)
                .unwrap_or_else(|| m.author_id.chars().take(8).collect())
        } else {
            m.author_id.chars().take(8).collect()
        };
        result.push(PinnedMessageResponse {
            id: m.id.map(|o| o.to_hex()).unwrap_or_default(),
            server_id: params.server_id.to_string(),
            channel_id: m.channel_id,
            author_id: m.author_id,
            username,
            content: m.content,
            created_at: m.created_at,
            pinned_by: m.pinned_by,
            pinned_at: m.pinned_at,
        });
    }
    Ok(Json(result))
}

async fn pin_message_handler(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<MessagePath>,
) -> AppResult<Json<serde_json::Value>> {
    let role = MembershipRepository::get_role(&state.db, auth.user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member".to_string()))?;
    if !role.can_delete_others_messages() {
        return Err(AppError::Forbidden("Insufficient permissions to pin messages".to_string()));
    }

    let message_id = ObjectId::parse_str(&params.message_id)
        .map_err(|_| AppError::BadRequest("Invalid message id".to_string()))?;
    let pinned_at = chrono::Utc::now().to_rfc3339();
    let pinned_by = auth.user_id.to_string();

    state.message_service
        .pin_message(message_id, &params.channel_id.to_string(), &pinned_by, &pinned_at)
        .await
        .map_err(|_| AppError::Internal("Failed to pin message".to_string()))?;

    let room_id = params.channel_id.to_string();
    state.hub.broadcast_room(&room_id, ServerEvent::MessagePinned {
        message_id: params.message_id.clone(),
        channel_id: room_id.clone(),
        pinned_by: pinned_by.clone(),
        pinned_at: pinned_at.clone(),
    }).await;

    Ok(Json(serde_json::json!({ "pinned": true })))
}

async fn unpin_message_handler(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<MessagePath>,
) -> AppResult<Json<serde_json::Value>> {
    let role = MembershipRepository::get_role(&state.db, auth.user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member".to_string()))?;
    if !role.can_delete_others_messages() {
        return Err(AppError::Forbidden("Insufficient permissions to unpin messages".to_string()));
    }

    let message_id = ObjectId::parse_str(&params.message_id)
        .map_err(|_| AppError::BadRequest("Invalid message id".to_string()))?;
    let unpinned_at = chrono::Utc::now().to_rfc3339();

    state.message_service
        .unpin_message(message_id, &params.channel_id.to_string())
        .await
        .map_err(|_| AppError::Internal("Failed to unpin message".to_string()))?;

    let room_id = params.channel_id.to_string();
    state.hub.broadcast_room(&room_id, ServerEvent::MessageUnpinned {
        message_id: params.message_id.clone(),
        channel_id: room_id.clone(),
        unpinned_by: auth.user_id.to_string(),
        unpinned_at,
    }).await;

    Ok(Json(serde_json::json!({ "unpinned": true })))
}

#[derive(Serialize)]
struct UploadResponse {
    url: String,
}

async fn upload_attachment(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ChannelPath>,
    mut multipart: Multipart,
) -> AppResult<Json<UploadResponse>> {
    if !MembershipRepository::is_member(&state.db, auth.user_id, params.server_id).await? {
        return Err(AppError::Forbidden("Not a member of this server".to_string()));
    }

    let upload_dir = state.config.upload_dir.clone();
    tokio::fs::create_dir_all(&upload_dir)
        .await
        .map_err(|e| AppError::Internal(format!("Cannot create upload dir: {e}")))?;

    let mut file_url: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart: {e}")))?
    {
        if field.name().unwrap_or("") != "file" {
            continue;
        }

        let file_name = field.file_name().unwrap_or("file").to_string();
        let content_type = field.content_type().unwrap_or("").to_string();

        let ext = match content_type.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            "application/pdf" => "pdf",
            _ => {
                let lower = file_name.to_lowercase();
                if lower.ends_with(".jpg") || lower.ends_with(".jpeg") { "jpg" }
                else if lower.ends_with(".png") { "png" }
                else if lower.ends_with(".gif") { "gif" }
                else if lower.ends_with(".webp") { "webp" }
                else if lower.ends_with(".pdf") { "pdf" }
                else {
                    return Err(AppError::Validation("Unsupported file type. Allowed: jpg, png, gif, webp, pdf".to_string()));
                }
            }
        };

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read file: {e}")))?;

        if data.len() > 10 * 1024 * 1024 {
            return Err(AppError::Validation("File must be under 10 MB".to_string()));
        }

        let filename = format!(
            "attach_{}_{}_{}.{}",
            auth.user_id,
            params.channel_id,
            chrono::Utc::now().timestamp_millis(),
            ext
        );
        let filepath = format!("{}/{}", upload_dir, filename);

        tokio::fs::write(&filepath, &data)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to save file: {e}")))?;

        file_url = Some(format!("/uploads/{}", filename));
    }

    let url = file_url
        .ok_or_else(|| AppError::BadRequest("No file field in multipart form".to_string()))?;

    Ok(Json(UploadResponse { url }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use crate::auth::test_require_auth;
    use crate::models::ChannelKind;
    use crate::repositories::{ServerRepository, UserRepository};
    use crate::test_utils::{delete_server, delete_user, test_config, try_test_pool};

    #[test]
    fn normalize_search_query_rejects_empty_or_too_long() {
        assert!(normalize_search_query("").is_err());
        assert!(normalize_search_query("   ").is_err());

        let too_long = "a".repeat(MAX_SEARCH_QUERY_LEN + 1);
        assert!(normalize_search_query(&too_long).is_err());
    }

    #[test]
    fn normalize_search_query_trims_whitespace() {
        let q = normalize_search_query("  hello world  ").unwrap();
        assert_eq!(q, "hello world");
    }

    #[tokio::test]
    async fn channel_list_create_update_delete_flow() {
        let Some(pool) = try_test_pool().await else { return; };
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://epitalk:Epitalk94!@localhost:5432/epitalk".to_string());
        let state = Arc::new(AppState::new(pool.clone(), test_config(&db_url)));

        let owner = UserRepository::create(
            &pool,
            &format!("chan-owner-{}@example.test", Uuid::new_v4()),
            "hash",
            &format!("chan_owner_{}", Uuid::new_v4().to_string().replace('-', "")),
        )
        .await
        .expect("create owner");

        let auth = test_require_auth(owner.id, &owner.email, &owner.username);
        let server = ServerRepository::create(&pool, "Channels", owner.id)
            .await
            .expect("create server");

        let listed = list_channels(State(state.clone()), auth.clone(), Path(ServerPath { server_id: server.id }))
            .await
            .expect("list channels")
            .0;
        assert!(!listed.is_empty());

        let created = create_channel(
            State(state.clone()),
            auth.clone(),
            Path(ServerPath { server_id: server.id }),
            Json(CreateChannelRequest { name: "alpha".to_string(), kind: ChannelKind::Text }),
        )
        .await
        .expect("create channel")
        .0;

        let updated = update_channel(
            State(state.clone()),
            auth.clone(),
            Path(ChannelPath { server_id: server.id, channel_id: created.id }),
            Json(UpdateChannelRequest { name: "beta".to_string() }),
        )
        .await
        .expect("update channel")
        .0;
        assert_eq!(updated.name, "beta");

        let messages = get_messages(
            State(state.clone()),
            auth.clone(),
            Path(ChannelPath { server_id: server.id, channel_id: created.id }),
            Query(MessagesQuery { before: None, per_page: 50 }),
        )
        .await
        .expect("get messages")
        .0;
        assert!(messages.is_empty());

        let _ = delete_channel(
            State(state.clone()),
            auth.clone(),
            Path(ChannelPath { server_id: server.id, channel_id: created.id }),
        )
        .await
        .expect("delete channel");

        delete_server(&pool, server.id).await;
        delete_user(&pool, owner.id).await;
    }
}
