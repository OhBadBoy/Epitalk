//! Health check route

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use crate::state::AppState;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(health_check))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "epitalk-backend",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_check_returns_expected_payload() {
        let Json(body) = health_check().await;

        assert_eq!(body["status"], "ok");
        assert_eq!(body["service"], "epitalk-backend");
        assert!(body["version"].is_string());
    }
}
