use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A single choice the player can make
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice {
    pub id: String,
    pub text: String,
    pub consequence_hint: Option<String>,
}

/// A narrative moment in the game
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NarrativeMoment {
    pub id: Uuid,
    pub text: String,
    pub speaker: Option<String>,
    pub mood: String, // "hopeful", "nihilistic", "neutral", "dark", "transcendent"
    pub choices: Vec<Choice>,
    pub timestamp: DateTime<Utc>,
}

/// Represents a single loop iteration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Loop {
    pub number: u64,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub choices_made: Vec<String>,
    pub outcome: Option<String>,
}

/// Memory that persists across loops (like Flowey)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersistentMemory {
    pub total_loops: u64,
    pub total_choices: u64,
    pub dark_choices: u64,
    pub light_choices: u64,
    pub key_memories: Vec<String>,
    pub character_deaths: HashMap<String, u64>,
    pub truths_discovered: Vec<String>,
    pub nihilism_score: i32, // -100 (hopeful) to +100 (nihilistic)
}

impl Default for PersistentMemory {
    fn default() -> Self {
        Self {
            total_loops: 0,
            total_choices: 0,
            dark_choices: 0,
            light_choices: 0,
            key_memories: Vec::new(),
            character_deaths: HashMap::new(),
            truths_discovered: Vec::new(),
            nihilism_score: 0,
        }
    }
}

/// A player session
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub name: Option<String>,
    pub current_loop: Loop,
    pub memory: PersistentMemory,
    pub narrative_history: Vec<NarrativeMoment>,
    pub created_at: DateTime<Utc>,
}

impl Player {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: None,
            current_loop: Loop {
                number: 1,
                started_at: now,
                ended_at: None,
                choices_made: Vec::new(),
                outcome: None,
            },
            memory: PersistentMemory::default(),
            narrative_history: Vec::new(),
            created_at: now,
        }
    }

    /// Reset the current loop but keep persistent memory
    pub fn reset_loop(&mut self) {
        self.memory.total_loops += 1;

        // Store the outcome of the previous loop
        if let Some(last_moment) = self.narrative_history.last() {
            if !self.memory.key_memories.contains(&last_moment.text) {
                if self.memory.key_memories.len() < 20 {
                    self.memory
                        .key_memories
                        .push(last_moment.text.clone());
                }
            }
        }

        let now = Utc::now();
        self.current_loop = Loop {
            number: self.memory.total_loops + 1,
            started_at: now,
            ended_at: None,
            choices_made: Vec::new(),
            outcome: None,
        };
        self.narrative_history.clear();
    }

    /// Record a choice and update memory
    pub fn make_choice(&mut self, choice_id: &str, is_dark: bool) {
        self.current_loop.choices_made.push(choice_id.to_string());
        self.memory.total_choices += 1;

        if is_dark {
            self.memory.dark_choices += 1;
            self.memory.nihilism_score = (self.memory.nihilism_score + 5).min(100);
        } else {
            self.memory.light_choices += 1;
            self.memory.nihilism_score = (self.memory.nihilism_score - 3).max(-100);
        }
    }

    /// Get narrative context for LLM
    pub fn get_narrative_context(&self) -> String {
        let mut context = String::new();

        context.push_str(&format!("Loop #{}\n", self.current_loop.number));
        context.push_str(&format!(
            "Nihilism Score: {} ({})\n",
            self.memory.nihilism_score,
            if self.memory.nihilism_score > 30 {
                "Descending into darkness"
            } else if self.memory.nihilism_score < -30 {
                "Finding meaning"
            } else {
                "Balanced on the edge"
            }
        ));

        if !self.memory.key_memories.is_empty() {
            context.push_str("\nMemories that persist:\n");
            for memory in self.memory.key_memories.iter().take(5) {
                context.push_str(&format!("- {}\n", memory));
            }
        }

        if !self.current_loop.choices_made.is_empty() {
            context.push_str("\nChoices this loop:\n");
            for choice in &self.current_loop.choices_made {
                context.push_str(&format!("- {}\n", choice));
            }
        }

        context
    }
}

/// Global game state
#[derive(Debug, Default)]
pub struct GameState {
    pub players: HashMap<Uuid, Player>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn create_player(&mut self) -> Player {
        let player = Player::new();
        self.players.insert(player.id, player.clone());
        player
    }

    pub fn get_player(&self, id: &Uuid) -> Option<&Player> {
        self.players.get(id)
    }

    pub fn get_player_mut(&mut self, id: &Uuid) -> Option<&mut Player> {
        self.players.get_mut(id)
    }
}
