//! API Routes

pub mod auth;
pub mod channels;
pub mod dm;
pub mod health;
pub mod invites;
pub mod members;
pub mod servers;
pub mod gifs;
pub mod messages;

use axum::Router;
use crate::state::AppState;
use std::sync::Arc;

/// Build the main API router
pub fn api_router() -> Router<Arc<AppState>> {
    Router::new()
        .merge(health::router())
        .nest("/auth", auth::routes())
        .nest("/gifs", gifs::router())
        .nest("/servers", servers::router())
        .nest("/messages", messages::router())
        .nest("/join", invites::join_router())
        .nest("/dm", dm::router())
}
