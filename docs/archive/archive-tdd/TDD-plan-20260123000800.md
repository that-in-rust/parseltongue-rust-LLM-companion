# TDD Progress Journal: Diff Visualization System

> **Document Type**: TDD Progress Tracking (Source of Truth)
> **Created**: 2026-01-23 00:08:00
> **Last Updated**: 2026-01-23 (PROJECT COMPLETE)
> **Project**: Parseltongue Diff Visualization System
> **Location**: `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/`

---

## PROJECT COMPLETE - ALL 7 PHASES GREEN

### Final Status: ALL PHASES COMPLETE

**ALL 7 PHASES COMPLETE**: The entire Diff Visualization System has been implemented following strict TDD methodology.

---

## Final Test Counts (Verified 2026-01-23)

| Crate | Unit Tests | Integration Tests | Doc Tests | Total |
|-------|------------|-------------------|-----------|-------|
| parseltongue (CLI) | 5 | - | - | **5** |
| parseltongue-core | 130 | - | 26 | **156** |
| pt01-folder-to-cozodb-streamer | 13 | - | - | **13** |
| pt08-http-code-query-server | 48 | 21 | 1 | **70** |
| **GRAND TOTAL** | **196** | **21** | **27** | **244** |

All tests passing. Zero failures. Zero ignored (except 11 doc tests requiring filesystem).

---

## What "DIFF IS THE PRODUCT" Delivers

### Core Value Proposition

The Diff Visualization System transforms Parseltongue from a static code analysis tool into a **change impact analysis platform**. Users can now:

1. **Compare Snapshots**: Take two database snapshots (before/after code changes) and instantly understand what changed
2. **See Blast Radius**: Understand which entities are affected by changes via BFS graph traversal
3. **Visualize Changes**: Get visualization-ready data for 3D graph rendering (Three.js compatible)
4. **CLI or API Access**: Use via command line (`parseltongue diff`) or HTTP endpoint

### Key Capabilities

| Feature | Description | Use Case |
|---------|-------------|----------|
| Key Normalization | Strips line numbers from entity keys for stable identity | Prevents false positives when code moves within a file |
| Entity Diffing | Detects Added/Removed/Modified/Moved/Relocated | Understand what changed between versions |
| Edge Diffing | Tracks dependency relationship changes | See when functions start/stop calling each other |
| Blast Radius | BFS traversal with configurable depth | Answer "what else is affected by this change?" |
| Cycle Detection | Prevents infinite loops in circular graphs | Robust handling of real-world codebases |
| External Entity Handling | Recognizes `unknown:0-0` pattern | Stops traversal at graph boundaries |

### CLI Usage

```bash
# Basic diff between two snapshots
parseltongue diff --base rocksdb:before.db --live rocksdb:after.db

# JSON output with custom blast radius depth
parseltongue diff --base rocksdb:v1.db --live rocksdb:v2.db --json --max-hops 5
```

### HTTP API Usage

```bash
# POST request to compare snapshots
curl -X POST http://localhost:7777/diff-analysis-compare-snapshots \
  -H "Content-Type: application/json" \
  -d '{
    "base_db": "rocksdb:path/to/before.db",
    "live_db": "rocksdb:path/to/after.db"
  }'

# With custom max_hops
curl -X POST "http://localhost:7777/diff-analysis-compare-snapshots?max_hops=5" \
  -H "Content-Type: application/json" \
  -d '{"base_db": "rocksdb:before.db", "live_db": "rocksdb:after.db"}'
```

---

## 3-Agent TDD Pipeline Documentation

### Pipeline Architecture

The implementation was completed using a 3-agent collaborative TDD pipeline:

```
+-------------------------+     +----------------------+     +------------------+
| executable-specs-mindset|---->| local-exec-specs     |---->| rust-coder-01    |
| (Product Vision)        |     | (Test Specification) |     | (Implementation) |
+-------------------------+     +----------------------+     +------------------+
         |                              |                            |
         v                              v                            v
   Requirements               Executable Specs             Passing Tests
   Design Decisions           Test Templates               Working Code
   API Contracts              Acceptance Criteria          Integration
```

### Agent Responsibilities

#### Agent 1: executable-specs-mindset (Product Vision)
- **Role**: Product architect and requirements analyst
- **Outputs**:
  - `TDD-plan-20260123000800.md` (this document)
  - `ADR_001_KEY_NORMALIZATION.md`
  - `04_DATA_STRUCTURES.md`
  - `07_RUST_INTERFACE_DEFINITIONS.md`
- **Key Decisions**:
  - Key normalization as critical foundation
  - CLI-first approach before HTTP
  - 4-word naming convention enforcement
  - Performance SLOs (10K keys < 100ms, etc.)

#### Agent 2: local-exec-specs (Test Specification)
- **Role**: Executable specification writer
- **Outputs**:
  - `05_EXECUTABLE_SPECS_DIFF_SYSTEM.md`
  - `06_TEST_STRATEGY_AND_FIXTURES.md`
  - `docs/EXECUTABLE_SPECS_diff_command.md`
  - `docs/specs/REQ-HTTP-DIFF-ANALYSIS-COMPARE-SNAPSHOTS.md`
- **Key Contributions**:
  - 30 test stub definitions
  - GIVEN/WHEN/THEN format for all specs
  - Test fixture data from real API responses
  - Performance contract specifications

