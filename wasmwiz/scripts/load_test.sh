#!/bin/bash
# Wasm Wizard Load Testing Script
# This script performs comprehensive load testing of the Wasm Wizard API

set -euo pipefail

# Configuration
BASE_URL="${BASE_URL:-http://localhost:8080}"
MAX_CONCURRENT="${MAX_CONCURRENT:-50}"
TOTAL_REQUESTS="${TOTAL_REQUESTS:-1000}"
TEST_DURATION="${TEST_DURATION:-60}"
OUTPUT_DIR="${OUTPUT_DIR:-./load_test_results}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

# Error handling
handle_error() {
    log "${RED}ERROR: Load testing failed on line $1${NC}"
    exit 1
}
trap 'handle_error $LINENO' ERR

# Check dependencies
check_dependencies() {
    log "Checking dependencies..."
    
    local missing_deps=()
    
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi
    
    if ! command -v ab &> /dev/null; then
        missing_deps+=("apache2-utils (for ab)")
    fi
    
    if ! command -v wrk &> /dev/null; then
        log "${YELLOW}Warning: wrk not found. Install with: sudo apt-get install wrk${NC}"
    fi
    
    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log "${RED}Missing dependencies: ${missing_deps[*]}${NC}"
        log "Install with: sudo apt-get install ${missing_deps[*]}"
        exit 1
    fi
    
    log "${GREEN}All dependencies satisfied${NC}"
}

# Pre-flight checks
pre_flight_checks() {
    log "Running pre-flight checks..."
    
    # Check if service is running
    if ! curl -f "$BASE_URL/health" >/dev/null 2>&1; then
        log "${RED}ERROR: Wasm Wizard service is not responding at $BASE_URL${NC}"
        exit 1
    fi
    
    # Check if metrics endpoint is accessible
    if ! curl -f "$BASE_URL/metrics" >/dev/null 2>&1; then
        log "${YELLOW}Warning: Metrics endpoint not accessible${NC}"
    fi
    
    log "${GREEN}Pre-flight checks passed${NC}"
}

# Setup test environment
setup_test_environment() {
    log "Setting up test environment..."
    
    mkdir -p "$OUTPUT_DIR"
    
    # Create test WASM file (minimal valid WASM module)
    cat > "$OUTPUT_DIR/test.wasm" << EOF
(module)
EOF
    
    # Convert to binary WASM
    if command -v wat2wasm &> /dev/null; then
        wat2wasm "$OUTPUT_DIR/test.wasm" -o "$OUTPUT_DIR/test.wasm"
    else
        # Create minimal binary WASM manually
        printf '\x00\x61\x73\x6d\x01\x00\x00\x00' > "$OUTPUT_DIR/test.wasm"
    fi
    
    log "Test environment ready"
}

# Health endpoint load test
test_health_endpoint() {
    log "Testing health endpoint performance..."
    
    # Apache Bench test
    log "Running Apache Bench test on /health..."
    ab -n "$TOTAL_REQUESTS" -c "$MAX_CONCURRENT" \
       -g "$OUTPUT_DIR/health_gnuplot.dat" \
       "$BASE_URL/health" > "$OUTPUT_DIR/health_ab_results.txt" 2>&1
    
    # Extract key metrics
    local requests_per_sec
    requests_per_sec=$(grep "Requests per second" "$OUTPUT_DIR/health_ab_results.txt" | awk '{print $4}')
    
    local time_per_request
    time_per_request=$(grep "Time per request" "$OUTPUT_DIR/health_ab_results.txt" | head -1 | awk '{print $4}')
    
    log "Health endpoint results:"
    log "  Requests/sec: $requests_per_sec"
    log "  Time/request: ${time_per_request}ms"
}

# WRK-based load test
test_with_wrk() {
    if ! command -v wrk &> /dev/null; then
        log "${YELLOW}Skipping wrk tests (not installed)${NC}"
        return
    fi
    
    log "Running wrk load test..."
    
    wrk -t12 -c400 -d${TEST_DURATION}s \
        --script="$OUTPUT_DIR/wrk_script.lua" \
        "$BASE_URL/health" > "$OUTPUT_DIR/wrk_results.txt" 2>&1
    
    # Create wrk script for more complex testing
    cat > "$OUTPUT_DIR/wrk_script.lua" << 'EOF'
-- WRK script for Wasm Wizard load testing
wrk.method = "GET"
wrk.headers["Content-Type"] = "application/json"

request = function()
    local path = "/health"
    return wrk.format(wrk.method, path)
end

response = function(status, headers, body)
    if status ~= 200 then
        print("Error: " .. status)
    end
end
EOF
    
    log "WRK test completed"
}

# API endpoint stress test
test_api_endpoints() {
    log "Testing API endpoint performance..."
    
    # Test metrics endpoint
    log "Testing /metrics endpoint..."
    ab -n 100 -c 10 "$BASE_URL/metrics" > "$OUTPUT_DIR/metrics_ab_results.txt" 2>&1
    
    # Test WASM upload endpoint (expect failures due to auth, but test performance)
    log "Testing /api/wasm/upload endpoint (auth failures expected)..."
    for i in {1..10}; do
        curl -X POST "$BASE_URL/api/wasm/upload" \
             -F "wasm=@$OUTPUT_DIR/test.wasm" \
             -F "name=test-module-$i" \
             -w "%{http_code},%{time_total},%{time_connect},%{time_starttransfer}\n" \
             -o /dev/null -s >> "$OUTPUT_DIR/upload_test_results.csv" 2>&1 || true
    done
    
    log "API endpoint tests completed"
}

