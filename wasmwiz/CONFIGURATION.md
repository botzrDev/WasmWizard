# Configuration Reference

## Overview

Wasm Wizard uses environment-based configuration with sensible defaults for different deployment environments. This document provides a complete reference for all configuration options.

## Environment Variables

### Core Configuration

| Variable | Default | Required | Description |
|----------|---------|----------|-------------|
| `ENVIRONMENT` | `development` | No | Runtime environment (`development`, `staging`, `production`) |
| `DATABASE_URL` | `postgres://wasm-wizard:wasm-wizard@localhost:5432/wasm-wizard_dev` | Yes* | PostgreSQL connection string |
| `REDIS_URL` | `redis://127.0.0.1:6379` | No | Redis connection string for rate limiting |
| `REDIS_ENABLED` | `false` | No | Enable Redis-based rate limiting |

*Required in staging/production environments

### Server Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `127.0.0.1` (dev) / `0.0.0.0` (prod) | Server bind address |
| `SERVER_PORT` | `8080` | Server port |
| `LOG_LEVEL` | `debug` (dev) / `info` (prod) | Logging verbosity |

### Security Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `API_SALT` | `dev-salt-please-change-in-production` | Salt for API key hashing (min 16 chars) |
| `AUTH_REQUIRED` | `false` (dev) / `true` (prod) | Enable authentication |
| `CSP_REPORT_URI` | - | URI for CSP violation reports |
| `CSP_ENABLE_NONCE` | `false` | Enable nonce-based CSP |

### WASM Execution Limits

| Variable | Default | Description |
|----------|---------|-------------|
| `MAX_WASM_SIZE` | `10485760` (10MB) | Maximum WASM module size in bytes |
| `MAX_INPUT_SIZE` | `1048576` (1MB) | Maximum input data size in bytes |
| `EXECUTION_TIMEOUT` | `5` | WASM execution timeout in seconds |
| `MEMORY_LIMIT` | `134217728` (128MB) | WASM memory limit in bytes |

## Environment-Specific Defaults

### Development Environment

```bash
# Automatic defaults when ENVIRONMENT=development
ENVIRONMENT=development
DATABASE_URL=postgres://wasm-wizard:wasm-wizard@localhost:5432/wasm-wizard_dev
REDIS_ENABLED=false
SERVER_HOST=127.0.0.1
LOG_LEVEL=debug
AUTH_REQUIRED=false
```

### Staging Environment

```bash
# Automatic defaults when ENVIRONMENT=staging
ENVIRONMENT=staging
REDIS_ENABLED=true
SERVER_HOST=0.0.0.0
LOG_LEVEL=debug
AUTH_REQUIRED=true
```

### Production Environment

```bash
# Automatic defaults when ENVIRONMENT=production
ENVIRONMENT=production
REDIS_ENABLED=true
SERVER_HOST=0.0.0.0
LOG_LEVEL=info
AUTH_REQUIRED=true
```

## Complete Configuration Examples

### Development Setup

```bash
# .env.development
ENVIRONMENT=development
DATABASE_URL=postgresql://wasm-wizard:password@localhost:5432/wasm-wizard_dev
API_SALT=dev-salt-change-in-production-16-chars-min
LOG_LEVEL=debug
```

### Production Setup

```bash
# .env.production
ENVIRONMENT=production
DATABASE_URL=postgresql://wasm-wizard:secure_password@db.example.com:5432/wasm-wizard_prod
REDIS_URL=redis://redis-cluster.example.com:6379
REDIS_ENABLED=true
API_SALT=your-very-secure-production-salt-here-32-chars
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
LOG_LEVEL=info
AUTH_REQUIRED=true
MAX_WASM_SIZE=10485760
MAX_INPUT_SIZE=1048576
EXECUTION_TIMEOUT=5
MEMORY_LIMIT=134217728
CSP_REPORT_URI=https://csp.example.com/report
```

### Docker Compose Override

