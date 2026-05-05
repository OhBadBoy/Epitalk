//! Invite routes
//!
//! Routes (nested under /servers/:server_id/invites):
//! - GET    /           - List all server invites (ADMIN+)
//! - GET    /active     - List active server invites only (ADMIN+)
//! - POST   /           - Create an invite (ADMIN+)
//! - DELETE /:invite_id - Delete an invite (ADMIN+)
//!
//! Routes (top-level for joining):
//! - POST /join         - Join a server via invite code

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::auth::RequireAuth;
use crate::error::{AppError, AppResult};
use crate::models::{
    CreateInviteRequest, InviteResponse, JoinServerRequest, MemberRole, ServerResponse,
};
use crate::repositories::{
    InviteRepository, MembershipRepository, ServerRepository,
};
use crate::state::AppState;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_invites).post(create_invite))
        .route("/active", get(list_active_invites))
        .route("/:invite_id", axum::routing::delete(delete_invite))
}

/// Join server router (mounted at /api/join)
pub fn join_router() -> Router<Arc<AppState>> {
    Router::new().route("/", post(join_server))
}

/// Path parameters
#[derive(serde::Deserialize)]
pub struct ServerPath {
    pub server_id: Uuid,
}

#[derive(serde::Deserialize)]
pub struct InvitePath {
    pub server_id: Uuid,
    pub invite_id: String, // invite_id is the code (string), not a UUID
}

