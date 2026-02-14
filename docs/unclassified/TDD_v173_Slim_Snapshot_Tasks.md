# TDD Task Tracking: v1.7.3 Slim Graph Snapshots

**Version**: 1.7.3
**Created**: 2026-02-13
**Status**: All tasks pending implementation

---

## Overview

This document tracks the 9-phase implementation plan for Parseltongue v1.7.3, which introduces `.ptgraph` snapshot files to enable Windows support by bypassing RocksDB filesystem locking issues.

**Target**: 21 of 24 endpoints working on Windows via pure-memory CozoDB loaded from MessagePack snapshots.

---

## Phase 1: Slim Types (entities.rs)

**Status**: PENDING

**File**: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/parseltongue-core/src/entities.rs`

**Estimated Lines**: +50

### Acceptance Criteria
- [ ] `SlimEntityGraphSnapshot` struct added with 9 fields:
  - `isgl1_key: String`
  - `file_path: String`
  - `line_start: usize`
  - `line_end: usize`
  - `entity_type: String`
  - `entity_class: String`
  - `language: String`
  - `root_subfolder_l1: String`
  - `root_subfolder_l2: String`
- [ ] `SlimEdgeGraphSnapshot` struct added with 3 fields:
  - `from_key: String`
  - `to_key: String`
  - `edge_type: String`
- [ ] `PtGraphSnapshotContainer` struct added with 7 fields:
  - `version: String` (always "1.7.3")
  - `generated_at: String` (ISO 8601 timestamp)
  - `source_directory: String`
  - `entity_count: usize`
  - `edge_count: usize`
  - `entities: Vec<SlimEntityGraphSnapshot>`
  - `edges: Vec<SlimEdgeGraphSnapshot>`
- [ ] All three structs derive `Serialize, Deserialize, Debug, Clone`
- [ ] Structs placed after `IgnoredFileRow` (around line 1796)
- [ ] `cargo build --release` compiles without errors
- [ ] No warnings in entities.rs

### Implementation Notes
- These are the core data transfer objects for snapshot serialization
- Must be public and re-exported via `pub use entities::*` in lib.rs (already configured)
- Field order matches export query column order for clarity

---

## Phase 2: Database Getter (streamer.rs)

**Status**: PENDING

**File**: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/pt01-folder-to-cozodb-streamer/src/streamer.rs`

**Estimated Lines**: +5

### Acceptance Criteria
- [ ] `get_database_storage_reference()` method added to `FileStreamerImpl` impl block
- [ ] Method signature: `pub fn get_database_storage_reference(&self) -> Arc<CozoDbStorage>`
- [ ] Method body: `self.db.clone()`
- [ ] Method placed after line 116 (after existing impl block methods)
- [ ] 4-word doc comment: `/// # 4-Word Name: get_database_storage_reference`
- [ ] `cargo build -p pt01-folder-to-cozodb-streamer` compiles without errors
- [ ] No warnings in streamer.rs

### Implementation Notes
- This exposes the private `db` field to pt02 after ingestion completes
- Required because `ToolFactory::create_streamer()` returns concrete `Arc<FileStreamerImpl>`, not a trait object
- The Arc clone is cheap (reference counting, not deep copy)

---

## Phase 3: Export + Import (cozo_client.rs)

**Status**: PENDING

