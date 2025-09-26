#!/bin/bash

# Validation script for GitHub PAT Manager
# Tests basic functionality without actually creating GitHub PATs

set -euo pipefail

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PAT_MANAGER="${SCRIPT_DIR}/github-pat-manager.sh"

# Colors
readonly GREEN='\033[0;32m'
readonly RED='\033[0;31m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m'

test_passed=0
test_failed=0

print_test_header() {
    echo
    echo -e "${BLUE}===========================================${NC}"
    echo -e "${BLUE}  GitHub PAT Manager Validation Tests${NC}"
    echo -e "${BLUE}===========================================${NC}"
    echo
}

print_test_result() {
    local test_name="$1"
    local result="$2"
    
    if [[ "$result" == "PASS" ]]; then
        echo -e "✅ ${GREEN}PASS${NC}: $test_name"
        ((test_passed++))
    else
        echo -e "❌ ${RED}FAIL${NC}: $test_name"
        ((test_failed++))
    fi
}

print_test_summary() {
    echo
    echo -e "${BLUE}===========================================${NC}"
    echo -e "${BLUE}  Test Summary${NC}"
    echo -e "${BLUE}===========================================${NC}"
    echo -e "✅ Tests Passed: ${GREEN}$test_passed${NC}"
    echo -e "❌ Tests Failed: ${RED}$test_failed${NC}"
    echo -e "📊 Total Tests:  $((test_passed + test_failed))"
    echo
    
    if [[ $test_failed -eq 0 ]]; then
        echo -e "${GREEN}🎉 All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}❌ Some tests failed.${NC}"
        return 1
    fi
}

# Test 1: Script exists and is executable
test_script_exists() {
    if [[ -f "$PAT_MANAGER" && -x "$PAT_MANAGER" ]]; then
        print_test_result "Script exists and is executable" "PASS"
    else
        print_test_result "Script exists and is executable" "FAIL"
    fi
}

# Test 2: Script shows help message
test_help_message() {
    if "$PAT_MANAGER" help > /dev/null 2>&1; then
        print_test_result "Help message displays correctly" "PASS"
    else
        print_test_result "Help message displays correctly" "FAIL"
    fi
}

# Test 3: Script validates dependencies (expect failure in CI)
test_dependency_validation() {
    # This test expects to fail in environments without gh/jq
    if "$PAT_MANAGER" list > /dev/null 2>&1; then
        print_test_result "Dependency validation (unexpected success)" "FAIL"
    else
        # Expected to fail without dependencies
        print_test_result "Dependency validation (expected failure)" "PASS"
    fi
}

# Test 4: Script handles invalid commands gracefully
test_invalid_command() {
    if "$PAT_MANAGER" invalid_command > /dev/null 2>&1; then
        print_test_result "Invalid command handling" "FAIL"
    else
        print_test_result "Invalid command handling" "PASS"
    fi
}

# Test 5: Script syntax validation
test_script_syntax() {
    if bash -n "$PAT_MANAGER" > /dev/null 2>&1; then
        print_test_result "Script syntax validation" "PASS"
    else
        print_test_result "Script syntax validation" "FAIL"
    fi
}

# Test 6: Required functions exist
test_functions_exist() {
    local functions=("main" "create_pat" "list_pats" "revoke_pat" "cleanup_expired" "show_usage")
    local all_functions_exist=true
    
    for func in "${functions[@]}"; do
        if ! grep -q "^${func}()" "$PAT_MANAGER"; then
            all_functions_exist=false
            break
        fi
    done
    
    if [[ "$all_functions_exist" == true ]]; then
        print_test_result "Required functions exist" "PASS"
    else
        print_test_result "Required functions exist" "FAIL"
    fi
}

# Test 7: No hardcoded secrets
test_no_hardcoded_secrets() {
    local sensitive_patterns=("ghp_" "github_pat_" "token.*=" "password.*=")
    local secrets_found=false
    
    for pattern in "${sensitive_patterns[@]}"; do
        if grep -i "$pattern" "$PAT_MANAGER" > /dev/null 2>&1; then
            secrets_found=true
            break
        fi
    done
    
    if [[ "$secrets_found" == false ]]; then
        print_test_result "No hardcoded secrets detected" "PASS"
    else
        print_test_result "No hardcoded secrets detected" "FAIL"
    fi
}

# Test 8: Security logging check
test_secure_logging() {
    # Check that tokens are not logged
    if grep -n "echo.*token\|printf.*token\|log.*token" "$PAT_MANAGER" | grep -v "log_info\|log_warn\|log_error\|Token ID" > /dev/null 2>&1; then
        print_test_result "Secure logging (no token exposure)" "FAIL"
    else
        print_test_result "Secure logging (no token exposure)" "PASS"
    fi
}

# Test 9: Example scripts exist
test_example_scripts() {
    local examples_dir="${SCRIPT_DIR}/examples"
    local expected_examples=("deployment-example.sh" "contributor-access.sh")
    local all_examples_exist=true
    
    for example in "${expected_examples[@]}"; do
        if [[ ! -f "$examples_dir/$example" ]]; then
            all_examples_exist=false
            break
        fi
    done
    
    if [[ "$all_examples_exist" == true ]]; then
        print_test_result "Example scripts exist" "PASS"
    else
        print_test_result "Example scripts exist" "FAIL"
    fi
}

# Test 10: Workflow file exists
test_workflow_exists() {
    local workflow_file="${SCRIPT_DIR}/../../.github/workflows/pat-automation.yml"
    
    if [[ -f "$workflow_file" ]]; then
        print_test_result "PAT automation workflow exists" "PASS"
    else
        print_test_result "PAT automation workflow exists" "FAIL"
    fi
}

# Main test execution
run_tests() {
    print_test_header
    
    echo -e "${YELLOW}Running validation tests...${NC}"
    echo
    
    test_script_exists
    test_help_message
    test_dependency_validation
    test_invalid_command
    test_script_syntax
    test_functions_exist
    test_no_hardcoded_secrets
    test_secure_logging
    test_example_scripts
    test_workflow_exists
    
    print_test_summary
}

# Show validation info
show_validation_info() {
    echo -e "${BLUE}GitHub PAT Manager Validation${NC}"
    echo
    echo "This validation script tests the PAT manager functionality without"
    echo "actually creating GitHub PATs or requiring authentication."
    echo
    echo "Tests include:"
    echo "  - Script accessibility and permissions"
    echo "  - Syntax and structure validation"
    echo "  - Security checks (no hardcoded secrets)"
    echo "  - Function completeness"
    echo "  - Example script availability"
    echo "  - Workflow configuration"
    echo
    echo "Note: Some tests expect failure in environments without"
    echo "GitHub CLI and jq dependencies."
    echo
}

# Main function
main() {
    if [[ "${1:-}" == "--info" ]]; then
        show_validation_info
        exit 0
    fi
    
    run_tests
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi