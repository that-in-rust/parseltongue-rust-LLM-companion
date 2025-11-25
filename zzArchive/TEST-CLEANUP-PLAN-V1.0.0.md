# Test Cleanup Plan - v1.0.0 Architecture

**Date**: 2025-11-25
**Status**: 🚧 EXECUTING
**Philosophy**: TDD-First (S01 + S06) - Delete obsolete, fix broken, keep relevant

---

## Executive Summary

**Current State**: 33 test files found
**Target State**: ~15-20 test files (DELETE 13-18 obsolete files)
**Reason**: v1.0.0 removed editing tools (pt03-pt06), only analysis/search remains

---

## PHASE 1: IMMEDIATE DELETIONS (High Confidence)

### 1.1 E2E Tests - DELETE BOTH (Editing Workflow)
```bash
rm crates/parseltongue-e2e-tests/tests/complete_workflow_test.rs
rm crates/parseltongue-e2e-tests/tests/orchestrator_workflow_test.rs
```
**Reason**: Test 6-tool editing workflow (pt01-pt06). v1.0.0 only has pt01, pt02, pt07.
**Line count**: ~600 lines of obsolete code

### 1.2 Tool Tests - DELETE 2 of 3
```bash
rm crates/parseltongue-core/tests/tool2_temporal_operations.rs  # Tests pt03 editing
rm crates/parseltongue-core/tests/tool3_prd_compliance.rs       # Tests pt03 writing
# KEEP: crates/parseltongue-core/tests/tool1_verification.rs   # Tests pt01 (valid)
```
**Reason**: tool2 and tool3 reference temporal editing operations from deleted pt03-pt06

### 1.3 pt08 Crate - ALREADY DELETED ✅
```bash
# Already executed: rm -rf crates/pt08-semantic-atom-cluster-builder
```

**Total Immediate Deletions**: 4 files (~1000+ lines)

---

## PHASE 2: SWIFT TESTS REVIEW (Medium Priority)

### 2.1 Swift Debug/Exploration Files - LIKELY DELETE
```
crates/parseltongue-core/tests/swift_ast_explorer.rs
crates/parseltongue-core/tests/swift_full_code_debug.rs
crates/parseltongue-core/tests/swift_protocol_debug.rs
crates/parseltongue-core/tests/swift_query_debug.rs
```
**Reason**: Filenames contain "debug" or "explorer" - likely exploratory code
**Action**: Review each file - if no `#[test]` functions or only `#[ignore]`, DELETE

### 2.2 Swift Production Tests - KEEP (Conditional)
```
crates/parseltongue-core/tests/swift_declaration_analysis.rs
crates/parseltongue-core/tests/swift_fix_validation.rs
crates/parseltongue-core/tests/swift_integration_test.rs
crates/parseltongue-core/tests/swift_protocol_query_test.rs
```
**Action**: Review each - if they have passing `#[test]` functions, KEEP. Otherwise DELETE.

**Decision Criteria**: Does Swift parsing work in v1.0.0? If yes, keep 1-2 comprehensive tests. If no, delete all.

---

## PHASE 3: CORE FUNCTIONALITY TESTS (Keep All)

### 3.1 parseltongue-core - KEEP (6 files)
```
✅ crates/parseltongue-core/tests/ast_exploration_test.rs
✅ crates/parseltongue-core/tests/cozo_storage_integration_tests.rs
✅ crates/parseltongue-core/tests/end_to_end_workflow.rs
✅ crates/parseltongue-core/tests/pt02_level00_zero_dependencies_test.rs
✅ crates/parseltongue-core/tests/query_based_extraction_test.rs
✅ crates/parseltongue-core/tests/query_json_graph_contract_tests.rs
```
**Action**: Run `cargo test` on each. Fix failures to match v1.0.0 architecture.

### 3.2 pt01 Tests - KEEP (6 files)
```
✅ crates/pt01-folder-to-cozodb-streamer/src/test_detector.rs
✅ crates/pt01-folder-to-cozodb-streamer/tests/complex_rust_patterns_test.rs
✅ crates/pt01-folder-to-cozodb-streamer/tests/multi_language_extraction_test.rs
✅ crates/pt01-folder-to-cozodb-streamer/tests/tdd_classification_test.rs
✅ crates/pt01-folder-to-cozodb-streamer/tests/tdd_dependency_extraction_test.rs
✅ crates/pt01-folder-to-cozodb-streamer/tests/tree_sitter_api_compatibility_test.rs
✅ crates/pt01-folder-to-cozodb-streamer/tests/verify_lsp_storage.rs
```
**Action**: Verify all pass. These are core to v1.0.0.

