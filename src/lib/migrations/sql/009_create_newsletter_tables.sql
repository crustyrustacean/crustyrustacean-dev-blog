-- Migration 009: Create newsletter tables
-- Newsletter subscription, issues, and delivery tracking

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
);

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
);

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
