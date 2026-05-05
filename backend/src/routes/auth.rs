//! Authentication routes
//!
//! Handles user registration, login, token refresh, profile retrieval & update.

use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::{
    auth::{PasswordService, RequireAuth, RequireAuthAllowExpired},
    error::{AppError, AppResult},
    models::user::UserStatus,
    models::User,
    repositories::UserRepository,
    state::AppState,
};

/// Build auth routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(me).patch(update_profile))
        .route(
            "/me/avatar",
            post(upload_avatar).layer(DefaultBodyLimit::max(10 * 1024 * 1024)),
        )
        .route("/me/avatar-url", post(set_avatar_from_url))
        .route("/me/email", post(change_email))
        .route("/me/password", post(change_password))
        .route("/refresh", post(refresh_token))
}

// ============================================================================
// Request/Response DTOs
// ============================================================================

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 3, max = 32, message = "Username must be 3-32 characters"))]
    pub username: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub banner_color_1: Option<String>,
    pub banner_color_2: Option<String>,
    pub status: UserStatus,
    pub theme: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            avatar_url: user.avatar_url,
            bio: user.bio,
            banner_color_1: user.banner_color_1,
            banner_color_2: user.banner_color_2,
            status: user.status,
            theme: user.theme,
            created_at: user.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn register_request_validation_rules() {
        // valide
        let ok = RegisterRequest {
            email: "user@example.com".into(),
            username: "user123".into(),
            password: "strongpass123".into(),
        };
        assert!(ok.validate().is_ok());

        // email invalide
        let bad_email = RegisterRequest {
            email: "not-an-email".into(),
            username: "user123".into(),
            password: "strongpass123".into(),
        };
        assert!(bad_email.validate().is_err());

        // username trop court
        let short_username = RegisterRequest {
            email: "user@example.com".into(),
            username: "ab".into(),
            password: "strongpass123".into(),
        };
        assert!(short_username.validate().is_err());

        // mot de passe trop court
        let short_password = RegisterRequest {
            email: "user@example.com".into(),
            username: "user123".into(),
            password: "short".into(),
        };
        assert!(short_password.validate().is_err());
    }

    #[test]
    fn login_request_validation_rules() {
        let ok = LoginRequest {
            email: "user@example.com".into(),
            password: "pass".into(),
        };
        assert!(ok.validate().is_ok());

        let bad_email = LoginRequest {
            email: "not-an-email".into(),
            password: "pass".into(),
        };
        assert!(bad_email.validate().is_err());
    }

    #[test]
    fn user_to_user_response_mapping() {
        let user = User {
            id: Uuid::new_v4(),
            email: "user@example.com".into(),
            password_hash: "hash".into(),
            username: "user123".into(),
            avatar_url: None,
            bio: Some("Hello".into()),
            banner_color_1: Some("#023BFC".into()),
            banner_color_2: Some("#3D6AFF".into()),
            status: UserStatus::Online,
            theme: "light".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let resp: UserResponse = user.clone().into();
        assert_eq!(resp.id, user.id);
        assert_eq!(resp.email, user.email);
        assert_eq!(resp.username, user.username);
        assert_eq!(resp.created_at, user.created_at);
        assert_eq!(resp.status, UserStatus::Online);
    }

    #[test]
    fn is_valid_hex_color_accepts_valid() {
        assert!(is_valid_hex_color("#023BFC"));
        assert!(is_valid_hex_color("#000000"));
        assert!(is_valid_hex_color("#FFFFFF"));
    }

    #[test]
    fn is_valid_hex_color_rejects_invalid() {
        assert!(!is_valid_hex_color("023BFC"));   // missing #
        assert!(!is_valid_hex_color("#GGGGGG"));  // invalid hex chars
        assert!(!is_valid_hex_color("#FFF"));      // too short
        assert!(!is_valid_hex_color("#FFFFFFFF")); // too long
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// Register a new user
///
/// POST /api/auth/register
#[axum::debug_handler]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    // Validate input
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Check if email already exists
    if UserRepository::find_by_email(&state.db, &payload.email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    // Check if username already exists
    if UserRepository::find_by_username(&state.db, &payload.username)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Username already taken".to_string()));
    }

    // Hash password
    let password_service = PasswordService::new();
    let password_hash = password_service.hash_password(&payload.password)?;

    // Create user
    let user = UserRepository::create(&state.db, &payload.email, &password_hash, &payload.username)
        .await?;

    // Generate token
    let token = state
        .jwt_service
        .generate_token(user.id, &user.email, &user.username)?;

    let response = AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiration_hours * 3600,
        user: user.into(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Login with email and password
///
/// POST /api/auth/login
#[axum::debug_handler]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    // Validate input
    payload.validate().map_err(|e| AppError::Validation(e.to_string()))?;

    // Find user by email
    let user = UserRepository::find_by_email(&state.db, &payload.email)
        .await?
        .ok_or(AppError::Unauthorized("Invalid email or password".to_string()))?;

    // Verify password
    let password_service = PasswordService::new();
    let valid = password_service.verify_password(&payload.password, &user.password_hash)?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid email or password".to_string()));
    }

    // Generate token
    let token = state
        .jwt_service
        .generate_token(user.id, &user.email, &user.username)?;

    let response = AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiration_hours * 3600,
        user: user.into(),
    };

    Ok(Json(response))
}

