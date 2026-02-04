//! Query execution.

use serde::{Deserialize, Serialize};

use crate::{error::DbError, schema::DatabaseSchema, DbConnection};

/// Result of a query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    /// Column names.
    pub columns: Vec<String>,
    /// Row data as JSON values.
    pub rows: Vec<Vec<serde_json::Value>>,
    /// Number of rows returned.
    pub row_count: usize,
}

/// Query executor.
#[derive(Debug)]
pub struct QueryExecutor {
    /// Database connection.
    db: DbConnection,
}

impl QueryExecutor {
    /// Create a new query executor.
    #[must_use]
    pub fn new(db: DbConnection) -> Self {
        Self { db }
    }

    /// Execute a SELECT query.
    ///
    /// # Errors
    /// Returns `DbError::NonSelectQuery` if the query is not a SELECT.
    #[allow(dead_code)]
    pub async fn execute_query(
        &self,
        sql: &str,
    ) -> Result<QueryResult, DbError> {
        // Validate it's a SELECT query
        let normalized = sql.trim_start().to_uppercase();
        if !normalized.starts_with("SELECT") {
            return Err(DbError::NonSelectQuery {
                sql: sql.to_string(),
            });
        }

        // TODO: Execute query and return results
        Ok(QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: 0,
        })
    }

    /// Introspect database schema.
    #[allow(dead_code)]
    pub async fn get_schema(
        &self,
        _table_filter: Option<&str>,
    ) -> Result<DatabaseSchema, DbError> {
        // TODO: Implement schema introspection
        Ok(DatabaseSchema::default())
    }
}
