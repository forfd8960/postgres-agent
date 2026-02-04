//! System prompts and prompt templates.

use serde::{Deserialize, Serialize};

/// System prompt for the PostgreSQL Agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPrompt {
    /// The base prompt text.
    pub base: String,
    /// Instructions for tool usage.
    pub tool_instructions: String,
    /// Instructions for SQL safety.
    pub safety_instructions: String,
    /// Format instructions for responses.
    pub format_instructions: String,
}

impl Default for SystemPrompt {
    fn default() -> Self {
        Self::standard()
    }
}

impl SystemPrompt {
    /// Create the standard system prompt for PostgreSQL query generation.
    #[must_use]
    pub fn standard() -> Self {
        Self {
            base: String::from(include_str!("prompts/base.txt")),
            tool_instructions: String::from(include_str!("prompts/tools.txt")),
            safety_instructions: String::from(include_str!("prompts/safety.txt")),
            format_instructions: String::from(include_str!("prompts/format.txt")),
        }
    }

    /// Get the full system prompt.
    #[must_use]
    pub fn full(&self) -> String {
        format!(
            "{}\n\n{}\n\n{}\n\n{}",
            self.base, self.tool_instructions, self.safety_instructions, self.format_instructions
        )
    }

    /// Get the base system prompt only.
    #[must_use]
    pub fn base_only(&self) -> &str {
        &self.base
    }

    /// With custom database schema.
    #[must_use]
    pub fn with_schema(&self, schema: &str) -> String {
        format!("{}\n\n## Database Schema\n\n{}", self.base, schema)
    }
}

/// Role for LLM messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptRole {
    /// System message.
    System,
    /// User message.
    User,
    /// Assistant message.
    Assistant,
    /// Tool result message.
    Tool,
}

/// A message for the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum PromptMessage {
    /// System message.
    #[serde(rename = "system")]
    System {
        /// Content.
        content: String,
    },
    /// User message.
    #[serde(rename = "user")]
    User {
        /// Content.
        content: String,
    },
    /// Assistant message.
    #[serde(rename = "assistant")]
    Assistant {
        /// Content.
        content: String,
        /// Optional tool calls.
        #[serde(default)]
        tool_calls: Vec<PromptToolCall>,
    },
    /// Tool result.
    #[serde(rename = "tool")]
    Tool {
        /// Tool call ID.
        tool_call_id: String,
        /// Tool name.
        name: String,
        /// Content.
        content: String,
    },
}

/// A tool call in a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptToolCall {
    /// Call ID.
    pub id: String,
    /// Tool name.
    pub r#type: String,
    /// Arguments.
    pub function: PromptToolCallFunction,
}

/// Function call within a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptToolCallFunction {
    /// Function name.
    pub name: String,
    /// Arguments as JSON string.
    pub arguments: String,
}

/// Prompt builder for constructing LLM requests.
#[derive(Debug, Default, Clone)]
pub struct PromptBuilder {
    messages: Vec<PromptMessage>,
    system_prompt: Option<SystemPrompt>,
}

impl PromptBuilder {
    /// Create a new prompt builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the system prompt.
    #[must_use]
    pub fn with_system_prompt(mut self, prompt: SystemPrompt) -> Self {
        self.system_prompt = Some(prompt);
        self
    }

    /// Add a system message.
    #[must_use]
    pub fn system(mut self, content: impl Into<String>) -> Self {
        self.messages.push(PromptMessage::System {
            content: content.into(),
        });
        self
    }

    /// Add a user message.
    #[must_use]
    pub fn user(mut self, content: impl Into<String>) -> Self {
        self.messages.push(PromptMessage::User {
            content: content.into(),
        });
        self
    }

    /// Add an assistant message.
    #[must_use]
    pub fn assistant(mut self, content: impl Into<String>) -> Self {
        self.messages.push(PromptMessage::Assistant {
            content: content.into(),
            tool_calls: Vec::new(),
        });
        self
    }

    /// Add a tool result.
    #[must_use]
    pub fn tool_result(
        mut self,
        tool_call_id: impl Into<String>,
        name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        self.messages.push(PromptMessage::Tool {
            tool_call_id: tool_call_id.into(),
            name: name.into(),
            content: content.into(),
        });
        self
    }

    /// Build the messages vector.
    #[must_use]
    pub fn build(self) -> Vec<PromptMessage> {
        let mut messages = Vec::new();

        // Add system prompt first if set
        if let Some(prompt) = self.system_prompt {
            messages.push(PromptMessage::System {
                content: prompt.full(),
            });
        }

        // Add all other messages
        messages.extend(self.messages);
        messages
    }
}

/// Conversation history for context.
#[derive(Debug, Clone, Default)]
pub struct ConversationHistory {
    messages: Vec<PromptMessage>,
    max_messages: usize,
    max_tokens: usize,
}

impl ConversationHistory {
    /// Create a new conversation history.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// With maximum message count.
    #[must_use]
    pub fn with_max_messages(mut self, max: usize) -> Self {
        self.max_messages = max;
        self
    }

    /// With maximum token count.
    #[must_use]
    pub fn with_max_tokens(mut self, max: usize) -> Self {
        self.max_tokens = max;
        self
    }

    /// Add a message.
    pub fn add(&mut self, message: PromptMessage) {
        self.messages.push(message);
        self.prune();
    }

    /// Add multiple messages.
    pub fn extend(&mut self, messages: impl IntoIterator<Item = PromptMessage>) {
        self.messages.extend(messages);
        self.prune();
    }

    /// Get all messages.
    #[must_use]
    pub fn messages(&self) -> &[PromptMessage] {
        &self.messages
    }

    /// Get the last N messages.
    #[must_use]
    pub fn recent(&self, count: usize) -> &[PromptMessage] {
        let start = self.messages.len().saturating_sub(count);
        &self.messages[start..]
    }

    /// Clear the history.
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Estimate token count (rough approximation).
    #[must_use]
    pub fn token_estimate(&self) -> usize {
        let total: usize = self
            .messages
            .iter()
            .map(|m| match m {
                PromptMessage::System { content } => content.len(),
                PromptMessage::User { content } => content.len(),
                PromptMessage::Assistant { content, .. } => content.len(),
                PromptMessage::Tool { content, .. } => content.len(),
            })
            .sum();
        total / 4
    }

    /// Prune messages if over limits.
    fn prune(&mut self) {
        // Prune by message count
        if self.messages.len() > self.max_messages {
            self.messages
                .drain(..self.messages.len() - self.max_messages);
        }

        // Prune by token estimate
        while self.token_estimate() > self.max_tokens && !self.messages.is_empty() {
            self.messages.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder() {
        let builder = PromptBuilder::new()
            .system("You are a helpful assistant.")
            .user("Hello")
            .assistant("Hi there!");

        let messages = builder.build();
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_system_prompt_full() {
        let prompt = SystemPrompt::default();
        let full = prompt.full();
        assert!(!full.is_empty());
        assert!(full.contains("PostgreSQL"));
    }

    #[test]
    fn test_conversation_history() {
        let mut history = ConversationHistory::with_max_messages(3);
        history.add(PromptMessage::user("Hello"));
        history.add(PromptMessage::assistant("Hi"));
        history.add(PromptMessage::user("How are you?"));
        history.add(PromptMessage::assistant("Good!"));

        // Should be pruned to 3 messages
        assert_eq!(history.messages().len(), 3);
    }
}
