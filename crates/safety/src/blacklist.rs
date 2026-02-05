//! SQL blacklist patterns.
//!
//! This module provides SQL blacklist patterns for detecting dangerous operations.
//! Note: The regex crate used here doesn't support look-around assertions,
//! so some patterns are simplified and additional validation may be needed.

use lazy_static::lazy_static;
use regex::Regex;

/// Blacklist patterns for dangerous SQL operations.
#[derive(Debug, Default)]
pub struct SqlBlacklist {
    /// List of compiled regex patterns with their names.
    patterns: Vec<(Regex, &'static str)>,
}

impl SqlBlacklist {
    /// Create a new SQL blacklist with default patterns.
    #[must_use]
    pub fn new() -> Self {
        let patterns = vec![
            // DROP operations
            (
                Regex::new(r"(?i)^DROP\s+").unwrap(),
                "DROP",
            ),
            // TRUNCATE operations
            (
                Regex::new(r"(?i)^TRUNCATE\s+").unwrap(),
                "TRUNCATE",
            ),
            // DELETE operations (all deletes are flagged - user must add WHERE explicitly)
            (
                Regex::new(r"(?i)^DELETE\s+").unwrap(),
                "DELETE",
            ),
            // GRANT/REVOKE
            (
                Regex::new(r"(?i)^(GRANT|REVOKE)\s+").unwrap(),
                "GRANT/REVOKE",
            ),
            // EXECUTE (potential code injection)
            (
                Regex::new(r"(?i)EXECUTE\s*\(").unwrap(),
                "EXECUTE",
            ),
        ];
        Self { patterns }
    }

    /// Check if SQL contains blacklisted patterns.
    #[must_use]
    pub fn contains_blacklisted(&self, sql: &str) -> bool {
        self.patterns.iter().any(|(p, _)| p.is_match(sql.trim_start()))
    }

    /// Get the first matching blacklisted pattern name.
    #[must_use]
    pub fn find_match(&self, sql: &str) -> Option<String> {
        let trimmed = sql.trim_start();
        for (pattern, name) in &self.patterns {
            if pattern.is_match(trimmed) {
                return Some((*name).to_string());
            }
        }
        None
    }
}

lazy_static! {
    /// Default SQL blacklist instance.
    pub static ref DEFAULT_BLACKLIST: SqlBlacklist = SqlBlacklist::new();
}

/// Get the default SQL blacklist.
#[must_use]
pub fn default_blacklist() -> SqlBlacklist {
    SqlBlacklist::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drop_detection() {
        let blacklist = SqlBlacklist::new();
        assert!(blacklist.contains_blacklisted("DROP TABLE users"));
        assert_eq!(blacklist.find_match("DROP TABLE users"), Some("DROP".to_string()));
    }

    #[test]
    fn test_truncate_detection() {
        let blacklist = SqlBlacklist::new();
        assert!(blacklist.contains_blacklisted("TRUNCATE TABLE users"));
        assert_eq!(blacklist.find_match("TRUNCATE TABLE users"), Some("TRUNCATE".to_string()));
    }

    #[test]
    fn test_delete_detection() {
        let blacklist = SqlBlacklist::new();
        assert!(blacklist.contains_blacklisted("DELETE FROM users"));
        assert_eq!(blacklist.find_match("DELETE FROM users"), Some("DELETE".to_string()));
    }

    #[test]
    fn test_grant_detection() {
        let blacklist = SqlBlacklist::new();
        assert!(blacklist.contains_blacklisted("GRANT SELECT ON users TO app"));
        assert_eq!(blacklist.find_match("GRANT SELECT ON users TO app"), Some("GRANT/REVOKE".to_string()));
    }

    #[test]
    fn test_execute_detection() {
        let blacklist = SqlBlacklist::new();
        assert!(blacklist.contains_blacklisted("EXECUTE (some_function)"));
        assert_eq!(blacklist.find_match("EXECUTE (some_function)"), Some("EXECUTE".to_string()));
    }

    #[test]
    fn test_select_allowed() {
        let blacklist = SqlBlacklist::new();
        assert!(!blacklist.contains_blacklisted("SELECT * FROM users"));
        assert_eq!(blacklist.find_match("SELECT * FROM users"), None);
    }

    #[test]
    fn test_whitespace_handling() {
        let blacklist = SqlBlacklist::new();
        assert!(blacklist.contains_blacklisted("  DELETE FROM users"));
        assert!(!blacklist.contains_blacklisted("  SELECT * FROM users"));
    }
}