**File**: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/parseltongue-core/src/storage/cozo_client.rs`

**Estimated Lines**: +150

### Acceptance Criteria

#### Export Methods
- [ ] `export_slim_entities_from_database()` added to `CozoDbStorage` impl
  - [ ] Signature: `pub async fn export_slim_entities_from_database(&self) -> anyhow::Result<Vec<SlimEntityGraphSnapshot>>`
  - [ ] CozoScript query: `?[ISGL1_key, file_path, entity_type, entity_class, language, root_subfolder_L1, root_subfolder_L2, interface_signature] := *CodeGraph{...}`
  - [ ] Parse `interface_signature` JSON to extract `line_start` and `line_end` (default to 0 on parse failure)
  - [ ] Return Vec of SlimEntityGraphSnapshot
  - [ ] 4-word name: export_slim_entities_from_database
- [ ] `export_slim_edges_from_database()` added to `CozoDbStorage` impl
  - [ ] Signature: `pub async fn export_slim_edges_from_database(&self) -> anyhow::Result<Vec<SlimEdgeGraphSnapshot>>`
  - [ ] CozoScript query: `?[from_key, to_key, edge_type] := *DependencyEdges{from_key, to_key, edge_type}`
  - [ ] Return Vec of SlimEdgeGraphSnapshot
  - [ ] 4-word name: export_slim_edges_from_database

#### Import Methods
- [ ] `insert_slim_entities_batch_directly()` added to `CozoDbStorage` impl
  - [ ] Signature: `pub async fn insert_slim_entities_batch_directly(&self, entities: &[SlimEntityGraphSnapshot]) -> anyhow::Result<()>`
  - [ ] Build CozoScript `:put CodeGraph {...}` with all 19 columns
  - [ ] Slim fields: populated from SlimEntityGraphSnapshot
  - [ ] Omitted fields: `Current_Code=""`, `Future_Code=NULL`, `interface_signature="{}"`, `TDD_Classification="not_a_test"`, etc.
  - [ ] Chunk into batches of 5000 entities if len > 5000
  - [ ] 4-word name: insert_slim_entities_batch_directly
- [ ] `insert_slim_edges_batch_directly()` added to `CozoDbStorage` impl
  - [ ] Signature: `pub async fn insert_slim_edges_batch_directly(&self, edges: &[SlimEdgeGraphSnapshot]) -> anyhow::Result<()>`
  - [ ] Build CozoScript `:put DependencyEdges {...}` with all 4 columns
  - [ ] Set `source_location=NULL` for all edges
  - [ ] Chunk into batches of 5000 edges if len > 5000
  - [ ] 4-word name: insert_slim_edges_batch_directly

#### General
- [ ] All methods are `async` and use `tokio` runtime
- [ ] All methods use `anyhow::Result` for error handling
- [ ] All methods have 4-word function names
- [ ] `cargo build -p parseltongue-core` compiles without errors
- [ ] No warnings in cozo_client.rs

### Implementation Notes
- Export queries run against the fully-populated CozoDB mem database created by pt01
- Import queries write to a fresh CozoDB mem database created by pt08 snapshot loader
- Critical: `Current_Code` must be `""` (empty string), NOT null, or detail view handler fails
- Critical: `interface_signature` must be `"{}"` (JSON object string), NOT null, to satisfy non-nullable constraint
- Batching prevents CozoDB query size limits on large codebases

---

## Phase 4: pt02 Crate (new crate)

**Status**: PENDING

**Files**:
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/pt02-folder-to-ram-snapshot/Cargo.toml`
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/pt02-folder-to-ram-snapshot/src/lib.rs`

**Estimated Lines**: +145 total (15 Cargo.toml + 130 lib.rs)

### Acceptance Criteria

#### Cargo.toml
- [ ] Package name: `pt02-folder-to-ram-snapshot`
- [ ] Version: `1.7.3`
- [ ] Dependencies:
  - [ ] `parseltongue-core = { workspace = true }`
  - [ ] `pt01-folder-to-cozodb-streamer = { workspace = true }`
  - [ ] `rmp-serde = { workspace = true }`
  - [ ] `serde = { workspace = true, features = ["derive"] }`
  - [ ] `tokio = { workspace = true, features = ["full"] }`
  - [ ] `anyhow = { workspace = true }`
  - [ ] `chrono = { workspace = true }`
  - [ ] `console = { workspace = true }`

#### src/lib.rs
- [ ] `generate_ptgraph_snapshot_file()` function signature:
  - [ ] `pub async fn generate_ptgraph_snapshot_file(source_dir: &str, workspace_path: &str) -> anyhow::Result<String>`
  - [ ] Returns path to generated .ptgraph file
- [ ] Implementation steps:
  1. [ ] Create `StreamerConfig` with `db_path = "mem"`
  2. [ ] Call `ToolFactory::create_streamer(config).await?`
  3. [ ] Call `streamer.stream_directory_with_parallel_rayon().await?`
  4. [ ] Call `streamer.get_database_storage_reference()`
  5. [ ] Call `db.export_slim_entities_from_database().await?`
  6. [ ] Call `db.export_slim_edges_from_database().await?`
  7. [ ] Build `PtGraphSnapshotContainer` with:
     - `version = "1.7.3"`
     - `generated_at = chrono::Utc::now().to_rfc3339()`
     - `source_directory = source_dir.to_string()`
     - `entity_count = entities.len()`
     - `edge_count = edges.len()`
     - `entities = entities`
     - `edges = edges`
  8. [ ] Serialize with `rmp_serde::to_vec(&container)?`
  9. [ ] Write to `{workspace_path}/analysis.ptgraph`
  10. [ ] Console output: "Snapshot written: {path} ({size} MB)"
  11. [ ] Return path
- [ ] 4-word function name: generate_ptgraph_snapshot_file
- [ ] Error handling: all steps wrapped in `anyhow::Context` for clear error messages
- [ ] `cargo build -p pt02-folder-to-ram-snapshot` compiles without errors
- [ ] No warnings

### Implementation Notes
- MessagePack chosen over JSON for 94% size reduction (research doc: RESEARCH-v173-serialization-formats.md)
- This crate uses pt01 internally, so full parsing pipeline is reused
- Peak RAM: ~3.2GB for 400K entities (one-time export, acceptable)
- Expected output size: ~2-8MB for typical codebases

---

## Phase 5: pt08 Snapshot Loader (new module + startup integration)

**Status**: PENDING

**Files**:
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/snapshot_loader_module.rs` (new)
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/lib.rs` (modify)
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/http_server_startup_runner.rs` (modify)

