# Multi-stage build for optimized production image
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations
COPY templates ./templates

# Build the application
RUN cargo build --release

# Production stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 wasmwiz

# Create necessary directories
RUN mkdir -p /tmp/wasm_modules && chown wasmwiz:wasmwiz /tmp/wasm_modules

# Copy the binary from builder stage
COPY --from=builder /app/target/release/wasmwiz /usr/local/bin/wasmwiz

# Set ownership and permissions
RUN chown wasmwiz:wasmwiz /usr/local/bin/wasmwiz
RUN chmod +x /usr/local/bin/wasmwiz

# Switch to non-root user
USER wasmwiz

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["wasmwiz"]
