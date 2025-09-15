#!/bin/bash

set -e

echo "🔨 Building Fibonacci WASM Example"

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
cp target/wasm32-wasi/release/fibonacci_wasm.wasm ../fibonacci.wasm

echo "✅ Build complete!"
echo "📁 WASM file: fibonacci.wasm"
echo "📊 File size: $(ls -lh ../fibonacci.wasm | awk '{print $5}')"

# Validate the WASM file
if command -v wasm-validate &> /dev/null; then
    echo "🔍 Validating WASM file..."
    wasm-validate ../fibonacci.wasm
    echo "✅ WASM validation passed!"
fi

# Show WASM exports
if command -v wasm-objdump &> /dev/null; then
    echo "📋 WASM exports:"
    wasm-objdump -x ../fibonacci.wasm | grep -A 20 "Export"
fi

echo ""
echo "🚀 Ready to test! Run ./test.sh to test the example."
echo ""
echo "💡 Available functions:"
echo "   - fib_recursive(n): Recursive implementation"
echo "   - fib_iterative(n): Iterative implementation"
echo "   - fib_matrix(n): Matrix exponentiation"
echo "   - fib_batch(start, count, result_ptr): Batch calculation"
echo "   - compare_algorithms(n): Performance comparison"
echo "   - get_algorithm_info(algorithm): Algorithm information"