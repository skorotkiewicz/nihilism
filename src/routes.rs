use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::config::Config;
use crate::game::{GameState, NarrativeMoment, Player};
use crate::llm::LlmClient;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub game: Arc<RwLock<GameState>>,
    pub llm: Arc<LlmClient>,
}

pub fn create_router(config: Config, game_state: Arc<RwLock<GameState>>) -> Router {
    let llm = Arc::new(LlmClient::new(config.clone()));

    let state = AppState {
        config,
        game: game_state,
        llm,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/game/new", post(new_game))
        .route("/api/game/{player_id}", get(get_game_state))
        .route("/api/game/{player_id}/start", post(start_narrative))
        .route("/api/game/{player_id}/choice", post(make_choice))
        .route("/api/game/{player_id}/reset", post(reset_loop))
        .layer(cors)
        .with_state(state)
}

async fn health_check() -> &'static str {
    "Nihilism game server is running. The loop continues..."
}

#[derive(Serialize)]
struct NewGameResponse {
    player: Player,
    message: String,
}

async fn new_game(State(state): State<AppState>) -> Json<NewGameResponse> {
    let mut game = state.game.write().await;
    let player = game.create_player();

    Json(NewGameResponse {
        player,
        message: "Welcome to the loop. You've been here before, even if you don't remember."
            .to_string(),
    })
}

#[derive(Serialize)]
struct GameStateResponse {
    player: Player,
    current_moment: Option<NarrativeMoment>,
}

async fn get_game_state(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> Result<Json<GameStateResponse>, StatusCode> {
    let game = state.game.read().await;

    let player = game.get_player(&player_id).ok_or(StatusCode::NOT_FOUND)?;

    let current_moment = player.narrative_history.last().cloned();

    Ok(Json(GameStateResponse {
        player: player.clone(),
        current_moment,
    }))
}

#[derive(Serialize)]
struct NarrativeResponse {
    moment: NarrativeMoment,
    loop_number: u64,
    nihilism_score: i32,
}

async fn start_narrative(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> Result<Json<NarrativeResponse>, StatusCode> {
    let game = state.game.read().await;
    let player = game.get_player(&player_id).ok_or(StatusCode::NOT_FOUND)?.clone();
    drop(game);

    let moment = state
        .llm
        .generate_narrative(&player, None)
        .await
        .map_err(|e| {
            tracing::error!("LLM error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut game = state.game.write().await;
    if let Some(p) = game.get_player_mut(&player_id) {
        p.narrative_history.push(moment.clone());
    }

    Ok(Json(NarrativeResponse {
        moment,
        loop_number: player.current_loop.number,
        nihilism_score: player.memory.nihilism_score,
    }))
}

#[derive(Deserialize)]
struct ChoiceRequest {
    choice_id: String,
    choice_text: String,
}

async fn make_choice(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
    Json(request): Json<ChoiceRequest>,
) -> Result<Json<NarrativeResponse>, StatusCode> {
    // First, update the player with the choice and get a copy
    let player = {
        let mut game = state.game.write().await;
        let player = game
            .get_player_mut(&player_id)
            .ok_or(StatusCode::NOT_FOUND)?;

        // Determine if this is a "dark" choice (simple heuristic for now)
        let is_dark = request.choice_id.contains("dark")
            || request.choice_id.contains("hurt")
            || request.choice_id.contains("ignore")
            || request.choice_id.contains("nihil")
            || request.choice_text.to_lowercase().contains("kill")
            || request.choice_text.to_lowercase().contains("abandon")
            || request.choice_text.to_lowercase().contains("nothing matters");

        player.make_choice(&request.choice_id, is_dark);
        player.clone()
    };

    // Generate the next narrative moment
    let choice = crate::game::Choice {
        id: request.choice_id,
        text: request.choice_text,
        consequence_hint: None,
    };

    let moment = state
        .llm
        .process_choice(&player, &choice)
        .await
        .map_err(|e| {
            tracing::error!("LLM error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Update the game state with the new moment
    let (loop_number, nihilism_score) = {
        let mut game = state.game.write().await;
        if let Some(p) = game.get_player_mut(&player_id) {
            p.narrative_history.push(moment.clone());
            (p.current_loop.number, p.memory.nihilism_score)
        } else {
            (1, 0)
        }
    };

    Ok(Json(NarrativeResponse {
        moment,
        loop_number,
        nihilism_score,
    }))
}

#[derive(Serialize)]
struct ResetResponse {
    player: Player,
    message: String,
}

async fn reset_loop(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> Result<Json<ResetResponse>, StatusCode> {
    let mut game = state.game.write().await;

    let player = game
        .get_player_mut(&player_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    player.reset_loop();

    let message = format!(
        "Loop #{} begins. Despite everything... it's still you.",
        player.current_loop.number
    );

    Ok(Json(ResetResponse {
        player: player.clone(),
        message,
    }))
}
