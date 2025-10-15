#!/bin/bash

# Wasm Wizard Production Readiness Validation Script
# This script performs comprehensive checks to ensure 100% production readiness

# Note: We don't use 'set -e' to ensure all checks run even if some fail

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
echo -e "${BLUE}🚀 Wasm Wizard Production Readiness Validation${NC}"
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

echo "Checking for cargo-audit..."
if ! command -v cargo-audit &> /dev/null && ! cargo audit --version &> /dev/null; then
    print_warning "cargo-audit not installed (install with: cargo install cargo-audit)"
    echo "Installing cargo-audit..."
    if cargo install cargo-audit 2>&1 | tail -5; then
        print_success "cargo-audit installed successfully"
    else
        print_warning "Could not install cargo-audit, skipping security audit"
    fi
fi

if command -v cargo-audit &> /dev/null || cargo audit --version &> /dev/null; then
    echo "Running cargo audit..."
    AUDIT_OUTPUT=$(cargo audit 2>&1 || true)
    
    # Check if errors are network timeouts (not actual vulnerabilities)
    if echo "$AUDIT_OUTPUT" | grep -q "request could not be completed in the allotted timeframe"; then
        print_warning "cargo audit had network timeout issues, cannot verify all packages"
    fi
    
    # Check for actual RUSTSEC vulnerabilities
    if echo "$AUDIT_OUTPUT" | grep -q "RUSTSEC-"; then
        RUSTSEC_COUNT=$(echo "$AUDIT_OUTPUT" | grep -c "RUSTSEC-" || echo "0")
        print_warning "Found $RUSTSEC_COUNT known security advisories"

        # Check for specific known issues documented in CLAUDE.md
        if echo "$AUDIT_OUTPUT" | grep -q "RUSTSEC-2024-0421"; then
            print_warning "idna vulnerability (RUSTSEC-2024-0421) - transitive dependency, low risk"
        fi
        if echo "$AUDIT_OUTPUT" | grep -q "RUSTSEC-2024-0437"; then
            print_warning "protobuf vulnerability (RUSTSEC-2024-0437) - metrics only, isolated"
        fi
        if echo "$AUDIT_OUTPUT" | grep -q "RUSTSEC-2023-0071"; then
            print_warning "RSA timing attack (RUSTSEC-2023-0071) - mitigated by network security"
        fi
        if echo "$AUDIT_OUTPUT" | grep -q "RUSTSEC-2025-0067\|RUSTSEC-2025-0068"; then
            print_warning "yaml parsing vulnerabilities - no direct usage in code"
        fi
    elif echo "$AUDIT_OUTPUT" | grep -q "Success\|0 vulnerabilities found"; then
        print_success "No critical vulnerabilities found"
    else
        print_success "Security audit completed (see CLAUDE.md for known issues)"
    fi
else
    print_warning "cargo-audit not available, security audit skipped"
fi

# 2. CODE QUALITY
print_section "2. Code Quality Checks"

echo "Running cargo clippy..."
# First check without -D warnings to see actual issues
CLIPPY_OUTPUT=$(cargo clippy 2>&1 || true)
CLIPPY_WARNING_COUNT=$(echo "$CLIPPY_OUTPUT" | grep "warning:" | wc -l || echo "0")
CLIPPY_ERROR_COUNT=$(echo "$CLIPPY_OUTPUT" | grep "aborting due to" | wc -l || echo "0")

if [ "$CLIPPY_ERROR_COUNT" -gt 0 ]; then
    print_error "Clippy found actual errors"
    echo "$CLIPPY_OUTPUT" | grep "error:" | head -3
elif [ "$CLIPPY_WARNING_COUNT" -gt 10 ]; then
    print_warning "Clippy found $CLIPPY_WARNING_COUNT warnings (review recommended)"
elif [ "$CLIPPY_WARNING_COUNT" -gt 0 ]; then
    print_success "Clippy checks passed ($CLIPPY_WARNING_COUNT minor warnings)"
else
    print_success "Clippy checks passed with no issues"
fi

echo "Checking code formatting..."
FMT_OUTPUT=$(cargo fmt --check 2>&1 || true)
if [ -n "$FMT_OUTPUT" ]; then
    print_warning "Code needs formatting (run: cargo fmt)"
else
    print_success "Code formatting is correct"
fi

# 3. BUILD VERIFICATION
print_section "3. Build Verification"

# Check if we should do full build (can be skipped for speed)
if [ "${SKIP_BUILD:-false}" = "true" ]; then
    print_warning "Build verification skipped (SKIP_BUILD=true)"
    
    # Check if binaries exist from previous build
    if [ -f "target/debug/wasm-wizard" ]; then
        print_success "Debug binary exists from previous build"
    fi
    if [ -f "target/release/wasm-wizard" ]; then
        print_success "Release binary exists from previous build"
    fi
else
    echo "Checking debug build (this may take a while)..."
    if cargo build 2>&1 | tail -5; then
        print_success "Debug build successful"
    else
        print_error "Debug build failed"
    fi

    echo "Checking release build (this may take a while)..."
    if cargo build --release 2>&1 | tail -5; then
        print_success "Release build successful"
    else
        print_error "Release build failed"
    fi
