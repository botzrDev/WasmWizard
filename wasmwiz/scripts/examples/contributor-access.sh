#!/bin/bash

# Example: Contributor Onboarding with Temporary PAT
# This script demonstrates how to provide temporary repository access to contributors

set -euo pipefail

# Configuration
# Declare SCRIPT_DIR separately to avoid masking return values
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
readonly PAT_MANAGER="${SCRIPT_DIR}/../github-pat-manager.sh"
readonly CONTRIBUTOR_LOG="${SCRIPT_DIR}/contributor-access.log"

# Colors for output
readonly GREEN='\033[0;32m'
readonly RED='\033[0;31m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] INFO: $1" >> "${CONTRIBUTOR_LOG}"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $1" >> "${CONTRIBUTOR_LOG}"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $1" >> "${CONTRIBUTOR_LOG}"
}

# Generate contributor access PAT
generate_contributor_pat() {
    local contributor_name="$1"
    local duration_hours="$2"
    local access_type="$3"
    local repositories="${4:-botzrDev/WasmWizard}"
    
    log_info "Generating contributor PAT for: $contributor_name"
    log_info "Access type: $access_type"
    log_info "Duration: $duration_hours hours"
    log_info "Repositories: $repositories"
    
    # Define scopes based on access type
    local scopes
    case "$access_type" in
        "read-only")
            scopes="repo:status,repo_deployment,public_repo"
            ;;
        "contributor")
            scopes="repo"
            ;;
        "reviewer")
            scopes="repo,read:org"
            ;;
        "maintainer")
            scopes="repo,read:org,write:discussion"
            ;;
        *)
            log_error "Unknown access type: $access_type"
            return 1
            ;;
    esac
    
    local description="Contributor Access: $contributor_name - $access_type"
    
    log_info "Creating PAT with scopes: $scopes"
    local pat_token
    if pat_token=$("$PAT_MANAGER" create \
        -d "$description" \
        -s "$scopes" \
        -e "$duration_hours" \
        -r "$repositories" 2>/dev/null); then
        
        log_success "PAT created successfully for $contributor_name"
        
        # Generate instructions for the contributor
        generate_contributor_instructions "$contributor_name" "$access_type" "$duration_hours" "$repositories" "$pat_token"
        
        return 0
    else
        log_error "Failed to create PAT for $contributor_name"
        return 1
    fi
}

# Generate contributor instructions
generate_contributor_instructions() {
    local contributor_name="$1"
    local access_type="$2"
    local duration_hours="$3"
    local repositories="$4"
    local pat_token="$5"
    
    # Declare instructions_file separately to avoid masking return values
    local instructions_file
    instructions_file="${SCRIPT_DIR}/contributor-instructions-$(date '+%Y%m%d-%H%M%S').txt"
    
    cat << EOF > "$instructions_file"
========================================
WasmWizard Contributor Access Instructions
========================================

Hello $contributor_name,

You have been granted temporary access to WasmWizard repository(ies).

ACCESS DETAILS:
- Access Type: $access_type
- Duration: $duration_hours hours
- Repositories: $repositories
- Created: $(date '+%Y-%m-%d %H:%M:%S UTC')
- Expires: $(date -d "+$duration_hours hours" '+%Y-%m-%d %H:%M:%S UTC')

SETUP INSTRUCTIONS:

1. Save your Personal Access Token:
   
   Export this token as an environment variable:
   export GITHUB_TOKEN="$pat_token"
   
   IMPORTANT: This token will only be shown once. Save it securely!

2. Clone the repository:
   
   git clone https://x-access-token:\$GITHUB_TOKEN@github.com/botzrDev/WasmWizard.git
   cd WasmWizard

3. Configure Git (if not already done):
   
   git config user.name "Your Name"
   git config user.email "your.email@example.com"

WHAT YOU CAN DO:

Read-Only Access:
- Clone and pull repositories
- View issues and pull requests
- Read repository content and history

Contributor Access:
- All read-only permissions
- Create branches and push commits
- Create pull requests
- Comment on issues and pull requests

Reviewer Access:
- All contributor permissions
- Review pull requests
- View organization information
- Access to team discussions

Maintainer Access:
- All reviewer permissions
- Write to discussions
- Advanced repository management

SECURITY NOTES:

- This token expires in $duration_hours hours
- Never share this token with others
- Do not commit this token to any repository
- Use the token only for authorized WasmWizard work
- Contact the team if you need extended access

GETTING STARTED:

1. Review the Contributing Guidelines:
   https://github.com/botzrDev/WasmWizard/blob/master/CONTRIBUTING.md

2. Check out open issues:
   https://github.com/botzrDev/WasmWizard/issues

3. Join our community discussions:
   https://github.com/botzrDev/WasmWizard/discussions

SUPPORT:

If you need help or have questions:
- Open an issue: https://github.com/botzrDev/WasmWizard/issues
- Start a discussion: https://github.com/botzrDev/WasmWizard/discussions
- Contact the maintainers

Thank you for contributing to WasmWizard!

========================================
Generated on $(date '+%Y-%m-%d %H:%M:%S UTC')
========================================
EOF

    log_success "Instructions generated: $instructions_file"
    echo
    echo "📋 Contributor instructions have been saved to:"
    echo "   $instructions_file"
    echo
    echo "🔒 SECURITY REMINDER:"
    echo "   - The PAT is included in the instructions file"
    echo "   - Share this file securely with the contributor"
    echo "   - Delete the file after the contributor has saved the token"
    echo
}

