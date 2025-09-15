#!/bin/bash

set -e

echo "🔨 Building Hello World WASM Example"

# Check if wasm32-wasi target is installed
if ! rustup target list | grep -q "wasm32-wasi"; then
    echo "📦 Installing wasm32-wasi target..."
    rustup target add wasm32-wasi
fi

# Build the WASM module
echo "🏗️  Compiling Rust to WASM..."
cd src
cargo build --target wasm32-wasi --release

# Copy the compiled WASM file to the parent directory
cp target/wasm32-wasi/release/hello_world_wasm.wasm ../hello_world.wasm

echo "✅ Build complete!"
echo "📁 WASM file: hello_world.wasm"
echo "📊 File size: $(ls -lh ../hello_world.wasm | awk '{print $5}')"

# Validate the WASM file
if command -v wasm-validate &> /dev/null; then
    echo "🔍 Validating WASM file..."
    wasm-validate ../hello_world.wasm
    echo "✅ WASM validation passed!"
fi

# Show WASM exports
if command -v wasm-objdump &> /dev/null; then
    echo "📋 WASM exports:"
    wasm-objdump -x ../hello_world.wasm | grep -A 10 "Export"
fi

echo ""
echo "🚀 Ready to test! Run ./test.sh to test the example."