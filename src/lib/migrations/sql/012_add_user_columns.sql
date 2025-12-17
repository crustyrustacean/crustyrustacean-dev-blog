-- Migration 012: Add columns to users table
-- These use ALTER TABLE which may fail if columns exist (handled gracefully)

-- Add email_verified column (0 = not verified, 1 = verified)
ALTER TABLE users ADD COLUMN email_verified INTEGER NOT NULL DEFAULT 0;

-- Add password_changed_at for JWT invalidation
ALTER TABLE users ADD COLUMN password_changed_at TEXT;

-- Add theme_preference for user customization
ALTER TABLE users ADD COLUMN theme_preference TEXT DEFAULT 'auto';

-- Promote first user to admin if needed (for existing databases)
UPDATE users
SET role = 'admin'
WHERE id = (SELECT id FROM users ORDER BY created_at ASC LIMIT 1)
AND role = 'subscriber'