#### Agent 3: rust-coder-01 (Implementation)
- **Role**: Rust implementation specialist
- **Outputs**:
  - `crates/parseltongue-core/src/diff/` (all implementation files)
  - `crates/parseltongue/src/commands/diff_command_execution_module.rs`
  - `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/diff_analysis_compare_handler.rs`
- **Key Contributions**:
  - All trait implementations
  - Unit and integration tests
  - CLI command integration
  - HTTP endpoint handler

### TDD Cycle Execution

Each phase followed strict RED -> GREEN -> REFACTOR:

1. **RED**: Agent 2 writes failing test spec
2. **GREEN**: Agent 3 implements minimal code to pass
3. **REFACTOR**: Agent 3 cleans up while maintaining green
4. **VALIDATE**: Agent 1 reviews against requirements

### Context Retention Strategy

The TDD Context Retainer role was critical for:
- Maintaining state across session interruptions
- Tracking test counts and phase status
- Documenting decisions and their rationale
- Enabling seamless handoffs between agents

---

---

## Part 1: Design Documentation Inventory

The following design documents have been completed and validated:

| Document | Purpose | Status |
|----------|---------|--------|
| `UNIFIED_SERVER_DESIGN_RESEARCH_20260122233900.md` | Core design decisions | Complete |
| `VISUALIZATION_RESEARCH_20260122233900.md` | 3D rendering approach (Three.js) | Complete |
| `ARCHITECTURE_API_GROUNDED_20260122233900.md` | API-validated architecture | Complete |
| `RUBBER_DUCK_DEBUG_REPORT_20260122235000.md` | API validation findings | Complete |
| `GAP_ANALYSIS_METHODOLOGY_EVOLUTION_20260122.md` | 12 gaps identified | Complete |
| `ADR_001_KEY_NORMALIZATION.md` | Critical: stable identity extraction | Complete |
| `04_DATA_STRUCTURES.md` | TypeScript + Rust type definitions | Complete |
| `05_EXECUTABLE_SPECS_DIFF_SYSTEM.md` | 18 executable specifications | Complete |
| `06_TEST_STRATEGY_AND_FIXTURES.md` | Test fixtures using real API data | Complete |
| `07_RUST_INTERFACE_DEFINITIONS.md` | Traits + 30 test stubs | Complete |

---

## Part 2: Tests Written (30 Total)

### 2.1 KeyNormalizerTrait Tests (8 tests)

| Test Name | Status | Target Method | Description |
|-----------|--------|---------------|-------------|
| `test_extract_stable_identity_basic_function` | STUB | `extract_stable_identity_from_key` | Basic function key parsing |
| `test_extract_stable_identity_external_entity` | STUB | `extract_stable_identity_from_key` | Handle `unknown:0-0` pattern |
| `test_extract_stable_identity_method_key` | STUB | `extract_stable_identity_from_key` | Method entity key parsing |
| `test_extract_stable_identity_invalid_format` | STUB | `extract_stable_identity_from_key` | Error handling for malformed keys |
| `test_check_keys_match_same_entity_different_lines` | STUB | `check_keys_match_stable_identity` | Same entity, different line numbers |
| `test_check_keys_match_different_entities` | STUB | `check_keys_match_stable_identity` | Different entities comparison |
| `test_check_key_is_external_entity_true` | STUB | `check_key_is_external_entity` | Detect external entity |
| `test_check_key_is_external_entity_false` | STUB | `check_key_is_external_entity` | Detect local entity |

### 2.2 EntityDifferTrait Tests (13 tests)

| Test Name | Status | Target Method | Description |
|-----------|--------|---------------|-------------|
| `test_compute_diff_empty_both_sets` | STUB | `compute_entity_diff_result` | Empty inputs handling |
| `test_compute_diff_added_entity` | STUB | `compute_entity_diff_result` | Detect added entity |
| `test_compute_diff_removed_entity` | STUB | `compute_entity_diff_result` | Detect removed entity |
| `test_compute_diff_moved_entity` | STUB | `compute_entity_diff_result` | Detect moved entity (line shift) |
| `test_compute_diff_unchanged_entity` | STUB | `compute_entity_diff_result` | Unchanged entity not in diff |
| `test_compute_diff_mixed_changes` | STUB | `compute_entity_diff_result` | Multiple change types |
| `test_classify_added` | STUB | `classify_single_entity_change` | Classification: Added |
| `test_classify_removed` | STUB | `classify_single_entity_change` | Classification: Removed |
| `test_classify_moved_same_file` | STUB | `classify_single_entity_change` | Classification: Moved |
| `test_classify_relocated_different_file` | STUB | `classify_single_entity_change` | Classification: Relocated |
| `test_calculate_lines_shifted_positive` | STUB | `calculate_lines_shifted_count` | Lines shifted down |
| `test_calculate_lines_shifted_negative` | STUB | `calculate_lines_shifted_count` | Lines shifted up |
| `test_calculate_lines_shifted_external_entity` | STUB | `calculate_lines_shifted_count` | External entity handling |

### 2.3 BlastRadiusCalculatorTrait Tests (5 tests)

