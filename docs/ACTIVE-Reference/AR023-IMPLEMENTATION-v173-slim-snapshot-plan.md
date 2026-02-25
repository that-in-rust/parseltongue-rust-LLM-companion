# IMPLEMENTATION: v1.7.3 Slim Snapshot Plan

**Date**: 2026-02-13
**Context**: Implementation plan for pt02 (MessagePack) and pt03 (JSON) slim graph snapshot exporters, plus pt08 snapshot loading. Based on codebase exploration and architectural research documented in:
- `docs/RESEARCH-v173-serialization-formats.md`
- `docs/DECISION-v173-pt02-pt03-endpoint-selection.md`
- `docs/THESIS-v173-slim-graph-address-model.md`

---

## Problem

CozoDB RocksDB fails on Windows (6 attempts v1.6.7-v1.7.2). Windows Defender scans/locks SST files during write-heavy ingestion. Every persistent backend (RocksDB, SQLite, Sled) breaks.

## Solution

pt02/pt03 serialize a **slim dependency graph** directly to file. pt08 loads the file into CozoDB `mem` (pure RAM, zero disk writes). 21 of 24 endpoints work identically. 3 dropped.

---

## Codebase Exploration Findings

### 1. pt01 Parsing Pipeline (streamer.rs)

The parallel Rayon pipeline has a clean 4-step architecture:

| Step | Lines | What Happens |
|------|-------|-------------|
| File walk | 682-720 | WalkDir collects all file paths, filters by language support |
| Parallel parse | 725-735 | Rayon `par_iter()` calls `process_file_sync_for_parallel()` per file |
| Aggregation | 739-765 | Collects into `Vec<CodeEntity>` + `Vec<DependencyEdge>` + 3 diagnostic Vecs |
| DB insertion | 770-823 | `tokio::join!` of 5 batch insert calls into CozoDB |

**Interception point**: Between line 765 (aggregation done) and line 770 (DB insertion starts). At this point, all data is in Rust structs, not yet in CozoDB.

**Key types at interception**:
```
all_entities:       Vec<CodeEntity>                              (line 743)
all_dependencies:   Vec<parseltongue_core::entities::DependencyEdge>  (line 744)
all_excluded_tests: Vec<ExcludedTestEntity>                      (line 745)
all_word_coverages: Vec<FileWordCoverageRow>                     (line 746)
```

### 2. How pt02 Gets Access to CozoDB After Ingestion

`ToolFactory::create_streamer(config)` returns `Arc<FileStreamerImpl>` (concrete type, NOT trait object). The `db` field on `FileStreamerImpl` is private (`db: Arc<CozoDbStorage>`).

**Solution**: Add a 3-line public getter:
```rust
pub fn get_database_storage_reference(&self) -> Arc<CozoDbStorage> {
    self.db.clone()
}
```

pt02 flow:
1. `ToolFactory::create_streamer(config_with_mem)` -- creates streamer with CozoDB mem
2. `streamer.stream_directory_with_parallel_rayon()` -- full pipeline, data lands in CozoDB mem
3. `streamer.get_database_storage_reference()` -- get CozoDB handle
4. Run export queries on the CozoDB handle
5. Convert to slim types, serialize, write file

### 3. pt08 Database Loading (http_server_startup_runner.rs)

**Hook point**: Lines 347-363. Currently branches on `db_path`:
```rust
if !db_path.is_empty() && db_path != "mem" {
    CozoDbStorage::new(db_path)  // rocksdb
} else {
    create_new_application_state()  // empty
}
```

Add new branches for `ptgraph:` and `json:` prefixes.

**State propagation**: `SharedApplicationStateContainer` holds `Arc<RwLock<Option<Arc<CozoDbStorage>>>>`. All 24 handlers use the same pattern: acquire read lock, clone Arc, release lock, query. Once a CozoDB instance is in the state, ALL handlers work regardless of how the data got there.

**What to skip for snapshots**: Initial filesystem scan (lines 380-397) and file watcher setup (lines 399-473) both check `db_path != "mem"`. For snapshots, these should be skipped.

### 4. CozoDB Schema (cozo_client.rs)

**CodeGraph** (19 columns):

| Column | Type | Nullable | Slim Import Value |
|--------|------|:--------:|-------------------|
| `ISGL1_key` | String | No (PK) | from slim |
| `Current_Code` | String? | Yes | `""` (empty, NOT null) |
| `Future_Code` | String? | Yes | null |
| `interface_signature` | String | No | `"{}"` |
| `TDD_Classification` | String | No | `"not_a_test"` |
| `lsp_meta_data` | String? | Yes | null |
| `current_ind` | Bool | No | true |
| `future_ind` | Bool | No | false |
| `Future_Action` | String? | Yes | null |
| `file_path` | String | No | from slim |
| `language` | String | No | from slim |
| `last_modified` | String | No | `"ptgraph_import"` |
| `entity_type` | String | No | from slim |
| `entity_class` | String | No | from slim |
| `birth_timestamp` | Int? | Yes | null |
| `content_hash` | String? | Yes | null |
| `semantic_path` | String? | Yes | null |
| `root_subfolder_L1` | String | No | from slim |
| `root_subfolder_L2` | String | No | from slim |

