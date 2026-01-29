# E2E Executable Specifications: Incremental Indexing Feature

> **Document ID**: E2E-SPECS-2026-01-29
> **Version**: 1.0.0
> **Created**: 2026-01-29
> **Status**: Specification Complete - Ready for Implementation

---

## Overview

This document defines executable specifications (WHEN...THEN...SHALL contracts) for end-to-end testing of the Parseltongue incremental indexing feature. These specifications verify that:

1. Initial indexing creates a valid graph from a fixture codebase
2. The HTTP server correctly serves indexed data
3. File modifications trigger hash-based change detection
4. The graph accurately reflects function additions
5. Dependency edges update correctly after changes
6. All operations meet performance contracts

### Architecture Context

- **Fixture Codebase**: `tests/e2e_fixtures/sample_codebase/` with ~8 CODE entities
- **Ingestion Method**: Programmatic via `FileStreamerImpl::stream_directory()`
- **HTTP Testing**: In-process via `tower::ServiceExt::oneshot` (no TCP binding)
- **Modification Target**: `calculator.rs` gains `divide()` function
- **Reindex Endpoint**: `POST /incremental-reindex-file-update?path=<absolute_path>`

---

## Fixture Codebase Structure

```
tests/e2e_fixtures/sample_codebase/
  src/
    main.rs           # 1 function: main()
    lib.rs            # 2 functions: init(), shutdown()
    calculator.rs     # 3 functions: add(), subtract(), multiply()
                      # After modification: 4 functions (+divide())
    utils/
      helpers.rs      # 2 functions: validate_input(), format_output()
  tests/
    unit_tests.rs     # 2 test functions (EXCLUDED from CODE graph)
```

### Expected Entity Counts

| State | CODE Entities | TEST Entities | Notes |
|-------|---------------|---------------|-------|
| Initial | 8 | 2 (excluded) | Only CODE entities indexed |
| After `divide()` added | 9 | 2 (excluded) | +1 function in calculator.rs |

---

## E2E-001: Initial Indexing Creates Valid Graph

### Problem Statement

Developers need confidence that the initial ingestion phase correctly parses all source files, extracts entities with proper ISGL1 keys, and stores them in CozoDB. Without this foundation, incremental updates have nothing to build upon.

### Specification

```
WHEN I call FileStreamerImpl::stream_directory() on sample_codebase/
  WITH root_dir = "tests/e2e_fixtures/sample_codebase"
  AND include_patterns = ["*.rs"]
  AND exclude_patterns = ["target/*"]
THEN SHALL create exactly 8 CODE entities in CodeGraph table
  AND SHALL NOT create entities from tests/unit_tests.rs (test exclusion)
  AND SHALL create entities with file_path containing absolute paths
  AND SHALL complete in < 5000ms for the fixture codebase
```

### Preconditions (GIVEN)

- Fresh in-memory CozoDB instance (`CozoDbStorage::new("mem")`)
- Schema created via `storage.create_schema().await`
- DependencyEdges schema created via `storage.create_dependency_edges_schema().await`
- Fixture codebase exists at expected path with all source files

### Test Data Requirements

| File | Expected Entities | Entity Names |
|------|-------------------|--------------|
| `src/main.rs` | 1 | `main` |
| `src/lib.rs` | 2 | `init`, `shutdown` |
| `src/calculator.rs` | 3 | `add`, `subtract`, `multiply` |
| `src/utils/helpers.rs` | 2 | `validate_input`, `format_output` |
| `tests/unit_tests.rs` | 0 (excluded) | test_add, test_subtract |

### Verification API Calls

```rust
// After stream_directory() completes:

// 1. Verify total CODE entity count
let count_result = storage.raw_query(
    "?[count(k)] := *CodeGraph{ISGL1_key: k, entity_class}, entity_class == 'CODE'"
).await;
assert_eq!(extract_count(count_result), 8);

// 2. Verify no TEST entities in database
let test_count = storage.raw_query(
    "?[count(k)] := *CodeGraph{ISGL1_key: k, entity_class}, entity_class == 'TEST'"
).await;
assert_eq!(extract_count(test_count), 0);

// 3. Verify specific entity exists with correct key format
let main_exists = storage.raw_query(
    "?[k] := *CodeGraph{ISGL1_key: k}, k ~ 'rust:fn:main:'"
).await;
assert!(main_exists.rows.len() >= 1);

// 4. Verify file_path is absolute
let paths = storage.raw_query(
    "?[fp] := *CodeGraph{file_path: fp}"
).await;
for row in paths.rows {
    let path = extract_string(&row[0]);
    assert!(path.starts_with('/'), "file_path must be absolute: {}", path);
}
```

### Edge Cases to Consider

| Scenario | Expected Behavior |
|----------|-------------------|
| Empty source file | Returns 0 entities (not error) |
| File with syntax errors | Partial extraction via tree-sitter |
| Nested modules | Entities have correct file_path |
| Re-exported items | Single entity at definition site |

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Total ingestion time | < 5000ms | `Instant::now()` around `stream_directory()` |
| Entities per second | > 50 | `entities_created / duration.as_secs_f64()` |
| Memory usage | < 100MB | (Not enforced in this test) |

