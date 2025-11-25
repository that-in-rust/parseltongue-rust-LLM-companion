# Test Suite Inventory - v1.0.0 Architecture Audit

**Date**: 2025-11-25
**Purpose**: Audit all tests for relevance to v1.0.0 (pure analysis/search, no editing)
**Philosophy**: Following TDD principles from S01-README-MOSTIMP.md and S06-design101-tdd-architecture-principles.md

---

## Summary

**Total Test Files Found**: 33
**Breakdown by Crate**:
- parseltongue-core: 16 test files
- parseltongue-e2e-tests: 2 test files
- pt01-folder-to-cozodb-streamer: 6 test files
- pt02-llm-cozodb-to-context-writer: 5 test files
- pt07-visual-analytics-terminal: 2 test files
- pt08-semantic-atom-cluster-builder: 1 test file (⚠️ SHOULD NOT EXIST)

---

## Test Inventory by Category

### Category 1: parseltongue-core Tests (16 files)

#### 1.1 Swift Language Tests (10 files) - QUESTIONABLE RELEVANCE
```
crates/parseltongue-core/tests/swift_ast_explorer.rs
crates/parseltongue-core/tests/swift_declaration_analysis.rs
crates/parseltongue-core/tests/swift_fix_validation.rs
crates/parseltongue-core/tests/swift_full_code_debug.rs
crates/parseltongue-core/tests/swift_integration_test.rs
crates/parseltongue-core/tests/swift_protocol_debug.rs
crates/parseltongue-core/tests/swift_protocol_query_test.rs
crates/parseltongue-core/tests/swift_query_debug.rs
```
**Status**: ⚠️ REVIEW NEEDED
**Concern**: 10 Swift-specific test files - are these debug/exploration files or production tests?
**Action**: Analyze if Swift parsing is properly tested or if these are experimental

#### 1.2 Core Functionality Tests (6 files) - KEEP
```
crates/parseltongue-core/tests/ast_exploration_test.rs
crates/parseltongue-core/tests/cozo_storage_integration_tests.rs
crates/parseltongue-core/tests/end_to_end_workflow.rs
crates/parseltongue-core/tests/pt02_level00_zero_dependencies_test.rs
crates/parseltongue-core/tests/query_based_extraction_test.rs
crates/parseltongue-core/tests/query_json_graph_contract_tests.rs
```
**Status**: ✅ LIKELY RELEVANT
**Reason**: Core functionality (storage, queries, workflows)
**Action**: Verify no pt03-pt06 dependencies

#### 1.3 Tool Tests (3 files) - REVIEW FOR pt03-pt06
```
crates/parseltongue-core/tests/tool1_verification.rs
crates/parseltongue-core/tests/tool2_temporal_operations.rs
crates/parseltongue-core/tests/tool3_prd_compliance.rs
```
**Status**: ⚠️ REVIEW NEEDED
**Concern**: "tool3" might reference editing tools (pt03-pt06)
**Action**: Check if these reference deleted tools

---

### Category 2: E2E Tests (2 files) - CRITICAL REVIEW

```
crates/parseltongue-e2e-tests/tests/complete_workflow_test.rs
crates/parseltongue-e2e-tests/tests/orchestrator_workflow_test.rs
```
**Status**: 🚨 HIGH PRIORITY REVIEW
**Concern**: E2E tests likely reference full workflow including pt03-pt06
**Action**: Update to test only pt01 → pt02 → pt07 workflow

---

### Category 3: pt01 Tests (6 files) - KEEP

```
crates/pt01-folder-to-cozodb-streamer/src/test_detector.rs
crates/pt01-folder-to-cozodb-streamer/tests/complex_rust_patterns_test.rs
crates/pt01-folder-to-cozodb-streamer/tests/multi_language_extraction_test.rs
crates/pt01-folder-to-cozodb-streamer/tests/tdd_classification_test.rs
crates/pt01-folder-to-cozodb-streamer/tests/tdd_dependency_extraction_test.rs
crates/pt01-folder-to-cozodb-streamer/tests/tree_sitter_api_compatibility_test.rs
crates/pt01-folder-to-cozodb-streamer/tests/verify_lsp_storage.rs
```
**Status**: ✅ KEEP
**Reason**: pt01 is core to v1.0.0 (ingestion)
**Action**: Verify all tests pass

---

### Category 4: pt02 Tests (5 files) - KEEP

```
crates/pt02-llm-cozodb-to-context-writer/tests/integration_tests.rs
crates/pt02-llm-cozodb-to-context-writer/tests/level0_tests.rs
crates/pt02-llm-cozodb-to-context-writer/tests/level1_tests.rs
crates/pt02-llm-cozodb-to-context-writer/tests/level2_tests.rs
crates/pt02-llm-cozodb-to-context-writer/tests/toon_token_efficiency_test.rs
```
**Status**: ✅ KEEP
**Reason**: pt02 is core to v1.0.0 (query/export)
**Action**: Verify all tests pass

---

### Category 5: pt07 Tests (2 files) - KEEP