/// Get current user profile
///
/// GET /api/auth/me
#[axum::debug_handler]
pub async fn me(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
) -> AppResult<Json<UserResponse>> {
    let user = UserRepository::find_by_id(&state.db, auth.user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user.into()))
}

/// Refresh JWT token
///
/// POST /api/auth/refresh
#[axum::debug_handler]
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    auth: RequireAuthAllowExpired,
) -> AppResult<Json<AuthResponse>> {
    // Get fresh user data
    let user = UserRepository::find_by_id(&state.db, auth.user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    // Generate new token
    let token = state
        .jwt_service
        .generate_token(user.id, &user.email, &user.username)?;

    let response = AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt_expiration_hours * 3600,
        user: user.into(),
    };

    Ok(Json(response))
}

// ============================================================================
// Profile update
// ============================================================================

/// Delete the old avatar file from disk if it exists
async fn delete_old_avatar(upload_dir: &str, avatar_url: Option<&str>) {
    if let Some(url) = avatar_url {
        // avatar_url is like "/uploads/avatar_xxx.jpg"
        if let Some(filename) = url.strip_prefix("/uploads/") {
            let filepath = format!("{}/{}", upload_dir, filename);
            let _ = tokio::fs::remove_file(&filepath).await;
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub bio: Option<String>,
    pub banner_color_1: Option<String>,
    pub banner_color_2: Option<String>,
    pub status: Option<UserStatus>,
    pub theme: Option<String>,
}

/// Validate a hex color string like #RRGGBB
fn is_valid_hex_color(s: &str) -> bool {
    if s.len() != 7 {
        return false;
    }
    let bytes = s.as_bytes();
    if bytes[0] != b'#' {
        return false;
    }
    bytes[1..].iter().all(|b| b.is_ascii_hexdigit())
}

/// Update current user profile
///
/// PATCH /api/auth/me
#[axum::debug_handler]
pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Json(payload): Json<UpdateProfileRequest>,
) -> AppResult<Json<UserResponse>> {
    // Validate username length if provided
    if let Some(ref name) = payload.username {
        let trimmed = name.trim();
        if trimmed.len() < 3 || trimmed.len() > 32 {
            return Err(AppError::Validation(
                "Username must be 3-32 characters".to_string(),
            ));
        }
    }

    // Validate bio length
    if let Some(ref bio) = payload.bio {
        if bio.len() > 500 {
            return Err(AppError::Validation(
                "Bio must be at most 500 characters".to_string(),
            ));
        }
    }

    // Validate banner colors
    if let Some(ref c) = payload.banner_color_1 {
        if !is_valid_hex_color(c) {
            return Err(AppError::Validation("Invalid banner_color_1 hex color".to_string()));
        }
    }
    if let Some(ref c) = payload.banner_color_2 {
        if !is_valid_hex_color(c) {
            return Err(AppError::Validation("Invalid banner_color_2 hex color".to_string()));
        }
    }

    // Validate theme
    if let Some(ref t) = payload.theme {
        if !["light", "ash", "dim", "dark"].contains(&t.as_str()) {
            return Err(AppError::Validation(
                "Theme must be one of: light, ash, dim, dark".to_string(),
            ));
        }
    }

    let user = UserRepository::update_profile(
        &state.db,
        auth.user_id,
        payload.username.as_deref(),
        payload.bio.as_deref(),
        None, // avatar_url updated via upload endpoint
        payload.banner_color_1.as_deref(),
        payload.banner_color_2.as_deref(),
        payload.status.as_ref(),
        payload.theme.as_deref(),
    )
    .await?;

    Ok(Json(user.into()))
}

/// Upload avatar image (JPEG, PNG, GIF)
///
/// POST /api/auth/me/avatar
#[axum::debug_handler]
pub async fn upload_avatar(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    mut multipart: Multipart,
) -> AppResult<Json<UserResponse>> {
    let upload_dir = state.config.upload_dir.clone();

    // Get current user to know old avatar
    let current_user = UserRepository::find_by_id(&state.db, auth.user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    // Create uploads directory
    tokio::fs::create_dir_all(&upload_dir)
        .await
        .map_err(|e| AppError::Internal(format!("Cannot create upload dir: {e}")))?;

    let mut avatar_path: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart: {e}")))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name != "avatar" {
            continue;
        }

        let content_type = field.content_type().unwrap_or("").to_string();
        let file_name = field.file_name().unwrap_or("").to_string();
        let ext = match content_type.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            _ => {
                // Fallback: detect from filename extension
                let lower = file_name.to_lowercase();
                if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
                    "jpg"
                } else if lower.ends_with(".png") {
                    "png"
                } else if lower.ends_with(".gif") {
                    "gif"
                } else if lower.ends_with(".webp") {
                    "webp"
                } else {
                    return Err(AppError::Validation(
                        "Only JPEG, PNG, GIF and WebP images are allowed".to_string(),
                    ));
                }
            }
        };

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read file: {e}")))?;

        // Limit to 10 MB
        if data.len() > 10 * 1024 * 1024 {
            return Err(AppError::Validation(
                "Avatar file must be under 10 MB".to_string(),
            ));
        }

        let filename = format!("avatar_{}_{}.{}", auth.user_id, chrono::Utc::now().timestamp(), ext);
        let filepath = format!("{}/{}", upload_dir, filename);

        tokio::fs::write(&filepath, &data)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to save file: {e}")))?;

        avatar_path = Some(format!("/uploads/{}", filename));
    }

    let avatar_url = avatar_path.ok_or_else(|| {
        AppError::BadRequest("No avatar field in multipart form".to_string())
    })?;

    // Delete old avatar file
    delete_old_avatar(&upload_dir, current_user.avatar_url.as_deref()).await;

    let user = UserRepository::update_profile(
        &state.db,
        auth.user_id,
        None,
        None,
        Some(&avatar_url),
        None,
        None,
        None,
        None,
    )
    .await?;

    Ok(Json(user.into()))
}

