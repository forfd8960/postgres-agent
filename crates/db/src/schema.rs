//! Schema introspection types.
//!
//! This module provides types for representing database schema information,
//! including tables, columns, and their metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Table information from schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaTable {
    /// Table name.
    #[serde(default)]
    pub table_name: String,
    /// Table schema.
    #[serde(default)]
    pub table_schema: String,
    /// Table type.
    #[serde(default)]
    pub table_type: TableType,
}

impl Default for SchemaTable {
    fn default() -> Self {
        Self {
            table_name: String::new(),
            table_schema: String::new(),
            table_type: TableType::BaseTable,
        }
    }
}

/// Column information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnInfo {
    /// Column name.
    #[serde(default)]
    pub column_name: String,
    /// Data type.
    #[serde(default)]
    pub data_type: String,
    /// Whether nullable.
    #[serde(default)]
    pub is_nullable: bool,
    /// Default value.
    #[serde(default)]
    pub column_default: Option<String>,
    /// Maximum character length.
    #[serde(default)]
    pub character_maximum_length: Option<i64>,
    /// Numeric precision.
    #[serde(default)]
    pub numeric_precision: Option<i64>,
    /// Numeric scale.
    #[serde(default)]
    pub numeric_scale: Option<i64>,
}

impl Default for ColumnInfo {
    fn default() -> Self {
        Self {
            column_name: String::new(),
            data_type: String::new(),
            is_nullable: true,
            column_default: None,
            character_maximum_length: None,
            numeric_precision: None,
            numeric_scale: None,
        }
    }
}

/// Type of table.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TableType {
    /// Regular table.
    #[default]
    BaseTable,
    /// Database view.
    View,
    /// Foreign table.
    ForeignTable,
}

/// Complete database schema.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSchema {
    /// Tables in the schema.
    #[serde(default)]
    pub tables: Vec<SchemaTable>,
    /// Columns by table name.
    #[serde(default)]
    pub columns: HashMap<String, Vec<ColumnInfo>>,
}

impl DatabaseSchema {
    /// Create a new empty database schema.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a table by name.
    #[must_use]
    pub fn get_table(&self, name: &str) -> Option<&SchemaTable> {
        self.tables.iter().find(|t| t.table_name == name)
    }

    /// Get columns for a specific table.
    #[must_use]
    pub fn get_columns(&self, table_name: &str) -> Option<&Vec<ColumnInfo>> {
        self.columns.get(table_name)
    }
}
