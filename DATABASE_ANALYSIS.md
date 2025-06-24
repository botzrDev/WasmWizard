# WasmWiz Database Analysis & Development Setup

## Current Database Architecture

### Database Schema Overview

The WasmWiz project uses **PostgreSQL** with a well-structured schema designed for a WASM execution platform with API key authentication and usage tracking.

#### Core Tables:

1. **users** - User account management
   - `id` (UUID, primary key)
   - `email` (unique identifier)
   - Standard timestamps

2. **subscription_tiers** - Service tier definitions
   - `id` (UUID, primary key) 
   - `name` (unique tier name)
   - `max_executions_per_minute` - Rate limiting
   - `max_executions_per_day` - Daily quotas
   - `max_memory_mb` - Memory limits (default 128MB)
   - `max_execution_time_seconds` - Timeout limits (default 5s)

3. **api_keys** - Authentication tokens
   - `id` (UUID, primary key)
   - `key_hash` (SHA-256 hash of actual API key)
   - `user_id` (foreign key to users)
   - `tier_id` (foreign key to subscription_tiers) 
   - `is_active` (boolean flag)

4. **usage_logs** - Execution analytics
   - `id` (UUID, primary key)
   - `api_key_id` (foreign key to api_keys)
   - `execution_duration_ms` - Performance tracking
   - `memory_peak_mb` - Resource usage
   - `status` - Success/failure tracking
   - `error_message` - Failure details
   - `wasm_module_size_bytes` - Module size tracking
   - `input_size_bytes` - Input size tracking

#### Default Data:
- **Free Tier**: 10 executions/min, 500/day, 128MB, 5s timeout
- **Basic Tier**: 100 executions/min, 10,000/day, 256MB, 10s timeout  
- **Pro Tier**: 500 executions/min, 50,000/day, 512MB, 30s timeout

#### Performance Optimizations:
- Indexed lookups for `api_keys.key_hash` (authentication)
- Indexed analytics queries for `usage_logs` by API key and timestamp
- Composite indexes for user/tier relationships
- Check constraints for data validation

### Current Database Configuration

**Production Setup** (docker-compose.yml):
- PostgreSQL 15-alpine
- Port: 5432
- Database: `wasmwiz`
- User: `wasmwiz`

**Development Setup** (docker-compose.dev.yml):
- PostgreSQL 15
- Port: 5433 (avoiding conflicts)
- Database: `wasmwiz_dev`
- User: `wasmwiz` / Password: `wasmwiz`
- Includes pgAdmin on port 5050

## Development Database Setup Plan

### Step 1: Use Port Range 7000-8000 for Development

We'll modify the development setup to use the requested port range:

- **PostgreSQL**: Port 7432 (instead of 5433)
- **Redis**: Port 7379 (instead of 6379)
- **pgAdmin**: Port 7050 (instead of 5050)
- **Application**: Port 8081 (as configured)

### Step 2: Environment Configuration

For development with no authentication required:
```bash
WASMWIZ_ENV=development
DATABASE_URL="postgresql://wasmwiz:wasmwiz@localhost:7432/wasmwiz_dev"
REDIS_URL="redis://127.0.0.1:7379"
SERVER_PORT=8081
AUTH_REQUIRED=false
REDIS_ENABLED=false
```

### Step 3: Database Migration Strategy

1. **Start development containers** with ports 7000-8000
2. **Run migrations** to create schema and default data
3. **Optionally seed test data** for development
4. **Test no-auth endpoints** for WASM execution

## Migration Files Analysis

### 20250610000000_create_initial_tables.sql
- ✅ Creates all core tables with proper relationships
- ✅ Includes UUID extension for PostgreSQL
- ✅ Seeds default subscription tiers  
- ✅ Creates essential indexes

### 20250623000000_add_performance_indexes.sql
- ✅ Adds performance indexes for analytics queries
- ✅ Adds data validation constraints
- ✅ Optimizes for common query patterns

Both migrations are well-structured and production-ready.

## Recommendations

1. **Immediate Setup**: Use the development docker-compose with modified ports
2. **Data Seeding**: Consider adding a test user and API key for authenticated testing
3. **Test Strategy**: Start with no-auth mode, then test authenticated endpoints
4. **Monitoring**: Use pgAdmin for database inspection during development

This setup mirrors production while remaining isolated for development and testing.
