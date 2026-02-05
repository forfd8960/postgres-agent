//! Main TUI application for PostgreSQL Agent.
//!
//! Provides the terminal UI application with event handling and rendering.

use thiserror::Error;

use crate::{
    components::{CommandPalette, Input, InputMode},
    views::ChatView,
};

/// TUI errors.
#[derive(Debug, Error)]
pub enum TuiError {
    /// Terminal initialization failed.
    #[error("Failed to initialize terminal")]
    InitError,

    /// Event handling failed.
    #[error("Event handling failed: {message}")]
    EventError { message: String },
}

/// Result type for TUI operations.
pub type TuiResult<T> = Result<T, TuiError>;

/// Application state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppState {
    /// Running normally.
    #[default]
    Running,
    /// Waiting for user input.
    Waiting,
    /// Processing a query.
    Processing,
    /// Error state.
    Error,
}

/// Main TUI application (UI-agnostic core).
#[derive(Debug)]
pub struct PostgresAgentTui {
    /// Chat view.
    chat_view: ChatView,
    /// Input component.
    input: Input,
    /// Command palette.
    command_palette: CommandPalette,
    /// Current state.
    state: AppState,
    /// Current view mode.
    view_mode: ViewMode,
    /// Database profile name.
    profile: String,
    /// Safety level.
    safety_level: String,
    /// Quit flag.
    should_quit: bool,
}

/// View modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// Chat/conversation view.
    #[default]
    Chat,
    /// Query results view.
    Results,
    /// Schema browser.
    Schema,
    /// Settings view.
    Settings,
}

impl std::fmt::Display for ViewMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chat => write!(f, "Chat"),
            Self::Results => write!(f, "Results"),
            Self::Schema => write!(f, "Schema"),
            Self::Settings => write!(f, "Settings"),
        }
    }
}

impl PostgresAgentTui {
    /// Create a new TUI application.
    #[must_use]
    pub fn new() -> Self {
        Self {
            chat_view: ChatView::new(),
            input: Input::with_placeholder("Ask about your database..."),
            command_palette: CommandPalette::new(),
            state: AppState::Waiting,
            view_mode: ViewMode::Chat,
            profile: "default".to_string(),
            safety_level: "balanced".to_string(),
            should_quit: false,
        }
    }

    /// Create with initial profile and safety level.
    #[must_use]
    pub fn with_profile(profile: impl Into<String>, safety: impl Into<String>) -> Self {
        let mut tui = Self::new();
        tui.profile = profile.into();
        tui.safety_level = safety.into();
        tui
    }

    /// Run the main event loop (stub - implement with actual terminal).
    ///
    /// # Errors
    /// Returns an error if event handling fails.
    pub fn run(&mut self) -> TuiResult<()> {
        // This is a stub - actual implementation would use crossterm/ratatui
        // For now, just mark as running
        self.state = AppState::Running;
        Ok(())
    }

    /// Handle input character.
    pub fn handle_input(&mut self, c: char) {
        if self.command_palette.is_visible() {
            let mut query = self.command_palette.search_query().to_string();
            query.push(c);
            self.command_palette.set_search_query(query);
            return;
        }

        match self.input.mode() {
            InputMode::Normal => {
                if !c.is_control() {
                    self.input.insert_char(c);
                }
            }
            InputMode::Insert => {
                if !c.is_control() {
                    self.input.insert_char(c);
                }
            }
            InputMode::Command => {
                if !c.is_control() {
                    self.input.insert_char(c);
                }
            }
        }
    }

    /// Handle special key.
    pub fn handle_special_key(&mut self, key: &str) {
        match key.as_ref() {
            "Enter" => {
                if self.command_palette.is_visible() {
                    self.handle_command_palette_selection();
                } else if self.input.submit() {
                    let query = self.input.get_submitted();
                    self.chat_view.add_user_message(&query);
                    self.state = AppState::Processing;
                    self.input.clear();
                }
            }
            "Esc" => {
                if self.command_palette.is_visible() {
                    self.command_palette.hide();
                } else {
                    self.input.set_mode(InputMode::Normal);
                }
            }
            "Backspace" => {
                if self.command_palette.is_visible() {
                    let mut query = self.command_palette.search_query().to_string();
                    query.pop();
                    self.command_palette.set_search_query(query);
                } else {
                    self.input.delete_before_cursor();
                }
            }
            "ArrowUp" | "Up" => {
                if self.command_palette.is_visible() {
                    self.command_palette.move_up();
                } else {
                    self.input.move_cursor_up();
                }
            }
            "ArrowDown" | "Down" => {
                if self.command_palette.is_visible() {
                    self.command_palette.move_down();
                } else {
                    self.input.move_cursor_down();
                }
            }
            "ArrowLeft" | "Left" => self.input.move_cursor_backward(),
            "ArrowRight" | "Right" => self.input.move_cursor_forward(),
            "Home" => self.input.move_cursor_to_start(),
            "End" => self.input.move_cursor_to_end(),
            "Tab" => {
                self.input.insert_text("    ");
            }
            "Delete" => self.input.delete_after_cursor(),
            _ => {}
        }
    }

