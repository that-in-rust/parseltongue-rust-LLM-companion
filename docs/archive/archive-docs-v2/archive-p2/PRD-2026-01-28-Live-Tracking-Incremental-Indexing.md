# PRD: Live Tracking / Incremental Indexing for Parseltongue

**Document ID**: PRD-2026-01-28
**Version**: 1.0.0
**Date**: 2026-01-28
**Status**: Approved
**Feature**: Real-time dependency graph updates when files change

---

## Executive Summary

Parseltongue currently requires a full codebase re-scan (`pt01`) to update the dependency graph. This PRD defines the "Live Tracking" feature that automatically updates the graph within 1 second of file saves, enabling real-time LLM context and dashboard updates.

**Critical Finding**: The `file_watcher_service_module/` doesn't exist yet in v1.4.0. The D04 architecture document describes a design target, not existing code. Implementation scope is larger than originally estimated.

**Recommended Approach**: Approach 1 (File Hash + Coarse-Grained Invalidation) - 20-100ms latency, 2-3 days for core implementation.

---

## 1. User Personas

### Persona A: AI-Assisted Developer ("Alex")
- **Job**: Backend developer using Claude Code with Parseltongue context
- **Pain**: Graph is stale after file saves, LLM suggests code based on outdated dependencies
- **Success**: Graph updates within 500ms of save, `/blast-radius-impact-analysis` reflects changes immediately

### Persona B: Engineering Lead with Dashboard ("Brianna")
- **Job**: Manager watching 3D dependency graph on React frontend
- **Pain**: Graph frozen at state from last full index, no live updates
- **Success**: WebSocket pushes entity events as developers save, graph animates transitions

### Persona C: CI/CD Pipeline ("Charlie")
- **Job**: Automated pipeline computing blast radius on PR merge
- **Pain**: Must do full re-ingest on every merge (30+ seconds for large repos)
- **Success**: Incremental updates in <200ms per file after initial ingest

### Persona D: MCP Integration Builder ("Dana")
- **Job**: Building MCP server wrapping Parseltongue for IDE integration
- **Pain**: No mechanism to know when graph has been updated after file save
- **Success**: WebSocket signals "graph updated" with timestamp for context refresh

---

## 2. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| End-to-end latency (save → graph update) | < 1000ms (p99) | Timestamp: file write → completion |
| Reindex latency per file | < 200ms (p99) for files under 500 LOC | Benchmark test |
| Zero data loss | All changed entities reflected | Automated verification test |
| Hash comparison | Skip unchanged files | Hash cache hit rate |

---

## 3. MVP Scope (Approach 1: File Hash + Coarse Invalidation)

### In Scope
1. **File hash cache** - SHA-256 tracking last-known hash per file
2. **Coarse invalidation** - On hash change: DELETE all entities from changed file, re-parse entire file, INSERT new entities
3. **Reuse existing parsers** - Call `Isgl1KeyGeneratorImpl::parse_source()` for tree-sitter parsing
4. **HTTP endpoint** - `/incremental-reindex-file-update?path=/path/to/file.rs`

### Out of Scope (Defer to Future)
- Salsa incremental computation framework (Approach 2)
- Tree-sitter incremental parsing
- ISGL1 v2 birth-timestamp identity
- Content-addressed storage / CRDTs (Approach 3)
- WebSocket streaming (can be added later)

---

## 4. Technical Implementation

### 4.1 New HTTP Endpoint

**Endpoint**: `/incremental-reindex-file-update`
**Method**: POST
**Parameters**:
- `path` (query): Absolute file path to reindex

**Response**:
```json
{
  "success": true,
  "data": {
    "file_path": "/path/to/file.rs",
    "entities_before": 5,
    "entities_after": 6,
    "entities_added": 2,
    "entities_removed": 1,
    "entities_modified": 0,
    "edges_added": 3,
    "edges_removed": 1,
    "hash_changed": true,
    "processing_time_ms": 45
  }
}
```

### 4.2 Core Function Signature

```rust
/// Incrementally reindex a single file
///
/// # 4-Word Name: handle_incremental_reindex_file_request
pub async fn handle_incremental_reindex_file_request(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<IncrementalReindexQueryParams>,
) -> impl IntoResponse
```

### 4.3 Implementation Steps

