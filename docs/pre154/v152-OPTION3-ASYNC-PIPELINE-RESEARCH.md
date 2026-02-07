# V152 Option 3: Async Pipeline Research Document

**Date**: 2026-02-07
**Version**: 1.5.2 (Proposed)
**Research Method**: Parseltongue HTTP API Analysis Only

---

## Executive Summary

This document presents comprehensive research findings for implementing an async pipeline architecture for Parseltongue's ingestion optimization. The research was conducted exclusively using the Parseltongue HTTP API to analyze the existing codebase structure, dependencies, and patterns.

**Overall Feasibility Score: 7/10**

---

## 1. Tokio Usage Analysis

### Current Async Infrastructure

| Tokio Component | Status | Location |
|-----------------|--------|----------|
| `tokio::sync::RwLock` | In use | External dependency |
| `tokio::sync::mpsc` | In use | External dependency |
| `tokio::fs` | Available | External dependency |
| `tokio::net::TcpListener` | In use | External dependency |
| `async_trait` | In use | External dependency |

### Findings

1. **Tokio is already a core dependency** - The HTTP server (pt08) uses Tokio runtime
2. **RwLock from tokio** is used for shared state management
3. **mpsc channels** are available from tokio for message passing
4. **AsyncReadExt** trait is imported, indicating async I/O capabilities exist

### Modules Already Async-Ready

Based on entity analysis:

```
pt08-http-code-query-server/
├── http_server_startup_runner.rs      [ASYNC - uses tokio runtime]
├── file_watcher_integration_service/  [ASYNC - event-driven]
├── incremental_reindex_core_logic.rs  [ASYNC-CAPABLE]
└── http_endpoint_handler_modules/     [ASYNC - axum handlers]
```

### Modules Requiring Async Conversion

```
pt01-folder-to-cozodb-streamer/
├── streamer.rs                        [SYNC - needs conversion]
├── file_watcher.rs                    [SYNC - uses notify]
└── lsp_client.rs                      [SYNC - blocking I/O]

parseltongue-core/
├── query_extractor.rs                 [SYNC - tree-sitter parsing]
└── storage/cozo_client.rs             [SYNC - database operations]
```

---

## 2. Blocking Operation Identification

### Tree-sitter Parsing Operations

**Entities Found**: 97 parse-related entities

Critical blocking operations:

| Function | Location | Block Type |
|----------|----------|------------|
| `parse_ruby_code_extract_all` | unresolved-reference | CPU-bound |
| `parse_javascript_code_extract_edges` | unresolved-reference | CPU-bound |
| `parse_typescript_code_extract_edges` | unresolved-reference | CPU-bound |
| `parse_java_code_extract_edges` | unresolved-reference | CPU-bound |
| `parse_go_code` | unresolved-reference | CPU-bound |
| `parse_cpp_code_extract_edges` | unresolved-reference | CPU-bound |
| `parse_csharp_code_extract_edges` | unresolved-reference | CPU-bound |
| `parse_php_code_extract_edges` | unresolved-reference | CPU-bound |

**Recommendation**: All tree-sitter parsing operations are CPU-bound and must use `tokio::task::spawn_blocking()`.

### File I/O Operations

**Entities Found**: 41 file-related entities

| Operation | Function | Requires spawn_blocking |
|-----------|----------|------------------------|
| File read | `read_file_content` | Yes - blocking I/O |
| File check | `is_file` | No - metadata only |
| Hash computation | `get_cached_file_hash_value` | Yes - file read |
| Directory walk | WalkDir | Yes - blocking iterator |

### Database Operations (CozoDbStorage)

**Methods requiring async consideration** (30+ methods):

```rust
// High-frequency operations needing optimization:
insert_entities_batch     // Batch DB write
insert_edges_batch        // Batch DB write
get_all_entities          // Full table scan
get_all_dependencies      // Full table scan
calculate_blast_radius    // Graph traversal
get_transitive_closure    // Recursive query
```

---

## 3. Channel Feasibility Analysis

