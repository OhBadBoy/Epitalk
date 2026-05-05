//! EpiTalk Backend - Main entry point
//! REST API for authentication, RBAC, servers, channels, members & invites

mod auth;
mod config;
mod db;
mod error;
mod models;
mod repositories;
mod routes;
mod services;
mod state;
mod ws;

#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod tests;

use axum::Router;
use axum::routing::get;
use std::net::SocketAddr;
use std::process::Command;
use std::sync::Arc;
use tokio::process::Command as TokioCommand;
use tokio::time::{sleep, Duration};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::ws::ws_upgrade::ws_handler;

const CHILD_MODE_ENV: &str = "EPITALK_CHILD_PROCESS";
const DISABLE_SUPERVISOR_ENV: &str = "EPITALK_DISABLE_SUPERVISOR";

fn should_use_supervisor() -> bool {
    let child_mode = std::env::var(CHILD_MODE_ENV).ok().as_deref() == Some("1");
    let disable_supervisor = std::env::var(DISABLE_SUPERVISOR_ENV).ok().as_deref() == Some("1");
    !child_mode && !disable_supervisor
}

fn configured_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
    .unwrap_or(3001)
}

fn kill_port_occupant(port: u16) {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("netstat")
            .args(["-ano", "-p", "tcp"])
            .output();

        let Ok(output) = output else {
            eprintln!("[supervisor] cannot inspect TCP listeners with netstat");
            return;
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let current_pid = std::process::id();
        for line in stdout.lines() {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 5 || cols[3] != "LISTENING" {
                continue;
            }

            let local_addr = cols[1];
            let is_target_port = local_addr.ends_with(&format!(":{port}"))
                || local_addr.ends_with(&format!("]:{port}"));
            if !is_target_port {
                continue;
            }

            let Ok(pid) = cols[4].parse::<u32>() else {
                continue;
            };
            if pid == current_pid {
                continue;
            }

            let _ = Command::new("taskkill")
                .args(["/F", "/PID", &pid.to_string()])
                .status();
            eprintln!("[supervisor] killed PID {} on port {}", pid, port);
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("sh")
            .args([
                "-c",
                &format!("lsof -ti tcp:{port} 2>/dev/null || fuser -n tcp {port} 2>/dev/null"),
            ])
            .output();

        let Ok(output) = output else {
            eprintln!("[supervisor] cannot inspect port occupants");
            return;
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let current_pid = std::process::id().to_string();
        for pid in stdout.split_whitespace() {
            if pid == current_pid {
                continue;
            }

            let _ = Command::new("kill").args(["-9", pid]).status();
            eprintln!("[supervisor] killed PID {} on port {}", pid, port);
        }
    }
}

fn run_supervisor() -> anyhow::Result<()> {
    let port = configured_port();
    let exe = std::env::current_exe()?;
    let args: Vec<String> = std::env::args().skip(1).collect();

    eprintln!(
        "[supervisor] active (auto-restart enabled) on port {} | set {}=1 to disable",
        port, DISABLE_SUPERVISOR_ENV
    );

    loop {
        kill_port_occupant(port);

        let status = Command::new(&exe)
            .args(&args)
            .env(CHILD_MODE_ENV, "1")
            .status()?;

        if status.success() {
            eprintln!("[supervisor] backend exited cleanly, stopping supervisor");
            return Ok(());
        }

        eprintln!(
            "[supervisor] backend crashed (status: {:?}), restarting in 1 second...",
            status.code()
        );
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load env first so the supervisor can read PORT from .env.
    dotenvy::dotenv().ok();

    if should_use_supervisor() {
        return run_supervisor();
    }

    // Initialize tracing in child/runtime mode only
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "epitalk_backend=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    run_backend().await
}

async fn run_backend() -> anyhow::Result<()> {
    // Load configuration
    let config = config::Config::from_env()?;
    tracing::info!("Configuration loaded");

    // Initialize PostgreSQL
    let pg_pool = db::postgres::create_pool(&config.database_url).await?;
    tracing::info!("PostgreSQL connected");

    // Initialize MongoDB (optional - accept both MONGO_URL and MONGODB_URI)
    let mongo_url = std::env::var("MONGO_URL").or_else(|_| std::env::var("MONGODB_URI"));

    let state = if let Ok(mongo_url) = mongo_url {
        tracing::info!("Connecting to MongoDB...");
        let mongo_client = mongodb::Client::with_uri_str(&mongo_url).await?;
        let mongo_db = mongo_client.database("epitalk_messages");
        tracing::info!("MongoDB connected");

        let base_state = state::AppState::new(pg_pool, config.clone());
        Arc::new(base_state.with_mongodb(mongo_db))
    } else {
        tracing::warn!("MONGO_URL/MONGODB_URI not set, WebSocket messages will not be persisted");
        Arc::new(state::AppState::new(pg_pool, config.clone()))
    };

    // Build router
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .nest("/api", routes::api_router())
        .nest_service("/uploads", ServeDir::new(&config.upload_dir))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Server listening on {}", addr);

    // Try to bind to the address. If the port is already in use, attempt a best-effort
    // cleanup by finding the process that listens on the port and terminating it,
    // then retry binding once. This helps `cargo run` behave like a restart when a
    // previous background run left the process listening.
    let mut try_bind = || async {
        tokio::net::TcpListener::bind(addr).await
    };

    let listener = match try_bind().await {
        Ok(l) => l,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            tracing::warn!(port = config.port, "Port already in use, trying to detect and kill the process holding it");
            // Attempt to detect the pid using lsof (works on macOS / Linux with lsof installed)
            if let Ok(output) = TokioCommand::new("lsof")
                .args(&["-t", &format!("-iTCP:{}", config.port), "-sTCP:LISTEN"])
                .output()
                .await
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if let Ok(pid) = line.trim().parse::<i32>() {
                            tracing::info!(pid, "Killing leftover process on port");
                            // Best-effort: first SIGTERM, then SIGKILL if still alive
                            let _ = TokioCommand::new("kill").arg(pid.to_string()).status().await;
                        }
                    }
                    // Give the OS a moment to free the port
                    sleep(Duration::from_millis(500)).await;
                    match try_bind().await {
                        Ok(l2) => l2,
                        Err(e2) => {
                            tracing::error!(error = %e2, "Failed to bind after killing process");
                            return Err(e2.into());
                        }
                    }
                } else {
                    tracing::error!(stderr = %String::from_utf8_lossy(&output.stderr), "lsof failed to list PIDs");
                    return Err(e.into());
                }
            } else {
                tracing::error!("Failed to run lsof to detect process occupying the port");
                return Err(e.into());
            }
        }
        Err(e) => return Err(e.into()),
    };

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod main_tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn configured_port_reads_env() {
        let _guard = env_lock().lock().expect("env lock");
        std::env::remove_var("PORT");
        assert_eq!(configured_port(), 3001);

        std::env::set_var("PORT", "4040");
        assert_eq!(configured_port(), 4040);
        std::env::remove_var("PORT");
    }

    #[test]
    fn should_use_supervisor_respects_flags() {
        let _guard = env_lock().lock().expect("env lock");
        std::env::remove_var(CHILD_MODE_ENV);
        std::env::remove_var(DISABLE_SUPERVISOR_ENV);
        assert!(should_use_supervisor());

        std::env::set_var(CHILD_MODE_ENV, "1");
        assert!(!should_use_supervisor());
        std::env::remove_var(CHILD_MODE_ENV);

        std::env::set_var(DISABLE_SUPERVISOR_ENV, "1");
        assert!(!should_use_supervisor());
        std::env::remove_var(DISABLE_SUPERVISOR_ENV);
    }
}
