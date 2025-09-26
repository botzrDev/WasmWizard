#!/bin/bash

# GitHub Personal Access Token (PAT) Manager for WasmWizard
# Securely creates, uses, and revokes temporary GitHub PATs
# Version: 1.0.0

set -euo pipefail

# Configuration with sensible defaults
readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly LOG_FILE="${LOG_FILE:-${SCRIPT_DIR}/pat-manager.log}"
readonly PAT_STORE_FILE="${PAT_STORE_FILE:-/tmp/.wasmwiz-pats.json}"
readonly DEFAULT_EXPIRATION_HOURS="${PAT_EXPIRATION_HOURS:-24}"
readonly DEFAULT_SCOPES="${PAT_SCOPES:-repo,read:org}"

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Logging functions (NEVER log sensitive tokens)
log_info() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${BLUE}[INFO]${NC} ${message}"
    echo "[${timestamp}] INFO: ${message}" >> "${LOG_FILE}"
}

log_warn() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${YELLOW}[WARN]${NC} ${message}" >&2
    echo "[${timestamp}] WARN: ${message}" >> "${LOG_FILE}"
}

log_error() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${RED}[ERROR]${NC} ${message}" >&2
    echo "[${timestamp}] ERROR: ${message}" >> "${LOG_FILE}"
}

log_success() {
    local message="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${GREEN}[SUCCESS]${NC} ${message}"
    echo "[${timestamp}] SUCCESS: ${message}" >> "${LOG_FILE}"
}

# Check dependencies
check_dependencies() {
    local missing_deps=()
    
    if ! command -v gh &> /dev/null; then
        missing_deps+=("gh (GitHub CLI)")
    fi
    
    if ! command -v jq &> /dev/null; then
        missing_deps+=("jq")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        echo "Please install the missing dependencies:"
        echo "  - GitHub CLI: https://cli.github.com/"
        echo "  - jq: https://stedolan.github.io/jq/"
        exit 1
    fi
}

# Validate GitHub authentication
check_auth() {
    if ! gh auth status &> /dev/null; then
        log_error "GitHub CLI not authenticated. Please run 'gh auth login' first."
        exit 1
    fi
    
    local user
    user=$(gh api user --jq '.login')
    log_info "Authenticated as GitHub user: ${user}"
}

# Calculate expiration date
calculate_expiration() {
    local hours="${1:-$DEFAULT_EXPIRATION_HOURS}"
    if command -v gdate &> /dev/null; then
        # macOS with GNU date (brew install coreutils)
        gdate -d "+${hours} hours" -u '+%Y-%m-%dT%H:%M:%SZ'
    else
        # Linux with GNU date
        date -d "+${hours} hours" -u '+%Y-%m-%dT%H:%M:%SZ'
    fi
}

# Create a new GitHub PAT
create_pat() {
    local description="$1"
    local scopes="${2:-$DEFAULT_SCOPES}"
    local expiration_hours="${3:-$DEFAULT_EXPIRATION_HOURS}"
    local repo_list="${4:-}"
    
    log_info "Creating GitHub PAT: ${description}"
    log_info "Scopes: ${scopes}"
    log_info "Expiration: ${expiration_hours} hours"
    
    local expiration_date
    if ! expiration_date=$(calculate_expiration "${expiration_hours}"); then
        log_error "Failed to calculate expiration date"
        return 1
    fi
    
    # Create the PAT using GitHub CLI
    local gh_cmd="gh auth refresh --scopes '${scopes}'"
    local pat_data
    
    # Build the gh api command for creating PAT
    local create_cmd="gh api user/tokens"
    local json_payload
    
    json_payload=$(jq -n \
        --arg description "$description" \
        --arg expires_at "$expiration_date" \
        --argjson scopes "$(echo "$scopes" | tr ',' '\n' | jq -R . | jq -s .)" \
        '{
            description: $description,
            scopes: $scopes,
            expires_at: $expires_at
        }')
    
    # Add repository restrictions if specified
    if [[ -n "$repo_list" ]]; then
        local repositories
        repositories=$(echo "$repo_list" | tr ',' '\n' | jq -R . | jq -s .)
        json_payload=$(echo "$json_payload" | jq --argjson repos "$repositories" '. + {repositories: $repos}')
    fi
    
    log_info "Creating PAT with GitHub API..."
    if pat_data=$(echo "$json_payload" | gh api user/tokens --input - 2>/dev/null); then
        local token_id token_value
        token_id=$(echo "$pat_data" | jq -r '.id')
        token_value=$(echo "$pat_data" | jq -r '.token')
        
        # Store PAT metadata (NEVER store the actual token value in logs!)
        store_pat_metadata "$token_id" "$description" "$expiration_date"
        
        log_success "PAT created successfully"
        log_info "Token ID: ${token_id}"
        log_info "Expires: ${expiration_date}"
        
        # Export the token for use (caller should handle securely)
        echo "$token_value"
        return 0
    else
        log_error "Failed to create GitHub PAT"
        return 1
    fi
}

