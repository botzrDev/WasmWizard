# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WasmWiz is a production-ready WebAssembly management and execution platform built with Rust. It provides secure WASM module execution with enterprise-grade security, monitoring, and scalability features.

## Core Architecture

- **Backend**: Rust with Actix-web framework (src/)
- **WebAssembly Runtime**: Wasmer with WASI sandboxing (5s max runtime, 128MB memory limits)
- **Database**: PostgreSQL 15+ with SQLx for migrations
- **Caching/Rate Limiting**: Redis 7 with distributed enforcement
- **Authentication**: JWT-based API keys with SHA-256 hashing and role-based tiers
- **Templates**: Server-side rendering using Askama
- **Deployment**: Docker containers with Kubernetes support

## Development Commands

### Building and Running
```bash
# Start development environment with database
cd wasmwiz
docker-compose -f docker-compose.dev.yml up -d

# Set up environment
cp .env.development .env

# Run database migrations
sqlx migrate run

# Run development server
cargo run

# Build release version
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with all features and verbose output
cargo test --all-features --verbose

# Run specific test
cargo test test_name

# Run tests with output capture disabled
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_tests
```

### Code Quality
```bash
# Format code
cargo fmt

# Check formatting without changes
cargo fmt --check

# Run clippy linter
cargo clippy -- -D warnings

# Fix clippy issues automatically
cargo clippy --fix --all-targets --all-features

# Security audit
cargo audit

# Check dependencies
cargo tree -d
```

### Load Testing
```bash
# Run comprehensive load tests
./scripts/load_test.sh

# Custom load test with parameters
./scripts/load_test.sh -u https://your-domain.com -c 100 -n 5000 -d 300
```

### Database Operations
```bash
# Reset development database
docker-compose -f docker-compose.dev.yml down -v
docker-compose -f docker-compose.dev.yml up -d

# Access pgAdmin (optional)
docker-compose -f docker-compose.dev.yml --profile tools up -d pgadmin
# Visit http://localhost:5050 (admin@wasmwiz.dev / admin)
```

### Production Deployment
```bash
# Generate production secrets
./scripts/generate_secrets.sh

# Start production stack
docker-compose -f docker-compose.production.yml up -d

# Backup database
./scripts/backup.sh

# Restore from backup
./scripts/restore.sh latest

# Kubernetes deployment
kubectl apply -f k8s/
```

## Project Structure

- `wasmwiz/src/` - Main Rust application code
  - `app.rs` - Application setup and configuration
  - `config.rs` - Configuration management
  - `handlers/` - HTTP request handlers
  - `middleware/` - Authentication, rate limiting, security
  - `models/` - Database models and domain types
  - `services/` - WASM execution, database operations
  - `monitoring/` - Metrics, health checks, observability
- `wasmwiz/migrations/` - SQLx database migrations
- `wasmwiz/templates/` - Askama HTML templates
- `wasmwiz/scripts/` - Deployment and operational scripts
- `wasmwiz/k8s/` - Kubernetes manifests
- `wasmwiz/tests/` - Integration tests

## Key Implementation Details

### WebAssembly Execution
- Uses Wasmer runtime with WASI support for sandboxed execution
- Enforces strict resource limits: 5 second timeout, 128MB memory
- Temporary file storage with TTL cleanup for uploaded modules
- Input/output validation and sanitization

### Authentication & Security
- JWT-based API keys stored as SHA-256 hashes
- Three-tier rate limiting (Free: 10 req/min, Basic: 100 req/min, Pro: 1000 req/min)
- Token bucket algorithm with Redis backing
- Security headers (HSTS, CSP, X-Frame-Options)
- Comprehensive audit logging

### Database Schema
- PostgreSQL with UUID primary keys
- Tables: api_keys, wasm_modules, usage_logs, rate_limits
- Automatic migrations on startup in development
- Connection pooling with SQLx

### Monitoring & Observability
- Prometheus metrics endpoint at `/metrics`
- Health checks at `/health`, `/health/live`, `/health/ready`
- Structured JSON logging with correlation IDs
- Grafana dashboards for visualization
- Custom business metrics tracking

## Development Workflow

### Environment Configuration
- Development: Auth disabled, local PostgreSQL, debug logging
- Staging: Auth enabled, external database required
- Production: Auth enabled, optimized settings, full monitoring

### Git Best Practices (from Copilot instructions)
- Use atomic commits: `git add . && git commit -m "message"`
- Check status before committing: `git status`
- Prefer descriptive commit messages
- Commit frequently with logical chunks of work

### File Editing Guidelines
- Make targeted, precise edits rather than large rewrites
- Include 3-5 lines of context for unambiguous string replacement
- Verify changes make sense in broader file context
- Follow existing code conventions and patterns

## API Endpoints

### Core WebAssembly Operations
- `POST /api/wasm/upload` - Upload WASM module
- `POST /api/wasm/execute` - Execute WASM with input
- `GET /api/wasm/modules` - List modules (paginated)
- `DELETE /api/wasm/modules/{id}` - Delete module

### Authentication
- `POST /api/auth/keys` - Generate API key
- `GET /api/auth/keys` - List user's keys
- `DELETE /api/auth/keys/{id}` - Revoke key
- `POST /api/auth/refresh` - Refresh JWT token

### Web Interface
- `GET /` - Dashboard
- `GET /api-keys` - Key management
- `GET /upload` - Module upload interface

## Performance Targets
- >1000 requests/second on health endpoints
- <50ms average response time for API calls
- <200ms 99th percentile response times
- Horizontal scaling to 100+ concurrent users

## Security & Dependency Management

### Known Vulnerabilities (Monitored)
WasmWiz currently has documented low-risk vulnerabilities from transitive dependencies:
- RUSTSEC-2023-0071 (rsa): Timing attack - mitigated by network security
- RUSTSEC-2024-0421 (idna): Punycode vulnerability - no direct usage
- RUSTSEC-2024-0437 (protobuf): Crash vulnerability - metrics only, isolated
- RUSTSEC-2025-0067/0068 (yaml): Parsing vulnerabilities - no direct usage

These pose minimal risk due to:
- WASM execution sandboxing with strict resource limits
- No direct usage of vulnerable functionality
- Comprehensive monitoring and network-level protections
- Regular dependency update monitoring

### Dependency Update Strategy
```bash
# Monthly security updates
cargo audit
cargo update

# Monitor Wasmer releases for dependency fixes
# Update when compatibility confirmed with test suite
cargo test --all-features
```

### Security Best Practices
- All vulnerabilities are tracked and risk-assessed
- CI/CD pipeline includes security scanning (cargo-audit, Trivy)
- Production deployment includes comprehensive monitoring
- Incident response procedures documented

## Important Conventions
- Prefer idiomatic Rust with proper error handling
- Use Result types and ? operator for error propagation
- Follow modular code organization patterns
- Ensure all user inputs are validated and sanitized
- Never log sensitive information (API keys, tokens)
- Write comprehensive tests for new features
- Document and risk-assess all security vulnerabilities