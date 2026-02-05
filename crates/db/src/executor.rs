//! Query execution.
//!
//! This module provides the [`QueryExecutor`] for executing queries
//! and introspecting database schemas.

use serde::{Deserialize, Serialize};
use sqlx::{Column, Row, TypeInfo};
use tokio::time::timeout;
use tracing::{debug, trace};

use crate::{
    error::DbError,
    schema::{ColumnInfo, DatabaseSchema, SchemaTable, TableType},
    DbConnection,
};

/// Result of a query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    /// Column names.
    pub columns: Vec<String>,
    /// Row data as JSON values.
    pub rows: Vec<serde_json::Map<String, serde_json::Value>>,
    /// Number of rows returned.
    pub row_count: usize,
    /// Query execution time in milliseconds.
    pub execution_time_ms: Option<u64>,
    /// Whether the result was truncated due to row limit.
    pub truncated: bool,
}

impl Default for QueryResult {
    fn default() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: 0,
            execution_time_ms: None,
            truncated: false,
        }
    }
}

/// Query executor.
///
/// Provides methods for executing SELECT queries and introspecting
/// database schemas.
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

    /// Execute a SELECT query with timeout.
    ///
    /// Validates that the query is a SELECT statement, then executes
    /// it with the configured query timeout. Results are returned
    /// as a [`QueryResult`] with columns and row data.
    ///
    /// # Errors
    /// Returns `DbError::NonSelectQuery` if the query is not a SELECT.
    /// Returns `DbError::Timeout` if the query exceeds the timeout.
    /// Returns `DbError::QueryFailed` if the query execution fails.
    pub async fn execute_query(&self, sql: &str) -> Result<QueryResult, DbError> {
        // Validate it's a SELECT query
        let normalized = sql.trim_start().to_uppercase();
        if !normalized.starts_with("SELECT") && !normalized.starts_with("WITH ") {
            debug!("Rejected non-SELECT query: {}", sql);
            return Err(DbError::NonSelectQuery {
                sql: sql.to_string(),
            });
        }

        trace!("Executing query: {}", sql);

        let pool = self.db.pool();
        let timeout_duration = self.db.query_timeout();

        let result = timeout(timeout_duration, async move {
            // Use fetch_all for simplicity - returns all rows at once
            let row_stream = sqlx::query(sql).fetch_all(pool).await?;

            let columns: Vec<String> = if let Some(first_row) = row_stream.first() {
                first_row.columns().iter().map(|c| c.name().to_string()).collect()
            } else {
                // No rows returned, try to get column info from empty query
                Vec::new()
            };

            let row_count = row_stream.len();
            let rows: Vec<serde_json::Map<String, serde_json::Value>> = row_stream
                .into_iter()
                .map(convert_row_to_json)
                .collect();

            Ok::<QueryResult, DbError>(QueryResult {
                columns,
                rows,
                row_count,
                execution_time_ms: None,
                truncated: false,
            })
        })
        .await;

        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(DbError::Timeout {
                timeout: self.db.config().query_timeout,
            }),
        }
    }

    /// Execute a SELECT query and return limited results.
    ///
    /// Similar to [`execute_query`](QueryExecutor::execute_query) but limits
    /// the number of rows returned to prevent large result sets.
    ///
    /// # Errors
    /// Same as [`execute_query`](QueryExecutor::execute_query).
    #[allow(dead_code)]
    pub async fn execute_query_limited(
        &self,
        sql: &str,
        limit: usize,
    ) -> Result<QueryResult, DbError> {
        // Validate it's a SELECT query
        let normalized = sql.trim_start().to_uppercase();
        if !normalized.starts_with("SELECT") && !normalized.starts_with("WITH ") {
            return Err(DbError::NonSelectQuery {
                sql: sql.to_string(),
            });
        }

        // Add LIMIT if not present
        let sql_with_limit = if normalized.contains("LIMIT") {
            sql.to_string()
        } else {
            format!("{} LIMIT {}", sql.trim_end().trim_end_matches(';'), limit)
        };

        trace!("Executing limited query: {}", sql_with_limit);

        let pool = self.db.pool();
        let timeout_duration = self.db.query_timeout();

        let result = timeout(timeout_duration, async move {
            let row_stream = sqlx::query(&sql_with_limit).fetch_all(pool).await?;

            let columns: Vec<String> = if let Some(first_row) = row_stream.first() {
                first_row.columns().iter().map(|c| c.name().to_string()).collect()
            } else {
                Vec::new()
            };

            let row_count = row_stream.len();
            let rows: Vec<serde_json::Map<String, serde_json::Value>> = row_stream
                .into_iter()
                .map(convert_row_to_json)
                .collect();

            Ok::<QueryResult, DbError>(QueryResult {
                columns,
                rows,
                row_count,
                execution_time_ms: None,
                truncated: row_count >= limit,
            })
        })
        .await;

        match result {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(DbError::Timeout {
                timeout: self.db.config().query_timeout,
            }),
        }
    }

    /// Introspect database schema.
    ///
    /// Retrieves information about all tables and columns in the database,
    /// optionally filtered by table name pattern.
    ///
    /// # Errors
    /// Returns `DbError::SchemaIntrospectionFailed` if the introspection fails.
    #[allow(dead_code)]
    pub async fn get_schema(
        &self,
        table_filter: Option<&str>,
    ) -> Result<DatabaseSchema, DbError> {
        debug!("Introspecting schema with filter: {:?}", table_filter);

        let pool = self.db.pool();

        // Query tables - manually map rows to SchemaTable
        let tables_sql = r#"
            SELECT
                table_schema,
                table_name,
                CASE table_type
                    WHEN 'BASE TABLE' THEN 'base_table'
                    WHEN 'VIEW' THEN 'view'
                    WHEN 'FOREIGN TABLE' THEN 'foreign_table'
                    ELSE 'base_table'
                END as table_type
            FROM information_schema.tables
            WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
            AND ($1::text IS NULL OR table_name LIKE $1 || '%')
            ORDER BY table_schema, table_name
        "#;

        let table_rows = sqlx::query(tables_sql)
            .bind(table_filter)
            .fetch_all(pool)
            .await?;

        let mut tables = Vec::new();
        for row in table_rows {
            let table_type_str: String = row.try_get(2)?;
            let table_type = match table_type_str.as_str() {
                "view" => TableType::View,
                "foreign_table" => TableType::ForeignTable,
                _ => TableType::BaseTable,
            };

            tables.push(SchemaTable {
                table_name: row.try_get(1)?,
                table_schema: row.try_get(0)?,
                table_type,
            });
        }

        // Query columns for each table
        let mut columns: Vec<(String, ColumnInfo)> = Vec::new();

        for table in &tables {
            let columns_sql = r#"
                SELECT
                    column_name,
                    data_type,
                    is_nullable,
                    column_default,
                    character_maximum_length,
                    numeric_precision,
                    numeric_scale
                FROM information_schema.columns
                WHERE table_schema = $1 AND table_name = $2
                ORDER BY ordinal_position
            "#;

            let col_rows = sqlx::query(columns_sql)
                .bind(&table.table_schema)
                .bind(&table.table_name)
                .fetch_all(pool)
                .await?;

            for row in col_rows {
                columns.push((
                    table.table_name.clone(),
                    ColumnInfo {
                        column_name: row.try_get(0)?,
                        data_type: row.try_get(1)?,
                        is_nullable: row.try_get(2)?,
                        column_default: row.try_get(3)?,
                        character_maximum_length: row.try_get(4)?,
                        numeric_precision: row.try_get(5)?,
                        numeric_scale: row.try_get(6)?,
                    },
                ));
            }
        }

        // Group columns by table name
        let mut column_map = std::collections::HashMap::new();
        for (table_name, col) in columns {
            column_map.entry(table_name).or_insert_with(Vec::new).push(col);
        }

        Ok(DatabaseSchema {
            tables,
            columns: column_map,
        })
    }

    /// List all table names.
    ///
    /// Returns a list of all table names in the database,
    /// optionally filtered by schema.
    ///
    /// # Errors
    /// Returns `DbError::QueryFailed` if the query fails.
    #[allow(dead_code)]
    pub async fn list_tables(
        &self,
        schema: Option<&str>,
    ) -> Result<Vec<String>, DbError> {
        let pool = self.db.pool();

        let schema_filter = schema.unwrap_or("public");

        let sql = r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = $1
            AND table_type = 'BASE TABLE'
            ORDER BY table_name
        "#;

        let rows: Vec<(String,)> = sqlx::query_as(sql)
            .bind(schema_filter)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                debug!("Failed to list tables: {}", e);
                crate::DbError::QueryFailed { sql: sql.to_string() }
            })?;

        Ok(rows.into_iter().map(|(t,)| t).collect())
    }

    /// Describe a specific table.
    ///
    /// Returns detailed column information for a single table.
    ///
    /// # Errors
    /// Returns `DbError::QueryFailed` if the query fails.
    #[allow(dead_code)]
    pub async fn describe_table(
        &self,
        table_name: &str,
    ) -> Result<Vec<ColumnInfo>, DbError> {
        let pool = self.db.pool();

        let sql = r#"
            SELECT
                column_name,
                data_type,
                is_nullable,
                column_default,
                character_maximum_length,
                numeric_precision,
                numeric_scale
            FROM information_schema.columns
            WHERE table_schema = 'public' AND table_name = $1
            ORDER BY ordinal_position
        "#;

        let rows = sqlx::query(sql)
            .bind(table_name)
            .fetch_all(pool)
            .await?;

        let mut columns = Vec::new();
        for row in rows {
            columns.push(ColumnInfo {
                column_name: row.try_get(0)?,
                data_type: row.try_get(1)?,
                is_nullable: row.try_get(2)?,
                column_default: row.try_get(3)?,
                character_maximum_length: row.try_get(4)?,
                numeric_precision: row.try_get(5)?,
                numeric_scale: row.try_get(6)?,
            });
        }

        Ok(columns)
    }
}

/// Convert a sqlx row to a JSON object.
fn convert_row_to_json(row: sqlx::postgres::PgRow) -> serde_json::Map<String, serde_json::Value> {
    let mut map = serde_json::Map::new();

    for (i, col) in row.columns().iter().enumerate() {
        let value = row.try_get::<serde_json::Value, _>(i).ok().unwrap_or_else(|| {
            let type_name = TypeInfo::name(col.type_info());
            serde_json::Value::String(format!("<{}>", type_name))
        });
        map.insert(Column::name(col).to_string(), value);
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_result_default() {
        let result = QueryResult::default();
        assert!(result.columns.is_empty());
        assert!(result.rows.is_empty());
        assert_eq!(result.row_count, 0);
    }
}
