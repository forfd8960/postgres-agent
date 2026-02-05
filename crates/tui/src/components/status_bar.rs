//! Status bar component for the TUI.
//!
//! Displays connection status, safety level, and other runtime information.

use std::fmt;

/// Connection status indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    /// Not connected.
    #[default]
    Disconnected,
    /// Currently connecting.
    Connecting,
    /// Connected successfully.
    Connected,
    /// Connection error.
    Error,
}

impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting..."),
            Self::Connected => write!(f, "Connected"),
            Self::Error => write!(f, "Error"),
        }
    }
}

/// Safety level indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyLevel {
    /// Read-only mode.
    ReadOnly,
    /// Balanced mode (requires confirmation).
    Balanced,
    /// Permissive mode (minimal checks).
    Permissive,
}

impl Default for SafetyLevel {
    fn default() -> Self {
        Self::Balanced
    }
}

impl fmt::Display for SafetyLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "ReadOnly"),
            Self::Balanced => write!(f, "Balanced"),
            Self::Permissive => write!(f, "Permissive"),
        }
    }
}

impl From<&str> for SafetyLevel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "read_only" | "readonly" => SafetyLevel::ReadOnly,
            "balanced" => SafetyLevel::Balanced,
            "permissive" => SafetyLevel::Permissive,
            _ => SafetyLevel::Balanced,
        }
    }
}

/// Status information to display.
#[derive(Debug, Default)]
pub struct StatusInfo {
    /// Database profile name.
    pub profile: String,
    /// Connection status.
    pub connection: ConnectionStatus,
    /// Safety level.
    pub safety: SafetyLevel,
    /// Execution time of last query.
    pub last_execution_time: Option<u64>,
    /// Number of rows returned.
    pub rows: Option<u64>,
    /// Current view mode.
    pub view_mode: String,
    /// Agent iteration count.
    pub iterations: u32,
}

impl StatusInfo {
    /// Create new status info.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the profile name.
    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = profile.into();
        self
    }

    /// Set the connection status.
    pub fn with_connection(mut self, connection: ConnectionStatus) -> Self {
        self.connection = connection;
        self
    }

    /// Set the safety level.
    pub fn with_safety(mut self, safety: SafetyLevel) -> Self {
        self.safety = safety;
        self
    }

    /// Set the last execution time.
    pub fn with_execution_time(mut self, ms: u64) -> Self {
        self.last_execution_time = Some(ms);
        self
    }

    /// Set the row count.
    pub fn with_rows(mut self, rows: u64) -> Self {
        self.rows = Some(rows);
        self
    }

    /// Set the view mode.
    pub fn with_view_mode(mut self, mode: impl Into<String>) -> Self {
        self.view_mode = mode.into();
        self
    }

    /// Set the iterations.
    pub fn with_iterations(mut self, iterations: u32) -> Self {
        self.iterations = iterations;
        self
    }
}

/// Status bar widget (UI-agnostic).
#[derive(Debug, Default)]
pub struct StatusBar {
    /// Status information to display.
    info: StatusInfo,
}

impl StatusBar {
    /// Create a new status bar.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a status bar with info.
    #[must_use]
    pub fn with_info(info: StatusInfo) -> Self {
        Self { info }
    }

    /// Update the status information.
    pub fn update(&mut self, info: StatusInfo) {
        self.info = info;
    }

    /// Get mutable access to status info.
    pub fn info_mut(&mut self) -> &mut StatusInfo {
        &mut self.info
    }

    /// Get reference to status info.
    #[must_use]
    pub fn info(&self) -> &StatusInfo {
        &self.info
    }
}

impl fmt::Display for StatusBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] | {} | {} | {}ms | {} rows | {} iter",
            self.info.profile,
            self.info.connection,
            self.info.safety,
            self.info.last_execution_time.unwrap_or(0),
            self.info.rows.unwrap_or(0),
            self.info.iterations,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_info_creation() {
        let info = StatusInfo::new()
            .with_profile("production")
            .with_connection(ConnectionStatus::Connected)
            .with_safety(SafetyLevel::Balanced)
            .with_execution_time(150)
            .with_rows(42)
            .with_view_mode("Chat");

        assert_eq!(info.profile, "production");
        assert_eq!(info.connection, ConnectionStatus::Connected);
        assert_eq!(info.rows, Some(42));
    }

    #[test]
    fn test_safety_level_from_str() {
        assert!(matches!(SafetyLevel::from("read_only"), SafetyLevel::ReadOnly));
        assert!(matches!(SafetyLevel::from("balanced"), SafetyLevel::Balanced));
        assert!(matches!(SafetyLevel::from("permissive"), SafetyLevel::Permissive));
    }

    #[test]
    fn test_connection_status_display() {
        assert_eq!(ConnectionStatus::Connected.to_string(), "Connected");
        assert_eq!(ConnectionStatus::Disconnected.to_string(), "Disconnected");
    }

    #[test]
    fn test_status_bar_display() {
        let info = StatusInfo::new()
            .with_profile("test")
            .with_connection(ConnectionStatus::Connected);

        let bar = StatusBar::with_info(info);
        let display = bar.to_string();
        assert!(display.contains("test"));
        assert!(display.contains("Connected"));
    }
}