**Estimated Lines**: +146 total (110 snapshot_loader_module.rs + 1 lib.rs + 35 http_server_startup_runner.rs)

### Acceptance Criteria

#### snapshot_loader_module.rs (new file)
- [ ] `load_ptgraph_snapshot_database()` function signature:
  - [ ] `pub async fn load_ptgraph_snapshot_database(ptgraph_path: &str) -> anyhow::Result<CozoDbStorage>`
- [ ] Implementation steps:
  1. [ ] Read file: `std::fs::read(ptgraph_path)?`
  2. [ ] Deserialize: `rmp_serde::from_slice::<PtGraphSnapshotContainer>(&bytes)?`
  3. [ ] Create CozoDB mem: `CozoDbStorage::new("mem").await?`
  4. [ ] Create schemas: `db.create_schema().await?`
  5. [ ] Bulk insert entities: `db.insert_slim_entities_batch_directly(&container.entities).await?`
  6. [ ] Bulk insert edges: `db.insert_slim_edges_batch_directly(&container.edges).await?`
  7. [ ] Console output: "Loaded snapshot: {entity_count} entities, {edge_count} edges from {source_directory}"
  8. [ ] Return db
- [ ] 4-word function name: load_ptgraph_snapshot_database
- [ ] Error handling: all steps wrapped in `anyhow::Context`
- [ ] File uses `use parseltongue_core::{CozoDbStorage, PtGraphSnapshotContainer};`

#### lib.rs modifications
- [ ] Add `pub mod snapshot_loader_module;` to module declarations

#### http_server_startup_runner.rs modifications
- [ ] Add `loaded_from_snapshot_flag: bool` field to `SharedApplicationStateContainer` struct
- [ ] Update `create_new_application_state()` constructor:
  - [ ] Set `loaded_from_snapshot_flag: false`
- [ ] Update `create_with_database_storage()` constructor:
  - [ ] Add `is_snapshot: bool` parameter
  - [ ] Set `loaded_from_snapshot_flag: is_snapshot`
- [ ] Modify `run_server_with_startup_initialization()` at lines 347-363:
  - [ ] Add branch for `db_path.starts_with("ptgraph:")`:
    - [ ] Extract path: `let snapshot_path = &db_path[8..];`
    - [ ] Call `snapshot_loader_module::load_ptgraph_snapshot_database(snapshot_path).await?`
    - [ ] Call `create_with_database_storage(db, true)`
  - [ ] Existing `!db_path.is_empty() && db_path != "mem"` branch:
    - [ ] Call `create_with_database_storage(db, false)` (not a snapshot)
