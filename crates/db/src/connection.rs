//! Database connection management.
//!
//! This module provides the [`DbConnection`] wrapper around sqlx's PgPool,
//! handling connection pooling, lifecycle management, and configuration.

use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgConnectOptions, PgPool};
use std::time::Duration;
use tracing::debug;

/// Database connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbConnectionConfig {
    /// Database connection URL or connection string.
    #[serde(default = "default_url")]
    pub url: String,
    /// Database host (alternative to url).
    #[serde(default)]
    pub host: Option<String>,
    /// Database port (alternative to url).
    #[serde(default)]
    pub port: Option<u16>,
    /// Database username.
    #[serde(default)]
    pub username: Option<String>,
    /// Database password (as plain string for serialization).
    #[serde(default)]
    pub password: Option<String>,
    /// Database name.
    #[serde(default)]
    pub database: Option<String>,
    /// SSL mode.
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: SslMode,
    /// Maximum pool size.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// Minimum idle connections.
    #[serde(default = "default_min_idle_connections")]
    pub min_idle_connections: u32,
    /// Connection timeout in seconds.
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
    /// Query execution timeout in seconds.
    #[serde(default = "default_query_timeout")]
    pub query_timeout: u64,
}

fn default_url() -> String {
    "postgres://postgres:postgres@localhost:5432/postgres".to_string()
}

fn default_ssl_mode() -> SslMode {
    SslMode::Prefer
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_idle_connections() -> u32 {
    2
}

fn default_connect_timeout() -> u64 {
    30
}

fn default_query_timeout() -> u64 {
    60
}

impl Default for DbConnectionConfig {
    fn default() -> Self {
        Self {
            url: default_url(),
            host: None,
            port: None,
            username: None,
            password: None,
            database: None,
            ssl_mode: default_ssl_mode(),
            max_connections: default_max_connections(),
            min_idle_connections: default_min_idle_connections(),
            connect_timeout: default_connect_timeout(),
            query_timeout: default_query_timeout(),
        }
    }
}

/// SSL mode for database connections.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SslMode {
    /// No SSL connection.
    #[default]
    Disable,
    /// Prefer SSL if available.
    Prefer,
    /// Require SSL connection.
    Require,
}

impl From<SslMode> for sqlx::postgres::PgSslMode {
    fn from(mode: SslMode) -> Self {
        match mode {
            SslMode::Disable => Self::Disable,
            SslMode::Prefer => Self::Prefer,
            SslMode::Require => Self::Require,
        }
    }
}

impl DbConnectionConfig {
    /// Build sqlx [`PgConnectOptions`] from this configuration.
    ///
    /// # Errors
    /// Returns an error if the connection string or options are invalid.
    pub fn to_connect_options(&self) -> Result<PgConnectOptions, crate::DbError> {
        // If URL is provided and host/username are not set, parse from URL
        if let (None, None, None, None) = (
            &self.host,
            &self.username,
            &self.password,
            &self.database,
        ) {
            let options: PgConnectOptions = self.url.parse().map_err(|_| {
                debug!("Failed to parse connection URL: {}", self.url);
                crate::DbError::ConnectionFailed
            })?;
            return Ok(options);
        }

        // Build from individual components
        let mut options = PgConnectOptions::new();

        if let Some(host) = &self.host {
            options = options.host(host);
        }

        if let Some(port) = self.port {
            options = options.port(port);
        }

        if let Some(username) = &self.username {
            options = options.username(username);
        }

        if let Some(password) = &self.password {
            options = options.password(password);
        }

        if let Some(database) = &self.database {
            options = options.database(database);
        }

        options = options.ssl_mode(self.ssl_mode.into());

        Ok(options)
    }
}

/// PostgreSQL connection pool wrapper.
///
/// This wrapper manages a sqlx [`PgPool`] and provides convenience methods
/// for connection lifecycle management.
#[derive(Debug, Clone)]
pub struct DbConnection {
    /// Connection configuration.
    config: DbConnectionConfig,
    /// SQLx connection pool.
    pool: PgPool,
}

impl DbConnection {
    /// Create a new connection from configuration.
    ///
    /// Establishes a connection pool using sqlx. The pool is created
    /// immediately but connections are lazily established as needed.
    ///
    /// # Errors
    /// Returns `DbError::ConnectionFailed` if the pool cannot be created,
    /// typically due to an invalid connection string or unreachable database.
    pub async fn new(config: &DbConnectionConfig) -> Result<Self, crate::DbError> {
        debug!(
            "Creating database connection pool: max={}, timeout={}s",
            config.max_connections, config.connect_timeout
        );

        let connect_options = config.to_connect_options()?;

        let pool = PgPool::connect_with(connect_options)
            .await
            .map_err(|e| {
                debug!("Failed to create connection pool: {}", e);
                crate::DbError::ConnectionFailed
            })?;

        Ok(Self {
            config: config.clone(),
            pool,
        })
    }

    /// Create a new connection from a connection URL string.
    ///
    /// Convenience method for simple connection scenarios.
    ///
    /// # Errors
    /// Returns `DbError::ConnectionFailed` if the pool cannot be created.
    #[allow(dead_code)]
    pub async fn from_url(url: &str) -> Result<Self, crate::DbError> {
        let config = DbConnectionConfig {
            url: url.to_string(),
            ..Default::default()
        };
        Self::new(&config).await
    }

    /// Get the connection pool reference.
    ///
    /// Provides access to the underlying sqlx pool for advanced operations.
    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get the connection configuration.
    #[must_use]
    pub fn config(&self) -> &DbConnectionConfig {
        &self.config
    }

    /// Check if the connection is healthy.
    ///
    /// Executes a simple query to verify connectivity.
    ///
    /// # Errors
    /// Returns an error if the database is not reachable.
    pub async fn health_check(&self) -> Result<(), crate::DbError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| {
                debug!("Health check failed: {}", e);
                crate::DbError::ConnectionFailed
            })
    }

    /// Get the query timeout duration.
    #[must_use]
    pub fn query_timeout(&self) -> Duration {
        Duration::from_secs(self.config.query_timeout)
    }

    /// Close all connections in the pool.
    ///
    /// This method gracefully closes all connections. After calling this,
    /// the connection pool cannot be used again.
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DbConnectionConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.connect_timeout, 30);
        assert_eq!(config.query_timeout, 60);
        assert_eq!(config.ssl_mode, SslMode::Prefer);
    }

    #[test]
    fn test_ssl_mode_conversion() {
        assert_eq!(sqlx::postgres::PgSslMode::from(SslMode::Disable), sqlx::postgres::PgSslMode::Disable);
        assert_eq!(sqlx::postgres::PgSslMode::from(SslMode::Prefer), sqlx::postgres::PgSslMode::Prefer);
        assert_eq!(sqlx::postgres::PgSslMode::from(SslMode::Require), sqlx::postgres::PgSslMode::Require);
    }
}
