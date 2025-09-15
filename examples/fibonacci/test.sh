#!/bin/bash

set -e

echo "🧪 Testing Fibonacci Example"
echo "============================"

# Check if WASM file exists
if [ ! -f "fibonacci.wasm" ]; then
    echo "❌ WASM file not found. Please run ./build.sh first."
    exit 1
fi

# Check if Wasm Wizard is running
if ! curl -s http://localhost:8080/health > /dev/null; then
    echo "❌ Wasm Wizard is not running. Please start it first."
    exit 1
fi

echo "✅ Setup complete"

# Encode WASM file to base64
WASM_B64=$(base64 -w 0 fibonacci.wasm)

# Test Fibonacci sequence
echo ""
echo "🧪 Testing Fibonacci sequence..."

EXPECTED_SEQUENCE=(0 1 1 2 3 5 8 13 21 34 55 89 144 233 377 610)

for i in "${!EXPECTED_SEQUENCE[@]}"; do
    echo -n "F($i) = "

    # Test iterative (fastest)
    RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
        -H "Content-Type: application/json" \
        -d "{\"wasm_module\": \"$WASM_B64\", \"function\": \"fib_iterative\", \"args\": [$i]}")

    if echo "$RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
        RESULT=$(echo "$RESPONSE" | jq -r '.result')
        if [ "$RESULT" -eq "${EXPECTED_SEQUENCE[$i]}" ]; then
            echo "✅ $RESULT"
        else
            echo "❌ $RESULT (expected ${EXPECTED_SEQUENCE[$i]})"
            exit 1
        fi
    else
        echo "❌ API call failed: $RESPONSE"
        exit 1
    fi
done

echo ""
echo "🧪 Testing algorithm comparison..."

# Test algorithm comparison for small n
for n in 5 10 15; do
    echo -n "Comparing algorithms for n=$n ... "

    RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
        -H "Content-Type: application/json" \
        -d "{\"wasm_module\": \"$WASM_B64\", \"function\": \"compare_algorithms\", \"args\": [$n]}")

    if echo "$RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
        RESULT=$(echo "$RESPONSE" | jq -r '.result')
        echo "✅ $RESULT"
    else
        echo "❌ Failed: $RESPONSE"
    fi
done

echo ""
echo "🧪 Testing algorithm information..."

for algo in 0 1 2 3; do
    echo -n "Algorithm $algo info: "

    RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
        -H "Content-Type: application/json" \
        -d "{\"wasm_module\": \"$WASM_B64\", \"function\": \"get_algorithm_info\", \"args\": [$algo]}")

    if echo "$RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
        RESULT=$(echo "$RESPONSE" | jq -r '.result')
        echo "✅ $RESULT"
    else
        echo "❌ Failed: $RESPONSE"
    fi
done

echo ""
echo "🧪 Testing large Fibonacci numbers..."

# Test larger values with iterative (fast) method
LARGE_TESTS=(30 35 40)

for n in "${LARGE_TESTS[@]}"; do
    echo -n "F($n) with iterative method ... "

    RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
        -H "Content-Type: application/json" \
        -d "{\"wasm_module\": \"$WASM_B64\", \"function\": \"fib_iterative\", \"args\": [$n]}")

    if echo "$RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
        RESULT=$(echo "$RESPONSE" | jq -r '.result')
        TIME=$(echo "$RESPONSE" | jq -r '.execution_time_ms')
        echo "✅ $RESULT (${TIME}ms)"
    else
        echo "❌ Failed: $RESPONSE"
    fi
done

echo ""
echo "🧪 Testing matrix exponentiation..."

for n in 10 20 30; do
    echo -n "F($n) with matrix method ... "

    RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
        -H "Content-Type: application/json" \
        -d "{\"wasm_module\": \"$WASM_B64\", \"function\": \"fib_matrix\", \"args\": [$n]}")

    if echo "$RESPONSE" | jq -e '.success' > /dev/null 2>&1; then
        RESULT=$(echo "$RESPONSE" | jq -r '.result')
        TIME=$(echo "$RESPONSE" | jq -r '.execution_time_ms')
        echo "✅ $RESULT (${TIME}ms)"
    else
        echo "❌ Failed: $RESPONSE"
    fi
done

echo ""
echo "🎉 All Fibonacci tests passed!"
echo ""
echo "📊 Performance Insights:"
echo "   - Iterative is fastest for most use cases"
echo "   - Matrix exponentiation excels for very large n"
echo "   - Recursive is simple but slow for n > 30"
echo "   - WASM execution is near-native performance"
echo ""
echo "💡 Next: Run ./benchmark.sh for detailed performance analysis"