1. Read file content, compute SHA-256 hash
2. Compare hash against cached value (if exists)
3. If unchanged, return early with `hash_changed: false`
4. Query existing entities WHERE `file_path == requested_path`
5. Delete those entities and their outgoing edges
6. Re-parse file using `Isgl1KeyGeneratorImpl::parse_source()`
7. Insert new entities and edges
8. Update hash cache
9. Return diff statistics

### 4.4 Existing Code to Reuse

| Component | Location | Function |
|-----------|----------|----------|
| File parsing | `pt01/streamer.rs:494-587` | `stream_file()` |
| Entity insertion | `parseltongue-core/storage/cozo_client.rs:774-801` | `insert_entity()` |
| Entity deletion | `parseltongue-core/storage/cozo_client.rs:843-860` | `delete_entity()` |
| Edge batch insert | `parseltongue-core/storage/cozo_client.rs:203-247` | `insert_edges_batch()` |

---

## 5. Simplified Architecture (User Decision)

**User Decision**: Skip workspace management and WebSocket to get incremental indexing working faster.

### Simplified Architecture
- **Single database** - Update the existing analysis.db directly
- **HTTP-only** - No WebSocket streaming; clients poll endpoints for latest data
- **On-demand** - Call endpoint when you want to reindex a file

---

## 6. Edge Cases

| Scenario | Handling |
|----------|----------|
| File doesn't exist | Return error with clear message |
| File outside indexed directory | Return error |
| Syntax errors in file | Tree-sitter produces partial AST, extract what we can |
| File deleted | Delete all entities, return success |
| Permission denied | Return error with details |
| Binary file | Skip, return error |

---

## 7. Performance Contracts

| Operation | Target |
|-----------|--------|
| File hash computation (1MB) | < 10ms |
| Entity query by file_path | < 10ms |
| Batch delete 100 entities | < 50ms |
| Insert 100 entities | < 500ms |
| Insert 100 edges | < 100ms |
| Complete reindex (single file) | < 500ms |
| Overall timeout | 30 seconds |

---

## 8. Files to Modify/Create

### New Files
- `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/incremental_reindex_file_handler.rs`

### Files to Modify
- `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/mod.rs`
- `crates/pt08-http-code-query-server/src/route_definition_builder_module.rs`
- `crates/parseltongue-core/src/storage/cozo_client.rs`
- `crates/pt08-http-code-query-server/Cargo.toml`

### Dependencies to Add
```toml
sha2 = "0.10"       # SHA-256 hashing
```

### 4-Word Naming Convention
- Handler module: `incremental_reindex_file_handler`
- Handler function: `handle_incremental_reindex_file_request`
- Query params: `IncrementalReindexQueryParams`
- Response payload: `IncrementalReindexResponsePayload`
- Endpoint: `/incremental-reindex-file-update`

---

## 9. Database Methods Needed

### New methods for `CozoDbStorage`:

```rust
/// Get all entities from a specific file
/// # 4-Word Name: get_entities_by_file_path
pub async fn get_entities_by_file_path(&self, file_path: &str) -> Result<Vec<CodeEntity>>

/// Delete multiple entities by their ISGL1 keys
/// # 4-Word Name: delete_entities_batch_by_keys
pub async fn delete_entities_batch_by_keys(&self, keys: &[String]) -> Result<usize>

/// Delete edges where from_key is in the provided list
/// # 4-Word Name: delete_edges_by_from_keys
pub async fn delete_edges_by_from_keys(&self, from_keys: &[String]) -> Result<usize>

/// Get or set file hash in cache
/// # 4-Word Name: get_cached_file_hash_value
pub async fn get_cached_file_hash_value(&self, file_path: &str) -> Result<Option<String>>

/// # 4-Word Name: set_cached_file_hash_value
pub async fn set_cached_file_hash_value(&self, file_path: &str, hash: &str) -> Result<()>
```

---

## 10. Estimate

| Phase | Duration |
|-------|----------|
| Database helper methods | 0.5 days |
| Handler implementation | 1 day |
| Wire endpoint to routes | 0.5 days |
| Hash caching | 0.5 days |
| Integration tests | 0.5 days |
| **Total** | **3 days** |

---

*PRD created 2026-01-28 from notes01-agent analysis, local-exec-specs specifications, and Explore agent investigation.*
