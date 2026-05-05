//! Direct message routes
//!
//! Routes:
//! - GET /dm/conversations - List DM conversations for the current user
//! - GET /dm/conversations/:peer_id/messages - Get DM message history (cursor-based)

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::auth::RequireAuth;
use crate::error::{AppError, AppResult};
use crate::repositories::UserRepository;
use crate::state::AppState;

#[derive(Serialize)]
pub struct DmConversationResponse {
    pub conversation_id: String,
    pub peer_id: String,
    pub peer_username: String,
    pub last_message: String,
    pub last_message_at: String,
}

#[derive(Deserialize)]
pub struct DmMessagesQuery {
    pub before: Option<String>,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_per_page() -> u64 {
    50
}

#[derive(Serialize)]
pub struct DmMessageResponse {
    pub id: String,
    pub conversation_id: String,
    pub author_id: String,
    pub username: String,
    pub content: String,
    pub created_at: String,
    pub reply_to: Option<String>,
    pub attachment_url: Option<String>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/conversations", get(list_conversations))
        .route("/conversations/:peer_id/messages", get(get_dm_messages))
        .route("/upload", post(upload_dm_attachment).layer(DefaultBodyLimit::max(10 * 1024 * 1024)))
}

/// List all DM conversations for the authenticated user
async fn list_conversations(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
) -> AppResult<Json<Vec<DmConversationResponse>>> {
    let user_id = auth.user_id.to_string();

    let raw_conversations = state.message_service.get_dm_conversations(&user_id).await;

    let mut results = Vec::new();

    for doc in raw_conversations {
        let conversation_id = doc.get_str("_id").unwrap_or_default().to_string();
        let last_message = doc.get_str("last_message").unwrap_or_default().to_string();
        let last_message_at = doc.get_str("last_message_at").unwrap_or_default().to_string();

        // Extract peer_id from conversation_id "dm:{uuid1}:{uuid2}"
        let peer_id = extract_peer_id(&conversation_id, &user_id)
            .unwrap_or_default();

        // Resolve peer username from Postgres
        let peer_username = if let Ok(uuid) = uuid::Uuid::parse_str(&peer_id) {
            match UserRepository::find_by_id(&state.db, uuid).await {
                Ok(Some(u)) => u.username,
                _ => peer_id.chars().take(8).collect(),
            }
        } else {
            peer_id.chars().take(8).collect()
        };

        results.push(DmConversationResponse {
            conversation_id,
            peer_id,
            peer_username,
            last_message,
            last_message_at,
        });
    }

    Ok(Json(results))
}

/// Get DM message history for a conversation with a specific peer
async fn get_dm_messages(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(peer_id): Path<String>,
    Query(query): Query<DmMessagesQuery>,
) -> AppResult<Json<Vec<DmMessageResponse>>> {
    let my_id = auth.user_id.to_string();
    let conversation_id = dm_conversation_id(&my_id, &peer_id);

    let messages = if let Some(ref before_hex) = query.before {
        let before_oid = ObjectId::parse_str(before_hex)
            .map_err(|_| AppError::BadRequest("invalid before cursor".to_string()))?;
        state
            .message_service
            .get_history_before(&conversation_id, &before_oid, query.per_page)
            .await
            .map_err(|_| AppError::Internal("Failed to fetch DM messages".to_string()))?
    } else {
        state
            .message_service
            .get_history(&conversation_id, 1, query.per_page)
            .await
            .map_err(|_| AppError::Internal("Failed to fetch DM messages".to_string()))?
    };

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
        result.push(DmMessageResponse {
            id: m.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            conversation_id: conversation_id.clone(),
            author_id: m.author_id,
            username,
            content: m.content,
            created_at: m.created_at,
            reply_to: m.reply_to.map(|oid| oid.to_hex()),
            attachment_url: m.attachment_url,
        });
    }

    Ok(Json(result))
}

#[derive(Serialize)]
struct UploadResponse {
    url: String,
}

async fn upload_dm_attachment(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    mut multipart: Multipart,
) -> AppResult<Json<UploadResponse>> {
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
            "dm_attach_{}_{}.{}",
            auth.user_id,
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

fn dm_conversation_id(a: &str, b: &str) -> String {
    if a < b {
        format!("dm:{}:{}", a, b)
    } else {
        format!("dm:{}:{}", b, a)
    }
}

/// Extract peer user_id from a conversation_id "dm:{uuid1}:{uuid2}"
fn extract_peer_id(conversation_id: &str, my_id: &str) -> Option<String> {
    let stripped = conversation_id.strip_prefix("dm:")?;
    let parts: Vec<&str> = stripped.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }
    if parts[0] == my_id {
        Some(parts[1].to_string())
    } else {
        Some(parts[0].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_peer_id_handles_both_orders() {
        let my_id = "user-a";
        let other = "user-b";

        let conv1 = format!("dm:{}:{}", my_id, other);
        let conv2 = format!("dm:{}:{}", other, my_id);

        assert_eq!(extract_peer_id(&conv1, my_id).as_deref(), Some(other));
        assert_eq!(extract_peer_id(&conv2, my_id).as_deref(), Some(other));
    }

    #[test]
    fn extract_peer_id_rejects_invalid_format() {
        assert!(extract_peer_id("", "user").is_none());
        assert!(extract_peer_id("dm:onlyone", "user").is_none());
        assert!(extract_peer_id("badprefix:user:a", "user").is_none());
    }

    #[test]
    fn dm_conversation_id_is_deterministic() {
        let a = "aaa";
        let b = "bbb";
        assert_eq!(dm_conversation_id(a, b), dm_conversation_id(b, a));
        assert!(dm_conversation_id(a, b).starts_with("dm:"));
    }
}