- [ ] Skip initial filesystem scan (lines 380-397):
  - [ ] Wrap in `if !state.loaded_from_snapshot_flag { ... }`
- [ ] Skip file watcher setup (lines 399-473):
  - [ ] Wrap in `if !state.loaded_from_snapshot_flag { ... }`
- [ ] `cargo build -p pt08-http-code-query-server` compiles without errors
- [ ] No warnings

### Implementation Notes
- CozoDB mem database is created fresh and populated from snapshot data
- File watcher is incompatible with snapshot mode (no live reindexing)
- Initial scan is unnecessary for snapshot mode (data is already complete)
- SharedApplicationStateContainer propagates snapshot flag to all handlers

---

## Phase 6: Endpoint Guards (3 handler files)

**Status**: PENDING

**Files**:
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/smart_context_token_budget_handler.rs`
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_diagnostics_coverage_handler.rs`
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_coverage_folder_handler.rs`

**Estimated Lines**: +15 total (~5 per file)

### Acceptance Criteria

#### All Three Handlers
- [ ] Add early-return check at top of handler function (before database query):
  ```rust
  if state.loaded_from_snapshot_flag {
      return Ok((
          StatusCode::NOT_IMPLEMENTED,
          Json(serde_json::json!({
              "error": "Endpoint not available in snapshot mode",
              "reason": "This endpoint requires Current_Code field data which is omitted in slim snapshots",
              "suggestion": "Use pt01 RocksDB mode for full feature set, or read source files directly via file_path"
          }))
      ));
  }
  ```
- [ ] Check placed immediately after `state.lock()` acquisition
- [ ] Return type matches existing handler signature
- [ ] `cargo build -p pt08-http-code-query-server` compiles without errors
- [ ] No warnings

### Implementation Notes
- These 3 endpoints require `Current_Code` field which is empty in slim snapshots
- 501 Not Implemented is the correct HTTP status (endpoint exists but unsupported in current mode)
- Graceful degradation: 21 of 24 endpoints work, 3 return clear error messages
- LLMs can read source files directly via `file_path` + `line_start`/`line_end` from entity detail

---

## Phase 7: CLI + Workspace (main.rs + Cargo.toml)

**Status**: PENDING

**Files**:
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/parseltongue/src/main.rs`
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/parseltongue/Cargo.toml`
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/Cargo.toml` (workspace root)

**Estimated Lines**: +86 total (80 main.rs + 3 parseltongue/Cargo.toml + 3 workspace Cargo.toml)

### Acceptance Criteria

#### main.rs modifications
- [ ] Import pt02 crate: `use pt02_folder_to_ram_snapshot::generate_ptgraph_snapshot_file;`
- [ ] Add subcommand to `build_cli()`:
  ```rust
  .subcommand(
      Command::new("pt02-folder-to-ram-snapshot")
          .about("Generate MessagePack snapshot file from codebase (Windows-compatible)")
          .arg(
              Arg::new("source")
                  .help("Source directory to analyze")
                  .required(true)
                  .index(1),
          )
  )
  ```
- [ ] Add match arm in `main()`:
  ```rust
  Some(("pt02-folder-to-ram-snapshot", sub_matches)) => {
      run_folder_to_ram_snapshot(sub_matches).await?;
  }
  ```
- [ ] Add handler function:
  ```rust
  async fn run_folder_to_ram_snapshot(matches: &ArgMatches) -> anyhow::Result<()> {
      let source = matches.get_one::<String>("source").unwrap();
      let workspace_path = create_timestamped_workspace_folder(source)?;
      let ptgraph_path = generate_ptgraph_snapshot_file(source, &workspace_path).await?;
      println!("Snapshot file: {}", ptgraph_path);
      println!("To serve: parseltongue pt08-http-code-query-server --db \"ptgraph:{}\"", ptgraph_path);
      Ok(())
  }
  ```
- [ ] 4-word function name: run_folder_to_ram_snapshot
- [ ] `cargo build -p parseltongue` compiles without errors
- [ ] No warnings in main.rs

