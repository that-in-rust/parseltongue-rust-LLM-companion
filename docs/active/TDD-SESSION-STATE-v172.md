# TDD Session State: v1.7.2 Windows mem→SQLite Backup Implementation

**Session Start**: 2026-02-12
**Current Phase**: RED (Tracking implementation by rust-coder-01 agent)
**Working Directory**: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator`

---

## Current Phase: Tracking & Documentation

This is a **context retention and tracking** session. The actual implementation is being performed in parallel by the `rust-coder-01` agent. This document serves as the persistent memory for the TDD workflow.

---

## Implementation Overview

### Problem Statement
Six previous Windows workarounds (v1.6.7-v1.7.1) failed due to Windows Defender filesystem interference:
- RocksDB: Write stalls at 75-144MB (Defender scans every SST file during compaction)
- SQLite direct writes: 12KB empty databases (transaction interference)
- Sled: Abandoned project, data loss bugs

### Solution: mem→SQLite Backup (v1.7.2)
On Windows only:
1. Use CozoDB's `mem` (in-memory) backend for ingestion → zero filesystem interaction
2. After ingestion completes, call `backup_db()` → atomic SQLite file creation
3. HTTP server opens the SQLite backup file normally

**Why this works**: No files for Defender to scan during ingestion, single atomic transaction for backup.

---

## Files Being Modified

### File 1: `crates/parseltongue-core/src/storage/cozo_client.rs`
**Current State**: Contains Windows-specific RocksDB OPTIONS file tuning (v1.6.7-v1.6.9)

**Changes Required**:
- **DELETE**: `write_rocksdb_options_file_tuned()` function (lines 44-83)
- **DELETE**: Call to that function (lines 123-126)
- **UPDATE**: Performance notes comment (line 104) - reference v1.7.2 mem→SQLite approach
- **ADD**: New method `backup_to_sqlite_file(&self, path: &str) -> Result<()>`

**Key Dependencies**:
- Uses `CozoDbStorage.db.backup_db()` from CozoDB crate
- Returns `ParseltongError::DatabaseError` on failure

---

### File 2: `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs`
**Current State**: Contains Windows-specific sequential insert code (v1.7.1, lines 780-839)

**Changes Required**:
- **DELETE**: Windows `#[cfg(target_os = "windows")]` sequential insert block (lines 830-839)
- **DELETE**: Mac/Linux `#[cfg(not(target_os = "windows"))]` wrapper (lines 780-828)
- **KEEP**: Only `tokio::join!` concurrent inserts (works for both mem and rocksdb)
- **ADD**: New method `backup_to_sqlite(&self, path: &str) -> Result<()>` (around line 1432)

**Key Dependencies**:
- Calls `self.db.backup_to_sqlite_file(path)` from cozo_client.rs
- Must be added before `#[cfg(test)]` section

**Performance Contract**:
- Concurrent inserts work for both mem (in-memory BTreeMap) and rocksdb
- No write contention in mem backend

---

### File 3: `crates/parseltongue/src/main.rs`
**Current State**: Windows uses SQLite directly (v1.7.0, lines 131-142)

**Changes Required**:
- **REPLACE**: Engine selection logic (lines 131-142)
  - Windows: `mem` backend + `backup_needed=true` flag
  - Mac/Linux: `rocksdb:` (unchanged)
- **ADD**: Backup call after ingestion (after line 173, before error log section)
  - Windows-only `#[cfg(target_os = "windows")]` block
  - Call `streamer.backup_to_sqlite(&backup_target)?`
- **UPDATE**: Display path logic (lines 208-209)
  - Show `sqlite:` prefix for Windows backup files
  - Preserve original path for Mac/Linux

**Key Variables**:
- `workspace_db_path`: Engine string passed to streamer ("mem" on Windows)
- `backup_target`: SQLite file path (`{workspace_dir}/analysis.db`)
- `backup_needed`: Boolean flag (true on Windows)
- `display_db_path`: User-facing path for "Next step" output

---

### File 4: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_coverage_folder_handler.rs`
**Current State**: Strips `rocksdb:`, `sled:`, and `sqlite:` prefixes (lines 254-258)

**Changes Required**:
- **DELETE**: `sled:` prefix handling (Sled project abandoned)
- **KEEP**: `rocksdb:` and `sqlite:` prefix handling (both still used)
- **UPDATE**: Line 256 - remove `.or_else(|| db_path.strip_prefix("sled:"))`

**Rationale**: HTTP server needs to strip prefixes to derive workspace directory path.

---

## Tests Written

No new tests required for this version. This is a **platform-specific implementation change** that maintains the existing API contract.

**Existing test coverage** (unchanged):
- Mac/Linux: Uses rocksdb backend (existing tests continue to pass)
- Windows: Uses mem backend during ingestion (same API, different backend)
- HTTP server: Opens sqlite: or rocksdb: files (no API changes)

