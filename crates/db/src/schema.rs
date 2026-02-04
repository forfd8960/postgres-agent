//! Schema introspection types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Table information from schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaTable {
    /// Table name.
    pub table_name: String,
    /// Table schema.
    pub table_schema: String,
    /// Table type.
    pub table_type: TableType,
}

/// Column information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnInfo {
    /// Column name.
    pub column_name: String,
    /// Data type.
    pub data_type: String,
    /// Whether nullable.
    pub is_nullable: bool,
    /// Default value.
    pub column_default: Option<String>,
    /// Maximum character length.
    pub character_maximum_length: Option<i64>,
    /// Numeric precision.
    pub numeric_precision: Option<i64>,
    /// Numeric scale.
    pub numeric_scale: Option<i64>,
}

/// Type of table.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TableType {
    /// Regular table.
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
    pub tables: Vec<SchemaTable>,
    /// Columns by table name.
    pub columns: HashMap<String, Vec<ColumnInfo>>,
}
