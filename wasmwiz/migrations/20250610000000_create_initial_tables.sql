-- migrations/20250610000000_create_initial_tables.sql

-- UP migration

-- Table for user accounts (minimal for MVP, will be expanded)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Table for subscription tiers
CREATE TABLE IF NOT EXISTS subscription_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    max_executions_per_minute INTEGER NOT NULL,
    max_executions_per_day INTEGER NOT NULL,
    max_memory_mb INTEGER NOT NULL DEFAULT 128, -- Default 128MB
    max_execution_time_seconds INTEGER NOT NULL DEFAULT 5, -- Default 5 seconds
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Insert default tiers (can be managed via admin interface later)
INSERT INTO subscription_tiers (name, max_executions_per_minute, max_executions_per_day, max_memory_mb, max_execution_time_seconds) VALUES
    ('Free', 10, 500, 128, 5) ON CONFLICT (name) DO NOTHING;
INSERT INTO subscription_tiers (name, max_executions_per_minute, max_executions_per_day, max_memory_mb, max_execution_time_seconds) VALUES
    ('Basic', 100, 10000, 256, 10) ON CONFLICT (name) DO NOTHING;
INSERT INTO subscription_tiers (name, max_executions_per_minute, max_executions_per_day, max_memory_mb, max_execution_time_seconds) VALUES
    ('Pro', 500, 50000, 512, 30) ON CONFLICT (name) DO NOTHING;


-- Table for API keys
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_hash TEXT NOT NULL UNIQUE, -- SHA-256 hash of the actual API key
    user_id UUID NOT NULL REFERENCES users(id),
    tier_id UUID NOT NULL REFERENCES subscription_tiers(id),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index for faster lookups by key_hash during authentication
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys (key_hash);

-- Table for usage logs
CREATE TABLE IF NOT EXISTS usage_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    execution_duration_ms INTEGER, -- Actual time taken for Wasm execution.
    memory_peak_mb NUMERIC(5,2), -- Peak memory usage during Wasm execution.
    status TEXT NOT NULL, -- e.g., 'success', 'execution_error', 'time_limit_exceeded', 'memory_limit_exceeded'
    error_message TEXT, -- If execution failed.
    wasm_module_size_bytes INTEGER, -- Size of the uploaded Wasm module.
    input_size_bytes INTEGER -- Size of the provided input string.
);

-- Index for faster filtering by api_key_id and timestamp for analytics
CREATE INDEX IF NOT EXISTS idx_usage_logs_api_key_id_timestamp ON usage_logs (api_key_id, timestamp DESC);