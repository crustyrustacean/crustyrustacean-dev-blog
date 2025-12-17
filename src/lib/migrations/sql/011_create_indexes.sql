-- Migration 011: Create indexes for performance
-- All indexes use IF NOT EXISTS for idempotency

-- Article indexes
CREATE INDEX IF NOT EXISTS idx_articles_author_id ON articles (author_id);
CREATE INDEX IF NOT EXISTS idx_articles_slug ON articles (slug);
CREATE INDEX IF NOT EXISTS idx_articles_draft ON articles (draft);
CREATE INDEX IF NOT EXISTS idx_articles_created_at ON articles (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_articles_draft_created_at ON articles (draft, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_articles_category_id ON articles (category_id);

-- Comment indexes
CREATE INDEX IF NOT EXISTS idx_comments_article_id ON comments (article_id);
CREATE INDEX IF NOT EXISTS idx_comments_author_id ON comments (author_id);

-- Tag indexes
CREATE INDEX IF NOT EXISTS idx_article_tags_article_id ON article_tags (article_id);
CREATE INDEX IF NOT EXISTS idx_article_tags_tag_id ON article_tags (tag_id);

-- Category indexes
CREATE INDEX IF NOT EXISTS idx_categories_slug ON categories (slug);

-- Social indexes
CREATE INDEX IF NOT EXISTS idx_user_favorites_user_id ON user_favorites (user_id);
CREATE INDEX IF NOT EXISTS idx_user_follows_follower_id ON user_follows (follower_id);

-- API key indexes
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys (user_id);

-- Media indexes
CREATE INDEX IF NOT EXISTS idx_media_user_id ON media_library (user_id);
CREATE INDEX IF NOT EXISTS idx_media_uploaded_at ON media_library (uploaded_at DESC);
CREATE INDEX IF NOT EXISTS idx_media_mime_type ON media_library (mime_type);
CREATE INDEX IF NOT EXISTS idx_media_usage_media_id ON media_usage (media_id);
CREATE INDEX IF NOT EXISTS idx_media_usage_article_slug ON media_usage (article_slug);

-- Newsletter indexes
CREATE INDEX IF NOT EXISTS idx_newsletter_subscribers_email ON newsletter_subscribers (email);
CREATE INDEX IF NOT EXISTS idx_newsletter_subscribers_confirmed ON newsletter_subscribers (confirmed);
CREATE INDEX IF NOT EXISTS idx_newsletter_subscribers_confirmation_token ON newsletter_subscribers (confirmation_token);
CREATE INDEX IF NOT EXISTS idx_newsletter_subscribers_unsubscribe_token ON newsletter_subscribers (unsubscribe_token);
CREATE INDEX IF NOT EXISTS idx_newsletter_issues_status ON newsletter_issues (status);
CREATE INDEX IF NOT EXISTS idx_newsletter_issues_author_id ON newsletter_issues (author_id);
CREATE INDEX IF NOT EXISTS idx_newsletter_delivery_logs_issue_id ON newsletter_delivery_logs (issue_id);
CREATE INDEX IF NOT EXISTS idx_newsletter_delivery_logs_subscriber_id ON newsletter_delivery_logs (subscriber_id);

-- Token indexes
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user_id ON password_reset_tokens (user_id);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_token ON password_reset_tokens (token);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_expires_at ON password_reset_tokens (expires_at);
CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_user_id ON email_verification_tokens (user_id);
CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_token ON email_verification_tokens (token);
CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_expires_at ON email_verification_tokens (expires_at)