### Current Channel Usage

| Channel Type | Usage | Source |
|--------------|-------|--------|
| `ChannelSendError` | Error type exists | pt01 errors |
| `tokio::sync::mpsc` | External dep available | tokio |

### Crossbeam Assessment

**Finding**: No crossbeam entities detected in the codebase.

This is advantageous because:
1. No migration from crossbeam to tokio channels required
2. Can use `tokio::sync::mpsc` directly
3. Unified async runtime without mixed channel types

### Proposed Channel Architecture

```
                    +-----------------+
                    | File Discovery  |
                    | (WalkDir async) |
                    +-----------------+
                            |
                            v
                    +---------------+
                    | mpsc::channel |
                    | (file paths)  |
                    +---------------+
                            |
            +---------------+---------------+
            |               |               |
            v               v               v
    +-------------+ +-------------+ +-------------+
    | Parser      | | Parser      | | Parser      |
    | Worker 1    | | Worker 2    | | Worker N    |
    | (blocking)  | | (blocking)  | | (blocking)  |
    +-------------+ +-------------+ +-------------+
            |               |               |
            +---------------+---------------+
                            |
                            v
                    +---------------+
                    | mpsc::channel |
                    | (entities)    |
                    +---------------+
                            |
                            v
                    +---------------+
                    | Batch Writer  |
                    | (async DB)    |
                    +---------------+
```

---

## 4. CozoDbStorage Async Analysis

### Current Implementation Status

**Struct Location**: `./crates/parseltongue-core/src/storage/cozo_client.rs`

**Current Status**: SYNCHRONOUS

### Methods Inventory (37 methods identified)

| Method Category | Count | Async Conversion Effort |
|-----------------|-------|------------------------|
| Schema creation | 3 | Low - one-time ops |
| Insert operations | 5 | Medium - batch support exists |
| Query operations | 8 | Medium - read-heavy |
| Delete operations | 3 | Low - infrequent |
| Utility methods | 8 | Low - pure functions |
| Complex queries | 10 | High - graph traversals |

### Methods Requiring Async Conversion

```rust
// Priority 1: High-frequency batch operations
pub async fn insert_entities_batch(&self, entities: Vec<CodeEntity>) -> Result<()>
pub async fn insert_edges_batch(&self, edges: Vec<DependencyEdge>) -> Result<()>
pub async fn delete_entities_batch_by_keys(&self, keys: Vec<String>) -> Result<()>

// Priority 2: Query operations
pub async fn get_all_entities(&self, limit: usize) -> Result<Vec<CodeEntity>>
pub async fn get_all_dependencies(&self, limit: usize) -> Result<Vec<DependencyEdge>>
pub async fn query_entities(&self, query: &str) -> Result<Vec<CodeEntity>>

// Priority 3: Complex graph operations
pub async fn calculate_blast_radius(&self, key: &str, hops: usize) -> Result<BlastRadius>
pub async fn get_transitive_closure(&self, key: &str) -> Result<Vec<String>>
```

### CozoDB Backend Consideration

CozoDB uses RocksDB backend which is NOT natively async. Options:

1. **spawn_blocking wrapper** - Wrap sync calls in spawn_blocking
2. **Connection pooling** - Use multiple DB connections
3. **Async-compatible mode** - Check if cozo supports async mode

**Recommendation**: Use `spawn_blocking` wrappers for all CozoDbStorage methods.

---

## 5. Streamer Refactoring Scope

### Blast Radius Analysis

#### stream_directory

**Total Affected Entities**: 7 (3 hops)

| Hop | Count | Entities |
|-----|-------|----------|
| 1 | 4 | execute_initial_codebase_scan, run_folder_to_cozodb_streamer, 2 test functions |
| 2 | 2 | start_http_server_blocking_loop, main |
| 3 | 1 | run_http_code_query_server |

#### stream_file

**Total Affected Entities**: 16 (3 hops)