/// List all invites for a server (ADMIN+ only)
async fn list_invites(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ServerPath>,
) -> AppResult<Json<Vec<InviteResponse>>> {
    let user_id = auth.user_id;

    // Check role
    let role = MembershipRepository::get_role(&state.db, user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if !role.can_create_invites() {
        return Err(AppError::Forbidden("Insufficient permissions to view invites".to_string()));
    }

    let invites = InviteRepository::find_by_server(&state.db, params.server_id).await?;
    let responses: Vec<InviteResponse> = invites.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// List only ACTIVE invites for a server (not expired, not at max uses) - ADMIN+ only
async fn list_active_invites(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ServerPath>,
) -> AppResult<Json<Vec<InviteResponse>>> {
    let user_id = auth.user_id;

    // Check role
    let role = MembershipRepository::get_role(&state.db, user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if !role.can_create_invites() {
        return Err(AppError::Forbidden("Insufficient permissions to view invites".to_string()));
    }

    let invites = InviteRepository::find_active_by_server(&state.db, params.server_id).await?;
    let responses: Vec<InviteResponse> = invites.into_iter().map(Into::into).collect();

    Ok(Json(responses))
}

/// Create a new invite (ADMIN+ only)
async fn create_invite(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ServerPath>,
    Json(payload): Json<CreateInviteRequest>,
) -> AppResult<Json<InviteResponse>> {
    let user_id = auth.user_id;

    // Check role
    let role = MembershipRepository::get_role(&state.db, user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if !role.can_create_invites() {
        return Err(AppError::Forbidden("Insufficient permissions to create invites".to_string()));
    }

    let invite = InviteRepository::create(
        &state.db,
        params.server_id,
        user_id,
        payload.expires_in_hours,
        payload.max_uses,
    )
    .await?;

    Ok(Json(invite.into()))
}

/// Delete an invite (ADMIN+ only)
async fn delete_invite(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<InvitePath>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = auth.user_id;

    // Check role
    let role = MembershipRepository::get_role(&state.db, user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if !role.can_create_invites() {
        return Err(AppError::Forbidden("Insufficient permissions to delete invites".to_string()));
    }

    // Verify invite belongs to server
    let invite = InviteRepository::find_by_code(&state.db, &params.invite_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Invite not found".to_string()))?;

    if invite.server_id != params.server_id {
        return Err(AppError::NotFound("Invite not found in this server".to_string()));
    }

    InviteRepository::delete(&state.db, &params.invite_id).await?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// Join a server using an invite code
async fn join_server(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Json(payload): Json<JoinServerRequest>,
) -> AppResult<Json<ServerResponse>> {
    let user_id = auth.user_id;

    // Find invite by code
    let invite = InviteRepository::find_by_code(&state.db, &payload.code)
        .await?
        .ok_or_else(|| AppError::NotFound("Invalid invite code".to_string()))?;

    // Check if invite is valid
    if !invite.is_valid() {
        return Err(AppError::BadRequest("Invite has expired or reached max uses".to_string()));
    }

    // Check if user is already a member
    if MembershipRepository::is_member(&state.db, user_id, invite.server_id).await? {
        return Err(AppError::Conflict("Already a member of this server".to_string()));
    }

    // Check if user is currently banned from this server
    if MembershipRepository::is_banned(&state.db, invite.server_id, user_id).await? {
        return Err(AppError::Forbidden(
            "You are banned from this server".to_string(),
        ));
    }

    // Add user as MEMBER
    MembershipRepository::create(&state.db, user_id, invite.server_id, MemberRole::Member).await?;

    // Increment invite use count
    InviteRepository::increment_uses(&state.db, &invite.code).await?;

    // Return server info
    let server = ServerRepository::find_by_id(&state.db, invite.server_id)
        .await?
        .ok_or_else(|| AppError::Internal("Server not found".to_string()))?;

    let member_count = ServerRepository::get_member_count(&state.db, invite.server_id).await?;
    let mut response = ServerResponse::from(server);
    response.member_count = Some(member_count);

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use crate::auth::test_require_auth;
    use crate::repositories::{MembershipRepository, ServerRepository, UserRepository};
    use crate::test_utils::{delete_server, delete_user, test_config, try_test_pool};

    #[tokio::test]
    async fn invite_create_list_join_delete_flow() {
        let Some(pool) = try_test_pool().await else { return; };
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://epitalk:Epitalk94!@localhost:5432/epitalk".to_string());
        let state = Arc::new(AppState::new(pool.clone(), test_config(&db_url)));

        let owner = UserRepository::create(
            &pool,
            &format!("inv-owner-{}@example.test", Uuid::new_v4()),
            "hash",
            &format!("inv_owner_{}", Uuid::new_v4().to_string().replace('-', "")),
        )
        .await
        .expect("create owner");

        let member = UserRepository::create(
            &pool,
            &format!("inv-member-{}@example.test", Uuid::new_v4()),
            "hash",
            &format!("inv_member_{}", Uuid::new_v4().to_string().replace('-', "")),
        )
        .await
        .expect("create member");

        let server = ServerRepository::create(&pool, "Invites", owner.id)
            .await
            .expect("create server");

        let auth = test_require_auth(owner.id, &owner.email, &owner.username);
        let invite = create_invite(
            State(state.clone()),
            auth.clone(),
            Path(ServerPath { server_id: server.id }),
            Json(CreateInviteRequest { expires_in_hours: Some(24), max_uses: Some(2) }),
        )
        .await
        .expect("create invite")
        .0;

        let listed = list_invites(
            State(state.clone()),
            auth.clone(),
            Path(ServerPath { server_id: server.id }),
        )
        .await
        .expect("list invites")
        .0;
        assert!(listed.iter().any(|i| i.code == invite.code));

        let join_auth = test_require_auth(member.id, &member.email, &member.username);
        let joined = join_server(
            State(state.clone()),
            join_auth.clone(),
            Json(JoinServerRequest { code: invite.code.clone() }),
        )
        .await
        .expect("join server")
        .0;
        assert_eq!(joined.id, server.id);

        let member_role = MembershipRepository::get_role(&pool, member.id, server.id)
            .await
            .expect("role")
            .expect("member role");
        assert_eq!(member_role, MemberRole::Member);

        let _ = delete_invite(
            State(state.clone()),
            auth.clone(),
            Path(InvitePath { server_id: server.id, invite_id: invite.code.clone() }),
        )
        .await
        .expect("delete invite");

        delete_server(&pool, server.id).await;
        delete_user(&pool, owner.id).await;
        delete_user(&pool, member.id).await;
    }
}
