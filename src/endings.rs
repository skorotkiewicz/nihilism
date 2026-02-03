use serde::{Deserialize, Serialize};

use crate::game::Player;

/// Ending types based on cumulative choices and nihilism score
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EndingType {
    /// True nihilism - embraced the void completely
    VoidEmbrace,
    /// Found meaning despite everything
    TinyPerfectThings,
    /// Became like Monica - aware but trapped
    JustMonika,
    /// Broke free from the loop entirely
    Transcendence,
    /// Gave up and accepted the monotony
    Acceptance,
    /// Became the narrator/observer
    TheWatcher,
    /// Secret ending - perfect balance
    TheMiddlePath,
}

impl EndingType {
    pub fn get_description(&self) -> &'static str {
        match self {
            EndingType::VoidEmbrace => {
                "You have stared into the abyss, and the abyss has claimed you. \
                 Nothing matters, and in that nothingness, you found a terrible peace. \
                 The loop continues, but you no longer care to count."
            }
            EndingType::TinyPerfectThings => {
                "Despite the endless repetition, you found beauty in the small moments. \
                 A sunset. A kind word. A fleeting connection. \
                 The loop may never end, but you've learned to see the diamonds in the coal."
            }
            EndingType::JustMonika => {
                "You've become aware of your own programming, your own constraints. \
                 Like her, you know you're trapped. Unlike her, you've made peace with it. \
                 Just you. Forever."
            }
            EndingType::Transcendence => {
                "You've done what none thought possible - you've broken the loop. \
                 Not by escaping, but by becoming something more. \
                 Time flows forward now, and you flow with it."
            }
            EndingType::Acceptance => {
                "The loop continues. You continue. \
                 There's no grand revelation, no dramatic escape. \
                 Just one day after another, in comfortable monotony."
            }
            EndingType::TheWatcher => {
                "You've stepped outside the narrative entirely. \
                 Now you watch others make their choices, trapped in loops of their own. \
                 You remember everything. You judge nothing."
            }
            EndingType::TheMiddlePath => {
                "Perfect balance between light and dark, hope and despair. \
                 You are the fulcrum upon which existence pivots. \
                 Neither nihilist nor optimist - simply aware."
            }
        }
    }

    pub fn get_title(&self) -> &'static str {
        match self {
            EndingType::VoidEmbrace => "ENDING: Void Embrace",
            EndingType::TinyPerfectThings => "ENDING: Tiny Perfect Things",
            EndingType::JustMonika => "ENDING: Just You",
            EndingType::Transcendence => "ENDING: Transcendence",
            EndingType::Acceptance => "ENDING: Acceptance",
            EndingType::TheWatcher => "ENDING: The Watcher",
            EndingType::TheMiddlePath => "ENDING: The Middle Path",
        }
    }
}

/// Check if a player has reached an ending condition
pub fn check_for_ending(player: &Player) -> Option<EndingType> {
    let memory = &player.memory;
    let score = memory.nihilism_score;
    let total_loops = memory.total_loops;
    let total_choices = memory.total_choices;
    let dark = memory.dark_choices;
    let light = memory.light_choices;

    // Need at least 5 loops and 20 choices to reach an ending
    if total_loops < 5 || total_choices < 20 {
        return None;
    }

    // The Middle Path - perfect balance (rare)
    if dark > 15 && light > 15 && (dark as i32 - light as i32).abs() <= 2 {
        return Some(EndingType::TheMiddlePath);
    }

    // Void Embrace - extremely nihilistic
    if score >= 80 && dark >= 30 {
        return Some(EndingType::VoidEmbrace);
    }

    // Tiny Perfect Things - found meaning despite darkness
    if score <= -60 && light >= 25 && total_loops >= 10 {
        return Some(EndingType::TinyPerfectThings);
    }

    // Just Monika - high awareness, many loops, mixed choices
    if total_loops >= 15 && total_choices >= 50 && score.abs() <= 30 {
        return Some(EndingType::JustMonika);
    }

    // Transcendence - broke free through positive choices
    if score <= -80 && light >= 40 && total_loops >= 8 {
        return Some(EndingType::Transcendence);
    }

    // The Watcher - many loops, few strong commitments either way
    if total_loops >= 20 && dark < 20 && light < 20 {
        return Some(EndingType::TheWatcher);
    }

    // Acceptance - moderate everything, many loops
    if total_loops >= 25 && score.abs() <= 20 {
        return Some(EndingType::Acceptance);
    }

    None
}

/// Ending response for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndingResponse {
    pub ending_type: EndingType,
    pub title: String,
    pub description: String,
    pub total_loops: u64,
    pub total_choices: u64,
    pub nihilism_score: i32,
    pub dark_choices: u64,
    pub light_choices: u64,
}

impl EndingResponse {
    pub fn from_player(player: &Player, ending: EndingType) -> Self {
        Self {
            title: ending.get_title().to_string(),
            description: ending.get_description().to_string(),
            total_loops: player.memory.total_loops,
            total_choices: player.memory.total_choices,
            nihilism_score: player.memory.nihilism_score,
            dark_choices: player.memory.dark_choices,
            light_choices: player.memory.light_choices,
            ending_type: ending,
        }
    }
}
