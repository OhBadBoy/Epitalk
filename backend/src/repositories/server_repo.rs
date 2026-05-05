//! Server repository

use crate::error::{AppError, AppResult};
use crate::models::{MemberRole, Server};
use sqlx::PgPool;
use uuid::Uuid;

pub struct ServerRepository;

impl ServerRepository {
    /// Find server by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<Server>> {
        let server = sqlx::query_as::<_, Server>(
            r#"
            SELECT id, name, owner_id, created_at, updated_at
            FROM servers
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(server)
    }

    /// Find all servers for a user (where they are a member)
    pub async fn find_by_user_id(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Server>> {
        let servers = sqlx::query_as::<_, Server>(
            r#"
            SELECT s.id, s.name, s.owner_id, s.created_at, s.updated_at
            FROM servers s
            INNER JOIN memberships m ON s.id = m.server_id
            WHERE m.user_id = $1
            ORDER BY s.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(servers)
    }

    /// Create a new server (also creates owner membership)
    pub async fn create(pool: &PgPool, name: &str, owner_id: Uuid) -> AppResult<Server> {
        // Start transaction
        let mut tx = pool.begin().await?;

        // Create server
        let server = sqlx::query_as::<_, Server>(
            r#"
            INSERT INTO servers (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(owner_id)
        .fetch_one(&mut *tx)
        .await?;

        // Create owner membership
        sqlx::query(
            r#"
            INSERT INTO memberships (user_id, server_id, role)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(owner_id)
        .bind(server.id)
        .bind(MemberRole::Owner)
        .execute(&mut *tx)
        .await?;

        // Create default #general channel
        sqlx::query(
            r#"
            INSERT INTO channels (server_id, name, kind)
            VALUES ($1, 'general', 'TEXT')
            "#,
        )
        .bind(server.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(server)
    }

    /// Update server
    pub async fn update(pool: &PgPool, id: Uuid, name: &str) -> AppResult<Server> {
        let server = sqlx::query_as::<_, Server>(
            r#"
            UPDATE servers
            SET name = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, owner_id, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(name)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Server not found".to_string()))?;

        Ok(server)
    }

    /// Delete server (cascades to memberships, channels, invites)
    pub async fn delete(pool: &PgPool, id: Uuid) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM servers WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Server not found".to_string()));
        }

        Ok(())
    }

    /// Transfer ownership to another member
    pub async fn transfer_ownership(
        pool: &PgPool,
        server_id: Uuid,
        new_owner_id: Uuid,
        old_owner_id: Uuid,
    ) -> AppResult<Server> {
        let mut tx = pool.begin().await?;

        // Update server owner
        let server = sqlx::query_as::<_, Server>(
            r#"
            UPDATE servers
            SET owner_id = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, owner_id, created_at, updated_at
            "#,
        )
        .bind(server_id)
        .bind(new_owner_id)
        .fetch_one(&mut *tx)
        .await?;

        // Update new owner's role to OWNER
        sqlx::query(
            r#"
            UPDATE memberships
            SET role = 'OWNER'
            WHERE server_id = $1 AND user_id = $2
            "#,
        )
        .bind(server_id)
        .bind(new_owner_id)
        .execute(&mut *tx)
        .await?;

        // Demote old owner to ADMIN
        sqlx::query(
            r#"
            UPDATE memberships
            SET role = 'ADMIN'
            WHERE server_id = $1 AND user_id = $2
            "#,
        )
        .bind(server_id)
        .bind(old_owner_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(server)
    }

    /// Get member count for a server
    pub async fn get_member_count(pool: &PgPool, server_id: Uuid) -> AppResult<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM memberships WHERE server_id = $1
            "#,
        )
        .bind(server_id)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::MemberRole;
    use crate::repositories::{MembershipRepository, UserRepository};
    use crate::test_utils::{delete_server, delete_user, try_test_pool};

    #[tokio::test]
    async fn create_update_transfer_and_delete_server() {
        let Some(pool) = try_test_pool().await else { return; };
        let owner_email = format!("owner-{}@example.test", Uuid::new_v4());
        let owner_username = format!("owner_{}", Uuid::new_v4().to_string().replace('-', ""));
        let new_owner_email = format!("owner2-{}@example.test", Uuid::new_v4());
        let new_owner_username = format!("owner2_{}", Uuid::new_v4().to_string().replace('-', ""));

        let owner = UserRepository::create(&pool, &owner_email, "hash", &owner_username)
            .await
            .expect("create owner");
        let new_owner = UserRepository::create(&pool, &new_owner_email, "hash", &new_owner_username)
            .await
            .expect("create new owner");

        let server = ServerRepository::create(&pool, "Server", owner.id)
            .await
            .expect("create server");

        let found = ServerRepository::find_by_id(&pool, server.id)
            .await
            .expect("find server")
            .expect("server exists");
        assert_eq!(found.owner_id, owner.id);

        let updated = ServerRepository::update(&pool, server.id, "Updated")
            .await
            .expect("update server");
        assert_eq!(updated.name, "Updated");

        MembershipRepository::create(&pool, new_owner.id, server.id, MemberRole::Member)
            .await
            .expect("add member");

        let transferred = ServerRepository::transfer_ownership(&pool, server.id, new_owner.id, owner.id)
            .await
            .expect("transfer ownership");
        assert_eq!(transferred.owner_id, new_owner.id);

        let count = ServerRepository::get_member_count(&pool, server.id)
            .await
            .expect("member count");
        assert!(count >= 2);

        delete_server(&pool, server.id).await;
        delete_user(&pool, owner.id).await;
        delete_user(&pool, new_owner.id).await;
    }
}
