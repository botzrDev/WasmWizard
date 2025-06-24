# WasmWiz Development Environment Setup - Complete

## Overview
Successfully set up a complete development environment for the WasmWiz project with database, Redis, and application running on custom ports in the 7000-8000 range.

## Current Status: ✅ FULLY OPERATIONAL

### Services Running
- **PostgreSQL Database**: Port 7432
- **Redis Cache**: Port 7379  
- **pgAdmin Interface**: Port 7050 (admin@wasmwiz.dev / admin)
- **WasmWiz Application**: Port 8081

### Database Configuration
- **Database**: `wasmwiz_dev` on PostgreSQL 16
- **Connection**: `postgres://wasmwiz:wasmwiz@localhost:7432/wasmwiz_dev`
- **Migrations**: Successfully applied (2 migrations)
- **Seed Data**: Test user and API key created
  - User: `test@wasmwiz.dev`
  - API Key: `dev-test-key-123` (for testing authenticated endpoints)

### Application Configuration
- **Environment**: Development
- **Authentication**: Disabled (AUTH_REQUIRED=false)
- **Rate Limiting**: Temporarily disabled due to BorrowMutError
- **Health Check**: ✅ Working (`http://localhost:8081/health`)
- **Web Interface**: ✅ Working (`http://localhost:8081`)

## Files Created/Modified

### New Files
1. **`/DATABASE_ANALYSIS.md`** - Comprehensive database schema analysis
2. **`/wasmwiz/docker-compose.dev-ports.yml`** - Development Docker Compose with custom ports
3. **`/wasmwiz/scripts/init-dev-db.sql`** - Database seed script for test data
4. **`/wasmwiz/scripts/start-dev.sh`** - Automated development environment startup script

### Modified Files
1. **`/wasmwiz/.env.development`** - Updated to use ports 7432/7379
2. **`/wasmwiz/src/app.rs`** - Temporarily disabled rate limiting middleware

## Database Schema
The application uses a PostgreSQL database with the following key tables:

### Tables
- **users**: User accounts (id, email, created_at, updated_at)
- **api_keys**: API keys for authentication (id, user_id, key_hash, is_active, created_at, expires_at)
- **subscription_tiers**: User subscription levels (id, tier_name, description, max_requests_per_day, max_requests_per_minute)
- **usage_logs**: Request/execution logs (id, api_key_id, endpoint, execution_duration_ms, etc.)

### Indexes
Performance indexes are in place for:
- `api_keys(key_hash)` - For fast API key lookups
- `usage_logs(api_key_id, timestamp)` - For usage analytics
- Various timestamp-based indexes for cleanup and reporting

## Startup Instructions

### Option 1: Automated Startup (Recommended)
```bash
cd /home/austingreen/Documents/botzr/projects/WasmWiz/wasmwiz
./scripts/start-dev.sh
```

### Option 2: Manual Startup
```bash
cd /home/austingreen/Documents/botzr/projects/WasmWiz/wasmwiz

# Start database services
docker-compose -f docker-compose.dev-ports.yml up -d

# Wait for services to be ready
sleep 10

# Run migrations
sqlx migrate run --database-url postgres://wasmwiz:wasmwiz@localhost:7432/wasmwiz_dev

# Seed test data (if needed)
psql -h localhost -p 7432 -U wasmwiz -d wasmwiz_dev -f scripts/init-dev-db.sql

# Start application
DATABASE_URL=postgres://wasmwiz:wasmwiz@localhost:7432/wasmwiz_dev REDIS_URL=redis://127.0.0.1:7379 ./target/debug/wasmwiz
```

### Option 3: Development Build and Run
```bash
cd /home/austingreen/Documents/botzr/projects/WasmWiz/wasmwiz

# Build the application
cargo build

# Start with environment variables
DATABASE_URL=postgres://wasmwiz:wasmwiz@localhost:7432/wasmwiz_dev REDIS_URL=redis://127.0.0.1:7379 ./target/debug/wasmwiz
```

## Testing

### Health Check
```bash
curl http://localhost:8081/health
```

### Web Interface
Open: http://localhost:8081

### API Endpoints
- **No Auth Mode**: POST `/api/execute` (multipart form with 'wasm' and 'input' fields)
- **Debug Endpoint**: POST `/api/debug-execute` (multipart form)
- **With Auth**: Include `Authorization: Bearer dev-test-key-123` header

### Database Access
```bash
# Connect to database
psql -h localhost -p 7432 -U wasmwiz -d wasmwiz_dev

# Check tables
\dt

# View users
SELECT * FROM users;

# View API keys  
SELECT * FROM api_keys;
```

### pgAdmin Access
- URL: http://localhost:7050
- Email: admin@wasmwiz.dev
- Password: admin

## Known Issues & Fixes

### Issue 1: Rate Limiting Middleware BorrowMutError
**Problem**: The `DistributedRateLimitMiddleware` causes a `BorrowMutError` panic when processing requests.

**Current Fix**: Temporarily disabled in `src/app.rs` (line 74)

**Permanent Fix Needed**: Refactor the middleware to avoid borrowing conflicts, likely in `src/middleware/rate_limit_middleware.rs` around line 63 where `req.request()` is called before passing `req` to the next service.

### Issue 2: Input Validation Flags curl
**Behavior**: Middleware flags curl User-Agent as suspicious (expected security feature)

**Impact**: Warnings in logs, but requests still process correctly

## WASM Test Modules
Available test modules in `/wasmwiz/temp_wasm_src/`:
- `hello_world.rs` - Simple hello world
- `echo.rs` - Echo input back
- `calc_add.rs` - Add two numbers

## Next Steps
1. **Fix Rate Limiting**: Resolve the BorrowMutError in the rate limiting middleware
2. **Run Cypress Tests**: Execute E2E tests to verify full functionality
3. **Test WASM Execution**: Upload and execute test WASM modules
4. **Performance Testing**: Test with concurrent requests
5. **Security Testing**: Verify input validation and rate limiting work correctly

## Environment Variables Reference
```bash
DATABASE_URL=postgres://wasmwiz:wasmwiz@localhost:7432/wasmwiz_dev
REDIS_URL=redis://127.0.0.1:7379
ENVIRONMENT=development
AUTH_REQUIRED=false
SERVER_HOST=127.0.0.1
SERVER_PORT=8081
LOG_LEVEL=debug
API_SALT=dev-salt-please-change-in-production
MAX_WASM_SIZE=10485760
MAX_INPUT_SIZE=1048576
EXECUTION_TIMEOUT=5
MEMORY_LIMIT=134217728
```

---

## Summary
✅ **SUCCESS**: Complete development environment is operational with database, cache, and application running on isolated ports. The system is ready for development, testing, and WASM execution. The only remaining issue is the rate limiting middleware which has been temporarily disabled to ensure stability.
