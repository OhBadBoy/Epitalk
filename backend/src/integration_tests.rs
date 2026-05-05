use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::connect_async;

use crate::config::Config;
use crate::routes;
use crate::state::AppState;
use crate::ws::ws_upgrade::ws_handler;

async fn start_server() -> (String, oneshot::Sender<()>) {
    if std::env::var("DATABASE_URL").is_err() {
        std::env::set_var(
            "DATABASE_URL",
            "postgres://epitalk:Epitalk94!@localhost:5433/epitalk",
        );
    }
    if std::env::var("MONGO_URL").is_err() {
        std::env::set_var("MONGO_URL", "mongodb://localhost:27017/epitalk_messages");
    }

    let config = Config::from_env().expect("config");
    let pg_pool = crate::db::postgres::create_pool(&config.database_url)
        .await
        .expect("postgres");

    let mongo_url = std::env::var("MONGO_URL").ok();
    let state = if let Some(mongo_url) = mongo_url {
        let mongo_client = mongodb::Client::with_uri_str(&mongo_url)
            .await
            .expect("mongo client");
        let mongo_db = mongo_client.database("epitalk_messages");
        Arc::new(AppState::new(pg_pool, config.clone()).with_mongodb(mongo_db))
    } else {
        Arc::new(AppState::new(pg_pool, config.clone()))
    };

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .nest("/api", routes::api_router())
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local addr");
    let base_url = format!("http://127.0.0.1:{}", addr.port());

    let (tx, rx) = oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = rx.await;
            })
            .await;
    });

    (base_url, tx)
}

async fn request_json(
    client: &reqwest::Client,
    method: reqwest::Method,
    url: &str,
    token: Option<&str>,
    body: Option<serde_json::Value>,
) -> (reqwest::StatusCode, serde_json::Value) {
    let mut req = client.request(method, url);
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    if let Some(body) = body {
        req = req.json(&body);
    }

    let resp = req.send().await.expect("request");
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    let json = if text.is_empty() {
        json!(null)
    } else {
        serde_json::from_str(&text).unwrap_or_else(|_| json!(null))
    };
    (status, json)
}

async fn register_user(
    client: &reqwest::Client,
    base_url: &str,
    email: &str,
    username: &str,
    password: &str,
) -> (String, String) {
    let url = format!("{}/api/auth/register", base_url);
    let (status, body) = request_json(
        client,
        reqwest::Method::POST,
        &url,
        None,
        Some(json!({
            "email": email,
            "username": username,
            "password": password,
        })),
    )
    .await;

    assert_eq!(status, reqwest::StatusCode::CREATED);
    let token = body["token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().to_string();
    (token, user_id)
}

async fn read_event_type(
    stream: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    expected: &[&str],
) -> serde_json::Value {
    let mut last = json!(null);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(12);

    while tokio::time::Instant::now() < deadline {
        let now = tokio::time::Instant::now();
        let remaining = deadline.saturating_duration_since(now);
        let slice = remaining.min(Duration::from_millis(900));

        match timeout(slice, stream.next()).await {
            Ok(Some(Ok(msg))) => {
                if msg.is_text() {
                    let txt = msg.into_text().unwrap();
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&txt) {
                        last = value.clone();
                        if let Some(t) = value.get("type").and_then(|v| v.as_str()) {
                            if expected.iter().any(|e| *e == t) {
                                return value;
                            }
                        }
                    }
                }
            }
            Ok(Some(Err(_))) => break,
            Ok(None) => break,
            Err(_) => continue,
        }
    }

    panic!("ws event not received; expected {:?}, last={}", expected, last);
}

async fn try_read_event_type(
    stream: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    expected: &[&str],
    max_wait: Duration,
) -> Option<serde_json::Value> {
    let deadline = tokio::time::Instant::now() + max_wait;

    while tokio::time::Instant::now() < deadline {
        let now = tokio::time::Instant::now();
        let remaining = deadline.saturating_duration_since(now);
        let slice = remaining.min(Duration::from_millis(120));

        match timeout(slice, stream.next()).await {
            Ok(Some(Ok(msg))) => {
                if msg.is_text() {
                    let txt = msg.into_text().ok()?;
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&txt) {
                        if let Some(t) = value.get("type").and_then(|v| v.as_str()) {
                            if expected.iter().any(|e| *e == t) {
                                return Some(value);
                            }
                        }
                    }
                }
            }
            Ok(Some(Err(_))) => return None,
            Ok(None) => return None,
            Err(_) => continue,
        }
    }

    None
}

