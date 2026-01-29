# TDD Session State: Live Tracking / Incremental Indexing Feature

**Session Started**: 2026-01-28
**Last Updated**: 2026-01-29
**Feature**: `/incremental-reindex-file-update` HTTP endpoint
**Current Phase**: STUB COMPLETE → Ready for RED (Executable specs written)
**Milestone**: Executable specifications completed with 26 test templates and 14 WHEN...THEN...SHALL contracts

---

## Feature Specification

### High-Level Goal
Implement a REST endpoint that enables incremental file re-indexing without full codebase reprocessing. When a file changes, the endpoint should:
1. Accept file path as input
2. Compute SHA-256 hash and compare with cached value
3. If hash changed: delete old entities, re-parse file, insert new entities
4. Return diff statistics (entities added/removed/modified)

### Performance Requirements (Updated with Executable Specs)
- Single file reindex: **< 500ms** (full cycle - REQ-INC-P7-001)
- Early return for unchanged files: **< 50ms** (REQ-INC-P7-002)
- Parsing throughput: **< 20ms per 1K LOC** (REQ-INC-P1-002)
- Minimal database I/O (batch operations)
- Idempotent: safe to call multiple times

### Executable Specifications Document
**Location**: `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/docs/INCREMENTAL_INDEXING_EXECUTABLE_SPECS.md`
**Status**: COMPLETE
**Created**: 2026-01-29
**Contracts**: 14 WHEN...THEN...SHALL requirements (REQ-INC-P1-001 through REQ-INC-P7-002)
**Test Templates**: 26 tests with GIVEN/WHEN/THEN structure

---

## Architecture Context

### Project Structure (8 Crates)
```
parseltongue-dependency-graph-generator/
├── crates/
│   ├── parseltongue/                    # CLI binary (dispatcher)
│   ├── parseltongue-core/               # Shared types, storage, tree-sitter
│   │   └── src/storage/cozo_client.rs   # Database operations
│   ├── pt01-folder-to-cozodb-streamer/  # Initial indexing tool
│   └── pt08-http-code-query-server/     # HTTP API server (OUR WORK HERE)
│       ├── src/
│       │   ├── lib.rs
│       │   ├── http_endpoint_handler_modules/
│       │   │   ├── mod.rs
│       │   │   ├── server_health_check_handler.rs
│       │   │   ├── blast_radius_impact_handler.rs
│       │   │   └── [13 other handlers...]
│       │   └── route_definition_builder_module.rs
│       └── tests/
│           └── http_server_integration_tests.rs
```

### Naming Convention (CRITICAL)
- **All names must be exactly 4 words** (LLM tokenization optimization)
- Functions: `handle_incremental_reindex_file_update()`
- Files: `incremental_reindex_file_handler.rs`
- Endpoint: `/incremental-reindex-file-update`

---

## EXECUTABLE SPECIFICATIONS SUMMARY (NEW - 2026-01-29)

### Implementation Phases Overview

The feature has been decomposed into **7 phases** with **26 test templates** covering **14 contractual requirements**:

#### Phase P1: FileParser Facade (Foundation)
**Location**: `parseltongue-core/src/file_parser.rs` (NEW FILE)
**Contracts**: REQ-INC-P1-001, REQ-INC-P1-002
**Tests**: 5 unit tests
**Purpose**: Thread-safe wrapper around QueryBasedExtractor for pt08
**Interface**:
```rust
pub struct FileParser { extractor: Mutex<QueryBasedExtractor> }
impl FileParser {
    pub fn create_new_parser_instance() -> Result<Self>;
    pub fn parse_file_to_entities(&self, path: &Path, content: &str)
        -> Result<(Vec<ParsedEntity>, Vec<DependencyEdge>)>;
}
```

#### Phase P2: State Integration (Foundation)
**Location**: `pt08/src/http_server_startup_runner.rs` (MODIFIED)
**Contracts**: REQ-INC-P2-001, REQ-INC-P2-002
**Tests**: 2 unit tests
**Purpose**: Add `parser_instance_option_arc: Option<Arc<FileParser>>` to SharedApplicationStateContainer
**Change**: Modify `create_with_database_and_parser()` constructor

