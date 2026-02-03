use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::game::Player;

const DATA_DIR: &str = "data/players";

/// Ensures the data directory exists
fn ensure_data_dir() -> Result<PathBuf> {
    let path = PathBuf::from(DATA_DIR);
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

/// Get the file path for a player's save file
fn get_player_path(player_id: &Uuid) -> PathBuf {
    PathBuf::from(DATA_DIR).join(format!("{}.json", player_id))
}

/// Save a player's state to disk
pub fn save_player(player: &Player) -> Result<()> {
    ensure_data_dir()?;
    let path = get_player_path(&player.id);
    let json = serde_json::to_string_pretty(player)?;
    fs::write(&path, json)?;
    tracing::debug!("Saved player {} to {:?}", player.id, path);
    Ok(())
}

/// Load a player's state from disk
pub fn load_player(player_id: &Uuid) -> Result<Option<Player>> {
    let path = get_player_path(player_id);
    if !path.exists() {
        return Ok(None);
    }
    let json = fs::read_to_string(&path)?;
    let player: Player = serde_json::from_str(&json)?;
    tracing::debug!("Loaded player {} from {:?}", player_id, path);
    Ok(Some(player))
}

/// Delete a player's save file
pub fn delete_player(player_id: &Uuid) -> Result<()> {
    let path = get_player_path(player_id);
    if path.exists() {
        fs::remove_file(&path)?;
        tracing::debug!("Deleted player {} save file", player_id);
    }
    Ok(())
}

/// List all saved player IDs
pub fn list_saved_players() -> Result<Vec<Uuid>> {
    ensure_data_dir()?;
    let path = PathBuf::from(DATA_DIR);
    let mut players = Vec::new();
    
    for entry in fs::read_dir(&path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        if name.ends_with(".json") {
            if let Some(id_str) = name.strip_suffix(".json") {
                if let Ok(id) = Uuid::parse_str(id_str) {
                    players.push(id);
                }
            }
        }
    }
    
    Ok(players)
}

/// Auto-save interval tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSaveConfig {
    pub enabled: bool,
    pub interval_choices: u32, // Save every N choices
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_choices: 3, // Auto-save every 3 choices
        }
    }
}