| Test Name | Status | Target Method | Description |
|-----------|--------|---------------|-------------|
| `test_calculate_neighbors_empty_changes` | STUB | `calculate_affected_neighbors_set` | No changes case |
| `test_calculate_neighbors_single_hop` | STUB | `calculate_affected_neighbors_set` | Single hop neighbor |
| `test_calculate_neighbors_reverse_edge` | STUB | `calculate_affected_neighbors_set` | Caller as neighbor |
| `test_calculate_neighbors_excludes_changed` | STUB | `calculate_affected_neighbors_set` | Changed entities excluded |
| `test_calculate_neighbors_deduplicates` | STUB | `calculate_affected_neighbors_set` | No duplicate neighbors |

### 2.4 DiffVisualizationTransformerTrait Tests (4 tests)

| Test Name | Status | Target Method | Description |
|-----------|--------|---------------|-------------|
| `test_transform_added_entity_status` | STUB | `transform_diff_to_visualization_nodes` | Added status assignment |
| `test_transform_neighbor_entity_status` | STUB | `transform_diff_to_visualization_nodes` | Neighbor status assignment |
| `test_transform_ambient_entity_status` | STUB | `transform_diff_to_visualization_nodes` | Ambient status assignment |
| `test_transform_external_entity_marked` | STUB | `transform_diff_to_visualization_nodes` | External entity flag |

### 2.5 Performance Contract Tests (3 tests)

| Test Name | Status | SLO | Description |
|-----------|--------|-----|-------------|
| `test_key_normalization_performance_contract` | STUB | 10,000 keys < 100ms | Key parsing performance |
| `test_diff_computation_performance_contract` | STUB | 1,000 entities < 500ms | Diff computation performance |
| `test_blast_radius_performance_contract` | STUB | 100 changed, 10K edges < 200ms | Blast radius performance |

---

## Part 3: Implementation Roadmap

### Phase 1: Core Key Normalization (Priority: CRITICAL)

**Rationale**: All diff operations depend on stable identity extraction. This MUST be implemented first.

```
Week 1, Days 1-2
================
Module: crates/parseltongue-core/src/diff/key_normalizer_impl.rs

1. Create module structure
2. Implement NormalizedEntityKeyData struct
3. Implement LineRangeData struct
4. Implement KeyNormalizationErrorType enum
5. Implement DefaultKeyNormalizerImpl
6. Pass all 8 KeyNormalizerTrait tests

Files to Create:
- crates/parseltongue-core/src/diff/mod.rs
- crates/parseltongue-core/src/diff/types.rs
- crates/parseltongue-core/src/diff/key_normalizer_impl.rs
```

**Test Order**:
1. `test_extract_stable_identity_basic_function` (simplest case)
2. `test_extract_stable_identity_method_key` (method variant)
3. `test_extract_stable_identity_external_entity` (edge case)
4. `test_extract_stable_identity_invalid_format` (error handling)
5. `test_check_keys_match_same_entity_different_lines` (matching logic)
6. `test_check_keys_match_different_entities` (non-matching)
7. `test_check_key_is_external_entity_true` (external detection)
8. `test_check_key_is_external_entity_false` (local detection)

### Phase 2: Entity Differ Implementation (Priority: HIGH)

```
Week 1, Days 3-5
================
Module: crates/parseltongue-core/src/diff/entity_differ_impl.rs

1. Implement EntityChangeTypeClassification enum
2. Implement DiffResultDataPayload struct
3. Implement DiffSummaryDataPayload struct
4. Implement EntityChangeDataItem struct
5. Implement DefaultEntityDifferImpl
6. Pass all 13 EntityDifferTrait tests

Dependencies:
- Requires Phase 1 complete (KeyNormalizerTrait)
```

**Test Order**:
1. `test_compute_diff_empty_both_sets` (base case)
2. `test_compute_diff_added_entity` (simple add)
3. `test_compute_diff_removed_entity` (simple remove)
4. `test_compute_diff_unchanged_entity` (no diff)
5. `test_compute_diff_moved_entity` (line shift)
6. `test_compute_diff_mixed_changes` (integration)
7. `test_classify_added` (classification unit)
8. `test_classify_removed` (classification unit)
9. `test_classify_moved_same_file` (classification unit)
10. `test_classify_relocated_different_file` (classification unit)
11. `test_calculate_lines_shifted_positive` (line math)
12. `test_calculate_lines_shifted_negative` (line math)
13. `test_calculate_lines_shifted_external_entity` (edge case)

### Phase 3: Blast Radius Calculator (Priority: HIGH)

```
Week 2, Days 1-2
================
Module: crates/parseltongue-core/src/diff/blast_radius_calculator_impl.rs

1. Implement EdgeDataPayload struct
2. Implement DefaultBlastRadiusCalculatorImpl
3. Pass all 5 BlastRadiusCalculatorTrait tests

Dependencies:
- Requires Phase 2 complete (needs entity change list)
- Requires access to dependency graph (existing API)
```

**Test Order**:
1. `test_calculate_neighbors_empty_changes` (base case)
2. `test_calculate_neighbors_single_hop` (simple case)
3. `test_calculate_neighbors_reverse_edge` (bidirectional)
4. `test_calculate_neighbors_excludes_changed` (exclusion logic)
5. `test_calculate_neighbors_deduplicates` (dedup logic)

### Phase 4: Visualization Transformer (Priority: MEDIUM)