#### Phase P3: Deletion Tests (Validation)
**Location**: `pt08/tests/http_server_integration_tests.rs` (ADDITIONS)
**Contracts**: REQ-INC-P3-001 through REQ-INC-P3-007
**Tests**: 7 integration tests
**Purpose**: Validate existing deletion logic (hash checking, entity removal, error cases)
**Coverage**: Early return, 404/400 errors, hash cache updates, edge deletion

#### Phase P4: Re-parsing Logic (Core)
**Location**: `pt08/src/http_endpoint_handler_modules/incremental_reindex_file_handler.rs:277` (REPLACEMENT)
**Contracts**: REQ-INC-P4-001, REQ-INC-P4-002, REQ-INC-P4-003
**Tests**: 2 integration tests
**Purpose**: Replace TODO placeholder with actual parsing + insertion logic
**Dependencies**: P1 (parser), P2 (state), P5 (conversion)

#### Phase P5: Entity Conversion (Foundation)
**Location**: `parseltongue-core/src/entity_conversion.rs` (NEW FILE)
**Contracts**: REQ-INC-P5-001, REQ-INC-P5-002
**Tests**: 5 unit tests
**Purpose**: Convert `ParsedEntity` → `CodeEntity` with ISGL1 key generation
**Interface**:
```rust
pub fn convert_parsed_to_code_entity(parsed: &ParsedEntity, file_path: &str, source: &str) -> Result<CodeEntity>;
pub fn generate_isgl1_key_string(lang: &Language, type: &str, name: &str, path: &str, lines: (usize, usize)) -> String;
pub fn detect_test_entity_class(file_path: &str, entity_name: &str) -> EntityClass;
```

#### Phase P6: Complete Cycle Test (Validation)
**Location**: `pt08/tests/http_server_integration_tests.rs` (ADDITIONS)
**Contracts**: REQ-INC-P6-001
**Tests**: 2 end-to-end tests
**Purpose**: Verify full delete → parse → insert → hash-update cycle
**Scenarios**: Entity count changes, edge handling

#### Phase P7: Performance Validation (Validation)
**Location**: `pt08/tests/http_server_integration_tests.rs` (ADDITIONS)
**Contracts**: REQ-INC-P7-001, REQ-INC-P7-002
**Tests**: 3 benchmark tests
**Purpose**: Validate < 500ms full cycle, < 50ms early return, batch performance

### Corrected Implementation Order
```
Phase 1 (Foundation):  P5 (entity conversion) → P1 (parser facade)
Phase 2 (Integration): P2 (state) + P3 (deletion tests) [parallel after P1]
Phase 3 (Core Logic):  P4 (re-parsing) [requires P1, P2, P5]
Phase 4 (Validation):  P6 (complete test) → P7 (performance)
```

**Rationale**: P5 must come first because P1's tests need entity conversion utilities. P3 can run parallel to P2 since it tests existing code.

### Files to Create (4 NEW)
1. `parseltongue-core/src/file_parser.rs` - FileParser facade
2. `parseltongue-core/src/entity_conversion.rs` - Entity conversion utilities
3. `parseltongue-core/tests/file_parser_facade_tests.rs` - Parser unit tests
4. `parseltongue-core/tests/entity_conversion_tests.rs` - Conversion unit tests

### Files to Modify (4 EXISTING)
1. `parseltongue-core/src/lib.rs` - Export new modules (`pub mod file_parser; pub mod entity_conversion;`)
2. `pt08/src/http_server_startup_runner.rs` - Add parser field to state (line ~31)
3. `pt08/src/http_endpoint_handler_modules/incremental_reindex_file_handler.rs` - Replace TODO at line 277-289
4. `pt08/tests/http_server_integration_tests.rs` - Add 26 test functions

### Test Count Breakdown
| Phase | Unit Tests | Integration Tests | Total |
|-------|-----------|------------------|-------|
| P1 | 5 | 0 | 5 |
| P2 | 2 | 0 | 2 |
| P3 | 0 | 7 | 7 |
| P4 | 0 | 2 | 2 |
| P5 | 5 | 0 | 5 |
| P6 | 0 | 2 | 2 |
| P7 | 0 | 3 | 3 |
| **TOTAL** | **12** | **14** | **26** |