    /// Handle control key.
    pub fn handle_control_key(&mut self, c: char) {
        match c {
            'c' if self.input.mode() == InputMode::Normal => {
                self.view_mode = ViewMode::Chat;
            }
            'r' if self.input.mode() == InputMode::Normal => {
                self.view_mode = ViewMode::Results;
            }
            's' if self.input.mode() == InputMode::Normal => {
                self.view_mode = ViewMode::Schema;
            }
            'p' if self.input.mode() == InputMode::Normal => {
                self.command_palette.show();
            }
            'q' if self.input.mode() == InputMode::Normal => {
                self.should_quit = true;
            }
            'i' => self.input.set_mode(InputMode::Insert),
            _ => {}
        }
    }

    /// Handle command palette selection.
    fn handle_command_palette_selection(&mut self) {
        let cmd_id = self.command_palette.selected_command().map(|cmd| cmd.id.clone());
        if let Some(cmd) = cmd_id {
            self.handle_command(&cmd);
        }
        self.command_palette.hide();
    }

    /// Handle a command.
    fn handle_command(&mut self, cmd: &str) {
        match cmd {
            "nav_chat" => {
                self.view_mode = ViewMode::Chat;
            }
            "nav_results" => {
                self.view_mode = ViewMode::Results;
            }
            "nav_schema" => {
                self.view_mode = ViewMode::Schema;
            }
            "app_quit" => {
                self.should_quit = true;
            }
            "query_clear" => {
                self.input.clear();
            }
            "db_refresh" => {
                self.chat_view.add_assistant_message("Refreshing database schema...");
            }
            _ => {
                self.chat_view
                    .add_assistant_message(&format!("Selected: {}", cmd));
            }
        }
    }

    /// Add an assistant response to the chat.
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.chat_view.add_assistant_message(content);
        self.state = AppState::Waiting;
    }

    /// Set processing state.
    pub fn set_processing(&mut self, is_processing: bool) {
        self.state = if is_processing {
            AppState::Processing
        } else {
            AppState::Waiting
        };
    }

    /// Set error state.
    pub fn set_error(&mut self, _message: impl Into<String>) {
        self.state = AppState::Error;
    }

    /// Get the current query from input.
    #[must_use]
    pub fn current_query(&self) -> Option<String> {
        let content = self.input.content();
        if content.is_empty() {
            None
        } else {
            Some(content.to_string())
        }
    }

    /// Get the chat view.
    #[must_use]
    pub fn chat_view(&self) -> &ChatView {
        &self.chat_view
    }

    /// Get mutable chat view.
    pub fn chat_view_mut(&mut self) -> &mut ChatView {
        &mut self.chat_view
    }

    /// Get the input.
    #[must_use]
    pub fn input(&self) -> &Input {
        &self.input
    }

    /// Get mutable input.
    pub fn input_mut(&mut self) -> &mut Input {
        &mut self.input
    }

    /// Get the command palette.
    #[must_use]
    pub fn command_palette(&self) -> &CommandPalette {
        &self.command_palette
    }

    /// Get mutable command palette.
    pub fn command_palette_mut(&mut self) -> &mut CommandPalette {
        &mut self.command_palette
    }

    /// Check if should quit.
    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Get the current state.
    #[must_use]
    pub fn state(&self) -> AppState {
        self.state
    }

    /// Get the view mode.
    #[must_use]
    pub fn view_mode(&self) -> ViewMode {
        self.view_mode
    }

    /// Get the profile.
    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    /// Get the safety level.
    #[must_use]
    pub fn safety_level(&self) -> &str {
        &self.safety_level
    }
}

impl Default for PostgresAgentTui {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tui_creation() {
        let tui = PostgresAgentTui::new();
        assert!(!tui.should_quit());
    }

    #[test]
    fn test_view_mode_display() {
        assert_eq!(ViewMode::Chat.to_string(), "Chat");
        assert_eq!(ViewMode::Results.to_string(), "Results");
    }

    #[test]
    fn test_app_state_transitions() {
        let mut tui = PostgresAgentTui::new();
        assert_eq!(tui.state(), AppState::Waiting);

        tui.set_processing(true);
        assert_eq!(tui.state(), AppState::Processing);

        tui.set_processing(false);
        assert_eq!(tui.state(), AppState::Waiting);
    }

    #[test]
    fn test_query_access() {
        let mut tui = PostgresAgentTui::new();
        assert!(tui.current_query().is_none());

        tui.input_mut().insert_text("SELECT 1");
        assert_eq!(tui.current_query(), Some("SELECT 1".to_string()));
    }

    #[test]
    fn test_command_handling() {
        let mut tui = PostgresAgentTui::new();

        // Test navigation commands
        tui.handle_command("nav_chat");
        assert_eq!(tui.view_mode(), ViewMode::Chat);

        tui.handle_command("nav_results");
        assert_eq!(tui.view_mode(), ViewMode::Results);

        // Test quit
        tui.handle_command("app_quit");
        assert!(tui.should_quit());
    }
}
