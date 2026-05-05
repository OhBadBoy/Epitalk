//! Membership repository

use crate::error::{AppError, AppResult};
use crate::models::{BanRecord, BanResponse, MemberRole, MemberResponse, Membership};
use sqlx::PgPool;
use uuid::Uuid;

pub struct MembershipRepository;

impl MembershipRepository {
    /// Find membership by user and server
    #[allow(dead_code)] // Used by RBAC middleware
    pub async fn find_by_user_and_server(
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
    ) -> AppResult<Option<Membership>> {
        let membership = sqlx::query_as::<_, Membership>(
            r#"
            SELECT id, user_id, server_id, role, joined_at
            FROM memberships
            WHERE user_id = $1 AND server_id = $2
            "#,
        )
        .bind(user_id)
        .bind(server_id)
        .fetch_optional(pool)
        .await?;

        Ok(membership)
    }

    /// Get all members of a server with user info
    pub async fn find_by_server(pool: &PgPool, server_id: Uuid) -> AppResult<Vec<MemberResponse>> {
        let members = sqlx::query_as::<_, MemberResponse>(
            r#"
            SELECT m.user_id, u.username, m.role, m.joined_at, u.avatar_url, u.status::TEXT AS status
            FROM memberships m
            INNER JOIN users u ON m.user_id = u.id
            WHERE m.server_id = $1
            ORDER BY 
                CASE m.role 
                    WHEN 'OWNER' THEN 1 
                    WHEN 'ADMIN' THEN 2 
                    ELSE 3 
                END,
                m.joined_at ASC
            "#,
        )
        .bind(server_id)
        .fetch_all(pool)
        .await?;

        Ok(members)
    }