#### crates/parseltongue/Cargo.toml modifications
- [ ] Add dependency: `pt02-folder-to-ram-snapshot = { workspace = true }`

#### Workspace Cargo.toml modifications
- [ ] Add to `[workspace.dependencies]`:
  - [ ] `rmp-serde = "1.3"`
  - [ ] `pt02-folder-to-ram-snapshot = { path = "crates/pt02-folder-to-ram-snapshot" }`
- [ ] Update workspace `version = "1.7.3"` (affects all crates)

#### General
- [ ] `cargo build --release` compiles entire workspace without errors
- [ ] No warnings in any modified file
- [ ] `parseltongue --help` shows pt02 subcommand
- [ ] `parseltongue pt02-folder-to-ram-snapshot --help` shows usage

### Implementation Notes
- Follows existing pt01/pt08 CLI pattern
- Workspace-level version bump cascades to all crates
- rmp-serde added as workspace dependency for consistency
- CLI output guides user to next step (pt08 with ptgraph: prefix)

---

## Phase 8: README Audit

**Status**: PENDING

**File**: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/README.md`

**Estimated Lines**: Variable (documentation review)

### Acceptance Criteria
- [ ] All 24 endpoint names match actual handler implementations
- [ ] HTTP server port default (7777) is correct
- [ ] CLI command examples are tested and accurate:
  - [ ] `parseltongue pt01-folder-to-cozodb-streamer .`
  - [ ] `parseltongue pt02-folder-to-ram-snapshot .`
  - [ ] `parseltongue pt08-http-code-query-server --db "rocksdb:..."`
  - [ ] `parseltongue pt08-http-code-query-server --db "ptgraph:..."`
- [ ] All curl examples return expected JSON (test against live server)
- [ ] Language support list matches parser capabilities
- [ ] Version number is 1.7.3 throughout
- [ ] Feature claims are accurate (no aspirational features listed)
- [ ] Build/test commands work as documented
- [ ] No broken internal links
- [ ] All code blocks use correct syntax highlighting
- [ ] Quick start guide completes successfully in <5 minutes

### Implementation Notes
- This is a **verification task**, not a rewrite
- README must be a contract, not a wishlist
- Every command must be copy-pasteable and work
- Every endpoint must return real data, not 404/500

---

## Phase 9: Testing Journal

**Status**: PENDING

**File**: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/slim_snapshot_testing_session_journal.md` (new)

**Estimated Lines**: ~200-400 (comprehensive test documentation)

### Acceptance Criteria
- [ ] 4-word filename: `slim_snapshot_testing_session_journal.md`
- [ ] File located at repository root
- [ ] Sections included:
  - [ ] Test environment (OS, Rust version, machine specs)
  - [ ] Build verification (cargo build/test output)
  - [ ] pt02 snapshot generation test (command, output, file size, timing)
  - [ ] pt08 snapshot loading test (startup logs, memory usage)
  - [ ] Endpoint compatibility matrix (all 24 endpoints tested):
    - [ ] 21 working endpoints: command + response sample + verification
    - [ ] 3 guarded endpoints: 501 error message verification
  - [ ] Comparison tests (RocksDB vs snapshot mode):
    - [ ] Entity count match
    - [ ] Edge count match
    - [ ] Query result equivalence
  - [ ] Performance metrics:
    - [ ] pt01 RocksDB ingestion time
    - [ ] pt02 snapshot generation time
    - [ ] pt08 RocksDB startup time
    - [ ] pt08 snapshot load time
    - [ ] RAM usage comparison
    - [ ] Disk usage comparison
  - [ ] Failure cases tested:
    - [ ] Invalid .ptgraph file
    - [ ] Missing snapshot file
    - [ ] Corrupted MessagePack data
  - [ ] Windows testing results (if available)
  - [ ] Known issues or limitations discovered
  - [ ] Sign-off: all acceptance criteria from PRD verified

### Implementation Notes
- This is the **final verification** before v1.7.3 is considered complete
- Serves as regression test reference for future versions
- Documents actual behavior vs. PRD specifications
- Must include both success and failure cases
- Should be reproducible by any developer

---

## Global Acceptance Criteria (All Phases Complete)