### Verification Test Template

```rust
/// E2E-001: Initial indexing creates valid graph
///
/// # 4-Word Name: test_initial_indexing_creates_graph
#[tokio::test]
async fn e2e_001_initial_indexing_creates_valid_graph() {
    use std::time::Instant;
    use std::sync::Arc;
    use std::path::PathBuf;

    // GIVEN: Fresh database and fixture codebase path
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/e2e_fixtures/sample_codebase");
    assert!(fixture_path.exists(), "Fixture codebase must exist");

    // Create streamer configuration
    let config = StreamerConfig {
        root_dir: fixture_path.clone(),
        db_path: "mem".to_string(),
        include_patterns: vec!["*.rs".to_string()],
        exclude_patterns: vec!["target/*".to_string()],
        max_file_size: 1_000_000,
    };

    let key_generator = Arc::new(Isgl1KeyGeneratorImpl::new());
    let test_detector = Arc::new(DefaultTestDetector::new());

    // WHEN: Streaming the directory
    let start = Instant::now();
    let streamer = FileStreamerImpl::new(config, key_generator, test_detector)
        .await
        .unwrap();
    let result = streamer.stream_directory().await.unwrap();
    let duration = start.elapsed();

    // THEN: 8 CODE entities created
    assert_eq!(result.entities_created, 8,
        "Expected 8 CODE entities, got {}", result.entities_created);

    // AND: Completes within 5 seconds
    assert!(duration.as_millis() < 5000,
        "Ingestion took {}ms, expected < 5000ms", duration.as_millis());

    // AND: No errors during processing
    assert!(result.errors.is_empty(),
        "Unexpected errors: {:?}", result.errors);
}
```

### Acceptance Criteria

- [ ] Fixture codebase created with correct structure
- [ ] 8 CODE entities created from 4 source files
- [ ] 0 TEST entities in database (test exclusion working)
- [ ] All ISGL1 keys follow format: `rust:fn:<name>:<sanitized_path>:<line_range>`
- [ ] All file_path values are absolute paths
- [ ] Ingestion completes in < 5000ms

---

## E2E-002: HTTP Server Serves Indexed Data Correctly

### Problem Statement

After initial indexing, the HTTP server must provide accurate views of the graph. The `/code-entities-list-all` and `/codebase-statistics-overview-summary` endpoints must reflect the current database state.

### Specification

```
WHEN I start HTTP server with indexed database
  WITH database containing 8 CODE entities from E2E-001
THEN calling GET /codebase-statistics-overview-summary
  SHALL return data.code_entities_total_count == 8
  AND SHALL return data.test_entities_total_count == 0
  AND SHALL return success == true

AND calling GET /code-entities-list-all
  SHALL return data.total_count == 8
  AND SHALL return data.entities as array with length 8
  AND SHALL include entity with key containing "calculator"
  AND SHALL include entities: add, subtract, multiply (not divide yet)
```

### Preconditions (GIVEN)

- Database populated via E2E-001 (8 CODE entities)
- HTTP server state created via `SharedApplicationStateContainer::create_with_database_storage()`
- Router built via `build_complete_router_instance(state)`

### Verification API Calls

```rust
// Using tower::ServiceExt::oneshot for in-process testing

// 1. Statistics endpoint
let stats_response = app.clone()
    .oneshot(Request::builder()
        .uri("/codebase-statistics-overview-summary")
        .body(Body::empty())
        .unwrap())
    .await
    .unwrap();
assert_eq!(stats_response.status(), StatusCode::OK);
let stats_json: serde_json::Value = parse_body(stats_response).await;
assert_eq!(stats_json["data"]["code_entities_total_count"], 8);
assert_eq!(stats_json["data"]["test_entities_total_count"], 0);

// 2. List all entities endpoint
let list_response = app.clone()
    .oneshot(Request::builder()
        .uri("/code-entities-list-all")
        .body(Body::empty())
        .unwrap())
    .await
    .unwrap();
assert_eq!(list_response.status(), StatusCode::OK);
let list_json: serde_json::Value = parse_body(list_response).await;
assert_eq!(list_json["data"]["total_count"], 8);

// 3. Verify calculator entities present (before adding divide)
let entities = list_json["data"]["entities"].as_array().unwrap();
let calc_entities: Vec<_> = entities.iter()
    .filter(|e| e["file_path"].as_str().unwrap().contains("calculator"))
    .collect();
assert_eq!(calc_entities.len(), 3, "calculator.rs should have 3 functions initially");

// 4. Verify divide() does NOT exist yet
let has_divide = entities.iter()
    .any(|e| e["key"].as_str().unwrap().contains(":divide:"));
assert!(!has_divide, "divide() should not exist before modification");
```

### Edge Cases to Consider

| Scenario | Expected Behavior |
|----------|-------------------|
| Empty database | Returns count 0, empty array |
| Server restart | Reconnects to database, shows same data |
| Concurrent requests | All return consistent counts |

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Statistics query latency | < 100ms | Response time measurement |
| List all entities latency | < 200ms | Response time measurement |
| Response body size | < 50KB for 8 entities | Body length check |

