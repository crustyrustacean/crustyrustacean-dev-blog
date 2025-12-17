// src/lib/migrations/mod.rs

//! Database migration system with version tracking.
//!
//! This module provides a migration runner that:
//! - Tracks applied migrations in a `_migrations` table
//! - Runs migrations in order by version number
//! - Supports idempotent migrations (safe to run multiple times)
//! - Stores SQL in separate files for easier review

use libsql::Connection;
use tracing::{info, warn};

/// A database migration.
pub struct Migration {
    /// Unique version number (should be sequential).
    pub version: i64,
    /// Short descriptive name for the migration.
    pub name: &'static str,
    /// SQL to apply the migration.
    pub up: &'static str,
    /// Optional SQL to rollback (not currently used but available for future).
    pub down: Option<&'static str>,
}

/// All migrations in order.
pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "create_users_table",
        up: include_str!("sql/001_create_users.sql"),
        down: Some("DROP TABLE IF EXISTS users"),
    },
    Migration {
        version: 2,
        name: "create_articles_table",
        up: include_str!("sql/002_create_articles.sql"),
        down: Some("DROP TABLE IF EXISTS articles"),
    },
    Migration {
        version: 3,
        name: "create_comments_table",
        up: include_str!("sql/003_create_comments.sql"),
        down: Some("DROP TABLE IF EXISTS comments"),
    },
    Migration {
        version: 4,
        name: "create_tags_table",
        up: include_str!("sql/004_create_tags.sql"),
        down: Some("DROP TABLE IF EXISTS tags; DROP TABLE IF EXISTS article_tags"),
    },
    Migration {
        version: 5,
        name: "create_categories_table",
        up: include_str!("sql/005_create_categories.sql"),
        down: Some("DROP TABLE IF EXISTS categories"),
    },
    Migration {
        version: 6,
        name: "create_social_tables",
        up: include_str!("sql/006_create_social_tables.sql"),
        down: Some("DROP TABLE IF EXISTS user_favorites; DROP TABLE IF EXISTS user_follows"),
    },
    Migration {
        version: 7,
        name: "create_api_keys_table",
        up: include_str!("sql/007_create_api_keys.sql"),
        down: Some("DROP TABLE IF EXISTS api_keys"),
    },
    Migration {
        version: 8,
        name: "create_media_tables",
        up: include_str!("sql/008_create_media_tables.sql"),
        down: Some("DROP TABLE IF EXISTS media_usage; DROP TABLE IF EXISTS media_library"),
    },
    Migration {
        version: 9,
        name: "create_newsletter_tables",
        up: include_str!("sql/009_create_newsletter_tables.sql"),
        down: Some("DROP TABLE IF EXISTS newsletter_delivery_logs; DROP TABLE IF EXISTS newsletter_issues; DROP TABLE IF EXISTS newsletter_subscribers"),
    },
    Migration {
        version: 10,
        name: "create_token_tables",
        up: include_str!("sql/010_create_token_tables.sql"),
        down: Some("DROP TABLE IF EXISTS email_verification_tokens; DROP TABLE IF EXISTS password_reset_tokens"),
    },
    Migration {
        version: 11,
        name: "create_indexes",
        up: include_str!("sql/011_create_indexes.sql"),
        down: None, // Indexes don't need rollback
    },
    Migration {
        version: 12,
        name: "add_user_columns",
        up: include_str!("sql/012_add_user_columns.sql"),
        down: None, // SQLite doesn't support DROP COLUMN easily
    },
    Migration {
        version: 13,
        name: "add_article_columns",
        up: include_str!("sql/013_add_article_columns.sql"),
        down: None,
    },
    Migration {
        version: 14,
        name: "add_performance_indexes",
        up: include_str!("sql/014_add_performance_indexes.sql"),
        down: None, // Indexes don't need rollback
    },
];

