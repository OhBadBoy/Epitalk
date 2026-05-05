//! Application state shared across handlers

use axum::extract::FromRef;
use crate::auth::JwtService;
use crate::config::Config;
use crate::db::message_repo::MessageRepo;
use crate::services::message_service::MessageService;
use crate::services::presence_service::PresenceService;
use crate::services::typing_service::TypingService;
use crate::ws::hub::Hub;
use mongodb::Database;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub jwt_service: JwtService,
    // MongoDB database
    pub mongo_db: Option<Database>,
    // WebSocket hub
    pub hub: Arc<Hub>,
    // Message service (MongoDB)
    pub message_service: Arc<MessageService>,
    // Presence service
    pub presence: Arc<PresenceService>,
    // Typing service
    pub typing_service: Arc<TypingService>,
}

impl AppState {
    pub fn new(db: PgPool, config: Config) -> Self {
        let jwt_service = JwtService::new(&config.jwt_secret, config.jwt_expiration_hours);

        // Create hub
        let hub = Arc::new(Hub::new());

        // Create presence service
        let presence = Arc::new(PresenceService::new());

        // Spawn a background scanner that marks idle users after threshold
        {
            let presence_clone = presence.clone();
            let hub_clone = Arc::new(Hub::new());
            // We want to broadcast to the real hub: use hub (not hub_clone)
            let hub_for_scan = hub.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    let changed = presence_clone.scan_for_idle();
                    for user_id in changed {
                        // After threshold we now consider the user offline
                        let event = crate::ws::protocol::ServerEvent::PresenceUpdated {
                            user_id: user_id.clone(),
                            status: "offline".to_string(),
                            last_activity: chrono::Utc::now().to_rfc3339(),
                        };
                        hub_for_scan.broadcast_all(event).await;
                    }
                }
            });
        }

        // Heartbeat cleanup for stale connections (network drop / crashed tab)
        {
            let presence_clone = presence.clone();
            let hub_for_cleanup = hub.clone();
            tokio::spawn(async move {
                let stale_after = std::time::Duration::from_secs(75);
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                    let now = chrono::Utc::now();

                    let mut stale_conns = Vec::new();
                    for entry in hub_for_cleanup.heartbeats.iter() {
                        let last = *entry.value();
                        let elapsed = now
                            .signed_duration_since(last)
                            .to_std()
                            .unwrap_or_default();
                        if elapsed >= stale_after {
                            stale_conns.push(*entry.key());
                        }
                    }

                    for conn_id in stale_conns {
                        let user_id = hub_for_cleanup
                            .connections
                            .iter()
                            .find_map(|kv| {
                                if kv.value().contains(&conn_id) {
                                    Some(kv.key().clone())
                                } else {
                                    None
                                }
                            });

                        let Some(user_id) = user_id else {
                            hub_for_cleanup.heartbeats.remove(&conn_id);
                            continue;
                        };

                        hub_for_cleanup.unregister_connection(&user_id, &conn_id);

                        if let Some(new_status) =
                            presence_clone.remove_connection(&user_id, &conn_id.to_string())
                        {
                            if new_status
                                == crate::services::presence_service::PresenceStatus::Offline
                            {
                                let offline_event =
                                    crate::ws::protocol::ServerEvent::UserOffline {
                                        user_id: user_id.clone(),
                                        status: "offline".to_string(),
                                    };
                                hub_for_cleanup.broadcast_all(offline_event).await;

                                let presence_event =
                                    crate::ws::protocol::ServerEvent::PresenceUpdated {
                                        user_id: user_id.clone(),
                                        status: "offline".to_string(),
                                        last_activity: chrono::Utc::now().to_rfc3339(),
                                    };
                                hub_for_cleanup.broadcast_all(presence_event).await;
                            }
                        }
                    }
                }
            });
        }

        // Create typing service
        let typing_service = Arc::new(TypingService::new());

        // Message service with a dummy repo (will be replaced when MongoDB is connected)
        // For now, create a placeholder that won't be used until MongoDB is configured
        let message_service = Arc::new(MessageService::new(MessageRepo::new_placeholder()));

        Self {
            db,
            config,
            jwt_service,
            mongo_db: None,
            hub,
            message_service,
            presence,
            typing_service,
        }
    }

    pub fn with_mongodb(mut self, mongo_db: Database) -> Self {
        // Create the real message repo with MongoDB
        let message_repo = MessageRepo::new(mongo_db.collection("messages"));
        self.message_service = Arc::new(MessageService::new(message_repo));
        self.mongo_db = Some(mongo_db);
        self
    }
}

// Implement FromRef to allow extractors to get AppState from Arc<AppState>
impl FromRef<Arc<AppState>> for AppState {
    fn from_ref(app_state: &Arc<AppState>) -> Self {
        app_state.as_ref().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn app_state_new_initializes_services_without_mongo() {
        // crée un pool PostgreSQL paresseux (ne se connecte pas réellement)
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://epitalk:Epitalk94!@localhost:5432/epitalk")
            .expect("lazy pool");

        let cfg = Config {
            database_url: "postgres://epitalk:Epitalk94!@localhost:5432/epitalk".into(),
            jwt_secret: "test-secret-min-32-chars--------------".into(),
            jwt_expiration_hours: 24,
            port: 3000,
            upload_dir: "./uploads".into(),
        };

        let state = AppState::new(pool, cfg.clone());

        assert!(state.mongo_db.is_none());
        assert_eq!(state.config.port, 3000);
        assert_eq!(state.config.jwt_expiration_hours, 24);

        // FromRef doit renvoyer un clone équivalent
        let arc = Arc::new(state);
        let recovered = AppState::from_ref(&arc);
        assert_eq!(recovered.config.port, 3000);
        assert_eq!(recovered.config.jwt_secret, cfg.jwt_secret);
    }
}
