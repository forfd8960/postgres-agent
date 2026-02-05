//! Database layer for PostgreSQL Agent.
//!
//! Provides PostgreSQL connection management, query execution,
//! and schema introspection.

#![warn(missing_docs)]

pub mod connection;
pub mod error;
pub mod executor;
pub mod schema;

pub use connection::{DbConnection, DbConnectionConfig, SslMode};
pub use error::DbError;
pub use executor::QueryExecutor;
pub use schema::{ColumnInfo, DatabaseSchema, SchemaTable, TableType};
