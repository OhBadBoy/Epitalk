//! Executable tests for typing protocol and service behavior.
//!
//! These tests assert the contract used by the real-time typing flow.

#[cfg(test)]
mod tests {
    use crate::services::typing_service::TypingService;
    use crate::ws::protocol::{ClientEvent, TYPING_THROTTLE_MS};
    use serde_json::json;

    #[test]
    fn typing_start_event_deserializes_channel_id() {
        let raw = json!({
            "type": "TypingStart",
            "payload": { "channel_id": "chan-42" }
        });

        let evt: ClientEvent = serde_json::from_value(raw).expect("deserialize TypingStart");
        match evt {
            ClientEvent::TypingStart { channel_id } => assert_eq!(channel_id, "chan-42"),
            _ => panic!("expected TypingStart variant"),
        }
    }

    #[test]
    fn typing_service_tracks_users_per_channel() {
        let svc = TypingService::new();
        svc.start_typing("chan-a", "u1");
        svc.start_typing("chan-a", "u2");
        svc.start_typing("chan-b", "u3");

        let mut in_a = svc.list_typing("chan-a");
        in_a.sort();

        assert_eq!(in_a, vec!["u1".to_string(), "u2".to_string()]);
        assert_eq!(svc.list_typing("chan-b"), vec!["u3".to_string()]);
    }

    #[test]
    fn typing_stop_removes_user_from_channel() {
        let svc = TypingService::new();
        svc.start_typing("chan-a", "u1");
        assert!(svc.is_typing("chan-a", "u1"));

        svc.stop_typing("chan-a", "u1");
        assert!(!svc.is_typing("chan-a", "u1"));
        assert!(svc.list_typing("chan-a").is_empty());
    }

    #[test]
    fn typing_throttle_constant_is_800ms() {
        assert_eq!(TYPING_THROTTLE_MS, 800);
    }
}
