-- migrations/20250610000000_create_initial_tables.down.sql

-- DOWN migration

DROP TABLE IF EXISTS usage_logs;
DROP TABLE IF EXISTS api_keys;
DROP TABLE IF EXISTS subscription_tiers;
DROP TABLE IF EXISTS users;