```
Week 2, Days 3-4
================
Module: crates/parseltongue-core/src/diff/visualization_transformer_impl.rs

1. Implement NodeVisualizationStatusType enum
2. Implement VisualizationNodeDataPayload struct
3. Implement DefaultDiffVisualizationTransformerImpl
4. Pass all 4 DiffVisualizationTransformerTrait tests

Dependencies:
- Requires Phase 2 and 3 complete
```

**Test Order**:
1. `test_transform_added_entity_status` (basic transform)
2. `test_transform_neighbor_entity_status` (neighbor handling)
3. `test_transform_ambient_entity_status` (ambient handling)
4. `test_transform_external_entity_marked` (external flag)

### Phase 5: Performance Validation (Priority: MEDIUM)

```
Week 2, Day 5
=============
Files: Update existing test modules

1. Generate test fixtures (10K keys, 10K edges)
2. Implement performance benchmarks
3. Pass all 3 performance contract tests

Dependencies:
- Requires all prior phases complete
```

### Phase 6: CLI Integration (Priority: HIGH)

```
Week 3, Days 1-3
================
New Command: parseltongue diff

1. Add diff subcommand to CLI
2. Implement dual-database loading
3. Wire up diff computation pipeline
4. Implement CLI output formatting
5. Add --json flag support

Files to Create/Modify:
- crates/parseltongue/src/commands/diff_command.rs
- crates/parseltongue/src/main.rs (add subcommand)
```

### Phase 7: HTTP Endpoint (Priority: MEDIUM)

```
Week 3, Days 4-5
================
Endpoint: GET /diff-analysis-compare-snapshots

1. Add endpoint to pt08-http-code-query-server
2. Implement request/response types
3. Wire up to diff computation
4. Add to API documentation

Files to Modify:
- crates/pt08-http-code-query-server/src/routes.rs
- crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/
```

---

## Part 4: Dependencies and Ordering

### Dependency Graph

```
                    Phase 1: Key Normalization
                              |
                              v
                    Phase 2: Entity Differ
                              |
              +---------------+---------------+
              |                               |
              v                               v
    Phase 3: Blast Radius           Phase 4: Visualization
              |                               |
              +---------------+---------------+
                              |
                              v
                    Phase 5: Performance
                              |
                              v
                    Phase 6: CLI Integration
                              |
                              v
                    Phase 7: HTTP Endpoint
```

### Critical Path

1. **KeyNormalizerTrait** - BLOCKER for all other work
2. **EntityDifferTrait** - BLOCKER for blast radius and visualization
3. **BlastRadiusCalculatorTrait** - Required for complete diff results
4. **CLI Integration** - Required before WebSocket (per design decision)

### Cross-Crate Dependencies

| Component | Crate | Depends On |
|-----------|-------|------------|
| Key Normalizer | parseltongue-core | None (new module) |
| Entity Differ | parseltongue-core | Key Normalizer |
| Blast Radius | parseltongue-core | Entity Differ, existing storage |
| Visualization Transformer | parseltongue-core | Entity Differ, Blast Radius |
| CLI diff command | parseltongue | All parseltongue-core components |
| HTTP endpoint | pt08-http-code-query-server | All parseltongue-core components |

---

## Part 5: Key Technical Decisions Captured

### 5.1 ADR-001: Key Normalization (CRITICAL)

**Problem**: Entity keys include line numbers. When code is added above a function, line numbers shift, causing false positive diffs.

**Solution**: Extract stable identity by stripping line number suffix.

```
Full Key:   rust:fn:main:__crates_src_main_rs:10-50
Stable ID:  rust:fn:main:__crates_src_main_rs
```

**Algorithm**:
```rust
fn extract_stable_identity(key: &str) -> &str {
    if let Some(last_colon) = key.rfind(':') {
        if let Some(second_last) = key[..last_colon].rfind(':') {
            let suffix = &key[second_last + 1..];
            if suffix.chars().all(|c| c.is_ascii_digit() || c == '-' || c == ':') {
                return &key[..second_last];
            }
        }
    }
    key
}
```

### 5.2 External Entity Handling

**Pattern**: External entities have `unknown:0-0` suffix
**Behavior**:
- Include edges TO external entities in diff
- Exclude external entities from blast radius traversal
- Mark as `is_external: true` in visualization

### 5.3 CLI-First Approach

**Decision**: Implement `parseltongue diff` CLI command BEFORE WebSocket streaming
**Rationale**:
- Simpler to test and validate
- Provides immediate value
- WebSocket builds on same diff logic

### 5.4 API Reuse

**Finding**: 80% of needed functionality exists in current API
**New Endpoints Required** (4 total):
1. `/diff-analysis-compare-snapshots` - Compute diff between bases
2. `/workspace-create-from-path` - Create workspace
3. `/workspace-list-all` - List workspaces
4. `/workspace-watch-toggle` - Enable/disable file watching

---

## Part 6: Test Fixtures Summary

### Real Entity Keys (from API)

```typescript
// Function example
"rust:fn:compute_blast_radius_by_hops:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_blast_radius_impact_handler_rs:185-277"

// Method example
"rust:method:new:__crates_parseltongue-core_src_storage_cozo_client_rs:38-54"

// External example
"rust:fn:map_err:unknown:0-0"
```

### Codebase Statistics

| Metric | Value |
|--------|-------|
| Total Code Entities | 215 |
| Total Dependency Edges | 2,880 |
| Languages Detected | Rust |
| Semantic Clusters | 49 |

---

## Part 7: Current Status and Next Steps

### Current Status Summary (2026-01-23) - PROJECT COMPLETE

| Phase | Status | Tests | Notes |
|-------|--------|-------|-------|
| 1 | **GREEN** | 12 | KeyNormalizerTrait - COMPLETE |
| 2 | **GREEN** | 7 | EntityDifferTrait - COMPLETE |
| 3 | **GREEN** | 7 | BlastRadiusCalculatorTrait - COMPLETE |
| 4 | **GREEN** | 6 | DiffVisualizationTransformerTrait - COMPLETE |
| 5 | **GREEN** | 3 | Performance validation - COMPLETE |
| 6 | **GREEN** | 5 | CLI diff command - COMPLETE |
| 7 | **GREEN** | 27 | HTTP endpoint - COMPLETE |

### Test Count Verification (2026-01-23) - FINAL

```
parseltongue CLI:                    5 tests passing
parseltongue-core:                 130 tests passing (unit)
parseltongue-core doc tests:        26 tests passing
pt01-folder-to-cozodb-streamer:     13 tests passing
pt08-http-code-query-server:        48 tests passing (unit)
pt08 integration tests:             21 tests passing
pt08 doc tests:                      1 test passing
-------------------------------------------------
TOTAL:                             244 tests passing
```

### CLI Diff Command - Verified Working

```bash
$ parseltongue diff --help
Compare two CozoDB snapshots...

Usage: parseltongue diff [OPTIONS] --base <base> --live <live>

Options:
  -b, --base <base>       Path to base/before database
  -l, --live <live>       Path to live/after database
      --json              Output results as JSON
      --max-hops <max-hops>  Maximum hops for blast radius [default: 2]
```

---

## Part 7A: Phase 7 Requirements - HTTP Endpoint

### Endpoint Specification

**Endpoint**: `POST /diff-analysis-compare-snapshots`

**Purpose**: Compare two CozoDB snapshots via HTTP API, returning diff results, blast radius, and visualization data.

### Request Format

```json
{
  "base_db": "rocksdb:path/to/base.db",
  "live_db": "rocksdb:path/to/live.db"
}
```

**Query Parameters**:
- `max_hops` (optional, default: 2): Maximum hops for blast radius calculation

### Response Format

```json
{
  "success": true,
  "endpoint": "/diff-analysis-compare-snapshots",
  "data": {
    "diff": {
      "summary": {
        "total_before_count": 100,
        "total_after_count": 105,
        "added_entity_count": 10,
        "removed_entity_count": 5,
        "modified_entity_count": 3,
        "unchanged_entity_count": 87,
        "relocated_entity_count": 0
      },
      "entity_changes": [...],
      "edge_changes": [...]
    },
    "blast_radius": {
      "origin_entity": "...",
      "affected_by_distance": { "1": [...], "2": [...] },
      "total_affected_count": 15,
      "max_depth_reached": 2
    },
    "visualization": {
      "nodes": [...],
      "edges": [...],
      "diff_summary": {...},
      "max_blast_radius_depth": 2
    }
  },
  "tokens": 1500
}
```

### Files to Create/Modify

1. **New Handler**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/diff_analysis_compare_handler.rs`
   - 4-word naming: `diff_analysis_compare_handler`
   - Handler function: `handle_diff_analysis_compare_snapshots`
   - Must follow existing handler pattern (see `blast_radius_impact_handler.rs`)

2. **Register Module**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/mod.rs`
   - Add: `pub mod diff_analysis_compare_handler;`

3. **Add Route**: `crates/pt08-http-code-query-server/src/route_definition_builder_module.rs`
   - Import handler
   - Add route: `.route("/diff-analysis-compare-snapshots", post(...))`

### Implementation Strategy

**Reuse from parseltongue-core**:
```rust
use parseltongue_core::diff::{
    // Types
    DiffResultDataPayload, BlastRadiusResultPayload,
    VisualizationGraphDataPayload, EntityDataPayload, EdgeDataPayload,

    // Implementations
    DefaultKeyNormalizerImpl, DefaultEntityDifferImpl,
    DefaultBlastRadiusCalculatorImpl, DefaultDiffVisualizationTransformerImpl,

    // Traits
    KeyNormalizerTrait, EntityDifferTrait,
    BlastRadiusCalculatorTrait, DiffVisualizationTransformerTrait,
};
```

**Key Implementation Steps**:
1. Parse request body to extract base_db and live_db paths
2. Open both databases using `CozoDbStorage::open_with_path_async()`
3. Load entities from both databases
4. Load edges from both databases
5. Compute entity diff using `DefaultEntityDifferImpl`
6. Compute blast radius using `DefaultBlastRadiusCalculatorImpl`
7. Transform to visualization using `DefaultDiffVisualizationTransformerImpl`
8. Return combined JSON response

### Test Plan for Phase 7

**Unit Tests** (in handler file):
1. `test_parse_diff_request_body_valid` - Valid JSON parsing
2. `test_parse_diff_request_body_missing_base` - Error on missing base_db
3. `test_parse_diff_request_body_missing_live` - Error on missing live_db

