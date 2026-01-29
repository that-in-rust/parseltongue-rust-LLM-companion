# TDD Progress Tracker: Incremental Reindex Feature

**Last Updated**: 2026-01-28 (Session Start)
**Current Phase**: STUB → RED (Ready to write tests)

---

## Quick Status Dashboard

```
████░░░░░░░░░░░░░░░░ 20% Complete

Phase Status:
  STUB ✓ Complete - Feature spec documented
  RED  ⏸ Ready to start - Tests not written yet
  GREEN ⏳ Pending - Awaiting RED completion
  REFACTOR ⏳ Pending - Awaiting GREEN completion
```

---

## Test Checklist (12 Total Tests)

### Database Layer (5 tests)
```
File: crates/parseltongue-core/tests/incremental_reindex_storage_tests.rs
Status: FILE NOT CREATED

[ ] Test 1: test_get_entities_by_file_path_returns_all_entities
    Phase: RED | Blocked by: None | Est: 5min

[ ] Test 2: test_delete_entities_batch_by_keys_removes_multiple
    Phase: RED | Blocked by: None | Est: 7min

[ ] Test 3: test_delete_edges_by_from_keys_removes_outgoing
    Phase: RED | Blocked by: None | Est: 8min

[ ] Test 4: test_get_cached_file_hash_value_retrieves_hash
    Phase: RED | Blocked by: FileHashCache schema | Est: 5min

[ ] Test 5: test_set_cached_file_hash_value_stores_hash
    Phase: RED | Blocked by: FileHashCache schema | Est: 5min
```

### HTTP Handler Layer (5 tests)
```
File: crates/pt08-http-code-query-server/tests/http_server_integration_tests.rs
Status: FILE EXISTS, TESTS NOT ADDED

[ ] Test 6: test_incremental_reindex_returns_400_missing_path
    Phase: RED | Blocked by: Tests 1-5 GREEN | Est: 4min

[ ] Test 7: test_incremental_reindex_returns_404_nonexistent_file
    Phase: RED | Blocked by: Tests 1-5 GREEN | Est: 4min

[ ] Test 8: test_incremental_reindex_returns_unchanged_for_same_hash
    Phase: RED | Blocked by: Tests 1-5 GREEN | Est: 6min

[ ] Test 9: test_incremental_reindex_returns_diff_for_changed_file
    Phase: RED | Blocked by: Tests 1-5 GREEN | Est: 8min

[ ] Test 10: test_incremental_reindex_handles_file_deletion_gracefully
    Phase: RED | Blocked by: Tests 1-5 GREEN | Est: 5min
```

### Integration Layer (2 tests)
```
File: crates/pt08-http-code-query-server/tests/incremental_reindex_integration_tests.rs
Status: FILE NOT CREATED

[ ] Test 11: test_full_incremental_reindex_workflow_end_to_end
    Phase: RED | Blocked by: Tests 1-10 GREEN | Est: 15min

[ ] Test 12: test_incremental_reindex_performance_under_500ms
    Phase: RED | Blocked by: Tests 1-10 GREEN | Est: 10min
```

---

## Implementation Checklist (5 Components)

### Storage Methods (5 methods)
```
File: crates/parseltongue-core/src/storage/cozo_client.rs
Status: METHODS NOT IMPLEMENTED

[ ] Method 1: create_file_hash_cache_schema()
    Lines: ~15 | Complexity: Low | Blocked by: None

[ ] Method 2: get_entities_by_file_path()
    Lines: ~30 | Complexity: Low | Blocked by: None

[ ] Method 3: delete_entities_batch_by_keys()
    Lines: ~35 | Complexity: Medium | Blocked by: None

[ ] Method 4: delete_edges_by_from_keys()
    Lines: ~35 | Complexity: Medium | Blocked by: None

[ ] Method 5: get_cached_file_hash_value()
    Lines: ~25 | Complexity: Low | Blocked by: Method 1

[ ] Method 6: set_cached_file_hash_value()
    Lines: ~30 | Complexity: Low | Blocked by: Method 1
```

### HTTP Handler (1 handler)
```
File: crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/incremental_reindex_file_handler.rs
Status: FILE NOT CREATED

[ ] Handler: handle_incremental_reindex_file_update()
    Lines: ~100 | Complexity: High | Blocked by: Storage methods GREEN
    Components:
      - Request validation
      - Hash computation (SHA-256)
      - File path resolution
      - Entity diff calculation
      - Database operations orchestration
      - Response formatting
```

### Route Registration (1 route)
```
File: crates/pt08-http-code-query-server/src/route_definition_builder_module.rs
Status: ROUTE NOT ADDED

[ ] Route: POST /incremental-reindex-file-update
    Lines: ~4 | Complexity: Trivial | Blocked by: Handler implementation
```

