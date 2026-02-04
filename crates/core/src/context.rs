//! Agent context and conversation management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// Role (user, assistant, tool, system).
    #[serde(default)]
    pub role: MessageRole,
    /// Message content.
    pub content: String,
    /// Timestamp.
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
    /// Optional: SQL that was generated.
    #[serde(default)]
    pub generated_sql: Option<String>,
    /// Optional: tool name for tool messages.
    #[serde(default)]
    pub tool_name: Option<String>,
}

impl Message {
    /// Create a new user message.
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            timestamp: Utc::now(),
            generated_sql: None,
            tool_name: None,
        }
    }

    /// Create a new assistant message.
    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            timestamp: Utc::now(),
            generated_sql: None,
            tool_name: None,
        }
    }

    /// Create a new tool message.
    #[must_use]
    pub fn tool(content: impl Into<String>, tool_name: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            timestamp: Utc::now(),
            generated_sql: None,
            tool_name: Some(tool_name.into()),
        }
    }

    /// Create a new system message.
    #[must_use]
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            timestamp: Utc::now(),
            generated_sql: None,
            tool_name: None,
        }
    }
}

/// Role of a message in the conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageRole {
    /// User input.
    User,
    /// Assistant response.
    Assistant,
    /// Tool execution result.
    Tool,
    /// System instructions.
    System,
}

impl Default for MessageRole {
    fn default() -> Self {
        MessageRole::User
    }
}

/// Statistics about the conversation context.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    /// Total message count.
    pub message_count: usize,
    /// Total token estimate.
    pub token_estimate: usize,
    /// User message count.
    pub user_message_count: usize,
    /// Assistant message count.
    pub assistant_message_count: usize,
    /// Tool call count.
    pub tool_call_count: usize,
}

/// The agent's context for a conversation session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// Conversation messages.
    messages: Vec<Message>,
    /// Maximum messages to retain.
    max_messages: usize,
    /// Maximum token estimate to retain.
    max_tokens: usize,
    /// Current database schema (cached).
    database_schema: Option<String>,
}

impl Default for AgentContext {
    fn default() -> Self {
        Self {
            messages: Vec::with_capacity(100),
            max_messages: 50,
            max_tokens: 8000,
            database_schema: None,
        }
    }
}

impl AgentContext {
    /// Create a new context with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new context with custom message limit.
    #[must_use]
    pub fn with_limit(max_messages: usize) -> Self {
        Self {
            messages: Vec::with_capacity(max_messages),
            max_messages,
            ..Default::default()
        }
    }

    /// Create a new context with custom token limit.
    #[must_use]
    pub fn with_token_limit(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            ..Default::default()
        }
    }

    /// Add a user message.
    pub fn add_user_message(&mut self, content: &str) {
        self.messages.push(Message::user(content));
        self.prune();
    }

    /// Add an assistant message.
    pub fn add_assistant_message(&mut self, content: &str) {
        self.messages.push(Message::assistant(content));
        self.prune();
    }

    /// Add a tool message.
    pub fn add_tool_message(&mut self, content: &str, tool_name: &str) {
        self.messages.push(Message::tool(content, tool_name));
        self.prune();
    }

    /// Add a system message.
    pub fn add_system_message(&mut self, content: &str) {
        self.messages.push(Message::system(content));
        self.prune();
    }

    /// Add a complete message.
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.prune();
    }

    /// Get all messages.
    #[must_use]
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Get the last N messages.
    #[must_use]
    pub fn recent_messages(&self, count: usize) -> &[Message] {
        let start = self.messages.len().saturating_sub(count);
        &self.messages[start..]
    }

    /// Get messages by role.
    #[must_use]
    pub fn messages_by_role(&self, role: MessageRole) -> Vec<&Message> {
        self.messages.iter().filter(|m| m.role == role).collect()
    }

    /// Get the last assistant message.
    #[must_use]
    pub fn last_assistant_message(&self) -> Option<&Message> {
        self.messages.iter().rev().find(|m| m.role == MessageRole::Assistant)
    }

    /// Get the last user message.
    #[must_use]
    pub fn last_user_message(&self) -> Option<&Message> {
        self.messages.iter().rev().find(|m| m.role == MessageRole::User)
    }

    /// Get the conversation history as a formatted string.
    #[must_use]
    pub fn history_string(&self) -> String {
        self.messages
            .iter()
            .map(|m| format!("[{:?}]: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get statistics about the current context.
    #[must_use]
    pub fn stats(&self) -> ContextStats {
        let user_count = self.messages.iter().filter(|m| m.role == MessageRole::User).count();
        let assistant_count = self.messages.iter().filter(|m| m.role == MessageRole::Assistant).count();
        let tool_count = self.messages.iter().filter(|m| m.role == MessageRole::Tool).count();

        ContextStats {
            message_count: self.messages.len(),
            token_estimate: self.estimate_tokens(),
            user_message_count: user_count,
            assistant_message_count: assistant_count,
            tool_call_count: tool_count,
        }
    }

    /// Estimate token count (rough approximation: 4 chars per token).
    #[must_use]
    pub fn estimate_tokens(&self) -> usize {
        let total_chars: usize = self.messages.iter().map(|m| m.content.len()).sum();
        total_chars / 4
    }

    /// Check if context is within token limits.
    #[must_use]
    pub fn within_token_limit(&self) -> bool {
        self.estimate_tokens() <= self.max_tokens
    }

    /// Set the cached database schema.
    pub fn set_database_schema(&mut self, schema: String) {
        self.database_schema = Some(schema);
    }

    /// Get the cached database schema.
    #[must_use]
    pub fn database_schema(&self) -> Option<&str> {
        self.database_schema.as_deref()
    }

    /// Clear all messages (reset conversation).
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get the number of messages.
    #[must_use]
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if there are no messages.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Set maximum messages limit.
    pub fn set_max_messages(&mut self, max: usize) {
        self.max_messages = max;
        self.prune();
    }

    /// Set maximum token limit.
    pub fn set_max_tokens(&mut self, max: usize) {
        self.max_tokens = max;
        self.prune();
    }

    /// Prune oldest messages if over limits.
    fn prune(&mut self) {
        // Prune by message count
        if self.messages.len() > self.max_messages {
            let remove_count = self.messages.len() - self.max_messages;
            self.messages.drain(..remove_count);
        }

        // Prune by token estimate
        while self.estimate_tokens() > self.max_tokens && !self.messages.is_empty() {
            self.messages.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
        assert!(msg.tool_name.is_none());
    }

    #[test]
    fn test_context_add_message() {
        let mut ctx = AgentContext::new();
        ctx.add_user_message("Test");
        assert_eq!(ctx.len(), 1);
    }

    #[test]
    fn test_context_pruning() {
        let mut ctx = AgentContext::with_limit(3);
        ctx.add_user_message("1");
        ctx.add_user_message("2");
        ctx.add_user_message("3");
        ctx.add_user_message("4");
        assert_eq!(ctx.len(), 3);
        assert_eq!(ctx.messages()[0].content, "2");
    }

    #[test]
    fn test_context_stats() {
        let mut ctx = AgentContext::new();
        ctx.add_user_message("Hello");
        ctx.add_assistant_message("Hi there!");
        let stats = ctx.stats();
        assert_eq!(stats.message_count, 2);
        assert_eq!(stats.user_message_count, 1);
        assert_eq!(stats.assistant_message_count, 1);
    }
}
