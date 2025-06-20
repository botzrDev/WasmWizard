# Multi-stage Docker build for production
FROM rust:1.75-slim as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/
COPY templates/ templates/
COPY static/ static/

# Build for production with optimizations
ENV CARGO_TERM_COLOR=always
RUN cargo build --release --locked

# Runtime image - minimal
FROM debian:bookworm-slim

# Install runtime dependencies including curl for health checks
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN groupadd -r wasmwiz && useradd -r -g wasmwiz wasmwiz

# Create directories for the application
RUN mkdir -p /app/temp_wasm_src /app/temp_wasm_test /app/static /app/templates \
    && chown -R wasmwiz:wasmwiz /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/wasmwiz /usr/local/bin/wasmwiz
COPY --from=builder --chown=wasmwiz:wasmwiz /app/static/ /app/static/
COPY --from=builder --chown=wasmwiz:wasmwiz /app/templates/ /app/templates/

# Set proper permissions
RUN chmod +x /usr/local/bin/wasmwiz

# Switch to non-root user
USER wasmwiz

# Set working directory
WORKDIR /app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Production environment variables
ENV ENVIRONMENT=production
ENV RUST_LOG=info
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080

CMD ["wasmwiz"]

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["wasmwiz"]