### Module Declaration (1 module)
```
File: crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/mod.rs
Status: MODULE NOT DECLARED

[ ] Module: pub mod incremental_reindex_file_handler;
    Lines: ~1 | Complexity: Trivial | Blocked by: Handler file creation
```

---

## Dependency Graph

```
Test Layer           Implementation Layer
───────────          ────────────────────

Test 1 ─────────────► get_entities_by_file_path()
Test 2 ─────────────► delete_entities_batch_by_keys()
Test 3 ─────────────► delete_edges_by_from_keys()
Test 4 ──┐
         ├─────────► create_file_hash_cache_schema()
Test 5 ──┘           ├─► get_cached_file_hash_value()
                     └─► set_cached_file_hash_value()

Tests 1-5 GREEN
    │
    ▼
Tests 6-10 RED ─────► handle_incremental_reindex_file_update()
                                │
                                ├─► Route registration
                                └─► Module declaration

Tests 6-10 GREEN
    │
    ▼
Tests 11-12 RED ────► Integration validation
```

---

## Time Tracking

| Phase | Estimated | Actual | Variance |
|-------|-----------|--------|----------|
| STUB | 30 min | 30 min | ✓ On time |
| RED (Tests 1-5) | 30 min | TBD | - |
| GREEN (Storage) | 45 min | TBD | - |
| RED (Tests 6-10) | 25 min | TBD | - |
| GREEN (Handler) | 60 min | TBD | - |
| RED (Tests 11-12) | 25 min | TBD | - |
| GREEN (Integration) | 15 min | TBD | - |
| REFACTOR | 30 min | TBD | - |
| **TOTAL** | **4h 20m** | **30m** | **-** |

---

## Current Blockers

**NONE** - Ready to proceed with Test 1

---

## Next Action

**IMMEDIATE**: Create test file and write Test 1

```bash
# 1. Create test file
touch crates/parseltongue-core/tests/incremental_reindex_storage_tests.rs

# 2. Add to the file:
cat > crates/parseltongue-core/tests/incremental_reindex_storage_tests.rs << 'EOF'
//! Tests for incremental file reindexing storage operations
//!
//! TDD Phase: RED - These tests verify database methods for:
//! - Querying entities by file path
//! - Batch entity deletion
//! - Edge cascade deletion
//! - File hash caching

use parseltongue_core::storage::CozoDbStorage;
use parseltongue_core::entities::*;

#[tokio::test]
async fn test_get_entities_by_file_path_returns_all_entities() {
    // Arrange
    let storage = CozoDbStorage::new("mem").await.unwrap();
    storage.create_schema().await.unwrap();

    // TODO: Insert 3 entities from "src/main.rs"
    // TODO: Insert 2 entities from "src/lib.rs"

    // Act
    let result = storage.get_entities_by_file_path("src/main.rs").await.unwrap();

    // Assert
    assert_eq!(result.len(), 3, "Should return exactly 3 entities from src/main.rs");
}
EOF

# 3. Run the test (expect compilation error - method doesn't exist)
cargo test test_get_entities_by_file_path_returns_all_entities
```

**Expected Output**: Compilation error - `no method named 'get_entities_by_file_path'`

This confirms we're in RED phase.

---

## Session Commands

### Resume Development
```bash
cd /Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator
cat TDD_PROGRESS_TRACKER.md  # Read this file
cargo test incremental_reindex  # Check current test status
git status  # Check for uncommitted work
```

### Run Specific Test Suites
```bash
# Storage layer only
cargo test -p parseltongue-core incremental_reindex

# HTTP layer only
cargo test -p pt08-http-code-query-server incremental_reindex

# All tests
cargo test incremental_reindex
```

### Check Implementation Status
```bash
# Check if storage methods exist
grep -n "get_entities_by_file_path\|delete_entities_batch\|delete_edges_by_from" \
  crates/parseltongue-core/src/storage/cozo_client.rs

# Check if handler exists
ls -la crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/incremental_reindex_file_handler.rs
```

---

## Critical Reminders

1. **4-Word Naming**: ALL names must be exactly 4 words
   - ✅ `handle_incremental_reindex_file_update`
   - ❌ `handle_reindex` (too short)

2. **TDD Discipline**: NEVER implement before test fails
   - Write test → Run → See RED → Implement → Run → See GREEN

3. **Commit After GREEN**: Each test/implementation pair = 1 commit

4. **Zero TODOs**: No `TODO`, `STUB`, `PLACEHOLDER` in final code

5. **Performance Target**: < 500ms end-to-end (measure in Test 12)

---

**Quick Status**: ████░░░░░░░░░░░░░░░░ 20% | Next: Write Test 1 (5 min)
