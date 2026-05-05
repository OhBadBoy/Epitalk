//! Server routes
//!
//! Routes:
//! - GET    /servers           - List user's servers
//! - POST   /servers           - Create a new server
//! - GET    /servers/:id       - Get server details
//! - PATCH  /servers/:id       - Update server
//! - DELETE /servers/:id       - Delete server
//! - POST   /servers/:id/leave - Leave server
//! - POST   /servers/:id/transfer - Transfer ownership

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::auth::RequireAuth;
use crate::error::{AppError, AppResult};
use crate::models::{
    CreateServerRequest, MemberRole, ServerResponse, UpdateServerRequest,
};
use crate::repositories::{MembershipRepository, ServerRepository};
use crate::state::AppState;

use super::channels;
use super::members;
use super::invites;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_servers).post(create_server))
        .route("/:server_id", get(get_server).patch(update_server).delete(delete_server))
        .route("/:server_id/leave", post(leave_server))
        .route("/:server_id/transfer", post(transfer_ownership))
        .nest("/:server_id/channels", channels::router())
        .nest("/:server_id/members", members::router())
        .nest("/:server_id/invites", invites::router())
}

/// List all servers the user is a member of
async fn list_servers(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
) -> AppResult<Json<Vec<ServerResponse>>> {
    let user_id = auth.user_id;

    let servers = ServerRepository::find_by_user_id(&state.db, user_id).await?;
    
    let mut responses = Vec::new();
    for server in servers {
        let member_count = ServerRepository::get_member_count(&state.db, server.id).await?;
        let mut response = ServerResponse::from(server);
        response.member_count = Some(member_count);
        responses.push(response);
    }

    Ok(Json(responses))
}

/// Create a new server
async fn create_server(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Json(payload): Json<CreateServerRequest>,
) -> AppResult<Json<ServerResponse>> {
    let user_id = auth.user_id;

    let server = ServerRepository::create(&state.db, &payload.name, user_id).await?;
    let mut response = ServerResponse::from(server);
    response.member_count = Some(1); // Owner is the first member

    Ok(Json(response))
}

