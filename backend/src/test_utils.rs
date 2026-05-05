use std::sync::Arc;

use axum::Router;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::models::{Channel, ChannelKind, Server, User};
use crate::repositories::{ChannelRepository, ServerRepository, UserRepository};
use crate::routes;
use crate::state::AppState;

pub async fn test_pool() -> PgPool {
    try_test_pool()
        .await
        .expect("DATABASE_URL must be reachable for tests")
}

pub async fn try_test_pool() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://epitalk:Epitalk94!@localhost:5432/epitalk".to_string());

    match PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&url)
        .await
    {
        Ok(pool) => Some(pool),
        Err(err) => {
            eprintln!("Skipping DB-backed tests: {}", err);
            None
        }
    }
}

pub fn test_config(database_url: &str) -> Config {
    Config {
        database_url: database_url.to_string(),
        jwt_secret: "test-secret-key-at-least-32-chars".to_string(),
        jwt_expiration_hours: 24,
        port: 0,
        upload_dir: "./uploads".to_string(),
    }
}

pub async fn test_state(pool: PgPool) -> Arc<AppState> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://epitalk:Epitalk94!@localhost:5432/epitalk".to_string());
    let cfg = test_config(&db_url);
    Arc::new(AppState::new(pool, cfg))
}

pub async fn test_app(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .nest("/api", routes::api_router())
        .with_state(state)
}

pub async fn create_user(pool: &PgPool, label: &str) -> User {
    let suffix = Uuid::new_v4();
    let email = format!("{}-{}@example.test", label, suffix);
    let username = format!("{}_{}", label, suffix.to_string().replace('-', ""));
    UserRepository::create(pool, &email, "hash", &username)
        .await
        .expect("create user")
}

pub async fn create_server(pool: &PgPool, owner_id: Uuid, name: &str) -> Server {
    ServerRepository::create(pool, name, owner_id)
        .await
        .expect("create server")
}

#[allow(dead_code)]
pub async fn create_channel(pool: &PgPool, server_id: Uuid, name: &str) -> Channel {
    ChannelRepository::create(pool, server_id, name, ChannelKind::Text)
        .await
        .expect("create channel")
}

pub async fn delete_server(pool: &PgPool, server_id: Uuid) {
    let _ = sqlx::query("DELETE FROM servers WHERE id = $1")
        .bind(server_id)
        .execute(pool)
        .await;
}

pub async fn delete_user(pool: &PgPool, user_id: Uuid) {
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
}

