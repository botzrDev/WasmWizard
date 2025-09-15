#!/bin/bash

# API Integration Example Build Script
# This script builds the WASM module for the API integration example

set -e

echo "🔨 Building API Integration WASM Module..."

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first."
    exit 1
fi

# Check if wasm32-wasi target is installed
if ! rustup target list | grep -q "wasm32-wasi (installed)"; then
    echo "📦 Installing wasm32-wasi target..."
    rustup target add wasm32-wasi
fi

# Navigate to the example directory
cd "$(dirname "$0")"

# Build the WASM module
echo "🏗️  Compiling Rust to WASM..."
cargo build --target wasm32-wasi --release

# Check if build was successful
if [ ! -f "target/wasm32-wasi/release/api_integration.wasm" ]; then
    echo "❌ Build failed - WASM file not found"
    exit 1
fi

# Get file size
WASM_SIZE=$(stat -c%s "target/wasm32-wasi/release/api_integration.wasm" 2>/dev/null || stat -f%z "target/wasm32-wasi/release/api_integration.wasm" 2>/dev/null || echo "unknown")
echo "✅ Build successful!"
echo "📊 WASM file size: $WASM_SIZE bytes"
echo "📁 Output: target/wasm32-wasi/release/api_integration.wasm"

# Copy to a convenient location for testing
mkdir -p dist
cp target/wasm32-wasi/release/api_integration.wasm dist/

echo "🎉 API Integration example built successfully!"
echo "🚀 Ready for deployment and testing"