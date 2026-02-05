//! Chat view for displaying conversations.
//!
//! Shows messages from the agent and user in a scrollable format.

use postgres_agent_core::context::{Message, MessageRole};
use std::fmt;

/// A chat message with metadata.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// The message content.
    pub content: String,
    /// The role of the sender.
    pub role: MessageRole,
    /// Whether this is a thinking/reasoning message.
    pub is_reasoning: bool,
    /// Whether the message is currently loading.
    pub is_loading: bool,
}

impl ChatMessage {
    /// Create a new user message.
    #[must_use]
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            role: MessageRole::User,
            is_reasoning: false,
            is_loading: false,
        }
    }

    /// Create a new assistant message.
    #[must_use]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            role: MessageRole::Assistant,
            is_reasoning: false,
            is_loading: false,
        }
    }

    /// Create a new reasoning message.
    #[must_use]
    pub fn reasoning(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            role: MessageRole::Assistant,
            is_reasoning: true,
            is_loading: false,
        }
    }

    /// Create a loading placeholder.
    #[must_use]
    pub fn loading() -> Self {
        Self {
            content: String::new(),
            role: MessageRole::Assistant,
            is_reasoning: false,
            is_loading: true,
        }
    }

    /// Convert from core Message type.
    #[must_use]
    pub fn from_core(message: &Message) -> Self {
        let mut chat_msg = Self {
            content: message.content.clone(),
            role: message.role,
            is_reasoning: false,
            is_loading: false,
        };
        if message.role == MessageRole::Assistant && message.content.starts_with("Thinking:") {
            chat_msg.is_reasoning = true;
        }
        chat_msg
    }
}

/// Chat view state.
#[derive(Debug, Default)]
pub struct ChatView {
    /// All messages in the chat.
    messages: Vec<ChatMessage>,
    /// Vertical scroll offset.
    scroll_offset: usize,
    /// Whether auto-scroll is enabled.
    auto_scroll: bool,
}

impl ChatView {
    /// Create a new chat view.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a message to the chat.
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    /// Add a user message.
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.add_message(ChatMessage::user(content));
    }

    /// Add an assistant message.
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.add_message(ChatMessage::assistant(content));
    }

    /// Add a reasoning message.
    pub fn add_reasoning_message(&mut self, content: impl Into<String>) {
        self.add_message(ChatMessage::reasoning(content));
    }

    /// Add a loading indicator.
    pub fn add_loading(&mut self) {
        self.add_message(ChatMessage::loading());
    }

    /// Remove the loading indicator.
    pub fn remove_loading(&mut self) {
        if let Some(last) = self.messages.last() {
            if last.is_loading {
                self.messages.pop();
            }
        }
    }

    /// Clear all messages.
    pub fn clear(&mut self) {
        self.messages.clear();
        self.scroll_offset = 0;
    }

    /// Scroll up by one line.
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Scroll down by one line.
    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    /// Scroll to the top.
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to the bottom.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.messages.len().saturating_sub(1);
        self.auto_scroll = true;
    }

    /// Toggle auto-scroll.
    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
    }

    /// Set auto-scroll.
    pub fn set_auto_scroll(&mut self, enabled: bool) {
        self.auto_scroll = enabled;
    }

    /// Check if at bottom.
    #[must_use]
    pub fn is_at_bottom(&self) -> bool {
        self.scroll_offset >= self.messages.len().saturating_sub(1)
    }

    /// Get the number of messages.
    #[must_use]
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get all messages.
    #[must_use]
    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    /// Get the scroll offset.
    #[must_use]
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get the last assistant message content.
    #[must_use]
    pub fn last_assistant_message(&self) -> Option<&str> {
        self.messages
            .iter()
            .rev()
            .find(|m| m.role == MessageRole::Assistant)
            .map(|m| m.content.as_str())
    }
}

impl fmt::Display for ChatView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.messages.is_empty() {
            writeln!(f, "Welcome to PostgreSQL Agent!")?;
            writeln!(f)?;
            writeln!(f, "Ask me questions about your database in natural language.")?;
            writeln!(f)?;
            writeln!(f, "Examples:")?;
            writeln!(f, "  - \"Show me all users\"")?;
            writeln!(f, "  - \"Count the number of orders\"")?;
            writeln!(f, "  - \"What is the database schema?\"")?;
            return Ok(());
        }

        let visible_start = self.scroll_offset;
        let visible_messages = &self.messages[visible_start..];

        for msg in visible_messages {
            let role_prefix = match msg.role {
                MessageRole::User => "User: ",
                MessageRole::Assistant => {
                    if msg.is_reasoning {
                        "Thinking: "
                    } else {
                        "Assistant: "
                    }
                }
                MessageRole::System => "System: ",
                MessageRole::Tool => "Tool: ",
            };

            if msg.is_loading {
                writeln!(f, "{} ...", role_prefix)?;
            } else {
                writeln!(f, "{}{}", role_prefix, msg.content)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use postgres_agent_core::context::MessageRole;

    #[test]
    fn test_chat_message_creation() {
        let user = ChatMessage::user("Hello");
        assert_eq!(user.content, "Hello");
        assert_eq!(user.role, MessageRole::User);

        let assistant = ChatMessage::assistant("Hi there!");
        assert_eq!(assistant.content, "Hi there!");
        assert_eq!(assistant.role, MessageRole::Assistant);

        let thinking = ChatMessage::reasoning("Let me think...");
        assert!(thinking.is_reasoning);
    }

    #[test]
    fn test_chat_view_operations() {
        let mut view = ChatView::new();
        assert!(view.is_empty());

        view.add_user_message("Hello");
        view.add_assistant_message("Hi!");
        assert_eq!(view.len(), 2);

        view.clear();
        assert!(view.is_empty());
    }

    #[test]
    fn test_scroll_operations() {
        let mut view = ChatView::new();

        for i in 0..10 {
            view.add_user_message(format!("Message {}", i));
        }

        assert!(!view.is_at_bottom());

        view.scroll_to_bottom();
        assert!(view.is_at_bottom());

        view.scroll_up();
        assert!(!view.is_at_bottom());

        view.scroll_to_top();
        assert_eq!(view.scroll_offset(), 0);
    }
}
