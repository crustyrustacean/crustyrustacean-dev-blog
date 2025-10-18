// src/lib/database.rs

use libsql::{Connection, Database};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pub db: Arc<Database>,
}

impl DatabaseConnection {
    pub fn connect(&self) -> Result<Connection, libsql::Error> {
        self.db.connect()
    }

    pub async fn run_migrations(&self) -> Result<(), libsql::Error> {
        let conn = self.connect()?;

        // Create users table
        conn.execute(
            r"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                bio TEXT,
                image TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            ",
            (),
        )
        .await?;

        // Create articles table
        conn.execute(
            r"
            CREATE TABLE IF NOT EXISTS articles (
                id TEXT PRIMARY KEY,
                slug TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                body TEXT NOT NULL,
                author_id TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (author_id) REFERENCES users (id) ON DELETE CASCADE
            );
            ",
            (),
        )
        .await?;

        // Create comments table
        conn.execute(
            r"
            CREATE TABLE IF NOT EXISTS comments (
                id TEXT PRIMARY KEY,
                body TEXT NOT NULL,
                author_id TEXT NOT NULL,
                article_id TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (author_id) REFERENCES users (id) ON DELETE CASCADE,
                FOREIGN KEY (article_id) REFERENCES articles (id) ON DELETE CASCADE
            );
            ",
            (),
        )
        .await?;

        // Create tags table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL
            )
            "#,
            (),
        )
        .await?;

        // Create article_tags junction table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS article_tags (
                article_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (article_id, tag_id),
                FOREIGN KEY (article_id) REFERENCES articles (id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
            )
            "#,
            (),
        )
        .await?;

        // Create user_favorites table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS user_favorites (
                user_id TEXT NOT NULL,
                article_id TEXT NOT NULL,
                PRIMARY KEY (user_id, article_id),
                FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
                FOREIGN KEY (article_id) REFERENCES articles (id) ON DELETE CASCADE
            )
            "#,
            (),
        )
        .await?;

        // Create user_follows table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS user_follows (
                follower_id TEXT NOT NULL,
                following_id TEXT NOT NULL,
                PRIMARY KEY (follower_id, following_id),
                FOREIGN KEY (follower_id) REFERENCES users (id) ON DELETE CASCADE,
                FOREIGN KEY (following_id) REFERENCES users (id) ON DELETE CASCADE
            )
            "#,
            (),
        )
        .await?;

        // Create indexes for better performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_author_id ON articles (author_id)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_slug ON articles (slug)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_comments_article_id ON comments (article_id)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_comments_author_id ON comments (author_id)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_article_tags_article_id ON article_tags (article_id)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_user_favorites_user_id ON user_favorites (user_id)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_user_follows_follower_id ON user_follows (follower_id)",
            (),
        )
        .await?;

        Ok(())
    }
}