/// Get server details
async fn get_server(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(server_id): Path<Uuid>,
) -> AppResult<Json<ServerResponse>> {
    let user_id = auth.user_id;

    // Check membership
    if !MembershipRepository::is_member(&state.db, user_id, server_id).await? {
        return Err(AppError::Forbidden("Not a member of this server".to_string()));
    }

    let server = ServerRepository::find_by_id(&state.db, server_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Server not found".to_string()))?;

    let member_count = ServerRepository::get_member_count(&state.db, server_id).await?;
    let mut response = ServerResponse::from(server);
    response.member_count = Some(member_count);

    Ok(Json(response))
}

/// Update server (ADMIN+ only)
async fn update_server(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(server_id): Path<Uuid>,
    Json(payload): Json<UpdateServerRequest>,
) -> AppResult<Json<ServerResponse>> {
    let user_id = auth.user_id;

    // Check role
    let role = MembershipRepository::get_role(&state.db, user_id, server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if !role.can_manage_channels() {
        return Err(AppError::Forbidden("Insufficient permissions".to_string()));
    }

    let server = ServerRepository::update(&state.db, server_id, &payload.name).await?;
    let member_count = ServerRepository::get_member_count(&state.db, server_id).await?;
    let mut response = ServerResponse::from(server);
    response.member_count = Some(member_count);

    Ok(Json(response))
}

/// Delete server (OWNER only)
async fn delete_server(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(server_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = auth.user_id;

    // Check role
    let role = MembershipRepository::get_role(&state.db, user_id, server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if role != MemberRole::Owner {
        return Err(AppError::Forbidden("Only the owner can delete the server".to_string()));
    }

    ServerRepository::delete(&state.db, server_id).await?;

    Ok(Json(serde_json::json!({ "deleted": true })))
}

/// Leave server (non-OWNER only)
async fn leave_server(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(server_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = auth.user_id;

    // Check role
    let role = MembershipRepository::get_role(&state.db, user_id, server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if role == MemberRole::Owner {
        return Err(AppError::BadRequest(
            "Owner cannot leave. Transfer ownership first or delete the server.".to_string()
        ));
    }

    MembershipRepository::delete(&state.db, user_id, server_id).await?;

    Ok(Json(serde_json::json!({ "left": true })))
}

/// Transfer ownership request
#[derive(serde::Deserialize)]
pub struct TransferOwnershipRequest {
    pub new_owner_id: Uuid,
}

/// Transfer server ownership (OWNER only)
async fn transfer_ownership(
    State(state): State<Arc<AppState>>,
    auth: RequireAuth,
    Path(server_id): Path<Uuid>,
    Json(payload): Json<TransferOwnershipRequest>,
) -> AppResult<Json<ServerResponse>> {
    let user_id = auth.user_id;

    // Check current user is owner
    let role = MembershipRepository::get_role(&state.db, user_id, server_id)
        .await?
        .ok_or_else(|| AppError::Forbidden("Not a member of this server".to_string()))?;

    if role != MemberRole::Owner {
        return Err(AppError::Forbidden("Only the owner can transfer ownership".to_string()));
    }

    // Check new owner is a member
    if !MembershipRepository::is_member(&state.db, payload.new_owner_id, server_id).await? {
        return Err(AppError::BadRequest("New owner must be a member of the server".to_string()));
    }

    let server = ServerRepository::transfer_ownership(
        &state.db,
        server_id,
        payload.new_owner_id,
        user_id,
    )
    .await?;

    let member_count = ServerRepository::get_member_count(&state.db, server_id).await?;
    let mut response = ServerResponse::from(server);
    response.member_count = Some(member_count);

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Json;
    use axum::extract::State;
    use crate::auth::test_require_auth;
    use crate::repositories::{MembershipRepository, UserRepository};
    use crate::test_utils::{delete_server as cleanup_server, delete_user, test_config, try_test_pool};

    #[tokio::test]
    async fn server_crud_and_transfer_flow() {
        let Some(pool) = try_test_pool().await else { return; };
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://epitalk:Epitalk94!@localhost:5432/epitalk".to_string());
        let state = Arc::new(AppState::new(pool.clone(), test_config(&db_url)));

        let owner = UserRepository::create(
            &pool,
            &format!("srv-owner-{}@example.test", Uuid::new_v4()),
            "hash",
            &format!("srv_owner_{}", Uuid::new_v4().to_string().replace('-', "")),
        )
        .await
        .expect("create owner");

        let member = UserRepository::create(
            &pool,
            &format!("srv-member-{}@example.test", Uuid::new_v4()),
            "hash",
            &format!("srv_member_{}", Uuid::new_v4().to_string().replace('-', "")),
        )
        .await
        .expect("create member");

        let owner_auth = test_require_auth(owner.id, &owner.email, &owner.username);
        let member_auth = test_require_auth(member.id, &member.email, &member.username);

        let created = create_server(
            State(state.clone()),
            owner_auth.clone(),
            Json(CreateServerRequest {
                name: "Test Server".to_string(),
            }),
        )
        .await
        .expect("create server")
        .0;

        let server_id = created.id;

        let servers = list_servers(State(state.clone()), owner_auth.clone())
            .await
            .expect("list servers");
        assert!(servers.0.iter().any(|s| s.id == server_id));

        let updated = update_server(
            State(state.clone()),
            owner_auth.clone(),
            Path(server_id),
            Json(UpdateServerRequest {
                name: "Updated".to_string(),
            }),
        )
        .await
        .expect("update server")
        .0;
        assert_eq!(updated.name, "Updated");

        MembershipRepository::create(&pool, member.id, server_id, MemberRole::Member)
            .await
            .expect("add member");

        let _ = leave_server(
            State(state.clone()),
            member_auth.clone(),
            Path(server_id),
        )
        .await
        .expect("leave server");

        MembershipRepository::create(&pool, member.id, server_id, MemberRole::Member)
            .await
            .expect("re-add member");

        let transferred = transfer_ownership(
            State(state.clone()),
            owner_auth.clone(),
            Path(server_id),
            Json(TransferOwnershipRequest { new_owner_id: member.id }),
        )
        .await
        .expect("transfer ownership")
        .0;
        assert_eq!(transferred.owner_id, member.id);

        let _ = delete_server(
            State(state.clone()),
            test_require_auth(member.id, &member.email, &member.username),
            Path(server_id),
        )
        .await
        .expect("delete server");

        cleanup_server(&pool, server_id).await;
        delete_user(&pool, owner.id).await;
        delete_user(&pool, member.id).await;
    }
}