```yaml
# docker-compose.override.yml
version: '3.8'
services:
  wasm-wizard:
    environment:
      - ENVIRONMENT=production
      - DATABASE_URL=postgresql://wasm-wizard:password@db:5432/wasm-wizard
      - REDIS_URL=redis://redis:6379
      - REDIS_ENABLED=true
      - API_SALT=${API_SALT}
      - MAX_WASM_SIZE=20971520  # 20MB for production
      - EXECUTION_TIMEOUT=10    # 10 seconds timeout
```

## Configuration Validation

The application validates configuration on startup and will fail to start with invalid settings:

### Validation Rules

- `API_SALT`: Must be at least 16 characters
- `SERVER_PORT`: Must be between 1-65535
- `MAX_WASM_SIZE`: Must be 1 byte to 100MB
- `MAX_INPUT_SIZE`: Must be 1 byte to 10MB
- `EXECUTION_TIMEOUT`: Must be 1-300 seconds
- `MEMORY_LIMIT`: Must be 1MB to 1GB
- `DATABASE_URL`: Required in staging/production
- `REDIS_URL`: Required when `REDIS_ENABLED=true` in production

### Validation Error Examples

```bash
# Invalid API salt
API_SALT=short
# Error: API_SALT must be at least 16 characters

# Invalid memory limit
MEMORY_LIMIT=1024
# Error: MEMORY_LIMIT must be between 1MB and 1GB

# Missing required database URL in production
ENVIRONMENT=production
# DATABASE_URL not set
# Error: DATABASE_URL must be set in production
```

## Advanced Configuration

### Custom WASM Execution Limits

```bash
# High-performance setup
MAX_WASM_SIZE=52428800    # 50MB
MEMORY_LIMIT=536870912   # 512MB
EXECUTION_TIMEOUT=30     # 30 seconds

# Resource-constrained setup
MAX_WASM_SIZE=5242880    # 5MB
MEMORY_LIMIT=67108864    # 64MB
EXECUTION_TIMEOUT=2      # 2 seconds
```

### Content Security Policy

```bash
# Enable CSP with reporting
CSP_ENABLE_NONCE=true
CSP_REPORT_URI=https://csp.example.com/report

# Strict CSP for high-security deployments
CSP_ENABLE_NONCE=true
CSP_REPORT_URI=https://security.example.com/csp-reports
```

### Database Connection Pool

```bash
# High-traffic production setup
DATABASE_URL=postgresql://user:pass@host:5432/db?sslmode=require&pool_max_conns=20&pool_min_conns=5

# Development with connection pooling
DATABASE_URL=postgresql://wasm-wizard:pass@localhost:5432/wasm-wizard_dev?pool_max_conns=5
```

## Monitoring Configuration

### Prometheus Metrics

The `/metrics` endpoint is automatically available and doesn't require additional configuration.

### Health Check Configuration

Health checks use the same database and filesystem paths as the main application.

### Log Configuration

```bash
# Structured JSON logging for production
LOG_LEVEL=info
RUST_LOG=wasm-wizard=info,actix_web=warn

# Debug logging for development
LOG_LEVEL=debug
RUST_LOG=wasm-wizard=debug,actix_web=debug,sqlx=debug
```

## Security Considerations

### API Salt Security

```bash
# Generate a secure random salt
openssl rand -hex 32
# Use output as API_SALT value

# Never use default values in production
API_SALT=dev-salt-please-change-in-production  # ❌ BAD
API_SALT=1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p7q8r9s0t1u2v3w4x5y6z  # ✅ GOOD
```

### Database Security

```bash
# Use SSL/TLS in production
DATABASE_URL=postgresql://user:pass@host:5432/db?sslmode=require

# Use connection pooling
DATABASE_URL=postgresql://user:pass@host:5432/db?pool_max_conns=10&pool_min_conns=2

# Never expose credentials
DATABASE_URL=postgresql://admin:password@localhost:5432/wasm-wizard  # ❌ BAD
DATABASE_URL=postgresql://readonly:limited@localhost:5432/wasm-wizard  # ✅ GOOD
```

### Network Security

```bash
# Bind to specific interface in production
SERVER_HOST=10.0.0.1  # Internal network interface

# Use non-standard port for security through obscurity
SERVER_PORT=8443

# Enable all security features in production
ENVIRONMENT=production
AUTH_REQUIRED=true
CSP_ENABLE_NONCE=true
```