# List active contributor PATs
list_contributor_pats() {
    log_info "Listing active contributor PATs..."
    
    echo "🔍 Active Contributor PATs:"
    echo "=========================="
    
    "$PAT_MANAGER" list | grep -i "contributor\|access" || {
        echo "No active contributor PATs found"
    }
}

# Revoke contributor access
revoke_contributor_access() {
    local token_id="$1"
    
    log_info "Revoking contributor access for token: $token_id"
    
    if "$PAT_MANAGER" revoke -i "$token_id"; then
        log_success "Contributor access revoked successfully"
    else
        log_error "Failed to revoke contributor access"
        return 1
    fi
}

# Show usage information
show_usage() {
    cat << EOF
WasmWizard Contributor Access Manager

USAGE:
    $(basename "$0") COMMAND [OPTIONS]

COMMANDS:
    grant       Grant temporary access to a contributor
    list        List active contributor PATs
    revoke      Revoke contributor access
    help        Show this help message

GRANT OPTIONS:
    -n, --name NAME           Contributor name (required)
    -t, --type TYPE          Access type (required)
    -d, --duration HOURS     Duration in hours (default: 8)
    -r, --repos REPOS        Repository list (default: botzrDev/WasmWizard)

ACCESS TYPES:
    read-only    Read-only access to repositories
    contributor  Standard contributor access (read/write)
    reviewer     Reviewer access (includes PR reviews)
    maintainer   Maintainer access (advanced permissions)

REVOKE OPTIONS:
    -i, --id TOKEN_ID        Token ID to revoke (required)

EXAMPLES:
    # Grant 8-hour contributor access
    $(basename "$0") grant -n "john.doe" -t "contributor" -d 8

    # Grant 24-hour reviewer access
    $(basename "$0") grant -n "jane.smith" -t "reviewer" -d 24

    # Grant read-only access for 4 hours
    $(basename "$0") grant -n "new.contributor" -t "read-only" -d 4

    # List active contributor PATs
    $(basename "$0") list

    # Revoke specific contributor access
    $(basename "$0") revoke -i 12345

SECURITY:
    - All PATs have automatic expiration
    - Access is limited to specified repositories
    - Comprehensive audit logging
    - Secure instruction generation

EOF
}

# Main function
main() {
    local command="${1:-help}"
    
    case "$command" in
        grant)
            shift
            local contributor_name=""
            local access_type=""
            local duration_hours="8"
            local repositories="botzrDev/WasmWizard"
            
            while [[ $# -gt 0 ]]; do
                case $1 in
                    -n|--name)
                        contributor_name="$2"
                        shift 2
                        ;;
                    -t|--type)
                        access_type="$2"
                        shift 2
                        ;;
                    -d|--duration)
                        duration_hours="$2"
                        shift 2
                        ;;
                    -r|--repos)
                        repositories="$2"
                        shift 2
                        ;;
                    *)
                        log_error "Unknown option: $1"
                        show_usage
                        exit 1
                        ;;
                esac
            done
            
            if [[ -z "$contributor_name" ]]; then
                log_error "Contributor name is required"
                show_usage
                exit 1
            fi
            
            if [[ -z "$access_type" ]]; then
                log_error "Access type is required"
                show_usage
                exit 1
            fi
            
            generate_contributor_pat "$contributor_name" "$duration_hours" "$access_type" "$repositories"
            ;;
        list)
            list_contributor_pats
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
                log_error "Token ID is required"
                show_usage
                exit 1
            fi
            
            revoke_contributor_access "$token_id"
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