# Memory and resource monitoring
monitor_resources() {
    log "Starting resource monitoring..."
    
    local monitor_pid
    {
        while true; do
            # Get container stats if running in Docker
            if docker ps | grep -q wasm-wizard; then
                docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}" | grep wasm-wizard
            fi
            
            # System memory and CPU
            free -h
            top -bn1 | grep "Cpu(s)" 
            
            sleep 5
        done
    } > "$OUTPUT_DIR/resource_monitor.log" 2>&1 &
    monitor_pid=$!
    
    echo $monitor_pid > "$OUTPUT_DIR/monitor.pid"
    log "Resource monitoring started (PID: $monitor_pid)"
}

# Stop monitoring
stop_monitoring() {
    if [[ -f "$OUTPUT_DIR/monitor.pid" ]]; then
        local monitor_pid
        monitor_pid=$(cat "$OUTPUT_DIR/monitor.pid")
        if kill -0 "$monitor_pid" 2>/dev/null; then
            kill "$monitor_pid"
            log "Resource monitoring stopped"
        fi
        rm "$OUTPUT_DIR/monitor.pid"
    fi
}

# Performance analysis
analyze_results() {
    log "Analyzing performance results..."
    
    # Create summary report
    cat > "$OUTPUT_DIR/performance_summary.md" << EOF
# Wasm Wizard Load Testing Results

Generated: $(date)

## Test Configuration
- Base URL: $BASE_URL
- Max Concurrent: $MAX_CONCURRENT
- Total Requests: $TOTAL_REQUESTS
- Test Duration: ${TEST_DURATION}s

## Health Endpoint Results
EOF
    
    if [[ -f "$OUTPUT_DIR/health_ab_results.txt" ]]; then
        echo "### Apache Bench Results" >> "$OUTPUT_DIR/performance_summary.md"
        echo '```' >> "$OUTPUT_DIR/performance_summary.md"
        grep -E "(Requests per second|Time per request|Transfer rate)" \
            "$OUTPUT_DIR/health_ab_results.txt" >> "$OUTPUT_DIR/performance_summary.md"
        echo '```' >> "$OUTPUT_DIR/performance_summary.md"
    fi
    
    if [[ -f "$OUTPUT_DIR/wrk_results.txt" ]]; then
        echo "### WRK Results" >> "$OUTPUT_DIR/performance_summary.md"
        echo '```' >> "$OUTPUT_DIR/performance_summary.md"
        cat "$OUTPUT_DIR/wrk_results.txt" >> "$OUTPUT_DIR/performance_summary.md"
        echo '```' >> "$OUTPUT_DIR/performance_summary.md"
    fi
    
    # Performance recommendations
    cat >> "$OUTPUT_DIR/performance_summary.md" << EOF

## Performance Analysis

### Baseline Expectations
- Health endpoint should handle >1000 req/s
- Average response time should be <50ms
- 99th percentile should be <200ms

### Recommendations
1. Monitor database connection pool utilization
2. Check Redis performance metrics
3. Consider horizontal scaling if CPU > 80%
4. Optimize static asset serving with CDN

### Next Steps
1. Run tests with different load patterns
2. Test with actual WASM execution workloads
3. Monitor long-term performance trends
EOF
    
    log "${GREEN}Performance analysis complete. See $OUTPUT_DIR/performance_summary.md${NC}"
}

# Cleanup function
cleanup() {
    log "Cleaning up..."
    stop_monitoring
    log "Load testing completed"
}

# Main execution
main() {
    log "${GREEN}Starting Wasm Wizard load testing...${NC}"
    
    # Setup trap for cleanup
    trap cleanup EXIT
    
    check_dependencies
    pre_flight_checks
    setup_test_environment
    
    # Start monitoring
    monitor_resources
    
    # Run tests
    test_health_endpoint
    test_with_wrk
    test_api_endpoints
    
    # Stop monitoring and analyze
    stop_monitoring
    analyze_results
    
    log "${GREEN}Load testing completed successfully!${NC}"
    log "Results available in: $OUTPUT_DIR"
}

# Help function
show_help() {
    cat << EOF
Wasm Wizard Load Testing Script

Usage: $0 [OPTIONS]

OPTIONS:
    -h, --help              Show this help message
    -u, --url URL          Base URL for testing (default: http://localhost:8080)
    -c, --concurrent N     Max concurrent requests (default: 50)
    -n, --requests N       Total number of requests (default: 1000)
    -d, --duration N       Test duration in seconds (default: 60)
    -o, --output DIR       Output directory (default: ./load_test_results)

EXAMPLES:
    $0                                    # Run with defaults
    $0 -u https://api.wasm-wizard.com        # Test production API
    $0 -c 100 -n 5000 -d 120            # Heavy load test
    $0 -o /tmp/results                   # Custom output directory

DEPENDENCIES:
    - curl
    - apache2-utils (ab)
    - jq
    - wrk (optional)
EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -u|--url)
            BASE_URL="$2"
            shift 2
            ;;
        -c|--concurrent)
            MAX_CONCURRENT="$2"
            shift 2
            ;;
        -n|--requests)
            TOTAL_REQUESTS="$2"
            shift 2
            ;;
        -d|--duration)
            TEST_DURATION="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        *)
            log "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Run main function
main "$@"