//! Executable tests for WebSocket message protocol contracts.
//!
//! These tests do not require a running backend process. They validate the
//! JSON protocol shape and content rules used by the message flow.

#[cfg(test)]
mod tests {
    use crate::ws::protocol::{validate_content, ClientEvent, MAX_CONTENT_LEN, ServerEvent};
    use serde_json::json;

    #[test]
    fn message_send_event_deserializes_expected_payload() {
        let raw = json!({
            "type": "MessageSend",
            "payload": {
                "channel_id": "chan-1",
                "content": "hello",
                "reply_to": "msg-0"
            }
        });

        let evt: ClientEvent = serde_json::from_value(raw).expect("deserialize MessageSend");
        match evt {
            ClientEvent::MessageSend {
                channel_id,
                content,
                reply_to,
            } => {
                assert_eq!(channel_id, "chan-1");
                assert_eq!(content, "hello");
                assert_eq!(reply_to.as_deref(), Some("msg-0"));
            }
            _ => panic!("expected MessageSend variant"),
        }
    }

    #[test]
    fn oversized_message_content_is_rejected() {
        let too_long = "x".repeat(MAX_CONTENT_LEN + 1);
        let err = validate_content(&too_long).expect_err("validation should fail");
        assert_eq!(err, "message content exceeds maximum length");
    }

    #[test]
    fn message_new_serialization_keeps_username_contract() {
        let evt = ServerEvent::MessageNew {
            id: "m1".to_string(),
            channel_id: "chan-1".to_string(),
            author_id: "u1".to_string(),
            username: "alice".to_string(),
            content: "hello".to_string(),
            created_at: "2026-04-10T12:00:00Z".to_string(),
            reply_to: None,
        };

        let v = serde_json::to_value(evt).expect("serialize MessageNew");
        assert_eq!(v["type"], "MessageNew");
        assert_eq!(v["payload"]["username"], "alice");
        assert_eq!(v["payload"]["channel_id"], "chan-1");
    }
}
