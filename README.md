# WasmWiz

WasmWiz is a scalable WebAssembly (WASM) execution service that allows users to securely run WebAssembly modules through a REST API.

## Features

- **WebAssembly Execution**: Execute WebAssembly modules securely with input/output handling
- **API Key Authentication**: Secure your API with key-based authentication
- **Tiered Rate Limiting**: Enforce rate limits based on subscription tier
- **Distributed Rate Limiting**: Uses Redis for synchronized rate limiting across multiple instances
- **Usage Tracking**: Track execution metrics per API key
- **Web Interface**: Simple UI for API key management and testing

## Architecture

WasmWiz is built using:

- **Rust**: For performance, safety, and WebAssembly support
- **Actix Web**: High-performance web framework
- **Wasmer**: WebAssembly runtime with WASI support
- **PostgreSQL**: For persistence and usage tracking
- **Redis**: For distributed rate limiting
- **Docker/Kubernetes**: For deployment and scaling

## Rate Limiting

WasmWiz implements a robust, distributed rate limiting system:

- **Redis-Based Rate Limiting**: Ensures rate limits are synchronized across multiple instances
- **Tier-Based Limits**: Different rate limits based on subscription tier (Free, Basic, Pro)
- **Fallback Mechanism**: Falls back to in-memory rate limiting if Redis is unavailable
- **Headers**: Returns standard rate limit headers with remaining requests

## Getting Started

### Prerequisites

- Docker and Docker Compose
- Rust 2024 Edition or later (for development)

### Running with Docker Compose

```bash
# Clone the repository
git clone https://github.com/your-org/wasmwiz.git
cd wasmwiz

# Start the services
docker-compose up -d

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
