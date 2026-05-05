//! Channel model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

/// Channel kind enum (matches PostgreSQL ENUM)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "channel_kind", rename_all = "UPPERCASE")]
pub enum ChannelKind {
    Text,
}

impl std::fmt::Display for ChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelKind::Text => write!(f, "TEXT"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Channel {
    pub id: Uuid,
    pub server_id: Uuid,
    pub name: String,
    pub kind: ChannelKind,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Channel response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelResponse {
    pub id: Uuid,
    pub server_id: Uuid,
    pub name: String,
    pub kind: ChannelKind,
    pub created_at: DateTime<Utc>,
}

impl From<Channel> for ChannelResponse {
    fn from(channel: Channel) -> Self {
        Self {
            id: channel.id,
            server_id: channel.server_id,
            name: channel.name,
            kind: channel.kind,
            created_at: channel.created_at,
        }
    }
}

/// Create channel request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct CreateChannelRequest {
    #[validate(length(min = 1, max = 100, message = "Channel name must be between 1 and 100 characters"))]
    pub name: String,
    #[serde(default = "default_channel_kind")]
    pub kind: ChannelKind,
}

fn default_channel_kind() -> ChannelKind {
    ChannelKind::Text
}

/// Update channel request
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct UpdateChannelRequest {
    #[validate(length(min = 1, max = 100, message = "Channel name must be between 1 and 100 characters"))]
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn default_kind_is_text() {
        assert!(matches!(default_channel_kind(), ChannelKind::Text));
    }

    #[test]
    fn create_channel_request_validation_on_name() {
        let ok = CreateChannelRequest {
            name: "general".into(),
            kind: ChannelKind::Text,
        };
        assert!(ok.validate().is_ok());

        let empty = CreateChannelRequest {
            name: "".into(),
            kind: ChannelKind::Text,
        };
        assert!(empty.validate().is_err());

        let long_name = "x".repeat(101);
        let too_long = CreateChannelRequest {
            name: long_name,
            kind: ChannelKind::Text,
        };
        assert!(too_long.validate().is_err());
    }
}
