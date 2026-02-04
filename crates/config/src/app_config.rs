//! Application configuration.

use serde::{Deserialize, Serialize};

use super::{DatabaseProfile, LlmConfig, SafetyConfig};

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// LLM provider configuration.
    #[serde(default)]
    pub llm: LlmConfig,

    /// Database profiles.
    #[serde(default)]
    pub databases: Vec<DatabaseProfile>,

    /// Agent behavior settings.
    #[serde(default)]
    pub agent: AgentConfig,

    /// Safety and security settings.
    #[serde(default)]
    pub safety: SafetyConfig,
}

/// Alias for AppConfig.
pub type Config = AppConfig;

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            databases: Vec::new(),
            agent: AgentConfig::default(),
            safety: SafetyConfig::default(),
        }
    }
}

/// Agent behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    /// Maximum conversation history to retain.
    #[serde(default = "default_max_history")]
    pub max_history: usize,

    /// Maximum reasoning iterations.
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,

    /// Default output format.
    #[serde(default)]
    pub default_output: String,
}

fn default_max_history() -> usize {
    50
}

fn default_max_iterations() -> u32 {
    10
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_history: default_max_history(),
            max_iterations: default_max_iterations(),
            default_output: "table".to_string(),
        }
    }
}