async fn wait_for_presence_event_user(
    stream: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    expected_types: &[&str],
    expected_user_id: &str,
    max_wait: Duration,
) -> bool {
    let deadline = tokio::time::Instant::now() + max_wait;

    while tokio::time::Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        let Some(evt) = try_read_event_type(stream, expected_types, remaining).await else {
            return false;
        };

        let user_id = evt
            .get("payload")
            .and_then(|p| p.get("user_id"))
            .and_then(|v| v.as_str());

        if user_id == Some(expected_user_id) {
            return true;
        }
    }

    false
}

#[tokio::test]
async fn http_and_ws_core_flow() {
    if std::env::var("EPITALK_SKIP_SLOW_TESTS").ok().as_deref() == Some("1") {
        return;
    }

    let (base_url, shutdown) = start_server().await;
    let client = reqwest::Client::new();

    let suffix = uuid::Uuid::new_v4().to_string();
    let short = &suffix[..8];
    let password = "Password123!";

    let (token_a, user_a) = register_user(
        &client,
        &base_url,
        &format!("user-a-{}@example.test", suffix),
        &format!("user_a_{}", short),
        password,
    )
    .await;

    let (token_b, user_b) = register_user(
        &client,
        &base_url,
        &format!("user-b-{}@example.test", suffix),
        &format!("user_b_{}", short),
        password,
    )
    .await;

    let server_url = format!("{}/api/servers", base_url);
    let (status_server, server_body) = request_json(
        &client,
        reqwest::Method::POST,
        &server_url,
        Some(&token_a),
        Some(json!({ "name": "Integration Server" })),
    )
    .await;
    assert_eq!(status_server, reqwest::StatusCode::OK);
    let server_id = server_body["id"].as_str().unwrap().to_string();

    let channel_url = format!("{}/api/servers/{}/channels", base_url, server_id);
    let channel_name = format!("chan-{}", short);
    let (status_channel, channel_body) = request_json(
        &client,
        reqwest::Method::POST,
        &channel_url,
        Some(&token_a),
        Some(json!({ "name": channel_name })),
    )
    .await;
    assert_eq!(status_channel, reqwest::StatusCode::OK);
    let channel_id = channel_body["id"].as_str().unwrap().to_string();

    let invite_url = format!("{}/api/servers/{}/invites", base_url, server_id);
    let (status_invite, invite_body) = request_json(
        &client,
        reqwest::Method::POST,
        &invite_url,
        Some(&token_a),
        Some(json!({ "expires_in_hours": 24, "max_uses": 5 })),
    )
    .await;
    assert_eq!(status_invite, reqwest::StatusCode::OK);
    let invite_code = invite_body["code"].as_str().unwrap();

    let join_url = format!("{}/api/join", base_url);
    let (status_join, _) = request_json(
        &client,
        reqwest::Method::POST,
        &join_url,
        Some(&token_b),
        Some(json!({ "code": invite_code })),
    )
    .await;
    assert_eq!(status_join, reqwest::StatusCode::OK);

    let ws_url_a = format!("{}/ws?token={}", base_url.replace("http", "ws"), token_a);
    let ws_url_b = format!("{}/ws?token={}", base_url.replace("http", "ws"), token_b);

    let (mut ws_a, _) = connect_async(ws_url_a).await.expect("ws a");
    let (mut ws_b, _) = connect_async(ws_url_b).await.expect("ws b");

    let join = json!({ "type": "JoinChannel", "payload": { "channel_id": channel_id } });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(join.to_string()))
        .await
        .expect("join a");
    ws_b.send(tokio_tungstenite::tungstenite::Message::Text(join.to_string()))
        .await
        .expect("join b");

    let _ = read_event_type(&mut ws_a, &["UserJoined"]).await;
    let _ = read_event_type(&mut ws_b, &["UserJoined"]).await;

    let send = json!({
        "type": "MessageSend",
        "payload": { "channel_id": channel_id, "content": "hello" }
    });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(send.to_string()))
        .await
        .expect("send message");

    let evt = read_event_type(&mut ws_b, &["MessageNew"]).await;
    let msg_id = evt["payload"]["id"].as_str().unwrap().to_string();

    let edit = json!({
        "type": "MessageEdit",
        "payload": { "channel_id": channel_id, "message_id": msg_id, "content": "edited" }
    });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(edit.to_string()))
        .await
        .expect("edit message");
    let _ = read_event_type(&mut ws_b, &["MessageEdited"]).await;

    let reaction = json!({ "type": "ReactionAdd", "payload": { "message_id": msg_id, "emoji": ":thumbsup:" } });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(reaction.to_string()))
        .await
        .expect("reaction add");
    let _ = read_event_type(&mut ws_b, &["ReactionAdded"]).await;

    let reaction_remove = json!({ "type": "ReactionRemove", "payload": { "message_id": msg_id, "emoji": ":thumbsup:" } });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(reaction_remove.to_string()))
        .await
        .expect("reaction remove");
    let _ = read_event_type(&mut ws_b, &["ReactionRemoved"]).await;

    let typing_start = json!({ "type": "TypingStart", "payload": { "channel_id": channel_id } });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(typing_start.to_string()))
        .await
        .expect("typing start");
    let _ = read_event_type(&mut ws_b, &["TypingStart"]).await;

    let typing_stop = json!({ "type": "TypingStop", "payload": { "channel_id": channel_id } });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(typing_stop.to_string()))
        .await
        .expect("typing stop");
    let _ = read_event_type(&mut ws_b, &["TypingStop"]).await;

    let presence = json!({ "type": "PresenceSet", "payload": { "status": "idle" } });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(presence.to_string()))
        .await
        .expect("presence");
    let _ = read_event_type(&mut ws_b, &["PresenceUpdated"]).await;

    let dm_send = json!({
        "type": "DmSend",
        "payload": { "recipient_id": user_b, "content": "hello dm" }
    });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(dm_send.to_string()))
        .await
        .expect("dm send");
    let dm_event = read_event_type(&mut ws_b, &["DmNew"]).await;
    let dm_id = dm_event["payload"]["id"].as_str().unwrap().to_string();
    let conv_id = dm_event["payload"]["conversation_id"].as_str().unwrap().to_string();

    let join_dm_a = json!({ "type": "JoinDm", "payload": { "peer_id": user_b } });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(join_dm_a.to_string()))
        .await
        .expect("join dm a");

    let join_dm_b = json!({ "type": "JoinDm", "payload": { "peer_id": user_a } });
    ws_b.send(tokio_tungstenite::tungstenite::Message::Text(join_dm_b.to_string()))
        .await
        .expect("join dm b");

    tokio::time::sleep(Duration::from_millis(200)).await;

    let dm_edit = json!({
        "type": "DmEdit",
        "payload": { "conversation_id": conv_id, "message_id": dm_id, "content": "dm edited" }
    });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(dm_edit.to_string()))
        .await
        .expect("dm edit");
    let _ = read_event_type(&mut ws_b, &["DmEdited"]).await;

    let dm_delete = json!({
        "type": "DmDelete",
        "payload": { "conversation_id": conv_id, "message_id": dm_id }
    });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(dm_delete.to_string()))
        .await
        .expect("dm delete");
    let _ = read_event_type(&mut ws_b, &["DmDeleted"]).await;

    let delete_msg = json!({
        "type": "MessageDelete",
        "payload": { "channel_id": channel_id, "message_id": msg_id }
    });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(delete_msg.to_string()))
        .await
        .expect("delete message");
    let _ = read_event_type(&mut ws_b, &["MessageDeleted"]).await;

    let _ = ws_a.close(None).await;
    let _ = ws_b.close(None).await;

    let bans_url = format!("{}/api/servers/{}/members/bans", base_url, server_id);
    let (status_bans, _) = request_json(&client, reqwest::Method::GET, &bans_url, Some(&token_a), None).await;
    assert_eq!(status_bans, reqwest::StatusCode::OK);

    let ban_url = format!("{}/api/servers/{}/members/{}/ban", base_url, server_id, user_b);
    let (status_ban, _) = request_json(
        &client,
        reqwest::Method::POST,
        &ban_url,
        Some(&token_a),
        Some(json!({ "reason": "spam" })),
    )
    .await;
    assert_eq!(status_ban, reqwest::StatusCode::OK);

    let (status_unban, _) = request_json(
        &client,
        reqwest::Method::DELETE,
        &ban_url,
        Some(&token_a),
        None,
    )
    .await;
    assert_eq!(status_unban, reqwest::StatusCode::OK);

    let _ = shutdown.send(());
}

