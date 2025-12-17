-- Migration 008: Create media library tables
-- File storage metadata and usage tracking

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
);

CREATE TABLE IF NOT EXISTS media_usage (
    id TEXT PRIMARY KEY,
    media_id TEXT NOT NULL,
    article_slug TEXT NOT NULL,
    usage_context TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (media_id) REFERENCES media_library (id) ON DELETE CASCADE,
    FOREIGN KEY (article_slug) REFERENCES articles (slug) ON DELETE CASCADE
)