**DependencyEdges** (4 columns):

| Column | Type | Nullable | Slim Import Value |
|--------|------|:--------:|-------------------|
| `from_key` | String | No (CK) | from slim |
| `to_key` | String | No (CK) | from slim |
| `edge_type` | String | No (CK) | from slim |
| `source_location` | String? | Yes | null |

**Critical finding**: `Current_Code` must be `""` (empty string), NOT null. The detail view handler (line 192 of `code_entity_detail_view_handler.rs`) uses `extract_string_value(&row[4])?` which returns `None` on null, propagating through `?` to return 404. Empty string passes through correctly.

**Critical finding**: `interface_signature` is NOT read by ANY handler in `http_endpoint_handler_modules/`. Only used by `incremental_reindex_core_logic.rs` (file watcher reindex, which is skipped for snapshot mode). Setting to `"{}"` satisfies the non-nullable constraint.

### 5. Handler Column Access Patterns

| Handler | Columns Queried | Slim Impact |
|---------|----------------|-------------|
| `code_entity_detail_view` | ISGL1_key, file_path, entity_type, entity_class, language, **Current_Code** | Returns `""` for code |
| `code_entities_list_all` | ISGL1_key, file_path, entity_type, language, L1, L2 | Works unchanged |
| `code_entities_search_fuzzy` | ISGL1_key (contains/starts_with) | Works unchanged |
| All edge/graph handlers | from_key, to_key, edge_type only | Works unchanged |
| All 7 graph algorithm handlers | DependencyEdges only | Works unchanged |
| `codebase_statistics_overview` | COUNT aggregates | Works unchanged |
| `folder_structure_discovery_tree` | root_subfolder_L1, L2 | Works unchanged |

**Result: 21 of 24 handlers need ZERO modification.**

---

## Implementation Tasks

### Task 1: Slim Types (entities.rs) ~50 lines

**File**: `crates/parseltongue-core/src/entities.rs`

Add after `IgnoredFileRow` (line 1796):
- `SlimEntityGraphSnapshot` -- 9 fields (isgl1_key, file_path, line_start, line_end, entity_type, entity_class, language, root_subfolder_l1, root_subfolder_l2)
- `SlimEdgeGraphSnapshot` -- 3 fields (from_key, to_key, edge_type)
- `PtGraphSnapshotContainer` -- 7 fields (version, generated_at, source_directory, entity_count, edge_count, entities, edges)

Already re-exported via `pub use entities::*` in lib.rs.

### Task 2: DB Getter (streamer.rs) ~5 lines

**File**: `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs`

Add to `FileStreamerImpl` after line 116:
```rust
/// # 4-Word Name: get_database_storage_reference
pub fn get_database_storage_reference(&self) -> Arc<CozoDbStorage> {
    self.db.clone()
}
```

### Task 3: Export Methods (cozo_client.rs) ~70 lines

**File**: `crates/parseltongue-core/src/storage/cozo_client.rs`

Add two methods to `CozoDbStorage`:
- `export_slim_entities_from_database()` -- CozoScript query on CodeGraph, parse interface_signature JSON for line numbers, return `Vec<SlimEntityGraphSnapshot>`
- `export_slim_edges_from_database()` -- CozoScript query on DependencyEdges, return `Vec<SlimEdgeGraphSnapshot>`

### Task 4: Import Methods (cozo_client.rs) ~80 lines

**File**: `crates/parseltongue-core/src/storage/cozo_client.rs`

Add two methods to `CozoDbStorage`:
- `insert_slim_entities_batch_directly(&[SlimEntityGraphSnapshot])` -- builds CozoScript `:put CodeGraph` with NULLs/defaults for omitted fields
- `insert_slim_edges_batch_directly(&[SlimEdgeGraphSnapshot])` -- builds CozoScript `:put DependencyEdges` with null source_location

### Task 5: pt02 Crate ~130 lines

**Create**: `crates/pt02-folder-to-ram-snapshot/`

- `Cargo.toml`: deps on pt01, parseltongue-core, rmp-serde 1.3, anyhow, serde, tokio, chrono, console
- `src/lib.rs`: `generate_ptgraph_snapshot_file()` function
  1. StreamerConfig with db_path "mem"
  2. ToolFactory::create_streamer()
  3. stream_directory_with_parallel_rayon()
  4. get_database_storage_reference()
  5. export_slim_entities + export_slim_edges
  6. Build PtGraphSnapshotContainer
  7. rmp_serde::to_vec() -> write .ptgraph file