### 3.3 pt02 Tests - KEEP (5 files)
```
✅ crates/pt02-llm-cozodb-to-context-writer/tests/integration_tests.rs
✅ crates/pt02-llm-cozodb-to-context-writer/tests/level0_tests.rs
✅ crates/pt02-llm-cozodb-to-context-writer/tests/level1_tests.rs
✅ crates/pt02-llm-cozodb-to-context-writer/tests/level2_tests.rs
✅ crates/pt02-llm-cozodb-to-context-writer/tests/toon_token_efficiency_test.rs
```
**Action**: Verify all pass. These are core to v1.0.0.

### 3.4 pt07 Tests - KEEP (2 files)
```
✅ crates/pt07-visual-analytics-terminal/tests/integration_cycle_detection.rs
✅ crates/pt07-visual-analytics-terminal/tests/integration_database_adapter.rs
```
**Action**: Verify all pass. These are core to v1.0.0.

---

## PHASE 4: EXECUTION PLAN

### Step 1: Delete Confirmed Obsolete Tests
```bash
# E2E tests (editing workflow)
rm crates/parseltongue-e2e-tests/tests/complete_workflow_test.rs
rm crates/parseltongue-e2e-tests/tests/orchestrator_workflow_test.rs

# Tool tests (pt03-pt06 references)
rm crates/parseltongue-core/tests/tool2_temporal_operations.rs
rm crates/parseltongue-core/tests/tool3_prd_compliance.rs
```

### Step 2: Review Swift Tests
```bash
# Check each Swift test file for actual test functions
for file in crates/parseltongue-core/tests/swift_*.rs; do
  echo "=== $file ==="
  grep -c "#\[test\]" "$file" || echo "0 tests"
  grep -c "#\[ignore\]" "$file" || echo "0 ignored"
done
```

### Step 3: Run Test Suite
```bash
cargo test --all 2>&1 | tee test-results-v1.0.0.txt
```

### Step 4: Analyze Failures
- Identify tests failing due to pt03-pt06 references
- Delete or fix each failure
- Re-run until 100% pass rate

### Step 5: Document Results
Create TEST-SUITE-REPORT-V1.0.0.md with:
- Total tests: before/after
- Deleted tests: list with reasons
- Passing tests: categorized by crate
- Test coverage: % of v1.0.0 features covered

---

## PHASE 5: TDD COMPLIANCE CHECK

### 5.1 Four-Word Naming Convention
**Review**: Do test function names follow 4-word pattern?
```rust
// ✅ CORRECT
#[test]
fn test_workflow_ingestion_query_visualization() {}

// ❌ WRONG
#[test]
fn test_ingest() {}  // Too short (2 words)
```

### 5.2 Executable Specifications
**Review**: Do tests have clear contracts?
```rust
/// **Executable Contract**: Query performance test
///
/// **Preconditions**: Database has 1000 entities
/// **Postconditions**: Query completes in <50ms
/// **Error Conditions**: Returns error if DB offline
#[test]
fn test_query_performance_meets_contract() {}
```

### 5.3 STUB → RED → GREEN → REFACTOR
**Review**: Are tests written in TDD cycle order?
- Check git history for test-first commits
- Verify tests fail when implementation is removed

---

## Expected Outcomes

### Before Cleanup
- Total test files: 33
- Obsolete tests: ~13-18 (40-55%)
- Passing tests: Unknown
- Test run time: Unknown

### After Cleanup
- Total test files: ~15-20
- Obsolete tests: 0 (deleted)
- Passing tests: 100%
- Test run time: <30 seconds
- Coverage: pt01 (ingest), pt02 (query), pt07 (visualize)

---

## Risk Assessment

### LOW RISK Deletions
- ✅ E2E tests (6-tool workflow doesn't exist)
- ✅ tool2/tool3 tests (pt03-pt06 don't exist)
- ✅ pt08 crate (not in v1.0.0 architecture)

### MEDIUM RISK Deletions
- ⚠️ Swift debug files (might have useful production tests)
- ⚠️ Temporal operations (might be used by pt02 in some way)

### HIGH RISK Actions
- 🚨 Fixing core tests (must not break pt01, pt02, pt07 functionality)

---

## Rollback Plan

If cleanup breaks critical functionality:
1. `git stash` all changes
2. Review failing test in detail
3. Determine if test reveals actual bug
4. Fix implementation, not test
5. Re-apply cleanup

---

**Next Action**: Execute Phase 1 deletions immediately