/// Set avatar from an external GIF URL (download and save locally)
///
/// POST /api/auth/me/avatar-url
#[derive(Debug, Deserialize)]
pub struct AvatarUrlRequest {
    pub url: String,
}

#[axum::debug_handler]
pub async fn set_avatar_from_url(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Json(body): Json<AvatarUrlRequest>,
) -> AppResult<Json<UserResponse>> {
    // Get current user to know old avatar
    let current_user = UserRepository::find_by_id(&state.db, auth.user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    // Validate URL
    let parsed = reqwest::Url::parse(&body.url)
        .map_err(|_| AppError::Validation("Invalid URL".to_string()))?;

    // Only allow HTTPS URLs from known GIF providers
    let host = parsed.host_str().unwrap_or("");
    let allowed_hosts = [
        "media.tenor.com",
        "media1.tenor.com",
        "c.tenor.com",
        "media.giphy.com",
        "i.giphy.com",
        "media0.giphy.com",
        "media1.giphy.com",
        "media2.giphy.com",
        "media3.giphy.com",
        "media4.giphy.com",
    ];
    if !allowed_hosts.iter().any(|h| host == *h) {
        return Err(AppError::Validation(
            "URL must be from Tenor or Giphy".to_string(),
        ));
    }
    if parsed.scheme() != "https" {
        return Err(AppError::Validation("URL must use HTTPS".to_string()));
    }

    // Download the GIF
    let client = reqwest::Client::new();
    let resp = client
        .get(parsed.clone())
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to download GIF: {e}")))?;

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let data = resp
        .bytes()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to read GIF data: {e}")))?;

    if data.len() > 10 * 1024 * 1024 {
        return Err(AppError::Validation(
            "GIF must be under 10 MB".to_string(),
        ));
    }

    // Determine extension from content-type or URL path
    let ext = if content_type.contains("gif") {
        "gif"
    } else if parsed.path().to_lowercase().contains(".gif") {
        "gif"
    } else if content_type.contains("webp") {
        "webp"
    } else {
        "gif" // Default to gif for Tenor/Giphy URLs
    };

    let upload_dir = state.config.upload_dir.clone();
    tokio::fs::create_dir_all(&upload_dir)
        .await
        .map_err(|e| AppError::Internal(format!("Cannot create upload dir: {e}")))?;

    let filename = format!(
        "avatar_{}_{}.{}",
        auth.user_id,
        chrono::Utc::now().timestamp(),
        ext
    );
    let filepath = format!("{}/{}", upload_dir, filename);

    tokio::fs::write(&filepath, &data)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to save file: {e}")))?;

    let avatar_url = format!("/uploads/{}", filename);

    // Delete old avatar file
    delete_old_avatar(&upload_dir, current_user.avatar_url.as_deref()).await;

    let user = UserRepository::update_profile(
        &state.db,
        auth.user_id,
        None,
        None,
        Some(&avatar_url),
        None,
        None,
        None,
        None,
    )
    .await?;

    Ok(Json(user.into()))
}

/// Change email address
///
/// POST /api/auth/me/email
#[derive(Debug, Deserialize, Validate)]
pub struct ChangeEmailRequest {
    #[validate(email(message = "Invalid email format"))]
    pub new_email: String,
    pub password: String,
}

#[axum::debug_handler]
pub async fn change_email(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Json(payload): Json<ChangeEmailRequest>,
) -> AppResult<Json<UserResponse>> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // Fetch current user to verify password
    let current_user = UserRepository::find_by_id(&state.db, auth.user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    let password_service = PasswordService::new();
    if !password_service.verify_password(&payload.password, &current_user.password_hash)? {
        return Err(AppError::Unauthorized(
            "Mot de passe incorrect".to_string(),
        ));
    }

    let user = UserRepository::update_email(&state.db, auth.user_id, &payload.new_email).await?;

    Ok(Json(user.into()))
}

/// Change password
///
/// POST /api/auth/me/password
#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[axum::debug_handler]
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Json(payload): Json<ChangePasswordRequest>,
) -> AppResult<Json<UserResponse>> {
    if payload.new_password.len() < 8 {
        return Err(AppError::Validation(
            "Le nouveau mot de passe doit faire au moins 8 caractères".to_string(),
        ));
    }

    // Fetch current user to verify old password
    let current_user = UserRepository::find_by_id(&state.db, auth.user_id)
        .await?
        .ok_or(AppError::NotFound("User not found".to_string()))?;

    let password_service = PasswordService::new();
    if !password_service.verify_password(&payload.current_password, &current_user.password_hash)? {
        return Err(AppError::Unauthorized(
            "Mot de passe actuel incorrect".to_string(),
        ));
    }

    let new_hash = password_service.hash_password(&payload.new_password)?;
    let user = UserRepository::update_password(&state.db, auth.user_id, &new_hash).await?;

    Ok(Json(user.into()))
}
