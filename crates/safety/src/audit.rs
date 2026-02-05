//! Audit logging.
//!
//! This module provides the [`AuditLogger`] for logging all database operations,
//! safety violations, and schema changes for compliance and debugging.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::debug;

/// Audit event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum AuditEvent {
    /// Query execution.
    Query {
        /// When the event occurred.
        timestamp: DateTime<Utc>,
        /// User who executed the query.
        user: String,
        /// Target database.
        database: String,
        /// The SQL query that was executed.
        query: String,
        /// Whether the query succeeded.
        success: bool,
        /// Duration in milliseconds.
        duration_ms: u64,
        /// Number of rows affected (if available).
        rows_affected: Option<i64>,
    },
    /// Schema modification.
    SchemaChange {
        /// When the change occurred.
        timestamp: DateTime<Utc>,
        /// User who made the change.
        user: String,
        /// Target database.
        database: String,
        /// Type of operation.
        operation: String,
        /// The SQL that was executed.
        sql: String,
        /// Whether the change was approved.
        approved: bool,
    },
    /// Safety violation detected.
    SafetyViolation {
        /// When the violation occurred.
        timestamp: DateTime<Utc>,
        /// User who triggered the violation.
        user: String,
        /// The blocked query.
        query: String,
        /// Reason for the violation.
        reason: String,
        /// Safety level at the time.
        safety_level: String,
    },
    /// Confirmation request.
    ConfirmationRequest {
        /// When the request was made.
        timestamp: DateTime<Utc>,
        /// User who requested confirmation.
        user: String,
        /// Operation being confirmed.
        operation: String,
        /// Confirmation level required.
        level: String,
        /// Whether confirmation was granted.
        granted: bool,
    },
}

/// Serialized audit record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRecord {
    /// Event timestamp.
    pub timestamp: DateTime<Utc>,
    /// Event type.
    pub event_type: String,
    /// Event data.
    pub data: serde_json::Value,
}

/// Audit logger configuration.
#[derive(Debug, Clone, Default)]
pub struct AuditConfig {
    /// Log file path (None = stdout only).
    pub path: Option<PathBuf>,
    /// Whether to use JSON format.
    pub json_format: bool,
    /// Maximum log file size in bytes (0 = unlimited).
    pub max_file_size: u64,
    /// Whether to include PII in logs (should be false).
    pub include_pii: bool,
}

impl AuditConfig {
    /// Create a config with a log file.
    #[must_use]
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            path: Some(path),
            json_format: true,
            max_file_size: 0,
            include_pii: false,
        }
    }

    /// Create a config with JSON formatting disabled (human-readable).
    #[must_use]
    pub fn human_readable(path: Option<PathBuf>) -> Self {
        Self {
            path,
            json_format: false,
            max_file_size: 0,
            include_pii: false,
        }
    }
}

/// Thread-safe audit logger.
#[derive(Debug)]
pub struct AuditLogger {
    /// Logger configuration.
    config: AuditConfig,
    /// Output file (protected by mutex for safe concurrent access).
    file: Option<Mutex<File>>,
    /// Current file size (for rotation).
    current_size: Mutex<u64>,
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(AuditConfig::default())
    }
}

impl AuditLogger {
    /// Create a new audit logger.
    pub fn new(config: AuditConfig) -> Self {
        let file = config.path.as_ref().and_then(|path| {
            match OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                Ok(f) => {
                    debug!("Audit log file opened: {:?}", path);
                    Some(Mutex::new(f))
                }
                Err(e) => {
                    debug!("Failed to open audit log file: {}", e);
                    None
                }
            }
        });