| Hop | Count | Description |
|-----|-------|-------------|
| 1 | 10 | 8 test functions + stream_directory method |
| 2 | 4 | Same as stream_directory hop 1 |
| 3 | 2 | start_http_server_blocking_loop, main |

### Call Chain Analysis

```
main()
  └── run_folder_to_cozodb_streamer()
        └── create_streamer()
              └── stream_directory()
                    └── stream_file()
                          ├── read_file_content()
                          ├── parse_*_code_extract_edges() [multiple]
                          └── insert_entities_batch()
```

### Complexity Hotspots Relevant to Refactoring

| Rank | Entity | Coupling Score | Impact |
|------|--------|----------------|--------|
| 6 | streamer.rs (file) | 136 outbound | HIGH |
| 12 | incremental_reindex_core_logic.rs | 82 outbound | MEDIUM |
| 13 | cozo_client.rs | 77 outbound | HIGH |
| 17 | query_extractor.rs | 73 outbound | MEDIUM |

---

## 6. Feasibility Assessment

### Scoring Breakdown

| Criteria | Score (1-10) | Rationale |
|----------|--------------|-----------|
| Tokio infrastructure | 9 | Already in place for HTTP server |
| Channel migration | 10 | No crossbeam to migrate |
| CozoDbStorage conversion | 6 | Many methods, but spawn_blocking viable |
| Streamer refactoring | 5 | 136 outbound dependencies |
| Tree-sitter compatibility | 7 | spawn_blocking works for CPU-bound |
| Test coverage risk | 6 | 10+ test functions affected |
| Breaking change risk | 5 | Many callers need async signatures |

**Overall Feasibility: 7/10**

---

## 7. Risks Identified

### High Priority Risks

1. **Signature Propagation**
   - Converting stream_file to async requires 16 dependent entities to change
   - All callers must become async or use block_on

2. **Tree-sitter Thread Safety**
   - Tree-sitter Parser is not Send+Sync
   - Must create parser per spawn_blocking task

3. **CozoDB Thread Model**
   - RocksDB is single-threaded writer
   - Concurrent writes may cause contention

### Medium Priority Risks

4. **Test Migration**
   - 8+ integration tests depend on stream_file
   - Tests need #[tokio::test] attribute

5. **Error Handling**
   - spawn_blocking panics need to be caught
   - JoinError needs conversion to application error

### Low Priority Risks

6. **Performance Regression**
   - spawn_blocking has overhead
   - Channel communication adds latency

---

## 8. Recommended Async Architecture

### Phase 1: Foundation (v1.5.2)

```rust
// New trait for async storage
#[async_trait]
pub trait AsyncCodeStorage {
    async fn insert_entities_batch(&self, entities: Vec<CodeEntity>) -> Result<()>;
    async fn insert_edges_batch(&self, edges: Vec<DependencyEdge>) -> Result<()>;
    async fn get_entities_by_file(&self, path: &str) -> Result<Vec<CodeEntity>>;
}

// Wrapper implementation
impl AsyncCodeStorage for CozoDbStorage {
    async fn insert_entities_batch(&self, entities: Vec<CodeEntity>) -> Result<()> {
        let db = self.clone();
        tokio::task::spawn_blocking(move || {
            db.insert_entities_batch_sync(entities)
        }).await?
    }
}
```

### Phase 2: Parallel Pipeline (v1.5.3)

```rust
pub struct AsyncFileStreamer {
    file_tx: mpsc::Sender<PathBuf>,
    entity_rx: mpsc::Receiver<Vec<CodeEntity>>,
    worker_handles: Vec<JoinHandle<()>>,
}

impl AsyncFileStreamer {
    pub async fn stream_directory(&self, root: PathBuf) -> Result<StreamStats> {
        // Producer: Walk directory and send file paths
        let file_tx = self.file_tx.clone();
        tokio::spawn(async move {
            for entry in WalkDir::new(&root) {
                file_tx.send(entry.path().to_path_buf()).await?;
            }
        });

        // Consumer: Collect entities and batch insert
        while let Some(entities) = self.entity_rx.recv().await {
            storage.insert_entities_batch(entities).await?;
        }
    }
}
```