### Requirement Traceability Matrix
| Requirement ID | Contract | Test Template | Implementation |
|---------------|----------|---------------|----------------|
| REQ-INC-P1-001 | FileParser parsing | P1.2, P1.3, P1.5 | `parse_file_to_entities()` |
| REQ-INC-P1-002 | Thread safety | P1.1, P1.4 | `Mutex<QueryBasedExtractor>` |
| REQ-INC-P2-001 | Parser in state | P2.1 | `parser_instance_option_arc` field |
| REQ-INC-P2-002 | Availability check | P2.2 | `Option<Arc<FileParser>>` pattern |
| REQ-INC-P3-001 | Early return | P3.1 | Hash comparison at line 260-265 |
| REQ-INC-P3-002 | Entity deletion | P3.2 | Deletion logic at line 275 |
| REQ-INC-P3-003 | File not found | P3.3 | Error handling |
| REQ-INC-P3-004 | Empty path | P3.4 | Validation |
| REQ-INC-P3-005 | Directory path | P3.5 | `is_file()` check |
| REQ-INC-P3-006 | Hash cache update | P3.6 | `set_cached_file_hash_value()` |
| REQ-INC-P3-007 | Edge deletion | P3.7 | Edge removal at line 275 |
| REQ-INC-P4-001 | File parsing | P4.1 | Parse + insert logic |
| REQ-INC-P4-002 | Edge insertion | P4.1 | `insert_edges_batch()` |
| REQ-INC-P4-003 | Graceful degradation | P4.2 | `if let Some(parser)` check |
| REQ-INC-P5-001 | Entity conversion | P5.1 | `convert_parsed_to_code_entity()` |
| REQ-INC-P5-002 | ISGL1 key format | P5.2 | `generate_isgl1_key_string()` |
| REQ-INC-P6-001 | Full cycle | P6.1, P6.2 | End-to-end workflow |
| REQ-INC-P7-001 | Performance <500ms | P7.1, P7.3 | Optimized implementation |
| REQ-INC-P7-002 | Early return <50ms | P7.2 | Hash check optimization |

---

## TDD Progress Tracking (ORIGINAL PLAN - Now Superseded by Executable Specs)

**NOTE**: The following sections represent the initial plan. Refer to `docs/INCREMENTAL_INDEXING_EXECUTABLE_SPECS.md` for the definitive specification.

### Phase 1: Database Helper Methods (RED Phase - NOW INTEGRATED INTO P3)

**Location**: `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/parseltongue-core/src/storage/cozo_client.rs`

**Test File**: `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/parseltongue-core/tests/incremental_reindex_storage_tests.rs`

#### Tests to Write (5 tests):

- [ ] **Test 1**: `test_get_entities_by_file_path_returns_all_entities`
  - **Purpose**: Verify retrieval of all entities for a given file path
  - **Setup**: Insert 3 entities from same file, 2 from different file
  - **Assertion**: Query returns exactly 3 entities matching file path
  - **Status**: NOT STARTED
  - **Estimated Complexity**: Low (simple Datalog query)

- [ ] **Test 2**: `test_delete_entities_batch_by_keys_removes_multiple`
  - **Purpose**: Verify batch deletion of entities by ISGL1 keys
  - **Setup**: Insert 5 entities, delete 3 by keys
  - **Assertion**: Only 2 entities remain after deletion
  - **Status**: NOT STARTED
  - **Estimated Complexity**: Medium (batch delete operation)

- [ ] **Test 3**: `test_delete_edges_by_from_keys_removes_outgoing`
  - **Purpose**: Verify deletion of all outgoing edges from specified entities
  - **Setup**: Create 5 edges (A→B, A→C, B→D, C→E, D→F), delete edges from A and B
  - **Assertion**: Only 3 edges remain (C→E, D→F, and one other)
  - **Status**: NOT STARTED
  - **Estimated Complexity**: Medium (requires DependencyEdges schema)

- [ ] **Test 4**: `test_get_cached_file_hash_value_retrieves_hash`
  - **Purpose**: Verify hash retrieval from cache table
  - **Setup**: Insert hash "abc123" for file "src/main.rs"
  - **Assertion**: Query returns "abc123"
  - **Status**: NOT STARTED
  - **Estimated Complexity**: Low (requires FileHashCache schema creation)

