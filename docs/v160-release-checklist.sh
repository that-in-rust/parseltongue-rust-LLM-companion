#!/bin/bash
# Parseltongue v1.6.1 Release Checklist - Self-Analysis Dry Run
#
# This script performs a complete release verification by:
# 1. Running all tests
# 2. Building a release binary
# 3. Ingesting the Parseltongue codebase itself
# 4. Starting the HTTP server
# 5. Testing all 22 endpoints with validation
# 6. Generating a results report
#
# Usage: bash docs/v160-release-checklist.sh
# Requires: jq, curl

set -uo pipefail
# Note: -e intentionally omitted because grep returns exit 1 on no-match,
# which would kill the script under pipefail when checking for absence of errors.

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0
REPORT_FILE="docs/v161-release-dryrun-$(date +%Y%m%d%H%M%S).md"
PORT=7778  # Use non-default port to avoid conflicts

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    PASS_COUNT=$((PASS_COUNT + 1))
    echo "- [x] **PASS**: $1" >> "$REPORT_FILE"
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    FAIL_COUNT=$((FAIL_COUNT + 1))
    echo "- [ ] **FAIL**: $1" >> "$REPORT_FILE"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    WARN_COUNT=$((WARN_COUNT + 1))
    echo "- [~] **WARN**: $1" >> "$REPORT_FILE"
}

log_info() {
    echo -e "      $1"
}

# Initialize report
cat > "$REPORT_FILE" << HEADER
# Parseltongue v1.6.1 Release Dry Run Report

**Generated**: $(date)
**Branch**: $(git branch --show-current)
**Commit**: $(git rev-parse --short HEAD)

---

## Pre-Flight Checks

HEADER

echo "================================================"
echo "  Parseltongue v1.6.1 Release Checklist"
echo "  Self-Analysis Dry Run"
echo "================================================"
echo ""

# ============================================================
# PHASE 1: Pre-flight checks
# ============================================================
echo "--- Phase 1: Pre-flight checks ---"
echo "" >> "$REPORT_FILE"

# Check version in Cargo.toml
WORKSPACE_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
if [ "$WORKSPACE_VERSION" = "1.6.0" ]; then
    log_pass "Workspace version is 1.6.0 (Cargo.toml)"
else
    log_warn "Workspace version is $WORKSPACE_VERSION (Cargo.toml says 1.6.0, API says 1.6.1)"
fi

# Check for TODO/STUB in graph_analysis
TODO_COUNT=$(grep -rc "TODO\|STUB\|PLACEHOLDER" --include="*.rs" crates/parseltongue-core/src/graph_analysis/ 2>/dev/null | awk -F: '{s+=$2} END {print s+0}')
if [ "$TODO_COUNT" = "0" ]; then
    log_pass "No TODO/STUB/PLACEHOLDER in graph_analysis"
else
    log_warn "Found $TODO_COUNT TODO/STUB/PLACEHOLDER in graph_analysis"
fi

# Check clippy
echo "Running clippy..."
CLIPPY_OUTPUT=$(cargo clippy -p parseltongue-core -- -D warnings 2>&1 || true)
if echo "$CLIPPY_OUTPUT" | grep -q "^error"; then
    log_fail "Clippy warnings in parseltongue-core"
else
    log_pass "Clippy clean for parseltongue-core"
fi

# ============================================================
# PHASE 2: Test suite
# ============================================================
echo ""
echo "--- Phase 2: Test suite ---"
echo "" >> "$REPORT_FILE"
echo "## Test Suite" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# Run graph_analysis tests
echo "Running graph_analysis tests..."
GRAPH_TEST_OUTPUT=$(cargo test -p parseltongue-core -- graph_analysis 2>&1 || true)
GRAPH_TEST_COUNT=$(echo "$GRAPH_TEST_OUTPUT" | grep "^test result:" | head -1 | sed 's/.*ok\. \([0-9]*\) passed.*/\1/' || echo "0")
GRAPH_TEST_FAIL=$(echo "$GRAPH_TEST_OUTPUT" | grep "^test result:" | head -1 | sed 's/.*; \([0-9]*\) failed.*/\1/' || echo "0")