#[tokio::test]
async fn ws_presence_connect_and_disconnect_flow() {
    let (base_url, shutdown) = start_server().await;
    let client = reqwest::Client::new();

    let suffix = uuid::Uuid::new_v4().to_string();
    let short = &suffix[..8];
    let password = "Password123!";

    let (token_a, _user_a) = register_user(
        &client,
        &base_url,
        &format!("presence-a-{}@example.test", suffix),
        &format!("presence_a_{}", short),
        password,
    )
    .await;

    let (token_b, user_b) = register_user(
        &client,
        &base_url,
        &format!("presence-b-{}@example.test", suffix),
        &format!("presence_b_{}", short),
        password,
    )
    .await;

    let ws_url_a = format!("{}/ws?token={}", base_url.replace("http", "ws"), token_a);
    let ws_url_b = format!("{}/ws?token={}", base_url.replace("http", "ws"), token_b);

    let (mut ws_a, _) = connect_async(ws_url_a).await.expect("ws a");
    let (mut ws_b, _) = connect_async(ws_url_b).await.expect("ws b");

    assert!(
        wait_for_presence_event_user(
            &mut ws_a,
            &["UserOnline", "PresenceUpdated"],
            &user_b,
            Duration::from_secs(4)
        )
        .await,
        "expected online event for connected user"
    );

    let _ = ws_b.close(None).await;

    assert!(
        wait_for_presence_event_user(
            &mut ws_a,
            &["UserOffline", "PresenceUpdated"],
            &user_b,
            Duration::from_secs(4)
        )
        .await,
        "expected offline event for disconnected user"
    );

    let _ = ws_a.close(None).await;
    let _ = shutdown.send(());
}

