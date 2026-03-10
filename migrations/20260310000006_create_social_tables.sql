-- Migration: Create social feature tables
-- User favorites and follows for social interactions

CREATE TABLE user_favorites (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    article_id UUID NOT NULL REFERENCES articles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, article_id)
);

CREATE TABLE user_follows (
    follower_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    following_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (follower_id, following_id),
    CONSTRAINT check_not_self_follow CHECK (follower_id != following_id)
);

CREATE INDEX idx_user_favorites_article_id ON user_favorites(article_id);
CREATE INDEX idx_user_follows_following_id ON user_follows(following_id);
