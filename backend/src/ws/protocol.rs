use serde::{Deserialize, Serialize};

/// Maximum size (in bytes) of a single WebSocket text frame.
pub const MAX_FRAME_BYTES: usize = 16_384; // 16 KiB

/// Maximum length (in characters) of a chat message body.
pub const MAX_CONTENT_LEN: usize = 4_000;

/// Minimum throttle interval between consecutive `TypingStart` events
/// from the same user (in milliseconds).
pub const TYPING_THROTTLE_MS: u64 = 800;

//
// CLIENT → SERVER
//
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientEvent {
    MessageSend {
        channel_id: String,
        content: String,
        reply_to: Option<String>,
        attachment_url: Option<String>,
    },
    MessageSchedule {
        channel_id: String,
        content: String,
        day_of_week: u8,
        hour: u8,
        minute: u8,
        reply_to: Option<String>,
    },
    MessageEdit {
        channel_id: String,
        message_id: String,
        content: String,
    },
    MessageDelete {
        channel_id: String,
        message_id: String,
    },
    /// Send a GIF as a message with normalized metadata
    MessageSendGif {
        channel_id: String,
        gif: GifPayload,
        // optional caption
        caption: Option<String>,
    },
    JoinChannel {
        channel_id: String,
    },
    LeaveChannel {
        channel_id: String,
    },
    TypingStart {
        channel_id: String,
    },
    TypingStop {
        channel_id: String,
    },
    /// Client requests to add a reaction to a message
    ReactionAdd {
        message_id: String,
        emoji: String,
    },
    /// Client requests to remove a reaction from a message
    ReactionRemove {
        message_id: String,
        emoji: String,
    },
    /// Send a direct message to another user
    DmSend {
        recipient_id: String,
        content: String,
        reply_to: Option<String>,
        attachment_url: Option<String>,
    },
    /// Edit a direct message
    DmEdit {
        conversation_id: String,
        message_id: String,
        content: String,
    },
    /// Delete a direct message
    DmDelete {
        conversation_id: String,
        message_id: String,
    },
    /// Send a GIF as a direct message
    DmSendGif {
        recipient_id: String,
        gif: GifPayload,
        caption: Option<String>,
    },
    /// Join a DM conversation room (loads history)
    JoinDm {
        peer_id: String,
    },
    /// Leave a DM conversation room
    LeaveDm {
        peer_id: String,
    },
    Ping,
    /// Changer son statut de présence (online/idle/dnd/offline)
    PresenceSet {
        status: String,
    },
}

//
// SERVER → CLIENT
//
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum ServerEvent {
    MessageNew {
        id: String,
        channel_id: String,
        author_id: String,
        username: String,
        content: String,
        created_at: String,
        reply_to: Option<String>,
        attachment_url: Option<String>,
    },
    MessageScheduled {
        channel_id: String,
        day_of_week: u8,
        hour: u8,
        minute: u8,
        scheduled_for: String,
    },
    MessageEdited {
        id: String,
        channel_id: String,
        author_id: String,
        username: String,
        content: String,
        edited_at: String,
    },
    MessageDeleted {
        id: String,
        channel_id: String,
    },

    MessageUpdated {
        id: String,
        channel_id: String,
        author_id: String,
        username: String,
        content: String,
        edited_at: String,
    },

    MessagePinned {
        message_id: String,
        channel_id: String,
        pinned_by: String,
        pinned_at: String,
    },

    MessageUnpinned {
        message_id: String,
        channel_id: String,
        unpinned_by: String,
        unpinned_at: String,
    },

    UserJoined {
        user_id: String,
        channel_id: String,
    },

    UserLeft {
        user_id: String,
        channel_id: String,
    },

    TypingStart {
        user_id: String,
        username: String,
        channel_id: String,
    },

    TypingStop {
        user_id: String,
        username: String,
        channel_id: String,
    },

    Pong,

    /// Structured error sent to the client.
    Error {
        code: String,
        message: String,
    },

    UserOnline {
        user_id: String,
        status: String,
    },

    UserOffline {
        user_id: String,
        status: String,
    },

    /// Snapshot de présence envoyé au client qui rejoint un channel.
    PresenceSync {
        users: Vec<PresenceUser>,
    },

    /// Mise à jour structurée de la présence d'un utilisateur
    PresenceUpdated {
        user_id: String,
        status: String,
        last_activity: String,
    },

    ReactionAdded {
        message_id: String,
        emoji: String,
        user_id: String,
        username: Option<String>,
    },

    ReactionRemoved {
        message_id: String,
        emoji: String,
        user_id: String,
    },

    /// A new direct message
    DmNew {
        id: String,
        conversation_id: String,
        author_id: String,
        username: String,
        content: String,
        created_at: String,
        reply_to: Option<String>,
        attachment_url: Option<String>,
    },

    /// A direct message was edited
    DmEdited {
        id: String,
        conversation_id: String,
        author_id: String,
        username: String,
        content: String,
        edited_at: String,
    },

    /// A direct message was deleted
    DmDeleted {
        id: String,
        conversation_id: String,
    },
}

