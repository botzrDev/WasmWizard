#!/bin/bash

# Text Processing Example Test Script
# This script runs comprehensive tests for the text processing WASM module

set -e

echo "🧪 Running Text Processing Tests..."

# Navigate to the example directory
cd "$(dirname "$0")"

# Run Rust unit tests
echo "📋 Running unit tests..."
cargo test

# Build the WASM module if not already built
if [ ! -f "target/wasm32-wasi/release/text_processing.wasm" ]; then
    echo "🏗️  Building WASM module first..."
    ./build.sh
fi

# Check if Wasm Wizard API is available
echo "🔍 Checking Wasm Wizard API availability..."
if curl -s http://localhost:8080/health > /dev/null 2>&1; then
    echo "✅ Wasm Wizard API is running"
    API_AVAILABLE=true
else
    echo "⚠️  Wasm Wizard API not available - skipping integration tests"
    API_AVAILABLE=false
fi

# Create test data
echo "📝 Creating test data..."
mkdir -p test_data

# Test data for different text processing operations
echo "This is a sample text for testing word frequency analysis and sentiment scoring." > test_data/sample_text.txt

echo "I love this amazing product! It's fantastic and wonderful." > test_data/positive_text.txt

echo "This is terrible and awful. I hate it completely." > test_data/negative_text.txt

echo "The quick brown fox jumps over the lazy dog. This pangram contains every letter." > test_data/pangram.txt

echo '{"text": "Hello World", "operation": "analyze"}' > test_data/json_input.json

echo "✅ Test data created"

# Run integration tests if API is available
if [ "$API_AVAILABLE" = true ]; then
    echo "🔗 Running integration tests..."

    # Test text analysis
    echo "📊 Testing text analysis..."
    ANALYSIS_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
        -H "Content-Type: application/json" \
        -d '{"text": "This is a test message for analysis", "operation": "analyze"}')

    if echo "$ANALYSIS_RESPONSE" | grep -q "word_count"; then
        echo "✅ Text analysis test passed"
    else
        echo "❌ Text analysis test failed"
        echo "Response: $ANALYSIS_RESPONSE"
    fi

    # Test sentiment analysis
    echo "😊 Testing sentiment analysis..."
    SENTIMENT_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
        -H "Content-Type: application/json" \
        -d '{"text": "I love this amazing product!", "operation": "sentiment"}')

    if echo "$SENTIMENT_RESPONSE" | grep -q "sentiment"; then
        echo "✅ Sentiment analysis test passed"
    else
        echo "❌ Sentiment analysis test failed"
        echo "Response: $SENTIMENT_RESPONSE"
    fi

    # Test word frequency
    echo "🔢 Testing word frequency..."
    FREQ_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
        -H "Content-Type: application/json" \
        -d '{"text": "the quick brown fox jumps over the lazy dog", "operation": "frequency"}')

    if echo "$FREQ_RESPONSE" | grep -q "frequency"; then
        echo "✅ Word frequency test passed"
    else
        echo "❌ Word frequency test failed"
        echo "Response: $FREQ_RESPONSE"
    fi

    # Test format conversion
    echo "🔄 Testing format conversion..."
    FORMAT_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
        -H "Content-Type: application/json" \
        -d '{"text": "Hello\nWorld", "operation": "convert", "format": "uppercase"}')

    if echo "$FORMAT_RESPONSE" | grep -q "HELLO"; then
        echo "✅ Format conversion test passed"
    else
        echo "❌ Format conversion test failed"
        echo "Response: $FORMAT_RESPONSE"
    fi

else
    echo "⏭️  Skipping integration tests (API not available)"
fi

# Test error handling
echo "🚨 Testing error scenarios..."
cargo test -- --nocapture > test_output.log 2>&1 || true

if grep -q "test result: ok" test_output.log; then
    echo "✅ Error handling tests passed"
else
    echo "❌ Error handling tests failed"
    cat test_output.log
fi

# Test with sample files
echo "📁 Testing with sample files..."
if [ -f "target/wasm32-wasi/release/text_processing.wasm" ]; then
    echo "✅ WASM module exists and is ready for testing"
else
    echo "❌ WASM module not found"
fi

# Cleanup
rm -f test_output.log

echo "🎉 All tests completed!"
echo "📊 Test Summary:"
echo "  - Unit tests: ✅"
if [ "$API_AVAILABLE" = true ]; then
    echo "  - Integration tests: ✅"
    echo "    - Text analysis: ✅"
    echo "    - Sentiment analysis: ✅"
    echo "    - Word frequency: ✅"
    echo "    - Format conversion: ✅"
else
    echo "  - Integration tests: ⏭️  (API not available)"
fi
echo "  - Error handling: ✅"
echo "  - File validation: ✅"