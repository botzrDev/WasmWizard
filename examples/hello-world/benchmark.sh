#!/bin/bash

set -e

echo "📊 Benchmarking Hello World Example"
echo "==================================="

# Check if WASM file exists
if [ ! -f "hello_world.wasm" ]; then
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
WASM_B64=$(base64 -w 0 hello_world.wasm)

# Benchmark parameters
ITERATIONS=100
TEST_NAME="Alice"

echo ""
echo "🚀 Running benchmark ($ITERATIONS iterations)..."

# Create JSON payload
JSON_PAYLOAD=$(cat <<EOF
{
  "wasm_module": "$WASM_B64",
  "function": "greet",
  "args": ["$TEST_NAME"]
}
EOF
)

# Arrays to store timing data
declare -a execution_times
total_time=0
min_time=999999
max_time=0

echo "Progress: [0/$ITERATIONS]"

for i in $(seq 1 $ITERATIONS); do
    # Make API call and capture timing
    start_time=$(date +%s%N)
    RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
        -H "Content-Type: application/json" \
        -d "$JSON_PAYLOAD")
    end_time=$(date +%s%N)

    # Calculate execution time in milliseconds
    execution_time=$(( (end_time - start_time) / 1000000 ))

    # Store timing data
    execution_times[$i]=$execution_time
    total_time=$((total_time + execution_time))

    # Update min/max
    if [ $execution_time -lt $min_time ]; then
        min_time=$execution_time
    fi
    if [ $execution_time -gt $max_time ]; then
        max_time=$execution_time
    fi

    # Progress indicator
    if [ $((i % 10)) -eq 0 ]; then
        echo "Progress: [$i/$ITERATIONS]"
    fi
done

echo "Progress: [$ITERATIONS/$ITERATIONS]"
echo ""

# Calculate statistics
average_time=$((total_time / ITERATIONS))

# Calculate standard deviation
sum_squared_diff=0
for time in "${execution_times[@]}"; do
    diff=$((time - average_time))
    squared_diff=$((diff * diff))
    sum_squared_diff=$((sum_squared_diff + squared_diff))
done
variance=$((sum_squared_diff / ITERATIONS))
std_dev=$(echo "sqrt($variance)" | bc -l 2>/dev/null || echo "0")

# Calculate percentiles
sorted_times=($(printf '%s\n' "${execution_times[@]}" | sort -n))
p50=${sorted_times[$((ITERATIONS / 2))]}
p95=${sorted_times[$((ITERATIONS * 95 / 100))]}
p99=${sorted_times[$((ITERATIONS * 99 / 100))]}

echo "📊 Benchmark Results:"
echo "===================="
echo "Iterations: $ITERATIONS"
echo "Test Input: '$TEST_NAME'"
echo "WASM Size: $(ls -lh hello_world.wasm | awk '{print $5}')"
echo ""
echo "Execution Time (ms):"
echo "  Min:     $min_time"
echo "  Max:     $max_time"
echo "  Average: $average_time"
echo "  P50:     $p50"
echo "  P95:     $p95"
echo "  P99:     $p99"
echo "  Std Dev: ${std_dev%.*}"
echo ""
echo "Throughput: $((1000 / average_time)) requests/second"
echo ""

# Test memory usage (single request)
echo "🧠 Memory Usage Test:"
MEMORY_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/execute \
    -H "Content-Type: application/json" \
    -d "$JSON_PAYLOAD")

if echo "$MEMORY_RESPONSE" | jq -e '.memory_used_kb' > /dev/null 2>&1; then
    memory_kb=$(echo "$MEMORY_RESPONSE" | jq -r '.memory_used_kb')
    echo "  Memory Used: $memory_kb KB"
else
    echo "  Memory tracking not available"
fi

echo ""
echo "💡 Performance Notes:"
echo "   - WASM execution includes sandboxing overhead"
echo "   - Memory usage is per-request"
echo "   - Times include network latency"
echo "   - For comparison, run the same test with native Rust"

echo ""
echo "🎯 Optimization Tips:"
echo "   - Use --release flag for production builds"
echo "   - Minimize string operations in hot paths"
echo "   - Consider caching compiled WASM modules"
echo "   - Profile with cargo flamegraph for bottlenecks"