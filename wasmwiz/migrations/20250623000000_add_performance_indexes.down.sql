-- migrations/20250623000000_add_performance_indexes.down.sql

-- Remove check constraints
ALTER TABLE subscription_tiers DROP CONSTRAINT IF EXISTS chk_subscription_tiers_limits;
ALTER TABLE usage_logs DROP CONSTRAINT IF EXISTS chk_usage_logs_execution_duration_ms;
ALTER TABLE usage_logs DROP CONSTRAINT IF EXISTS chk_usage_logs_memory_peak_mb;
ALTER TABLE usage_logs DROP CONSTRAINT IF EXISTS chk_usage_logs_status;

-- Remove indexes
DROP INDEX IF EXISTS idx_usage_logs_key_status;
DROP INDEX IF EXISTS idx_usage_logs_status;
DROP INDEX IF EXISTS idx_api_keys_user_tier;
DROP INDEX IF EXISTS idx_api_keys_is_active;
DROP INDEX IF EXISTS idx_api_keys_user_id;
DROP INDEX IF EXISTS idx_usage_logs_timestamp;
