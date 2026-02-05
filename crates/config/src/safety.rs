//! Safety configuration.

use serde::{Deserialize, Serialize};

/// Safety level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SafetyLevel {
    /// Maximum safety - read-only, no modifications.
    #[default]
    ReadOnly,
    /// Balanced safety - confirmations for DML/DDL.
    Balanced,
    /// Permissive - faster execution with minimal checks.
    Permissive,
}

/// Safety and security settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SafetyConfig {
    /// Default safety level.
    #[serde(default)]
    pub safety_level: SafetyLevel,

    /// Whether to require confirmation for mutations.
    #[serde(default = "default_require_confirmation")]
    pub require_confirmation: bool,

    /// Whether to show SQL preview before execution.
    #[serde(default = "default_show_sql_preview")]
    pub show_sql_preview: bool,

    /// Maximum query length.
    #[serde(default = "default_max_query_length")]
    pub max_query_length: usize,
}

fn default_require_confirmation() -> bool {
    true
}

fn default_show_sql_preview() -> bool {
    true
}

fn default_max_query_length() -> usize {
    10_000
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            safety_level: SafetyLevel::default(),
            require_confirmation: default_require_confirmation(),
            show_sql_preview: default_show_sql_preview(),
            max_query_length: default_max_query_length(),
        }
    }
}
