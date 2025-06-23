-- migrations/20250623000000_add_performance_indexes.sql

-- Add index for timestamp on usage_logs table for faster cleanup operations
CREATE INDEX IF NOT EXISTS idx_usage_logs_timestamp ON usage_logs (timestamp);

-- Add index for user_id on api_keys table for faster listing of keys by user
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys (user_id);

-- Add index for is_active on api_keys table for filtering active keys
CREATE INDEX IF NOT EXISTS idx_api_keys_is_active ON api_keys (is_active);

-- Add composite index for user_id and tier_id on api_keys for billing/analytics
CREATE INDEX IF NOT EXISTS idx_api_keys_user_tier ON api_keys (user_id, tier_id);

-- Add index for the status column on usage_logs for filtering by status
CREATE INDEX IF NOT EXISTS idx_usage_logs_status ON usage_logs (status);

-- Add composite index for api_key_id and status for usage statistics
CREATE INDEX IF NOT EXISTS idx_usage_logs_key_status ON usage_logs (api_key_id, status);

-- Add check constraint for valid status values
ALTER TABLE usage_logs ADD CONSTRAINT chk_usage_logs_status
CHECK (status IN ('success', 'execution_error', 'time_limit_exceeded', 'memory_limit_exceeded', 'validation_error'));

-- Add check constraint for positive memory_peak_mb
ALTER TABLE usage_logs ADD CONSTRAINT chk_usage_logs_memory_peak_mb
CHECK (memory_peak_mb IS NULL OR memory_peak_mb >= 0);

-- Add check constraint for positive execution_duration_ms
ALTER TABLE usage_logs ADD CONSTRAINT chk_usage_logs_execution_duration_ms
CHECK (execution_duration_ms IS NULL OR execution_duration_ms >= 0);

-- Add constraint for subscription_tiers to ensure valid limits
ALTER TABLE subscription_tiers ADD CONSTRAINT chk_subscription_tiers_limits
CHECK (
    max_executions_per_minute > 0 AND
    max_executions_per_day > 0 AND
    max_memory_mb > 0 AND
    max_execution_time_seconds > 0 AND
    max_executions_per_day >= max_executions_per_minute
);