**Smoke test verification** (manual):
```bash
# Mac/Linux (should use rocksdb, ~1599 entities for self-ingestion)
parseltongue pt01-folder-to-cozodb-streamer .
# Output should show: Workspace: parseltongue20260212HHMMSS
#                     Database: rocksdb:parseltongue20260212HHMMSS/analysis.db

# Windows (should use mem→sqlite backup)
parseltongue pt01-folder-to-cozodb-streamer .
# Output should show: Engine: mem→SQLite (in-memory ingestion → SQLite backup)
#                     ↓ Saving to disk...
#                     Saved: parseltongue20260212HHMMSS/analysis.db
```

---

## Implementation Progress

### Completed Changes
*(To be updated by rust-coder-01 agent as work proceeds)*

- [ ] A1: cozo_client.rs - Remove RocksDB OPTIONS file code
- [ ] A1: cozo_client.rs - Add backup_to_sqlite_file() method
- [ ] A2: streamer.rs - Remove Windows sequential insert code
- [ ] A2: streamer.rs - Add backup_to_sqlite() method
- [ ] A3: ingestion_coverage_folder_handler.rs - Remove sled: prefix handling
- [ ] B1: main.rs - Update engine selection logic
- [ ] B2: main.rs - Add backup call after ingestion
- [ ] B3: main.rs - Fix display path for Windows

### Current Focus
**Awaiting implementation by rust-coder-01 agent**

---

## Next Steps

1. **Implementation**: rust-coder-01 agent applies changes to 4 files
2. **Build Verification**: `cargo build --release`
3. **Test Verification**: `cargo test --all`
4. **Smoke Test (Mac)**: Verify rocksdb backend still works (~1599 entities)
5. **Version Bump**: Update version to 1.7.2 in Cargo.toml files
6. **Commit**: Create git commit with changes

---

## Context Notes

### Key Decisions Made

**Why mem backend instead of SQLite direct writes?**
- mem backend = pure in-memory BTreeMap, zero filesystem interaction during ingestion
- SQLite direct writes failed due to Windows Defender interference between transactions
- mem→backup pattern is CozoDB-recommended: "Import data into mem, then backup"

**Why backup_db() instead of manual export?**
- CozoDB's `backup_db()` creates a fresh SQLite file in a single atomic transaction
- No gaps for Defender to interfere between transactions
- The backup file IS a working SQLite database (can be opened with `sqlite:` prefix)

**Why only Windows?**
- Mac/Linux: RocksDB works perfectly, no changes needed
- Windows: Unique filesystem behavior (Defender scans) requires workaround
- Platform-specific solution minimizes code complexity

### Technical Debt Identified

**v1.7.2 removes significant technical debt**:
1. ❌ RocksDB OPTIONS file tuning (v1.6.7) - removed
2. ❌ Windows sequential inserts (v1.7.1) - removed
3. ❌ Sled references (v1.6.9) - removed
4. ✅ Clean platform-specific abstraction - Windows uses mem, Mac/Linux uses rocksdb

**Remaining simplification opportunities**:
- None identified - this is the cleanest solution

---

## Cross-File Dependencies

### Dependency Flow
```
main.rs (binary)
  └─> streamer.rs (Tool 1)
       └─> cozo_client.rs (Core)
            └─> CozoDB crate (external)

ingestion_coverage_folder_handler.rs (Tool 8)
  └─> Uses database path format conventions
       └─> Must handle sqlite: and rocksdb: prefixes
```

### Critical Ordering
1. **cozo_client.rs** must be updated first (defines `backup_to_sqlite_file()`)
2. **streamer.rs** depends on cozo_client.rs method
3. **main.rs** depends on streamer.rs method
4. **ingestion_coverage_folder_handler.rs** is independent (prefix handling only)

---

## Performance Implications

### Memory Usage (Windows mem backend)
- Parseltongue self-ingestion: ~1,600 entities + ~10,000 edges ≈ **5-10MB RAM**
- Large codebase (50K entities, 100K edges): ≈ **100MB RAM**
- Safe for 16GB machines (typical Windows development environment)

### Speed Comparison
- **Mac/Linux (rocksdb)**: ~2 seconds for self-ingestion (unchanged)
- **Windows (mem→backup)**: Expected ~2-3 seconds
  - Ingestion: Faster (pure memory, no filesystem I/O)
  - Backup: ~1 second (single SQLite transaction)
  - **Net effect**: Similar or faster than previous Windows approaches

### HTTP Server Performance
- No changes - opens sqlite: or rocksdb: files normally
- All 22 endpoints unchanged

---

## Error Handling Strategy

### Backup Failures
If `backup_db()` fails on Windows:
- Return `StreamerError` with descriptive message
- No partial database (backup is atomic - all or nothing)
- User sees clear error, can retry ingestion

### Fallback Strategy
None needed - if mem backend fails (OOM), user should:
1. Close other applications to free memory
2. Use `--exclude` flags to reduce scope
3. Report issue (but unlikely on 16GB+ machines)

