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
use crate::endings::{check_for_ending, EndingResponse, EndingType};
use crate::game::{GameState, NarrativeMoment, Player};
use crate::llm::LlmClient;
use crate::persistence;

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
        .route("/api/game/load/{player_id}", get(load_game))
        .route("/api/game/save/{player_id}", post(save_game))
        .route("/api/game/list", get(list_saves))
        .route("/api/game/{player_id}", get(get_game_state))
        .route("/api/game/{player_id}/start", post(start_narrative))
        .route("/api/game/{player_id}/choice", post(make_choice))
        .route("/api/game/{player_id}/reset", post(reset_loop))
        .route("/api/game/{player_id}/ending", get(check_ending))
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

    // Auto-save new player
    if let Err(e) = persistence::save_player(&player) {
        tracing::warn!("Failed to auto-save new player: {}", e);
    }

    Json(NewGameResponse {
        player,
        message: "Welcome to the loop. You've been here before, even if you don't remember."
            .to_string(),
    })
}

#[derive(Serialize)]
struct LoadGameResponse {
    player: Player,
    message: String,
    found: bool,
}

async fn load_game(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> Result<Json<LoadGameResponse>, StatusCode> {
    // Try to load from disk
    match persistence::load_player(&player_id) {
        Ok(Some(player)) => {
            // Add to in-memory state
            let mut game = state.game.write().await;
            game.players.insert(player.id, player.clone());

            Ok(Json(LoadGameResponse {
                player,
                message: "I remember you... welcome back to the loop.".to_string(),
                found: true,
            }))
        }
        Ok(None) => {
            // Check if in memory
            let game = state.game.read().await;
            if let Some(player) = game.get_player(&player_id) {
                Ok(Json(LoadGameResponse {
                    player: player.clone(),
                    message: "You never left the loop.".to_string(),
                    found: true,
                }))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(e) => {
            tracing::error!("Failed to load player: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize)]
struct SaveGameResponse {
    success: bool,
    message: String,
}

async fn save_game(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> Result<Json<SaveGameResponse>, StatusCode> {
    let game = state.game.read().await;
    let player = game.get_player(&player_id).ok_or(StatusCode::NOT_FOUND)?;

    match persistence::save_player(player) {
        Ok(()) => Ok(Json(SaveGameResponse {
            success: true,
            message: "Your journey has been etched into the void.".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to save player: {}", e);
            Ok(Json(SaveGameResponse {
                success: false,
                message: format!("Failed to save: {}", e),
            }))
        }
    }
}

#[derive(Serialize)]
struct ListSavesResponse {
    saves: Vec<Uuid>,
}

async fn list_saves() -> Result<Json<ListSavesResponse>, StatusCode> {
    match persistence::list_saved_players() {
        Ok(saves) => Ok(Json(ListSavesResponse { saves })),
        Err(e) => {
            tracing::error!("Failed to list saves: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize)]
struct GameStateResponse {
    player: Player,
    current_moment: Option<NarrativeMoment>,
    ending: Option<EndingResponse>,
}

async fn get_game_state(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> Result<Json<GameStateResponse>, StatusCode> {
    let game = state.game.read().await;

    let player = game.get_player(&player_id).ok_or(StatusCode::NOT_FOUND)?;
    let current_moment = player.narrative_history.last().cloned();
    
    // Check for endings
    let ending = check_for_ending(player).map(|e| EndingResponse::from_player(player, e));

    Ok(Json(GameStateResponse {
        player: player.clone(),
        current_moment,
        ending,
    }))
}

#[derive(Serialize)]
struct NarrativeResponse {
    moment: NarrativeMoment,
    loop_number: u64,
    nihilism_score: i32,
    ending: Option<EndingResponse>,
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
    let (loop_number, nihilism_score, ending) = if let Some(p) = game.get_player_mut(&player_id) {
        p.narrative_history.push(moment.clone());
        let ending = check_for_ending(p).map(|e| EndingResponse::from_player(p, e));
        (p.current_loop.number, p.memory.nihilism_score, ending)
    } else {
        (1, 0, None)
    };

    Ok(Json(NarrativeResponse {
        moment,
        loop_number,
        nihilism_score,
        ending,
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

        // Determine if this is a "dark" choice (heuristics)
        let choice_lower = request.choice_text.to_lowercase();
        let id_lower = request.choice_id.to_lowercase();
        
        let is_dark = id_lower.contains("dark")
            || id_lower.contains("hurt")
            || id_lower.contains("ignore")
            || id_lower.contains("nihil")
            || id_lower.contains("cruel")
            || id_lower.contains("abandon")
            || choice_lower.contains("kill")
            || choice_lower.contains("abandon")
            || choice_lower.contains("nothing matters")
            || choice_lower.contains("don't care")
            || choice_lower.contains("meaningless")
            || choice_lower.contains("leave them")
            || choice_lower.contains("walk away");

        player.make_choice(&request.choice_id, is_dark);
        player.clone()
    };

    // Auto-save every 3 choices
    if player.memory.total_choices % 3 == 0 {
        if let Err(e) = persistence::save_player(&player) {
            tracing::warn!("Auto-save failed: {}", e);
        }
    }

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
    let (loop_number, nihilism_score, ending) = {
        let mut game = state.game.write().await;
        if let Some(p) = game.get_player_mut(&player_id) {
            p.narrative_history.push(moment.clone());
            let ending = check_for_ending(p).map(|e| EndingResponse::from_player(p, e));
            (p.current_loop.number, p.memory.nihilism_score, ending)
        } else {
            (1, 0, None)
        }
    };

    Ok(Json(NarrativeResponse {
        moment,
        loop_number,
        nihilism_score,
        ending,
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

    // Save after reset
    if let Err(e) = persistence::save_player(player) {
        tracing::warn!("Failed to save after reset: {}", e);
    }

    let message = format!(
        "Loop #{} begins. Despite everything... it's still you.",
        player.current_loop.number
    );

    Ok(Json(ResetResponse {
        player: player.clone(),
        message,
    }))
}

#[derive(Serialize)]
struct EndingCheckResponse {
    has_ending: bool,
    ending: Option<EndingResponse>,
}

async fn check_ending(
    State(state): State<AppState>,
    Path(player_id): Path<Uuid>,
) -> Result<Json<EndingCheckResponse>, StatusCode> {
    let game = state.game.read().await;
    let player = game.get_player(&player_id).ok_or(StatusCode::NOT_FOUND)?;

    let ending = check_for_ending(player).map(|e| EndingResponse::from_player(player, e));

    Ok(Json(EndingCheckResponse {
        has_ending: ending.is_some(),
        ending,
    }))
}