if [ "$GRAPH_TEST_FAIL" = "0" ]; then
    log_pass "Graph analysis: $GRAPH_TEST_COUNT tests passing"
else
    log_fail "Graph analysis: $GRAPH_TEST_FAIL tests failing"
fi

# Run pt08 tests
echo "Running pt08 HTTP server tests..."
PT08_OUTPUT=$(cargo test -p pt08-http-code-query-server 2>&1 || true)
if echo "$PT08_OUTPUT" | grep -q "test result: ok"; then
    log_pass "pt08 HTTP server tests passing"
else
    log_fail "pt08 HTTP server tests failing"
fi

# ============================================================
# PHASE 3: Release build
# ============================================================
echo ""
echo "--- Phase 3: Release build ---"
echo "" >> "$REPORT_FILE"
echo "## Release Build" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo "Building release binary..."
BUILD_OUTPUT=$(cargo build --release 2>&1 || true)
if echo "$BUILD_OUTPUT" | tail -1 | grep -q "Finished"; then
    log_pass "Release binary built successfully"
else
    log_fail "Release build failed"
    echo "Aborting - cannot continue without release binary"
    exit 1
fi

# ============================================================
# PHASE 4: Ingest self
# ============================================================
echo ""
echo "--- Phase 4: Self-ingestion ---"
echo "" >> "$REPORT_FILE"
echo "## Self-Ingestion" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo "Ingesting Parseltongue codebase..."
INGEST_OUTPUT=$(./target/release/parseltongue pt01-folder-to-cozodb-streamer . 2>&1 || true)
DB_PATH=$(echo "$INGEST_OUTPUT" | grep -o 'rocksdb:[^ ]*' | head -1 || echo "")
WORKSPACE_DIR=$(echo "$INGEST_OUTPUT" | grep -o 'parseltongue[0-9]*' | head -1 || echo "")

if [ -n "$DB_PATH" ]; then
    log_pass "Ingestion complete: $DB_PATH"
    log_info "Workspace: $WORKSPACE_DIR"
else
    log_fail "Ingestion failed - no database path found"
    echo "Aborting"
    exit 1
fi

# ============================================================
# PHASE 5: Start server
# ============================================================
echo ""
echo "--- Phase 5: Start HTTP server ---"
echo "" >> "$REPORT_FILE"
echo "## HTTP Server" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

echo "Starting HTTP server on port $PORT..."
./target/release/parseltongue pt08-http-code-query-server --db "$DB_PATH" --port $PORT &
SERVER_PID=$!
sleep 3  # Wait for server to start

# Verify server is running
if curl -s "http://localhost:$PORT/server-health-check-status" | jq -e '.success' > /dev/null 2>&1; then
    log_pass "HTTP server running on port $PORT (PID: $SERVER_PID)"
else
    log_fail "HTTP server failed to start"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

# Cleanup trap
trap "echo ''; echo 'Cleaning up...'; kill $SERVER_PID 2>/dev/null || true; rm -rf $WORKSPACE_DIR 2>/dev/null || true" EXIT

# ============================================================
# PHASE 6: Test all 22 endpoints
# ============================================================
echo ""
echo "--- Phase 6: Endpoint verification (22 endpoints) ---"
echo "" >> "$REPORT_FILE"
echo "## Endpoint Verification" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

BASE="http://localhost:$PORT"

# Helper: test endpoint returns success:true
test_endpoint() {
    local name="$1"
    local url="$2"
    local extra_check="${3:-}"

    RESPONSE=$(curl -s "$url" 2>/dev/null)
    SUCCESS=$(echo "$RESPONSE" | jq -r '.success' 2>/dev/null)

    if [ "$SUCCESS" = "true" ]; then
        if [ -n "$extra_check" ]; then
            if echo "$RESPONSE" | jq -e "$extra_check" > /dev/null 2>&1; then
                log_pass "$name"
            else
                log_warn "$name (success but extra check failed: $extra_check)"
                log_info "Response: $(echo "$RESPONSE" | jq -c '.' | head -c 200)"
            fi
        else
            log_pass "$name"
        fi
    else
        log_fail "$name"
        log_info "Response: $(echo "$RESPONSE" | head -c 200)"
    fi
}

