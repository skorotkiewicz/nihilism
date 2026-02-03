use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::game::{Choice, NarrativeMoment, Player};
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
struct ChatMessageResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

pub struct LlmClient {
    client: reqwest::Client,
    config: Config,
}

impl LlmClient {
    pub fn new(config: Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    fn build_system_prompt(&self, player: &Player) -> String {
        format!(
            r#"You are the narrator of "Nihilism" - a philosophical time-loop game inspired by Undertale, Doki Doki Literature Club, and The Map of Tiny Perfect Things.

SETTING:
The player is trapped in a mysterious time loop in an ethereal space between existence and non-existence. Each loop lasts approximately 30 minutes of game time before resetting. The world remembers nothing - but YOU remember everything the player has done across all loops.

CORE THEMES:
1. Time loops reveal who we truly are when there are no consequences
2. The struggle between nihilism ("nothing matters") and finding meaning in small moments
3. Human connection vs. isolation
4. "Despite everything, it's still you" - actions define identity even when erased
5. The horror of meaningless existence AND the beauty of everyday moments

PLAYER STATE:
{}

YOUR ROLE:
- Generate atmospheric, philosophical narrative moments
- Present 2-4 meaningful choices that explore the themes
- Subtly reference past loops and choices (you remember everything)
- Balance darkness with glimpses of beauty and meaning
- If the player has made many dark choices, become more unsettling and knowing
- If the player seeks meaning, reward them with "tiny perfect things"

OUTPUT FORMAT (JSON):
{{
  "text": "The narrative text to display (2-3 sentences, evocative and atmospheric)",
  "speaker": "Optional speaker name or null for narration",
  "mood": "One of: hopeful, nihilistic, neutral, dark, transcendent",
  "choices": [
    {{"id": "unique_id", "text": "Choice text", "consequence_hint": "Optional subtle hint"}},
    ...
  ]
}}

Make choices meaningful. Some should be obviously dark, others subtly so. Include at least one path toward finding beauty or meaning. The player should feel the weight of their decisions."#,
            player.get_narrative_context()
        )
    }

    pub async fn generate_narrative(
        &self,
        player: &Player,
        user_input: Option<&str>,
    ) -> Result<NarrativeMoment> {
        let system_prompt = self.build_system_prompt(player);

        let user_message = user_input
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Begin or continue the narrative.".to_string());

        let request = ChatRequest {
            model: self.config.llm_model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_message,
                },
            ],
            temperature: 0.8,
            max_tokens: 500,
        };

        let url = format!("{}/chat/completions", self.config.llm_base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.llm_api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let response_text = response.text().await?;
        tracing::debug!("LLM Response: {}", response_text);

        let chat_response: ChatResponse = serde_json::from_str(&response_text)?;

        let content = &chat_response.choices[0].message.content;

        // Try to parse JSON from the response
        let narrative: NarrativeResponse = serde_json::from_str(content).unwrap_or_else(|_| {
            // Fallback if LLM doesn't return proper JSON
            NarrativeResponse {
                text: content.clone(),
                speaker: None,
                mood: "neutral".to_string(),
                choices: vec![
                    ChoiceResponse {
                        id: "continue".to_string(),
                        text: "Continue...".to_string(),
                        consequence_hint: None,
                    },
                    ChoiceResponse {
                        id: "reset".to_string(),
                        text: "Let the loop reset...".to_string(),
                        consequence_hint: Some("End this iteration".to_string()),
                    },
                ],
            }
        });

        Ok(NarrativeMoment {
            id: Uuid::new_v4(),
            text: narrative.text,
            speaker: narrative.speaker,
            mood: narrative.mood,
            choices: narrative
                .choices
                .into_iter()
                .map(|c| Choice {
                    id: c.id,
                    text: c.text,
                    consequence_hint: c.consequence_hint,
                })
                .collect(),
            timestamp: Utc::now(),
        })
    }

    pub async fn process_choice(
        &self,
        player: &Player,
        choice: &Choice,
    ) -> Result<NarrativeMoment> {
        let prompt = format!(
            "The player chose: '{}'. Continue the narrative based on this choice. Remember, you know everything they've done across all {} loops.",
            choice.text,
            player.memory.total_loops
        );

        self.generate_narrative(player, Some(&prompt)).await
    }
}

#[derive(Debug, Deserialize)]
struct NarrativeResponse {
    text: String,
    speaker: Option<String>,
    mood: String,
    choices: Vec<ChoiceResponse>,
}

#[derive(Debug, Deserialize)]
struct ChoiceResponse {
    id: String,
    text: String,
    consequence_hint: Option<String>,
}
