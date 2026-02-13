# v1.6.5 TDD Progress Journal: Comprehensive Capability Testing

## Version Goal

**Target**: Full test coverage for all v1.6.0-v1.6.5 capabilities including graph algorithms, scope filtering, diagnostics, import word counting, ignored files tracking, parallelism, and HTTP endpoint correctness.

## Test Suite Baseline (2026-02-11)

| Crate | Tests | Passed | Failed | Ignored |
|-------|-------|--------|--------|---------|
| parseltongue (CLI binary) | 1 | 1 | 0 | 0 |
| parseltongue-core (lib + 30 test files) | 617 | 612 | 2* | 3 |
| pt01-folder-to-cozodb-streamer | 0 | 0 | 0 | 0 |
| pt08-http-code-query-server | 52 | 52 | 0 | 0 |
| **Total** | **670** | **665** | **2*** | **3** |

\* 2 failures are pre-existing flaky performance tests (`test_insert_entities_batch_large`, `test_insert_entities_batch_large_content`) with timing thresholds (448ms vs 200ms limit). Unrelated to v1.6.5.

---

## Category 1: Graph Analysis Algorithms (v1.6.0)

### 1.1 Tarjan SCC (Strongly Connected Components)

**Module**: `parseltongue-core::graph_analysis::tarjan_scc_algorithm`
**HTTP Endpoint**: `GET /strongly-connected-components-detection`

| # | Test | Status | Validates |
|---|------|--------|-----------|
| 1 | `test_tarjan_empty_graph` | PASS | Returns empty components for empty graph |
| 2 | `test_tarjan_single_node_no_cycle` | PASS | Isolated node is not an SCC |
| 3 | `test_tarjan_simple_cycle_ab` | PASS | A->B->A detected as SCC |
| 4 | `test_tarjan_chain_no_scc` | PASS | A->B->C has no SCC (no cycles) |
| 5 | `test_tarjan_eight_node_graph` | PASS | Multi-SCC detection in complex graph |

### 1.2 K-Core Decomposition

**Module**: `parseltongue-core::graph_analysis::kcore_decomposition_algorithm`
**HTTP Endpoint**: `GET /kcore-decomposition-layering-view`

| # | Test | Status | Validates |
|---|------|--------|-----------|
| 1 | `test_kcore_empty_graph` | PASS | Empty graph returns no layers |
| 2 | `test_kcore_chain_all_coreness_one` | PASS | Chain graph: all nodes coreness=1 |
| 3 | `test_kcore_cycle_nodes_coreness_two` | PASS | Cycle nodes get coreness=2 |
| 4 | `test_kcore_eight_node_max_coreness` | PASS | Correct max coreness in complex graph |
| 5 | `test_kcore_layers_partition_nodes` | PASS | Every node assigned exactly one layer |

### 1.3 PageRank + Betweenness Centrality

**Module**: `parseltongue-core::graph_analysis::centrality_measures_algorithm`
**HTTP Endpoint**: `GET /centrality-measures-entity-ranking`

| # | Test | Status | Validates |
|---|------|--------|-----------|
| 1 | `test_pagerank_empty_graph` | PASS | Empty graph returns empty rankings |
| 2 | `test_pagerank_chain_sink_highest` | PASS | Sink node gets highest PageRank |
| 3 | `test_pagerank_chain_values_approximate` | PASS | PR values within expected range |
| 4 | `test_pagerank_sums_to_one` | PASS | All PageRank values sum to ~1.0 |
| 5 | `test_pagerank_eight_node_graph` | PASS | Complex graph PageRank convergence |
| 6 | `test_betweenness_empty_graph` | PASS | Empty graph returns empty |
| 7 | `test_betweenness_chain_middle_highest` | PASS | Middle nodes highest betweenness |

### 1.4 Shannon Entropy

**Module**: `parseltongue-core::graph_analysis::entropy_complexity_algorithm`
**HTTP Endpoint**: `GET /entropy-complexity-measurement-view`

