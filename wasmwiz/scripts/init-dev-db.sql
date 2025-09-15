-- scripts/init-dev-db.sql
-- Development database initialization script
-- This runs automatically when the PostgreSQL container starts

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create a test user and API key for development testing
DO $$
DECLARE
    test_user_id UUID;
    free_tier_id UUID;
    test_api_key_hash TEXT;
BEGIN
    -- Insert test user if not exists
    INSERT INTO users (id, email, created_at, updated_at)
    VALUES (
        uuid_generate_v4(),
        'test@wasm-wizard.dev',
        NOW(),
        NOW()
    )
    ON CONFLICT (email) DO NOTHING
    RETURNING id INTO test_user_id;

    -- Get the user ID if it already exists
    IF test_user_id IS NULL THEN
        SELECT id INTO test_user_id FROM users WHERE email = 'test@wasm-wizard.dev';
    END IF;

    -- Get the Free tier ID
    SELECT id INTO free_tier_id FROM subscription_tiers WHERE name = 'Free';

    -- Create API key hash for 'dev-test-key-123'
    test_api_key_hash := encode(sha256('dev-test-key-123'::bytea), 'hex');

    -- Insert test API key if not exists
    INSERT INTO api_keys (id, key_hash, user_id, tier_id, is_active, created_at, updated_at)
    VALUES (
        uuid_generate_v4(),
        test_api_key_hash,
        test_user_id,
        free_tier_id,
        true,
        NOW(),
        NOW()
    )
    ON CONFLICT (key_hash) DO NOTHING;

    RAISE NOTICE 'Development test data created:';
    RAISE NOTICE '  User: test@wasm-wizard.dev (ID: %)', test_user_id;
    RAISE NOTICE '  API Key: dev-test-key-123';
    RAISE NOTICE '  Tier: Free';
END $$;
