//! Command palette for quick actions.
//!
//! Provides a searchable command palette for quick access to actions.

use std::fmt;

/// A command with its display information.
#[derive(Debug, Clone)]
pub struct Command {
    /// Command identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Key binding to trigger.
    pub key_binding: &'static str,
    /// Category for grouping.
    pub category: String,
}

impl Command {
    /// Create a new command.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        key_binding: &'static str,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            key_binding,
            category: category.into(),
        }
    }
}

/// Command palette state.
#[derive(Debug, Default)]
pub struct CommandPalette {
    /// All available commands.
    commands: Vec<Command>,
    /// Filtered commands based on search.
    filtered_commands: Vec<Command>,
    /// Current search query.
    search_query: String,
    /// Selection index.
    selected_index: usize,
    /// Whether the palette is visible.
    is_visible: bool,
}

impl CommandPalette {
    /// Create a new command palette.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the default set of commands.
    #[must_use]
    pub fn default_commands() -> Vec<Command> {
        vec![
            // Navigation
            Command::new(
                "nav_chat",
                "Chat View",
                "Switch to chat/conversation view",
                "Ctrl+C",
                "Navigation",
            ),
            Command::new(
                "nav_results",
                "Results View",
                "Switch to query results view",
                "Ctrl+R",
                "Navigation",
            ),
            Command::new(
                "nav_schema",
                "Schema Browser",
                "Browse database schema",
                "Ctrl+S",
                "Navigation",
            ),
            // Query
            Command::new(
                "query_execute",
                "Execute Query",
                "Execute the current query",
                "Enter",
                "Query",
            ),
            Command::new(
                "query_clear",
                "Clear Query",
                "Clear the current query",
                "Esc",
                "Query",
            ),
            // Database
            Command::new(
                "db_refresh",
                "Refresh Schema",
                "Refresh database schema cache",
                "Ctrl+F5",
                "Database",
            ),
            // Application
            Command::new(
                "app_quit",
                "Quit",
                "Exit the application",
                "Ctrl+Q",
                "Application",
            ),
            Command::new(
                "app_help",
                "Help",
                "Show help information",
                "F1",
                "Application",
            ),
        ]
    }

    /// Show the command palette.
    pub fn show(&mut self) {
        self.is_visible = true;
        self.selected_index = 0;
        self.filter_commands();
    }

    /// Hide the command palette.
    pub fn hide(&mut self) {
        self.is_visible = false;
        self.search_query.clear();
        self.filter_commands();
    }

    /// Check if visible.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Set the search query.
    pub fn set_search_query(&mut self, query: impl Into<String>) {
        self.search_query = query.into();
        self.filter_commands();
    }

    /// Get the search query.
    #[must_use]
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// Filter commands based on search query.
    fn filter_commands(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.filtered_commands = Self::default_commands();
        } else {
            self.filtered_commands = Self::default_commands()
                .into_iter()
                .filter(|cmd| {
                    cmd.name.to_lowercase().contains(&query)
                        || cmd.description.to_lowercase().contains(&query)
                        || cmd.category.to_lowercase().contains(&query)
                })
                .collect();
        }
        if self.selected_index >= self.filtered_commands.len() {
            self.selected_index = self.filtered_commands.len().saturating_sub(1);
        }
    }

    /// Move selection up.
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down.
    pub fn move_down(&mut self) {
        if self.selected_index < self.filtered_commands.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Get the selected command.
    #[must_use]
    pub fn selected_command(&self) -> Option<&Command> {
        self.filtered_commands.get(self.selected_index)
    }

    /// Get all filtered commands.
    #[must_use]
    pub fn filtered_commands(&self) -> &[Command] {
        &self.filtered_commands
    }

    /// Get selected index.
    #[must_use]
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get commands.
    #[must_use]
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }
}

impl fmt::Display for CommandPalette {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Command Palette")?;
        writeln!(f, "Search: {}", self.search_query)?;
        writeln!(f, "---")?;

        for (idx, cmd) in self.filtered_commands.iter().enumerate() {
            let prefix = if idx == self.selected_index { ">" } else { " " };
            writeln!(
                f,
                "{} {} | {} | {}",
                prefix, cmd.name, cmd.key_binding, cmd.description
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("test", "Test Command", "A test", "F1", "Test");
        assert_eq!(cmd.id, "test");
        assert_eq!(cmd.name, "Test Command");
        assert_eq!(cmd.key_binding, "F1");
    }

    #[test]
    fn test_palette_visibility() {
        let mut palette = CommandPalette::new();
        assert!(!palette.is_visible());

        palette.show();
        assert!(palette.is_visible());

        palette.hide();
        assert!(!palette.is_visible());
    }

    #[test]
    fn test_command_filtering() {
        let mut palette = CommandPalette::new();
        palette.show();

        palette.set_search_query("chat");
        assert!(!palette.filtered_commands().is_empty());

        palette.set_search_query("nonexistentxyz123");
        assert!(palette.filtered_commands().is_empty());
    }

    #[test]
    fn test_selection_movement() {
        let mut palette = CommandPalette::new();
        palette.show();

        palette.move_down();
        assert_eq!(palette.selected_index(), 1);

        palette.move_up();
        assert_eq!(palette.selected_index(), 0);
    }
}
