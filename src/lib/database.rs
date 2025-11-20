// src/lib/database.rs

// dependencies
use libsql::{Connection, Database};
use std::sync::Arc;

// struct type to represent a database connection
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    pub db: Arc<Database>,
}

// methods for the database connection
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
                disabled INTEGER NOT NULL DEFAULT 0,
                role TEXT NOT NULL DEFAULT 'subscriber',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            ",
            (),
        )
        .await?;

        // Add disabled column to existing users table (for backward compatibility)
        let _ = conn
            .execute(
                r"ALTER TABLE users ADD COLUMN disabled INTEGER NOT NULL DEFAULT 0",
                (),
            )
            .await;
        // Ignore error if column already exists

        // Add role column to existing users table (for backward compatibility)
        let _ = conn
            .execute(
                r"ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'subscriber'",
                (),
            )
            .await;
        // Ignore error if column already exists

        // Promote the first user to admin if they don't already have admin role
        // This handles the case where existing users were created before RBAC was implemented
        let _ = conn
            .execute(
                r"UPDATE users
              SET role = 'admin'
              WHERE id = (SELECT id FROM users ORDER BY created_at ASC LIMIT 1)
              AND role = 'subscriber'",
                (),
            )
            .await;

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
                category_id TEXT,
                draft INTEGER NOT NULL DEFAULT 0,
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

        // Create categories table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS categories (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                slug TEXT UNIQUE NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
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

        // Create api_keys table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                name TEXT NOT NULL,
                key_hash TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_used_at TEXT,
                expires_at TEXT,
                FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
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
            "CREATE INDEX IF NOT EXISTS idx_article_tags_tag_id ON article_tags (tag_id)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_draft ON articles (draft)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_created_at ON articles (created_at DESC)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_draft_created_at ON articles (draft, created_at DESC)",
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

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys (user_id)",
            (),
        )
        .await?;

        // Create media_library table
        conn.execute(
            r#"
    CREATE TABLE IF NOT EXISTS media_library (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        filename TEXT NOT NULL,
        storage_path TEXT NOT NULL,
        title TEXT,
        alt_text TEXT,
        caption TEXT,
        description TEXT,
        mime_type TEXT NOT NULL,
        file_size INTEGER NOT NULL,
        width INTEGER,
        height INTEGER,
        uploaded_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
    )
    "#,
            (),
        )
        .await?;

        // Create media_usage table (tracks which articles use which media)
        conn.execute(
            r#"
    CREATE TABLE IF NOT EXISTS media_usage (
        id TEXT PRIMARY KEY,
        media_id TEXT NOT NULL,
        article_slug TEXT NOT NULL,
        usage_context TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        FOREIGN KEY (media_id) REFERENCES media_library (id) ON DELETE CASCADE,
        FOREIGN KEY (article_slug) REFERENCES articles (slug) ON DELETE CASCADE
    )
    "#,
            (),
        )
        .await?;

        // Create indexes for media_library
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_media_user_id ON media_library (user_id)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_media_uploaded_at ON media_library (uploaded_at DESC)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_media_mime_type ON media_library (mime_type)",
            (),
        )
        .await?;

        // Create indexes for media_usage
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_media_usage_media_id ON media_usage (media_id)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_media_usage_article_slug ON media_usage (article_slug)",
            (),
        )
        .await?;

        // Add featured_image_id column to articles (if it doesn't exist)
        // Note: SQLite doesn't support adding columns with foreign keys via ALTER TABLE
        conn.execute("ALTER TABLE articles ADD COLUMN featured_image_id TEXT", ())
            .await
            .ok(); // Use .ok() to ignore error if column already exists

        // Add category_id column to articles (if it doesn't exist)
        conn.execute("ALTER TABLE articles ADD COLUMN category_id TEXT", ())
            .await
            .ok(); // Use .ok() to ignore error if column already exists

        // Add draft column to articles (if it doesn't exist)
        // 0 = published (default), 1 = draft
        conn.execute(
            "ALTER TABLE articles ADD COLUMN draft INTEGER NOT NULL DEFAULT 0",
            (),
        )
        .await
        .ok(); // Use .ok() to ignore error if column already exists

        // Create index for categories
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_categories_slug ON categories (slug)",
            (),
        )
        .await?;

        // Create index for articles category_id
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_articles_category_id ON articles (category_id)",
            (),
        )
        .await?;

        // Create newsletter_subscribers table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS newsletter_subscribers (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                name TEXT,
                confirmation_token TEXT UNIQUE,
                unsubscribe_token TEXT UNIQUE NOT NULL,
                confirmed INTEGER NOT NULL DEFAULT 0,
                subscribed_at TEXT NOT NULL DEFAULT (datetime('now')),
                confirmed_at TEXT,
                unsubscribed_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
            (),
        )
        .await?;

        // Create newsletter_issues table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS newsletter_issues (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                subject TEXT NOT NULL,
                body TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'draft',
                author_id TEXT NOT NULL,
                scheduled_at TEXT,
                sent_at TEXT,
                recipient_count INTEGER DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (author_id) REFERENCES users (id) ON DELETE CASCADE
            )
            "#,
            (),
        )
        .await?;

        // Create newsletter_delivery_logs table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS newsletter_delivery_logs (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                subscriber_id TEXT NOT NULL,
                status TEXT NOT NULL,
                error_message TEXT,
                sent_at TEXT NOT NULL DEFAULT (datetime('now')),
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (issue_id) REFERENCES newsletter_issues (id) ON DELETE CASCADE,
                FOREIGN KEY (subscriber_id) REFERENCES newsletter_subscribers (id) ON DELETE CASCADE
            )
            "#,
            (),
        )
        .await?;

        // Create indexes for newsletter tables
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_newsletter_subscribers_email ON newsletter_subscribers (email)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_newsletter_subscribers_confirmed ON newsletter_subscribers (confirmed)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_newsletter_subscribers_confirmation_token ON newsletter_subscribers (confirmation_token)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_newsletter_subscribers_unsubscribe_token ON newsletter_subscribers (unsubscribe_token)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_newsletter_issues_status ON newsletter_issues (status)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_newsletter_issues_author_id ON newsletter_issues (author_id)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_newsletter_delivery_logs_issue_id ON newsletter_delivery_logs (issue_id)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_newsletter_delivery_logs_subscriber_id ON newsletter_delivery_logs (subscriber_id)",
            (),
        )
        .await?;

        // Create password_reset_tokens table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS password_reset_tokens (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                token TEXT UNIQUE NOT NULL,
                expires_at TEXT NOT NULL,
                used INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
            )
            "#,
            (),
        )
        .await?;

        // Create indexes for password_reset_tokens
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user_id ON password_reset_tokens (user_id)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_token ON password_reset_tokens (token)",
            (),
        )
        .await?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_expires_at ON password_reset_tokens (expires_at)",
            (),
        )
        .await?;

        Ok(())
    }
}
