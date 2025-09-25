-- migrations/20250613000001_add_rate_limits_table.sql

-- UP migration

-- Table for storing rate limiting state
CREATE TABLE IF NOT EXISTS rate_limits (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    window_start TIMESTAMP WITH TIME ZONE NOT NULL,
    window_end TIMESTAMP WITH TIME ZONE NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    limit_type TEXT NOT NULL, -- 'minute', 'hour', 'day'
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for efficient rate limit lookups
CREATE INDEX IF NOT EXISTS idx_rate_limits_api_key_id ON rate_limits (api_key_id);
CREATE INDEX IF NOT EXISTS idx_rate_limits_window ON rate_limits (window_start, window_end);
CREATE INDEX IF NOT EXISTS idx_rate_limits_type ON rate_limits (limit_type);

-- Composite index for the most common query pattern
CREATE INDEX IF NOT EXISTS idx_rate_limits_api_key_window ON rate_limits (api_key_id, window_start, window_end, limit_type);