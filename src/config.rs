use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub llm_base_url: String,
    pub llm_api_key: String,
    pub llm_model: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3001),
            llm_base_url: env::var("LLM_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080/v1".to_string()),
            llm_api_key: env::var("LLM_API_KEY").unwrap_or_else(|_| "sk-none".to_string()),
            llm_model: env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4".to_string()),
        }
    }
}
