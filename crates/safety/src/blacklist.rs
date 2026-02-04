//! SQL blacklist patterns.

use lazy_static::lazy_static;
use regex::Regex;

/// Blacklist patterns for dangerous SQL operations.
#[derive(Debug, Default)]
pub struct SqlBlacklist {
    /// List of compiled regex patterns.
    patterns: Vec<Regex>,
}

impl SqlBlacklist {
    /// Create a new SQL blacklist with default patterns.
    #[must_use]
    pub fn new() -> Self {
        let patterns = vec![
            // DROP operations
            Regex::new(r"(?i)DROP\s+(TABLE|DATABASE|SCHEMA|EVENT|INDEX|PROCEDURE|FUNCTION|TRIGGER|VIEW)")
                .unwrap(),
            // TRUNCATE operations
            Regex::new(r"(?i)TRUNCATE\s+").unwrap(),
            // DELETE without WHERE
            Regex::new(r"(?i)DELETE\s+FROM\s+\w+\s*(?!.*WHERE)").unwrap(),
            // GRANT/REVOKE
            Regex::new(r"(?i)(GRANT|REVOKE)\s+").unwrap(),
            // EXECUTE (potential code injection)
            Regex::new(r"(?i)EXECUTE\s*\(").unwrap(),
        ];
        Self { patterns }
    }

    /// Check if SQL contains blacklisted patterns.
    #[must_use]
    pub fn contains_blacklisted(&self, sql: &str) -> bool {
        self.patterns.iter().any(|p| p.is_match(sql))
    }

    /// Get the first matching blacklisted pattern.
    #[must_use]
    pub fn find_match(&self, sql: &str) -> Option<String> {
        self.patterns
            .iter()
            .find_map(|p| {
                p.find(sql).map(|m| {
                    let matched = m.as_str().to_string();
                    // Extract just the command type
                    matched.split_whitespace().next().unwrap_or("").to_string()
                })
            })
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
