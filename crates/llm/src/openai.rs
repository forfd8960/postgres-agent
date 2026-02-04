//! OpenAI provider implementation.

use async_trait::async_trait;

use super::{client::LlmClient, error::LlmError, provider::ProviderInfo, ProviderConfig};

/// OpenAI provider (stub).
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
        // Stub implementation
        Ok("LLM response placeholder".to_string())
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            provider: self.config.provider_type.clone(),
            model: self.config.model.clone(),
        }
    }
}
