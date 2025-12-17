// src/lib/database.rs

//! Database connection and migration management.

use libsql::{Connection, Database};
use std::sync::Arc;

use crate::migrations;

/// Wrapper type for a database connection.
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pub db: Arc<Database>,
}

impl DatabaseConnection {
    /// Get a connection to the database.
    pub fn connect(&self) -> Result<Connection, libsql::Error> {
        self.db.connect()
    }

    /// Run all pending database migrations.
    ///
    /// This method uses the versioned migration system which:
    /// - Tracks applied migrations in a `_migrations` table
    /// - Only runs migrations that haven't been applied yet
    /// - Handles idempotent operations safely
    pub async fn run_migrations(&self) -> Result<(), libsql::Error> {
        let conn = self.connect()?;

        migrations::run_migrations(&conn)
            .await
            .map_err(|e| match e {
                migrations::MigrationError::Database(db_err) => db_err,
                migrations::MigrationError::VersionMismatch { expected, found } => {
                    libsql::Error::Misuse(format!(
                        "Migration version mismatch: expected {}, found {}",
                        expected, found
                    ))
                }
            })
    }

    /// Get the current migration version.
    pub async fn get_migration_version(&self) -> Result<i64, libsql::Error> {
        let conn = self.connect()?;

        migrations::get_current_version(&conn)
            .await
            .map_err(|e| match e {
                migrations::MigrationError::Database(db_err) => db_err,
                _ => libsql::Error::Misuse("Unexpected migration error".to_string()),
            })
    }

    /// Get pending migrations that haven't been applied yet.
    pub async fn get_pending_migrations(
        &self,
    ) -> Result<Vec<&'static migrations::Migration>, libsql::Error> {
        let conn = self.connect()?;

        migrations::get_pending_migrations(&conn)
            .await
            .map_err(|e| match e {
                migrations::MigrationError::Database(db_err) => db_err,
                _ => libsql::Error::Misuse("Unexpected migration error".to_string()),
            })
    }
}
