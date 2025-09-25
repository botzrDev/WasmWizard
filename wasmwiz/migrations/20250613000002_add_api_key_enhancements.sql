-- migrations/20250613000002_add_api_key_enhancements.sql

-- UP migration

-- Add expiration date and last used tracking to API keys
ALTER TABLE api_keys 
ADD COLUMN IF NOT EXISTS expires_at TIMESTAMP WITH TIME ZONE,
ADD COLUMN IF NOT EXISTS last_used_at TIMESTAMP WITH TIME ZONE;

-- Index for efficient expiration cleanup
CREATE INDEX IF NOT EXISTS idx_api_keys_expires_at ON api_keys (expires_at) WHERE expires_at IS NOT NULL;

-- Index for last used tracking
CREATE INDEX IF NOT EXISTS idx_api_keys_last_used ON api_keys (last_used_at);