# Store PAT metadata (not the token itself!)
store_pat_metadata() {
    local token_id="$1"
    local description="$2"
    local expiration="$3"
    local timestamp=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
    
    # Create or update PAT store file
    local store_data="{}"
    if [[ -f "$PAT_STORE_FILE" ]]; then
        store_data=$(cat "$PAT_STORE_FILE")
    fi
    
    local updated_data
    updated_data=$(echo "$store_data" | jq \
        --arg id "$token_id" \
        --arg desc "$description" \
        --arg exp "$expiration" \
        --arg created "$timestamp" \
        '.[$id] = {
            description: $desc,
            created_at: $created,
            expires_at: $exp,
            status: "active"
        }')
    
    echo "$updated_data" > "$PAT_STORE_FILE"
    chmod 600 "$PAT_STORE_FILE"
}

# List active PATs
list_pats() {
    log_info "Listing active GitHub PATs..."
    
    if [[ ! -f "$PAT_STORE_FILE" ]]; then
        log_info "No stored PAT metadata found"
        return 0
    fi
    
    local current_time
    current_time=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
    
    echo "Active PATs:"
    echo "============"
    
    jq -r --arg current "$current_time" '
        to_entries[] | 
        select(.value.status == "active" and .value.expires_at > $current) | 
        "ID: \(.key)\nDescription: \(.value.description)\nCreated: \(.value.created_at)\nExpires: \(.value.expires_at)\n"
    ' "$PAT_STORE_FILE" || log_warn "No active PATs found"
}

# Revoke a PAT
revoke_pat() {
    local token_id="$1"
    
    log_info "Revoking GitHub PAT: ${token_id}"
    
    if gh api "user/tokens/${token_id}" --method DELETE &> /dev/null; then
        # Update metadata
        if [[ -f "$PAT_STORE_FILE" ]]; then
            local updated_data
            updated_data=$(jq --arg id "$token_id" '.[$id].status = "revoked"' "$PAT_STORE_FILE")
            echo "$updated_data" > "$PAT_STORE_FILE"
        fi
        
        log_success "PAT revoked successfully: ${token_id}"
    else
        log_error "Failed to revoke PAT: ${token_id}"
        return 1
    fi
}