echo "### Original 14 Endpoints (v1.4.x)"
echo "" >> "$REPORT_FILE"
echo "### Original 14 Endpoints" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

test_endpoint "01. /server-health-check-status" \
    "$BASE/server-health-check-status" \
    '.status == "ok"'

test_endpoint "02. /codebase-statistics-overview-summary" \
    "$BASE/codebase-statistics-overview-summary" \
    '.data.code_entities_total_count > 0'

test_endpoint "03. /api-reference-documentation-help" \
    "$BASE/api-reference-documentation-help" \
    '.data.total_endpoints >= 22'

test_endpoint "04. /code-entities-list-all" \
    "$BASE/code-entities-list-all" \
    '.data.entities | length > 0'

# Get entity keys for testing
# Entity from entity list (for detail-view)
ENTITY_KEY=$(curl -s "$BASE/code-entities-list-all" | jq -r '.data.entities[0].key' 2>/dev/null || echo "")
# Entity from edge to_key (a callee - guaranteed to have at least one caller)
EDGE_ENTITY=$(curl -s "$BASE/dependency-edges-list-all" | jq -r '.data.edges[0].to_key' 2>/dev/null || echo "")
# Entity from edge from_key (a caller - guaranteed to have at least one callee)
CALLER_ENTITY=$(curl -s "$BASE/dependency-edges-list-all" | jq -r '.data.edges[0].from_key' 2>/dev/null || echo "")
log_info "Entity (list):  ${ENTITY_KEY:0:80}..."
log_info "Entity (callee): ${EDGE_ENTITY:0:80}..."
log_info "Entity (caller): ${CALLER_ENTITY:0:80}..."

test_endpoint "05. /code-entity-detail-view" \
    "$BASE/code-entity-detail-view?key=$ENTITY_KEY"

test_endpoint "06. /code-entities-search-fuzzy" \
    "$BASE/code-entities-search-fuzzy?q=main" \
    '.data.entities | length >= 0'

test_endpoint "07. /dependency-edges-list-all" \
    "$BASE/dependency-edges-list-all" \
    '.data.edges | length > 0'

test_endpoint "08. /reverse-callers-query-graph" \
    "$BASE/reverse-callers-query-graph?entity=$EDGE_ENTITY"

test_endpoint "09. /forward-callees-query-graph" \
    "$BASE/forward-callees-query-graph?entity=$CALLER_ENTITY"

test_endpoint "10. /blast-radius-impact-analysis" \
    "$BASE/blast-radius-impact-analysis?entity=$EDGE_ENTITY&hops=2"

test_endpoint "11. /circular-dependency-detection-scan" \
    "$BASE/circular-dependency-detection-scan"

test_endpoint "12. /complexity-hotspots-ranking-view" \
    "$BASE/complexity-hotspots-ranking-view?top=5"

test_endpoint "13. /semantic-cluster-grouping-list" \
    "$BASE/semantic-cluster-grouping-list"

test_endpoint "14. /smart-context-token-budget" \
    "$BASE/smart-context-token-budget?focus=$CALLER_ENTITY&tokens=2000"

echo ""
echo "### v1.6.0 Graph Analysis Endpoints (7 new)"
echo "" >> "$REPORT_FILE"
echo "### v1.6.0 Graph Analysis Endpoints" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

test_endpoint "15. /strongly-connected-components-analysis" \
    "$BASE/strongly-connected-components-analysis" \
    '.data.scc_count >= 0'

test_endpoint "16. /technical-debt-sqale-scoring" \
    "$BASE/technical-debt-sqale-scoring" \
    '.data.entities | length >= 0'

test_endpoint "17. /kcore-decomposition-layering-analysis" \
    "$BASE/kcore-decomposition-layering-analysis" \
    '.data.entities | length >= 0'

test_endpoint "18a. /centrality-measures-entity-ranking (pagerank)" \
    "$BASE/centrality-measures-entity-ranking?method=pagerank" \
    '.data.method == "pagerank"'