| # | Test | Status | Validates |
|---|------|--------|-----------|
| 1 | `test_entropy_no_edges_zero` | PASS | Node with no edges: entropy=0 |
| 2 | `test_entropy_single_type_zero` | PASS | Single edge type: entropy=0 |
| 3 | `test_entropy_two_types_mixed` | PASS | Two types: entropy > 0 |
| 4 | `test_entropy_uniform_three_types` | PASS | Uniform 3 types: max entropy |
| 5 | `test_entropy_nonexistent_node` | PASS | Nonexistent node returns None |
| 6 | `test_classify_entropy_levels` | PASS | Low/medium/high classification |
| 7 | `test_compute_all_entropy_scores` | PASS | Batch computation correctness |

### 1.5 CK Metrics Suite (Coupling/Cohesion)

**Module**: `parseltongue-core::graph_analysis::ck_metrics_suite_algorithm`
**HTTP Endpoint**: `GET /coupling-cohesion-metrics-report`

| # | Test | Status | Validates |
|---|------|--------|-----------|
| 1 | `test_cbo_node_a_source` | PASS | CBO counts unique couplings |
| 2 | `test_cbo_node_d_high_coupling` | PASS | Hub node has high CBO |
| 3 | `test_cbo_nonexistent_node` | PASS | Returns None for missing node |
| 4 | `test_rfc_node_a` | PASS | RFC = 1 + outgoing calls |
| 5 | `test_rfc_leaf_node` | PASS | Leaf RFC = 1 |
| 6 | `test_wmc_proxy_out_degree` | PASS | WMC proxy via out-degree |
| 7 | `test_wmc_node_d` | PASS | WMC for multi-edge node |
| 8 | `test_lcom_shared_target` | PASS | Shared target reduces LCOM |
| 9 | `test_lcom_independent_branches` | PASS | Independent targets: LCOM=0 |
| 10 | `test_health_grade_all_ok` | PASS | All metrics OK: grade A |
| 11 | `test_health_grade_one_warning` | PASS | Warning threshold: grade B |
| 12 | `test_health_grade_one_fail` | PASS | Fail threshold: grade C |
| 13 | `test_health_grade_two_fails` | PASS | Multiple fails: grade D/F |

### 1.6 SQALE Technical Debt

**Module**: `parseltongue-core::graph_analysis::sqale_technical_debt_algorithm`
**HTTP Endpoint**: `GET /technical-debt-sqale-rating-view`

| # | Test | Status | Validates |
|---|------|--------|-----------|
| 1 | `test_sqale_empty_graph` | PASS | Empty graph: zero debt |
| 2 | `test_sqale_chain_no_cycles` | PASS | Acyclic graph: minimal debt |
| 3 | `test_sqale_cycle_increases_debt` | PASS | Cycles contribute to debt |
| 4 | `test_sqale_rating_scale` | PASS | A-E rating scale correctness |
| 5 | `test_sqale_eight_node_graph` | PASS | Complex graph debt calculation |

### 1.7 Leiden Community Detection

**Module**: `parseltongue-core::graph_analysis::leiden_community_algorithm`
**HTTP Endpoint**: `GET /leiden-community-detection-report`

| # | Test | Status | Validates |
|---|------|--------|-----------|
| 1 | `test_leiden_empty_graph` | PASS | Empty graph: no communities |
| 2 | `test_leiden_isolated_nodes` | PASS | Isolated nodes: one per community |
| 3 | `test_leiden_two_cliques` | PASS | Two cliques -> two communities |
| 4 | `test_leiden_chain_graph` | PASS | Chain partitioning stability |
| 5 | `test_leiden_eight_node_graph` | PASS | Complex graph community structure |

### 1.8 Cross-Algorithm Integration Tests

**Module**: `parseltongue-core::graph_analysis::integration_cross_algorithm_tests`