**Integration Tests** (in `tests/http_server_integration_tests.rs`):
1. `test_diff_analysis_endpoint_returns_200` - Happy path
2. `test_diff_analysis_endpoint_invalid_db_paths` - Error handling
3. `test_diff_analysis_endpoint_respects_max_hops` - Query param works

### Handler Template

```rust
//! Diff analysis comparison endpoint handler
//!
//! # 4-Word Naming: diff_analysis_compare_handler
//!
//! Endpoint: POST /diff-analysis-compare-snapshots?max_hops=N
//!
//! Compares two database snapshots and returns diff + blast radius.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use parseltongue_core::diff::{/* ... */};

/// Request body for diff comparison
#[derive(Debug, Deserialize)]
pub struct DiffCompareRequestPayload {
    pub base_db: String,
    pub live_db: String,
}

/// Query parameters for diff endpoint
#[derive(Debug, Deserialize)]
pub struct DiffCompareQueryParams {
    #[serde(default = "default_max_hops")]
    pub max_hops: u32,
}

fn default_max_hops() -> u32 { 2 }

/// Handle diff analysis comparison request
pub async fn handle_diff_analysis_compare_snapshots(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<DiffCompareQueryParams>,
    Json(body): Json<DiffCompareRequestPayload>,
) -> impl IntoResponse {
    // Implementation here...
}
```

### Acceptance Criteria for Phase 7

- [ ] Handler file created with 4-word naming
- [ ] Route registered as POST endpoint
- [ ] Request body parsing works
- [ ] Both databases can be opened
- [ ] Diff computed correctly
- [ ] Blast radius calculated
- [ ] Visualization data generated
- [ ] JSON response format matches spec
- [ ] Error responses for invalid inputs
- [ ] At least 3 unit tests passing
- [ ] At least 2 integration tests passing

### Blockers / Open Questions for Phase 7

**Potential Blockers**:
- [ ] Need to verify `CozoDbStorage::open_with_path_async()` can open arbitrary paths
- [ ] May need to add helper function to load entities/edges from arbitrary storage

**Questions**:
- Should the endpoint accept database paths or workspace IDs?
- How should we handle cleanup of opened databases?
- Should there be a timeout for large database comparisons?

### Verification Commands for Phase 7

```bash
# Build HTTP server
cargo build -p pt08-http-code-query-server

# Run tests
cargo test -p pt08-http-code-query-server

# Test endpoint manually (after implementation)
curl -X POST http://localhost:7777/diff-analysis-compare-snapshots \
  -H "Content-Type: application/json" \
  -d '{"base_db":"rocksdb:base.db","live_db":"rocksdb:live.db"}'
```

---

## Part 8: Context for Resuming Work

### Quick Resume Checklist

When resuming this TDD session:

1. **Check current phase**: Phase 7 (HTTP Endpoint) - NOT STARTED
2. **Check last test status**: Run `cargo test -p parseltongue-core --lib` (130 passing)
3. **Verify CLI works**: Run `cargo run -p parseltongue -- diff --help`
4. **Review this document**: Part 7A has Phase 7 requirements
5. **Key files for Phase 7**:
   - `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/diff_analysis_compare_handler.rs` (CREATE)
   - `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/mod.rs` (ADD MODULE)
   - `crates/pt08-http-code-query-server/src/route_definition_builder_module.rs` (ADD ROUTE)

### File Locations

| Purpose | Path |
|---------|------|
| This journal | `docs/TDD-plan-20260123000800.md` |
| Executable specs for diff command | `docs/EXECUTABLE_SPECS_diff_command.md` |
| Trait definitions | `docs/07_RUST_INTERFACE_DEFINITIONS.md` |
| Test fixtures | `docs/06_TEST_STRATEGY_AND_FIXTURES.md` |
| Data structures | `docs/04_DATA_STRUCTURES.md` |
| Executable specs | `docs/05_EXECUTABLE_SPECS_DIFF_SYSTEM.md` |
| Key normalization ADR | `docs/ADR_001_KEY_NORMALIZATION.md` |

### Diff Module Files (All Complete)

| File | Purpose | Tests |
|------|---------|-------|
| `crates/parseltongue-core/src/diff/mod.rs` | Module exports | - |
| `crates/parseltongue-core/src/diff/types.rs` | All struct/enum definitions | - |
| `crates/parseltongue-core/src/diff/traits.rs` | All trait definitions | - |
| `crates/parseltongue-core/src/diff/key_normalizer_impl.rs` | KeyNormalizerTrait impl | 12 |
| `crates/parseltongue-core/src/diff/entity_differ_impl.rs` | EntityDifferTrait impl | 7 |
| `crates/parseltongue-core/src/diff/blast_radius_calculator_impl.rs` | BlastRadiusCalculatorTrait impl | 7 |
| `crates/parseltongue-core/src/diff/visualization_transformer_impl.rs` | DiffVisualizationTransformerTrait impl | 6 |
| `crates/parseltongue/src/commands/diff_command_execution_module.rs` | CLI diff command | 5 |
| `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/diff_analysis_compare_handler.rs` | HTTP endpoint handler | 27 |

### Documentation Files Created

| File | Purpose |
|------|---------|
| `docs/TDD-plan-20260123000800.md` | TDD progress journal (this document) |
| `docs/EXECUTABLE_SPECS_diff_command.md` | CLI command specifications |
| `docs/specs/REQ-HTTP-DIFF-ANALYSIS-COMPARE-SNAPSHOTS.md` | HTTP endpoint specifications |

### Key Insights from Design Phase

1. **Key instability is the root cause** of diff false positives - solve this first
2. **External entities (unknown:0-0)** represent graph boundaries - stop traversal there
3. **CLI-first** means we can validate diff logic before tackling WebSocket complexity
4. **80% API reuse** - don't reinvent, integrate with existing endpoints
5. **Performance SLOs**:
   - Key normalization: 10us per key
   - Diff computation: 500ms for 1K entities
   - Blast radius: 200ms for 100 changed + 10K edges

### Commands to Run

```bash
# Build project
cargo build --release

# Run all tests
cargo test --all

# Run specific crate tests
cargo test -p parseltongue-core

# Check for TODOs (must be clean before commit)
grep -r "TODO\|STUB\|PLACEHOLDER" --include="*.rs" crates/

# Start HTTP server (for API testing)
parseltongue pt08-http-code-query-server \
  --db "rocksdb:parseltongue20260122/analysis.db"
```

---

## Part 9: Progress Log

### Session: 2026-01-23 00:08:00 (Initial)

**Status**: TDD Progress Journal Created

**Completed**:
- Created comprehensive TDD tracking document
- Documented all 30 test stubs with their status
- Created implementation roadmap with phases
- Captured all key technical decisions
- Established dependency ordering
- Defined next immediate steps

**Next Session Goal**:
- Create module structure in parseltongue-core
- Move first test stub to code
- Begin RED phase for `test_extract_stable_identity_basic_function`

### Session: 2026-01-23 (Implementation Complete)

**Status**: TDD RED->GREEN Complete for All 4 Phases

**Completed**:
- Phase 1: KeyNormalizerTrait - 12 tests passing
- Phase 2: EntityDifferTrait - 7 tests passing
- Phase 3: BlastRadiusCalculatorTrait - 7 tests passing
- Phase 4: DiffVisualizationTransformerTrait - 6 tests passing
- Integration tests - 6 tests passing
- Total: 130 tests passing in parseltongue-core

**Files Created**:
```
crates/parseltongue-core/src/diff/
├── mod.rs                              # Module exports
├── types.rs                            # All struct/enum definitions (469 lines)
├── traits.rs                           # All trait definitions (382 lines)
├── key_normalizer_impl.rs              # KeyNormalizerTrait impl (400 lines)
├── entity_differ_impl.rs               # EntityDifferTrait impl (559 lines)
├── blast_radius_calculator_impl.rs     # BlastRadiusCalculatorTrait impl (227 lines)
├── visualization_transformer_impl.rs   # DiffVisualizationTransformerTrait impl (330 lines)
└── tests/
    └── mod.rs                          # Integration tests (189 lines)
```

**Key Implementation Details**:
1. Key normalization handles 4, 5, and multi-segment keys
2. External entities (unknown:0-0) properly detected
3. Stable identity matching ignores line number changes
4. Blast radius uses BFS with cycle detection
5. Visualization transformer maps change types to visual statuses

**Next Steps (Phase 5-7)**:
- Phase 5: Performance validation tests
- Phase 6: CLI `parseltongue diff` command
- Phase 7: HTTP endpoint `/diff-analysis-compare-snapshots`

---

### Session: 2026-01-23 08:xx (CLI Integration Started)

**Status**: Phase 6 IN PROGRESS - CLI Diff Command has compilation errors

**Files Created**:
```
crates/parseltongue/src/commands/diff_command_execution_module.rs  # 551 lines
```

**What Was Implemented**:
- `DiffCommandArgsPayload` struct with base/live paths, json flag, max_hops
- `execute_diff_analysis_command()` async function - main entry point
- Database loading: `load_database_storage_async()`, `load_database_entity_snapshot()`, `load_database_edges_snapshot()`
- Diff computation pipeline wired up
- Human-readable and JSON output formatters
- Unit tests for helper functions (4 tests)

**Compilation Errors BLOCKING Progress** (7 errors):

1. **`error[E0433]`**: `serde` crate not linked for `JsonDiffOutputPayload` serialization
   - Location: Line 235 (`#[derive(serde::Serialize)]`)
   - Fix: Add `serde` dependency to `parseltongue/Cargo.toml` with `derive` feature

2. **`error[E0609]`**: `InterfaceSignature` has no field `start_line`/`end_line`
   - Location: Lines 130-132
   - Fix: Check actual `InterfaceSignature` struct fields in `parseltongue-core/src/entities.rs`

3. **`error[E0308]`**: Type mismatches (2 instances)
   - Likely related to storage API return types
   - Fix: Verify `CozoDbStorage` method signatures

4. **`error[E0277]`**: `JsonDiffOutputPayload` does not implement `Serialize`
   - Cascading from E0433 (serde not linked)

5. **`error[E0597]`**: Lifetime issue with `line_diff` variable
   - Location: Line 362 (`line_diff.as_str()`)
   - Fix: Return owned `String` instead of `&str` from tuple

---

### Session: 2026-01-23 (Phase 6 COMPLETE)

**Status**: Phase 6 GREEN - All compilation errors FIXED, CLI diff command WORKING