### Verification Test Template

```rust
/// E2E-002: HTTP server serves indexed data correctly
///
/// # 4-Word Name: test_http_serves_indexed_data
#[tokio::test]
async fn e2e_002_http_server_serves_indexed_data_correctly() {
    // GIVEN: Database with 8 CODE entities (reuse E2E-001 setup or mock)
    let storage = setup_indexed_database().await; // Helper that creates 8 entities
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // WHEN: Calling statistics endpoint
    let stats_response = app.clone()
        .oneshot(Request::builder()
            .uri("/codebase-statistics-overview-summary")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    // THEN: Returns correct counts
    assert_eq!(stats_response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(stats_response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);
    assert_eq!(json["data"]["code_entities_total_count"], 8);
    assert_eq!(json["data"]["test_entities_total_count"], 0);

    // WHEN: Calling list all endpoint
    let list_response = app.clone()
        .oneshot(Request::builder()
            .uri("/code-entities-list-all")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    // THEN: Returns 8 entities
    assert_eq!(list_response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["data"]["total_count"], 8);
    let entities = json["data"]["entities"].as_array().unwrap();
    assert_eq!(entities.len(), 8);

    // AND: calculator has exactly 3 functions (add, subtract, multiply)
    let calc_count = entities.iter()
        .filter(|e| e["file_path"].as_str().unwrap_or("").contains("calculator"))
        .count();
    assert_eq!(calc_count, 3);
}
```

### Acceptance Criteria

- [ ] Statistics endpoint returns code_entities_total_count == 8
- [ ] Statistics endpoint returns test_entities_total_count == 0
- [ ] List endpoint returns exactly 8 entities
- [ ] All calculator.rs functions (add, subtract, multiply) are listed
- [ ] No divide() function exists before modification
- [ ] Response times under performance targets

---

## E2E-003: File Modification is Detected via Hash Change

### Problem Statement

When a file is modified, the incremental reindex endpoint must detect the change by comparing SHA-256 hashes. If the hash matches the cached value, no work is performed (early return). If it differs, the file is re-processed.

### Specification

```
WHEN I call POST /incremental-reindex-file-update?path=<calculator.rs>
  WITH file content unchanged from initial indexing
THEN SHALL return hash_changed: false
  AND SHALL return entities_added: 0, entities_removed: 0
  AND SHALL complete in < 100ms (cache hit performance)

WHEN I modify calculator.rs to add divide() function
  AND call POST /incremental-reindex-file-update?path=<calculator.rs>
THEN SHALL return hash_changed: true
  AND SHALL return entities_before: 3
  AND SHALL return entities_removed: 3 (old entities deleted)
  AND SHALL return entities_added: 4 (new entities inserted)
  AND SHALL complete in < 500ms
```

### Preconditions (GIVEN)

- Database populated via E2E-001
- Hash cache populated for calculator.rs during initial indexing
- FileParser available in server state for re-parsing

### Test Data: Modified calculator.rs

**Before (3 functions):**
```rust
// src/calculator.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
```

**After (4 functions):**
```rust
// src/calculator.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

pub fn divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}
```

### Verification API Calls

```rust
// Phase 1: Unchanged file - early return
let unchanged_response = app.clone()
    .oneshot(Request::builder()
        .method("POST")
        .uri(format!("/incremental-reindex-file-update?path={}", calc_path))
        .body(Body::empty())
        .unwrap())
    .await
    .unwrap();

let json: serde_json::Value = parse_body(unchanged_response).await;
assert_eq!(json["data"]["hash_changed"], false);
assert_eq!(json["data"]["entities_added"], 0);
assert_eq!(json["data"]["entities_removed"], 0);
assert!(json["data"]["processing_time_ms"].as_u64().unwrap() < 100);

// Phase 2: Modify file
std::fs::write(&calc_path, MODIFIED_CALCULATOR_CONTENT).unwrap();

// Phase 3: Changed file - full reindex
let changed_response = app.clone()
    .oneshot(Request::builder()
        .method("POST")
        .uri(format!("/incremental-reindex-file-update?path={}", calc_path))
        .body(Body::empty())
        .unwrap())
    .await
    .unwrap();

let json: serde_json::Value = parse_body(changed_response).await;
assert_eq!(json["data"]["hash_changed"], true);
assert_eq!(json["data"]["entities_before"], 3);
assert_eq!(json["data"]["entities_removed"], 3);
assert_eq!(json["data"]["entities_added"], 4);
assert!(json["data"]["processing_time_ms"].as_u64().unwrap() < 500);
```

### Edge Cases to Consider

| Scenario | Expected Behavior |
|----------|-------------------|
| File deleted after hash cached | Return 404 "File not found" |
| Whitespace-only changes | Hash changes, but same entities |
| Comment-only changes | Hash changes, entity count same |
| File permissions denied | Return 500 with error message |

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Unchanged file (cache hit) | < 100ms | `processing_time_ms` in response |
| Changed file with 4 entities | < 500ms | `processing_time_ms` in response |
| Hash computation (10KB file) | < 10ms | Internal timing |