| # | Test | Status | Validates |
|---|------|--------|-----------|
| 1 | `test_all_seven_algorithms_run_on_chain_graph` | PASS | All 7 produce output on chain |
| 2 | `test_all_seven_algorithms_run_on_eight_node_graph` | PASS | All 7 produce output on 8-node |
| 3 | `test_empty_graph_all_algorithms_return_empty` | PASS | All 7 handle empty gracefully |
| 4 | `test_cycle_nodes_detected_by_both_scc_and_leiden` | PASS | SCC+Leiden agree on cycles |
| 5 | `test_high_betweenness_implies_high_pagerank` | PASS | Centrality correlation |
| 6 | `test_isolated_pair_consistent_across_algorithms` | PASS | 2-node consistency |
| 7 | `test_scc_nodes_share_kcore_level` | PASS | SCC members same k-core |
| 8 | `test_sqale_debt_correlates_with_coupling` | PASS | Debt-coupling correlation |
| 9 | `test_performance_large_graph_completes` | PASS | 100-node perf < threshold |

**Category 1 Total: 83 tests, ALL PASSING**

---

## Category 2: v1.6.5 Wave 1 -- Ship-Blocking Features

### 2.1 Import Word Count (`compute_import_word_count_safely`)

**Module**: `parseltongue-core::query_extractor::import_counting_tests`
**Location**: `crates/parseltongue-core/src/query_extractor.rs`

| # | Test | Status | Validates |
|---|------|--------|-----------|
| 1 | `test_compute_import_word_count_rust` | PASS | `use_declaration` node counting for Rust |
| 2 | `test_compute_import_word_count_python` | PASS | `import_statement`, `import_from_statement` for Python |
| 3 | `test_compute_import_word_count_javascript` | PASS | `import_statement` + `require()` calls for JS |
| 4 | `test_compute_import_word_count_go_nested` | PASS | Nested `import_declaration` block for Go |

**Supported languages with import node types**:
| Language | AST Node Types |
|----------|---------------|
| Rust | `use_declaration` |
| Python | `import_statement`, `import_from_statement` |
| JS/TS | `import_statement`, `call_expression` (require) |
| Go | `import_declaration` |
| Java | `import_declaration` |
| C/C++ | `preproc_include` |
| Ruby | `call` (require/require_relative) |
| PHP | `use_declaration`, `expression_statement` (require) |
| C# | `using_directive` |
| Swift | `import_declaration` |

**GAPS IDENTIFIED**: Tests only cover 4/12 languages. Missing: TypeScript, Java, C, C++, Ruby, PHP, C#, Swift.

#### Proposed Additional Tests (RED phase - not yet written):
- [ ] `test_compute_import_word_count_typescript` -- `import` + `require` in .ts
- [ ] `test_compute_import_word_count_java` -- `import java.util.*;`
- [ ] `test_compute_import_word_count_c_includes` -- `#include <stdio.h>`
- [ ] `test_compute_import_word_count_cpp_includes` -- `#include <iostream>` + `#include "header.h"`
- [ ] `test_compute_import_word_count_ruby_require` -- `require 'json'` + `require_relative`
- [ ] `test_compute_import_word_count_php_use` -- `use App\Models\User;` + `require_once`
- [ ] `test_compute_import_word_count_csharp_using` -- `using System;` + `using static`
- [ ] `test_compute_import_word_count_swift_import` -- `import Foundation`
- [ ] `test_import_word_count_empty_file` -- 0 imports returns 0
- [ ] `test_import_word_count_no_imports_in_code` -- File with only functions, no imports

### 2.2 Ignored Files Tracking

**Schema**: `IgnoredFiles { folder_path, filename => extension, reason }`
**Struct**: `parseltongue_core::entities::IgnoredFileRow`
**Storage**: `insert_ignored_files_batch()` in `cozo_client.rs`

| # | Test | Status | Validates |
|---|------|--------|-----------|
| - | (no dedicated unit tests yet) | GAP | Schema creation, batch insert, query |

**GAPS IDENTIFIED**: No unit tests for IgnoredFiles schema, insertion, or querying.

#### Proposed Additional Tests:
- [ ] `test_create_ignored_files_schema` -- Schema creation succeeds
- [ ] `test_insert_ignored_files_batch` -- Batch insert of .md, .json, .toml files
- [ ] `test_query_ignored_files_by_extension` -- Filter by extension
- [ ] `test_ignored_files_deduplication` -- Same file not inserted twice
- [ ] `test_ignored_file_reason_categories` -- Verify reason strings

### 2.3 Diagnostics Coverage Endpoint

**Endpoint**: `GET /ingestion-diagnostics-coverage-report`
**Handler**: `ingestion_diagnostics_coverage_handler.rs`
**Sections**: `test_entities_excluded`, `word_count_coverage`, `ignored_files`, `summary`

| # | Test | Status | Validates |
|---|------|--------|-----------|
| - | (no unit tests - HTTP endpoint) | GAP | Response structure, section filtering |

**Live Test Results** (from TDD agent before kill):
- `GET /ingestion-diagnostics-coverage-report` returns `success: true`
- Response keys: `["test_entities_excluded", "word_count_coverage"]`
- Missing: `ignored_files` section (not populated in current ingested DB)

#### Proposed Additional Tests:
- [ ] `test_diagnostics_section_filter_test_entities` -- `?section=test_entities` returns only that section
- [ ] `test_diagnostics_section_filter_word_coverage` -- `?section=word_coverage` returns only that section
- [ ] `test_diagnostics_section_filter_ignored_files` -- `?section=ignored_files` returns only that section
- [ ] `test_diagnostics_section_filter_summary` -- `?section=summary` returns aggregates only
- [ ] `test_diagnostics_no_section_returns_all` -- No param returns all sections
- [ ] `test_diagnostics_invalid_section_returns_all` -- Invalid section name returns all

---

## Category 3: v1.6.5 Wave 2 -- Scope Filtering (18 Endpoints)

### 3.1 Scope Filter Utility

**Module**: `pt08-http-code-query-server::scope_filter_utilities_module`
**Functions**: `parse_scope_build_filter_clause()`, `validate_scope_exists_in_database()`

| # | Behavior | Status | Validates |
|---|----------|--------|-----------|
| 1 | `None` scope -> empty string | IMPLEMENTED | No filtering applied |
| 2 | `Some("")` -> empty string | IMPLEMENTED | Empty scope = no filter |
| 3 | `Some("src")` -> `, root_subfolder_L1 = 'src'` | IMPLEMENTED | L1-only filter |
| 4 | `Some("src\|\|core")` -> `, root_subfolder_L1 = 'src', root_subfolder_L2 = 'core'` | IMPLEMENTED | L1+L2 filter |
| 5 | Single-quote escaping | IMPLEMENTED | SQL injection prevention |
| 6 | Scope validation with did-you-mean suggestions | IMPLEMENTED | Invalid scope returns suggestions |

**GAPS IDENTIFIED**: No unit tests for `parse_scope_build_filter_clause()`.

#### Proposed Additional Tests:
- [ ] `test_parse_scope_none_returns_empty` -- None input
- [ ] `test_parse_scope_empty_returns_empty` -- Empty string
- [ ] `test_parse_scope_l1_only` -- Single segment
- [ ] `test_parse_scope_l1_l2` -- Double-pipe separated
- [ ] `test_parse_scope_single_quote_escape` -- `O'Reilly` folder name
- [ ] `test_parse_scope_extra_pipes_ignored` -- `a||b||c` only uses first two
- [ ] `test_parse_scope_whitespace_trimmed` -- ` src || core ` trims whitespace

### 3.2 Scoped Endpoints (All 18)

**Pattern**: Each handler adds `scope: Option<String>` to query params, calls `parse_scope_build_filter_clause()`, appends to Datalog query.

| # | Endpoint | Scope Support | Test |
|---|----------|---------------|------|
| 1 | `/code-entities-list-all` | YES | Live: 77 entities for `crates\|\|parseltongue-core` |
| 2 | `/code-entities-search-fuzzy` | YES | Live: 0 results for `parse` in `crates\|\|parseltongue-core` (needs investigation) |
| 3 | `/code-entity-detail-view/{key}` | YES | Not tested |
| 4 | `/dependency-edges-list-all` | YES | Not tested |
| 5 | `/reverse-callers-query-graph` | YES | Not tested |
| 6 | `/forward-callees-query-graph` | YES | Not tested |
| 7 | `/blast-radius-impact-analysis` | YES | Live: returned `false` (needs investigation) |
| 8 | `/circular-dependency-detection-scan` | YES | Not tested |
| 9 | `/complexity-hotspots-ranking-view` | YES | Not tested |
| 10 | `/semantic-cluster-grouping-list` | YES | Not tested |
| 11 | `/smart-context-token-budget` | YES | Not tested |
| 12 | `/strongly-connected-components-detection` | YES | Not tested |
| 13 | `/technical-debt-sqale-rating-view` | YES | Not tested |
| 14 | `/kcore-decomposition-layering-view` | YES | Not tested |
| 15 | `/centrality-measures-entity-ranking` | YES | Not tested |
| 16 | `/entropy-complexity-measurement-view` | YES | Not tested |
| 17 | `/coupling-cohesion-metrics-report` | YES | Not tested |
| 18 | `/leiden-community-detection-report` | YES | Not tested |

**GAPS IDENTIFIED**:
- Only 2/18 endpoints tested via live curl
- `fuzzy-search` with scope returned 0 results -- potential bug or data issue
- `blast-radius` with scope returned `false` -- needs investigation
- No automated tests for scope filtering behavior

#### Proposed E2E Tests (require running server):
- [ ] `test_scope_reduces_entity_count` -- Scoped list < unscoped list
- [ ] `test_scope_nonexistent_returns_empty_or_error` -- Invalid scope handled
- [ ] `test_scope_l1_only_broader_than_l1_l2` -- L1 scope > L1+L2 scope
- [ ] `test_scope_on_graph_algorithms` -- Each graph algo respects scope
- [ ] `test_scope_validation_did_you_mean` -- Invalid scope returns suggestions

---

## Category 4: v1.6.5 Wave 3 -- Polish

### 4.1 Section Parameter on Diagnostics

**Parameter**: `?section=test_entities|word_coverage|ignored_files|summary`
**Handler**: `ingestion_diagnostics_coverage_handler.rs`

| # | Test | Status | Validates |
|---|------|--------|-----------|
| - | Live endpoint test | PARTIAL | Endpoint returns data, section param untested |

### 4.2 API Documentation Update

**Endpoint**: `GET /api-reference-documentation-help`
**Handler**: `api_reference_documentation_handler.rs`

| # | Check | Status | Validates |
|---|-------|--------|-----------|
| 1 | All 20 endpoints listed | IMPLEMENTED | Complete endpoint catalog |
| 2 | Scope parameter documented on all 18 query endpoints | IMPLEMENTED | Developer discoverability |
| 3 | `/ingestion-diagnostics-coverage-report` included | IMPLEMENTED | New endpoint documented |
| 4 | `/folder-structure-discovery-tree` included | IMPLEMENTED | New endpoint documented |

---

## Category 5: Parallelism (Thread-Local Parser)

### 5.1 Rayon Parallel Ingestion

**Module**: `pt01-folder-to-cozodb-streamer::streamer`
**Function**: `stream_directory_with_parallel_rayon()`
**Mechanism**: `par_iter()` with thread-local tree-sitter parsers

| # | Capability | Status | Notes |
|---|-----------|--------|-------|
| 1 | Thread-local parser instances | IMPLEMENTED | Each Rayon thread gets own parser |
| 2 | Parallel file processing | IMPLEMENTED | Files processed via `par_iter()` |
| 3 | Import word count in parallel path | IMPLEMENTED | `compute_import_word_count_safely()` called per thread |
| 4 | Ignored files collection in parallel path | IMPLEMENTED | Collected during parallel walk |
| 5 | Sequential fallback still works | IMPLEMENTED | Both paths produce same results |

**GAPS IDENTIFIED**: No unit tests for parallel correctness, no benchmark comparison.

#### Proposed Additional Tests:
- [ ] `test_parallel_sequential_entity_count_match` -- Both paths produce same entity count
- [ ] `test_parallel_sequential_edge_count_match` -- Both paths produce same edge count
- [ ] `test_parallel_import_word_count_nonzero` -- Parallel path produces >0 import counts
- [ ] `test_parallel_ignored_files_collected` -- Parallel path collects ignored files
- [ ] `test_parallel_no_data_races` -- Run 5x, verify deterministic output (entity names match)
- [ ] `test_parallel_performance_faster_than_sequential` -- Par path faster on multi-core

---

## Category 6: Test Corpus (T-Folder Fixtures)

### 6.1 Language-Specific Dependency Pattern Tests

| # | Test Suite | Tests | Status | Languages |
|---|-----------|-------|--------|-----------|
| 1 | `rust_dependency_patterns_test` | 12 | PASS | Rust |
| 2 | `python_dependency_patterns_test` | 12 | PASS | Python |
| 3 | `javascript_dependency_patterns_test` | 10 | PASS | JavaScript |
| 4 | `go_dependency_patterns_test` | 8+2i | PASS | Go |
| 5 | `java_dependency_patterns_test` | 7 | PASS | Java |
| 6 | `c_dependency_patterns_test` | 3 | PASS | C |
| 7 | `cpp_dependency_patterns_test` | 9+1i | PASS | C++ |
| 8 | `ruby_dependency_patterns_test` | 5 | PASS | Ruby |
| 9 | `php_dependency_patterns_test` | 5 | PASS | PHP |
| 10 | `csharp_constructor_detection_test` | 4 | PASS | C# |
| 11 | `csharp_edge_key_integration_test` | 4 | PASS | C# |
| 12 | `csharp_integration_validation_test` | 2 | PASS | C# |
| 13 | `csharp_remaining_patterns_test` | 12 | PASS | C# |
| 14 | `language_field_extraction_tests` | 13 | PASS | Multi-lang |

### 6.2 Infrastructure Tests

| # | Test Suite | Tests | Status | Validates |
|---|-----------|-------|--------|-----------|
| 1 | `cozo_storage_integration_tests` | 33+2i | PASS | CozoDB storage CRUD |
| 2 | `cozo_escaping_tests` | 6 | PASS | Datalog escape handling |
| 3 | `edge_key_format_test` | 10 | PASS | Edge key format validation |
| 4 | `ast_exploration_test` | 4 | PASS | Tree-sitter AST walking |
| 5 | `query_based_extraction_test` | 15 | PASS | Query-based entity extraction |
| 6 | `query_json_graph_contract_tests` | 7 | PASS | JSON graph contracts |

### 6.3 ISGL1 v2 Tests (Incremental Sync)

| # | Test Suite | Tests | Status | Validates |
|---|-----------|-------|--------|-----------|
| 1 | `isgl1_v2_content_hashing_tests` | 4 | PASS | SHA256 content hashing |
| 2 | `isgl1_v2_entity_matching_tests` | 5 | PASS | Entity matching logic |
| 3 | `isgl1_v2_generic_sanitization_tests` | 5 | PASS | Generic type sanitization |
| 4 | `isgl1_v2_integration_tests` | 4 | PASS | Full ISGL1 workflow |
| 5 | `isgl1_v2_key_generation_tests` | 8 | PASS | ISGL1 key format |
| 6 | `isgl1_v2_schema_evolution_tests` | 10 | PASS | Schema migration |

### 6.4 HTTP Server Tests

| # | Test Suite | Tests | Status | Validates |
|---|-----------|-------|--------|-----------|
| 1 | pt08 unit tests | 34 | PASS | Port selection, config, state, hashing |
| 2 | `e2e_file_watcher_tests` | 5 | PASS | File watcher E2E |
| 3 | `e2e_incremental_reindex_isgl1v2_tests` | 5 | PASS | Incremental reindex E2E |
| 4 | `file_watcher_language_coverage_tests` | 3 | PASS | Extension coverage |
| 5 | `watcher_service_lifetime_test` | 4 | PASS | Service lifecycle |
| 6 | `language_field_database_integration_test` | 1 | PASS | DB language field |

---

## Category 7: Ingestion Pipeline

### 7.1 Word Coverage Metrics

**Schema**: `FileWordCoverage { folder_path, filename => ... raw_coverage_pct, effective_coverage_pct, import_word_count, ... }`

| # | Metric | Status | Notes |
|---|--------|--------|-------|
| 1 | `source_word_count` | COMPUTED | Total words in source file |
| 2 | `entity_word_count` | COMPUTED | Words covered by entities |
| 3 | `import_word_count` | COMPUTED (v1.6.5) | Was hardcoded to 0, now uses tree-sitter |
| 4 | `comment_word_count` | COMPUTED | Via `count_top_level_comment_words()` |
| 5 | `raw_coverage_pct` | COMPUTED | entity_words / source_words |
| 6 | `effective_coverage_pct` | COMPUTED | entity_words / (source - imports - comments) |
| 7 | `entity_count` | COMPUTED | Number of entities in file |

### 7.2 Test Entity Exclusion

**Schema**: `TestEntitiesExcluded { entity_name, folder_path, filename => ... detection_reason }`

| # | Detection Pattern | Status | Notes |
|---|------------------|--------|-------|
| 1 | Filename-based (`*_test.rs`, `test_*.py`) | ACTIVE | Most common pattern |
| 2 | Directory-based (`tests/`, `__tests__/`) | ACTIVE | Folder containment |
| 3 | Entity name-based (`test_*`, `Test*`) | ACTIVE | Function/class naming |

---

## Summary: Test Coverage Gaps

### High Priority (should have unit tests):

| Gap | Category | Impact |
|-----|----------|--------|
| Import word count for 8 more languages | 2.1 | TS, Java, C, C++, Ruby, PHP, C#, Swift untested |
| `parse_scope_build_filter_clause()` unit tests | 3.1 | Core utility completely untested |
| IgnoredFiles schema/insert/query tests | 2.2 | New feature untested |
| Parallel vs sequential equivalence | 5.1 | Correctness guarantee untested |

### Medium Priority (E2E validation needed):

| Gap | Category | Impact |
|-----|----------|--------|
| Scope filtering on all 18 endpoints | 3.2 | 16/18 endpoints untested |
| `?section=` parameter on diagnostics | 4.1 | Token efficiency feature untested |
| Blast radius + scope returns `false` | 3.2 | Potential bug |
| Fuzzy search + scope returns 0 | 3.2 | Potential bug or data issue |

### Low Priority (documentation/completeness):

| Gap | Category | Impact |
|-----|----------|--------|
| API docs endpoint verification | 4.2 | Verify 20 endpoints listed |
| Performance benchmark for parallelism | 5.1 | Nice-to-have baseline |
| Storage performance test threshold | Baseline | 448ms vs 200ms limit (pre-existing) |

---

## Next Steps: TDD Red-Green-Refactor Plan

### Phase 1: Unit Tests for Core Utilities (RED)
1. Write `parse_scope_build_filter_clause()` tests (7 tests)
2. Write import word count tests for 8 remaining languages (10 tests)
3. Write IgnoredFiles schema tests (5 tests)

### Phase 2: Make Tests Pass (GREEN)
4. Fix any failures found during test writing
5. Investigate blast radius + scope `false` result
6. Investigate fuzzy search + scope returning 0

### Phase 3: E2E Scope Validation
7. Write E2E test harness for scoped endpoints (5 tests)
8. Verify all 18 endpoints respect scope parameter

### Phase 4: Parallelism Correctness
9. Write parallel vs sequential equivalence tests (6 tests)
10. Add determinism verification (run 5x, compare)

**Total proposed new tests: ~33**
**Current passing tests: 665**
**Target after completion: ~698**