- [ ] **Test 5**: `test_set_cached_file_hash_value_stores_hash`
  - **Purpose**: Verify hash storage/update in cache table
  - **Setup**: Set hash "def456" for file "src/lib.rs", then update to "xyz789"
  - **Assertion**: Latest hash retrieved is "xyz789"
  - **Status**: NOT STARTED
  - **Estimated Complexity**: Low (upsert operation)

#### Implementation Methods Required (5 methods):

**In `CozoDbStorage` struct**:

1. `pub async fn get_entities_by_file_path(&self, file_path: &str) -> Result<Vec<CodeEntity>>`
   - Datalog query: Filter CodeGraph by file_path column
   - Return: Vec of matching entities

2. `pub async fn delete_entities_batch_by_keys(&self, keys: &[String]) -> Result<usize>`
   - Datalog query: Batch `:rm` operation
   - Return: Count of deleted entities

3. `pub async fn delete_edges_by_from_keys(&self, from_keys: &[String]) -> Result<usize>`
   - Datalog query: Delete from DependencyEdges where from_key IN keys
   - Return: Count of deleted edges

4. `pub async fn get_cached_file_hash_value(&self, file_path: &str) -> Result<Option<String>>`
   - Datalog query: Select hash from FileHashCache
   - Return: Some(hash) or None

5. `pub async fn set_cached_file_hash_value(&self, file_path: &str, hash: &str) -> Result<()>`
   - Datalog query: `:put` to FileHashCache (upsert)
   - Return: Success/error

**Schema Addition Required**:
```rust
pub async fn create_file_hash_cache_schema(&self) -> Result<()> {
    let schema = r#"
        :create FileHashCache {
            file_path: String =>
            sha256_hash: String,
            last_indexed: String
        }
    "#;
    // ... execute schema creation
}
```

---

### Phase 2: HTTP Handler Tests (RED Phase)

**Location**: `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/tests/http_server_integration_tests.rs`

**Handler File**: `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/incremental_reindex_file_handler.rs`

#### Tests to Write (5 tests):

- [ ] **Test 6**: `test_incremental_reindex_returns_400_missing_path`
  - **Endpoint**: `POST /incremental-reindex-file-update` (no body)
  - **Assertion**: HTTP 400, error message mentions "file_path required"
  - **Status**: NOT STARTED

- [ ] **Test 7**: `test_incremental_reindex_returns_404_nonexistent_file`
  - **Endpoint**: `POST /incremental-reindex-file-update {"file_path": "/fake/path.rs"}`
  - **Assertion**: HTTP 404, error message "file not found"
  - **Status**: NOT STARTED

- [ ] **Test 8**: `test_incremental_reindex_returns_unchanged_for_same_hash`
  - **Setup**: Index file once, call endpoint again without changing file
  - **Assertion**: HTTP 200, `{"hash_changed": false, "entities_modified": 0}`
  - **Status**: NOT STARTED

- [ ] **Test 9**: `test_incremental_reindex_returns_diff_for_changed_file`
  - **Setup**: Index file, modify file content, call endpoint
  - **Assertion**: HTTP 200, `{"hash_changed": true, "entities_added": 1, "entities_removed": 1, "entities_modified": 0}`
  - **Status**: NOT STARTED

- [ ] **Test 10**: `test_incremental_reindex_handles_file_deletion_gracefully`
  - **Setup**: Index file, delete file from filesystem, call endpoint
  - **Assertion**: HTTP 200, `{"hash_changed": true, "entities_removed": N}`
  - **Status**: NOT STARTED

#### Handler Implementation Required:

**File**: `incremental_reindex_file_handler.rs`

