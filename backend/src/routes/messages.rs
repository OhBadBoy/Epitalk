//! Message routes
//!
//! Routes:
//! - GET /messages/:message_id - Get a single message by mongo id

use axum::{extract::{Path, State}, routing::get, Json, Router};
use mongodb::bson::oid::ObjectId;
use std::sync::Arc;

use crate::auth::RequireAuth;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::repositories::UserRepository;
use crate::repositories::ChannelRepository;

#[derive(serde::Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub server_id: Option<String>,
    pub channel_id: String,
    pub author_id: String,
    pub username: String,
    pub content: String,
    pub created_at: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/:message_id", get(get_message))
}

async fn get_message(
    State(state): State<Arc<AppState>>,
    _auth: RequireAuth,
    Path(message_id): Path<String>,
) -> AppResult<Json<MessageResponse>> {
    let oid = ObjectId::parse_str(&message_id)
        .map_err(|_| AppError::BadRequest("invalid message id".to_string()))?;

    let msg = state
        .message_service
        .get_message_by_id(&oid)
        .await
        .ok_or_else(|| AppError::NotFound("Message not found".to_string()))?;

    // try to find server id from channel id (channel ids are UUID strings)
    let server_id = match uuid::Uuid::parse_str(&msg.channel_id) {
        Ok(uuid) => ChannelRepository::find_by_id(&state.db, uuid).await.ok().flatten().map(|c| c.server_id.to_string()),
        Err(_) => None,
    };

    // Resolve username if possible
    let username = if let Ok(uuid) = uuid::Uuid::parse_str(&msg.author_id) {
        UserRepository::find_by_id(&state.db, uuid).await.ok().flatten().map(|u| u.username).unwrap_or_else(|| msg.author_id.chars().take(8).collect())
    } else {
        msg.author_id.chars().take(8).collect()
    };

    let resp = MessageResponse {
        id: msg.id.map(|oid| oid.to_hex()).unwrap_or_default(),
        server_id,
        channel_id: msg.channel_id,
        author_id: msg.author_id,
        username,
        content: msg.content,
        created_at: msg.created_at,
    };

    Ok(Json(resp))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use crate::auth::test_require_auth;
    use crate::repositories::UserRepository;
    use crate::test_utils::{delete_user, test_config, try_test_pool};

    #[tokio::test]
    async fn get_message_rejects_invalid_id() {
        let Some(pool) = try_test_pool().await else { return; };
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://epitalk:Epitalk94!@localhost:5432/epitalk".to_string());
        let state = Arc::new(AppState::new(pool.clone(), test_config(&db_url)));

        let user = UserRepository::create(
            &pool,
            &format!("msg-{}@example.test", uuid::Uuid::new_v4()),
            "hash",
            &format!("msg_{}", uuid::Uuid::new_v4().to_string().replace('-', "")),
        )
        .await
        .expect("create user");

        let auth = test_require_auth(user.id, &user.email, &user.username);
        let result = get_message(State(state.clone()), auth, Path("bad-id".to_string())).await;
        assert!(matches!(result, Err(AppError::BadRequest(_))));

        delete_user(&pool, user.id).await;
    }
}