    /// Add a member to a server
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
        role: MemberRole,
    ) -> AppResult<Membership> {
        let membership = sqlx::query_as::<_, Membership>(
            r#"
            INSERT INTO memberships (user_id, server_id, role)
            VALUES ($1, $2, $3)
            RETURNING user_id, server_id, role, joined_at
            "#,
        )
        .bind(user_id)
        .bind(server_id)
        .bind(role)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.constraint() == Some("uq_memberships_user_server") {
                    return AppError::Conflict("User is already a member of this server".to_string());
                }
            }
            AppError::Database(e)
        })?;

        Ok(membership)
    }

    /// Update member role
    pub async fn update_role(
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
        role: MemberRole,
    ) -> AppResult<Membership> {
        let membership = sqlx::query_as::<_, Membership>(
            r#"
            UPDATE memberships
            SET role = $3
            WHERE user_id = $1 AND server_id = $2
            RETURNING user_id, server_id, role, joined_at
            "#,
        )
        .bind(user_id)
        .bind(server_id)
        .bind(role)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Membership not found".to_string()))?;

        Ok(membership)
    }

    /// Remove a member from a server
    pub async fn delete(pool: &PgPool, user_id: Uuid, server_id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM memberships
            WHERE user_id = $1 AND server_id = $2
            "#,
        )
        .bind(user_id)
        .bind(server_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Membership not found".to_string()));
        }

        Ok(())
    }

    /// Check if user is member of server
    pub async fn is_member(pool: &PgPool, user_id: Uuid, server_id: Uuid) -> AppResult<bool> {
        let exists: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM memberships
                WHERE user_id = $1 AND server_id = $2
            )
            "#,
        )
        .bind(user_id)
        .bind(server_id)
        .fetch_one(pool)
        .await?;

        Ok(exists.0)
    }

    /// Get user's role in a server
    pub async fn get_role(
        pool: &PgPool,
        user_id: Uuid,
        server_id: Uuid,
    ) -> AppResult<Option<MemberRole>> {
        let role: Option<(MemberRole,)> = sqlx::query_as(
            r#"
            SELECT role FROM memberships
            WHERE user_id = $1 AND server_id = $2
            "#,
        )
        .bind(user_id)
        .bind(server_id)
        .fetch_optional(pool)
        .await?;

        Ok(role.map(|(r,)| r))
    }

    // -------------------------------------------------------------------------
    // Ban methods
    // -------------------------------------------------------------------------

    /// Ban a user from a server.
    /// If `expires_at` is None → permanent ban.
    /// If the user is already banned, returns a Conflict error.
    pub async fn ban_user(
        pool: &PgPool,
        server_id: Uuid,
        user_id: Uuid,
        banned_by: Uuid,
        reason: Option<String>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AppResult<BanRecord> {
        let ban = sqlx::query_as::<_, BanRecord>(
            r#"
            INSERT INTO bans (server_id, user_id, banned_by, reason, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (server_id, user_id)
            DO UPDATE SET
                banned_by  = EXCLUDED.banned_by,
                reason     = EXCLUDED.reason,
                expires_at = EXCLUDED.expires_at,
                created_at = NOW()
            RETURNING id, server_id, user_id, banned_by, reason, expires_at, created_at
            "#,
        )
        .bind(server_id)
        .bind(user_id)
        .bind(banned_by)
        .bind(reason)
        .bind(expires_at)
        .fetch_one(pool)
        .await?;

        Ok(ban)
    }

    /// Remove a ban (unban a user).
    pub async fn unban_user(
        pool: &PgPool,
        server_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM bans
            WHERE server_id = $1 AND user_id = $2
            "#,
        )
        .bind(server_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Ban not found".to_string()));
        }
        Ok(())
    }

    /// Check if a user is currently banned from a server.
    /// Ignores expired bans (expires_at < NOW()).
    pub async fn is_banned(
        pool: &PgPool,
        server_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<bool> {
        let exists: (bool,) = sqlx::query_as(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM bans
                WHERE server_id = $1
                  AND user_id   = $2
                  AND (expires_at IS NULL OR expires_at > NOW())
            )
            "#,
        )
        .bind(server_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(exists.0)
    }

    /// List all active bans for a server (with username).
    pub async fn find_bans_by_server(
        pool: &PgPool,
        server_id: Uuid,
    ) -> AppResult<Vec<BanResponse>> {
        let bans = sqlx::query_as::<_, BanResponse>(
            r#"
            SELECT b.id, b.user_id, u.username, b.banned_by,
                   b.reason, b.expires_at, b.created_at
            FROM bans b
            INNER JOIN users u ON b.user_id = u.id
            WHERE b.server_id = $1
              AND (b.expires_at IS NULL OR b.expires_at > NOW())
            ORDER BY b.created_at DESC
            "#,
        )
        .bind(server_id)
        .fetch_all(pool)
        .await?;

        Ok(bans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::{ServerRepository, UserRepository};
    use crate::test_utils::{delete_server, delete_user, try_test_pool};

    #[tokio::test]
    async fn membership_lifecycle_and_bans() {
        let Some(pool) = try_test_pool().await else { return; };
        let owner = UserRepository::create(
            &pool,
            &format!("own-{}@example.test", Uuid::new_v4()),
            "hash",
            &format!("own_{}", Uuid::new_v4().to_string().replace('-', "")),
        )
        .await
        .expect("create owner");

        let member = UserRepository::create(
            &pool,
            &format!("mem-{}@example.test", Uuid::new_v4()),
            "hash",
            &format!("mem_{}", Uuid::new_v4().to_string().replace('-', "")),
        )
        .await
        .expect("create member");

        let server = ServerRepository::create(&pool, "Members", owner.id)
            .await
            .expect("create server");

        let membership = MembershipRepository::create(&pool, member.id, server.id, MemberRole::Member)
            .await
            .expect("create membership");
        assert_eq!(membership.role, MemberRole::Member);

        let is_member = MembershipRepository::is_member(&pool, member.id, server.id)
            .await
            .expect("is_member");
        assert!(is_member);

        let role = MembershipRepository::get_role(&pool, member.id, server.id)
            .await
            .expect("get_role")
            .expect("role");
        assert_eq!(role, MemberRole::Member);

        let updated = MembershipRepository::update_role(&pool, member.id, server.id, MemberRole::Moderator)
            .await
            .expect("update role");
        assert_eq!(updated.role, MemberRole::Moderator);

        let ban = MembershipRepository::ban_user(
            &pool,
            server.id,
            member.id,
            owner.id,
            Some("spam".to_string()),
            None,
        )
        .await
        .expect("ban user");
        assert_eq!(ban.user_id, member.id);

        let banned = MembershipRepository::is_banned(&pool, server.id, member.id)
            .await
            .expect("is_banned");
        assert!(banned);

        let bans = MembershipRepository::find_bans_by_server(&pool, server.id)
            .await
            .expect("list bans");
        assert!(bans.iter().any(|b| b.user_id == member.id));

        MembershipRepository::unban_user(&pool, server.id, member.id)
            .await
            .expect("unban user");

        let banned_after = MembershipRepository::is_banned(&pool, server.id, member.id)
            .await
            .expect("is_banned");
        assert!(!banned_after);

        MembershipRepository::delete(&pool, member.id, server.id)
            .await
            .expect("delete membership");

        delete_server(&pool, server.id).await;
        delete_user(&pool, owner.id).await;
        delete_user(&pool, member.id).await;
    }
}
