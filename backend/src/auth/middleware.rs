//! Authentication middleware and extractors
//!
//! Provides Axum extractors for JWT authentication and RBAC enforcement.

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    auth::Claims,
    error::AppError,
    models::MemberRole,
    repositories::MembershipRepository,
    state::AppState,
};

/// Authenticated user extracted from JWT token
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields will be used when implementing user profile features
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub claims: Claims,
}

impl AuthUser {
    /// Get the user ID
    #[allow(dead_code)] // Will be used in future routes
    pub fn id(&self) -> Uuid {
        self.user_id
    }
}

/// Extractor that requires a valid JWT token
#[derive(Debug, Clone)]
pub struct RequireAuth(pub AuthUser);

impl std::ops::Deref for RequireAuth {
    type Target = AuthUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Extractor that requires a valid JWT signature, but allows expired tokens.
///
/// Intended only for token refresh flows.
#[derive(Debug, Clone)]
pub struct RequireAuthAllowExpired(pub AuthUser);

impl std::ops::Deref for RequireAuthAllowExpired {
    type Target = AuthUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireAuth
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                AppError::Unauthorized("Missing Authorization header".to_string()).into_response()
            })?;

        // Extract Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                AppError::Unauthorized("Invalid Authorization header format. Use: Bearer <token>".to_string())
                    .into_response()
            })?;

        // Validate token
        let token_data = app_state
            .jwt_service
            .validate_token(token)
            .map_err(|e| e.into_response())?;

        let claims = token_data.claims;

        Ok(RequireAuth(AuthUser {
            user_id: claims.sub,
            email: claims.email.clone(),
            username: claims.username.clone(),
            claims,
        }))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireAuthAllowExpired
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| {
                AppError::Unauthorized("Missing Authorization header".to_string()).into_response()
            })?;

        // Extract Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                AppError::Unauthorized("Invalid Authorization header format. Use: Bearer <token>".to_string())
                    .into_response()
            })?;

        // Validate token signature, but allow expiration for refresh flow
        let token_data = app_state
            .jwt_service
            .validate_token_allow_expired(token)
            .map_err(|e| e.into_response())?;

        let claims = token_data.claims;

        Ok(RequireAuthAllowExpired(AuthUser {
            user_id: claims.sub,
            email: claims.email.clone(),
            username: claims.username.clone(),
            claims,
        }))
    }
}

/// Extractor that requires a specific role in a server
///
/// Use with Path<Uuid> for server_id extraction
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used when implementing advanced RBAC
pub struct RequireRole {
    pub auth_user: AuthUser,
    pub server_id: Uuid,
    pub role: MemberRole,
}

#[allow(dead_code)] // Role checking methods for future RBAC implementation
impl RequireRole {
    /// Check if user has at least the required role
    pub fn has_permission(&self, required: MemberRole) -> bool {
        self.role.is_higher_or_equal(&required)
    }

    /// Check if user is the owner
    pub fn is_owner(&self) -> bool {
        matches!(self.role, MemberRole::Owner)
    }

    /// Check if user is admin or owner
    pub fn is_admin_or_above(&self) -> bool {
        self.role.can_manage_channels()
    }

    /// Check if user is moderator or above (Owner, Admin, Moderator)
    pub fn is_moderator_or_above(&self) -> bool {
        self.role.can_create_invites()
    }
}

/// Helper function to extract and validate user role in a server
#[allow(dead_code)] // Will be used in future RBAC middleware
pub async fn get_user_role_in_server(
    app_state: &AppState,
    user_id: Uuid,
    server_id: Uuid,
) -> Result<MemberRole, AppError> {
    let membership = MembershipRepository::find_by_user_and_server(&app_state.db, user_id, server_id)
        .await?
        .ok_or(AppError::Forbidden(
            "You are not a member of this server".to_string(),
        ))?;

    Ok(membership.role)
}

/// Macro to simplify role checking in handlers
#[macro_export]
macro_rules! require_role {
    ($state:expr, $user_id:expr, $server_id:expr, $min_role:expr) => {{
        let role = $crate::auth::middleware::get_user_role_in_server($state, $user_id, $server_id).await?;
        if !role.has_permission(&$min_role) {
            return Err($crate::error::AppError::Forbidden(format!(
                "Requires {} role or higher",
                stringify!($min_role)
            )));
        }
        role
    }};
}

/// Helper to get the current user ID from auth - replaces the placeholder function
#[allow(dead_code)] // Alternative to direct field access
pub fn get_current_user_id(auth: &RequireAuth) -> Uuid {
    auth.user_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_permissions() {
        let owner = MemberRole::Owner;
        let admin = MemberRole::Admin;
        let member = MemberRole::Member;

        // Owner has all permissions
        assert!(owner.is_higher_or_equal(&MemberRole::Member));
        assert!(owner.is_higher_or_equal(&MemberRole::Admin));
        assert!(owner.is_higher_or_equal(&MemberRole::Owner));

        // Admin has member and admin permissions
        assert!(admin.is_higher_or_equal(&MemberRole::Member));
        assert!(admin.is_higher_or_equal(&MemberRole::Admin));
        assert!(!admin.is_higher_or_equal(&MemberRole::Owner));

        // Member only has member permission
        assert!(member.is_higher_or_equal(&MemberRole::Member));
        assert!(!member.is_higher_or_equal(&MemberRole::Admin));
        assert!(!member.is_higher_or_equal(&MemberRole::Owner));
    }
}