        Self {
            config,
            file,
            current_size: Mutex::new(0),
        }
    }

    /// Create a logger that writes to stdout.
    #[must_use]
    pub fn stdout() -> Self {
        Self::new(AuditConfig {
            path: None,
            json_format: true,
            max_file_size: 0,
            include_pii: false,
        })
    }

    /// Log an audit event.
    pub fn log(&self, event: &AuditEvent) {
        let record = self.serialize_event(event);

        // Write to file if configured
        if let Some(ref file_mutex) = self.file {
            if let Ok(mut file) = file_mutex.lock() {
                self.write_to_file(&record, &mut file);
            }
        }

        // Also write to stdout for containerized environments
        if self.config.path.is_none() {
            self.write_stdout(&record);
        }
    }

    /// Log a query execution.
    pub fn log_query(
        &self,
        user: &str,
        database: &str,
        query: &str,
        success: bool,
        duration_ms: u64,
        rows_affected: Option<i64>,
    ) {
        let event = AuditEvent::Query {
            timestamp: Utc::now(),
            user: user.to_string(),
            database: database.to_string(),
            query: self.sanitize_query(query),
            success,
            duration_ms,
            rows_affected,
        };
        self.log(&event);
    }

    /// Log a schema change.
    pub fn log_schema_change(
        &self,
        user: &str,
        database: &str,
        operation: &str,
        sql: &str,
        approved: bool,
    ) {
        let event = AuditEvent::SchemaChange {
            timestamp: Utc::now(),
            user: user.to_string(),
            database: database.to_string(),
            operation: operation.to_string(),
            sql: self.sanitize_query(sql),
            approved,
        };
        self.log(&event);
    }

    /// Log a safety violation.
    pub fn log_safety_violation(
        &self,
        user: &str,
        query: &str,
        reason: &str,
        safety_level: &str,
    ) {
        let event = AuditEvent::SafetyViolation {
            timestamp: Utc::now(),
            user: user.to_string(),
            query: self.sanitize_query(query),
            reason: reason.to_string(),
            safety_level: safety_level.to_string(),
        };
        self.log(&event);
    }

    /// Serialize an event to a record.
    fn serialize_event(&self, event: &AuditEvent) -> AuditRecord {
        let timestamp = match event {
            AuditEvent::Query { timestamp, .. } => *timestamp,
            AuditEvent::SchemaChange { timestamp, .. } => *timestamp,
            AuditEvent::SafetyViolation { timestamp, .. } => *timestamp,
            AuditEvent::ConfirmationRequest { timestamp, .. } => *timestamp,
        };

        let event_type = match event {
            AuditEvent::Query { .. } => "query",
            AuditEvent::SchemaChange { .. } => "schema_change",
            AuditEvent::SafetyViolation { .. } => "safety_violation",
            AuditEvent::ConfirmationRequest { .. } => "confirmation_request",
        };

        let data = serde_json::to_value(event).unwrap_or_else(|_| serde_json::json!({}));

        AuditRecord {
            timestamp,
            event_type: event_type.to_string(),
            data,
        }
    }

    /// Write a record to the file.
    fn write_to_file(&self, record: &AuditRecord, file: &mut File) {
        if self.config.json_format {
            if let Ok(line) = serde_json::to_string(record) {
                let _ = writeln!(file, "{}", line);
                let _ = file.flush();
            }
        } else {
            // Human-readable format
            let line = format!(
                "[{}] {}: {}\n",
                record.timestamp,
                record.event_type,
                serde_json::to_string_pretty(&record.data).unwrap_or_default()
            );
            let _ = writeln!(file, "{}", line);
            let _ = file.flush();
        }
    }

    /// Write a record to stdout.
    fn write_stdout(&self, record: &AuditRecord) {
        if self.config.json_format {
            if let Ok(line) = serde_json::to_string(record) {
                println!("{}", line);
            }
        } else {
            println!(
                "[AUDIT] {} {}",
                record.event_type,
                serde_json::to_string_pretty(&record.data).unwrap_or_default()
            );
        }
    }

    /// Sanitize a query for logging (remove sensitive data).
    fn sanitize_query(&self, query: &str) -> String {
        if self.config.include_pii {
            return query.to_string();
        }

        // Basic sanitization - remove obvious sensitive patterns
        // In production, you'd want more comprehensive sanitization
        let sanitized = regex::Regex::new(r"(?i)(password|secret|token|api_key|auth)[\s]*=[\s]*[^\s,;]+")
            .ok()
            .and_then(|re| {
                if re.is_match(query) {
                    Some(re.replace_all(query, "$1=[REDACTED]").to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| query.to_string());

        sanitized
    }
}

/// Create a default audit logger.
#[must_use]
pub fn create_default_logger() -> AuditLogger {
    AuditLogger::stdout()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_serialization() {
        let event = AuditEvent::Query {
            timestamp: Utc::now(),
            user: "test_user".to_string(),
            database: "test_db".to_string(),
            query: "SELECT 1".to_string(),
            success: true,
            duration_ms: 5,
            rows_affected: Some(1),
        };

        let logger = AuditLogger::stdout();
        let record = logger.serialize_event(&event);

        assert_eq!(record.event_type, "query");
        assert!(!serde_json::to_string(&record.data).unwrap().is_empty());
    }

    #[test]
    fn test_query_sanitization() {
        let logger = AuditLogger::stdout();

        let with_password = "SELECT * FROM users WHERE password = 'secret123'";
        let sanitized = logger.sanitize_query(with_password);

        assert!(sanitized.contains("password=[REDACTED]"));
    }
}
