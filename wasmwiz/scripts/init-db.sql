-- Production database initialization script
-- This script sets up the database with production-ready settings

-- Create database if it doesn't exist
-- (This is handled by POSTGRES_DB environment variable)

-- Set production-optimized settings
ALTER SYSTEM SET shared_preload_libraries = 'pg_stat_statements';
ALTER SYSTEM SET log_statement = 'mod';
ALTER SYSTEM SET log_min_duration_statement = 1000; -- Log slow queries
ALTER SYSTEM SET log_checkpoints = on;
ALTER SYSTEM SET log_connections = on;
ALTER SYSTEM SET log_disconnections = on;
ALTER SYSTEM SET log_lock_waits = on;

-- Performance optimizations
ALTER SYSTEM SET max_connections = 100;
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;

-- Security settings
ALTER SYSTEM SET ssl = off; -- Will be handled by reverse proxy
ALTER SYSTEM SET log_statement = 'ddl'; -- Log DDL statements
ALTER SYSTEM SET log_min_messages = 'warning';

-- Apply settings (requires restart, but handled by Docker)
SELECT pg_reload_conf();