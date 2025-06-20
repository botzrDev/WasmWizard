# WasmWiz

A production-ready WebAssembly management and execution platform.

## Features

- **WebAssembly Execution**: Secure, sandboxed execution of WASM modules
- **REST API Interface**: Complete HTTP API for module management and execution
- **API Key Authentication**: JWT-based authentication with role-based access
- **Tiered Rate Limiting**: Redis-backed distributed rate limiting
- **Usage Tracking**: Comprehensive metrics and logging
- **Health Monitoring**: Built-in health checks and Prometheus metrics
- **Production Ready**: Security hardening, structured logging, Docker support

## Architecture

WasmWiz is built with production-grade technologies:

- **Rust 2024**: Memory-safe, high-performance system programming
- **Actix Web 4**: Async web framework with excellent performance
- **Wasmer 6.0**: Latest WebAssembly runtime with WASI support
- **PostgreSQL 15**: ACID-compliant database for persistent storage
- **Redis 7**: High-performance caching and rate limiting
- **Docker**: Containerized deployment with multi-stage builds
- **Structured Logging**: JSON logging for production observability

## Quick Start

### Prerequisites

- Rust 1.75+ (for development)
- Docker & Docker Compose (for deployment)
- PostgreSQL 14+ (if running locally)

### Environment Variables

```bash
# Required
DATABASE_URL=postgresql://user:password@localhost/wasmwiz
API_SALT=your-secure-salt-minimum-16-characters

# Optional (with defaults)
ENVIRONMENT=production                    # development, staging, production
SERVER_HOST=0.0.0.0                     # 127.0.0.1 for development
SERVER_PORT=8080
LOG_LEVEL=info                           # debug, info, warn, error
REDIS_URL=redis://127.0.0.1:6379
MAX_WASM_SIZE=10485760                   # 10MB
MAX_INPUT_SIZE=1048576                   # 1MB
EXECUTION_TIMEOUT=5                      # seconds
MEMORY_LIMIT=134217728                   # 128MB
```

### Running Locally

```bash
# Clone and setup
git clone https://github.com/your-org/wasmwiz.git
cd wasmwiz/wasmwiz

# Set environment variables
export DATABASE_URL="postgresql://wasmwiz:password@localhost/wasmwiz"
export API_SALT="development-salt-change-in-production"

# Run database migrations
sqlx migrate run

# Start development server
cargo run
```

### Docker Deployment

```bash
# Production deployment with Docker Compose
docker-compose up -d

# Or build and run manually
docker build -t wasmwiz .
docker run -p 8080:8080 \
  -e DATABASE_URL="postgresql://..." \
  -e API_SALT="secure-production-salt" \
  wasmwiz
```

## API Endpoints

### Health & Monitoring
- `GET /health` - Comprehensive health check with dependencies
- `GET /health/live` - Kubernetes liveness probe
- `GET /health/ready` - Kubernetes readiness probe
- `GET /metrics` - Prometheus metrics

### WebAssembly Operations
- `POST /api/wasm/upload` - Upload WASM module
- `POST /api/wasm/execute` - Execute WASM module
- `GET /api/wasm/modules` - List available modules
- `DELETE /api/wasm/modules/{id}` - Delete module

### Authentication
- `POST /api/auth/keys` - Generate API key
- `GET /api/auth/keys` - List API keys
- `DELETE /api/auth/keys/{id}` - Revoke API key

### Web Interface
- `GET /` - Dashboard
- `GET /api-keys` - Key management interface

## Security Features

- **Input Validation**: All inputs validated and sanitized
- **WASM Sandboxing**: Complete isolation of WASM execution
- **SQL Injection Protection**: Prepared statements throughout
- **Rate Limiting**: Prevents abuse and DoS attacks
- **Secure Headers**: HTTPS recommended, security headers
- **Non-root Container**: Docker runs as non-privileged user
- **Resource Limits**: Memory and execution time constraints

## Production Deployment

### Environment Setup

1. **Database**: Set up PostgreSQL with proper credentials
2. **Redis**: Configure Redis for rate limiting
3. **Secrets**: Use secure, randomly generated API_SALT
4. **TLS**: Configure HTTPS reverse proxy (nginx/traefik)
5. **Monitoring**: Set up log aggregation and metrics collection

### Kubernetes Example

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wasmwiz
spec:
  replicas: 3
  selector:
    matchLabels:
      app: wasmwiz
  template:
    spec:
      containers:
      - name: wasmwiz
        image: wasmwiz:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: wasmwiz-secrets
              key: database-url
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
```

### Performance Tuning

- **Workers**: Automatically scales to available CPU cores in production
- **Connection Pooling**: Optimized database connection management
- **Memory Limits**: Configurable WASM execution memory limits
- **Caching**: Redis-based caching for frequently accessed data
- **Async I/O**: Non-blocking operations throughout

## Monitoring & Observability

### Structured Logging
- **JSON Format**: Machine-readable logs in production
- **Log Levels**: Configurable via LOG_LEVEL environment variable
- **Request Tracing**: Correlation IDs for request tracking
- **Error Context**: Rich error information with stack traces

### Metrics
- **Prometheus**: Built-in metrics endpoint
- **Custom Metrics**: WASM execution time, memory usage, error rates
- **System Metrics**: CPU, memory, disk usage
- **Business Metrics**: API key usage, rate limit hits

### Health Checks
- **Dependency Checks**: Database, Redis connectivity
- **Resource Checks**: Memory usage, disk space
- **Liveness/Readiness**: Kubernetes-compatible probes

## Development

### Testing

```bash
# Unit tests
cargo test

# Integration tests with testcontainers
cargo test --test integration_tests

# Load testing
# Use tools like wrk, hey, or k6
```

### Code Quality

```bash
# Linting
cargo clippy -- -D warnings

# Formatting
cargo fmt --check

# Security audit
cargo audit

# Dependency check
cargo tree -d
```

# The API will be available at http://localhost:8080
```

### Environment Variables

- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_URL`: Redis connection string (for distributed rate limiting)
- `SERVER_HOST`: Host to bind to (default: 127.0.0.1)
- `SERVER_PORT`: Port to listen on (default: 8080)
- `API_SALT`: Salt for API key hashing (min 16 chars, required)
- `MAX_WASM_SIZE`: Maximum WASM module size in bytes (default: 10MB)
- `MAX_INPUT_SIZE`: Maximum input size in bytes (default: 1MB)
- `EXECUTION_TIMEOUT`: Maximum execution time in seconds (default: 5)
- `MEMORY_LIMIT`: Maximum memory usage in bytes (default: 128MB)