**Status**: PENDING

### Build & Test
- [ ] `cargo build --release` - zero errors, zero warnings across entire workspace
- [ ] `cargo test --all` - all tests pass (no regressions)
- [ ] `cargo clippy --all` - no warnings
- [ ] `grep -r "TODO\|STUB\|PLACEHOLDER" --include="*.rs" crates/` - zero results

### Functional Tests (PRD Acceptance Criteria)
- [ ] Test 1: `cargo build --release` compiles cleanly
- [ ] Test 2: `cargo test --all` passes all tests
- [ ] Test 3: `parseltongue pt02-folder-to-ram-snapshot .` produces .ptgraph file and exits 0
- [ ] Test 4: `parseltongue pt08-http-code-query-server --db "ptgraph:analysis.ptgraph"` starts successfully
- [ ] Test 5: `curl localhost:7777/codebase-statistics-overview-summary` returns entity count matching pt01
- [ ] Test 6: `curl localhost:7777/strongly-connected-components-analysis` returns valid JSON
- [ ] Test 7: `curl localhost:7777/smart-context-token-budget` returns 501 "not available in snapshot mode"

### Documentation
- [ ] README.md verified accurate (Phase 8)
- [ ] Testing journal complete (Phase 9)
- [ ] CLAUDE.md updated with pt02 usage examples

### Version Control
- [ ] All Cargo.toml files show version 1.7.3
- [ ] Git status clean (all files committed)
- [ ] Commit message follows convention: "feat(v1.7.3): slim graph snapshots for Windows support"

---

## Implementation Notes

### Build Order (Sequential Dependencies)
1. Phase 1 → Phase 2 → Phase 3 must be sequential (types before use)
2. Phase 4 depends on Phase 1-3 (pt02 uses slim types and export methods)
3. Phase 5 depends on Phase 1-3 (pt08 loader uses slim types and import methods)
4. Phase 6 depends on Phase 5 (guards check snapshot flag from startup)
5. Phase 7 depends on Phase 4 (CLI calls pt02 crate)
6. Phase 8-9 depend on all previous phases (verification)

### Parallel Work Opportunities
- Phase 4 and Phase 5 can be built in parallel (after Phase 3 complete)
- Phase 6 can start immediately after Phase 5 complete
- README audit (Phase 8) can proceed as soon as CLI is working (Phase 7)

### TDD Cycle per Phase
1. Write failing test (if applicable)
2. Implement minimal code to pass
3. Verify `cargo build` succeeds
4. Verify `cargo test` passes
5. Refactor for clarity
6. Move to next phase

### File Watching Note
Snapshot mode intentionally disables file watching because:
- Snapshots are static exports (no live updates)
- File watcher requires `Current_Code` for incremental reindex
- Windows file locking issues apply to file watcher too

---

## Risk Mitigation Tracking

| Risk | Status | Mitigation Applied |
|------|--------|-------------------|
| Peak RAM during pt02 (~3.2GB) | PENDING | Document requirement; pt02 is one-time export |
| interface_signature JSON parsing | PENDING | Default to line_start=0, line_end=0 on failure |
| CozoDB batch insert limits | PENDING | Chunk into 5000-entity batches in import methods |
| Current_Code empty vs null | PENDING | Explicitly set `Current_Code=""` in slim insert |
| Windows testing availability | PENDING | Document in testing journal if unavailable |

---

## Definition of Done

Version 1.7.3 is **COMPLETE** when:
1. All 9 phase checklists show 100% completion
2. All global acceptance criteria pass
3. Testing journal documents full test session
4. README audit shows zero inaccuracies
5. Git commit created with clean status
6. No TODOs/stubs remain in codebase

**Current Status**: 0 of 9 phases complete (0%)

---

## Next Actions for rust-coder-01 Agent

**Start with Phase 1** (Slim Types):
1. Read `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/crates/parseltongue-core/src/entities.rs`
2. Locate insertion point after `IgnoredFileRow` struct
3. Add three structs with derives and doc comments
4. Verify compilation
5. Mark Phase 1 checklist complete

**Then proceed sequentially through Phase 2-7, then Phase 8-9.**

