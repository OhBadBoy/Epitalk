//! Application error types

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
            }
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body;
    use futures_util::FutureExt;

    fn extract_status_and_message(err: AppError) -> (StatusCode, String) {
        let response = err.into_response();
        let status = response.status();
        let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
            .now_or_never()
            .unwrap()
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        (status, v["error"].as_str().unwrap().to_string())
    }

    #[test]
    fn variants_map_to_expected_status_codes() {
        use AppError::*;

        let (status, msg) = extract_status_and_message(NotFound("x".into()));
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(msg, "x");

        let (status, msg) = extract_status_and_message(BadRequest("bad".into()));
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(msg, "bad");

        let (status, msg) = extract_status_and_message(Unauthorized("no".into()));
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(msg, "no");

        let (status, msg) = extract_status_and_message(Forbidden("stop".into()));
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(msg, "stop");

        let (status, msg) = extract_status_and_message(Conflict("c".into()));
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(msg, "c");

        let (status, msg) = extract_status_and_message(Validation("v".into()));
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(msg, "v");

        let (status, msg) = extract_status_and_message(Internal("oops".into()));
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(msg, "Internal error");
    }
}
