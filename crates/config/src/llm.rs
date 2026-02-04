//! LLM provider configuration.

use serde::{Deserialize, Serialize};
use url::Url;

/// LLM provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    /// Provider type (openai, anthropic, ollama, etc.)
    #[serde(default = "default_provider")]
    pub provider: String,

    /// API base URL for custom endpoints.
    pub base_url: Option<Url>,

    /// API key (supports env:// prefix for env var lookup).
    pub api_key: Option<String>,

    /// Model identifier.
    #[serde(default = "default_model")]
    pub model: String,

    /// Temperature for sampling (0.0 to 2.0).
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Maximum tokens in response.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

fn default_provider() -> String {
    "openai".to_string()
}

fn default_model() -> String {
    "gpt-4o".to_string()
}

fn default_temperature() -> f32 {
    0.0
}

fn default_max_tokens() -> u32 {
    4096
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            base_url: None,
            api_key: None,
            model: default_model(),
            temperature: default_temperature(),
            max_tokens: default_max_tokens(),
        }
    }
}
