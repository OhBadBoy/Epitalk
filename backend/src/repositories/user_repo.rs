//! User repository

use crate::error::{AppError, AppResult};
use crate::models::user::UserStatus;
use crate::models::User;
use sqlx::PgPool;
use uuid::Uuid;

pub struct UserRepository;

impl UserRepository {
    /// Find user by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, username, avatar_url, bio,
                   banner_color_1, banner_color_2, status, theme, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(pool: &PgPool, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, username, avatar_url, bio,
                   banner_color_1, banner_color_2, status, theme, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find user by username
    pub async fn find_by_username(pool: &PgPool, username: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, username, avatar_url, bio,
                   banner_color_1, banner_color_2, status, theme, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Create a new user
    pub async fn create(
        pool: &PgPool,
        email: &str,
        password_hash: &str,
        username: &str,
    ) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, username)
            VALUES ($1, $2, $3)
            RETURNING id, email, password_hash, username, avatar_url, bio,
                      banner_color_1, banner_color_2, status, theme, created_at, updated_at
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(username)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.constraint() == Some("users_email_key") {
                    return AppError::Conflict("Email already exists".to_string());
                }
            }
            AppError::Database(e)
        })?;

        Ok(user)
    }

    /// Update user profile
    pub async fn update_profile(
        pool: &PgPool,
        id: Uuid,
        username: Option<&str>,
        bio: Option<&str>,
        avatar_url: Option<&str>,
        banner_color_1: Option<&str>,
        banner_color_2: Option<&str>,
        status: Option<&UserStatus>,
        theme: Option<&str>,
    ) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET username      = COALESCE($2, username),
                bio           = COALESCE($3, bio),
                avatar_url    = COALESCE($4, avatar_url),
                banner_color_1 = COALESCE($5, banner_color_1),
                banner_color_2 = COALESCE($6, banner_color_2),
                status        = COALESCE($7, status),
                theme         = COALESCE($8, theme),
                updated_at    = NOW()
            WHERE id = $1
            RETURNING id, email, password_hash, username, avatar_url, bio,
                      banner_color_1, banner_color_2, status, theme, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(username)
        .bind(bio)
        .bind(avatar_url)
        .bind(banner_color_1)
        .bind(banner_color_2)
        .bind(status)
        .bind(theme)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user)
    }

    /// Update user email
    pub async fn update_email(
        pool: &PgPool,
        id: Uuid,
        new_email: &str,
    ) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET email = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, email, password_hash, username, avatar_url, bio,
                      banner_color_1, banner_color_2, status, theme, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(new_email)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.constraint() == Some("users_email_key") {
                    return AppError::Conflict("Email already exists".to_string());
                }
            }
            AppError::Database(e)
        })?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user)
    }

    /// Update only the status column — called on WS connect/disconnect.
    pub async fn update_status(pool: &PgPool, id: Uuid, status: &UserStatus) -> AppResult<()> {
        sqlx::query(
            "UPDATE users SET status = $2, updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .bind(status)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Update user password hash
    pub async fn update_password(
        pool: &PgPool,
        id: Uuid,
        new_password_hash: &str,
    ) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET password_hash = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, email, password_hash, username, avatar_url, bio,
                      banner_color_1, banner_color_2, status, theme, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(new_password_hash)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{delete_user, try_test_pool};

    #[tokio::test]
    async fn create_and_find_user_by_email_and_id() {
        let Some(pool) = try_test_pool().await else { return; };
        let email = format!("user-{}@example.test", Uuid::new_v4());
        let username = format!("user_{}", Uuid::new_v4().to_string().replace('-', ""));

        let user = UserRepository::create(&pool, &email, "hash", &username)
            .await
            .expect("create user");

        let by_email = UserRepository::find_by_email(&pool, &email)
            .await
            .expect("find by email")
            .expect("user should exist");
        assert_eq!(by_email.id, user.id);

        let by_id = UserRepository::find_by_id(&pool, user.id)
            .await
            .expect("find by id")
            .expect("user should exist");
        assert_eq!(by_id.email, email);

        delete_user(&pool, user.id).await;
    }

    #[tokio::test]
    async fn update_profile_and_email_conflict() {
        let Some(pool) = try_test_pool().await else { return; };
        let email1 = format!("profile-{}@example.test", Uuid::new_v4());
        let email2 = format!("profile-{}@example.test", Uuid::new_v4());
        let username1 = format!("profile_{}", Uuid::new_v4().to_string().replace('-', ""));
        let username2 = format!("profile_{}", Uuid::new_v4().to_string().replace('-', ""));

        let user1 = UserRepository::create(&pool, &email1, "hash", &username1)
            .await
            .expect("create user1");
        let user2 = UserRepository::create(&pool, &email2, "hash", &username2)
            .await
            .expect("create user2");

        let updated = UserRepository::update_profile(
            &pool,
            user1.id,
            Some("newname"),
            Some("bio"),
            None,
            Some("#000000"),
            Some("#111111"),
            Some(&UserStatus::Idle),
            Some("dark"),
        )
        .await
        .expect("update profile");
        assert_eq!(updated.username, "newname");
        assert_eq!(updated.theme, "dark");

        let conflict = UserRepository::update_email(&pool, user2.id, &email1).await;
        assert!(matches!(conflict, Err(AppError::Conflict(_))));

        delete_user(&pool, user1.id).await;
        delete_user(&pool, user2.id).await;
    }
}
