//! Member routes
//!
//! Routes (nested under /servers/:server_id/members):
//! - GET    /                    - List server members
//! - GET    /:user_id            - Get member details
//! - PATCH  /:user_id/role       - Update member role (OWNER only)
//! - DELETE /:user_id            - Kick member (ADMIN+, not OWNER)
//! - POST   /:user_id/ban        - Ban member (ADMIN+, permanent or temporary)
//! - DELETE /:user_id/ban        - Unban member (ADMIN+)
//! - GET    /bans                - List active bans (members only)

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::auth::RequireAuth;
use crate::error::{AppError, AppResult};
use crate::models::{BanMemberRequest, BanResponse, MemberResponse, MemberRole, UpdateMemberRoleRequest};
use crate::repositories::MembershipRepository;
use crate::state::AppState;
use crate::ws::protocol::ServerEvent;
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_members))
        .route("/bans", get(list_bans))
        .route("/:user_id", get(get_member).delete(kick_member))
        .route("/:user_id/role", axum::routing::patch(update_member_role))
        .route(
            "/:user_id/ban",
            axum::routing::post(ban_member).delete(unban_member),
        )
}

/// Path parameters
#[derive(serde::Deserialize)]
pub struct ServerPath {
    pub server_id: Uuid,
}

#[derive(serde::Deserialize)]
pub struct MemberPath {
    pub server_id: Uuid,
    pub user_id: Uuid,
}

pub(crate) fn validate_ban_permissions(
    caller_id: Uuid,
    target_id: Uuid,
    caller_role: MemberRole,
    target_role: MemberRole,
) -> AppResult<()> {
    if caller_id == target_id {
        return Err(AppError::BadRequest("Cannot ban yourself.".to_string()));
    }

    if !caller_role.can_manage_channels() {
        return Err(AppError::Forbidden(
            "Insufficient permissions to ban members".to_string(),
        ));
    }

    if target_role == MemberRole::Owner {
        return Err(AppError::Forbidden("Cannot ban the server owner".to_string()));
    }

    if caller_role == MemberRole::Admin && target_role == MemberRole::Admin {
        return Err(AppError::Forbidden("Admins cannot ban other admins".to_string()));
    }

    Ok(())
}

pub(crate) fn build_ban_message(reason: Option<&str>) -> String {
    match reason {
        Some(reason) if !reason.trim().is_empty() => {
            format!("Vous avez ete banni de ce serveur. Motif: {}", reason)
        }
        _ => "Vous avez ete banni de ce serveur.".to_string(),
    }
}

pub(crate) fn is_ban_active(expires_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    match expires_at {
        None => true,
        Some(ts) => ts > now,
    }
}

/// List all members of a server
async fn list_members(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ServerPath>,
) -> AppResult<Json<Vec<MemberResponse>>> {
    let user_id = auth.user_id;

    // Check membership
    if !MembershipRepository::is_member(&state.db, user_id, params.server_id).await? {
        return Err(AppError::Forbidden("Not a member of this server".to_string()));
    }

    let members = MembershipRepository::find_by_server(&state.db, params.server_id).await?;

    Ok(Json(members))
}

/// Get member details
async fn get_member(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<MemberPath>,
) -> AppResult<Json<MemberResponse>> {
    let current_user_id = auth.user_id;

    // Check membership
    if !MembershipRepository::is_member(&state.db, current_user_id, params.server_id).await? {
        return Err(AppError::Forbidden("Not a member of this server".to_string()));
    }

    let members = MembershipRepository::find_by_server(&state.db, params.server_id).await?;
    let member = members
        .into_iter()
        .find(|m| m.user_id == params.user_id)
        .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

    Ok(Json(member))
}