```rust
use crate::structured_error_handling_types::HttpServerErrorTypes;
use axum::{extract::State, Json};
use parseltongue_core::storage::CozoDbStorage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct IncrementalReindexFileRequest {
    pub file_path: String,
}

#[derive(Debug, Serialize)]
pub struct IncrementalReindexFileResponse {
    pub hash_changed: bool,
    pub entities_added: usize,
    pub entities_removed: usize,
    pub entities_modified: usize,
    pub new_hash: Option<String>,
}

pub async fn handle_incremental_reindex_file_update(
    State(db): State<Arc<CozoDbStorage>>,
    Json(request): Json<IncrementalReindexFileRequest>,
) -> Result<Json<IncrementalReindexFileResponse>, HttpServerErrorTypes> {
    // STUB: To be implemented in GREEN phase
    todo!("Implement incremental file reindex")
}
```

**Route Registration** in `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/route_definition_builder_module.rs`:

```rust
.route(
    "/incremental-reindex-file-update",
    post(incremental_reindex_file_handler::handle_incremental_reindex_file_update),
)
```

---

### Phase 3: Integration Tests (RED Phase)

**Location**: `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/tests/incremental_reindex_integration_tests.rs`

#### Tests to Write (2 tests):

- [ ] **Test 11**: `test_full_incremental_reindex_workflow_end_to_end`
  - **Workflow**:
    1. Index Rust file with 2 functions
    2. Modify file (add 1 function, remove 1 function)
    3. Call `/incremental-reindex-file-update`
    4. Verify database reflects changes
    5. Query `/code-entities-list-all` to confirm
  - **Status**: NOT STARTED
  - **Estimated Duration**: 15 minutes to write

- [ ] **Test 12**: `test_incremental_reindex_performance_under_500ms`
  - **Setup**: Create realistic Rust file (500 LOC, 20 entities)
  - **Measure**: Time from request to response
  - **Assertion**: Total duration < 500ms
  - **Status**: NOT STARTED
  - **Estimated Duration**: 10 minutes to write

---

## Implementation Dependencies

### Cross-Crate Dependencies
```
pt08-http-code-query-server (Handler)
    ↓ depends on
parseltongue-core (Storage methods)
    ↓ depends on
cozo (Database engine)
```

### External Dependencies Required
- `sha2` crate (SHA-256 hashing) - **Already in Cargo.toml?** [VERIFY]
- `tokio::fs` (async file I/O) - **Already available via Tokio**

---

## Technical Debt & Design Decisions

### Decision 1: Hash Storage Schema
**Question**: Should FileHashCache be a separate table or column in CodeGraph?

**Decision**: **Separate table (FileHashCache)**
- **Rationale**:
  - CodeGraph stores entities (functions, structs), not files
  - One file contains many entities (1:N relationship)
  - Easier to query "has this file changed?" without entity-level joins
  - Follows single-responsibility principle

### Decision 2: Hash Comparison Strategy
**Question**: When should we compute file hash?

**Decision**: **Compute on-demand during endpoint call**
- **Rationale**:
  - Avoids file watching complexity (out of scope for v1)
  - Idempotent: safe to call anytime
  - User controls when to check for changes
  - Simpler error handling (file read errors surfaced immediately)

### Decision 3: Batch vs Individual Entity Deletion
**Question**: Delete entities one-by-one or batch?

**Decision**: **Batch deletion**
- **Rationale**:
  - Performance: 1 Datalog query vs N queries
  - Transactional: all-or-nothing semantics
  - Matches existing `insert_edges_batch` pattern in codebase

### Decision 4: Edge Deletion Strategy
**Question**: Delete edges when deleting entities?

**Decision**: **YES - Cascade delete outgoing edges**
- **Rationale**:
  - Prevents orphaned edges (from_key pointing to deleted entities)
  - Maintains graph consistency
  - Incoming edges naturally become invalid (can be queried as "dead links")

---

## Current Blockers

**None** - Ready to begin RED phase

---

## Next Steps (UPDATED - Based on Executable Specs)

### IMMEDIATE: Begin Phase P5 (Entity Conversion Foundation)

**Rationale**: P5 has zero dependencies and is required by P1's tests.

### Step 1: Create Entity Conversion Module (15 minutes)
1. Create `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/parseltongue-core/src/entity_conversion.rs`
2. Create `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/parseltongue-core/tests/entity_conversion_tests.rs`
3. Write 5 failing tests:
   - `test_basic_entity_conversion` (P5.1)
   - `test_isgl1_key_format` (P5.2)
   - `test_detection_by_path` (P5.3)
   - `test_path_sanitization` (P5.4)
   - `test_code_snippet_extraction` (P5.5)