### Verification Test Template

```rust
/// E2E-003: File modification is detected via hash change
///
/// # 4-Word Name: test_file_modification_detected_hash
#[tokio::test]
async fn e2e_003_file_modification_detected_via_hash_change() {
    use std::io::Write;
    use tempfile::TempDir;

    // GIVEN: Database with indexed calculator.rs
    let temp_dir = TempDir::new().unwrap();
    let calc_path = setup_temp_fixture(&temp_dir).await;
    let storage = index_fixture_to_database(&temp_dir.path()).await;
    let state = SharedApplicationStateContainer::create_with_database_storage(storage);
    let app = build_complete_router_instance(state);

    // Prime the hash cache by calling reindex once
    let _ = app.clone()
        .oneshot(Request::builder()
            .method("POST")
            .uri(format!("/incremental-reindex-file-update?path={}", calc_path.display()))
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    // WHEN: Calling reindex on unchanged file
    let unchanged_response = app.clone()
        .oneshot(Request::builder()
            .method("POST")
            .uri(format!("/incremental-reindex-file-update?path={}", calc_path.display()))
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    // THEN: Early return with hash_changed: false
    let body = axum::body::to_bytes(unchanged_response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["data"]["hash_changed"], false);
    assert_eq!(json["data"]["entities_added"], 0);
    assert!(json["data"]["processing_time_ms"].as_u64().unwrap() < 100,
        "Unchanged file should return in <100ms");

    // WHEN: Modifying file to add divide()
    let modified_content = include_str!("fixtures/calculator_with_divide.rs");
    std::fs::write(&calc_path, modified_content).unwrap();

    // AND: Calling reindex on changed file
    let changed_response = app.clone()
        .oneshot(Request::builder()
            .method("POST")
            .uri(format!("/incremental-reindex-file-update?path={}", calc_path.display()))
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    // THEN: Full reindex with hash_changed: true
    let body = axum::body::to_bytes(changed_response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["data"]["hash_changed"], true);
    assert_eq!(json["data"]["entities_before"], 3);
    assert_eq!(json["data"]["entities_removed"], 3);
    assert_eq!(json["data"]["entities_added"], 4);
    assert!(json["data"]["processing_time_ms"].as_u64().unwrap() < 500,
        "Changed file should complete in <500ms");
}
```

### Acceptance Criteria

- [ ] Unchanged file returns hash_changed: false in < 100ms
- [ ] Modified file returns hash_changed: true
- [ ] entities_before correctly reports 3
- [ ] entities_removed correctly reports 3
- [ ] entities_added correctly reports 4
- [ ] Processing completes in < 500ms

---

## E2E-004: Graph Reflects Function Additions Correctly

### Problem Statement

After incremental reindex, the graph must accurately reflect the new state of the file. The `divide()` function must be queryable, and the entity count must update to 9 total CODE entities.

### Specification

```
WHEN incremental reindex of modified calculator.rs completes (E2E-003)
THEN GET /code-entities-list-all
  SHALL return data.total_count == 9 (was 8, now +1)
  AND SHALL include entity with key containing ":divide:"
  AND SHALL include entity with name "divide"

AND GET /code-entity-detail-view/{divide_key}
  SHALL return Current_Code containing "pub fn divide"
  AND SHALL return entity_type == "function"
  AND SHALL return file_path containing "calculator.rs"
```

### Preconditions (GIVEN)

- E2E-003 completed successfully
- Database now contains 9 CODE entities
- `divide()` entity exists with ISGL1 key

### Verification API Calls

```rust
// 1. Verify total count increased to 9
let list_response = app.clone()
    .oneshot(Request::builder()
        .uri("/code-entities-list-all")
        .body(Body::empty())
        .unwrap())
    .await
    .unwrap();

let json: serde_json::Value = parse_body(list_response).await;
assert_eq!(json["data"]["total_count"], 9);

// 2. Find the divide entity
let entities = json["data"]["entities"].as_array().unwrap();
let divide_entity = entities.iter()
    .find(|e| e["key"].as_str().unwrap().contains(":divide:"))
    .expect("divide() entity must exist");

// 3. Verify divide entity properties
let divide_key = divide_entity["key"].as_str().unwrap();
assert!(divide_key.starts_with("rust:fn:divide:"));
assert_eq!(divide_entity["entity_type"], "function");
assert!(divide_entity["file_path"].as_str().unwrap().contains("calculator.rs"));

// 4. Get detailed view of divide()
let detail_response = app.clone()
    .oneshot(Request::builder()
        .uri(format!("/code-entity-detail-view/{}", urlencoded(divide_key)))
        .body(Body::empty())
        .unwrap())
    .await
    .unwrap();

let detail_json: serde_json::Value = parse_body(detail_response).await;
assert_eq!(detail_json["success"], true);

let current_code = detail_json["data"]["current_code"].as_str().unwrap();
assert!(current_code.contains("pub fn divide"));
assert!(current_code.contains("Option<i32>"));
```

