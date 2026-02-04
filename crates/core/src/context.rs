//! Agent context and conversation management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// Role (user, assistant, tool, system).
    pub role: MessageRole,
    /// Message content.
    pub content: String,
    /// Timestamp.
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
    /// Optional: SQL that was generated.
    pub generated_sql: Option<String>,
}

/// Role of a message in the conversation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageRole {
    User,
    Assistant,
    Tool,
    System,
}

/// The agent's context for a conversation.
#[derive(Debug, Clone, Default)]
pub struct AgentContext {
    /// Conversation messages.
    messages: Vec<Message>,
    /// Maximum messages to retain.
    max_messages: usize,
}

impl AgentContext {
    /// Create a new context with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new context with a custom message limit.
    #[must_use]
    pub fn with_limit(max_messages: usize) -> Self {
        Self {
            messages: Vec::with_capacity(max_messages),
            max_messages,
        }
    }

    /// Add a user message.
    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(Message {
            role: MessageRole::User,
            content: content.to_string(),
            timestamp: Utc::now(),
            generated_sql: None,
        });
        self.prune();
    }

    /// Add an assistant message.
    pub fn add_assistant_message(&mut self, content: &str) {
        self.messages.push(Message {
            role: MessageRole::Assistant,
            content: content.to_string(),
            timestamp: Utc::now(),
            generated_sql: None,
        });
        self.prune();
    }

    /// Prune oldest messages if over limit.
    fn prune(&mut self) {
        if self.messages.len() > self.max_messages {
            let remove_count = self.messages.len() - self.max_messages;
            self.messages.drain(..remove_count);
        }
    }
}
