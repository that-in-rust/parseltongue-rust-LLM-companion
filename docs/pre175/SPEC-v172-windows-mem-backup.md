# SPEC: v1.7.2 — Windows In-Memory Ingestion with SQLite Backup

## Progress Tracker

### Part A: UNDO Previous Windows Hacks

- [ ] **A1**: Delete `write_rocksdb_options_file_tuned()` from cozo_client.rs (lines 44-83)
  - [ ] Remove function definition
  - [ ] Remove call to function (lines 123-126)
  - [ ] Update performance notes comment
  - [ ] Add `backup_to_sqlite_file()` method

- [ ] **A2**: Remove `#[cfg(target_os)]` blocks from streamer.rs (lines 780-839)
  - [ ] Delete Windows-specific sequential insert code
  - [ ] Keep ONLY `tokio::join!` concurrent inserts
  - [ ] Add `backup_to_sqlite()` method (around line 1432)

- [ ] **A3**: Remove `sled:` prefix stripping from ingestion_coverage_folder_handler.rs (line 256)
  - [ ] Keep `sqlite:` prefix handling
  - [ ] Remove `.or_else(|| db_path.strip_prefix("sled:"))`

### Part B: ADD mem→backup for Windows

- [ ] **B1**: New engine selection in main.rs (lines 131-142)
  - [ ] Replace current `#[cfg(target_os)]` engine selection
  - [ ] Add `mem` backend for Windows with `backup_needed` flag
  - [ ] Add `backup_target` path variable

- [ ] **B2**: Backup call after ingestion completes (after line 173)
  - [ ] Add backup logic in Windows-only `#[cfg]` block
  - [ ] Call `streamer.backup_to_sqlite(&backup_target)`
  - [ ] Print "Saving to disk..." and "Saved:" messages

- [ ] **B3**: Display path fix for "Next step" output (lines 208-209)
  - [ ] Add `display_db_path` variable logic
  - [ ] Show `sqlite:` prefix on Windows backup path
  - [ ] Preserve original path on Mac/Linux

### Verification

- [ ] `cargo build --release` succeeds
- [ ] `cargo test --all` passes
- [ ] Smoke test on Mac (rocksdb, ~1599 entities)
- [ ] No TODOs/STUBs in modified code
- [ ] Version bump to 1.7.2

---

## Problem
Every persistent storage backend fails on Windows during ingestion:
- **RocksDB**: Write stalls at 75-144MB (Windows Defender scans every new SST file during compaction)
- **SQLite direct writes**: 12KB empty database (silent transaction failures, possibly Defender interference between transactions)
- **Sled**: Abandoned project, data loss bugs

All backends work perfectly on Mac/Linux. The problem is Windows-specific filesystem interference during write-heavy operations.

## Solution
On Windows only, use CozoDB's `mem` (in-memory) backend for ingestion, then call `backup_db()` to atomically dump everything to a SQLite file.

### Why this works
- `mem` backend: Zero filesystem interaction during ingestion. Pure in-memory BTreeMap. No files for Defender to scan.
- `backup_db()`: Creates a fresh SQLite file, writes ALL data in a single atomic transaction. No gaps between transactions for Defender to interfere.
- CozoDB officially recommends this pattern: "Import data into the mem backend, then backup. The backup file IS a working SQLite-backed database."

### Memory usage
- ~1,600 entities + ~10,000 edges ≈ 5-10MB in RAM
- Even 50K entities + 100K edges ≈ 100MB
- Safe for 16GB machines

## What changes (Windows only)

### User experience: ZERO changes
Same CLI commands, same output, same workflow:
```
parseltongue pt01-folder-to-cozodb-streamer .
parseltongue pt08-http-code-query-server --db "sqlite:parseltongueXXX/analysis.db"
```

### Code changes

#### File 1: `crates/parseltongue/src/main.rs` (run_folder_to_cozodb_streamer)
- Remove all existing `#[cfg(target_os = "windows")]` engine selection code
- On Windows: use `mem` for ingestion, then backup to `sqlite:workspace/analysis.db`
- On Mac/Linux: unchanged (`rocksdb:workspace/analysis.db`)

```rust
// Windows: ingest into memory, then backup to SQLite
#[cfg(target_os = "windows")]
let (workspace_db_path, backup_needed) = if db == "mem" {
    ("mem".to_string(), false)
} else {
    println!("  Engine: {} (in-memory ingestion → SQLite backup)", style("mem→SQLite").green());
    ("mem".to_string(), true)
};
#[cfg(target_os = "windows")]
let backup_target = format!("{}/analysis.db", workspace_dir);

// Mac/Linux: direct RocksDB (unchanged)
#[cfg(not(target_os = "windows"))]
let workspace_db_path = if db == "mem" {
    "mem".to_string()
} else {
    format!("rocksdb:{}/analysis.db", workspace_dir)
};
#[cfg(not(target_os = "windows"))]
let backup_needed = false;
```

After ingestion completes, before printing results:
```rust
// Windows: backup mem database to SQLite file
#[cfg(target_os = "windows")]
if backup_needed {
    println!("  {} Saving to disk...", style("↓").cyan());
    streamer.backup_to_sqlite(&backup_target)?;
    println!("  Saved: {}", style(&backup_target).yellow());
}
```

The "Next step" output on Windows should print `sqlite:` prefix:
```rust
#[cfg(target_os = "windows")]
let display_db_path = if backup_needed {
    format!("sqlite:{}", backup_target)
} else {
    workspace_db_path.clone()
};
#[cfg(not(target_os = "windows"))]
let display_db_path = workspace_db_path.clone();
```

#### File 2: `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs`
- Remove all `#[cfg(target_os = "windows")]` sequential insert code
- Keep ONLY the `tokio::join!` concurrent inserts (works for both mem and rocksdb)
- Add a `backup_to_sqlite` method:

```rust
/// Backup in-memory database to SQLite file (Windows mem→SQLite workflow)
pub async fn backup_to_sqlite(&self, path: &str) -> Result<()> {
    self.db.backup_to_sqlite_file(path).await
}
```

#### File 3: `crates/parseltongue-core/src/storage/cozo_client.rs`
- Add `backup_to_sqlite_file` method to `CozoDbStorage`:

```rust
/// Backup database to a SQLite file using CozoDB's backup_db()
pub async fn backup_to_sqlite_file(&self, path: &str) -> Result<()> {
    self.db.backup_db(path).map_err(|e| anyhow::anyhow!("Backup failed: {:?}", e))
}
```

- Remove any Windows-specific OPTIONS file code
- Keep the `sqlite:` prefix handling in `new()` (needed for HTTP server to open backup files)

#### File 4: `crates/pt08-http-code-query-server/.../ingestion_coverage_folder_handler.rs`
- Keep `sqlite:` prefix stripping (needed for backup files)
- Remove `sled:` prefix stripping (Sled is gone)

## What does NOT change
- Mac/Linux: everything identical (RocksDB, tokio::join!, same performance)
- HTTP server: no changes (opens sqlite: or rocksdb: based on --db flag)
- All 22 endpoints: no changes
- Test suite: no changes (tests run on Mac/Linux with mem or rocksdb)

## Previous failed attempts (removed in this version)
- v1.6.7: RocksDB direct IO for compaction — no effect on Windows
- v1.6.8: Windows-only chunked batch inserts — no effect
- v1.6.9: Sled on Windows — abandoned project, data loss
- v1.7.0: SQLite on Windows — 12KB empty database
- v1.7.1: Sequential SQLite inserts on Windows — still 12KB empty