### Phase 3: Full Async (v1.5.4)

```rust
pub async fn stream_file_async(
    path: PathBuf,
    storage: Arc<dyn AsyncCodeStorage>,
) -> Result<FileStreamResult> {
    // Read file content (blocking I/O)
    let content = tokio::task::spawn_blocking(move || {
        std::fs::read_to_string(&path)
    }).await??;

    // Parse with tree-sitter (CPU-bound)
    let entities = tokio::task::spawn_blocking(move || {
        parse_code_extract_entities(&content, &language)
    }).await??;

    // Insert to database (async)
    storage.insert_entities_batch(entities).await?;

    Ok(FileStreamResult { ... })
}
```

---

## 9. Migration Path

### Step 1: Prepare Infrastructure (v1.5.2-alpha)

- [ ] Add `async_trait` to Cargo.toml (already present)
- [ ] Create `AsyncCodeStorage` trait
- [ ] Implement spawn_blocking wrappers for CozoDbStorage
- [ ] Add tokio mpsc channels to streamer crate

### Step 2: Convert Core Components (v1.5.2-beta)

- [ ] Convert `stream_file` to async with spawn_blocking for parsing
- [ ] Convert `stream_directory` to async
- [ ] Update FileStreamer struct to use channels
- [ ] Convert 10 test functions to async

### Step 3: Parallel Workers (v1.5.3)

- [ ] Implement worker pool for file parsing
- [ ] Add configurable parallelism (default: num_cpus)
- [ ] Implement backpressure via bounded channels
- [ ] Add metrics for pipeline stages

### Step 4: Optimize (v1.5.4)

- [ ] Profile and tune channel buffer sizes
- [ ] Add batch size optimization
- [ ] Implement priority queue for changed files
- [ ] Add cancellation support

---

## 10. Codebase Statistics Summary

| Metric | Value |
|--------|-------|
| Total Code Entities | 1,070 |
| Total Dependency Edges | 7,068 |
| Languages Detected | 10 |
| Circular Dependencies | 0 |
| Streamer Module Coupling | 136 |
| CozoDbStorage Methods | 37 |
| Parse Functions | 12 (per language) |

---

## 11. Appendix: Key Entity References

### Critical Path Entities

```
rust:fn:stream_directory:unresolved-reference:0-0
rust:fn:stream_file:unresolved-reference:0-0
rust:method:stream_directory:____crates_pt01_folder_to_cozodb_streamer_src_streamer:T1779334742
rust:impl:CozoDbStorage:____crates_parseltongue_core_src_storage_cozo_client:T1778853907
rust:fn:start_http_server_blocking_loop:____crates_pt08_http_code_query_server_src_http_server_startup_runner:T1686216071
```

### External Dependencies for Async

```
rust:module:mpsc:external-dependency-tokio:0-0
rust:module:RwLock:external-dependency-tokio:0-0
rust:module:AsyncReadExt:external-dependency-tokio:0-0
rust:module:async_trait:external-dependency-async_trait:0-0
rust:module:WalkDir:external-dependency-walkdir:0-0
rust:module:DebouncedEvent:external-dependency-notify_debouncer_full:0-0
```

---

## 12. Conclusion

The async pipeline optimization is **feasible with moderate effort**. The main advantages are:

1. Tokio infrastructure already exists in pt08
2. No crossbeam dependencies to migrate
3. Clear separation between blocking (tree-sitter) and async (I/O) operations

The main challenges are:

1. High coupling in streamer.rs (136 dependencies)
2. Need to propagate async signatures through call chain
3. Tree-sitter parser not being Send+Sync

**Recommended Approach**: Incremental migration starting with spawn_blocking wrappers, then adding parallelism through worker pools and channels.

**Estimated Timeline**: 3-4 versions (v1.5.2 through v1.5.5)

---

*Document generated using Parseltongue HTTP API analysis on 2026-02-07*