4. Run → **EXPECT COMPILATION ERRORS** (functions don't exist)

### Step 2: Implement Entity Conversion (GREEN - 25 minutes)
1. Implement `convert_parsed_to_code_entity()`
2. Implement `generate_isgl1_key_string()`
3. Implement `extract_code_snippet_lines()`
4. Implement `detect_test_entity_class()`
5. Implement `sanitize_path_for_key()`
6. Export module in `parseltongue-core/src/lib.rs`
7. Run tests → **EXPECT ALL 5 PASS**

### Step 3: Create FileParser Facade (RED - 20 minutes)
1. Create `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/parseltongue-core/src/file_parser.rs`
2. Create `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/parseltongue-core/tests/file_parser_facade_tests.rs`
3. Write 5 failing tests:
   - `test_parser_creation_succeeds` (P1.1)
   - `test_parse_rust_file_extracts` (P1.2)
   - `test_unsupported_extension_returns_error` (P1.3)
   - `test_thread_safe_concurrent_parsing` (P1.4)
   - `test_empty_file_returns_empty` (P1.5)
4. Run → **EXPECT COMPILATION ERRORS**

### Step 4: Implement FileParser (GREEN - 30 minutes)
1. Implement `FileParser` struct with `Mutex<QueryBasedExtractor>`
2. Implement `create_new_parser_instance()`
3. Implement `parse_file_to_entities()`
4. Implement `detect_language_from_path()` (private)
5. Export module in `parseltongue-core/src/lib.rs`
6. Run tests → **EXPECT ALL 5 PASS**

### Step 5: Add Parser to State (RED → GREEN - 20 minutes)
1. Write 2 tests in `pt08/src/http_server_startup_runner.rs`:
   - `test_state_includes_parser_instance` (P2.1)
   - `test_parser_accessible_from_state` (P2.2)
2. Run → **EXPECT COMPILATION ERRORS**
3. Add `parser_instance_option_arc: Option<Arc<FileParser>>` field to `SharedApplicationStateContainer`
4. Implement `create_with_database_and_parser()` method
5. Run tests → **EXPECT BOTH PASS**

### Step 6: Write Deletion Integration Tests (RED - 35 minutes)
1. Add 7 tests to `pt08/tests/http_server_integration_tests.rs`:
   - `test_unchanged_file_returns_early` (P3.1)
   - `test_changed_file_deletes_entities` (P3.2)
   - `test_file_not_found_returns_404` (P3.3)
   - `test_empty_path_returns_400` (P3.4)
   - `test_directory_path_returns_400` (P3.5)
   - `test_hash_cache_updates_after_change` (P3.6)
   - `test_edge_deletion_with_entities` (P3.7)
2. Run → **MAY PASS** (tests existing logic) or **FAIL** (missing features)

### Step 7: Implement Re-parsing Logic (GREEN - 40 minutes)
1. Open `pt08/src/http_endpoint_handler_modules/incremental_reindex_file_handler.rs`
2. Navigate to line 277 (TODO placeholder)
3. Replace with parsing + insertion logic (template in specs)
4. Add imports: `use parseltongue_core::file_parser::FileParser;`
5. Add imports: `use parseltongue_core::entity_conversion::*;`
6. Write 2 tests:
   - `test_reparsing_inserts_new_entities` (P4.1)
   - `test_parser_unavailable_degrades_gracefully` (P4.2)
7. Run P3 + P4 tests → **EXPECT ALL PASS**

### Step 8: Complete Cycle Tests (RED → GREEN - 25 minutes)
1. Add 2 tests to integration suite:
   - `test_complete_reindex_cycle` (P6.1)
   - `test_reindex_with_edges` (P6.2)
2. Run → **EXPECT FAILURES** (edge cases)
3. Fix any discovered issues
4. Run → **EXPECT ALL PASS**

### Step 9: Performance Validation (RED → GREEN - 30 minutes)
1. Add 3 tests to integration suite:
   - `test_performance_under_500ms` (P7.1)
   - `test_early_return_under_50ms` (P7.2)
   - `test_benchmark_multiple_files` (P7.3)
2. Run → **EXPECT FAILURES** (performance not optimized)
3. Profile and optimize bottlenecks
4. Run → **EXPECT ALL PASS** (< 500ms target)

### Step 10: Refactor and Document (30 minutes)
1. Run `cargo clippy --all` and fix warnings
2. Add doc comments to public APIs
3. Extract any duplicated logic
4. Run full test suite → **EXPECT ALL 26 PASS**
5. Update CLAUDE.md with new endpoint documentation

**Total Estimated Time**: 4.5 hours

**Critical Path**: P5 → P1 → P2 → P4 → P6 → P7 (P3 can run in parallel to P2)

---

## Performance Metrics (Updated with Executable Specs)

| Operation | Target (Contract) | Test | Status |
|-----------|------------------|------|--------|
| Parser initialization | <5ms | P1-AC1 | NOT MEASURED |
| Parse 1K LOC Rust file | <20ms | REQ-INC-P1-002, P1-AC2 | NOT MEASURED |
| Early return (unchanged file) | **<50ms** | **REQ-INC-P7-002, P7.2** | **NOT MEASURED** |
| Full reindex cycle (100 LOC, 5 entities) | **<500ms** | **REQ-INC-P7-001, P7.1** | **NOT MEASURED** |
| 10 file batch reindex | <5s total | P7.3 | NOT MEASURED |

**Critical Measurements**: P7.1 (500ms) and P7.2 (50ms) are contract requirements, not aspirational goals.

---

## Test Coverage Goals (Updated - 26 Total Tests)

### Unit Tests (12 tests)
- **P1 (FileParser)**: 5 tests - parser creation, Rust parsing, error handling, thread safety, empty files
- **P2 (State Integration)**: 2 tests - parser field presence, accessibility
- **P5 (Entity Conversion)**: 5 tests - conversion, ISGL1 keys, test detection, path sanitization, snippet extraction

### Integration Tests (14 tests)
- **P3 (Deletion Logic)**: 7 tests - early return, entity/edge deletion, 404/400 errors, hash cache updates
- **P4 (Re-parsing)**: 2 tests - entity insertion, graceful degradation
- **P6 (Complete Cycle)**: 2 tests - full workflow, edge handling
- **P7 (Performance)**: 3 tests - 500ms target, 50ms early return, batch processing

**Total**: 26 tests (12 unit + 14 integration)
**Coverage Target**: 100% of new code paths (P1-P7)

---

## Context Notes

### Key Insights from Codebase
1. **CozoDB Storage Pattern**: All mutations use `:put` (upsert) or `:rm` (delete) queries
2. **Error Handling**: `ParseltongError::DatabaseError` for DB operations, `HttpServerErrorTypes` for HTTP layer
3. **Existing Patterns to Follow**:
   - `blast_radius_impact_handler.rs` - good example of complex handler
   - `insert_edges_batch` method - template for batch operations
   - `get_all_entities` method - template for query-all operations

### Assumptions Made
1. File paths are absolute (no relative path resolution needed)
2. Only Rust files supported initially (can extend to other languages later)
3. File deletion from filesystem means "remove all entities" (not an error condition)
4. Hash cache is never manually invalidated (always trust filesystem as source of truth)

### Questions for Later
- Should we add a "last_indexed" timestamp to FileHashCache? (Not critical for v1)
- Should we emit metrics/logs for reindex operations? (Nice-to-have)
- Should we support bulk file reindex (array of paths)? (Future enhancement)

---

## Session Metadata (Updated with Executable Specs)

### Files to Create (4 NEW)
1. `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/parseltongue-core/src/file_parser.rs` (FileParser facade - ~80 LOC)
2. `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/parseltongue-core/src/entity_conversion.rs` (Conversion utilities - ~120 LOC)
3. `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/parseltongue-core/tests/file_parser_facade_tests.rs` (P1 tests - ~110 LOC)
4. `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/parseltongue-core/tests/entity_conversion_tests.rs` (P5 tests - ~90 LOC)

### Files to Modify (4 EXISTING)
1. `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/parseltongue-core/src/lib.rs`
   - Add: `pub mod file_parser;`
   - Add: `pub mod entity_conversion;`

2. `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/http_server_startup_runner.rs`
   - Add field: `parser_instance_option_arc: Option<Arc<FileParser>>`
   - Add method: `create_with_database_and_parser(storage: CozoDbStorage) -> Self`
   - Add tests: P2.1, P2.2 (~30 LOC)

3. `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/incremental_reindex_file_handler.rs`
   - Replace lines 277-289 (TODO) with parsing logic (~50 LOC)
   - Add imports for FileParser and entity_conversion

4. `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/tests/http_server_integration_tests.rs`
   - Add 14 test functions for P3, P4, P6, P7 (~600 LOC)

### Lines of Code Estimate (Updated)
- **New modules**: ~400 LOC (file_parser + entity_conversion + implementation)
- **Modified files**: ~100 LOC (state changes + handler replacement)
- **Test code**: ~830 LOC (26 tests total)
- **Total**: ~1,330 LOC (2.4x original estimate - more comprehensive)

---

## Commit Strategy (Updated for 7 Phases)

**Branch**: `feature/incremental-reindex-file-update`

**Commit Sequence** (strict TDD, one commit per GREEN phase):
1. `docs: add executable specifications for incremental reindex (26 tests, 14 contracts)` ✓ DONE
2. `test(P5): add failing entity conversion tests (RED phase)`
3. `feat(P5): implement entity conversion utilities (GREEN phase)`
4. `test(P1): add failing FileParser facade tests (RED phase)`
5. `feat(P1): implement FileParser with thread-safe wrapper (GREEN phase)`
6. `test(P2): add parser state integration tests (RED phase)`
7. `feat(P2): add parser instance to SharedApplicationStateContainer (GREEN phase)`
8. `test(P3): add deletion logic integration tests (RED phase - 7 tests)`
9. `fix(P3): resolve any deletion edge cases discovered (GREEN phase)`
10. `test(P4): add re-parsing logic tests (RED phase)`
11. `feat(P4): implement file re-parsing with entity insertion (GREEN phase)`
12. `test(P6): add complete cycle integration tests (RED phase)`
13. `fix(P6): resolve workflow integration issues (GREEN phase)`
14. `test(P7): add performance validation tests (RED phase)`
15. `perf(P7): optimize to meet <500ms contract (GREEN phase)`
16. `refactor: extract utilities and add doc comments`
17. `docs: update CLAUDE.md with /incremental-reindex-file-update endpoint`

**Current Status**: Step 1 complete (executable specs written)
**Next Commit**: Step 2 (P5 tests)

---

## Resume Checklist

When resuming this session, verify:
- [ ] Review executable specs: `docs/INCREMENTAL_INDEXING_EXECUTABLE_SPECS.md`
- [ ] Current TDD phase: Check header of this file (currently: **STUB COMPLETE → Ready for RED**)
- [ ] Last completed phase: Check commit history (`git log --oneline | head -10`)
- [ ] Next immediate step: See "Next Steps (UPDATED)" section above
- [ ] Run test suite to confirm baseline: `cargo test --all`
- [ ] Check for compilation warnings: `cargo clippy --all`

**Quick Resume Commands**:
```bash
cd /Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator

# View current status
cat TDD_SESSION_STATE.md | head -10

# View executable specs
cat docs/INCREMENTAL_INDEXING_EXECUTABLE_SPECS.md | grep "^##" | head -20

# Check test status
cargo test --all 2>&1 | grep -E "test result|running"

# Check git status
git status --short
```

**Phase Progression Tracker**:
```
✓ STUB: Executable specifications written (2026-01-29)
▶ P5 RED: Entity conversion tests (NEXT)
○ P5 GREEN: Entity conversion implementation
○ P1 RED: FileParser tests
○ P1 GREEN: FileParser implementation
○ P2 RED/GREEN: State integration
○ P3 RED: Deletion tests
○ P4 RED/GREEN: Re-parsing logic
○ P6 RED/GREEN: Complete cycle
○ P7 RED/GREEN: Performance validation
○ REFACTOR: Cleanup and documentation
```

---

**END OF TDD SESSION STATE**

*This document is the single source of truth for the incremental reindex feature development. Update this file after every RED/GREEN/REFACTOR cycle.*
