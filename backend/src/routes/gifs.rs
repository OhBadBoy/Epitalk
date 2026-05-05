use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json, Router,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest::Url;
use std::sync::Arc;

use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/search", get(search_gifs))
}

#[derive(Debug, Deserialize)]
pub struct GifQuery {
    pub q: String,
    pub limit: Option<u8>,
    pub provider: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GifResult {
    pub id: String,
    pub url: String,
    pub preview: Option<String>,
    pub provider: String,
}

#[derive(Debug, Serialize)]
pub struct GifResponse {
    pub results: Vec<GifResult>,
}

#[axum::debug_handler]
async fn search_gifs(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<GifQuery>,
) -> (StatusCode, Json<GifResponse>) {
    let q = params.q;
    let limit = params.limit.unwrap_or(12).min(50);
    let provider = params.provider.unwrap_or_else(|| "auto".to_string());

    tracing::info!("GIF search params: q={}, limit={}, provider={}", q, limit, provider);
    tracing::info!("TENOR_API_KEY present={}, GIPHY_API_KEY present={}", std::env::var("TENOR_API_KEY").is_ok(), std::env::var("GIPHY_API_KEY").is_ok());

    // Decide provider: prefer TENOR if configured, else GIPHY
    let use_tenor = match provider.as_str() {
        "tenor" => true,
        "giphy" => false,
        _ => std::env::var("TENOR_API_KEY").is_ok(),
    };

    // Build request
    let client = reqwest::Client::new();
    // Try Tenor first
    if use_tenor {
        if let Ok(key) = std::env::var("TENOR_API_KEY") {
            let mut url = Url::parse("https://tenor.googleapis.com/v2/search").unwrap();
            url.query_pairs_mut()
                .append_pair("q", &q)
                .append_pair("key", &key)
                .append_pair("limit", &limit.to_string());
            match client.get(url).send().await {
                Ok(resp) => match resp.json::<Value>().await {
                    Ok(val) => {
                        // Normalize Tenor response
                        if let Some(items) = val.get("results").and_then(|v| v.as_array()) {
                            let mut results: Vec<GifResult> = Vec::new();
                            for item in items.iter().take(limit as usize) {
                                let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                // try several paths for a gif url
                                let url = item
                                    .get("media_formats")
                                    .and_then(|m| m.get("gif").or_else(|| m.get("mediumgif")))
                                    .and_then(|g| g.get("url"))
                                    .and_then(|u| u.as_str())
                                    .map(|s| s.to_string())
                                    .or_else(|| item.get("url").and_then(|u| u.as_str()).map(|s| s.to_string()))
                                    .unwrap_or_default();

                                let preview = item
                                    .get("media_formats")
                                    .and_then(|m| m.get("tinygif").or_else(|| m.get("nanogif")))
                                    .and_then(|p| p.get("url"))
                                    .and_then(|u| u.as_str())
                                    .map(|s| s.to_string());

                                if !url.is_empty() {
                                    results.push(GifResult { id, url, preview, provider: "tenor".to_string() });
                                }
                            }
                            return (StatusCode::OK, Json(GifResponse { results }));
                        }
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(GifResponse { results: vec![] }));
                    }
                    Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(GifResponse { results: vec![] })),
                },
                Err(_) => return (StatusCode::BAD_GATEWAY, Json(GifResponse { results: vec![] })),
            }
        }
    }

    // Fallback to Giphy
    if let Ok(key) = std::env::var("GIPHY_API_KEY") {
        let mut url = Url::parse("https://api.giphy.com/v1/gifs/search").unwrap();
        url.query_pairs_mut()
            .append_pair("api_key", &key)
            .append_pair("q", &q)
            .append_pair("limit", &limit.to_string());
        match client.get(url).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(val) => {
                    if let Some(items) = val.get("data").and_then(|v| v.as_array()) {
                        let mut results: Vec<GifResult> = Vec::new();
                        for item in items.iter().take(limit as usize) {
                            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            let url = item
                                .get("images")
                                .and_then(|imgs| imgs.get("original").or_else(|| imgs.get("fixed_width")))
                                .and_then(|o| o.get("url"))
                                .and_then(|u| u.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_default();

                            let preview = item
                                .get("images")
                                .and_then(|imgs| imgs.get("preview_gif").or_else(|| imgs.get("fixed_width_small_still")))
                                .and_then(|p| p.get("url"))
                                .and_then(|u| u.as_str())
                                .map(|s| s.to_string());

                            if !url.is_empty() {
                                results.push(GifResult { id, url, preview, provider: "giphy".to_string() });
                            }
                        }
                        return (StatusCode::OK, Json(GifResponse { results }));
                    }
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(GifResponse { results: vec![] }));
                }
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(GifResponse { results: vec![] })),
            },
            Err(_) => return (StatusCode::BAD_GATEWAY, Json(GifResponse { results: vec![] })),
        }
    }

    tracing::warn!("No TENOR_API_KEY or GIPHY_API_KEY configured — GIF search unavailable");
    (StatusCode::OK, Json(GifResponse { results: vec![] }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn search_gifs_returns_empty_without_keys() {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://epitalk:Epitalk94!@localhost:5432/epitalk")
            .expect("lazy pool");
        let cfg = crate::config::Config {
            database_url: "postgres://epitalk:Epitalk94!@localhost:5432/epitalk".to_string(),
            jwt_secret: "test-secret-key-at-least-32-chars".to_string(),
            jwt_expiration_hours: 24,
            port: 0,
            upload_dir: "./uploads".to_string(),
        };
        let state = crate::state::AppState::new(pool, cfg);

        let (status, Json(payload)) = search_gifs(
            State(std::sync::Arc::new(state)),
            Query(GifQuery {
                q: "cats".to_string(),
                limit: Some(5),
                provider: None,
            }),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert!(payload.results.is_empty());
    }
}
