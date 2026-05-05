use axum::{extract::Path, extract::State, routing::patch, Json, Router};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::RequireAuth;
use crate::error::AppResult;
use crate::models::UserResponse;
use crate::repositories::UserRepository;
use crate::state::AppState;
use crate::ws::protocol::ServerEvent;

#[derive(Debug, Deserialize)]
struct StatusPayload {
    status: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/:id/status", patch(patch_status))
}

async fn patch_status(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(id): Path<Uuid>,
    Json(payload): Json<StatusPayload>,
) -> AppResult<Json<UserResponse>> {
    // Only allow users to change their own status (or admin later)
    if auth.user_id != id {
        return Err(crate::error::AppError::Forbidden(
            "Cannot change other user's status".to_string(),
        ));
    }

    // Validate status value
    let allowed = ["online", "dnd", "offline"];
    if !allowed.contains(&payload.status.as_str()) {
        return Err(crate::error::AppError::BadRequest(
            "Invalid status value".to_string(),
        ));
    }

    // Update in DB
    let user = UserRepository::set_status(&state.db, id, &payload.status).await?;

    // Broadcast via WebSocket to all clients
    let event = ServerEvent::UserStatusChanged {
        user_id: id.to_string(),
        status: payload.status.clone(),
    };

    // Fire and forget the broadcast
    let hub = state.hub.clone();
    tokio::spawn(async move {
        let _ = hub.broadcast_all(event).await;
    });

    Ok(Json(UserResponse::from(user)))
}
