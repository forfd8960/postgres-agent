//! Text input component for the TUI.
//!
//! Provides a text input field for entering queries and commands.

use std::fmt;

/// Input mode for the text area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal editing mode.
    #[default]
    Normal,
    /// Insert mode.
    Insert,
    /// Command mode (like vim).
    Command,
}

/// Text input component state.
#[derive(Debug, Default)]
pub struct Input {
    /// Current text content.
    content: String,
    /// Cursor position within the text.
    cursor: usize,
    /// Current input mode.
    mode: InputMode,
    /// Placeholder text when empty.
    placeholder: String,
}

impl Input {
    /// Create a new input component.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new input with a placeholder.
    #[must_use]
    pub fn with_placeholder(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            ..Default::default()
        }
    }

    /// Get the current text content.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get mutable access to content.
    pub fn content_mut(&mut self) -> &mut String {
        &mut self.content
    }

    /// Clear the input.
    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
    }

    /// Check if the input is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Insert a character at the cursor position.
    pub fn insert_char(&mut self, c: char) {
        self.content.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Insert text at the cursor position.
    pub fn insert_text(&mut self, text: &str) {
        self.content.insert_str(self.cursor, text);
        self.cursor += text.chars().count();
    }

    /// Delete the character before the cursor.
    pub fn delete_before_cursor(&mut self) {
        if self.cursor > 0 {
            self.content.remove(self.cursor - 1);
            self.cursor = self.cursor.saturating_sub(1);
        }
    }

    /// Delete the character after the cursor.
    pub fn delete_after_cursor(&mut self) {
        if self.cursor < self.content.len() {
            self.content.remove(self.cursor);
        }
    }

    /// Move the cursor forward by one character.
    pub fn move_cursor_forward(&mut self) {
        if self.cursor < self.content.len() {
            self.cursor += 1;
        }
    }

    /// Move the cursor backward by one character.
    pub fn move_cursor_backward(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move the cursor to the beginning of the line.
    pub fn move_cursor_to_start(&mut self) {
        self.cursor = 0;
    }

    /// Move the cursor to the end of the line.
    pub fn move_cursor_to_end(&mut self) {
        self.cursor = self.content.len();
    }

    /// Move cursor up (to start for single-line input).
    pub fn move_cursor_up(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor down (to end for single-line input).
    pub fn move_cursor_down(&mut self) {
        self.cursor = self.content.len();
    }

    /// Set the input mode.
    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    /// Get the current input mode.
    #[must_use]
    pub fn mode(&self) -> InputMode {
        self.mode
    }

    /// Get the placeholder text.
    #[must_use]
    pub fn placeholder(&self) -> &str {
        &self.placeholder
    }

    /// Submit the input (enter key pressed).
    #[must_use]
    pub fn submit(&self) -> bool {
        !self.content.trim().is_empty()
    }

    /// Get the submitted content (trimmed).
    #[must_use]
    pub fn get_submitted(&self) -> String {
        self.content.trim().to_string()
    }

    /// Get cursor position.
    #[must_use]
    pub fn cursor(&self) -> usize {
        self.cursor
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_creation() {
        let input = Input::new();
        assert!(input.is_empty());
        assert_eq!(input.mode(), InputMode::Normal);
    }

    #[test]
    fn test_input_insertion() {
        let mut input = Input::new();
        input.insert_text("SELECT ");
        input.insert_text("* FROM users");

        assert_eq!(input.content(), "SELECT * FROM users");
        assert_eq!(input.cursor(), 19);
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = Input::new();
        input.insert_text("hello");

        assert_eq!(input.cursor, 5);

        input.move_cursor_backward();
        assert_eq!(input.cursor, 4);

        input.move_cursor_to_start();
        assert_eq!(input.cursor, 0);

        input.move_cursor_to_end();
        assert_eq!(input.cursor, 5);
    }

    #[test]
    fn test_deletion() {
        let mut input = Input::new();
        input.insert_text("hello");

        // Cursor is at position 5 after insert. delete_before_cursor()
        // removes the character before cursor (position 4 = 'o')
        input.delete_before_cursor();

        assert_eq!(input.content(), "hell");
    }

    #[test]
    fn test_clear() {
        let mut input = Input::new();
        input.insert_text("test");
        assert!(!input.is_empty());

        input.clear();
        assert!(input.is_empty());
    }

    #[test]
    fn test_submit() {
        let mut input = Input::new();
        assert!(!input.submit());

        input.insert_text("  SELECT 1  ");
        assert!(input.submit());
        assert_eq!(input.get_submitted(), "SELECT 1");
    }
}
