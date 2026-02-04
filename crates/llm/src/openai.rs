//! OpenAI provider implementation.

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt::Debug;

use super::{client::LlmClient, error::LlmError, provider::ProviderInfo, ProviderConfig};

/// OpenAI provider implementation.
#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    /// Provider configuration.
    config: ProviderConfig,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider.
    #[allow(dead_code)]
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl LlmClient for OpenAiProvider {
    async fn complete(&self, _prompt: &str) -> Result<String, LlmError> {
        // Stub implementation - full implementation in Phase 2
        Ok("LLM response placeholder".to_string())
    }

    async fn generate_decision(&self, _context_json: &Value) -> Result<Value, LlmError> {
        // Stub implementation - returns a default final_answer decision
        // Full implementation will call OpenAI API with tool calling support
        Ok(serde_json::json!({
            "type": "final_answer",
            "answer": "I need more context to provide a decision."
        }))
    }

    async fn generate_structured<T: DeserializeOwned + Debug>(
        &self,
        _prompt: &str,
        _schema: &T,
    ) -> Result<T, LlmError> {
        // Stub implementation - full implementation in Phase 2
        Err(LlmError::NoResponse)
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            provider: self.config.provider_type.clone(),
            model: self.config.model.clone(),
        }
    }
}
