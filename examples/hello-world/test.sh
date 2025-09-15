#!/bin/bash

set -e

echo "🧪 Testing Hello World Example"
echo "==============================="

# Check if WASM file exists
if [ ! -f "hello_world.wasm" ]; then
    echo "❌ WASM file not found. Please run ./build.sh first."
    exit 1
fi

# Check if Wasm Wizard is running
if ! curl -s http://localhost:8080/health > /dev/null; then
    echo "❌ Wasm Wizard is not running. Please start it first:"
    echo "   cd ../../wasmwiz"
    echo "   docker-compose -f docker-compose.dev.yml up -d"
    echo "   cargo run"
    exit 1
fi

echo "✅ Wasm Wizard is running"

# Encode WASM file to base64
WASM_B64=$(base64 -w 0 hello_world.wasm)
echo "📦 WASM module encoded (size: $(echo $WASM_B64 | wc -c) chars)"

# Test cases
TEST_CASES=(
    "World"
    "Alice"
    "Bob"
    ""
    "🚀 WASM"
    "Hello World with Spaces"
)

echo ""
echo "🧪 Running test cases..."

for name in "${TEST_CASES[@]}"; do
    echo -n "Testing: '$name' ... "

    # Create JSON payload
    JSON_PAYLOAD=$(cat <<EOF
{
  "wasm_module": "$WASM_B64",
  "function": "greet",
  "args": ["$name"]
}
EOF
)

    # Make API call
    RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
        -H "Content-Type: application/json" \
        -d "$JSON_PAYLOAD")

    # Check if request was successful
    if echo "$RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
        RESULT=$(echo "$RESPONSE" | jq -r '.result')
        TIME=$(echo "$RESPONSE" | jq -r '.execution_time_ms')
        echo "✅ '$RESULT' (${TIME}ms)"
    else
        echo "❌ Failed: $RESPONSE"
        exit 1
    fi
done

echo ""
echo "🧪 Testing additional functions..."

# Test add_numbers function
echo -n "Testing add_numbers(5, 3) ... "
ADD_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
    -H "Content-Type: application/json" \
    -d "{\"wasm_module\": \"$WASM_B64\", \"function\": \"add_numbers\", \"args\": [5, 3]}")

if echo "$ADD_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
    RESULT=$(echo "$ADD_RESPONSE" | jq -r '.result')
    echo "✅ Result: $RESULT"
else
    echo "❌ Failed: $ADD_RESPONSE"
fi

# Test get_version function
echo -n "Testing get_version() ... "
VERSION_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
    -H "Content-Type: application/json" \
    -d "{\"wasm_module\": \"$WASM_B64\", \"function\": \"get_version\", \"args\": []}")

if echo "$VERSION_RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
    RESULT=$(echo "$VERSION_RESPONSE" | jq -r '.result')
    echo "✅ $RESULT"
else
    echo "❌ Failed: $VERSION_RESPONSE"
fi

echo ""
echo "🎉 All tests passed!"
echo ""
echo "💡 Tips:"
echo "   - Try modifying src/lib.rs and rebuilding"
echo "   - Check the API documentation for more options"
echo "   - Run ./benchmark.sh for performance tests"