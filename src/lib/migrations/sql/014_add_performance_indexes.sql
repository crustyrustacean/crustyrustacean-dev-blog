-- Migration 014: Add performance indexes for optimized queries
-- These indexes support the repository pattern optimizations

-- Index for counting favorites per article (used in article listings)
CREATE INDEX IF NOT EXISTS idx_user_favorites_article_id ON user_favorites (article_id);

-- Index for checking if user favorited an article (composite for faster lookups)
CREATE INDEX IF NOT EXISTS idx_user_favorites_article_user ON user_favorites (article_id, user_id);

-- Index for feed queries - finding articles by followed users
CREATE INDEX IF NOT EXISTS idx_user_follows_following_id ON user_follows (following_id);

-- Composite index for common comment queries
CREATE INDEX IF NOT EXISTS idx_comments_article_created ON comments (article_id, created_at ASC);