/// Error type for migration operations.
#[derive(Debug)]
pub enum MigrationError {
    Database(libsql::Error),
    VersionMismatch { expected: i64, found: i64 },
}

impl From<libsql::Error> for MigrationError {
    fn from(err: libsql::Error) -> Self {
        MigrationError::Database(err)
    }
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::Database(e) => write!(f, "Database error: {}", e),
            MigrationError::VersionMismatch { expected, found } => {
                write!(f, "Version mismatch: expected {}, found {}", expected, found)
            }
        }
    }
}

impl std::error::Error for MigrationError {}

/// Check if a migration has been applied.
async fn is_migration_applied(conn: &Connection, version: i64) -> Result<bool, MigrationError> {
    let mut rows = conn
        .query(
            "SELECT version FROM _migrations WHERE version = ?",
            libsql::params![version],
        )
        .await?;

    Ok(rows.next().await?.is_some())
}

/// Record that a migration has been applied.
async fn record_migration(
    conn: &Connection,
    version: i64,
    name: &str,
) -> Result<(), MigrationError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO _migrations (version, name, applied_at) VALUES (?, ?, ?)",
        libsql::params![version, name, now],
    )
    .await?;
    Ok(())
}

/// Get the current migration version.
pub async fn get_current_version(conn: &Connection) -> Result<i64, MigrationError> {
    let mut rows = conn
        .query(
            "SELECT MAX(version) FROM _migrations",
            (),
        )
        .await?;

    if let Some(row) = rows.next().await? {
        let version: Option<i64> = row.get(0).ok();
        Ok(version.unwrap_or(0))
    } else {
        Ok(0)
    }
}

/// Run all pending migrations.
pub async fn run_migrations(conn: &Connection) -> Result<(), MigrationError> {
    // Create migrations tracking table if it doesn't exist
    conn.execute(
        r"CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
        (),
    )
    .await?;

    let mut applied_count = 0;

    // Run each migration if not already applied
    for migration in MIGRATIONS {
        if !is_migration_applied(conn, migration.version).await? {
            info!(
                "Running migration {}: {}",
                migration.version, migration.name
            );

            // Execute each statement in the migration
            // Split by semicolons but be careful with empty statements
            for statement in migration.up.split(';') {
                let statement = statement.trim();
                if !statement.is_empty() {
                    // Some statements might fail if they're adding columns that already exist
                    // We handle this gracefully for ALTER TABLE statements
                    if statement.to_uppercase().contains("ALTER TABLE") {
                        if let Err(e) = conn.execute(statement, ()).await {
                            warn!(
                                "ALTER TABLE statement may have failed (column might exist): {}",
                                e
                            );
                        }
                    } else {
                        conn.execute(statement, ()).await?;
                    }
                }
            }

            // Record the migration as applied
            record_migration(conn, migration.version, migration.name).await?;
            applied_count += 1;
        }
    }

    if applied_count > 0 {
        info!("Applied {} migration(s)", applied_count);
    } else {
        info!("Database is up to date (no pending migrations)");
    }

    Ok(())
}

/// Get list of pending migrations.
pub async fn get_pending_migrations(conn: &Connection) -> Result<Vec<&'static Migration>, MigrationError> {
    // Ensure migrations table exists
    conn.execute(
        r"CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )",
        (),
    )
    .await?;

    let mut pending = Vec::new();
    for migration in MIGRATIONS {
        if !is_migration_applied(conn, migration.version).await? {
            pending.push(migration);
        }
    }
    Ok(pending)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_are_sequential() {
        let mut last_version = 0;
        for migration in MIGRATIONS {
            assert!(
                migration.version > last_version,
                "Migration {} has non-sequential version",
                migration.name
            );
            last_version = migration.version;
        }
    }

    #[test]
    fn test_all_migrations_have_names() {
        for migration in MIGRATIONS {
            assert!(
                !migration.name.is_empty(),
                "Migration {} has empty name",
                migration.version
            );
        }
    }
}
