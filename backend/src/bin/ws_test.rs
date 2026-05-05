use tokio::time::{sleep, Duration};
use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use uuid::Uuid;
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    email: String,
    username: String,
    iat: i64,
    exp: i64,
}

fn make_token(secret: &str, user_id: Uuid, email: &str, username: &str) -> String {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        username: username.to_string(),
        iat: now.timestamp(),
        exp: (now + chrono::Duration::hours(1)).timestamp(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .expect("failed to encode token")
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, DecodingKey, Validation};

    #[test]
    fn make_token_roundtrip() {
        let secret = "test-secret-key-at-least-32-chars";
        let user_id = Uuid::new_v4();
        let token = make_token(secret, user_id, "a@example.test", "alice");

        let data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .expect("decode token");

        assert_eq!(data.claims.sub, user_id);
        assert_eq!(data.claims.email, "a@example.test");
        assert_eq!(data.claims.username, "alice");
    }
}

async fn run_client(
    name: &str,
    token: String,
    duration_secs: u64,
    assert_tx: tokio::sync::mpsc::UnboundedSender<String>,
    mut control_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
) {
    let ws_url = format!("ws://127.0.0.1:3001/ws?token={}", token);
    println!("[{}] Connecting to {}", name, ws_url);
    let (ws_stream, resp) = connect_async(ws_url).await.expect("Failed to connect");
    println!("[{}] Connected - HTTP status: {}", name, resp.status());

    let (mut write, mut read) = ws_stream.split();

    // Spawn a reader
    let reader_name = name.to_string();
    let mut read = read;
    let tx = assert_tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(m) => {
                    if m.is_text() {
                        let txt = m.into_text().unwrap();
                        println!("[{}] RECV: {}", reader_name, txt);
                        // forward to assertion channel (ignore send errors)
                        let _ = tx.send(txt);
                    } else if m.is_close() {
                        println!("[{}] Connection closed by server", reader_name);
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("[{}] read error: {}", reader_name, e);
                    break;
                }
            }
        }
    });

    // control commands are handled in the main loop via `select!` on control_rx

    // Send periodic Ping events as JSON messages to refresh presence
    let ping = r#"{"type":"Ping"}"#;
    let mut elapsed = 0u64;
    // Join a known channel so we can validate history persistence
    let channel_id = "17b4d448-1de6-48e5-8b35-de519cba71b1";
    let join = serde_json::json!({ "type": "JoinChannel", "payload": { "channel_id": channel_id } });
    if let Err(e) = write.send(tokio_tungstenite::tungstenite::Message::Text(join.to_string())).await {
        eprintln!("[{}] write error (JoinChannel): {}", name, e);
    } else {
        println!("[{}] Sent JoinChannel to {}", name, channel_id);
    }

    // If this client is A-1, send a test message to a known channel to validate history persistence
    if name == "A-1" {
        // Replace this channel id with a valid channel in your DB if needed
        let channel_id = "17b4d448-1de6-48e5-8b35-de519cba71b1";
        let msg = serde_json::json!({
            "type": "MessageSend",
            "payload": {
                "channel_id": channel_id,
                "content": "test message from ws_test",
                "reply_to": null
            }
        });
        if let Err(e) = write.send(tokio_tungstenite::tungstenite::Message::Text(msg.to_string())).await {
            eprintln!("[{}] write error (MessageSend): {}", name, e);
        } else {
            println!("[{}] Sent test MessageSend to channel {}", name, channel_id);
        }
    }
    while elapsed < duration_secs {
        tokio::select! {
            biased;
            maybe_cmd = control_rx.recv() => {
                if let Some(cmd) = maybe_cmd {
                    if cmd == "close" {
                        println!("[{}] control loop: sending Close", name);
                        let _ = write.send(tokio_tungstenite::tungstenite::Message::Close(None)).await;
                        break;
                    }
                } else {
                    // channel closed
                }
            }
            _ = sleep(Duration::from_secs(5)) => {
                if let Err(e) = write.send(tokio_tungstenite::tungstenite::Message::Text(ping.into())).await {
                    eprintln!("[{}] write error: {}", name, e);
                    break;
                }
                elapsed += 5;
            }
        }
    }

    // wait briefly after close so server has time to process the close handshake
    sleep(Duration::from_millis(500)).await;
    println!("[{}] Finished", name);
}

