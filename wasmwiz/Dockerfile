# Multi-stage Dockerfile for WasmWiz production deployment
# Stage 1: Build environment
FROM rust:1.81-slim-bookworm AS builder

# Install system dependencies required for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user for building
RUN useradd -m -u 1001 builduser
USER builduser
WORKDIR /app

# Copy dependency files first for better caching
COPY --chown=builduser:builduser Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -f target/release/deps/wasmwiz*

# Copy the actual source code
COPY --chown=builduser:builduser src ./src
COPY --chown=builduser:builduser migrations ./migrations
COPY --chown=builduser:builduser templates ./templates
COPY --chown=builduser:builduser static ./static

# Build the actual application
RUN cargo build --release

# Stage 2: Runtime environment
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create a non-root user for running the application
RUN useradd -m -u 1001 wasmwiz

# Create necessary directories
RUN mkdir -p /app/templates /app/static /tmp/wasm_modules \
    && chown -R wasmwiz:wasmwiz /app /tmp/wasm_modules

# Copy the binary from builder stage
COPY --from=builder /app/target/release/wasmwiz /usr/local/bin/wasmwiz

# Copy static files and templates
COPY --chown=wasmwiz:wasmwiz templates /app/templates
COPY --chown=wasmwiz:wasmwiz static /app/static

# Set up the user and working directory
USER wasmwiz
WORKDIR /app

# Environment variables for production
ENV ENVIRONMENT=production
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV LOG_LEVEL=info
ENV RUST_LOG=info,wasmwiz=info
ENV WASM_TEMP_DIR=/tmp/wasm_modules

# Expose the port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["wasmwiz"]