#[tokio::test]
async fn ws_typing_start_is_throttled() {
    let (base_url, shutdown) = start_server().await;
    let client = reqwest::Client::new();

    let suffix = uuid::Uuid::new_v4().to_string();
    let short = &suffix[..8];
    let password = "Password123!";

    let (token_a, _user_a) = register_user(
        &client,
        &base_url,
        &format!("typing-a-{}@example.test", suffix),
        &format!("typing_a_{}", short),
        password,
    )
    .await;

    let (token_b, _user_b) = register_user(
        &client,
        &base_url,
        &format!("typing-b-{}@example.test", suffix),
        &format!("typing_b_{}", short),
        password,
    )
    .await;

    let server_url = format!("{}/api/servers", base_url);
    let (status_server, server_body) = request_json(
        &client,
        reqwest::Method::POST,
        &server_url,
        Some(&token_a),
        Some(json!({ "name": "Typing Integration" })),
    )
    .await;
    assert_eq!(status_server, reqwest::StatusCode::OK);
    let server_id = server_body["id"].as_str().unwrap().to_string();

    let channel_url = format!("{}/api/servers/{}/channels", base_url, server_id);
    let (status_channel, channel_body) = request_json(
        &client,
        reqwest::Method::POST,
        &channel_url,
        Some(&token_a),
        Some(json!({ "name": format!("typing-{}", short) })),
    )
    .await;
    assert_eq!(status_channel, reqwest::StatusCode::OK);
    let channel_id = channel_body["id"].as_str().unwrap().to_string();

    let invite_url = format!("{}/api/servers/{}/invites", base_url, server_id);
    let (status_invite, invite_body) = request_json(
        &client,
        reqwest::Method::POST,
        &invite_url,
        Some(&token_a),
        Some(json!({ "expires_in_hours": 24, "max_uses": 5 })),
    )
    .await;
    assert_eq!(status_invite, reqwest::StatusCode::OK);

    let join_url = format!("{}/api/join", base_url);
    let (status_join, _) = request_json(
        &client,
        reqwest::Method::POST,
        &join_url,
        Some(&token_b),
        Some(json!({ "code": invite_body["code"].as_str().unwrap() })),
    )
    .await;
    assert_eq!(status_join, reqwest::StatusCode::OK);

    let ws_url_a = format!("{}/ws?token={}", base_url.replace("http", "ws"), token_a);
    let ws_url_b = format!("{}/ws?token={}", base_url.replace("http", "ws"), token_b);

    let (mut ws_a, _) = connect_async(ws_url_a).await.expect("ws a");
    let (mut ws_b, _) = connect_async(ws_url_b).await.expect("ws b");

    let join = json!({ "type": "JoinChannel", "payload": { "channel_id": channel_id } });
    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(join.to_string()))
        .await
        .expect("join a");
    ws_b.send(tokio_tungstenite::tungstenite::Message::Text(join.to_string()))
        .await
        .expect("join b");

    let _ = read_event_type(&mut ws_a, &["UserJoined"]).await;
    let _ = read_event_type(&mut ws_b, &["UserJoined"]).await;

    let typing = json!({ "type": "TypingStart", "payload": { "channel_id": channel_id } });

    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(typing.to_string()))
        .await
        .expect("typing start #1");
    let first_evt = read_event_type(&mut ws_b, &["TypingStart"]).await;
    assert_eq!(
        first_evt["payload"]["channel_id"].as_str(),
        Some(channel_id.as_str())
    );

    ws_a.send(tokio_tungstenite::tungstenite::Message::Text(typing.to_string()))
        .await
        .expect("typing start #2");

    let maybe_second = try_read_event_type(&mut ws_b, &["TypingStart"], Duration::from_millis(500)).await;
    assert!(
        maybe_second.is_none(),
        "second TypingStart should be throttled within 500ms"
    );

    let _ = ws_a.close(None).await;
    let _ = ws_b.close(None).await;
    let _ = shutdown.send(());
}