# Cleanup expired PATs
cleanup_expired() {
    log_info "Cleaning up expired PATs..."
    
    if [[ ! -f "$PAT_STORE_FILE" ]]; then
        log_info "No PAT metadata to clean up"
        return 0
    fi
    
    local current_time
    current_time=$(date -u '+%Y-%m-%dT%H:%M:%SZ')
    
    local expired_ids
    expired_ids=$(jq -r --arg current "$current_time" '
        to_entries[] | 
        select(.value.status == "active" and .value.expires_at <= $current) | 
        .key
    ' "$PAT_STORE_FILE")
    
    if [[ -n "$expired_ids" ]]; then
        while IFS= read -r token_id; do
            log_info "Found expired PAT: ${token_id}"
            revoke_pat "$token_id"
        done <<< "$expired_ids"
    else
        log_info "No expired PATs found"
    fi
}

# Usage information
show_usage() {
    cat << EOF
GitHub PAT Manager for WasmWizard

USAGE:
    $(basename "$0") [COMMAND] [OPTIONS]

COMMANDS:
    create      Create a new temporary PAT
    list        List active PATs
    revoke      Revoke a specific PAT
    cleanup     Clean up expired PATs
    help        Show this help message

CREATE OPTIONS:
    -d, --description DESCRIPTION    Description for the PAT (required)
    -s, --scopes SCOPES             Comma-separated list of scopes (default: repo,read:org)
    -e, --expiration HOURS          Expiration time in hours (default: 24)
    -r, --repos REPOS              Comma-separated list of repositories (optional)

REVOKE OPTIONS:
    -i, --id TOKEN_ID              Token ID to revoke (required)

ENVIRONMENT VARIABLES:
    PAT_EXPIRATION_HOURS           Default expiration in hours (default: 24)
    PAT_SCOPES                     Default scopes (default: repo,read:org)
    LOG_FILE                       Log file path (default: ./pat-manager.log)
    PAT_STORE_FILE                 PAT metadata store (default: /tmp/.wasmwiz-pats.json)

EXAMPLES:
    # Create a PAT for CI/CD deployment
    $(basename "$0") create -d "WasmWizard CI/CD Deploy" -s "repo,read:org" -e 2

    # Create a PAT for contributor access
    $(basename "$0") create -d "Contributor Access" -s "repo" -e 8 -r "botzrDev/WasmWizard"

    # List active PATs
    $(basename "$0") list

    # Revoke a specific PAT
    $(basename "$0") revoke -i 12345

    # Clean up expired PATs
    $(basename "$0") cleanup

SECURITY NOTES:
    - PAT values are never logged or stored persistently
    - Use secure environment variables to pass tokens to other processes
    - Always revoke PATs when no longer needed
    - Store PAT metadata is created with 600 permissions for security

EOF
}

# Main function
main() {
    local command="${1:-help}"
    
    case "$command" in
        create)
            shift
            local description=""
            local scopes="$DEFAULT_SCOPES"
            local expiration_hours="$DEFAULT_EXPIRATION_HOURS"
            local repos=""
            
            while [[ $# -gt 0 ]]; do
                case $1 in
                    -d|--description)
                        description="$2"
                        shift 2
                        ;;
                    -s|--scopes)
                        scopes="$2"
                        shift 2
                        ;;
                    -e|--expiration)
                        expiration_hours="$2"
                        shift 2
                        ;;
                    -r|--repos)
                        repos="$2"
                        shift 2
                        ;;
                    *)
                        log_error "Unknown option: $1"
                        show_usage
                        exit 1
                        ;;
                esac
            done
            
            if [[ -z "$description" ]]; then
                log_error "Description is required for creating a PAT"
                show_usage
                exit 1
            fi
            
            check_dependencies
            check_auth
            create_pat "$description" "$scopes" "$expiration_hours" "$repos"
            ;;
        list)
            check_dependencies
            check_auth
            list_pats
            ;;
        revoke)
            shift
            local token_id=""
            
            while [[ $# -gt 0 ]]; do
                case $1 in
                    -i|--id)
                        token_id="$2"
                        shift 2
                        ;;
                    *)
                        log_error "Unknown option: $1"
                        show_usage
                        exit 1
                        ;;
                esac
            done
            
            if [[ -z "$token_id" ]]; then
                log_error "Token ID is required for revoking a PAT"
                show_usage
                exit 1
            fi
            
            check_dependencies
            check_auth
            revoke_pat "$token_id"
            ;;
        cleanup)
            check_dependencies
            check_auth
            cleanup_expired
            ;;
        help|--help|-h)
            show_usage
            ;;
        *)
            log_error "Unknown command: $command"
            show_usage
            exit 1
            ;;
    esac
}

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi