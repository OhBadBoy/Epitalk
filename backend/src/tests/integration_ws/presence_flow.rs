//! Executable tests for presence lifecycle semantics.
//!
//! These tests validate the behavior expected by the WebSocket presence flow
//! without requiring a live network server.

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::services::presence_service::{PresenceService, PresenceStatus};

    #[test]
    fn first_connection_transitions_user_to_online() {
        let svc = PresenceService::new();
        let changed = svc.add_connection("u1", "c1");
        assert!(changed);
        assert!(svc.is_online("u1"));
    }

    #[test]
    fn multiple_connections_require_last_disconnect_for_offline() {
        let svc = PresenceService::new();
        let _ = svc.add_connection("u1", "c1");
        let changed_second = svc.add_connection("u1", "c2");
        assert!(!changed_second);

        let first_remove = svc.remove_connection("u1", "c1");
        assert!(first_remove.is_none());
        assert!(svc.is_online("u1"));

        let second_remove = svc.remove_connection("u1", "c2");
        assert_eq!(second_remove, Some(PresenceStatus::Offline));
        assert!(!svc.is_online("u1"));
    }

    #[test]
    fn refresh_activity_reactivates_idle_user() {
        let svc = PresenceService::new();
        let _ = svc.set_online("u1");
        let _ = svc.set_status("u1", PresenceStatus::Idle);
        let changed = svc.refresh_activity("u1");

        assert!(changed);
        assert_eq!(svc.get_status("u1"), PresenceStatus::Online);
    }

    #[test]
    fn scan_for_idle_marks_stale_online_users() {
        let mut svc = PresenceService::new();
        svc.idle_threshold = Duration::from_millis(1);

        let _ = svc.set_online("u1");
        std::thread::sleep(Duration::from_millis(3));

        let changed = svc.scan_for_idle();
        assert!(changed.contains(&"u1".to_string()));
        assert_eq!(svc.get_status("u1"), PresenceStatus::Offline);
    }
}
