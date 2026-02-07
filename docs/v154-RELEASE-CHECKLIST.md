# v1.5.4 Release Checklist

**Date**: 2026-02-07
**Fix**: RwLock Deadlock in HTTP Endpoints

---

## Summary

Fixed critical deadlock bug where HTTP endpoints would hang indefinitely due to holding RwLock read guard across `.await` boundary.

---

## Pre-Release Verification

### Build
- [x] `cargo clean` - Removed 305MB
- [x] `cargo build --release` - Success (1m 40s)
- [x] No compiler warnings in parseltongue crates

### Ingestion Test
- [x] Self-ingestion: `parseltongue pt01 .`
- [x] Time: **2.63s** (comparable to baseline)
- [x] Entities: 2,529 (1,074 CODE after test exclusion)
- [x] Edges: 7,193

### All 14 Endpoints

| # | Endpoint | Status | Notes |
|---|----------|--------|-------|
| 1 | `/server-health-check-status` | ✅ | ok |
| 2 | `/codebase-statistics-overview-summary` | ✅ | 1,074 entities, 7,193 edges |
| 3 | `/api-reference-documentation-help` | ✅ | |
| 4 | `/code-entities-list-all` | ✅ | |
| 5 | `/code-entity-detail-view` | ✅ | **FIXED** (was hanging) |
| 6 | `/code-entities-search-fuzzy` | ✅ | 49 matches for "stream" |
| 7 | `/dependency-edges-list-all` | ✅ | |
| 8 | `/reverse-callers-query-graph` | ✅ | 8 callers for build_cli |
| 9 | `/forward-callees-query-graph` | ✅ | 6 callees for main |
| 10 | `/blast-radius-impact-analysis` | ✅ | |
| 11 | `/circular-dependency-detection-scan` | ✅ | |
| 12 | `/complexity-hotspots-ranking-view` | ✅ | 3 hotspots |
| 13 | `/semantic-cluster-grouping-list` | ✅ | 105 clusters |
| 14 | `/smart-context-token-budget` | ✅ | |

### Multi-Language Support

| Language | Entities | Status |
|----------|----------|--------|
| Java | 22 | ✅ |
| TypeScript | 10 | ✅ |
| PHP | 5 | ✅ |
| Ruby | 4 | ✅ |
| Rust | 3 | ✅ |
| JavaScript | 3 | ✅ |
| C# | 3 | ✅ |
| Python | 2 | ✅ |
| C++ | 1 | ✅ |
| Go | ✅ | (in codebase) |

---

## Files Changed

10 handlers fixed (RwLock pattern):
- `code_entity_detail_view_handler.rs`
- `code_entities_fuzzy_search_handler.rs`
- `reverse_callers_query_graph_handler.rs`
- `forward_callees_query_graph_handler.rs`
- `blast_radius_impact_handler.rs`
- `complexity_hotspots_ranking_handler.rs`
- `circular_dependency_detection_handler.rs`
- `code_entities_list_all_handler.rs`
- `dependency_edges_list_handler.rs`
- `semantic_cluster_grouping_handler.rs`
- `smart_context_token_budget_handler.rs`

---

## Release

- [ ] Update version in Cargo.toml files to 1.5.4
- [ ] Commit with message: `fix: resolve RwLock deadlock in HTTP endpoints (v1.5.4)`
- [ ] Tag: `v1.5.4`
- [ ] Push to origin

---

## What Was Fixed

**Before (deadlock)**:
```rust
let db_guard = state.database_storage_connection_arc.read().await;
if let Some(storage) = db_guard.as_ref() {
    storage.raw_query(...).await;  // Lock held across await = DEADLOCK
}
```

**After (safe)**:
```rust
let storage = {
    let db_guard = state.database_storage_connection_arc.read().await;
    db_guard.as_ref().cloned()?  // Clone Arc, release lock
};
storage.raw_query(...).await;  // No lock held
```
