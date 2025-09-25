-- migrations/20250613000000_add_wasm_modules_table.sql

-- UP migration

-- Table for storing WASM modules
CREATE TABLE IF NOT EXISTS wasm_modules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    description TEXT,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    wasm_data BYTEA NOT NULL, -- Binary WASM module data
    size_bytes INTEGER NOT NULL,
    sha256_hash TEXT NOT NULL UNIQUE, -- Hash for deduplication and integrity
    upload_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_executed TIMESTAMP WITH TIME ZONE,
    execution_count INTEGER NOT NULL DEFAULT 0,
    is_public BOOLEAN NOT NULL DEFAULT FALSE, -- Allow sharing modules
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_wasm_modules_user_id ON wasm_modules (user_id);
CREATE INDEX IF NOT EXISTS idx_wasm_modules_sha256_hash ON wasm_modules (sha256_hash);
CREATE INDEX IF NOT EXISTS idx_wasm_modules_upload_time ON wasm_modules (upload_time DESC);
CREATE INDEX IF NOT EXISTS idx_wasm_modules_public ON wasm_modules (is_public) WHERE is_public = TRUE;