### Edge Cases to Consider

| Scenario | Expected Behavior |
|----------|-------------------|
| Function renamed | Old entity removed, new entity added |
| Function deleted | Entity count decreases |
| Function signature changed | Same entity key, updated code |

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| List query after update | < 200ms | Response time |
| Detail view query | < 100ms | Response time |
| Entity key lookup | < 50ms | Database query time |

### Verification Test Template

```rust
/// E2E-004: Graph reflects function additions correctly
///
/// # 4-Word Name: test_graph_reflects_function_additions
#[tokio::test]
async fn e2e_004_graph_reflects_function_additions_correctly() {
    // GIVEN: Database after E2E-003 (divide() added to calculator.rs)
    let (storage, app) = setup_after_divide_added().await;

    // WHEN: Querying all entities
    let list_response = app.clone()
        .oneshot(Request::builder()
            .uri("/code-entities-list-all")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    // THEN: Total count is 9
    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["data"]["total_count"], 9,
        "Expected 9 entities after adding divide()");

    // AND: divide() entity exists
    let entities = json["data"]["entities"].as_array().unwrap();
    let divide_entity = entities.iter()
        .find(|e| {
            let key = e["key"].as_str().unwrap_or("");
            key.contains(":divide:")
        })
        .expect("divide() entity must exist in graph");

    // AND: Entity has correct properties
    assert_eq!(divide_entity["entity_type"], "function");
    assert!(divide_entity["file_path"].as_str().unwrap().contains("calculator.rs"));

    // WHEN: Getting detail view of divide()
    let divide_key = divide_entity["key"].as_str().unwrap();
    let encoded_key = urlencoding::encode(divide_key);

    let detail_response = app.clone()
        .oneshot(Request::builder()
            .uri(format!("/code-entity-detail-view/{}", encoded_key))
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    // THEN: Returns divide() code
    let body = axum::body::to_bytes(detail_response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["success"], true);

    let current_code = json["data"]["current_code"].as_str().unwrap_or("");
    assert!(current_code.contains("pub fn divide"),
        "Current code should contain divide function definition");
    assert!(current_code.contains("Option<i32>"),
        "Current code should contain return type");
}
```

### Acceptance Criteria

- [ ] Total entity count is 9 (was 8)
- [ ] divide() entity found in list
- [ ] divide() has correct ISGL1 key format
- [ ] divide() has entity_type "function"
- [ ] divide() has file_path containing "calculator.rs"
- [ ] Detail view returns correct Current_Code

---

## E2E-005: Edge Updates Cascade Correctly

### Problem Statement

When `calculator.rs` is modified, not only must the entities update, but any dependency edges must also be updated. Old edges from deleted entities must be removed, and new edges from added entities must be inserted.

### Specification

```
WHEN calculator.rs initially has add() calling validate_input()
  AND incremental reindex adds divide() which also calls validate_input()
THEN the DependencyEdges table
  SHALL contain edge from add() -> validate_input() (preserved)
  AND SHALL contain NEW edge from divide() -> validate_input() (added)
  AND SHALL NOT contain edges from old entities that were deleted

SPECIFICALLY:
WHEN I query GET /dependency-edges-list-all after reindex
THEN SHALL return edges including:
  - from_key containing "divide" -> to_key containing "validate_input"
AND response.data.edges_added in reindex response SHALL be >= 1
```

### Preconditions (GIVEN)

- E2E-004 completed successfully
- divide() function calls validate_input() internally
- Initial calculator.rs functions also reference helpers

### Test Data: Modified calculator.rs with Dependencies

```rust
// src/calculator.rs (modified with divide() that has dependency)
use crate::utils::helpers::validate_input;

pub fn add(a: i32, b: i32) -> i32 {
    validate_input(a);
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

pub fn divide(a: i32, b: i32) -> Option<i32> {
    validate_input(b);  // New dependency!
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}
```

### Verification API Calls

```rust
// 1. Check reindex response for edge counts
let reindex_json = // ... from E2E-003
assert!(reindex_json["data"]["edges_added"].as_u64().unwrap() >= 1,
    "At least one new edge should be added for divide() -> validate_input()");

// 2. Query all edges
let edges_response = app.clone()
    .oneshot(Request::builder()
        .uri("/dependency-edges-list-all")
        .body(Body::empty())
        .unwrap())
    .await
    .unwrap();

let edges_json: serde_json::Value = parse_body(edges_response).await;
let edges = edges_json["data"]["edges"].as_array().unwrap();

// 3. Find edge from divide to validate_input
let divide_to_validate = edges.iter()
    .find(|e| {
        let from = e["from_key"].as_str().unwrap_or("");
        let to = e["to_key"].as_str().unwrap_or("");
        from.contains(":divide:") && to.contains(":validate_input:")
    });

assert!(divide_to_validate.is_some(),
    "Edge from divide() to validate_input() must exist");

// 4. Verify old edges from deleted entities are gone
// (Query for edges from entities that no longer exist)
let orphan_edges = edges.iter()
    .filter(|e| {
        let from = e["from_key"].as_str().unwrap_or("");
        // Check for edges from entities that were in old calculator.rs
        // but shouldn't exist after clean reindex
        from.contains("OLD_ENTITY_PREFIX") // placeholder
    })
    .count();

assert_eq!(orphan_edges, 0, "No orphan edges should remain");
```

