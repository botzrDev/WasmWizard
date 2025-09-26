#!/bin/bash

# Example: CI/CD Deployment with Temporary PAT
# This script demonstrates how to use the PAT manager for deployment workflows

set -euo pipefail

# Configuration
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PAT_MANAGER="${SCRIPT_DIR}/../github-pat-manager.sh"
readonly DEPLOYMENT_LOG="${SCRIPT_DIR}/deployment.log"

# Colors for output
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly RED='\033[0;31m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $1" >> "${DEPLOYMENT_LOG}"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" >&2
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARN: $1" >> "${DEPLOYMENT_LOG}"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $1" >> "${DEPLOYMENT_LOG}"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $1" >> "${DEPLOYMENT_LOG}"
}

# Check if PAT manager exists
check_pat_manager() {
    if [[ ! -f "$PAT_MANAGER" ]]; then
        log_error "PAT manager script not found at: $PAT_MANAGER"
        exit 1
    fi
    
    if [[ ! -x "$PAT_MANAGER" ]]; then
        log_error "PAT manager script is not executable: $PAT_MANAGER"
        exit 1
    fi
}

# Deployment function with PAT
deploy_with_pat() {
    local environment="$1"
    local duration_hours="$2"
    
    log_info "Starting deployment to $environment environment"
    log_info "PAT duration: $duration_hours hours"
    
    # Create deployment-specific PAT
    local description="WasmWizard Deploy to $environment $(date '+%Y-%m-%d %H:%M')"
    local scopes="repo,packages:write,contents:write"
    
    log_info "Creating temporary PAT for deployment..."
    local pat_token
    if ! pat_token=$("$PAT_MANAGER" create \
        -d "$description" \
        -s "$scopes" \
        -e "$duration_hours" 2>/dev/null); then
        log_error "Failed to create deployment PAT"
        return 1
    fi
    
    log_success "Temporary PAT created successfully"
    
    # Use the PAT for deployment operations (never log the actual token)
    export GITHUB_TOKEN="$pat_token"
    
    # Example deployment steps
    deploy_application "$environment"
    local deploy_exit_code=$?
    
    # Always cleanup, even if deployment failed
    log_info "Cleaning up expired PATs..."
    "$PAT_MANAGER" cleanup
    
    if [[ $deploy_exit_code -eq 0 ]]; then
        log_success "Deployment to $environment completed successfully"
    else
        log_error "Deployment to $environment failed"
        return $deploy_exit_code
    fi
}

# Simulate deployment operations
deploy_application() {
    local environment="$1"
    
    log_info "Executing deployment steps for $environment..."
    
    # Step 1: Clone repository (using PAT)
    log_info "Step 1: Cloning repository..."
    if clone_repository; then
        log_success "Repository cloned successfully"
    else
        log_error "Failed to clone repository"
        return 1
    fi
    
    # Step 2: Build application
    log_info "Step 2: Building application..."
    if build_application; then
        log_success "Application built successfully"
    else
        log_error "Failed to build application"
        return 1
    fi
    
    # Step 3: Run tests
    log_info "Step 3: Running tests..."
    if run_tests; then
        log_success "Tests passed successfully"
    else
        log_error "Tests failed"
        return 1
    fi
    
    # Step 4: Deploy to environment
    log_info "Step 4: Deploying to $environment..."
    if deploy_to_environment "$environment"; then
        log_success "Deployed to $environment successfully"
    else
        log_error "Failed to deploy to $environment"
        return 1
    fi
    
    # Step 5: Verify deployment
    log_info "Step 5: Verifying deployment..."
    if verify_deployment "$environment"; then
        log_success "Deployment verification passed"
    else
        log_error "Deployment verification failed"
        return 1
    fi
    
    return 0
}

# Individual deployment steps (mock implementations)
clone_repository() {
    log_info "Cloning WasmWizard repository..."
    # In real implementation: git clone https://x-access-token:$GITHUB_TOKEN@github.com/botzrDev/WasmWizard.git
    sleep 2  # Simulate clone time
    return 0
}

build_application() {
    log_info "Building WasmWizard application..."
    # In real implementation: cargo build --release
    sleep 3  # Simulate build time
    return 0
}

run_tests() {
    log_info "Running application tests..."
    # In real implementation: cargo test --all-features
    sleep 2  # Simulate test time
    return 0
}

deploy_to_environment() {
    local environment="$1"
    log_info "Deploying to $environment environment..."
    
    case "$environment" in
        staging)
            log_info "Deploying to staging server..."
            # kubectl apply -f k8s/staging/
            sleep 2
            ;;
        production)
            log_info "Deploying to production server..."
            # kubectl apply -f k8s/production/
            sleep 3
            ;;
        *)
            log_error "Unknown environment: $environment"
            return 1
            ;;
    esac
    
    return 0
}

verify_deployment() {
    local environment="$1"
    log_info "Verifying deployment in $environment..."
    
    # In real implementation: health checks, smoke tests, etc.
    sleep 1
    return 0
}

# Show usage information
show_usage() {
    cat << EOF
WasmWizard CI/CD Deployment Example

USAGE:
    $(basename "$0") [ENVIRONMENT] [DURATION_HOURS]

ARGUMENTS:
    ENVIRONMENT      Target environment (staging|production)
    DURATION_HOURS   PAT expiration time in hours (default: 1)

EXAMPLES:
    # Deploy to staging with 1-hour PAT
    $(basename "$0") staging 1

    # Deploy to production with 30-minute PAT
    $(basename "$0") production 0.5

    # Deploy to staging with default 1-hour PAT
    $(basename "$0") staging

DESCRIPTION:
    This script demonstrates how to use the GitHub PAT manager for secure
    CI/CD deployments. It creates a temporary PAT, uses it for deployment
    operations, and automatically cleans up expired tokens.

SECURITY:
    - PATs are created with minimal required scopes
    - Tokens are automatically cleaned up after use
    - No sensitive information is logged
    - Short expiration times limit exposure window

EOF
}

# Main function
main() {
    local environment="${1:-}"
    local duration_hours="${2:-1}"
    
    if [[ -z "$environment" ]]; then
        log_error "Environment argument is required"
        show_usage
        exit 1
    fi
    
    if [[ "$environment" != "staging" && "$environment" != "production" ]]; then
        log_error "Environment must be 'staging' or 'production'"
        show_usage
        exit 1
    fi
    
    # Validate duration
    if ! [[ "$duration_hours" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
        log_error "Duration must be a positive number"
        show_usage
        exit 1
    fi
    
    log_info "WasmWizard CI/CD Deployment Example"
    log_info "Environment: $environment"
    log_info "PAT Duration: $duration_hours hours"
    echo
    
    # Check prerequisites
    check_pat_manager
    
    # Execute deployment
    if deploy_with_pat "$environment" "$duration_hours"; then
        log_success "Deployment pipeline completed successfully"
        exit 0
    else
        log_error "Deployment pipeline failed"
        exit 1
    fi
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi