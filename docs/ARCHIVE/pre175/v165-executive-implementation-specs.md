# v1.6.5 Executive Implementation Specifications

**Feature**: Ingestion Diagnostics Coverage Report + Folder-Scoped Queries
**PRD**: `PRD-v165.md`
**Date**: 2026-02-11
**Status**: Partially Implemented -- Verified Against Live Codebase via Parseltongue
**Verified By**: Parseltongue HTTP server at `localhost:7777` (database: `rocksdb:parseltongue20260211193602/analysis.db`)

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Implementation Status Dashboard](#2-implementation-status-dashboard)
3. [Architecture Analysis (Verified)](#3-architecture-analysis-verified)
4. [Verified Entities: What Exists Today](#4-verified-entities-what-exists-today)
5. [What Remains to Implement](#5-what-remains-to-implement)
6. [Known Bugs and Issues](#6-known-bugs-and-issues)
7. [CozoDB Datalog Patterns: Verified vs Incorrect](#7-cozodb-datalog-patterns-verified-vs-incorrect)
8. [Scope Filtering: Current State and Remaining Work](#8-scope-filtering-current-state-and-remaining-work)
9. [Implementation Order (Updated)](#9-implementation-order-updated)
10. [Dependency Chain Analysis (Verified)](#10-dependency-chain-analysis-verified)
11. [Risk Assessment (Updated)](#11-risk-assessment-updated)
12. [Test Strategy (Updated)](#12-test-strategy-updated)
13. [Parallel Folder Streaming Architecture (NEW)](#13-parallel-folder-streaming-architecture-new)
14. [Acceptance Verification (Updated)](#14-acceptance-verification-updated)
15. [Research-Backed Solutions for 6 Remaining Issues (NEW)](#15-research-backed-solutions-for-6-remaining-issues-new)

---

## 1. Executive Summary

v1.6.5 delivers three interconnected features plus a performance fix:

1. **Ingestion Diagnostics Coverage Report** -- Two diagnostic report sections currently working (excluded tests + word coverage), one section missing (ignored files)
2. **Folder-Scoped Queries** -- `?scope=` parameter working on 2/18 query endpoints, discovery endpoint working
3. **Dual Coverage Metrics** -- Raw coverage working, effective coverage working with comment_word_count, but import_word_count hardcoded to 0
4. **Parallel Folder Streaming** (Phase 5) -- Thread-local parser fix to eliminate Mutex serialization in Rayon parallel path

### Live Metrics (from Parseltongue E2E run 2026-02-11)

| Metric | Value | Status |
|--------|-------|--------|
| Total code entities | 1,583 | VERIFIED |
| Total test entities excluded | 1,319 | VERIFIED |
| Total dependency edges | 10,305 | VERIFIED |
| Languages detected | 12 | VERIFIED |
| Word coverage files | 302 | VERIFIED |
| Avg raw coverage | 70.5% | VERIFIED |
| Avg effective coverage | 76.9% | VERIFIED |
| Files with comment_word_count > 0 | 175/302 | VERIFIED |
| Files with import_word_count > 0 | 0/302 | NOT IMPLEMENTED |
| Scope filtering on list-all | Working | VERIFIED |
| Scope filtering on fuzzy-search | Working | VERIFIED |
| Scope filtering on other 16 handlers | Not implemented | VERIFIED |
| Folder discovery endpoint | Working | VERIFIED |
| Diagnostics endpoint | Partial (2/3 sections) | VERIFIED |

---

## 2. Implementation Status Dashboard

### Phase 1: CozoDB Schema + Path Normalization -- DONE

| Item | Status | Verified Entity Key |
|------|--------|---------------------|
| CodeGraph schema with L1/L2 columns | DONE | `rust:method:create_schema:____crates_parseltongue_core_src_storage_cozo_client:T1617210344` |
| TestEntitiesExcluded relation | DONE | `rust:method:create_test_entities_excluded_schema:____crates_parseltongue_core_src_storage_cozo_client:T1759833328` |
| FileWordCoverage relation | DONE | `rust:method:create_file_word_coverage_schema:____crates_parseltongue_core_src_storage_cozo_client:T1873118821` |
| path_utils.rs module | DONE | `rust:mod:path_utils:____crates_parseltongue_core_src_storage_mod:T1817309875` |
| extract_subfolder_levels_from_path | DONE | Only visible as unresolved-reference (test-classified in entities.rs) |
| normalize_split_file_path | DONE | Only visible as unresolved-reference (test-classified) |
| `./` prefix strip in L1/L2 extraction | FIXED | L1 values are correct: `crates`, `test-fixtures`, `tests`, `.stable` |

### Phase 2: pt01 Ingestion Collection -- MOSTLY DONE

| Item | Status | Evidence |
|------|--------|----------|
| Test entity exclusion collection | DONE | 1,319 test entities in TestEntitiesExcluded |
| Word coverage computation | DONE | 302 files in FileWordCoverage |
| Comment word counting | DONE | 175/302 files have comment_word_count > 0 |
| Import word counting | NOT DONE | 0/302 files have import_word_count > 0 (hardcoded to 0) |
| L1/L2 population on CodeGraph | DONE | Folder discovery shows correct hierarchy |
| Batch inserts | DONE | `rust:method:insert_test_entities_excluded_batch:...T1748707063`, `rust:method:insert_file_word_coverage_batch:...T1814019910` |

### Phase 3: HTTP Endpoints -- PARTIALLY DONE

| Item | Status | Evidence |
|------|--------|----------|
| Diagnostics coverage handler | PARTIAL | `rust:fn:handle_ingestion_diagnostics_coverage_report:...T1776646120` -- Missing ignored_files section |
| Folder discovery handler | DONE | `rust:fn:handle_folder_structure_discovery_tree:...T1645203230` |
| Scope filter utilities | DONE | `rust:fn:parse_scope_build_filter_clause:...T1768007593` + 5 more functions |
| Scope on code-entities-list-all | DONE | Verified: `?scope=crates||parseltongue-core` returns 77 entities |
| Scope on code-entities-search-fuzzy | DONE | Verified: `?scope=crates||parseltongue-core` filters correctly |
| Scope on 16 other handlers | NOT DONE | Verified: only 2 callers of parse_scope_build_filter_clause |
| API docs updated | NOT DONE | /api-reference-documentation-help shows 22 endpoints, missing 2 new ones |
| Route registration | DONE | Both new endpoints respond with HTTP 200 |

### Phase 4: E2E Verification -- IN PROGRESS

| Item | Status |
|------|--------|
| Diagnostics endpoint returns 200 | DONE |
| Test entities section populated | DONE (1,319 entities) |
| Word coverage section populated | DONE (302 files) |
| Ignored files section populated | NOT DONE |
| Scope filtering E2E | PARTIAL (2/18 handlers) |

### Phase 5: Thread-Local Parser Parallelism -- NOT STARTED

| Item | Status |
|------|--------|
| Thread-local Isgl1KeyGeneratorImpl | NOT DONE |
| process_file_sync_with_local_generator | NOT DONE |
| Eliminate Mutex<Parser> contention | NOT DONE |
| Unwrap audit | NOT DONE |

---

## 3. Architecture Analysis (Verified)

### Current Architecture (Verified via Parseltongue 2026-02-11)

Entity counts by crate (from `?scope=` queries):

| Crate | CODE Entities | Handler Files |
|-------|---------------|---------------|
| parseltongue-core | 77 | N/A |
| pt01-folder-to-cozodb-streamer | 19 | N/A |
| pt08-http-code-query-server | 243 | 26 handler files |
| Total (all scopes) | 1,583 | -- |
| Unresolved references (L1=".") | 841 (792 unresolved + 49 external) | -- |

### Verified File Layout

```
crates/parseltongue-core/src/
    storage/
        cozo_client.rs             # CozoDbStorage: 45 methods verified
            create_schema()                              # T1617210344
            create_test_entities_excluded_schema()        # T1759833328 (v1.6.5)
            create_file_word_coverage_schema()             # T1873118821 (v1.6.5)
            insert_entities_batch()                        # T1630071988 (has L1/L2 population)
            insert_test_entities_excluded_batch()           # T1748707063 (v1.6.5)
            insert_file_word_coverage_batch()                # T1814019910 (v1.6.5)
            execute_query()                                 # T1641298641
            raw_query()                                     # T1651597764
            escape_for_cozo_string()                        # T1768181028
            entity_to_params()                              # T1695720697 (has L1/L2 handling)
        path_utils.rs              # Module exists (T1817309875), functions test-classified
        mod.rs                     # Exports cozo_client, path_utils
    query_json_graph_helpers.rs    # Graph traversal utilities
    query_json_graph_errors.rs     # Error types
    graph_analysis/mod.rs          # 10 analysis algorithm modules
    lib.rs                         # Module declarations

crates/pt01-folder-to-cozodb-streamer/src/
    lib.rs                         # Module declarations for:
        streamer                   # T1720556776 (CRITICAL: parallel ingestion)
        isgl1_generator            # T1756797584 (CRITICAL: parser mutexes)
        test_detector              # T1634183939
        cli                        # T1820666524
        errors                     # T1641114273
        file_watcher               # T1598757650
        external_dependency_handler # T1702642386
        lsp_client                 # T1669882274

crates/pt08-http-code-query-server/src/
    scope_filter_utilities_module.rs  # NOT inside http_endpoint_handler_modules/
        parse_scope_build_filter_clause()   # T1768007593
        validate_scope_exists_in_database() # T1688222808
        fetch_scope_suggestions_from_database() # T1593590416
        build_scope_existence_query()       # T1615502725
        escape_single_quotes()              # T1682526422
        extract_string_from_datavalue()     # T1823826471
    lib.rs                                  # Declares all modules
    http_endpoint_handler_modules/
        mod.rs                              # 26 handler module declarations
        ingestion_diagnostics_coverage_handler.rs  # v1.6.5 (13 entities)
        folder_structure_discovery_handler.rs       # v1.6.5 (7 entities)
        code_entities_list_all_handler.rs           # Has scope filtering (6 entities)
        code_entities_fuzzy_search_handler.rs       # Has scope filtering (7 entities)
        blast_radius_impact_handler.rs              # NO scope filtering (10 entities)
        [... 21 more handler files ...]
    route_definition_builder_module.rs
```

### Blast Radius From create_schema

Verified via `/blast-radius-impact-analysis?entity=rust:method:create_schema:____crates_parseltongue_core_src_storage_cozo_client:T1617210344&hops=2`:

The `create_schema` entity is the root of the schema dependency tree. Changes here are additive only (new columns added at the end of the CodeGraph schema, new sub-schema methods called after existing schema creation). This is safe because pt01 always creates a fresh timestamped workspace.

---

## 4. Verified Entities: What Exists Today

### parseltongue-core (77 entities)

**cozo_client.rs** -- Verified methods (key ones for v1.6.5):

| Method | Entity Key | Status |
|--------|------------|--------|
| `create_schema` | `rust:method:create_schema:...T1617210344` | Has L1/L2 columns + calls new schema methods |
| `create_test_entities_excluded_schema` | `rust:method:...:T1759833328` | DONE -- creates TestEntitiesExcluded relation |
| `create_file_word_coverage_schema` | `rust:method:...:T1873118821` | DONE -- creates FileWordCoverage relation |
| `insert_entities_batch` | `rust:method:...:T1630071988` | DONE -- populates L1/L2 via path_utils |
| `insert_test_entities_excluded_batch` | `rust:method:...:T1748707063` | DONE -- batch insert to TestEntitiesExcluded |
| `insert_file_word_coverage_batch` | `rust:method:...:T1814019910` | DONE -- batch insert to FileWordCoverage |

**Verified CodeGraph schema** (19 columns, from create_schema code):
```
:create CodeGraph {
    ISGL1_key: String =>
    Current_Code: String?,
    Future_Code: String?,
    interface_signature: String,
    TDD_Classification: String,
    lsp_meta_data: String?,
    current_ind: Bool,
    future_ind: Bool,
    Future_Action: String?,
    file_path: String,
    language: String,
    last_modified: String,
    entity_type: String,
    entity_class: String,
    birth_timestamp: Int?,
    content_hash: String?,
    semantic_path: String?,
    root_subfolder_L1: String,    -- v1.6.5
    root_subfolder_L2: String     -- v1.6.5
}
```

**Verified TestEntitiesExcluded schema** (from create_test_entities_excluded_schema code):
```
:create TestEntitiesExcluded {
    entity_name: String,
    folder_path: String,
    filename: String
    =>
    entity_class: String,
    language: String,
    line_start: Int,
    line_end: Int,
    detection_reason: String
}
```

**Verified FileWordCoverage schema** (from create_file_word_coverage_schema code):
```
:create FileWordCoverage {
    folder_path: String,
    filename: String
    =>
    language: String,
    source_word_count: Int,
    entity_word_count: Int,
    import_word_count: Int,
    comment_word_count: Int,
    raw_coverage_pct: Float,
    effective_coverage_pct: Float,
    entity_count: Int
}
```

### pt08-http-code-query-server (243 entities)

**scope_filter_utilities_module.rs** -- 6 entities (verified):

| Function | Entity Key | What It Does |
|----------|------------|--------------|
| `parse_scope_build_filter_clause` | `T1768007593` | Builds Datalog filter clause from `?scope=` value |
| `validate_scope_exists_in_database` | `T1688222808` | Validates scope exists, returns suggestions if not |
| `fetch_scope_suggestions_from_database` | `T1593590416` | Queries distinct L1/L2 combos, filters by starting letter |
| `build_scope_existence_query` | `T1615502725` | Builds Datalog existence check query |
| `escape_single_quotes` | `T1682526422` | Escapes `'` for CozoDB string literals |
| `extract_string_from_datavalue` | `T1823826471` | Extracts String from CozoDB DataValue |

**IMPORTANT: Actual file location**: `crates/pt08-http-code-query-server/src/scope_filter_utilities_module.rs`
(NOT inside `http_endpoint_handler_modules/` -- the old spec was wrong about this)

**ingestion_diagnostics_coverage_handler.rs** -- 13 entities (verified):

| Entity | Type | Purpose |
|--------|------|---------|
| `handle_ingestion_diagnostics_coverage_report` | fn | Main handler -- queries test entities + word coverage |
| `query_test_entities_excluded_from_database` | fn | Queries TestEntitiesExcluded relation |
| `query_word_coverage_from_database` | fn | Queries FileWordCoverage relation |
| `extract_string_from_datavalue` | fn | DataValue string extraction |
| `extract_i64_from_datavalue` | fn | DataValue i64 extraction |
| `extract_f64_from_datavalue` | fn | DataValue f64 extraction |
| + 7 structs | struct | Response/data payload types |

**folder_structure_discovery_handler.rs** -- 7 entities (verified):

| Entity | Type | Purpose |
|--------|------|---------|
| `handle_folder_structure_discovery_tree` | fn | Main handler -- queries L1/L2 tree |
| `query_folder_structure_from_database` | fn | Builds hierarchical L1/L2 structure from CodeGraph |
| `extract_string_from_datavalue` | fn | DataValue string extraction |
| + 4 structs | struct | Response/data payload types |

---

## 5. What Remains to Implement

### 5.1 import_word_count (Priority: HIGH)

**Problem**: All 302 files in FileWordCoverage have `import_word_count: 0`. The value is hardcoded to 0 during ingestion.

**What the PRD specifies**: During `execute_dependency_query()` in `query_extractor.rs`, accumulate byte ranges from `@dependency.*` captures. After the loop, compute `import_word_count` from deduplicated byte ranges.

**Implementation needed**:
- Modify `execute_dependency_query()` (in `crates/parseltongue-core/src/query_extractor.rs`) to:
  1. Track byte ranges when capture name starts with `@dependency`
  2. Deduplicate overlapping ranges
  3. Return `deduplicated_import_ranges: Vec<(usize, usize)>` alongside existing return values
- Modify the streamer's word coverage computation to use these ranges instead of hardcoded 0

**Impact on effective_coverage_pct**: Currently `effective_coverage_pct` is slightly higher than `raw_coverage_pct` (76.9% vs 70.5%) because only comment words are subtracted. With imports subtracted too, effective coverage will increase further for import-heavy files.

### 5.2 Scope Filtering on 16 Remaining Handlers (Priority: HIGH)

**Currently working** (verified via reverse-callers of `parse_scope_build_filter_clause`):
1. `code_entities_list_all_handler.rs` -- via `query_entities_with_filter_from_database`
2. `code_entities_fuzzy_search_handler.rs` -- via `search_entities_by_query_from_database`

**Remaining 16 handlers that need scope support**:

| # | Handler File | Main Function | Complexity |
|---|-------------|---------------|------------|
| 1 | `code_entity_detail_view_handler.rs` | `handle_code_entity_detail_view` | Low -- single entity lookup |
| 2 | `dependency_edges_list_handler.rs` | `handle_dependency_edges_list_all` | Medium -- needs DependencyEdges join |
| 3 | `reverse_callers_query_graph_handler.rs` | `handle_reverse_callers_query_graph` | Medium -- traversal from entity |
| 4 | `forward_callees_query_graph_handler.rs` | `handle_forward_callees_query_graph` | Medium -- traversal from entity |
| 5 | `blast_radius_impact_handler.rs` | `handle_blast_radius_impact_analysis` | High -- multi-hop BFS traversal |
| 6 | `circular_dependency_detection_handler.rs` | `handle_circular_dependency_detection_scan` | High -- full graph cycle detection |
| 7 | `complexity_hotspots_ranking_handler.rs` | `handle_complexity_hotspots_ranking_view` | Medium -- coupling scores |
| 8 | `semantic_cluster_grouping_handler.rs` | `handle_semantic_cluster_grouping_list` | Medium -- label propagation |
| 9 | `smart_context_token_budget_handler.rs` | `handle_smart_context_token_budget` | Medium -- BFS + token estimation |
| 10 | `strongly_connected_components_handler.rs` | `handle_strongly_connected_components_analysis` | High -- Tarjan SCC |
| 11 | `technical_debt_sqale_handler.rs` | `handle_technical_debt_sqale_scoring` | Medium -- debt scoring |
| 12 | `kcore_decomposition_layering_handler.rs` | `handle_kcore_decomposition_layering_analysis` | High -- k-core decomposition |
| 13 | `centrality_measures_entity_handler.rs` | `handle_centrality_measures_entity_ranking` | High -- PageRank/betweenness |
| 14 | `entropy_complexity_measurement_handler.rs` | `handle_entropy_complexity_measurement_scores` | Medium -- Shannon entropy |
| 15 | `coupling_cohesion_metrics_handler.rs` | `handle_coupling_cohesion_metrics_suite` | High -- CK metrics |
| 16 | `leiden_community_detection_handler.rs` | `handle_leiden_community_detection_clusters` | High -- community detection |

**Implementation pattern** (verified from working handlers):

For each handler:
1. Add `pub scope: Option<String>` to the handler's query params struct
2. In the database query function, call `parse_scope_build_filter_clause(&scope)` from `crate::scope_filter_utilities_module`
3. Append the scope clause to the Datalog `*CodeGraph{...}` atom -- the clause goes INSIDE the braces
4. Ensure `root_subfolder_L1` and `root_subfolder_L2` are listed as bound variables in the atom

**Special cases for graph-analysis handlers** (items 6, 10, 12, 13, 15, 16):
These handlers use `build_graph_from_database_edges()` to construct an in-memory graph, then run algorithms on it. Scope filtering must happen at the entity-fetch step, constraining which entities enter the graph. Edge queries may also need scoping (filter edges where from_key entity is in scope).

### 5.3 Ignored Files Report in Diagnostics Endpoint (Priority: MEDIUM)

**Current state**: The `handle_ingestion_diagnostics_coverage_report` handler only returns 2 sections:
- `test_entities_excluded` (from TestEntitiesExcluded)
- `word_count_coverage` (from FileWordCoverage)

**Missing**: The `ignored_files` section (files with unsupported extensions).

**Implementation**: The existing `ingestion_coverage_folder_handler.rs` has all the infrastructure:
- `derive_workspace_directory_from_database()` -- derives source directory from DB path
- `walk_directory_collect_files_and_errors()` -- walks filesystem
- `is_file_eligible_for_parsing()` -- checks Language::from_file_path()

The diagnostics handler needs to call these functions and group ignored files by extension.

### 5.4 API Documentation Update (Priority: LOW)

**Current state**: `/api-reference-documentation-help` lists 22 endpoints. Missing:
- `/ingestion-diagnostics-coverage-report`
- `/folder-structure-discovery-tree`

**Also missing**: `?scope=` parameter documentation on existing endpoints.

**File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/api_reference_documentation_handler.rs`

---

## 6. Known Bugs and Issues

### Bug 1: Scope Filter Clause Was Inside Atom Braces -- FIXED

**Original bug**: The scope filter clause was being generated with CozoDB atom binding syntax (`:`) inside the atom braces:
```datalog
-- WRONG (old bug): Binding inside atom
?[key] := *CodeGraph{ISGL1_key: key, root_subfolder_L1: 'crates'}
```

**Fix applied**: The clause now uses equality constraint (`=`) syntax and is placed inside the atom braces as a condition:
```datalog
-- CORRECT (current): Equality constraint inside atom
?[key] := *CodeGraph{ISGL1_key: key, file_path, entity_type, entity_class, language,
           root_subfolder_L1, root_subfolder_L2}, root_subfolder_L1 = 'crates'
```

**Verified**: The actual `parse_scope_build_filter_clause` function generates:
```rust
format!(", root_subfolder_L1 = '{}'", escape_single_quotes(l1))
```

**IMPORTANT CORRECTION**: The clause is actually appended OUTSIDE the `*CodeGraph{...}` atom as a separate condition in the rule body (after the atom, separated by `,`). The atom must bind `root_subfolder_L1` and `root_subfolder_L2` as variables, then the `=` filter is applied as a separate clause.

**Verified working Datalog** (from `query_entities_with_filter_from_database`):
```datalog
?[key, file_path, entity_type, entity_class, language] :=
    *CodeGraph{ISGL1_key: key, file_path, entity_type, entity_class, language,
               root_subfolder_L1, root_subfolder_L2},
    root_subfolder_L1 = 'crates', root_subfolder_L2 = 'parseltongue-core'
```

### Bug 2: `./` Prefix in File Paths -- PARTIALLY FIXED

**Original bug**: File paths like `./crates/parseltongue-core/src/lib.rs` had the `./` prefix, causing `extract_subfolder_levels_from_path` to return `L1="."` for ALL entities.

**Fix applied**: `extract_subfolder_levels_from_path` now strips the `./` prefix before extracting L1/L2. The L1/L2 columns are correct:
- `crates`, `test-fixtures`, `tests`, `.stable` (correct L1 values)
- `parseltongue-core`, `pt01-folder-to-cozodb-streamer`, etc. (correct L2 values)

**Remaining issue**: The `file_path` column in CodeGraph still contains the `./` prefix. Entity file_paths show `./crates/parseltongue-core/src/query_json_graph_errors.rs`. This is cosmetic but inconsistent with the L1/L2 values which have the prefix stripped.

### Bug 3: import_word_count Hardcoded to 0 -- NOT YET FIXED

**Current state**: All 302 files in FileWordCoverage have `import_word_count: 0`.

**Impact**: `effective_coverage_pct` is only slightly higher than `raw_coverage_pct` (76.9% vs 70.5%) because only comment words are subtracted. With imports subtracted, the gap would widen for import-heavy files, giving a more accurate picture of "real code coverage."

**Root cause**: The `@dependency.*` capture byte range accumulation has not been implemented in `execute_dependency_query()` in `query_extractor.rs`.

### Bug 4: L1="" for 13 Entities with Absolute Paths

**Current state**: 13 entities have absolute file paths (e.g., `/Users/amuldotexe/Desktop/...`), which causes `extract_subfolder_levels_from_path` to produce an empty L1.

**Root cause**: These entities appear to be duplicate ingestions where the file path was not normalized to relative form. They are visible in the folder discovery tree as `l1: ""` with `l2_children: ["Users"]`.

**Impact**: Minimal -- these 13 entities are outliers. They appear when Parseltongue ingests its own codebase and some files are referenced by absolute path.

### Bug 5: Coverage Exceeds 100% for Many Files

**Current state**: 71 files have `raw_coverage_pct > 100%`, 83 files have `effective_coverage_pct > 100%`.

**Root cause**: Overlapping entity content. When nested entities (e.g., a method inside a struct impl block) are counted, both the parent and child entity content overlap. The entity word counts are summed, but the overlapping regions are counted multiple times.

**Impact**: Expected behavior per PRD Section 4: "Coverage > 100%: Overlapping entity content (e.g., nested functions counted in both parent and child)." This is documented and acceptable.

---

## 7. CozoDB Datalog Patterns: Verified vs Incorrect

### VERIFIED WORKING Patterns

**1. Entity list with scope filter** (from `query_entities_with_filter_from_database`):
```datalog
?[key, file_path, entity_type, entity_class, language] :=
    *CodeGraph{ISGL1_key: key, file_path, entity_type, entity_class, language,
               root_subfolder_L1, root_subfolder_L2},
    root_subfolder_L1 = 'crates', root_subfolder_L2 = 'parseltongue-core'
```

**2. Folder discovery tree** (from `query_folder_structure_from_database`):
```datalog
?[l1, l2, entity] :=
    *CodeGraph{ISGL1_key: entity, root_subfolder_L1: l1, root_subfolder_L2: l2}
```
Note: This query returns individual rows (no aggregation). Grouping is done in Rust code using HashMap.

**3. Test entities excluded** (from `query_test_entities_excluded_from_database`):
```datalog
?[entity_name, folder_path, filename, entity_class, language, line_start, line_end, detection_reason] :=
    *TestEntitiesExcluded{entity_name, folder_path, filename, entity_class, language,
                          line_start, line_end, detection_reason}
```

**4. Word coverage** (from `query_word_coverage_from_database`):
```datalog
?[folder_path, filename, language, source_word_count, entity_word_count,
  import_word_count, comment_word_count, raw_coverage_pct, effective_coverage_pct, entity_count] :=
    *FileWordCoverage{folder_path, filename, language, source_word_count, entity_word_count,
                      import_word_count, comment_word_count, raw_coverage_pct,
                      effective_coverage_pct, entity_count}
```

**5. Scope existence check** (from `build_scope_existence_query`):
```datalog
?[entity] :=
    *CodeGraph{ISGL1_key: entity, root_subfolder_L1: l1, root_subfolder_L2: l2},
    l1 = 'crates', l2 = 'parseltongue-core'
```

**6. Scope suggestions** (from `fetch_scope_suggestions_from_database`):
```datalog
?[l1, l2] := *CodeGraph{root_subfolder_L1: l1, root_subfolder_L2: l2}
```

### INCORRECT Patterns (from old spec -- DO NOT USE)

**WRONG -- scope filter with `:` binding inside atom**:
```datalog
-- WRONG: This is Datalog variable binding, not filtering
?[key] := *CodeGraph{ISGL1_key: key, root_subfolder_L1: 'crates'}
```
CozoDB does NOT allow literal values in atom binding positions. You must bind to a variable first, then filter with `=`.

**WRONG -- scope filter appended inline**:
```datalog
-- WRONG: Old spec pattern
?[ISGL1_key, file_path] := *CodeGraph{ISGL1_key, file_path, root_subfolder_L1: 'src', root_subfolder_L2: 'core'}
```
Same issue: literal values cannot appear in atom binding positions in CozoDB Datalog.

**WRONG -- aggregation in discovery query**:
```datalog
-- WRONG: Old spec pattern
?[l1, l2, count(key)] := *CodeGraph{ISGL1_key: key, root_subfolder_L1: l1, root_subfolder_L2: l2}
```
This MAY work in CozoDB but the actual implementation does NOT use aggregation. It returns raw rows and groups in Rust. The aggregation version is not verified.

### Key Insight: How Scope Filtering Actually Works

The `parse_scope_build_filter_clause` function returns a string like:
```
, root_subfolder_L1 = 'crates', root_subfolder_L2 = 'parseltongue-core'
```

This string is appended AFTER the `*CodeGraph{...}` atom (outside the braces) but INSIDE the rule body. The atom must bind the variables `root_subfolder_L1` and `root_subfolder_L2`, and the `= 'value'` conditions act as filters.

The full format string in the code looks like:
```rust
format!(
    "?[key, file_path, ...] := *CodeGraph{{ISGL1_key: key, file_path, ..., root_subfolder_L1, root_subfolder_L2{}}}",
    scope_clause
)
```

Wait -- looking more carefully at the verified code, the scope_clause is actually placed inside the format string INSIDE the `{{}}` braces. Let me re-examine.

From the actual verified code in `query_entities_with_filter_from_database`:
```rust
let query = format!(
    "?[key, file_path, entity_type, entity_class, language] := \
     *CodeGraph{{ISGL1_key: key, file_path, entity_type, entity_class, language, \
     root_subfolder_L1, root_subfolder_L2{}}}",
    scope_clause
);
```

And `scope_clause` = `", root_subfolder_L1 = 'crates', root_subfolder_L2 = 'parseltongue-core'"`.

So the final query becomes:
```
?[key, file_path, entity_type, entity_class, language] :=
    *CodeGraph{ISGL1_key: key, file_path, entity_type, entity_class, language,
               root_subfolder_L1, root_subfolder_L2,
               root_subfolder_L1 = 'crates', root_subfolder_L2 = 'parseltongue-core'}
```

**This means the `=` filter IS inside the CozoDB atom braces.** CozoDB appears to accept `variable = 'literal'` constraints inside atoms. This is the verified working pattern.

---

## 8. Scope Filtering: Current State and Remaining Work

### Scope Utility Module Location

**Actual**: `crates/pt08-http-code-query-server/src/scope_filter_utilities_module.rs`

**NOT**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/scope_filter_utils.rs` (as stated in old spec)

**Module declaration**: `rust:mod:scope_filter_utilities_module:____crates_pt08_http_code_query_server_src_lib:T1843417011` in pt08's `lib.rs`

### Actual Function Names (vs Old Spec Names)

| Old Spec Name | Actual Name | Entity Key |
|---------------|-------------|------------|
| `build_scope_filter_clause` | `parse_scope_build_filter_clause` | T1768007593 |
| `parse_scope_filter_components` | Does not exist as separate function | -- |
| `validate_scope_against_database` | `validate_scope_exists_in_database` | T1688222808 |
| `escape_for_cozo_string` (core) | `escape_single_quotes` (in scope module) | T1682526422 |

### Handlers With Scope (2/18) -- Verified via Reverse Callers

Callers of `parse_scope_build_filter_clause` (4 edges, 2 unique handlers after dedup):

1. `rust:fn:query_entities_with_filter_from_database:____crates_..._code_entities_list_all_handler:T1703180460`
   - Source: `code_entities_list_all_handler.rs:115`

2. `rust:fn:search_entities_by_query_from_database:____crates_..._code_entities_fuzzy_search_handler:T1632088531`
   - Source: `code_entities_fuzzy_search_handler.rs:139`

### How Scope is Applied in Working Handlers

From `query_entities_with_filter_from_database` (verified code):

```rust
// Step 1: Get scope clause from shared utility
let scope_clause = parse_scope_build_filter_clause(scope_filter);

// Step 2: Build query with scope clause inside atom braces
let query = format!(
    "?[key, file_path, entity_type, entity_class, language] := \
     *CodeGraph{{ISGL1_key: key, file_path, entity_type, entity_class, language, \
     root_subfolder_L1, root_subfolder_L2{}}}",
    scope_clause
);
```

Key requirements for each handler:
1. Must bind `root_subfolder_L1` and `root_subfolder_L2` as variables in the atom
2. Scope clause is appended after those variable bindings, inside the atom braces
3. The `parse_scope_build_filter_clause` function handles None/empty gracefully (returns empty string)

### Remaining 16 Handlers: Implementation Approach

**Simple handlers** (direct CodeGraph queries, 2-3 lines change each):
- `code_entity_detail_view_handler.rs`
- `dependency_edges_list_handler.rs`
- `complexity_hotspots_ranking_handler.rs`

**Medium handlers** (need scope on entity fetch + edge traversal):
- `reverse_callers_query_graph_handler.rs`
- `forward_callees_query_graph_handler.rs`
- `blast_radius_impact_handler.rs`
- `semantic_cluster_grouping_handler.rs`
- `smart_context_token_budget_handler.rs`

**Complex handlers** (use `build_graph_from_database_edges()` -- need scope on both entity and edge queries):
- `circular_dependency_detection_handler.rs`
- `strongly_connected_components_handler.rs`
- `technical_debt_sqale_handler.rs`
- `kcore_decomposition_layering_handler.rs`
- `centrality_measures_entity_handler.rs`
- `entropy_complexity_measurement_handler.rs`
- `coupling_cohesion_metrics_handler.rs`
- `leiden_community_detection_handler.rs`

---

## 9. Implementation Order (Updated)

### Remaining Phase 2 Work: import_word_count

**Step 1**: Modify `execute_dependency_query()` in `crates/parseltongue-core/src/query_extractor.rs`
- During query execution, when a capture name starts with `@dependency`, accumulate `(start_byte, end_byte)` from the matched node
- After all captures processed, deduplicate overlapping byte ranges
- Return `import_word_count` computed from the deduplicated ranges: `source[start..end].split_whitespace().count()`

**Step 2**: Modify the streamer's file processing to use the new import_word_count instead of hardcoded 0

**Checkpoint**: Re-ingest Parseltongue codebase, verify `import_word_count > 0` for files with imports (most Rust files have `use` statements).

### Remaining Phase 3 Work: Scope on 16 Handlers + Ignored Files + API Docs

**Step 1**: Add scope support to each of the 16 remaining handlers (see section 8 for categorization)
- For each handler, add `scope: Option<String>` to its query params struct
- Call `parse_scope_build_filter_clause(&params.scope)` from `crate::scope_filter_utilities_module`
- Append clause to Datalog queries that touch CodeGraph

**Step 2**: Add ignored files section to diagnostics handler
- Import `derive_workspace_directory_from_database`, `walk_directory_collect_files_and_errors`, `is_file_eligible_for_parsing` from `ingestion_coverage_folder_handler`
- Walk filesystem, filter files where `Language::from_file_path()` returns None
- Group by extension, include in diagnostics response

**Step 3**: Update API documentation handler
- Add `/ingestion-diagnostics-coverage-report` and `/folder-structure-discovery-tree` to the category list in `build_api_documentation_categories`
- Add `?scope=` parameter description to all 18 query endpoints

**Checkpoint**: All 18 handlers accept `?scope=`, diagnostics has 3 sections, API docs list 24+ endpoints.

### Phase 5: Thread-Local Parser Parallelism (NEW)

See Section 13 for detailed architecture.

**Step 1**: Create thread-local `Isgl1KeyGeneratorImpl` instances for Rayon threads
**Step 2**: Add `process_file_sync_with_local_generator()` method
**Step 3**: Update `stream_directory_with_parallel_rayon()` to use thread-local generators
**Step 4**: Eliminate shared `Mutex<Parser>` contention
**Step 5**: Audit and remove dangerous `unwrap()` on mutex locks
**Step 6**: Benchmark: verify > 3x speedup on multi-core

---

## 10. Dependency Chain Analysis (Verified)

### Verified Dependency Graph

```
create_schema (T1617210344) -- DONE, has L1/L2 + calls sub-schemas
  |-- create_test_entities_excluded_schema (T1759833328) -- DONE
  |-- create_file_word_coverage_schema (T1873118821) -- DONE
  |
  |-- insert_entities_batch (T1630071988) -- DONE, populates L1/L2 via path_utils
  |     |-- entity_to_params (T1695720697) -- DONE, has L1/L2 handling
  |     \-- extract_subfolder_levels_from_path (in path_utils) -- DONE
  |
  |-- insert_test_entities_excluded_batch (T1748707063) -- DONE
  |-- insert_file_word_coverage_batch (T1814019910) -- DONE
  |
  |-- parse_scope_build_filter_clause (T1768007593) -- DONE
  |     |-- code_entities_list_all_handler -- DONE (calls via query_entities_with_filter)
  |     |-- code_entities_fuzzy_search_handler -- DONE (calls via search_entities_by_query)
  |     \-- [16 other handlers] -- NOT DONE
  |
  |-- validate_scope_exists_in_database (T1688222808) -- DONE
  |     |-- build_scope_existence_query (T1615502725) -- DONE
  |     \-- fetch_scope_suggestions_from_database (T1593590416) -- DONE
  |
  |-- handle_ingestion_diagnostics_coverage_report (T1776646120) -- PARTIAL
  |     |-- query_test_entities_excluded_from_database (T1869267552) -- DONE
  |     |-- query_word_coverage_from_database (T1891517826) -- DONE
  |     \-- [ignored files section] -- NOT DONE
  |
  \-- handle_folder_structure_discovery_tree (T1645203230) -- DONE
        \-- query_folder_structure_from_database (T1825619215) -- DONE
```

### Shared Utilities (Verified)

| Utility | Location | Callers |
|---------|----------|---------|
| `parse_scope_build_filter_clause` | `scope_filter_utilities_module.rs` | 2 handlers (need 18) |
| `validate_scope_exists_in_database` | Same | Called by handlers that validate scope |
| `escape_single_quotes` | Same | Called by scope filter builder |
| `derive_workspace_directory_from_database` | `ingestion_coverage_folder_handler.rs` | Coverage handler, needs diagnostics handler |
| `walk_directory_collect_files_and_errors` | Same | Coverage handler, needs diagnostics handler |
| `extract_subfolder_levels_from_path` | `path_utils.rs` (core) | `insert_entities_batch` |
| `normalize_split_file_path` | `path_utils.rs` (core) | Streamer (test exclusion + word coverage) |

---

## 11. Risk Assessment (Updated)

### Remaining High Risk

#### 1. Scope Filtering on Graph-Analysis Handlers

**Risk**: The 8 graph-analysis handlers (SCC, k-core, centrality, entropy, coupling, Leiden, technical debt, circular dependency) all use `build_graph_from_database_edges()` to construct in-memory graphs. Adding scope filtering requires:
- Filtering entities at the CodeGraph query level
- Potentially filtering edges where the source entity is in scope
- Ensuring graph algorithms work correctly on subgraphs (some algorithms assume connected graphs)

**Mitigation**:
- Start with entity-level filtering only (filter which entities enter the graph)
- Edge filtering follows naturally: only edges where both endpoints are in the entity set
- Test each algorithm with scoped subgraphs
- Some algorithms (SCC, k-core) work fine on subgraphs; others (community detection) may give different results

#### 2. import_word_count Implementation

**Risk**: Modifying `execute_dependency_query()` in `query_extractor.rs` touches the dependency extraction pipeline. The module is not directly visible in the code graph (classified as test), so verification must be done through E2E testing.

**Mitigation**:
- The change is additive: accumulate byte ranges alongside existing dependency extraction
- Byte range deduplication prevents overcounting
- E2E test: reingest Parseltongue, verify `import_word_count > 0` for files with `use` statements

### Remaining Medium Risk

#### 3. API Documentation Handler Update

**Risk**: `build_api_documentation_categories()` is a hardcoded function with category lists. Adding new endpoints and `?scope=` parameter descriptions requires careful editing.

**Mitigation**: Follow existing patterns exactly. The handler at `api_reference_documentation_handler.rs` has 7 entities including the builder function.

### Lowered Risk (From Original)

#### 4. Streamer.rs Modification -- RISK REDUCED

**Original risk**: CBO: 144, Health Grade: F.

**Current**: Phase 1 and Phase 2 modifications to streamer.rs are COMPLETE. Test entity exclusion collection and word coverage computation are working. The only remaining streamer change is for Phase 5 (thread-local parsers), which is a separate refactor.

#### 5. Schema Migration -- RISK ELIMINATED

All schema changes are DONE and verified working. CodeGraph has 19 columns, TestEntitiesExcluded and FileWordCoverage are populated with live data.

---

## 12. Test Strategy (Updated)

### Live Test Results (2026-02-11 E2E Run)

| Test | Result | Details |
|------|--------|---------|
| Total entities ingested | 1,583 CODE + 1,319 TEST = 2,902 | PASS |
| Folder discovery | Shows correct L1/L2 hierarchy | PASS |
| Scope crates||parseltongue-core | Returns 77 entities | PASS |
| Scope crates | Returns 345 entities | PASS |
| No scope | Returns 1,583 entities | PASS |
| Diagnostics test entities | 1,319 in TestEntitiesExcluded | PASS |
| Word coverage files | 302 files | PASS |
| comment_word_count > 0 | 175/302 files | PASS |
| import_word_count > 0 | 0/302 files | FAIL (not implemented) |
| effective >= raw | True for all 302 files | PASS |
| Coverage > 100% files | 71 raw, 83 effective | EXPECTED |
| L1/L2 values correct | crates, test-fixtures, tests, .stable | PASS |
| L1="." entities | 841 (unresolved + external refs) | EXPECTED |
| L1="" entities | 13 (absolute path anomaly) | KNOWN ISSUE |

### Tests Still Needed

#### For import_word_count Fix:
```rust
#[test]
fn test_import_word_count_rust_use_statements() {
    // Ingest a file with: use std::collections::HashMap;
    // Verify import_word_count >= 3 (use, std, collections, HashMap)
}

#[test]
fn test_import_word_count_python_imports() {
    // Ingest: from os.path import join
    // Verify import_word_count > 0
}

#[test]
fn test_effective_coverage_increases_with_imports() {
    // File with 100 source words, 20 import words, 50 entity words
    // raw = 50%, effective = 50/(100-20) = 62.5%
    // Verify effective > raw by meaningful margin
}
```

#### For Scope on Remaining 16 Handlers:
```rust
#[test]
fn test_blast_radius_with_scope_filter() {
    // Ingest multi-crate fixture
    // blast-radius without scope -> many entities
    // blast-radius with scope=crates||parseltongue-core -> fewer entities
    // All returned entities have L1=crates, L2=parseltongue-core
}

#[test]
fn test_circular_dependency_with_scope() {
    // Cycles may exist within a scope but not across scopes
    // Verify scoped cycle detection finds only in-scope cycles
}
```

#### For Ignored Files Section:
```rust
#[test]
fn test_diagnostics_ignored_files_section() {
    // Ingest a directory with .md, .json, .toml files alongside .rs
    // Diagnostics endpoint should list ignored files grouped by extension
    // Verify .md files appear in ignored list
}
```

---

## 13. Parallel Folder Streaming Architecture (NEW)

### Problem: Parser Mutex Serializes All Rayon Threads

`stream_directory_with_parallel_rayon()` in `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs` already uses Rayon `par_iter()` over discovered files. However, the `Isgl1KeyGeneratorImpl` (in `crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs`) contains:

```
parsers: HashMap<Language, Arc<Mutex<Parser>>>     // 13 language parsers behind mutexes
query_extractor: Mutex<QueryBasedExtractor>        // Single shared extractor behind mutex
```

Every Rayon thread must `parser_mutex.lock().unwrap()` to parse a file. This serializes the tree-sitter parsing stage, negating most of the Rayon parallelism.

### Verified Module Structure

From Parseltongue entity queries:
- `rust:mod:isgl1_generator:____crates_pt01_folder_to_cozodb_streamer_src_lib:T1756797584`
- `rust:mod:streamer:____crates_pt01_folder_to_cozodb_streamer_src_lib:T1720556776`

Both modules are declared in `crates/pt01-folder-to-cozodb-streamer/src/lib.rs`. The internal code of these modules is classified as TestImplementation by the test detector (likely due to the large number of `#[test]` functions), so individual functions are not visible as CODE entities.

### Current Two-Phase Architecture

The existing parallel path already works in two phases:
1. **Phase 1 (parallel)**: `par_iter()` over files, parse each, produce `(FileResult, Vec<CodeEntity>, Vec<DependencyEdge>)`
2. **Phase 2 (sequential)**: `collect()` results, `insert_entities_batch()`, `insert_edges_batch()`

The parallelism in Phase 1 is bottlenecked by the shared Mutex<Parser>.

### Fix: Thread-Local Parsers (~50 Lines)

```rust
// In streamer.rs or isgl1_generator.rs
thread_local! {
    static LOCAL_GENERATOR: RefCell<Isgl1KeyGeneratorImpl> =
        RefCell::new(Isgl1KeyGeneratorImpl::new());
}

// In the Rayon par_iter closure:
.par_iter()
.map(|file_path| {
    LOCAL_GENERATOR.with(|gen| {
        let gen = gen.borrow();
        self.process_file_sync_with_local_generator(file_path, &gen)
    })
})
```

### New Method Needed

`process_file_sync_with_local_generator()` -- A copy of the existing `process_file_sync_for_parallel()` that takes `&Isgl1KeyGeneratorImpl` as a parameter instead of using `self.key_generator`.

### Trade-offs

| Aspect | Current | After Fix |
|--------|---------|-----------|
| Mutex contention | High -- all threads serialize | Zero -- no shared state |
| Memory per thread | 1 parser set shared | 8 parser sets (8 threads) |
| Init cost | One-time ~25ms | One-time ~200ms (8 threads x 13 languages) |
| Parsing throughput | ~1x (serialized) | ~N x (N = core count) |
| Failure isolation | One thread panic poisons all | Thread-local: isolated |

### Unwrap Safety Audit

The `parser_mutex.lock().unwrap()` is the only production unwrap that poses real risk:
- If any Rayon thread panics while holding the lock, the mutex is poisoned
- All subsequent threads calling `.lock().unwrap()` will also panic (chain reaction)

The thread-local fix eliminates this entirely: no mutex, no lock, no poison, no unwrap.

**Existing defensive patterns in streamer.rs** (already correct):
- `if let Ok(mut stats) = self.stats.lock() { ... }` -- graceful skip
- `self.stats.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).clone()` -- recovers from poison
- `match self.query_extractor.lock() { Ok(...) => ..., Err(e) => eprintln!(...) }` -- handled

### Files to Modify

| File | Change |
|------|--------|
| `crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs` | Add `process_file_sync_with_local_generator()`, possibly `new_for_thread_local()` |
| `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs` | Update `stream_directory_with_parallel_rayon()` to use `thread_local!` generators |

### Implementation Steps

1. Create `Isgl1KeyGeneratorImpl::new_for_thread_local()` (or reuse existing `new()`)
2. Add `thread_local!` static in `streamer.rs`
3. Add `process_file_sync_with_local_generator(&self, file_path: &Path, gen: &Isgl1KeyGeneratorImpl)` method
4. Update `stream_directory_with_parallel_rayon()` to use thread-local generators in `par_iter()` closure
5. Remove or dead-code the `parser_mutex.lock().unwrap()` codepath
6. Write benchmark test: parallel ingestion of Parseltongue's own codebase, verify speedup > 3x on 4+ cores
7. Write robustness test: verify a single malformed file does not crash other threads

### Acceptance Criteria for Phase 5

- [ ] `stream_directory_with_parallel_rayon()` achieves > 3x speedup on 4-core machines vs sequential
- [ ] No shared `Mutex<Parser>` contention between Rayon threads during parsing
- [ ] Each Rayon thread owns its own tree-sitter `Parser` instances (thread-local)
- [ ] Parser initialization overhead < 500ms for 8 threads x 13 languages
- [ ] A single malformed file does not crash/poison other threads
- [ ] Zero production `unwrap()` calls on mutex locks in parallel codepath
- [ ] Entities and edges still batch-inserted sequentially after parallel parse (correctness preserved)
- [ ] `cargo test --all` passes with thread-local parsers (no regressions)
- [ ] Ingestion results identical between sequential and parallel modes

---

## 14. Acceptance Verification (Updated)

### Functional -- What Already Passes

- [x] `GET /ingestion-diagnostics-coverage-report` returns HTTP 200
- [x] Excluded test entities section lists all tests with folder/filename/language/reason (1,319 entities)
- [x] Word count coverage section shows dual metrics per file (302 files)
- [x] Comment word count populated from AST walk (175/302 files > 0)
- [x] `effective_coverage_pct >= raw_coverage_pct` for all files
- [x] `root_subfolder_L1` and `root_subfolder_L2` populated for every entity
- [x] `GET /folder-structure-discovery-tree` returns L1/L2 tree with counts
- [x] `?scope=crates` filters to L1 only (345 entities vs 1,583 total)
- [x] `?scope=crates||parseltongue-core` filters to L1+L2 (77 entities)
- [x] Absent `?scope=` returns full results (1,583 entities -- backward compatible)
- [x] Invalid scope returns error + suggestions (same starting letter)
- [x] Scope filtering at Datalog level (not Rust post-filter)
- [x] Root-level files: `L1: "."`, `L2: ""`

### Functional -- What Still Needs to Pass

- [ ] Ignored files section lists all unsupported extensions, grouped
- [ ] Import word count populated from `@dependency.*` captures (all 12 languages)
- [ ] All 18 query endpoints accept `?scope=` (only 2/18 done)
- [ ] Folder names with spaces work via URL encoding
- [ ] All stored paths are relative to workspace root (13 entities have absolute paths)
- [ ] Response includes summary aggregates including ignored files
- [ ] API documentation updated for new endpoints and scope parameter

### Non-Functional

- [x] Diagnostics endpoint responds quickly (< 5s observed)
- [x] `cargo test --all` passes with zero failures
- [ ] Zero TODOs/STUBs in committed code
- [ ] All function names follow 4WNC
- [ ] Pre-commit checklist passes

### Edge Cases

- [x] File with zero words returns `coverage_pct: 100.0`
- [x] Root-level files: `folder_path: ""`
- [x] Files with only comments: low raw, higher effective
- [x] Files where `import + comment >= source`: effective = 100% (saturating subtraction)
- [x] Coverage > 100% for nested entities (71 files raw, 83 files effective)
- [ ] Empty codebase (zero files) returns empty arrays (not errors) -- UNTESTED
- [ ] Binary files in ignored files list -- UNTESTED (ignored files section not implemented)

---

## 15. Research-Backed Solutions for 6 Remaining Issues (NEW)

After rubber-duck debugging against the live Parseltongue server (2026-02-11), 6 issues were identified. Three general-purpose research agents performed web research (industry surveys: JaCoCo, SonarQube, Semgrep, CodeQL, Neo4j, TigerGraph) and returned 3 possible solutions per issue with trade-offs. Below are all solutions analyzed, with the **recommended** approach marked.

---

### Issue 1: import_word_count Hardcoded to 0

**Impact**: 0/302 files have import_word_count > 0. effective_coverage_pct underestimates true effective coverage.

#### Solution A: Accumulate During execute_dependency_query()

| Aspect | Assessment |
|--------|-----------|
| Approach | During existing dependency extraction in `query_extractor.rs`, when capture name matches `@dependency.*`, accumulate `(start_byte, end_byte)` ranges. Compute word count from deduplicated ranges. |
| Accuracy | Highest -- uses exact same captures that identify dependencies |
| Complexity | Medium -- couples import counting to dependency pipeline |
| Performance | Best -- no extra tree-sitter pass |
| Testability | Hard -- must test through full dependency pipeline |
| Risk | Modifying hot path; test-classified module makes verification harder |

#### Solution B: Separate Tree-Sitter Query Pass -- RECOMMENDED

| Aspect | Assessment |
|--------|-----------|
| Approach | New `compute_import_word_count_safely()` function targeting import node types (not capture names). Follows existing `compute_comment_word_count_safely()` pattern. |
| Accuracy | High -- AST-level node matching across all 12 languages |
| Complexity | **Low** -- decoupled, follows existing pattern |
| Performance | Good -- extra tree-sitter query pass is ~1ms per file |
| Testability | **Easy** -- independent function, test with fixture files |
| Risk | Low -- additive, no modification to existing pipelines |

Import node types per language:

| Language | Node types |
|----------|-----------|
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

#### Solution C: Regex-Based Counting

| Aspect | Assessment |
|--------|-----------|
| Approach | Language-specific regex patterns (e.g., `^use\s`, `^import\s`, `^#include`) |
| Accuracy | Low -- misses multi-line imports, aliased requires, conditional imports |
| Complexity | Low |
| Performance | Fastest |
| Risk | **High** -- architectural inconsistency with AST-based approach; inaccurate for edge cases |

#### Decision: Solution B

Rationale: Follows the existing `compute_comment_word_count_safely()` pattern. Decoupled, testable, and accurate across all 12 languages. The extra tree-sitter pass costs ~1ms/file which is negligible vs the 100ms+ per-file parse time.

**Byte range deduplication**: Tree-sitter has NO built-in dedup across patterns. Industry standard is sort + merge overlapping intervals (O(n log n)).

**Word count metric**: Research confirmed no industry tool uses word count (JaCoCo: bytecode instructions, SonarQube: executable lines, Semgrep: parse coverage lines). Keep `split_whitespace().count()` -- formatting bias cancels in ratio metrics.

**Files**: `query_extractor.rs` (add function), `streamer.rs` (replace hardcoded 0)

---

### Issue 2: Scope Filtering on 16 Remaining Handlers

**Impact**: Only 2/18 handlers support `?scope=`. LLM agents cannot scope graph analysis queries.

#### Solution A: Modify Each Handler Individually

| Aspect | Assessment |
|--------|-----------|
| Approach | Add `scope: Option<String>` to each handler's params, call `parse_scope_build_filter_clause()`, append to queries |
| Lines | ~80 total (~5 per handler) |
| Maintainability | Low -- duplicates scope wiring in 16 places |
| Risk | Low per handler, but tedious and error-prone at scale |

#### Solution B: Axum Middleware/Extractor

| Aspect | Assessment |
|--------|-----------|
| Approach | Create `ScopeExtractor` that parses `?scope=` and validates. Handlers receive pre-parsed scope. |
| Lines | ~110 |
| Maintainability | Medium -- centralizes parsing but scope still must be threaded to Datalog queries |
| Risk | Medium -- over-engineered; Axum extractors don't help with query construction |

#### Solution C: Shared `build_scoped_graph_from_edges()` + Mechanical Pattern -- RECOMMENDED

| Aspect | Assessment |
|--------|-----------|
| Approach | Modify existing `build_graph_from_database_edges()` to accept optional scope. 8 complex handlers automatically get scope. Simple/medium handlers get the mechanical 3-line pattern. |
| Lines | ~90 new + removes ~150 duplication |
| Maintainability | **High** -- single point of scope logic for graph algorithms |
| Risk | Medium -- must verify all 8 algorithms produce correct results on subgraphs |

**Graph subgraph filtering best practice**: Neo4j and TigerGraph consensus is **filter-then-compute** (induced subgraph), NOT compute-then-filter. Scope filtering constrains which entities enter the graph; algorithms run on the subgraph.

**Handler categorization**:

| Category | Handlers | Pattern |
|----------|----------|---------|
| Simple (3) | detail_view, edges_list, complexity_hotspots | Add scope to params + Datalog query |
| Medium (5) | reverse_callers, forward_callees, blast_radius, semantic_cluster, smart_context | Scope on entity fetch + edge traversal |
| Complex (8) | circular_dep, SCC, tech_debt, kcore, centrality, entropy, coupling, leiden | Shared scoped graph builder |

#### Decision: Solution C+A hybrid

Rationale: 7 handlers share `build_graph_from_database_edges()`. Adding scope parameter there gives 8 handlers scope support with one change. Remaining 8 get the mechanical pattern.

**Files**: 16 handler files + shared graph builder location (TBD after codebase exploration)

---

### Issue 3: Ignored Files Section Missing from Diagnostics

**Impact**: Diagnostics endpoint returns 2/3 sections. LLM agents cannot see which files were skipped.

#### Solution A: Query-Time Filesystem Walk

| Aspect | Assessment |
|--------|-----------|
| Approach | At query time, walk workspace directory, filter by `Language::from_file_path() == None` |
| Lines | ~50 |
| Consistency | **Poor** -- other sections (test entities, word coverage) use ingestion-time DB storage |
| Reliability | **Fragile** -- fails if workspace directory moves or changes after ingestion |
| Precedent | Semgrep prints ignored files at scan time (similar approach) |

#### Solution B: Ingestion-Time DB Storage -- RECOMMENDED

| Aspect | Assessment |
|--------|-----------|
| Approach | New `IgnoredFiles` CozoDB relation. Collect ignored files during pt01 ingestion. Query from DB at endpoint time. |
| Lines | ~120 (schema + streamer + handler) |
| Consistency | **High** -- matches TestEntitiesExcluded and FileWordCoverage pattern |
| Reliability | **High** -- data persists with the analysis database |
| Precedent | **CodeQL** stores diagnostics at analysis time (validated by research) |

Schema:
```
:create IgnoredFiles {
    folder_path: String, filename: String =>
    extension: String, reason: String
}
```

#### Solution C: Hybrid Counts + Walk

| Aspect | Assessment |
|--------|-----------|
| Approach | Store aggregate counts at ingestion time, filesystem walk for file-level detail at query time |
| Lines | ~150 |
| Complexity | Over-engineered -- two data sources for one section |

#### Decision: Solution B

Rationale: Consistent with the ingestion-time storage pattern used by TestEntitiesExcluded and FileWordCoverage. CodeQL precedent validates this approach. Data survives workspace directory moves.

**Files**: `cozo_client.rs` (schema + batch insert), `entities.rs` (IgnoredFileRow), `streamer.rs` (collect during walk), `ingestion_diagnostics_coverage_handler.rs` (query + render)

---

### Issue 4: PRD Documentation Drift

**Impact**: PRD names don't match actual function names in code. Misleading for new contributors.

#### Solution A: Update PRD to Match Code

| Aspect | Assessment |
|--------|-----------|
| Approach | Find-and-replace PRD function names with actual names |
| Risk | PRD has extensive cross-references; risk of incomplete updates |

#### Solution B: Update Code to Match PRD

| Aspect | Assessment |
|--------|-----------|
| Approach | Rename functions in code to match PRD names |
| Risk | **High** -- breaks existing tests, handler registrations, import paths |

#### Solution C: Add Corrections Section to PRD -- RECOMMENDED

| Aspect | Assessment |
|--------|-----------|
| Approach | Add "Corrections and Errata" section documenting actual vs PRD names |
| Risk | Low -- additive, doesn't break anything |

#### Decision: Solution C

Rationale: Don't rename working functions. Add errata section with:
- Function name mapping (PRD name -> actual name)
- Scope filter utility actual location
- CozoDB Datalog syntax correction
- CodeGraph column count correction (19, not 17)

**Files**: `docs/PRD-v165.md`

---

### Issue 5: API Documentation Not Updated

**Impact**: `/api-reference-documentation-help` lists 22 endpoints, missing 2 new v1.6.5 endpoints and `?scope=` parameter docs.

#### Solution: Mechanical Update

No alternatives needed -- this is a straightforward addition to the hardcoded documentation builder.

1. Add `/ingestion-diagnostics-coverage-report` to Diagnostics category
2. Add `/folder-structure-discovery-tree` to Navigation category
3. Add `?scope=` parameter description to all 18 query endpoints
4. Document `?section=` parameter on diagnostics endpoint

**Files**: `api_reference_documentation_handler.rs`

---

### Issue 6: Diagnostics Endpoint Token Efficiency

**Impact**: Full diagnostics response could be 55K+ tokens (1,319 test entities + 302 coverage files + ignored files). LLM agents waste context window on unused sections.

#### Solution A: Keep All-in-One

| Aspect | Assessment |
|--------|-----------|
| Approach | Return everything always |
| Token efficiency | **Poor** -- 55K+ tokens even when agent needs one section |
| Simplicity | High |

#### Solution B: Three Separate Endpoints

| Aspect | Assessment |
|--------|-----------|
| Approach | `/diagnostics-test-entities`, `/diagnostics-word-coverage`, `/diagnostics-ignored-files` |
| Token efficiency | **Best** -- each endpoint returns only its section |
| Endpoint proliferation | **High** -- adds 3 endpoints, violates single-report design |

#### Solution C: `?section=` Query Parameter -- RECOMMENDED

| Aspect | Assessment |
|--------|-----------|
| Approach | `?section=test_entities|word_coverage|ignored_files|summary`. No param = return all (backward compatible). |
| Token efficiency | Good -- agent requests only what it needs |
| Backward compatibility | **Full** -- absent parameter returns complete report |
| Complexity | Low (~20 lines) |

#### Decision: Solution C

Rationale: Balances token efficiency with clean API design. Backward compatible. `summary` section returns only aggregates (counts, averages) without file-level detail -- ideal for quick status checks.

**Files**: `ingestion_diagnostics_coverage_handler.rs`

---

### Implementation Waves

| Wave | Issues | Priority | Est. Lines |
|------|--------|----------|-----------|
| **Wave 1: Ship-Blocking** | Issue 1 (import_word_count) + Issue 3 (ignored files) | HIGH | ~155 |
| **Wave 2: Scope Expansion** | Issue 2 (scope on 16 handlers) | HIGH | ~90 |
| **Wave 3: Polish** | Issue 6 (?section=) + Issue 5 (API docs) + Issue 4 (PRD errata) | MEDIUM | ~90 |

### Verification Per Wave

**Wave 1**: Re-ingest codebase, verify `import_word_count > 0` for files with imports, verify `ignored_files.total_count > 0` in diagnostics.

**Wave 2**: Query `blast-radius-impact-analysis?entity=X&scope=crates||parseltongue-core`, verify all returned entities are in scope.

**Wave 3**: Query `ingestion-diagnostics-coverage-report?section=summary`, verify only aggregates returned. Verify API docs list 24+ endpoints.

---

## Summary of Changes from Original Spec

### Corrections Made

1. **Scope filter utility file location**: Changed from `http_endpoint_handler_modules/scope_filter_utils.rs` to actual `scope_filter_utilities_module.rs` at `pt08/src/` level
2. **Function names corrected**: `build_scope_filter_clause` -> `parse_scope_build_filter_clause`, `validate_scope_against_database` -> `validate_scope_exists_in_database`, `parse_scope_filter_components` -> does not exist
3. **Datalog syntax corrected**: Old spec showed `: 'value'` binding syntax; actual uses `= 'value'` equality syntax inside the atom braces
4. **Scope filtering status corrected**: Old spec implied all 18 handlers ready; actually only 2/18 done
5. **Diagnostics endpoint corrected**: Old spec implied 3 sections; actually only 2/3 done (missing ignored files)
6. **CodeGraph schema column count corrected**: Old spec said 17; actual is 19 (already includes L1/L2)
7. **create_schema call pattern corrected**: Old spec said "calls 0 sub-schemas"; actual already calls both new schema methods

### New Sections Added

8. **Section 2: Implementation Status Dashboard** -- Live metrics and per-phase status
9. **Section 6: Known Bugs and Issues** -- 5 documented bugs with root causes
10. **Section 7: CozoDB Datalog Patterns** -- Verified working vs incorrect patterns
11. **Section 8: Scope Filtering Current State** -- Detailed analysis of 2/18 handlers done
12. **Section 13: Parallel Folder Streaming Architecture** -- Full Phase 5 specification
13. **Section 15: Research-Backed Solutions for 6 Remaining Issues** -- 3 solutions per issue with trade-offs, industry research (JaCoCo, SonarQube, CodeQL, Neo4j, TigerGraph), recommended approach per issue, 3-wave implementation order

### Entity Keys Verified Against Parseltongue

All entity keys in this document were verified via `curl http://localhost:7777/code-entity-detail-view?key=...` and `curl http://localhost:7777/code-entities-search-fuzzy?q=...` on 2026-02-11.

---

## File Inventory

### Files Already Created (v1.6.5)

| File | Purpose | Entities |
|------|---------|----------|
| `crates/parseltongue-core/src/storage/path_utils.rs` | Path normalization + L1/L2 extraction | Module visible, functions test-classified |
| `crates/pt08-http-code-query-server/src/scope_filter_utilities_module.rs` | Scope filter building, validation, suggestions | 6 CODE entities |
| `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_diagnostics_coverage_handler.rs` | Diagnostics report handler (2/3 sections) | 13 CODE entities |
| `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/folder_structure_discovery_handler.rs` | Folder L1/L2 discovery | 7 CODE entities |

### Files Already Modified (v1.6.5)

| File | Modification |
|------|-------------|
| `crates/parseltongue-core/src/storage/cozo_client.rs` | Added L1/L2 to CodeGraph, 2 new schemas, 2 new batch inserts |
| `crates/parseltongue-core/src/entities.rs` | Added ExcludedTestEntity, FileWordCoverageRow structs |
| `crates/parseltongue-core/src/storage/mod.rs` | Added `pub mod path_utils;` |
| `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs` | Test entity collection, word coverage computation, batch inserts |
| `crates/pt08-http-code-query-server/src/lib.rs` | Added `pub mod scope_filter_utilities_module;` |
| `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/mod.rs` | Added 2 new handler modules |
| `crates/pt08-http-code-query-server/src/route_definition_builder_module.rs` | Added 2 new routes |
| `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/code_entities_list_all_handler.rs` | Added scope parameter + filtering |
| `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/code_entities_fuzzy_search_handler.rs` | Added scope parameter + filtering |

### Files Still Needing Modification

| File | Wave | Modification Needed | Solution Ref |
|------|------|-------------------|-------------|
| `crates/parseltongue-core/src/query_extractor.rs` | 1 | ADD `compute_import_word_count_safely()` -- separate tree-sitter query pass | Issue 1, Sol B |
| `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs` | 1 | USE real import_word_count + collect ignored files during walk | Issue 1+3, Sol B |
| `crates/parseltongue-core/src/storage/cozo_client.rs` | 1 | ADD IgnoredFiles schema + `insert_ignored_files_batch()` | Issue 3, Sol B |
| `crates/parseltongue-core/src/entities.rs` | 1 | ADD `IgnoredFileRow` struct | Issue 3, Sol B |
| `ingestion_diagnostics_coverage_handler.rs` | 1,3 | ADD ignored_files section + `?section=` query parameter | Issue 3+6, Sol B+C |
| 16 handler files in pt08 | 2 | ADD `?scope=` parameter (mechanical pattern + shared graph builder) | Issue 2, Sol C+A |
| Shared graph builder (location TBD) | 2 | MODIFY `build_graph_from_database_edges()` to accept optional scope | Issue 2, Sol C |
| `api_reference_documentation_handler.rs` | 3 | ADD 2 new endpoints + `?scope=` + `?section=` param docs | Issue 5 |
| `docs/PRD-v165.md` | 3 | ADD "Corrections and Errata" section | Issue 4, Sol C |
| `crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs` | 5 | Thread-local parser fix (Phase 5 -- separate from issues above) | PRD Section 13 |

### Estimated Remaining Effort (Research-Backed)

| Wave | Task | Est. Lines | Complexity | Solution |
|------|------|-----------|------------|----------|
| 1 | import_word_count (separate query pass) | ~30 in query_extractor + ~5 in streamer | Medium | Issue 1, Sol B |
| 1 | IgnoredFiles schema + ingestion collection | ~50 schema + ~30 streamer | Low-Medium | Issue 3, Sol B |
| 1 | Ignored files in diagnostics handler | ~40 handler | Low | Issue 3, Sol B |
| 2 | Scope on 8 simple/medium handlers | ~5-10 lines each = ~65 | Low | Issue 2, Sol A |
| 2 | Shared scoped graph builder for 8 complex handlers | ~30 new, removes ~150 duplication | Medium | Issue 2, Sol C |
| 3 | `?section=` parameter on diagnostics | ~20 | Low | Issue 6, Sol C |
| 3 | API documentation update | ~40 | Low | Issue 5 |
| 3 | PRD errata section | ~30 | Low | Issue 4, Sol C |
| 5 | Thread-local parsers (Phase 5) | ~50 | Medium | PRD Section 13 |
| | **Total remaining** | **~335 lines** | |

---

*Verified by Parseltongue code graph analysis on 2026-02-11*
*Research agents completed 2026-02-11: web research covering JaCoCo, SonarQube, Semgrep, CodeQL, Neo4j, TigerGraph*
*Database: rocksdb:parseltongue20260211193602/analysis.db*
*Server: http://localhost:7777 (uptime: 1580s at time of verification)*
