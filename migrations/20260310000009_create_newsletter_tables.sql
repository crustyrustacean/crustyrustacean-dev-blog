-- Migration: Create newsletter tables
-- Newsletter subscription, issues, and delivery tracking

CREATE TABLE newsletter_subscribers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(100),
    confirmation_token VARCHAR(255) UNIQUE,
    unsubscribe_token VARCHAR(255) UNIQUE NOT NULL,
    confirmed BOOLEAN NOT NULL DEFAULT FALSE,
    subscribed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ,
    unsubscribed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE newsletter_issues (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    scheduled_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    recipient_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE newsletter_delivery_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    issue_id UUID NOT NULL REFERENCES newsletter_issues(id) ON DELETE CASCADE,
    subscriber_id UUID NOT NULL REFERENCES newsletter_subscribers(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL,
    error_message TEXT,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_newsletter_subscribers_email ON newsletter_subscribers(email);
CREATE INDEX idx_newsletter_subscribers_unsubscribe_token ON newsletter_subscribers(unsubscribe_token);
CREATE INDEX idx_newsletter_issues_status ON newsletter_issues(status);
CREATE INDEX idx_newsletter_issues_author_id ON newsletter_issues(author_id);
CREATE INDEX idx_newsletter_delivery_logs_issue_id ON newsletter_delivery_logs(issue_id);
CREATE INDEX idx_newsletter_delivery_logs_subscriber_id ON newsletter_delivery_logs(subscriber_id);
