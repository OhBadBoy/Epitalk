//! User model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

/// User status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Type)]
#[sqlx(type_name = "user_status", rename_all = "UPPERCASE")]
pub enum UserStatus {
    #[serde(rename = "ONLINE")]
    Online,
    #[serde(rename = "IDLE")]
    Idle,
    #[serde(rename = "DND")]
    Dnd,
    #[serde(rename = "OFFLINE")]
    Offline,
}

impl Default for UserStatus {
    fn default() -> Self {
        Self::Online
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub banner_color_1: Option<String>,
    pub banner_color_2: Option<String>,
    pub status: UserStatus,
    pub theme: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User response (without password)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
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
    pub created_at: DateTime<Utc>,
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

/// Create user request
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)] // Alternate way to create users (future admin features)
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub username: String,
}
