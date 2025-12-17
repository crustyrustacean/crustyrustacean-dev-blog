-- Migration 003: Create comments table
-- User comments on articles

CREATE TABLE IF NOT EXISTS comments (
    id TEXT PRIMARY KEY,
    body TEXT NOT NULL,
    author_id TEXT NOT NULL,
    article_id TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (author_id) REFERENCES users (id) ON DELETE CASCADE,
    FOREIGN KEY (article_id) REFERENCES articles (id) ON DELETE CASCADE
)
