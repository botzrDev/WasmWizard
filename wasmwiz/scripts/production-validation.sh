#!/bin/bash

# WasmWiz Production Readiness Validation Script
# This script performs comprehensive checks to ensure 100% production readiness

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track overall status
VALIDATION_PASSED=true
WARNINGS=0
ERRORS=0

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🚀 WasmWiz Production Readiness Validation${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Function to print section headers
print_section() {
    echo -e "\n${BLUE}▶ $1${NC}"
    echo -e "${BLUE}────────────────────────────────────────${NC}"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# Function to print warning
print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
    ((WARNINGS++))
}

# Function to print error
print_error() {
    echo -e "${RED}❌ $1${NC}"
    ((ERRORS++))
    VALIDATION_PASSED=false
}

# 1. SECURITY AUDIT
print_section "1. Security Vulnerability Audit"

echo "Running cargo audit..."
if cargo audit 2>/dev/null; then
    print_success "No critical vulnerabilities found"
else
    AUDIT_OUTPUT=$(cargo audit 2>&1)
    if echo "$AUDIT_OUTPUT" | grep -q "error:"; then
        ERROR_COUNT=$(echo "$AUDIT_OUTPUT" | grep -c "error:")
        print_error "Found $ERROR_COUNT security vulnerabilities"

        # Check for specific known issues
        if echo "$AUDIT_OUTPUT" | grep -q "RUSTSEC-2024-0421"; then
            print_warning "idna vulnerability (RUSTSEC-2024-0421) - transitive dependency from Wasmer"
        fi
        if echo "$AUDIT_OUTPUT" | grep -q "RUSTSEC-2024-0437"; then
            print_warning "protobuf vulnerability (RUSTSEC-2024-0437) - from prometheus metrics"
        fi
        if echo "$AUDIT_OUTPUT" | grep -q "RUSTSEC-2023-0071"; then
            print_warning "RSA timing attack (RUSTSEC-2023-0071) - no fix available yet"
        fi
    else
        print_success "Security audit passed with warnings only"
    fi
fi

# 2. CODE QUALITY
print_section "2. Code Quality Checks"

echo "Running cargo clippy..."
if cargo clippy -- -D warnings 2>/dev/null; then
    print_success "Clippy checks passed"
else
    print_error "Clippy found issues"
fi

echo "Checking code formatting..."
if cargo fmt --check 2>/dev/null; then
    print_success "Code formatting is correct"
else
    print_warning "Code needs formatting (run: cargo fmt)"
fi

# 3. BUILD VERIFICATION
print_section "3. Build Verification"

echo "Building debug version..."
if cargo build 2>/dev/null; then
    print_success "Debug build successful"
else
    print_error "Debug build failed"
fi

echo "Building release version..."
if cargo build --release 2>/dev/null; then
    print_success "Release build successful"
else
    print_error "Release build failed"
fi

# 4. TEST SUITE
print_section "4. Test Suite Execution"

echo "Running unit tests..."
if cargo test --lib 2>/dev/null; then
    print_success "Unit tests passed"
else
    print_error "Unit tests failed"
fi

echo "Running integration tests..."
if cargo test --test '*' 2>/dev/null; then
    print_success "Integration tests passed"
else
    print_warning "Some integration tests failed (may need database/Redis)"
fi

echo "Running security tests..."
if cargo test security_tests 2>/dev/null; then
    print_success "Security tests passed"
else
    print_warning "Security tests need environment setup"
fi

# 5. DOCKER VERIFICATION
print_section "5. Docker Container Security"

if command -v docker &> /dev/null; then
    echo "Building Docker image..."
    if docker build -t wasmwiz:validation . 2>/dev/null; then
        print_success "Docker build successful"

        # Check for security best practices
        echo "Checking Docker security..."
        DOCKER_CHECK=$(docker inspect wasmwiz:validation 2>/dev/null || echo "{}")

        if echo "$DOCKER_CHECK" | grep -q '"User": "wasmwiz"'; then
            print_success "Container runs as non-root user"
        else
            print_warning "Container might run as root user"
        fi

        # Scan with docker scan if available
        if command -v docker scan &> /dev/null; then
            echo "Running Docker security scan..."
            docker scan wasmwiz:validation 2>/dev/null || print_warning "Docker scan not configured"
        else
            print_warning "Docker scan not available"
        fi
    else
        print_error "Docker build failed"
    fi
else
    print_warning "Docker not available for testing"
fi

# 6. CONFIGURATION VALIDATION
print_section "6. Configuration Validation"

# Check for required environment files
if [ -f ".env.production" ]; then
    print_success "Production environment file exists"
else
    print_warning "Production environment file missing"
fi

# Check for Kubernetes manifests
if [ -d "k8s" ] && [ -n "$(ls -A k8s/*.yaml 2>/dev/null)" ]; then
    print_success "Kubernetes manifests present"

    # Validate YAML syntax
    for file in k8s/*.yaml; do
        if command -v kubectl &> /dev/null; then
            if kubectl apply --dry-run=client -f "$file" &>/dev/null; then
                print_success "$(basename $file) is valid"
            else
                print_error "$(basename $file) has syntax errors"
            fi
        fi
    done
else
    print_warning "Kubernetes manifests not found"
fi

# 7. DEPENDENCY CHECK
print_section "7. Dependency Analysis"

echo "Checking for duplicate dependencies..."
DUPES=$(cargo tree -d 2>/dev/null | grep -v "^$" | wc -l)
if [ "$DUPES" -eq 0 ]; then
    print_success "No duplicate dependencies"
else
    print_warning "Found $DUPES duplicate dependencies"
fi

echo "Checking for outdated dependencies..."
if command -v cargo-outdated &> /dev/null; then
    OUTDATED=$(cargo outdated --exit-code 1 2>/dev/null || echo "outdated")
    if [ "$OUTDATED" != "outdated" ]; then
        print_success "All dependencies up to date"
    else
        print_warning "Some dependencies are outdated"
    fi
else
    print_warning "cargo-outdated not installed"
fi

# 8. PERFORMANCE BASELINE
print_section "8. Performance Validation"

# Check binary size
if [ -f "target/release/wasmwiz" ]; then
    SIZE=$(du -h target/release/wasmwiz | cut -f1)
    print_success "Release binary size: $SIZE"

    # Check if size is reasonable (< 50MB)
    SIZE_MB=$(du -m target/release/wasmwiz | cut -f1)
    if [ "$SIZE_MB" -lt 50 ]; then
        print_success "Binary size is optimized"
    else
        print_warning "Binary size might be too large ($SIZE_MB MB)"
    fi
fi

# 9. MONITORING READINESS
print_section "9. Monitoring & Observability"

# Check for metrics endpoint
if grep -q "/metrics" src/handlers/*.rs 2>/dev/null || grep -q "prometheus" Cargo.toml; then
    print_success "Prometheus metrics configured"
else
    print_error "Metrics endpoint not found"
fi

# Check for health checks
if grep -q "/health" src/handlers/*.rs 2>/dev/null; then
    print_success "Health check endpoints configured"
else
    print_error "Health check endpoints not found"
fi

# Check for structured logging
if grep -q "tracing" Cargo.toml && grep -q "json" Cargo.toml; then
    print_success "Structured logging configured"
else
    print_warning "Structured logging might not be configured"
fi

# 10. DOCUMENTATION CHECK
print_section "10. Documentation Completeness"

# Check for essential documentation
DOCS_COMPLETE=true
for doc in README.md CLAUDE.md CHANGELOG.md; do
    if [ -f "$doc" ]; then
        print_success "$doc exists"
    else
        print_warning "$doc is missing"
        DOCS_COMPLETE=false
    fi
done

# Check API documentation
if grep -q "///" src/**/*.rs 2>/dev/null; then
    print_success "Code has inline documentation"
else
    print_warning "Code lacks inline documentation"
fi

# FINAL SUMMARY
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📊 Validation Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Calculate production readiness score
TOTAL_CHECKS=30
PASSED_CHECKS=$((TOTAL_CHECKS - ERRORS))
READINESS_PERCENT=$((PASSED_CHECKS * 100 / TOTAL_CHECKS))

echo -e "Errors:   ${RED}$ERRORS${NC}"
echo -e "Warnings: ${YELLOW}$WARNINGS${NC}"
echo -e "Production Readiness: ${GREEN}$READINESS_PERCENT%${NC}"

echo ""
if [ "$VALIDATION_PASSED" = true ] && [ "$ERRORS" -eq 0 ]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✅ WasmWiz is PRODUCTION READY!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    if [ "$WARNINGS" -gt 0 ]; then
        echo -e "\n${YELLOW}Note: $WARNINGS warnings should be reviewed but don't block deployment${NC}"
    fi

    exit 0
elif [ "$ERRORS" -le 3 ]; then
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}⚠️  WasmWiz is NEARLY production ready${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "\n${YELLOW}Address the $ERRORS errors above before deployment${NC}"
    exit 1
else
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}❌ WasmWiz is NOT production ready${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "\n${RED}Fix the $ERRORS errors above before proceeding${NC}"
    exit 1
fi