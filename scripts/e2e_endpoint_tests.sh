#!/bin/bash
# Parseltongue E2E Endpoint Tests
# Version: 1.4.1
# Date: 2026-01-29
#
# Usage: ./scripts/e2e_endpoint_tests.sh [database_path]
# Example: ./scripts/e2e_endpoint_tests.sh parseltongue20260128124417/analysis.db
#
# Prerequisites:
# - Built binary: cargo build --release
# - curl installed
# - jq installed (optional, for JSON formatting)
#
# This script tests all 16 HTTP endpoints and reports pass/fail status.

set -e

# Configuration
BASE_URL="${PARSELTONGUE_URL:-http://localhost:7777}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DB_PATH="${1:-parseltongue20260128124417/analysis.db}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

# Helper functions
print_header() {
    echo ""
    echo "=============================================="
    echo "  $1"
    echo "=============================================="
    echo ""
}

print_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASS_COUNT++))
}

print_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAIL_COUNT++))
}

print_skip() {
    echo -e "${YELLOW}[SKIP]${NC} $1"
    ((SKIP_COUNT++))
}

print_info() {
    echo -e "      $1"
}

# Test function: checks if response contains expected string
test_endpoint() {
    local name="$1"
    local endpoint="$2"
    local method="${3:-GET}"
    local expected="${4:-success}"
    local data="$5"

    local response
    local http_code

    if [ "$method" = "POST" ]; then
        if [ -n "$data" ]; then
            response=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}${endpoint}" \
                -H "Content-Type: application/json" \
                -d "$data" 2>/dev/null)
        else
            response=$(curl -s -w "\n%{http_code}" -X POST "${BASE_URL}${endpoint}" 2>/dev/null)
        fi
    else
        response=$(curl -s -w "\n%{http_code}" "${BASE_URL}${endpoint}" 2>/dev/null)
    fi

    http_code=$(echo "$response" | tail -1)
    body=$(echo "$response" | sed '$d')

    if [ "$http_code" = "200" ] && echo "$body" | grep -q "$expected"; then
        print_pass "$name"
        return 0
    else
        print_fail "$name (HTTP $http_code)"
        print_info "Response: $(echo "$body" | head -c 100)..."
        return 1
    fi
}

# Wait for server to be ready
wait_for_server() {
    local max_attempts=30
    local attempt=0

    echo "Waiting for server at $BASE_URL..."
    while [ $attempt -lt $max_attempts ]; do
        if curl -s "${BASE_URL}/server-health-check-status" > /dev/null 2>&1; then
            echo "Server is ready!"
            return 0
        fi
        ((attempt++))
        sleep 1
    done

    echo "Server did not start within ${max_attempts} seconds"
    return 1
}

# Main test execution
main() {
    print_header "Parseltongue E2E Endpoint Tests"

    echo "Configuration:"
    echo "  Base URL: $BASE_URL"
    echo "  Database: $DB_PATH"
    echo ""

    # Wait for server
    if ! wait_for_server; then
        echo "ERROR: Server not available. Please start the server first:"
        echo "  ./target/release/parseltongue pt08-http-code-query-server --db \"rocksdb:$DB_PATH\""
        exit 1
    fi

    print_header "Phase 1: Core Endpoints"

    test_endpoint "1. Health Check" "/server-health-check-status" "GET" "success"
    test_endpoint "2. Statistics Overview" "/codebase-statistics-overview-summary" "GET" "success"
    test_endpoint "3. API Documentation" "/api-reference-documentation-help" "GET" "success"

    print_header "Phase 2: Entity Endpoints"

    test_endpoint "4. List All Entities" "/code-entities-list-all" "GET" "success"
    test_endpoint "5. Entity Detail View" "/code-entity-detail-view?key=rust:fn:main" "GET" "success"
    test_endpoint "6. Fuzzy Search" "/code-entities-search-fuzzy?q=handle" "GET" "success"

    print_header "Phase 3: Graph Query Endpoints"

    test_endpoint "7. Reverse Callers" "/reverse-callers-query-graph?entity=rust:fn:main" "GET" "success"
    test_endpoint "8. Forward Callees" "/forward-callees-query-graph?entity=rust:fn:main" "GET" "success"
    test_endpoint "9. Dependency Edges" "/dependency-edges-list-all" "GET" "success"

    print_header "Phase 4: Analysis Endpoints"

    test_endpoint "10. Blast Radius" "/blast-radius-impact-analysis?entity=rust:fn:main&hops=2" "GET" "success"
    test_endpoint "11. Circular Dependencies" "/circular-dependency-detection-scan" "GET" "success"
    test_endpoint "12. Complexity Hotspots" "/complexity-hotspots-ranking-view?top=5" "GET" "success"
    test_endpoint "13. Semantic Clusters" "/semantic-cluster-grouping-list" "GET" "success"

    print_header "Phase 5: Advanced Endpoints"

    test_endpoint "14. Smart Context" "/smart-context-token-budget?focus=rust:fn:main&tokens=4000" "GET" "success"
    test_endpoint "15. File Watcher Status" "/file-watcher-status-check" "GET" "success"

    print_header "Phase 6: Incremental Reindex (POST)"

    # Test with a real file if it exists
    TEST_FILE="${PROJECT_ROOT}/crates/parseltongue-core/src/lib.rs"
    if [ -f "$TEST_FILE" ]; then
        test_endpoint "16. Incremental Reindex" "/incremental-reindex-file-update?path=${TEST_FILE}" "POST" "success"
    else
        print_skip "16. Incremental Reindex (test file not found)"
    fi

    print_header "Phase 7: Error Handling Tests"

    # Test error conditions
    echo "Testing error conditions..."

    # Missing parameter
    response=$(curl -s "${BASE_URL}/code-entity-detail-view" 2>/dev/null)
    if echo "$response" | grep -q "missing"; then
        print_pass "Missing parameter error handling"
    else
        print_fail "Missing parameter error handling"
    fi

    # File not found
    response=$(curl -s -X POST "${BASE_URL}/incremental-reindex-file-update?path=/nonexistent/file.rs" 2>/dev/null)
    if echo "$response" | grep -q "not found"; then
        print_pass "File not found error handling"
    else
        print_fail "File not found error handling"
    fi

    print_header "Test Summary"

    echo -e "${GREEN}Passed:${NC} $PASS_COUNT"
    echo -e "${RED}Failed:${NC} $FAIL_COUNT"
    echo -e "${YELLOW}Skipped:${NC} $SKIP_COUNT"
    echo ""

    TOTAL=$((PASS_COUNT + FAIL_COUNT))
    if [ $FAIL_COUNT -eq 0 ]; then
        echo -e "${GREEN}All tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}Some tests failed.${NC}"
        exit 1
    fi
}

# Run main
main "$@"
