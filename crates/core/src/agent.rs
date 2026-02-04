//! Agent core implementation.

use serde::{Deserialize, Serialize};

use crate::context::AgentContext;

/// Configuration for agent behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    /// Maximum number of reasoning iterations.
    pub max_iterations: u32,
    /// Whether to require confirmation for mutations.
    pub require_confirmation: bool,
    /// Safety level for operations.
    pub safety_level: SafetyLevel,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            require_confirmation: true,
            safety_level: SafetyLevel::Balanced,
        }
    }
}

/// Safety levels controlling agent behavior.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SafetyLevel {
    /// Maximum safety - read-only, no modifications.
    ReadOnly,
    /// Balanced safety - confirmations for DML/DDL.
    Balanced,
    /// Permissive - faster execution with minimal checks.
    Permissive,
}

/// The core agent that implements the ReAct reasoning loop.
#[derive(Debug)]
pub struct PostgresAgent {
    /// Context manager for conversation state.
    pub context: AgentContext,
    /// Agent configuration.
    pub config: AgentConfig,
}

impl PostgresAgent {
    /// Create a new agent with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: AgentContext::new(),
            config: AgentConfig::default(),
        }
    }
}

impl Default for PostgresAgent {
    fn default() -> Self {
        Self::new()
    }
}