**Fixes Applied**:
1. Added serde dependency to `crates/parseltongue/Cargo.toml`
2. Fixed `InterfaceSignature.line_range.start/end` field access
3. Fixed lifetime issue with `line_diff.as_str()`
4. Fixed `DependencyEdge` field conversions
5. Integrated diff subcommand into main.rs

**Verification**:
```bash
$ cargo test -p parseltongue
test result: ok. 5 passed; 0 failed

$ cargo run -p parseltongue -- diff --help
Usage: parseltongue diff [OPTIONS] --base <base> --live <live>
```

**Current Test Counts**:
- parseltongue-core: 130 tests passing
- parseltongue CLI: 5 tests passing
- Total: 135 tests passing

**Next Session Goal**:
~~- Begin Phase 7: Implement `/diff-analysis-compare-snapshots` HTTP endpoint~~
~~- Follow TDD: Write failing tests first, then implement handler~~

**COMPLETED** - Phase 7 implemented and all tests passing.

---

### Session: 2026-01-23 (Phase 7 COMPLETE - PROJECT FINISHED)

**Status**: ALL 7 PHASES GREEN - PROJECT COMPLETE

**Phase 7 Implementation Completed**:
- Created executable specification: `docs/specs/REQ-HTTP-DIFF-ANALYSIS-COMPARE-SNAPSHOTS.md`
- Implemented handler: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/diff_analysis_compare_handler.rs`
- Registered route as POST `/diff-analysis-compare-snapshots`
- Added 27 unit tests in handler module
- All 21 integration tests passing

**Files Created in Phase 7**:
```
docs/specs/REQ-HTTP-DIFF-ANALYSIS-COMPARE-SNAPSHOTS.md    # Executable specification (1427 lines)
crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/diff_analysis_compare_handler.rs
```

**HTTP Endpoint Features**:
- Request validation (missing/empty fields)
- Database path format validation (rocksdb: prefix required)
- Query parameter: `max_hops` (default: 2, max: 10)
- Full diff computation pipeline
- Blast radius calculation
- Visualization data transformation
- Token estimation in response
- Consistent error response format with codes

**Final Verification**:
```bash
$ cargo test --all
# 244 tests passing across all crates

$ cargo run -p parseltongue -- diff --help
# CLI command working

# HTTP endpoint ready:
# POST /diff-analysis-compare-snapshots
```

**Project Deliverables Complete**:
1. Core diff module with 4 trait implementations
2. Performance tests for large graphs
3. CLI `parseltongue diff` command
4. HTTP endpoint `/diff-analysis-compare-snapshots`
5. Comprehensive documentation and specs

---

## Appendix A: Entity Change Type Truth Table

| In Base | In Live | Keys Match | File Same | Classification |
|---------|---------|------------|-----------|----------------|
| Yes     | Yes     | Yes        | -         | UNCHANGED      |
| Yes     | Yes     | No         | Yes       | MOVED          |
| Yes     | Yes     | No         | No        | RELOCATED      |
| Yes     | No      | -          | -         | REMOVED        |
| No      | Yes     | -          | -         | ADDED          |

---

## Appendix B: Visualization Status Colors

| Status   | Color   | Size  | Animation     | Label  |
|----------|---------|-------|---------------|--------|
| added    | #00ff88 | 1.5x  | Pulse in      | [+]    |
| removed  | #ff4444 | 1.0x  | Fade out      | [-]    |
| modified | #ffcc00 | 1.2x  | Subtle pulse  | [~]    |
| moved    | #4a9eff | 1.2x  | Arrow         | [->]   |
| neighbor | #ffa94d | 1.0x  | Glow          |        |
| ambient  | #888888 | 0.5x  | None          |        |

---

## Appendix C: Module Structure (Proposed)

```
crates/parseltongue-core/src/
  diff/
    mod.rs                              # Module exports
    types.rs                            # All struct/enum definitions
    key_normalizer_impl.rs              # KeyNormalizerTrait impl
    entity_differ_impl.rs               # EntityDifferTrait impl
    blast_radius_calculator_impl.rs     # BlastRadiusCalculatorTrait impl
    visualization_transformer_impl.rs   # DiffVisualizationTransformerTrait impl
    tests/
      mod.rs                            # Test module exports
      key_normalizer_tests.rs           # KeyNormalizer unit tests
      entity_differ_tests.rs            # EntityDiffer unit tests
      blast_radius_tests.rs             # BlastRadius unit tests
      visualization_tests.rs            # Visualization unit tests
      performance_tests.rs              # Performance contract tests
```

---

*This document is the single source of truth for TDD progress on the Parseltongue Diff Visualization System.*

---

## PROJECT COMPLETION CERTIFICATE

**Project**: Parseltongue Diff Visualization System
**Status**: COMPLETE
**Completion Date**: 2026-01-23
**Total Development Phases**: 7
**Total Tests**: 244
**Test Pass Rate**: 100%

**Key Deliverables**:
- 4 core traits with full implementations
- CLI command: `parseltongue diff`
- HTTP endpoint: `POST /diff-analysis-compare-snapshots`
- 3 executable specification documents
- Comprehensive TDD progress journal

**Document Version**: 2.0.0 (FINAL)
**Last Updated**: 2026-01-23 (PROJECT COMPLETE)
**Status**: Archived - All phases complete
