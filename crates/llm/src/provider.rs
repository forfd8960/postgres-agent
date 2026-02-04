//! Provider configuration.

use serde::{Deserialize, Serialize};
use url::Url;

/// Provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    /// Provider type (openai, anthropic, etc.)
    pub provider_type: String,
    /// API base URL.
    pub base_url: Option<Url>,
    /// API key.
    pub api_key: Option<String>,
    /// Model identifier.
    pub model: String,
    /// Temperature for sampling.
    pub temperature: f32,
    /// Maximum tokens in response.
    pub max_tokens: u32,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: "openai".to_string(),
            base_url: None,
            api_key: None,
            model: "gpt-4o".to_string(),
            temperature: 0.0,
            max_tokens: 4096,
        }
    }
}

/// Provider information.
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// Provider type.
    pub provider: String,
    /// Model identifier.
    pub model: String,
}