#[tokio::main]
async fn main() {
    // read secret from env or fallback to example
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        "super_secret_jwt_key_change_in_production_min_32_chars".to_string()
    });

    // Two user ids: same user (multi-conn) and another user
    let user_a = Uuid::new_v4();
    let user_b = Uuid::new_v4();

    println!("Using JWT secret: {}", if secret.len() > 6 { "(redacted)" } else { &secret });

    let token_a1 = make_token(&secret, user_a, "a@example.test", "userA-conn1");
    let token_a2 = make_token(&secret, user_a, "a@example.test", "userA-conn2");
    let token_b = make_token(&secret, user_b, "b@example.test", "userB");

    // Run three clients: two for same user, one for different user
    let (assert_tx, mut assert_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let (ctrl_a1_tx, ctrl_a1_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let (ctrl_a2_tx, ctrl_a2_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let (ctrl_b_tx, ctrl_b_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let c1 = tokio::spawn(run_client("A-1", token_a1, 60, assert_tx.clone(), ctrl_a1_rx));
    // small stagger
    sleep(Duration::from_millis(200)).await;
    let c2 = tokio::spawn(run_client("A-2", token_a2, 60, assert_tx.clone(), ctrl_a2_rx));
    sleep(Duration::from_millis(200)).await;
    let c3 = tokio::spawn(run_client("B", token_b, 60, assert_tx.clone(), ctrl_b_rx));

    // Wait a bit for clients to exchange messages
    // Expectation 1: The test message sent by A-1 should be observed as a MessageNew by at least one client
    use tokio::time::timeout;
    let mut found_message = false;
    let expected_sub = "test message from ws_test";
    let wait_res = timeout(Duration::from_secs(20), async {
        while let Some(msg) = assert_rx.recv().await {
            if msg.contains(expected_sub) {
                println!("[ws_test] Assertion: observed test message in server broadcast/historic message");
                found_message = true;
                break;
            }
        }
    }).await;

    if wait_res.is_err() {
        eprintln!("[ws_test] ERROR: did not observe test message within timeout");
        // attempt to still wait for tasks to finish but exit with error
        let _ = tokio::join!(c1, c2, c3);
        std::process::exit(2);
    }

    // Expectation 2: PresenceUpdated for user_a with status "online" should be broadcast
    // We'll wait a short period and check any presence updated payload mentioning the user's UUID
    let user_a_str = user_a.to_string();
    let mut found_presence_online = false;
    let wait_res2 = timeout(Duration::from_secs(16), async {
        while let Some(msg) = assert_rx.recv().await {
            if msg.contains("PresenceUpdated") && msg.contains(&user_a_str) && msg.contains("online") {
                println!("[ws_test] Assertion: observed PresenceUpdated online for user_a");
                found_presence_online = true;
                break;
            }
        }
    }).await;

    if wait_res2.is_err() {
        eprintln!("[ws_test] WARNING: did not observe PresenceUpdated online event for user_a within timeout");
    }

    // Now: multi-conn assertions
    // Step 1: close A-2 gracefully and assert user A remains online
    println!("[ws_test] Closing A-2 (should keep user A online)");
    let _ = ctrl_a2_tx.send("close".to_string());

    // wait briefly for events
    let mut saw_offline = false;
    let mut saw_online_after_close = false;
    let wait_res3 = timeout(Duration::from_secs(10), async {
        while let Some(msg) = assert_rx.recv().await {
            if msg.contains("PresenceUpdated") && msg.contains(&user_a.to_string()) && msg.contains("offline") {
                saw_offline = true;
                break;
            }
            if msg.contains("PresenceUpdated") && msg.contains(&user_a.to_string()) && msg.contains("online") {
                saw_online_after_close = true;
            }
        }
    }).await;

    if wait_res3.is_err() {
        // no offline observed yet, which is expected (user should remain online)
        println!("[ws_test] No offline observed after closing one conn (expected)");
    }

    if saw_offline {
        eprintln!("[ws_test] FAIL: user went offline after closing only one of multiple connections");
        let _ = tokio::join!(c1, c2, c3);
        std::process::exit(4);
    }

    println!("[ws_test] Now closing A-1 (last connection) to assert offline event");
    let _ = ctrl_a1_tx.send("close".to_string());

    // Wait for offline event
    let mut saw_offline_final = false;
    let wait_res4 = timeout(Duration::from_secs(16), async {
        while let Some(msg) = assert_rx.recv().await {
            if msg.contains("PresenceUpdated") && msg.contains(&user_a.to_string()) && msg.contains("offline") {
                saw_offline_final = true;
                break;
            }
        }
    }).await;

    if wait_res4.is_err() || !saw_offline_final {
        eprintln!("[ws_test] FAIL: did not observe offline event after closing last connection");
        let _ = tokio::join!(c1, c2, c3);
        std::process::exit(5);
    }

    println!("[ws_test] SUCCESS: multi-conn presence assertions passed");

    let _ = tokio::join!(c1, c2, c3);
    println!("All clients finished");

    if !found_message {
        eprintln!("[ws_test] FAIL: Message persistence/broadcast assertion failed");
        std::process::exit(3);
    }

    println!("[ws_test] SUCCESS: basic assertions passed (message persisted/broadcast)");
}