## Troubleshooting

### Common Configuration Issues

#### Database Connection Failed
```bash
# Check DATABASE_URL format
DATABASE_URL=postgresql://user:pass@host:5432/db

# Test connection manually
psql "$DATABASE_URL"

# Check network connectivity
telnet db-host 5432
```

#### Redis Connection Failed
```bash
# Check REDIS_URL format
REDIS_URL=redis://username:password@host:port/db

# Test connection manually
redis-cli -u "$REDIS_URL" ping

# Verify Redis is running
systemctl status redis
```

#### Invalid Configuration on Startup
```bash
# Check validation errors in logs
docker-compose logs wasm-wizard

# Validate configuration manually
cargo run -- --validate-config

# Check environment variables
env | grep -E "(DATABASE|REDIS|API_SALT)"
```

### Performance Tuning

#### High Throughput Configuration
```bash
# Increase limits for high-traffic deployments
MAX_WASM_SIZE=20971520      # 20MB
EXECUTION_TIMEOUT=15        # 15 seconds
MEMORY_LIMIT=268435456      # 256MB

# Optimize database connections
DATABASE_URL=postgresql://user:pass@host:5432/db?pool_max_conns=50&pool_min_conns=10

# Enable Redis for rate limiting
REDIS_ENABLED=true
REDIS_URL=redis://redis-cluster:6379
```

#### Resource-Constrained Configuration
```bash
# Reduce limits for low-resource deployments
MAX_WASM_SIZE=2097152       # 2MB
EXECUTION_TIMEOUT=3         # 3 seconds
MEMORY_LIMIT=33554432       # 32MB

# Minimal database connections
DATABASE_URL=postgresql://user:pass@host:5432/db?pool_max_conns=5&pool_min_conns=1

# Disable Redis if not needed
REDIS_ENABLED=false
```

## Migration Guide

### Upgrading from Development to Production

1. **Set Environment**
   ```bash
   export ENVIRONMENT=production
   ```

2. **Configure Database**
   ```bash
   export DATABASE_URL="postgresql://user:secure_pass@prod-db:5432/wasm-wizard"
   ```

3. **Enable Security**
   ```bash
   export AUTH_REQUIRED=true
   export API_SALT="$(openssl rand -hex 32)"
   ```

4. **Configure Redis** (if using)
   ```bash
   export REDIS_ENABLED=true
   export REDIS_URL="redis://prod-redis:6379"
   ```

5. **Set Production Limits**
   ```bash
   export MAX_WASM_SIZE=10485760  # 10MB
   export EXECUTION_TIMEOUT=5     # 5 seconds
   ```

### Environment File Templates

#### .env.development
```bash
ENVIRONMENT=development
DATABASE_URL=postgresql://wasm-wizard:wasm-wizard@localhost:5432/wasm-wizard_dev
API_SALT=dev-salt-change-in-production-16-chars-min
LOG_LEVEL=debug
AUTH_REQUIRED=false
```

#### .env.staging
```bash
ENVIRONMENT=staging
DATABASE_URL=postgresql://wasm-wizard:staging_pass@staging-db:5432/wasm-wizard_staging
REDIS_URL=redis://staging-redis:6379
REDIS_ENABLED=true
API_SALT=staging-salt-32-chars-secure-random-string
LOG_LEVEL=info
AUTH_REQUIRED=true
```

#### .env.production
```bash
ENVIRONMENT=production
DATABASE_URL=postgresql://wasm-wizard:prod_pass@prod-db:5432/wasm-wizard_prod?sslmode=require
REDIS_URL=redis://prod-redis-cluster:6379
REDIS_ENABLED=true
API_SALT=prod-salt-64-chars-highly-secure-random-string
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
LOG_LEVEL=info
AUTH_REQUIRED=true
MAX_WASM_SIZE=10485760
MAX_INPUT_SIZE=1048576
EXECUTION_TIMEOUT=5
MEMORY_LIMIT=134217728
CSP_ENABLE_NONCE=true
CSP_REPORT_URI=https://security.example.com/csp-reports
```