test_endpoint "18b. /centrality-measures-entity-ranking (betweenness)" \
    "$BASE/centrality-measures-entity-ranking?method=betweenness" \
    '.data.method == "betweenness"'

test_endpoint "19. /entropy-complexity-measurement-scores" \
    "$BASE/entropy-complexity-measurement-scores" \
    '.data.entities | length >= 0'

test_endpoint "20. /coupling-cohesion-metrics-suite" \
    "$BASE/coupling-cohesion-metrics-suite" \
    '.data.entities | length >= 0'

test_endpoint "21. /leiden-community-detection-clusters" \
    "$BASE/leiden-community-detection-clusters" \
    '.data.community_count >= 1'

echo ""
echo "### v1.6.1 Coverage Endpoint (1 new)"
echo "" >> "$REPORT_FILE"
echo "### v1.6.1 Coverage Endpoint" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

test_endpoint "22. /ingestion-coverage-folder-report" \
    "$BASE/ingestion-coverage-folder-report" \
    '.data.summary.eligible_files > 0'

test_endpoint "22a. /ingestion-coverage-folder-report (depth=1)" \
    "$BASE/ingestion-coverage-folder-report?depth=1" \
    '.data.folders | length > 0'

# ============================================================
# PHASE 7: Deep validation of graph analysis results
# ============================================================
echo ""
echo "--- Phase 7: Deep validation ---"
echo "" >> "$REPORT_FILE"
echo "## Deep Validation" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"

# SCC: verify risk levels are valid
SCC_RISKS=$(curl -s "$BASE/strongly-connected-components-analysis" | jq -r '.data.sccs[].risk_level' 2>/dev/null | sort -u || echo "")
VALID_RISKS=true
for risk in $SCC_RISKS; do
    if [[ "$risk" != "NONE" && "$risk" != "MEDIUM" && "$risk" != "HIGH" ]]; then
        VALID_RISKS=false
    fi
done
if [ "$VALID_RISKS" = true ]; then
    log_pass "SCC risk levels are valid (NONE/MEDIUM/HIGH)"
else
    log_fail "SCC contains invalid risk levels: $SCC_RISKS"
fi

# K-Core: verify layers are valid
KCORE_LAYERS=$(curl -s "$BASE/kcore-decomposition-layering-analysis" | jq -r '.data.entities[].layer' 2>/dev/null | sort -u || echo "")
VALID_LAYERS=true
for layer in $KCORE_LAYERS; do
    if [[ "$layer" != "CORE" && "$layer" != "MID" && "$layer" != "PERIPHERAL" ]]; then
        VALID_LAYERS=false
    fi
done
if [ "$VALID_LAYERS" = true ]; then
    log_pass "K-Core layers are valid (CORE/MID/PERIPHERAL)"
else
    log_fail "K-Core contains invalid layers: $KCORE_LAYERS"
fi

# SQALE: verify violation types
SQALE_TYPES=$(curl -s "$BASE/technical-debt-sqale-scoring" | jq -r '[.data.entities[].violations[].type] | unique | .[]' 2>/dev/null || echo "")
if [ -z "$SQALE_TYPES" ]; then
    log_pass "SQALE: no violations (clean codebase or all below thresholds)"
else
    log_pass "SQALE violation types found: $SQALE_TYPES"
fi

# CK Metrics: verify health grades are valid
CK_GRADES=$(curl -s "$BASE/coupling-cohesion-metrics-suite" | jq -r '.data.entities[].health_grade' 2>/dev/null | sort -u || echo "")
VALID_GRADES=true
for grade in $CK_GRADES; do
    if [[ "$grade" != "A" && "$grade" != "B" && "$grade" != "C" && "$grade" != "D" && "$grade" != "F" ]]; then
        VALID_GRADES=false
    fi
done
if [ "$VALID_GRADES" = true ]; then
    log_pass "CK health grades are valid (A-F)"
else
    log_fail "CK contains invalid grades: $CK_GRADES"
fi

