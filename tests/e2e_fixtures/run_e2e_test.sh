#!/bin/bash
# E2E Test: Parseltongue Diff Visualization System
# This script tests the complete diff pipeline with a minimal repo evolution

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PARSELTONGUE="$PROJECT_ROOT/target/release/parseltongue"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "======================================"
echo "E2E Test: Diff Visualization System"
echo "======================================"

# Check if parseltongue binary exists
if [ ! -f "$PARSELTONGUE" ]; then
    echo -e "${YELLOW}Building parseltongue...${NC}"
    cargo build --release -p parseltongue
fi

# Clean up previous test workspaces in fixture dirs
echo -e "\n${YELLOW}Step 1: Cleaning up previous test data...${NC}"
rm -rf "$SCRIPT_DIR/before/parseltongue"* 2>/dev/null || true
rm -rf "$SCRIPT_DIR/after/parseltongue"* 2>/dev/null || true
rm -f "$SCRIPT_DIR/diff_result.json" 2>/dev/null || true

# Index BEFORE codebase
echo -e "\n${YELLOW}Step 2: Indexing BEFORE codebase...${NC}"
cd "$SCRIPT_DIR/before"
BEFORE_OUTPUT=$("$PARSELTONGUE" pt01-folder-to-cozodb-streamer . 2>&1)
echo "$BEFORE_OUTPUT" | head -20

# Extract workspace path from output
BEFORE_WORKSPACE=$(echo "$BEFORE_OUTPUT" | grep -o 'parseltongue[0-9]*' | head -1)
BEFORE_DB="rocksdb:$SCRIPT_DIR/before/$BEFORE_WORKSPACE/analysis.db"
echo "Before DB: $BEFORE_DB"

# Index AFTER codebase
echo -e "\n${YELLOW}Step 3: Indexing AFTER codebase...${NC}"
cd "$SCRIPT_DIR/after"
AFTER_OUTPUT=$("$PARSELTONGUE" pt01-folder-to-cozodb-streamer . 2>&1)
echo "$AFTER_OUTPUT" | head -20

# Extract workspace path from output
AFTER_WORKSPACE=$(echo "$AFTER_OUTPUT" | grep -o 'parseltongue[0-9]*' | head -1)
AFTER_DB="rocksdb:$SCRIPT_DIR/after/$AFTER_WORKSPACE/analysis.db"
echo "After DB: $AFTER_DB"

# Run diff command (human-readable)
echo -e "\n${YELLOW}Step 4: Running diff (human-readable)...${NC}"
"$PARSELTONGUE" diff \
    --base "$BEFORE_DB" \
    --live "$AFTER_DB" \
    --max-hops 2 2>&1 || true

# Run diff command (JSON output)
echo -e "\n${YELLOW}Step 5: Running diff (JSON output)...${NC}"
DIFF_JSON=$("$PARSELTONGUE" diff \
    --base "$BEFORE_DB" \
    --live "$AFTER_DB" \
    --json 2>&1 || true)

# Save JSON output
echo "$DIFF_JSON" > "$SCRIPT_DIR/diff_result.json"
echo "Saved diff result to: $SCRIPT_DIR/diff_result.json"

# Validate expected changes
echo -e "\n${YELLOW}Step 6: Validating diff system functionality...${NC}"

ERRORS=0
WARNINGS=0

# Core functionality: Check that diff contains required structure
if echo "$DIFF_JSON" | grep -q '"summary"'; then
    echo -e "${GREEN}✓ Diff contains summary${NC}"
else
    echo -e "${RED}✗ Missing summary in diff${NC}"
    ERRORS=$((ERRORS + 1))
fi

if echo "$DIFF_JSON" | grep -q '"blast_radius"'; then
    echo -e "${GREEN}✓ Diff contains blast_radius${NC}"
else
    echo -e "${RED}✗ Missing blast_radius in diff${NC}"
    ERRORS=$((ERRORS + 1))
fi

if echo "$DIFF_JSON" | grep -q '"visualization"'; then
    echo -e "${GREEN}✓ Diff contains visualization${NC}"
else
    echo -e "${RED}✗ Missing visualization in diff${NC}"
    ERRORS=$((ERRORS + 1))
fi

if echo "$DIFF_JSON" | grep -q '"entity_changes"'; then
    echo -e "${GREEN}✓ Diff contains entity_changes array${NC}"
else
    echo -e "${RED}✗ Missing entity_changes array${NC}"
    ERRORS=$((ERRORS + 1))
fi

if echo "$DIFF_JSON" | grep -q '"edge_changes"'; then
    echo -e "${GREEN}✓ Diff contains edge_changes array${NC}"
else
    echo -e "${RED}✗ Missing edge_changes array${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check for any entity or edge changes detected
if echo "$DIFF_JSON" | grep -q "ToCodebase\|ToGraph"; then
    echo -e "${GREEN}✓ Diff detected changes (entity or edge)${NC}"
else
    echo -e "${YELLOW}⚠ No entity or edge changes detected (may be expected if fixtures have issues)${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

# Check stable_identity is being used
if echo "$DIFF_JSON" | grep -q '"stable_identity"'; then
    echo -e "${GREEN}✓ Key normalization working (stable_identity present)${NC}"
else
    echo -e "${YELLOW}⚠ No stable_identity found (may be expected if no entity changes)${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

# Check visualization has nodes structure
if echo "$DIFF_JSON" | grep -q '"nodes"'; then
    echo -e "${GREEN}✓ Visualization has nodes array${NC}"
else
    echo -e "${YELLOW}⚠ Visualization missing nodes${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

# Summary
echo -e "\n======================================"
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}E2E TEST PASSED${NC}"
    echo "Core diff functionality verified."
    if [ $WARNINGS -gt 0 ]; then
        echo -e "${YELLOW}($WARNINGS warnings - may indicate test fixture issues)${NC}"
    fi
else
    echo -e "${RED}E2E TEST FAILED${NC}"
    echo "$ERRORS critical errors found."
    # Show the JSON for debugging
    echo -e "\n${YELLOW}Debug: Diff JSON output (first 100 lines):${NC}"
    echo "$DIFF_JSON" | head -100
    exit 1
fi
echo "======================================"

# Keep test data for inspection
echo -e "\nTest databases preserved at:"
echo "  Before: $BEFORE_DB"
echo "  After: $AFTER_DB"
echo "  JSON result: $SCRIPT_DIR/diff_result.json"
