mod config;
mod endings;
mod game;
mod llm;
mod persistence;
mod routes;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::game::GameState;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nihilism=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    tracing::info!("Starting Nihilism game server...");
    tracing::info!("LLM API Base URL: {}", config.llm_base_url);

    let game_state = Arc::new(RwLock::new(GameState::new()));
    let app = routes::create_router(config.clone(), game_state);

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
