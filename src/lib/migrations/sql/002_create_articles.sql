-- Migration 002: Create articles table
-- Blog posts with draft support and category assignment

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
)
