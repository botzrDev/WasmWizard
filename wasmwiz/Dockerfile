# Multi-stage Dockerfile for WasmWiz production deployment
# Based on the architecture described in README.md and Cargo.toml dependencies

# Stage 1: Build dependencies and cache
FROM rust:1.89-slim-bookworm AS chef
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef
WORKDIR /app

# Stage 2: Plan dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies (cached layer)
FROM chef AS dependencies
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 4: Build application
FROM chef AS builder
COPY . .
COPY --from=dependencies /app/target target
COPY --from=dependencies /usr/local/cargo /usr/local/cargo
RUN cargo build --release

# Stage 5: Runtime image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd --create-home --shell /bin/bash --uid 1000 wasmwiz

# Create application directories
RUN mkdir -p /app /tmp/wasm_modules \
    && chown -R wasmwiz:wasmwiz /app /tmp/wasm_modules

# Copy application binary
COPY --from=builder /app/target/release/wasmwiz /app/wasmwiz

# Copy static files and templates
COPY --from=builder /app/static /app/static
COPY --from=builder /app/templates /app/templates

# Set ownership for application files
RUN chown -R wasmwiz:wasmwiz /app

# Switch to non-root user
USER wasmwiz
WORKDIR /app

# Environment variables with secure defaults
ENV ENVIRONMENT=production \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080 \
    LOG_LEVEL=info \
    RUST_LOG=info,wasmwiz=info \
    WASM_TEMP_DIR=/tmp/wasm_modules \
    MAX_WASM_SIZE=10485760 \
    MAX_INPUT_SIZE=1048576 \
    EXECUTION_TIMEOUT=5 \
    MEMORY_LIMIT=134217728

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:${SERVER_PORT}/health || exit 1

# Expose port
EXPOSE 8080

# Run the application
CMD ["./wasmwiz"]