#[derive(Debug, Serialize, Clone)]
pub struct PresenceUser {
    pub user_id: String,
    pub status: String,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reaction {
    pub emoji: String,
    pub user_id: String,
    pub username: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GifPayload {
    pub id: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

// ─────────────────────────────────────────────────────────────────
// Validation helpers
// ─────────────────────────────────────────────────────────────────

pub fn validate_channel_id(id: &str) -> Result<(), &'static str> {
    if id.is_empty() {
        return Err("channel_id must not be empty");
    }
    if id.len() > 128 {
        return Err("channel_id too long");
    }
    if id.chars().any(|c| c.is_control()) {
        return Err("channel_id contains control characters");
    }
    Ok(())
}

pub fn validate_content(content: &str) -> Result<(), &'static str> {
    if content.is_empty() {
        return Err("message content must not be empty");
    }
    if content.len() > MAX_CONTENT_LEN {
        return Err("message content exceeds maximum length");
    }
    Ok(())
}

pub fn validate_schedule(day_of_week: u8, hour: u8, minute: u8) -> Result<(), &'static str> {
    if !(1..=7).contains(&day_of_week) {
        return Err("day_of_week must be between 1 (Mon) and 7 (Sun)");
    }
    if hour > 23 {
        return Err("hour must be between 0 and 23");
    }
    if minute > 59 {
        return Err("minute must be between 0 and 59");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_channel_id_rejects_empty_and_too_long() {
        assert!(validate_channel_id("").is_err());

        let long_id = "a".repeat(129);
        assert!(validate_channel_id(&long_id).is_err());
    }

    #[test]
    fn validate_channel_id_rejects_control_chars() {
        let id = "chan\nnel";
        assert!(validate_channel_id(id).is_err());
    }

    #[test]
    fn validate_channel_id_accepts_normal_strings() {
        assert!(validate_channel_id("general").is_ok());
        assert!(validate_channel_id("channel-123").is_ok());
    }

    #[test]
    fn validate_content_rejects_empty_and_too_long() {
        assert!(validate_content("").is_err());

        let long = "x".repeat(MAX_CONTENT_LEN + 1);
        assert!(validate_content(&long).is_err());
    }

    #[test]
    fn validate_content_accepts_valid_message() {
        assert!(validate_content("hello world").is_ok());
    }

    #[test]
    fn validate_schedule_rejects_out_of_range_values() {
        assert!(validate_schedule(0, 10, 30).is_err());
        assert!(validate_schedule(8, 10, 30).is_err());
        assert!(validate_schedule(2, 24, 30).is_err());
        assert!(validate_schedule(2, 10, 60).is_err());
    }

    #[test]
    fn validate_schedule_accepts_valid_values() {
        assert!(validate_schedule(1, 0, 0).is_ok());
        assert!(validate_schedule(7, 23, 59).is_ok());
    }

    #[test]
    fn server_event_serialization_roundtrip() {
        let event = ServerEvent::MessageNew {
            id: "1".into(),
            channel_id: "chan".into(),
            author_id: "user".into(),
            username: "alice".into(),
            content: "hello".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            reply_to: None,
            attachment_url: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(value["type"], "MessageNew");
        assert_eq!(value["payload"]["content"], "hello");
    }

    #[test]
    fn server_event_serialization_message_scheduled() {
        let event = ServerEvent::MessageScheduled {
            channel_id: "chan".into(),
            day_of_week: 3,
            hour: 14,
            minute: 45,
            scheduled_for: "2026-04-15T14:45:00Z".into(),
        };

        let value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&event).unwrap()).unwrap();

        assert_eq!(value["type"], "MessageScheduled");
        assert_eq!(value["payload"]["day_of_week"], 3);
    }

    #[test]
    fn server_event_serialization_updated_and_pin_variants() {
        let updated = ServerEvent::MessageUpdated {
            id: "m1".into(),
            channel_id: "chan".into(),
            author_id: "user".into(),
            username: "alice".into(),
            content: "edited".into(),
            edited_at: "2026-03-20T00:00:00Z".into(),
        };
        let pinned = ServerEvent::MessagePinned {
            message_id: "m1".into(),
            channel_id: "chan".into(),
            pinned_by: "mod".into(),
            pinned_at: "2026-03-20T00:00:01Z".into(),
        };
        let unpinned = ServerEvent::MessageUnpinned {
            message_id: "m1".into(),
            channel_id: "chan".into(),
            unpinned_by: "mod".into(),
            unpinned_at: "2026-03-20T00:00:02Z".into(),
        };

        let updated_v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&updated).unwrap()).unwrap();
        let pinned_v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&pinned).unwrap()).unwrap();
        let unpinned_v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&unpinned).unwrap()).unwrap();

        assert_eq!(updated_v["type"], "MessageUpdated");
        assert_eq!(updated_v["payload"]["content"], "edited");
        assert_eq!(pinned_v["type"], "MessagePinned");
        assert_eq!(pinned_v["payload"]["message_id"], "m1");
        assert_eq!(unpinned_v["type"], "MessageUnpinned");
        assert_eq!(unpinned_v["payload"]["message_id"], "m1");
    }
}
