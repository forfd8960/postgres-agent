//! LLM client trait.

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use super::error::LlmError;
use super::provider::ProviderInfo;

/// Trait for LLM client implementations.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Generate a text completion.
    async fn complete(&self, prompt: &str) -> Result<String, LlmError>;

    /// Get provider information.
    fn provider_info(&self) -> ProviderInfo;
}
