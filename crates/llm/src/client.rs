//! LLM client trait.

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt::Debug;

use super::error::LlmError;
use super::provider::ProviderInfo;

/// Trait for LLM client implementations.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Generate a text completion.
    async fn complete(&self, prompt: &str) -> Result<String, LlmError>;

    /// Generate a decision from JSON context.
    async fn generate_decision(
        &self,
        context_json: &Value,
    ) -> Result<Value, LlmError>;

    /// Generate structured output with a schema.
    async fn generate_structured<T: DeserializeOwned + Debug>(
        &self,
        prompt: &str,
        schema: &T,
    ) -> Result<T, LlmError>;

    /// Get provider information.
    fn provider_info(&self) -> ProviderInfo;
}
