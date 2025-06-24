#!/bin/bash
# scripts/start-dev.sh
# Comprehensive development environment startup script

set -e

echo "🚀 Starting WasmWiz Development Environment"
echo "=============================================="

# Function to check if a port is available
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo "❌ Port $port is already in use"
        return 1
    else
        echo "✅ Port $port is available"
        return 0
    fi
}

# Function to wait for service to be ready
wait_for_service() {
    local service_name=$1
    local port=$2
    local max_attempts=30
    local attempt=1

    echo "⏳ Waiting for $service_name to be ready on port $port..."
    
    while [ $attempt -le $max_attempts ]; do
        if nc -z localhost $port 2>/dev/null; then
            echo "✅ $service_name is ready!"
            return 0
        fi
        echo "   Attempt $attempt/$max_attempts - waiting..."
        sleep 2
        ((attempt++))
    done
    
    echo "❌ $service_name failed to start within expected time"
    return 1
}

# Check required ports
echo "🔍 Checking port availability..."
check_port 7432 || exit 1  # PostgreSQL
check_port 7379 || exit 1  # Redis
check_port 7050 || exit 1  # pgAdmin (optional)
check_port 8081 || exit 1  # WasmWiz server

# Start database services
echo "🗄️  Starting database services..."
docker-compose -f docker-compose.dev-ports.yml up -d postgres_dev redis_dev

# Wait for services to be ready
wait_for_service "PostgreSQL" 7432
wait_for_service "Redis" 7379

# Run database migrations
echo "📊 Running database migrations..."
export DATABASE_URL="postgresql://wasmwiz:wasmwiz@localhost:7432/wasmwiz_dev"
cd ../wasmwiz
sqlx migrate run --source ./migrations || {
    echo "❌ Migration failed. Check your database connection."
    exit 1
}

echo "✅ Database migrations completed successfully"

# Build the application
echo "🔨 Building WasmWiz application..."
cargo build || {
    echo "❌ Build failed. Check your Rust code."
    exit 1
}

echo "✅ Build completed successfully"

# Set up environment variables
export WASMWIZ_ENV=development
export DATABASE_URL="postgresql://wasmwiz:wasmwiz@localhost:7432/wasmwiz_dev"
export REDIS_URL="redis://127.0.0.1:7379"
export SERVER_PORT=8081
export AUTH_REQUIRED=false
export REDIS_ENABLED=false
export LOG_LEVEL=debug

echo "🌟 Development environment is ready!"
echo ""
echo "📋 Configuration:"
echo "   Database URL: $DATABASE_URL"
echo "   Redis URL: $REDIS_URL"
echo "   Server Port: $SERVER_PORT"
echo "   Auth Required: $AUTH_REQUIRED"
echo "   Log Level: $LOG_LEVEL"
echo ""
echo "🔗 Useful URLs:"
echo "   Application: http://localhost:8081"
echo "   Health Check: http://localhost:8081/health"
echo "   pgAdmin: http://localhost:7050 (admin@wasmwiz.dev / admin)"
echo ""
echo "🧪 Test API Key (if auth enabled): dev-test-key-123"
echo ""
echo "🚀 Starting WasmWiz server..."
echo "   (Press Ctrl+C to stop)"
echo ""

# Start the application
./target/debug/wasmwiz
