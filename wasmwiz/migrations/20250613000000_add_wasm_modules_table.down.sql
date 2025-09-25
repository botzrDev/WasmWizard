-- migrations/20250613000000_add_wasm_modules_table.down.sql

-- DOWN migration

-- Drop indexes first
DROP INDEX IF EXISTS idx_wasm_modules_public;
DROP INDEX IF EXISTS idx_wasm_modules_upload_time;
DROP INDEX IF EXISTS idx_wasm_modules_sha256_hash;
DROP INDEX IF EXISTS idx_wasm_modules_user_id;

-- Drop table
DROP TABLE IF EXISTS wasm_modules;