//! OpenAI provider implementation using async-openai.

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt::Debug;

use super::client::LlmClient;
use super::conversion::{
    create_tool_definitions, from_openai_response, to_openai_messages, OpenAiChatRequest,
    OpenAiChatResponse, OpenAiMessage,
};
use super::error::LlmError;
use super::provider::{ProviderConfig, ProviderInfo};
use super::prompt::{ConversationHistory, PromptBuilder, PromptMessage, PromptRole, SystemPrompt};

/// OpenAI provider implementation.
#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    /// Provider configuration.
    config: ProviderConfig,
    /// System prompt.
    system_prompt: SystemPrompt,
    /// Conversation history.
    history: ConversationHistory,
    /// Whether to use actual API.
    use_api: bool,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider.
    #[allow(dead_code)]
    pub fn new(config: ProviderConfig) -> Self {
        Self::with_prompt(config, SystemPrompt::default())
    }

    /// Create a new OpenAI provider with custom system prompt.
    #[allow(dead_code)]
    pub fn with_prompt(config: ProviderConfig, prompt: SystemPrompt) -> Self {
        Self {
            config,
            system_prompt: prompt,
            history: ConversationHistory::new().with_max_messages(20),
            use_api: false, // Set to true when API keys are configured
        }
    }

    /// Enable/disable API usage.
    #[allow(dead_code)]
    pub fn set_use_api(&mut self, use_api: bool) {
        self.use_api = use_api;
    }

    /// Build an OpenAI chat request from prompt messages.
    fn build_request(&self, messages: &[PromptMessage]) -> OpenAiChatRequest {
        let openai_messages = to_openai_messages(messages);

        OpenAiChatRequest {
            model: self.config.model.clone(),
            messages: openai_messages,
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            tools: create_tool_definitions(),
            response_format: serde_json::json!({ "type": "json_object" }),
        }
    }

    /// Call the OpenAI API (stub - enable with real API keys).
    async fn call_api(&self, _request: &OpenAiChatRequest) -> Result<OpenAiChatResponse, LlmError> {
        // Stub implementation - would use async-openai in production
        // Enable by setting use_api = true and configuring API key
        Err(LlmError::ApiError {
            message: "API not configured. Set OPENAI_API_KEY environment variable.".to_string(),
        })
    }
}

#[async_trait]
impl LlmClient for OpenAiProvider {
    async fn complete(&self, prompt: &str) -> Result<String, LlmError> {
        if self.use_api {
            let messages = PromptBuilder::new()
                .with_system_prompt(self.system_prompt.clone())
                .user(prompt)
                .build();

            let request = self.build_request(&messages);
            let response = self.call_api(&request).await?;
            from_openai_response(&response).map(|v| v.to_string())
        } else {
            // Stub response
            Ok(format!(
                "This is a placeholder response for: {}",
                prompt.lines().next().unwrap_or(prompt)
            ))
        }
    }

    async fn generate_decision(&self, context_json: &Value) -> Result<Value, LlmError> {
        if self.use_api {
            // Convert context JSON to prompt messages
            let messages = convert_context_to_messages(context_json, &self.system_prompt);

            // Build and send request
            let request = self.build_request(&messages);
            let response = self.call_api(&request).await?;

            from_openai_response(&response)
        } else {
            // Stub decision - check context for tool calls
            let has_user_message = context_json
                .get("messages")
                .and_then(|m| m.as_array())
                .map(|arr| {
                    arr.iter().any(|m| {
                        m.get("role")
                            .and_then(|r| r.as_str())
                            .map(|r| r == "user")
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);

            if has_user_message {
                // Suggest using tools
                Ok(serde_json::json!({
                    "type": "reasoning",
                    "thought": "The user has asked a query. I should analyze what they're asking and determine if I need to explore the schema or execute a query."
                }))
            } else {
                Ok(serde_json::json!({
                    "type": "final_answer",
                    "answer": "I need more context to provide a useful response."
                }))
            }
        }
    }

    async fn generate_structured<T: DeserializeOwned + Debug>(
        &self,
        prompt: &str,
        _schema: &T,
    ) -> Result<T, LlmError> {
        if self.use_api {
            let content = self.complete(prompt).await?;
            serde_json::from_str(&content).map_err(|e| LlmError::ApiError {
                message: format!("Failed to parse structured response: {}", e),
            })
        } else {
            Err(LlmError::NoResponse)
        }
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            provider: self.config.provider_type.clone(),
            model: self.config.model.clone(),
        }
    }
}

/// Convert context JSON to prompt messages.
fn convert_context_to_messages(context: &Value, system_prompt: &SystemPrompt) -> Vec<PromptMessage> {
    let mut messages = Vec::new();

    // Add system prompt
    messages.push(PromptMessage::System {
        content: system_prompt.full(),
    });

    // Convert context messages
    if let Some(arr) = context.get("messages").and_then(|m| m.as_array()) {
        for item in arr {
            if let (Some(role_str), Some(content)) = (
                item.get("role").and_then(|r| r.as_str()),
                item.get("content").and_then(|c| c.as_str()),
            ) {
                let role = match role_str {
                    "user" => PromptRole::User,
                    "assistant" => PromptRole::Assistant,
                    "tool" => PromptRole::Tool,
                    "system" => PromptRole::System,
                    _ => PromptRole::User,
                };

                match role {
                    PromptRole::System => {
                        messages.push(PromptMessage::System {
                            content: content.to_string(),
                        });
                    }
                    PromptRole::User => {
                        messages.push(PromptMessage::User {
                            content: content.to_string(),
                        });
                    }
                    PromptRole::Assistant => {
                        messages.push(PromptMessage::Assistant {
                            content: content.to_string(),
                            tool_calls: Vec::new(),
                        });
                    }
                    PromptRole::Tool => {
                        messages.push(PromptMessage::Tool {
                            tool_call_id: item
                                .get("call_id")
                                .and_then(|c| c.as_str())
                                .unwrap_or("default")
                                .to_string(),
                            name: item
                                .get("tool_name")
                                .and_then(|t| t.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            content: content.to_string(),
                        });
                    }
                }
            }
        }
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_openai_provider_new() {
        let config = ProviderConfig::default();
        let provider = OpenAiProvider::new(config);
        assert_eq!(provider.provider_info().model, "gpt-4o");
    }

    #[test]
    fn test_openai_provider_with_prompt() {
        let config = ProviderConfig::default();
        let custom_prompt = SystemPrompt::default();
        let provider = OpenAiProvider::with_prompt(config, custom_prompt);
        assert!(!provider.system_prompt.full().is_empty());
    }

    #[test]
    fn test_convert_context_to_messages() {
        let context = json!({
            "messages": [
                {"role": "user", "content": "Hello"},
                {"role": "assistant", "content": "Hi there!"}
            ]
        });
        let prompt = SystemPrompt::default();
        let messages = convert_context_to_messages(&context, &prompt);

        assert_eq!(messages.len(), 3); // System + User + Assistant
    }

    #[test]
    fn test_stub_complete() {
        let config = ProviderConfig::default();
        let provider = OpenAiProvider::new(config);

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(provider.complete("Test prompt"));

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("Test prompt"));
    }
}