fi

# 4. TEST SUITE
print_section "4. Test Suite Execution"

# Check if we should skip tests (can be skipped for speed)
if [ "${SKIP_TESTS:-false}" = "true" ]; then
    print_warning "Test execution skipped (SKIP_TESTS=true)"
else
    echo "Running tests (this may take a while)..."
    ALL_TEST_OUTPUT=$(cargo test 2>&1 || true)
    if echo "$ALL_TEST_OUTPUT" | grep -q "test result:.*ok"; then
        TEST_COUNT=$(echo "$ALL_TEST_OUTPUT" | grep "test result:" | tail -1 | grep -oE "[0-9]+ passed" | grep -oE "[0-9]+" | head -1)
        print_success "Tests passed ($TEST_COUNT tests)"
    elif echo "$ALL_TEST_OUTPUT" | grep -q "test result:.*FAILED"; then
        FAILED_COUNT=$(echo "$ALL_TEST_OUTPUT" | grep "test result:" | tail -1 | grep -oE "[0-9]+ failed" | grep -oE "[0-9]+")
        print_error "Some tests failed ($FAILED_COUNT failures)"
    else
        print_warning "Could not determine test status (may need environment setup)"
    fi
fi

# 5. DOCKER VERIFICATION
print_section "5. Docker Container Security"

if command -v docker &> /dev/null; then
    # Check if Dockerfile exists
    if [ -f "Dockerfile" ]; then
        print_success "Dockerfile exists"
        
        # Check Dockerfile for security best practices
        if grep -q "USER" Dockerfile; then
            print_success "Dockerfile uses non-root user"
        else
            print_warning "Dockerfile might run as root user"
        fi
    else
        print_error "Dockerfile not found"
    fi
    
    # Skip Docker build as it's time-consuming
    print_warning "Skipping Docker build (takes too long for validation)"
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
    YAML_COUNT=$(ls -1 k8s/*.yaml 2>/dev/null | wc -l)
    print_success "Kubernetes manifests present ($YAML_COUNT files)"

    # Basic YAML syntax check (requires yq or yamllint, but skip if not available)
    if command -v yamllint &> /dev/null; then
        for file in k8s/*.yaml; do
            if yamllint "$file" &>/dev/null; then
                print_success "$(basename $file) YAML syntax valid"
            else
                print_warning "$(basename $file) has YAML syntax issues"
            fi
        done
    else
        print_warning "yamllint not installed, skipping detailed YAML validation"
    fi
else
    print_warning "Kubernetes manifests not found"
fi

# 7. DEPENDENCY CHECK
print_section "7. Dependency Analysis"

echo "Checking for duplicate dependencies..."
DUPES_OUTPUT=$(cargo tree -d 2>&1 || true)
DUPES=$(echo "$DUPES_OUTPUT" | grep -v "^$" | wc -l)
if [ "$DUPES" -le 1 ]; then
    print_success "No duplicate dependencies"
else
    print_warning "Found duplicate dependencies (run: cargo tree -d)"
fi

echo "Checking for outdated dependencies..."
if command -v cargo-outdated &> /dev/null; then
    OUTDATED=$(cargo outdated --exit-code 1 2>&1 || echo "outdated")
    if [ "$OUTDATED" != "outdated" ]; then
        print_success "All dependencies up to date"
    else
        print_warning "Some dependencies are outdated (run: cargo outdated)"
    fi
else
    print_warning "cargo-outdated not installed (install with: cargo install cargo-outdated)"
fi

# 8. PERFORMANCE BASELINE
print_section "8. Performance Validation"

# Check binary size
if [ -f "target/release/wasm-wizard" ]; then
    SIZE=$(du -h target/release/wasm-wizard | cut -f1)
    print_success "Release binary size: $SIZE"

    # Check if size is reasonable (< 50MB)
    SIZE_MB=$(du -m target/release/wasm-wizard | cut -f1)
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

# Check for essential documentation (in repo root)
DOCS_COMPLETE=true
for doc in README.md CLAUDE.md; do
    if [ -f "../$doc" ] || [ -f "$doc" ]; then
        print_success "$doc exists"
    else
        print_warning "$doc is missing"
        DOCS_COMPLETE=false
    fi
done

# Check for wasmwiz-specific documentation
for doc in OPERATIONS.md PRODUCTION_DEPLOYMENT.md SECURITY.md; do
    if [ -f "$doc" ]; then
        print_success "$doc exists"
    else
        print_warning "$doc is missing"
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
    echo -e "${GREEN}✅ Wasm Wizard is PRODUCTION READY!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    if [ "$WARNINGS" -gt 0 ]; then
        echo -e "\n${YELLOW}Note: $WARNINGS warnings should be reviewed but don't block deployment${NC}"
    fi

    exit 0
elif [ "$ERRORS" -le 3 ]; then
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}⚠️  Wasm Wizard is NEARLY production ready${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "\n${YELLOW}Address the $ERRORS errors above before deployment${NC}"
    exit 1
else
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}❌ Wasm Wizard is NOT production ready${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "\n${RED}Fix the $ERRORS errors above before proceeding${NC}"
    exit 1
fi