### Edge Cases to Consider

| Scenario | Expected Behavior |
|----------|-------------------|
| Circular dependency introduced | Edge created both directions |
| External dependency (stdlib) | Edge to unresolved key |
| Self-referential function | Edge from key to same key |
| Removed dependency | Edge deleted |

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Edge deletion batch | < 50ms | `edges_removed` timing |
| Edge insertion batch | < 100ms | `edges_added` timing |
| Edge query (all) | < 200ms | Response time |

### Verification Test Template

```rust
/// E2E-005: Edge updates cascade correctly
///
/// # 4-Word Name: test_edge_updates_cascade_correctly
#[tokio::test]
async fn e2e_005_edge_updates_cascade_correctly() {
    // GIVEN: Database after E2E-004 with divide() calling validate_input()
    let (storage, app, reindex_response) = setup_with_divide_and_dependencies().await;

    // THEN: Reindex response shows edges_added >= 1
    let reindex_json = reindex_response;
    assert!(reindex_json["data"]["edges_added"].as_u64().unwrap() >= 1,
        "At least one edge should be added for divide() dependencies");

    // WHEN: Querying all edges
    let edges_response = app.clone()
        .oneshot(Request::builder()
            .uri("/dependency-edges-list-all")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    // THEN: Edge from divide to validate_input exists
    let body = axum::body::to_bytes(edges_response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let edges = json["data"]["edges"].as_array().unwrap();

    let divide_edge = edges.iter().find(|e| {
        let from = e["from_key"].as_str().unwrap_or("");
        let to = e["to_key"].as_str().unwrap_or("");
        from.contains(":divide:") && to.contains(":validate_input:")
    });

    assert!(divide_edge.is_some(),
        "Edge divide() -> validate_input() must exist");

    // AND: Edge type is correct
    let edge = divide_edge.unwrap();
    assert_eq!(edge["edge_type"], "Calls",
        "Edge type should be 'Calls'");

    // AND: No orphan edges exist (verify cleanup)
    // Count edges from calculator.rs entities
    let calc_edges: Vec<_> = edges.iter()
        .filter(|e| e["from_key"].as_str().unwrap_or("").contains("calculator"))
        .collect();

    // Should have edges from all 4 calculator functions (if they have deps)
    assert!(calc_edges.len() >= 1,
        "Calculator functions should have dependency edges");
}
```

### Acceptance Criteria

- [ ] Reindex response shows edges_added >= 1
- [ ] Edge from divide() to validate_input() exists
- [ ] Edge type is "Calls"
- [ ] Old edges from deleted entities are removed
- [ ] No orphan edges in database
- [ ] Edge query returns in < 200ms

---

## E2E-006: Full Cycle Performance Contract is Met

### Problem Statement

The complete E2E cycle - from initial indexing through modification and reindex - must meet the performance contracts defined in PRD-2026-01-28. This test measures the full workflow timing.

### Specification

```
WHEN I execute the complete E2E workflow:
  1. Create fresh database
  2. Index fixture codebase (8 entities)
  3. Query statistics and list
  4. Modify calculator.rs
  5. Call incremental reindex
  6. Verify graph update
THEN the complete cycle
  SHALL complete in < 5000ms total
  AND unchanged file cache hit SHALL be < 100ms
  AND changed file reindex SHALL be < 500ms
  AND no step SHALL exceed 2000ms individually
```

### Preconditions (GIVEN)

- Clean test environment
- Fixture codebase available
- No external network calls

### Performance Breakdown

| Phase | Target | Contains |
|-------|--------|----------|
| Database setup | < 500ms | Schema creation |
| Initial indexing | < 2000ms | Parse 5 files, insert 8 entities |
| HTTP queries (2x) | < 500ms | Statistics + List all |
| File modification | < 50ms | Write 1 file |
| Incremental reindex | < 500ms | Hash, delete, parse, insert |
| Verification queries | < 500ms | Final checks |
| **Total** | **< 5000ms** | Full E2E cycle |

### Verification API Calls

