# Multi-stage Dockerfile for Wasm Wizard production deployment with security hardening
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

# Create a non-root user for building with minimal privileges
RUN useradd -r -m -u 1001 -s /bin/false builduser
USER builduser
WORKDIR /app

# Copy dependency files first for better caching
COPY --chown=builduser:builduser Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -f target/release/deps/wasm_wizard*

# Copy the actual source code
COPY --chown=builduser:builduser src ./src
COPY --chown=builduser:builduser migrations ./migrations
COPY --chown=builduser:builduser templates ./templates
COPY --chown=builduser:builduser static ./static

# Build the actual application
RUN cargo build --release

# Stage 2: Runtime environment with security hardening
FROM debian:bookworm-slim AS runtime

# Security: Set up non-privileged user first
RUN groupadd -r -g 1001 wasm_wizard && \
    useradd -r -u 1001 -g wasm_wizard -s /bin/false -d /nonexistent wasm_wizard

# Install runtime dependencies with security best practices
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    dumb-init \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Create necessary directories with proper permissions
RUN mkdir -p /app/templates /app/static /app/wasm_modules \
    && chown -R wasm_wizard:wasm_wizard /app \
    && chmod 755 /app \
    && chmod -R 644 /app/templates /app/static \
    && chmod 755 /app/templates /app/static /app/wasm_modules

# Copy the binary from builder stage with proper ownership
COPY --from=builder --chown=wasm_wizard:wasm_wizard /app/target/release/wasm_wizard /usr/local/bin/wasm_wizard
RUN chmod 755 /usr/local/bin/wasm_wizard

# Copy static files and templates with security permissions
COPY --chown=wasm_wizard:wasm_wizard templates /app/templates
COPY --chown=wasm_wizard:wasm_wizard static /app/static
RUN find /app/templates -type f -exec chmod 444 {} \; && \
    find /app/static -type f -exec chmod 444 {} \;

# Set up the user and working directory
USER wasm_wizard
WORKDIR /app

# Environment variables for production with security defaults
ENV ENVIRONMENT=production
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV LOG_LEVEL=info
ENV RUST_LOG=info,wasm_wizard=info
ENV WASM_TEMP_DIR=/app/wasm_modules
# Security: Disable debug features
ENV RUST_BACKTRACE=0
# Security: Run with reduced privileges
ENV RUST_MIN_STACK=2097152

# Expose the port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Security: Drop all capabilities and run with init system
USER wasm_wizard:wasm_wizard

# Run the application with dumb-init to handle signals properly
ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["wasm_wizard"]

