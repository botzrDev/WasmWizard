-- migrations/20250613000002_add_api_key_enhancements.down.sql

-- DOWN migration

-- Drop indexes first
DROP INDEX IF EXISTS idx_api_keys_last_used;
DROP INDEX IF EXISTS idx_api_keys_expires_at;

-- Remove columns
ALTER TABLE api_keys 
DROP COLUMN IF EXISTS last_used_at,
DROP COLUMN IF EXISTS expires_at;