```rust
let total_start = Instant::now();

// Phase 1: Setup
let setup_start = Instant::now();
let storage = CozoDbStorage::new("mem").await.unwrap();
storage.create_schema().await.unwrap();
storage.create_dependency_edges_schema().await.unwrap();
let setup_duration = setup_start.elapsed();
assert!(setup_duration.as_millis() < 500, "Setup: {}ms", setup_duration.as_millis());

// Phase 2: Initial indexing
let index_start = Instant::now();
let result = streamer.stream_directory().await.unwrap();
let index_duration = index_start.elapsed();
assert!(index_duration.as_millis() < 2000, "Indexing: {}ms", index_duration.as_millis());
assert_eq!(result.entities_created, 8);

// Phase 3: HTTP queries
let query_start = Instant::now();
let _ = query_stats(&app).await;
let _ = query_list(&app).await;
let query_duration = query_start.elapsed();
assert!(query_duration.as_millis() < 500, "Queries: {}ms", query_duration.as_millis());

// Phase 4: File modification
let mod_start = Instant::now();
std::fs::write(&calc_path, MODIFIED_CONTENT).unwrap();
let mod_duration = mod_start.elapsed();
assert!(mod_duration.as_millis() < 50, "Modification: {}ms", mod_duration.as_millis());

// Phase 5: Incremental reindex
let reindex_start = Instant::now();
let reindex_response = call_incremental_reindex(&app, &calc_path).await;
let reindex_duration = reindex_start.elapsed();
assert!(reindex_duration.as_millis() < 500, "Reindex: {}ms", reindex_duration.as_millis());

// Phase 6: Verification
let verify_start = Instant::now();
let final_count = query_entity_count(&app).await;
assert_eq!(final_count, 9);
let verify_duration = verify_start.elapsed();
assert!(verify_duration.as_millis() < 500, "Verify: {}ms", verify_duration.as_millis());

// Total
let total_duration = total_start.elapsed();
assert!(total_duration.as_millis() < 5000,
    "Total E2E cycle: {}ms, expected < 5000ms", total_duration.as_millis());
```

### Performance Contract Summary

| Metric | Target | Actual (placeholder) |
|--------|--------|----------------------|
| Unchanged file (cache hit) | < 100ms | TBD |
| Changed file with <100 entities | < 500ms | TBD |
| Full E2E cycle | < 5000ms | TBD |
| Database setup | < 500ms | TBD |
| Initial indexing (8 entities) | < 2000ms | TBD |
| HTTP queries | < 500ms | TBD |

### Verification Test Template

```rust
/// E2E-006: Full cycle performance contract is met
///
/// # 4-Word Name: test_full_cycle_performance_contract
#[tokio::test]
async fn e2e_006_full_cycle_performance_contract_is_met() {
    use std::time::Instant;
    use tempfile::TempDir;

    // Track all phase timings
    let mut timings: Vec<(&str, u128)> = Vec::new();
    let total_start = Instant::now();

    // Phase 1: Database setup
    let phase_start = Instant::now();
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();
    storage.create_dependency_edges_schema().await.unwrap();
    storage.create_file_hash_cache_schema().await.unwrap();
    timings.push(("Database setup", phase_start.elapsed().as_millis()));
    assert!(timings.last().unwrap().1 < 500,
        "Database setup: {}ms > 500ms", timings.last().unwrap().1);

    // Phase 2: Create fixture and index
    let phase_start = Instant::now();
    let temp_dir = TempDir::new().unwrap();
    create_fixture_codebase(&temp_dir).await;
    let streamer = create_streamer(&temp_dir, &storage).await;
    let result = streamer.stream_directory().await.unwrap();
    timings.push(("Initial indexing", phase_start.elapsed().as_millis()));
    assert!(timings.last().unwrap().1 < 2000,
        "Initial indexing: {}ms > 2000ms", timings.last().unwrap().1);
    assert_eq!(result.entities_created, 8);

    // Phase 3: HTTP queries
    let phase_start = Instant::now();
    let state = SharedApplicationStateContainer::create_with_database_storage(storage.clone());
    let app = build_complete_router_instance(state);

    let stats = query_stats(&app).await;
    assert_eq!(stats["data"]["code_entities_total_count"], 8);

    let list = query_list(&app).await;
    assert_eq!(list["data"]["total_count"], 8);
    timings.push(("HTTP queries", phase_start.elapsed().as_millis()));
    assert!(timings.last().unwrap().1 < 500,
        "HTTP queries: {}ms > 500ms", timings.last().unwrap().1);

    // Phase 4: File modification
    let phase_start = Instant::now();
    let calc_path = temp_dir.path()
        .join("src")
        .join("calculator.rs");
    let modified_content = include_str!("fixtures/calculator_with_divide.rs");
    std::fs::write(&calc_path, modified_content).unwrap();
    timings.push(("File modification", phase_start.elapsed().as_millis()));
    assert!(timings.last().unwrap().1 < 50,
        "File modification: {}ms > 50ms", timings.last().unwrap().1);

    // Phase 5: Incremental reindex
    let phase_start = Instant::now();
    let reindex_response = app.clone()
        .oneshot(Request::builder()
            .method("POST")
            .uri(format!("/incremental-reindex-file-update?path={}", calc_path.display()))
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    timings.push(("Incremental reindex", phase_start.elapsed().as_millis()));
    assert!(timings.last().unwrap().1 < 500,
        "Incremental reindex: {}ms > 500ms", timings.last().unwrap().1);

    let body = axum::body::to_bytes(reindex_response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["hash_changed"], true);
    assert_eq!(json["data"]["entities_added"], 4);

    // Phase 6: Verification
    let phase_start = Instant::now();
    let final_list = query_list(&app).await;
    assert_eq!(final_list["data"]["total_count"], 9);

    let has_divide = final_list["data"]["entities"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["key"].as_str().unwrap_or("").contains(":divide:"));
    assert!(has_divide, "divide() must exist after reindex");
    timings.push(("Verification", phase_start.elapsed().as_millis()));

    // Total timing
    let total_duration = total_start.elapsed();
    timings.push(("TOTAL", total_duration.as_millis()));

    // Print timing report
    println!("\n=== E2E Performance Report ===");
    for (phase, ms) in &timings {
        println!("  {}: {}ms", phase, ms);
    }
    println!("===============================\n");

    // Final assertion
    assert!(total_duration.as_millis() < 5000,
        "Total E2E cycle: {}ms, expected < 5000ms", total_duration.as_millis());
}
```