### Task 6: pt03 Crate ~100 lines

**Create**: `crates/pt03-folder-to-json-exporter/`

Same as pt02 but uses `serde_json::to_writer_pretty()` for .json output.

### Task 7: pt08 Snapshot Loader ~110 lines

**Create**: `crates/pt08-http-code-query-server/src/snapshot_loader_module.rs`

Two functions:
- `load_ptgraph_snapshot_database(path) -> CozoDbStorage` -- read file, rmp_serde deserialize, create CozoDB mem, create schemas, bulk insert slim data
- `load_json_snapshot_database(path) -> CozoDbStorage` -- same with serde_json

Register in `crates/pt08-http-code-query-server/src/lib.rs`.

### Task 8: pt08 Startup Integration ~35 lines

**File**: `crates/pt08-http-code-query-server/src/http_server_startup_runner.rs`

- Add `loaded_from_snapshot_flag: bool` to `SharedApplicationStateContainer`
- Update both constructors (`create_new_application_state`, `create_with_database_storage`)
- Add `ptgraph:` and `json:` branches at lines 347-363
- Skip initial scan + file watcher when `is_snapshot == true`

### Task 9: Endpoint Guards ~15 lines

**Files** (3 handlers):
- `smart_context_token_budget_handler.rs`
- `ingestion_diagnostics_coverage_handler.rs`
- `ingestion_coverage_folder_handler.rs`

Add early-return when `state.loaded_from_snapshot_flag` is true. Return 501 with explanation.

### Task 10: CLI Subcommands ~80 lines

**File**: `crates/parseltongue/src/main.rs`

- Add `pt02-folder-to-ram-snapshot` and `pt03-folder-to-json-exporter` subcommands to `build_cli()`
- Add match arms in main()
- Add handler functions `run_folder_to_ram_snapshot()` and `run_folder_to_json_exporter()`

**File**: `crates/parseltongue/Cargo.toml` -- add pt02, pt03 deps

### Task 11: Workspace Config ~3 lines

**File**: `Cargo.toml` (workspace root)
- Add `rmp-serde = "1.3"` to `[workspace.dependencies]`
- Bump `version = "1.7.3"`

---

## Files Summary

| Action | File | Est. Lines |
|--------|------|:----------:|
| Modify | `crates/parseltongue-core/src/entities.rs` | +50 |
| Modify | `crates/parseltongue-core/src/storage/cozo_client.rs` | +150 |
| Modify | `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs` | +5 |
| Create | `crates/pt02-folder-to-ram-snapshot/Cargo.toml` | +15 |
| Create | `crates/pt02-folder-to-ram-snapshot/src/lib.rs` | +130 |
| Create | `crates/pt03-folder-to-json-exporter/Cargo.toml` | +15 |
| Create | `crates/pt03-folder-to-json-exporter/src/lib.rs` | +100 |
| Create | `crates/pt08-http-code-query-server/src/snapshot_loader_module.rs` | +110 |
| Modify | `crates/pt08-http-code-query-server/src/http_server_startup_runner.rs` | +35 |
| Modify | `crates/pt08-http-code-query-server/src/lib.rs` | +1 |
| Modify | 3 handler files (guards) | +15 |
| Modify | `crates/parseltongue/src/main.rs` | +80 |
| Modify | `crates/parseltongue/Cargo.toml` | +3 |
| Modify | `Cargo.toml` (workspace) | +3 |
| **Total** | | **~712** |

---

## Verification Plan

1. `cargo build --release` -- clean compile
2. `cargo test --all` -- all existing tests pass
3. `parseltongue pt02-folder-to-ram-snapshot .` -- creates .ptgraph
4. `parseltongue pt03-folder-to-json-exporter .` -- creates .json
5. `parseltongue pt08-http-code-query-server --db "ptgraph:path/analysis.ptgraph"` -- server starts
6. `curl localhost:7777/server-health-check-status` -- 200 OK
7. `curl localhost:7777/codebase-statistics-overview-summary` -- entity count matches
8. `curl localhost:7777/strongly-connected-components-analysis` -- graph algo works
9. `curl localhost:7777/smart-context-token-budget` -- 501 "not available"
10. Verify: .ptgraph < .json < RocksDB folder size

---

## Risk Register

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Peak RAM during pt02 ingestion (~3.2GB at 400K entities) | Can't run pt02 on 8GB machine for Linux kernel | pt02 is one-time CLI. Run on beefy machine, copy .ptgraph. |
| interface_signature JSON parsing for line_start/line_end | Export fails if JSON is malformed | Default to line_start=0, line_end=0 on parse failure |
| CozoDB batch insert size limits | Large codebases may need chunking | Chunk into 5000-entity batches (same pattern as existing insert_entities_batch) |
| Current_Code="" vs NULL semantics | Detail view handler returns empty code string | Acceptable -- LLM gets file_path to read source directly |
