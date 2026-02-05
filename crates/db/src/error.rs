//! Database errors.

use thiserror::Error;

/// Errors from database operations.
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Failed to connect to database")]
    ConnectionFailed,

    #[error("Query failed: {sql}")]
    QueryFailed { sql: String },

    #[error("Non-SELECT query not allowed in read-only mode")]
    NonSelectQuery { sql: String },

    #[error("Query exceeded timeout of {timeout}s")]
    Timeout { timeout: u64 },

    #[error("Schema introspection failed")]
    SchemaIntrospectionFailed,

    #[error("Database error: {source}")]
    Database {
        #[from]
        source: sqlx::Error,
    },
}