### Acceptance Criteria

- [ ] Total E2E cycle completes in < 5000ms
- [ ] Database setup < 500ms
- [ ] Initial indexing < 2000ms
- [ ] HTTP queries < 500ms
- [ ] File modification < 50ms
- [ ] Incremental reindex < 500ms
- [ ] No single phase exceeds 2000ms
- [ ] All assertions pass

---

## Fixture Files to Create

### Directory Structure

```
tests/e2e_fixtures/
  sample_codebase/
    src/
      main.rs
      lib.rs
      calculator.rs
      utils/
        helpers.rs
        mod.rs
    tests/
      unit_tests.rs
    Cargo.toml
  fixtures/
    calculator_with_divide.rs   # Modified version for E2E-003+
```

### File Contents

#### `tests/e2e_fixtures/sample_codebase/src/main.rs`
```rust
//! Main entry point

use crate::lib::{init, shutdown};
use crate::calculator::add;

fn main() {
    init();
    let result = add(1, 2);
    println!("Result: {}", result);
    shutdown();
}
```

#### `tests/e2e_fixtures/sample_codebase/src/lib.rs`
```rust
//! Library root

pub mod calculator;
pub mod utils;

pub fn init() {
    println!("Initializing...");
}

pub fn shutdown() {
    println!("Shutting down...");
}
```

#### `tests/e2e_fixtures/sample_codebase/src/calculator.rs`
```rust
//! Calculator module with basic arithmetic

use crate::utils::helpers::validate_input;

pub fn add(a: i32, b: i32) -> i32 {
    validate_input(a);
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
```

#### `tests/e2e_fixtures/sample_codebase/src/utils/mod.rs`
```rust
//! Utility modules

pub mod helpers;
```

#### `tests/e2e_fixtures/sample_codebase/src/utils/helpers.rs`
```rust
//! Helper functions

pub fn validate_input(value: i32) {
    if value < 0 {
        panic!("Negative values not allowed");
    }
}

pub fn format_output(value: i32) -> String {
    format!("Result: {}", value)
}
```

#### `tests/e2e_fixtures/sample_codebase/tests/unit_tests.rs`
```rust
//! Unit tests (should be EXCLUDED from CODE graph)

#[test]
fn test_add() {
    assert_eq!(crate::calculator::add(1, 2), 3);
}

#[test]
fn test_subtract() {
    assert_eq!(crate::calculator::subtract(5, 3), 2);
}
```

#### `tests/e2e_fixtures/sample_codebase/Cargo.toml`
```toml
[package]
name = "sample_codebase"
version = "0.1.0"
edition = "2021"

[dependencies]
```

#### `tests/e2e_fixtures/fixtures/calculator_with_divide.rs`
```rust
//! Calculator module with basic arithmetic (+ divide)

use crate::utils::helpers::validate_input;

pub fn add(a: i32, b: i32) -> i32 {
    validate_input(a);
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

pub fn divide(a: i32, b: i32) -> Option<i32> {
    validate_input(b);
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}
```

---

## Summary of All Specifications

| Spec ID | Title | Key Assertion | Performance Target |
|---------|-------|---------------|-------------------|
| E2E-001 | Initial indexing creates valid graph | 8 CODE entities created | < 5000ms |
| E2E-002 | HTTP server serves indexed data | Statistics show count 8 | < 200ms per query |
| E2E-003 | File modification detected via hash | hash_changed: true | < 500ms reindex |
| E2E-004 | Graph reflects function additions | divide() entity exists, count 9 | < 200ms query |
| E2E-005 | Edge updates cascade correctly | divide -> validate_input edge | < 200ms query |
| E2E-006 | Full cycle performance met | All phases within targets | < 5000ms total |

---

## Implementation Checklist

- [ ] Create `tests/e2e_fixtures/sample_codebase/` directory structure
- [ ] Create all fixture source files
- [ ] Create `calculator_with_divide.rs` modified fixture
- [ ] Implement E2E-001 test
- [ ] Implement E2E-002 test
- [ ] Implement E2E-003 test
- [ ] Implement E2E-004 test
- [ ] Implement E2E-005 test
- [ ] Implement E2E-006 test
- [ ] All tests pass with `cargo test e2e`
- [ ] Performance contracts validated

---

*Document generated following WHEN...THEN...SHALL contract format and TDD-First methodology.*
*Based on PRD-2026-01-28 and existing INCREMENTAL_INDEXING_EXECUTABLE_SPECS.md*