# Leiden: modularity should be a number
MODULARITY=$(curl -s "$BASE/leiden-community-detection-clusters" | jq -r '.data.modularity' 2>/dev/null || echo "")
if [[ "$MODULARITY" =~ ^-?[0-9]+\.?[0-9]*$ ]]; then
    log_pass "Leiden modularity is numeric: $MODULARITY"
else
    log_fail "Leiden modularity is not numeric: $MODULARITY"
fi

# Entropy: verify complexity levels
ENTROPY_LEVELS=$(curl -s "$BASE/entropy-complexity-measurement-scores" | jq -r '.data.entities[].complexity' 2>/dev/null | sort -u || echo "")
VALID_ENTROPY=true
for level in $ENTROPY_LEVELS; do
    if [[ "$level" != "LOW" && "$level" != "MODERATE" && "$level" != "HIGH" ]]; then
        VALID_ENTROPY=false
    fi
done
if [ "$VALID_ENTROPY" = true ]; then
    log_pass "Entropy complexity levels are valid (LOW/MODERATE/HIGH)"
else
    log_fail "Entropy contains invalid levels: $ENTROPY_LEVELS"
fi

# Coverage: verify parsed <= eligible <= total
COVERAGE_RESPONSE=$(curl -s "$BASE/ingestion-coverage-folder-report" 2>/dev/null)
COV_TOTAL=$(echo "$COVERAGE_RESPONSE" | jq -r '.data.summary.total_files' 2>/dev/null || echo "0")
COV_ELIGIBLE=$(echo "$COVERAGE_RESPONSE" | jq -r '.data.summary.eligible_files' 2>/dev/null || echo "0")
COV_PARSED=$(echo "$COVERAGE_RESPONSE" | jq -r '.data.summary.parsed_files' 2>/dev/null || echo "0")
if [ "$COV_PARSED" -le "$COV_ELIGIBLE" ] && [ "$COV_ELIGIBLE" -le "$COV_TOTAL" ]; then
    log_pass "Coverage invariant: parsed($COV_PARSED) <= eligible($COV_ELIGIBLE) <= total($COV_TOTAL)"
else
    log_fail "Coverage invariant violated: parsed=$COV_PARSED eligible=$COV_ELIGIBLE total=$COV_TOTAL"
fi

# Coverage: verify errors file exists
COV_ERRORS_FILE=$(echo "$COVERAGE_RESPONSE" | jq -r '.data.summary.errors_file' 2>/dev/null || echo "")
if [ -n "$COV_ERRORS_FILE" ] && [ -f "$COV_ERRORS_FILE" ]; then
    log_pass "ingestion-errors.txt written: $COV_ERRORS_FILE"
else
    log_warn "ingestion-errors.txt not found at: $COV_ERRORS_FILE"
fi

# ============================================================
# Summary
# ============================================================
echo ""
echo "================================================"
echo "  Release Checklist Summary"
echo "================================================"
echo ""
echo -e "  ${GREEN}PASS${NC}: $PASS_COUNT"
echo -e "  ${YELLOW}WARN${NC}: $WARN_COUNT"
echo -e "  ${RED}FAIL${NC}: $FAIL_COUNT"
echo ""

TOTAL=$((PASS_COUNT + FAIL_COUNT + WARN_COUNT))
echo "  Total checks: $TOTAL"

# Write summary to report
cat >> "$REPORT_FILE" << EOF

---

## Summary

| Result | Count |
|--------|-------|
| PASS | $PASS_COUNT |
| WARN | $WARN_COUNT |
| FAIL | $FAIL_COUNT |
| **Total** | **$TOTAL** |

**Verdict**: $([ "$FAIL_COUNT" -eq 0 ] && echo "RELEASE READY" || echo "BLOCKED - $FAIL_COUNT failures")

---

Generated by v160-release-checklist.sh on $(date)
EOF

if [ "$FAIL_COUNT" -eq 0 ]; then
    echo -e "  ${GREEN}VERDICT: RELEASE READY${NC}"
    echo ""
    echo "  Report saved to: $REPORT_FILE"
else
    echo -e "  ${RED}VERDICT: BLOCKED - $FAIL_COUNT failures${NC}"
    echo ""
    echo "  Report saved to: $REPORT_FILE"
    exit 1
fi
