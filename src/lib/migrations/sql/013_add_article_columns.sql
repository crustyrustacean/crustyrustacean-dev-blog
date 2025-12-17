-- Migration 013: Add columns to articles table
-- These use ALTER TABLE which may fail if columns exist (handled gracefully)

-- Add featured_image_id for article thumbnails
ALTER TABLE articles ADD COLUMN featured_image_id TEXT
