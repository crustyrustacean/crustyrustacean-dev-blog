-- Migration 001: Create users table
-- Core user accounts with authentication and profile information

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