---

## Git Commit Plan

**Commit Message** (when implementation complete):
```
feat(v1.7.2): Windows mem→SQLite backup, remove 6 failed workarounds

Problem: Windows Defender filesystem interference broke every persistent backend:
- RocksDB: 75MB write stall (Defender scans SST files during compaction)
- SQLite direct: 12KB empty DB (transaction interference)
- Sled: Abandoned, data loss bugs

Solution: mem→SQLite backup pattern (Windows only)
- Ingest into mem backend (pure in-memory, zero filesystem I/O)
- Call backup_db() after ingestion (atomic SQLite file creation)
- HTTP server opens SQLite backup normally

Changes:
A1. cozo_client.rs: Delete RocksDB OPTIONS tuning, add backup_to_sqlite_file()
A2. streamer.rs: Remove Windows sequential inserts, add backup_to_sqlite()
A3. ingestion_coverage_folder_handler.rs: Remove sled: prefix handling
B1. main.rs: Windows uses mem backend with backup_needed flag
B2. main.rs: Backup call after ingestion completes
B3. main.rs: Display sqlite: prefix for Windows in "Next step" output

Mac/Linux: No changes, rocksdb continues to work perfectly
Windows: mem backend during ingestion, SQLite file after backup
Memory: 5-10MB for typical ingestion, safe for 16GB machines

Removes technical debt from v1.6.7-v1.7.1 (6 failed attempts)

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
```

---

## Verification Checklist

Before marking v1.7.2 complete:

### Build & Test
- [ ] `cargo build --release` succeeds with zero warnings
- [ ] `cargo test --all` passes with zero failures
- [ ] `cargo clean && cargo build --release` succeeds (full rebuild)

### Code Quality
- [ ] No TODOs, STUBs, or PLACEHOLDER comments in modified files
- [ ] All function names follow 4-Word Naming Convention
- [ ] No Windows-specific code outside main.rs and streamer.rs backup logic
- [ ] Comments explain "why" (Windows Defender workaround), not just "what"

### Smoke Test (Mac/Linux)
- [ ] Ingest parseltongue self (should show ~1599 entities)
- [ ] Database path shows `rocksdb:parseltongueXXXX/analysis.db`
- [ ] HTTP server starts successfully
- [ ] `/codebase-statistics-overview-summary` returns correct entity count

### Manual Verification (Windows - if available)
- [ ] Ingest shows "Engine: mem→SQLite" message
- [ ] Backup progress messages appear ("Saving to disk...", "Saved: ...")
- [ ] Database path shows `sqlite:parseltongueXXXX/analysis.db` in output
- [ ] HTTP server opens SQLite file successfully

### Version Management
- [ ] Version bumped to 1.7.2 in root Cargo.toml
- [ ] CHANGELOG.md entry added (if exists)
- [ ] Git status clean (no uncommitted changes)

---

## Questions & Blockers

### Open Questions
- None currently - spec is complete and unambiguous

### Resolved Questions
1. **Q**: Why not use SQLite with WAL mode for Windows?
   **A**: Tried in v1.7.0/v1.7.1 - Defender still interfered between transactions

2. **Q**: Why not increase RocksDB buffer sizes further?
   **A**: Tried in v1.6.7/v1.6.8 - no effect, Defender scans are the bottleneck

3. **Q**: Why not disable Defender?
   **A**: Not a viable solution - users can't/won't disable security software

### Blockers
- None - all dependencies available, spec complete

---

## References

### Related Documentation
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/docs/SPEC-v172-windows-mem-backup.md` - Full specification
- `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator/docs/WINDOWS-SPECIFIC-CODE-TO-REMOVE.md` - Code removal guide

### CozoDB Documentation
- `backup_db()` method: Creates SQLite file in single atomic transaction
- mem backend: Pure in-memory BTreeMap, zero filesystem interaction
- Recommended pattern: "Import data into mem backend, then backup"

### Previous Versions (Failed Approaches)
- v1.6.7: RocksDB direct IO for compaction
- v1.6.8: Windows chunked batch inserts
- v1.6.9: Sled on Windows
- v1.7.0: SQLite on Windows
- v1.7.1: Sequential SQLite inserts

All failed due to Windows Defender filesystem interference.

---

## Session Metadata

**Platform**: darwin (macOS)
**Working Directory**: `/Users/amuldotexe/Desktop/A01_20260131/parseltongue-dependency-graph-generator`
**Git Branch**: main
**Git Status**: Clean (untracked files: test-fixtures/v156-sql-test/, tracking docs)

**Implementation Approach**: Parallel agent execution
- **rust-coder-01**: Implements code changes
- **context-retention-specialist** (this agent): Tracks progress, maintains state

**Expected Duration**: 30-45 minutes for full implementation + verification

---

**Last Updated**: 2026-02-12 (session start)
**Next Update**: After rust-coder-01 completes first file modification
