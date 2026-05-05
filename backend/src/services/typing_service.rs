use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::time::Duration;

/// Service de typing : gère les indicateurs "en train d'écrire"
#[derive(Clone)]
pub struct TypingService {
    /// (channel_id, user_id) -> timestamp
    typing: DashMap<(String, String), DateTime<Utc>>,
    /// Durée après laquelle le typing expire
    timeout: Duration,
}

impl TypingService {
    pub fn new() -> Self {
        Self {
            typing: DashMap::new(),
            timeout: Duration::from_secs(5),
        }
    }

    /// Marquer un utilisateur comme en train d'écrire dans un channel
    #[allow(dead_code)]
    pub fn start_typing(&self, channel_id: &str, user_id: &str) {
        self.typing
            .insert((channel_id.to_string(), user_id.to_string()), Utc::now());
    }

    /// Arrêter le typing
    #[allow(dead_code)]
    pub fn stop_typing(&self, channel_id: &str, user_id: &str) {
        self.typing
            .remove(&(channel_id.to_string(), user_id.to_string()));
    }

    /// Vérifier si un utilisateur est en train d'écrire
    #[allow(dead_code)]
    pub fn is_typing(&self, channel_id: &str, user_id: &str) -> bool {
        if let Some(entry) = self
            .typing
            .get(&(channel_id.to_string(), user_id.to_string()))
        {
            let elapsed = Utc::now().signed_duration_since(*entry.value());
            elapsed < chrono::Duration::from_std(self.timeout).unwrap()
        } else {
            false
        }
    }

    /// Récupérer la liste des utilisateurs en train d'écrire dans un channel
    #[allow(dead_code)]
    pub fn list_typing(&self, channel_id: &str) -> Vec<String> {
        let now = Utc::now();
        let timeout = chrono::Duration::from_std(self.timeout).unwrap();

        self.typing
            .iter()
            .filter_map(|entry| {
                let (ch, user) = entry.key();
                if ch == channel_id && now.signed_duration_since(*entry.value()) < timeout {
                    Some(user.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Nettoyer les entrées expirées
    #[allow(dead_code)]
    pub fn cleanup(&self) {
        let now = Utc::now();
        let timeout = chrono::Duration::from_std(self.timeout).unwrap();

        self.typing.retain(|_, timestamp| {
            now.signed_duration_since(*timestamp) < timeout
        });
    }
}

impl Default for TypingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as ChronoDuration;

    #[test]
    fn start_and_stop_typing_updates_state() {
        let service = TypingService::new();

        service.start_typing("channel-1", "user-1");
        assert!(service.is_typing("channel-1", "user-1"));
        assert!(!service.is_typing("channel-1", "user-2"));

        let typing_users = service.list_typing("channel-1");
        assert_eq!(typing_users, vec!["user-1".to_string()]);

        service.stop_typing("channel-1", "user-1");
        assert!(!service.is_typing("channel-1", "user-1"));
        assert!(service.list_typing("channel-1").is_empty());
    }

    #[test]
    fn cleanup_removes_expired_entries() {
        let service = TypingService::new();

        // Insère une entrée plus vieille que le timeout pour forcer le nettoyage
        let past = Utc::now() - ChronoDuration::seconds(10);
        service
            .typing
            .insert(("channel-1".to_string(), "user-1".to_string()), past);

        assert_eq!(service.list_typing("channel-1").len(), 0);

        service.cleanup();

        // L'entrée doit avoir été supprimée
        assert!(service.list_typing("channel-1").is_empty());
        assert!(!service.is_typing("channel-1", "user-1"));
    }
}
