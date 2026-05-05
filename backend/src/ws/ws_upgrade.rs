use axum::{
    extract::{ws::WebSocketUpgrade, Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::jwt::validate_token;
use crate::state::AppState;
use crate::ws::connection::handle_connection;

#[derive(Deserialize)]
pub struct WsQuery {
    token: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 1. Validate JWT
    let claims = match validate_token(&params.token, &state.config.jwt_secret) {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Unauthorized",
                    "message": "Invalid or expired WebSocket token"
                })),
            )
                .into_response();
        },
    };

    let user_id = claims.sub;
    let hub = state.hub.clone();
    let message_service = state.message_service.clone();
    let presence = state.presence.clone();
    let typing_service = state.typing_service.clone();
    let pg_pool = state.db.clone();

    ws.on_upgrade(move |socket| async move {
        let conn_id = Uuid::new_v4();
        let user_id_str = user_id.to_string();

        handle_connection(
            socket,
            hub,
            message_service,
            presence,
            typing_service,
            pg_pool,
            user_id_str,
            conn_id,
        )
        .await;
    })
}
