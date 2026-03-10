-- Migration: Create media library tables
-- File storage metadata and usage tracking

CREATE TABLE media_library (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    storage_path VARCHAR(500) NOT NULL,
    title VARCHAR(255),
    alt_text TEXT,
    caption TEXT,
    description TEXT,
    mime_type VARCHAR(100) NOT NULL,
    file_size BIGINT NOT NULL,
    width INTEGER,
    height INTEGER,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE media_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    media_id UUID NOT NULL REFERENCES media_library(id) ON DELETE CASCADE,
    article_slug VARCHAR(255) NOT NULL REFERENCES articles(slug) ON DELETE CASCADE,
    usage_context VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_media_library_user_id ON media_library(user_id);
CREATE INDEX idx_media_usage_media_id ON media_usage(media_id);
CREATE INDEX idx_media_usage_article_slug ON media_usage(article_slug);
