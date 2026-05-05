use dashmap::{DashMap, DashSet};
use tokio::sync::mpsc::UnboundedSender;
use axum::extract::ws::Message;
use uuid::Uuid;
use chrono::{Utc, DateTime};

use crate::ws::protocol::ServerEvent;

pub type ConnId = Uuid;
pub type UserId = String;
pub type RoomId = String;

#[derive(Clone)]
pub struct Hub {
    /// user_id -> set of connection ids
    pub connections: DashMap<UserId, DashSet<ConnId>>,

    /// conn_id -> websocket sender
    pub sockets: DashMap<ConnId, UnboundedSender<Message>>,

    /// room_id -> set of connection ids
    pub rooms: DashMap<RoomId, DashSet<ConnId>>,

    /// last heartbeat per connection
    pub heartbeats: DashMap<ConnId, DateTime<Utc>>,
}

impl Hub {
    pub fn new() -> Self {
        Self {
            connections: DashMap::new(),
            sockets: DashMap::new(),
            rooms: DashMap::new(),
            heartbeats: DashMap::new(),
        }
    }

    // ---------------------------------------------------------
    // CONNECTION MANAGEMENT
    // ---------------------------------------------------------

    pub fn register_connection(
        &self,
        user_id: &UserId,
        conn_id: ConnId,
        sender: UnboundedSender<Message>,
    ) {
        self.sockets.insert(conn_id, sender);

        let set = self.connections.entry(user_id.clone()).or_default();
        set.insert(conn_id);

        self.heartbeats.insert(conn_id, Utc::now());
    }

    pub fn unregister_connection(&self, user_id: &UserId, conn_id: &ConnId) {
        if let Some(set) = self.connections.get_mut(user_id) {
            set.remove(conn_id);
            if set.is_empty() {
                self.connections.remove(user_id);
            }
        }

        self.sockets.remove(conn_id);
        self.heartbeats.remove(conn_id);

        // remove from all rooms
        for room in self.rooms.iter_mut() {
            room.value().remove(conn_id);
        }
    }

    pub fn heartbeat(&self, conn_id: &ConnId) {
        self.heartbeats.insert(*conn_id, Utc::now());
    }

    /// Force-disconnect all active connections for a user.
    ///
    /// Used by moderation flows (e.g. ban) to apply effects immediately.
    pub fn disconnect_user(&self, user_id: &UserId) {
        let conn_ids: Vec<ConnId> = self
            .connections
            .get(user_id)
            .map(|set| set.iter().map(|id| *id).collect())
            .unwrap_or_default();

        if conn_ids.is_empty() {
            return;
        }

        for conn_id in conn_ids {
            if let Some(sender) = self.sockets.get(&conn_id) {
                let _ = sender.send(Message::Close(None));
            }

            self.sockets.remove(&conn_id);
            self.heartbeats.remove(&conn_id);

            for room in self.rooms.iter_mut() {
                room.value().remove(&conn_id);
            }
        }

        self.connections.remove(user_id);
    }

    /// Send an event to all active connections of a given user.
    pub fn send_event_to_user(&self, user_id: &UserId, event: ServerEvent) {
        let Some(conn_set) = self.connections.get(user_id) else {
            return;
        };

        let json = match serde_json::to_string(&event) {
            Ok(v) => v,
            Err(_) => return,
        };

        for conn_id in conn_set.iter() {
            if let Some(sender) = self.sockets.get(&*conn_id) {
                let _ = sender.send(Message::Text(json.clone().into()));
            }
        }
    }

    // ---------------------------------------------------------
    // ROOM MANAGEMENT
    // ---------------------------------------------------------

    pub fn join_room(&self, room_id: &RoomId, conn_id: ConnId) {
        let room = self.rooms.entry(room_id.clone()).or_default();
        room.insert(conn_id);
    }

    pub fn leave_room(&self, room_id: &RoomId, conn_id: &ConnId) {
        if let Some(room) = self.rooms.get(room_id) {
            room.remove(conn_id);
        }
    }

    // ---------------------------------------------------------
    // BROADCAST TO ROOM
    // ---------------------------------------------------------

    pub async fn broadcast_room(&self, room_id: &RoomId, event: ServerEvent) {
        let json = serde_json::to_string(&event).unwrap();

        if let Some(room) = self.rooms.get(room_id) {
            for conn_id in room.iter() {
                if let Some(sender) = self.sockets.get(&*conn_id) {
                    let _ = sender.send(Message::Text(json.clone().into()));
                }
            }
        }
    }

    // ---------------------------------------------------------
    // BROADCAST TO ALL
    // ---------------------------------------------------------

    pub async fn broadcast_all(&self, event: ServerEvent) {
        let json = serde_json::to_string(&event).unwrap();

        for sender in self.sockets.iter() {
            let _ = sender.value().send(Message::Text(json.clone().into()));
        }
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::unbounded_channel;

    #[tokio::test]
    async fn join_leave_and_broadcast_room_sends_messages() {
        let hub = Hub::new();
        let (tx, mut rx) = unbounded_channel();
        let user_id = "user-1".to_string();
        let conn_id = Uuid::new_v4();
        let room_id = "room-1".to_string();

        hub.register_connection(&user_id, conn_id, tx);
        hub.join_room(&room_id, conn_id);

        let event = ServerEvent::Pong;
        hub.broadcast_room(&room_id, event).await;

        // On doit recevoir au moins un message texte
        let msg = rx.try_recv().expect("expected a message");
        match msg {
            Message::Text(text) => {
                let v: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(v["type"], "Pong");
            }
            other => panic!("unexpected message: {:?}", other),
        }

        hub.leave_room(&room_id, &conn_id);

        // Après avoir quitté la room, la room peut encore exister mais ne doit plus contenir la connexion
        let is_still_in_room = hub
            .rooms
            .get(&room_id)
            .map(|room| room.contains(&conn_id))
            .unwrap_or(false);

        assert!(!is_still_in_room);
    }

    #[tokio::test]
    async fn disconnect_user_closes_and_removes_connections() {
        let hub = Hub::new();
        let (tx1, mut rx1) = unbounded_channel();
        let (tx2, mut rx2) = unbounded_channel();

        let user_id = "user-1".to_string();
        let conn_1 = Uuid::new_v4();
        let conn_2 = Uuid::new_v4();

        hub.register_connection(&user_id, conn_1, tx1);
        hub.register_connection(&user_id, conn_2, tx2);

        let room_id = "room-1".to_string();
        hub.join_room(&room_id, conn_1);
        hub.join_room(&room_id, conn_2);

        hub.disconnect_user(&user_id);

        assert!(!hub.connections.contains_key(&user_id));
        assert!(!hub.sockets.contains_key(&conn_1));
        assert!(!hub.sockets.contains_key(&conn_2));

        if let Some(room) = hub.rooms.get(&room_id) {
            assert!(!room.contains(&conn_1));
            assert!(!room.contains(&conn_2));
        }

        let msg1 = rx1.try_recv().expect("expected close frame on conn_1");
        let msg2 = rx2.try_recv().expect("expected close frame on conn_2");

        match msg1 {
            Message::Close(_) => {}
            other => panic!("unexpected message on conn_1: {:?}", other),
        }

        match msg2 {
            Message::Close(_) => {}
            other => panic!("unexpected message on conn_2: {:?}", other),
        }
    }
}
