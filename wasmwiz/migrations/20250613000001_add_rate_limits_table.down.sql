-- migrations/20250613000001_add_rate_limits_table.down.sql

-- DOWN migration

-- Drop indexes first
DROP INDEX IF EXISTS idx_rate_limits_api_key_window;
DROP INDEX IF EXISTS idx_rate_limits_type;
DROP INDEX IF EXISTS idx_rate_limits_window;
DROP INDEX IF EXISTS idx_rate_limits_api_key_id;

-- Drop table
DROP TABLE IF EXISTS rate_limits;