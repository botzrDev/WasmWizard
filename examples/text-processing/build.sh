#!/bin/bash

set -e

echo "🔨 Building Text Processing WASM Example"

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
cp target/wasm32-wasi/release/text_processing_wasm.wasm ../text_processing.wasm

echo "✅ Build complete!"
echo "📁 WASM file: text_processing.wasm"
echo "📊 File size: $(ls -lh ../text_processing.wasm | awk '{print $5}')"

# Validate the WASM file
if command -v wasm-validate &> /dev/null; then
    echo "🔍 Validating WASM file..."
    wasm-validate ../text_processing.wasm
    echo "✅ WASM validation passed!"
fi

echo ""
echo "🚀 Ready to test! Run ./test.sh to test the example."
echo ""
echo "💡 Available functions:"
echo "   - analyze_text(text): Comprehensive text analysis"
echo "   - sentiment_score(text): Basic sentiment analysis (-1.0 to 1.0)"
echo "   - word_frequency(text, top_n): Most frequent words"
echo "   - convert_case(text, mode): Case conversion (0=upper, 1=lower, 2=title)"
echo "   - search_replace(text, search, replace): Find and replace"
echo "   - get_processing_info(): Module information"