```
crates/pt07-visual-analytics-terminal/tests/integration_cycle_detection.rs
crates/pt07-visual-analytics-terminal/tests/integration_database_adapter.rs
```
**Status**: ✅ KEEP
**Reason**: pt07 is core to v1.0.0 (visualization)
**Action**: Verify all tests pass

---

### Category 6: pt08 Tests (1 file) - DELETE

```
crates/pt08-semantic-atom-cluster-builder/src/algorithms/lpa.rs (contains tests)
```
**Status**: ❌ DELETE ENTIRE CRATE
**Reason**: pt08 doesn't exist in v1.0.0 architecture (only pt01, pt02, pt07)
**Action**: Delete entire pt08 directory

---

## Inline Tests (in src files)

### parseltongue-core
- entities.rs: ✅ KEEP (entity tests)
- entity_class_specifications.rs: ✅ KEEP (specs)
- error.rs: ✅ KEEP (error handling)
- interfaces.rs: ✅ KEEP (core interfaces)
- output_path_resolver.rs: ✅ KEEP (path resolution)
- serializers/json.rs: ✅ KEEP (JSON serialization)
- serializers/toon.rs: ✅ KEEP (TOON format)
- temporal.rs: ⚠️ REVIEW (temporal operations - editing related?)

### pt01
- cli.rs: ✅ KEEP (CLI tests)
- isgl1_generator.rs: ✅ KEEP (key generation)
- test_detector.rs: ✅ KEEP (test classification)
- v090_specifications.rs: ✅ KEEP (specifications)
- lsp_client.rs: ✅ KEEP (LSP integration)
- streamer.rs: ✅ KEEP (streaming logic)

### pt02
- cli.rs: ✅ KEEP (CLI tests)
- cozodb_adapter.rs: ✅ KEEP (database adapter)
- entity_class_integration_tests.rs: ✅ KEEP (entity class tests)
- export_trait.rs: ✅ KEEP (export interface)
- exporters/level0.rs: ✅ KEEP (level 0 export)
- exporters/level1.rs: ✅ KEEP (level 1 export)
- exporters/level2.rs: ✅ KEEP (level 2 export)
- models.rs: ✅ KEEP (data models)
- query_builder.rs: ✅ KEEP (query construction)

### pt07
- cycle_detection.rs: ✅ KEEP (cycle detection)
- filter_implementation_edges_only.rs: ✅ KEEP (edge filtering)
- filter_implementation_entities_only.rs: ✅ KEEP (entity filtering)
- adapter.rs: ✅ KEEP (database adapter)
- conversion.rs: ✅ KEEP (data conversion)
- render_box_drawing_unicode.rs: ✅ KEEP (rendering)
- render_color_emoji_terminal.rs: ✅ KEEP (terminal output)
- render_progress_bar_horizontal.rs: ✅ KEEP (progress bars)

### parseltongue (main binary)
- main.rs: ⚠️ REVIEW (might have CLI tests referencing pt03-pt06)

---

## Action Plan

### Phase 1: CRITICAL - Delete Obsolete Code
- [ ] Delete entire `crates/pt08-semantic-atom-cluster-builder/` directory
- [ ] Verify pt08 removed from workspace Cargo.toml

### Phase 2: HIGH PRIORITY - Fix E2E Tests
- [ ] Review `complete_workflow_test.rs` - remove pt03-pt06 references
- [ ] Review `orchestrator_workflow_test.rs` - remove pt03-pt06 references
- [ ] Update to test: pt01 → pt02 → pt07 workflow only

### Phase 3: MEDIUM PRIORITY - Review Tool Tests
- [ ] Review `tool1_verification.rs` - verify it's pt01
- [ ] Review `tool2_temporal_operations.rs` - verify it's pt02
- [ ] Review `tool3_prd_compliance.rs` - check if it references pt03 (DELETE if so)

### Phase 4: LOW PRIORITY - Swift Test Consolidation
- [ ] Analyze all 10 Swift test files
- [ ] Determine if they're exploratory/debug files
- [ ] Delete debug files, keep production tests
- [ ] Consider consolidating into 1-2 comprehensive Swift tests

### Phase 5: VERIFICATION
- [ ] Run `cargo test --all` and document failures
- [ ] Fix failing tests one by one following TDD
- [ ] Ensure 100% test pass rate
- [ ] Update this document with final status

---

## Test Philosophy Compliance (S01 + S06)

### TDD Cycle Compliance
- ✅ All tests should follow: STUB → RED → GREEN → REFACTOR
- ⚠️ Many tests might be stale (written before v1.0.0 refactor)
- 🎯 Goal: Every test validates a concrete requirement

### 4-Word Naming Convention
- ⚠️ Review test function names for 4-word compliance
- Example: `test_workflow_ingestion_query_visualization()` (4 words after `test_`)

### Executable Specifications
- ✅ Tests should have clear preconditions, postconditions, error conditions
- ⚠️ Review tests for "contract-driven" structure

---

## Next Steps

1. Execute Phase 1 (delete pt08)
2. Run `cargo test --all` to identify broken tests
3. Systematically fix each failure following TDD
4. Update this document with final results
5. Create TEST-SUITE-REPORT.md with pass/fail statistics

---

**Status**: 🚧 IN PROGRESS
**Last Updated**: 2025-11-25
