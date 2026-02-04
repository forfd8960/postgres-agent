//! Audit logging.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Audit event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuditEvent {
    /// Query execution.
    Query {
        timestamp: DateTime<Utc>,
        user: String,
        database: String,
        query: String,
        success: bool,
        duration_ms: u64,
    },
    /// Schema modification.
    SchemaChange {
        timestamp: DateTime<Utc>,
        user: String,
        database: String,
        operation: String,
        sql: String,
        approved: bool,
    },
    /// Safety violation.
    SafetyViolation {
        timestamp: DateTime<Utc>,
        user: String,
        query: String,
        reason: String,
    },
}

/// Audit logger.
#[derive(Debug)]
pub struct AuditLogger {
    /// Log file path.
    path: Option<PathBuf>,
    /// Whether JSON format.
    json_format: bool,
}

impl AuditLogger {
    /// Create a new audit logger.
    #[allow(dead_code)]
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            path,
            json_format: true,
        }
    }

    /// Log an audit event.
    #[allow(dead_code)]
    pub fn log(&self, event: &AuditEvent) {
        if let Some(ref path) = self.path {
            // TODO: Implement audit logging to file
            let _ = path;
        }
    }
}