/// Update member role (OWNER only)
async fn update_member_role(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<MemberPath>,
    Json(payload): Json<UpdateMemberRoleRequest>,
) -> AppResult<Json<MemberResponse>> {
    let current_user_id = auth.user_id;

    // Check caller's role
    let caller_role = MembershipRepository::get_role(&state.db, current_user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if !caller_role.can_manage_members() {
        return Err(AppError::Forbidden("Only the owner can manage roles".to_string()));
    }

    // Cannot change owner's role
    let target_role = MembershipRepository::get_role(&state.db, params.user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

    if target_role == MemberRole::Owner {
        return Err(AppError::BadRequest(
            "Cannot change owner's role. Use transfer ownership instead.".to_string()
        ));
    }

    // Cannot set someone to OWNER via this endpoint
    if payload.role == MemberRole::Owner {
        return Err(AppError::BadRequest(
            "Cannot set role to OWNER. Use transfer ownership endpoint.".to_string()
        ));
    }

    // Update role
    let _membership = MembershipRepository::update_role(
        &state.db,
        params.user_id,
        params.server_id,
        payload.role,
    )
    .await?;

    // Get user info for response
    let members = MembershipRepository::find_by_server(&state.db, params.server_id).await?;
    let member = members
        .into_iter()
        .find(|m| m.user_id == params.user_id)
        .ok_or_else(|| AppError::Internal("Failed to fetch updated member".to_string()))?;

    Ok(Json(member))
}

/// Kick member (ADMIN+, cannot kick OWNER or higher/equal role)
async fn kick_member(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<MemberPath>,
) -> AppResult<Json<serde_json::Value>> {
    let current_user_id = auth.user_id;

    // Cannot kick yourself (use leave instead)
    if current_user_id == params.user_id {
        return Err(AppError::BadRequest("Cannot kick yourself. Use leave endpoint.".to_string()));
    }

    // Check caller's role
    let caller_role = MembershipRepository::get_role(&state.db, current_user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    // Must be ADMIN or OWNER to kick
    if !caller_role.can_delete_others_messages() {
        return Err(AppError::Forbidden("Insufficient permissions to kick members".to_string()));
    }

    // Check target's role
    let target_role = MembershipRepository::get_role(&state.db, params.user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

    // Cannot kick owner
    if target_role == MemberRole::Owner {
        return Err(AppError::Forbidden("Cannot kick the server owner".to_string()));
    }

    // Admin cannot kick other admins
    if caller_role == MemberRole::Admin && target_role == MemberRole::Admin {
        return Err(AppError::Forbidden("Admins cannot kick other admins".to_string()));
    }

    // Perform kick
    MembershipRepository::delete(&state.db, params.user_id, params.server_id).await?;

    // Apply kick effect immediately on all active WebSocket sessions.
    state.hub.disconnect_user(&params.user_id.to_string());

    Ok(Json(serde_json::json!({ "kicked": true })))
}

/// Ban a member (ADMIN or OWNER only).
/// `expires_at = null` in the body = permanent ban.
/// The banned user is also removed from the server.
async fn ban_member(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<MemberPath>,
    Json(payload): Json<BanMemberRequest>,
) -> AppResult<Json<BanResponse>> {
    let current_user_id = auth.user_id;

    // Caller must be ADMIN or OWNER
    let caller_role = MembershipRepository::get_role(&state.db, current_user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    // Target must exist on the server
    let target_role = MembershipRepository::get_role(&state.db, params.user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

    validate_ban_permissions(current_user_id, params.user_id, caller_role, target_role)?;

    // Create the ban record
    let ban = MembershipRepository::ban_user(
        &state.db,
        params.server_id,
        params.user_id,
        current_user_id,
        payload.reason,
        payload.expires_at,
    )
    .await?;

    let ban_message = build_ban_message(ban.reason.as_deref());

    state.hub.send_event_to_user(
        &params.user_id.to_string(),
        ServerEvent::Error {
            code: "BANNED".to_string(),
            message: ban_message,
        },
    );

    // Also remove the user from the server (banned = no longer a member)
    let _ = MembershipRepository::delete(&state.db, params.user_id, params.server_id).await;

    // Apply ban effect immediately on all active WebSocket sessions.
    state.hub.disconnect_user(&params.user_id.to_string());

    // Build response (we need username)
    let ban_response = sqlx::query_as::<_, BanResponse>(
        r#"
        SELECT b.id, b.user_id, u.username, b.banned_by,
               b.reason, b.expires_at, b.created_at
        FROM bans b
        INNER JOIN users u ON b.user_id = u.id
        WHERE b.id = $1
        "#,
    )
    .bind(ban.id)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::Database)?;

    Ok(Json(ban_response))
}

/// Unban a member (ADMIN or OWNER only).
async fn unban_member(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<MemberPath>,
) -> AppResult<Json<serde_json::Value>> {
    let current_user_id = auth.user_id;

    // Caller must be ADMIN or OWNER
    let caller_role = MembershipRepository::get_role(&state.db, current_user_id, params.server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if !caller_role.can_manage_channels() {
        return Err(AppError::Forbidden(
            "Insufficient permissions to unban members".to_string(),
        ));
    }

    MembershipRepository::unban_user(&state.db, params.server_id, params.user_id).await?;

    Ok(Json(serde_json::json!({ "unbanned": true })))
}

/// List all active bans for a server (members only).
async fn list_bans(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(params): Path<ServerPath>,
) -> AppResult<Json<Vec<BanResponse>>> {
    let user_id = auth.user_id;

    // Must be a member of the server
    if !MembershipRepository::is_member(&state.db, user_id, params.server_id).await? {
        return Err(AppError::Forbidden("Not a member of this server".to_string()));
    }

    let bans = MembershipRepository::find_bans_by_server(&state.db, params.server_id).await?;

    Ok(Json(bans))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::State;
    use crate::auth::test_require_auth;
    use crate::repositories::{ServerRepository, UserRepository};
    use crate::test_utils::{delete_server, delete_user, test_config, try_test_pool};

    #[tokio::test]
    async fn members_list_kick_and_ban_flow() {
        let Some(pool) = try_test_pool().await else { return; };
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://epitalk:Epitalk94!@localhost:5432/epitalk".to_string());
        let state = Arc::new(AppState::new(pool.clone(), test_config(&db_url)));

        let owner = UserRepository::create(
            &pool,
            &format!("mem-owner-{}@example.test", Uuid::new_v4()),
            "hash",
            &format!("mem_owner_{}", Uuid::new_v4().to_string().replace('-', "")),
        )
        .await
        .expect("create owner");

        let member = UserRepository::create(
            &pool,
            &format!("mem-user-{}@example.test", Uuid::new_v4()),
            "hash",
            &format!("mem_user_{}", Uuid::new_v4().to_string().replace('-', "")),
        )
        .await
        .expect("create member");

        let server = ServerRepository::create(&pool, "Members", owner.id)
            .await
            .expect("create server");

        MembershipRepository::create(&pool, member.id, server.id, MemberRole::Member)
            .await
            .expect("add member");

        let owner_auth = test_require_auth(owner.id, &owner.email, &owner.username);

        let listed = list_members(
            State(state.clone()),
            owner_auth.clone(),
            Path(ServerPath { server_id: server.id }),
        )
        .await
        .expect("list members")
        .0;
        assert!(listed.iter().any(|m| m.user_id == member.id));

        let _ = kick_member(
            State(state.clone()),
            owner_auth.clone(),
            Path(MemberPath { server_id: server.id, user_id: member.id }),
        )
        .await
        .expect("kick member");

        MembershipRepository::create(&pool, member.id, server.id, MemberRole::Member)
            .await
            .expect("re-add member");

        let _ = ban_member(
            State(state.clone()),
            owner_auth.clone(),
            Path(MemberPath { server_id: server.id, user_id: member.id }),
            Json(BanMemberRequest { reason: Some("spam".to_string()), expires_at: None }),
        )
        .await
        .expect("ban member");

        let bans = list_bans(
            State(state.clone()),
            owner_auth.clone(),
            Path(ServerPath { server_id: server.id }),
        )
        .await
        .expect("list bans")
        .0;
        assert!(bans.iter().any(|b| b.user_id == member.id));

        let _ = unban_member(
            State(state.clone()),
            owner_auth.clone(),
            Path(MemberPath { server_id: server.id, user_id: member.id }),
        )
        .await
        .expect("unban member");

        delete_server(&pool, server.id).await;
        delete_user(&pool, owner.id).await;
        delete_user(&pool, member.id).await;
    }
}
