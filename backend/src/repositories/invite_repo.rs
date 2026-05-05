//! Invite repository

use crate::error::{AppError, AppResult};
use crate::models::Invite;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct InviteRepository;

impl InviteRepository {
    /// Find invite by code (primary key)
    pub async fn find_by_code(pool: &PgPool, code: &str) -> AppResult<Option<Invite>> {
        let invite = sqlx::query_as::<_, Invite>(
            r#"
            SELECT code, server_id, created_by, expires_at, max_uses, uses, created_at
            FROM invites
            WHERE code = $1
            "#,
        )
        .bind(code)
        .fetch_optional(pool)
        .await?;

        Ok(invite)
    }

    /// Find all invites for a server
    pub async fn find_by_server(pool: &PgPool, server_id: Uuid) -> AppResult<Vec<Invite>> {
        let invites = sqlx::query_as::<_, Invite>(
            r#"
            SELECT code, server_id, created_by, expires_at, max_uses, uses, created_at
            FROM invites
            WHERE server_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(server_id)
        .fetch_all(pool)
        .await?;

        Ok(invites)
    }

    /// Find all ACTIVE invites for a server (not expired, not at max uses)
    pub async fn find_active_by_server(pool: &PgPool, server_id: Uuid) -> AppResult<Vec<Invite>> {
        let invites = sqlx::query_as::<_, Invite>(
            r#"
            SELECT code, server_id, created_by, expires_at, max_uses, uses, created_at
            FROM invites
            WHERE server_id = $1
              AND (expires_at IS NULL OR expires_at > NOW())
              AND (max_uses IS NULL OR uses < max_uses)
            ORDER BY created_at DESC
            "#,
        )
        .bind(server_id)
        .fetch_all(pool)
        .await?;

        Ok(invites)
    }

    /// Create a new invite
    pub async fn create(
        pool: &PgPool,
        server_id: Uuid,
        created_by: Uuid,
        expires_in_hours: Option<i64>,
        max_uses: Option<i32>,
    ) -> AppResult<Invite> {
        // Generate a unique code (8 characters)
        let code = generate_invite_code();

        // Calculate expiration
        let expires_at = expires_in_hours.map(|hours| Utc::now() + Duration::hours(hours));

        let invite = sqlx::query_as::<_, Invite>(
            r#"
            INSERT INTO invites (server_id, code, created_by, expires_at, max_uses)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING code, server_id, created_by, expires_at, max_uses, uses, created_at
            "#,
        )
        .bind(server_id)
        .bind(&code)
        .bind(created_by)
        .bind(expires_at)
        .bind(max_uses)
        .fetch_one(pool)
        .await?;

        Ok(invite)
    }

    /// Increment use count (when someone joins via invite)
    pub async fn increment_uses(pool: &PgPool, code: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE invites
            SET uses = uses + 1
            WHERE code = $1
            "#,
        )
        .bind(code)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete invite by code
    pub async fn delete(pool: &PgPool, code: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM invites WHERE code = $1")
            .bind(code)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Invite not found".to_string()));
        }

        Ok(())
    }

    /// Delete expired invites (cleanup job)
    #[allow(dead_code)] // For scheduled cleanup tasks
    pub async fn delete_expired(pool: &PgPool) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM invites
            WHERE expires_at IS NOT NULL AND expires_at < NOW()
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

/// Generate a random invite code (8 alphanumeric characters)
fn generate_invite_code() -> String {
    // Use UUID v4 and take first 8 characters (after removing hyphens)
    let uuid = Uuid::new_v4().to_string().replace('-', "");
    uuid.chars().take(8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::{ServerRepository, UserRepository};
    use crate::test_utils::{delete_server, delete_user, try_test_pool};

    #[tokio::test]
    async fn create_list_increment_and_delete_invite() {
        let Some(pool) = try_test_pool().await else { return; };
        let owner = UserRepository::create(
            &pool,
            &format!("inv-{}@example.test", Uuid::new_v4()),
            "hash",
            &format!("inv_{}", Uuid::new_v4().to_string().replace('-', "")),
        )
        .await
        .expect("create owner");

        let server = ServerRepository::create(&pool, "Invites", owner.id)
            .await
            .expect("create server");

        let invite = InviteRepository::create(&pool, server.id, owner.id, Some(24), Some(3))
            .await
            .expect("create invite");

        let found = InviteRepository::find_by_code(&pool, &invite.code)
            .await
            .expect("find by code")
            .expect("invite exists");
        assert_eq!(found.server_id, server.id);

        InviteRepository::increment_uses(&pool, &invite.code)
            .await
            .expect("increment uses");

        let active = InviteRepository::find_active_by_server(&pool, server.id)
            .await
            .expect("list active");
        assert!(active.iter().any(|i| i.code == invite.code));

        InviteRepository::delete(&pool, &invite.code)
            .await
            .expect("delete invite");

        delete_server(&pool, server.id).await;
        delete_user(&pool, owner.id).await;
    }
}
