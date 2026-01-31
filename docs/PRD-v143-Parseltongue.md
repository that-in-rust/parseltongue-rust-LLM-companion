# PRD: Parseltongue v1.4.3 - Fix File Watching

**Version**: 1.4.3
**Status**: Draft
**Created**: 2026-01-31
**Target Release**: 2026-02-01
**Companion Document**: [THESIS-PRD-v143-File-Watching.md](./THESIS-PRD-v143-File-Watching.md)

---

## Executive Summary

**Problem**: Parseltongue v1.4.2 reports file watcher as "running" but **never detects file changes** (0 events processed despite multiple edits).

**Root Cause**: The file watcher uses `blocking_send` in notify's callback thread, which deadlocks when the tokio event loop is slow. Manual incremental reindex works perfectly, proving the issue is isolated to the watcher.

**Solution**: Replace the broken implementation with `notify-debouncer-full` crate (battle-tested, correct event loop handling, built-in debouncing).

**Impact**: Users currently believe file watching works when it's silently broken. This fix enables the promised "always-on" file watching.

---

## Goals

### Primary Goal
Enable **automatic file change detection** that triggers incremental reindexing without user intervention.

### Success Metrics
1. **Event Detection**: `events_processed_total_count` increments when files change
2. **Reindex Trigger**: Logs show `[FileWatcher] Reindexed <file>: +X entities, -Y entities`
3. **Latency**: P99 < 500ms from file save to reindex complete
4. **Reliability**: 0 silent failures (all errors logged)

### Non-Goals
- Live WebSocket streaming (deferred to v1.5.0)
- Diff computation between base/live databases (apwbd feature, deferred)
- Workspace management (apwbd feature, deferred)
- Temporal coupling analysis (apwbd feature, deferred)

---

## Current State Analysis

### What Works ✅
1. **Manual Incremental Reindex** (POST `/incremental-reindex-file-update`)
   - SHA-256 file hashing
   - Entity diff computation (+added, -removed)
   - Edge updates (delete old, insert new)
   - Performance: 9-14ms for typical files

2. **Status Endpoint** (GET `/file-watcher-status-check`)
   - Returns watcher metadata
   - Shows enabled/running flags
   - Reports monitored extensions

3. **Startup Flow**
   - Creates `NotifyFileWatcherProvider`
   - Calls `start_file_watcher_service()`
   - Prints "✓ File watcher started: ."

### What's Broken ❌
1. **Auto File Detection**
   - Symptom: 0 events processed after multiple file edits
   - Root Cause: `blocking_send` in notify callback (line 131 of file_watcher.rs)
   - Impact: Users believe auto-watch works when it doesn't

2. **Event Loop**
   - `tokio::spawn` task never receives events from channel
   - Race condition between notify thread and tokio runtime
   - No error handling when `blocking_send` fails

### Test Results (v1.4.2)

```bash
# Create test file
echo "pub fn test() {}" > test_live_update.rs

# Trigger manual reindex (WORKS)
curl -X POST "http://localhost:7777/incremental-reindex-file-update?path=./test_live_update.rs"
# Response: {"success": true, "data": {"entities_added": 1, "processing_time_ms": 9}}

# Modify file and check status (BROKEN)
echo "pub fn test2() {}" >> test_live_update.rs
sleep 2
curl "http://localhost:7777/file-watcher-status-check" | jq '.data.events_processed_total_count'
# Result: 0 (SHOULD be 1 or higher)
```

---

## Requirements

### Functional Requirements

#### REQ-FW-001: Automatic File Change Detection
**WHEN** a user saves a file with extension `.rs`, `.py`, `.js`, `.ts`, `.go`, or `.java`
**THEN** the system SHALL:
1. Detect the change within 500ms
2. Trigger incremental reindex automatically
3. Increment `events_processed_total_count`
4. Log: `[FileWatcher] Processing Modified: <path>`

**Acceptance Test**:
```bash
# Edit file
echo "// change" >> src/main.rs

# Wait 1 second
sleep 1

# Check events
curl http://localhost:7777/file-watcher-status-check | jq '.data.events_processed_total_count'
# MUST return: >= 1
```

#### REQ-FW-002: Debouncing
**WHEN** a file is saved multiple times within 100ms
**THEN** the system SHALL:
1. Coalesce events (trigger reindex only once)
2. Log: `[FileWatcher] Debouncing: waiting 100ms for more changes`

**Acceptance Test**:
```bash
# Rapid edits
for i in {1..10}; do echo "// $i" >> src/main.rs; done

# Wait 1 second
sleep 1

# Check events (should be 1-2, not 10)
curl http://localhost:7777/file-watcher-status-check | jq '.data.events_processed_total_count'
# MUST return: <= 2
```

#### REQ-FW-003: Extension Filtering
**WHEN** a non-code file is saved (e.g., `.md`, `.txt`, `.pdf`)
**THEN** the system SHALL:
1. Increment `events_processed_total_count` (event detected)
2. Skip reindex (log: `[FileWatcher] Skipping non-code file: README.md`)

#### REQ-FW-004: Graceful Degradation
**WHEN** file watcher fails to start (permissions, unsupported filesystem)
**THEN** the system SHALL:
1. Log: `⚠ Warning: File watcher failed to start: <error>`
2. Continue serving HTTP endpoints
3. Set `watcher_running_status_flag: false`
4. Manual reindex endpoint MUST still work

### Non-Functional Requirements

#### REQ-NFR-001: Performance
- **Latency**: P50 < 100ms, P99 < 500ms (file save to reindex start)
- **Memory**: < 10MB overhead for watcher service
- **CPU**: < 5% idle, < 20% during reindex

#### REQ-NFR-002: Observability
**MUST** log:
- Watcher start: `[FileWatcher] Started: watching . for [rs, py, js, ts, go, java]`
- Events: `[FileWatcher] Event detected: Modified src/main.rs`
- Debouncing: `[FileWatcher] Debouncing: waiting 100ms`
- Reindex: `[FileWatcher] Reindexed src/main.rs: +2 entities, -1 entities (45ms)`
- Errors: `[FileWatcher] Error: Failed to reindex: <error>`

#### REQ-NFR-003: Backwards Compatibility
**MUST** maintain:
- All existing HTTP endpoints (no breaking changes)
- Database format (no schema changes)
- CLI interface (no new required flags)

**MUST NOT**:
- Reintroduce `--watch` flag (removed in v1.4.2)
- Change endpoint URLs or response formats

---

## Implementation Plan

### Phase 1: Update Dependencies
**File**: `crates/pt01-folder-to-cozodb-streamer/Cargo.toml`

```toml
[dependencies]
notify = "6.1"
notify-debouncer-full = "0.3"  # ADD THIS LINE
async-trait = "0.1"
thiserror = "2.0"
tokio = { version = "1.43", features = ["sync", "time"] }
```

**Verification**:
```bash
cargo build -p pt01-folder-to-cozodb-streamer
```

### Phase 2: Rewrite NotifyFileWatcherProvider
**File**: `crates/pt01-folder-to-cozodb-streamer/src/file_watcher.rs`

**Key Changes**:
1. Replace `RecommendedWatcher` with `notify_debouncer_full::Debouncer`
2. Remove manual `mpsc::channel` and `tokio::spawn`
3. Store debouncer in struct (keep it alive)
4. Use debouncer's built-in event handler

**Implementation** (see THESIS document for full code):

```rust
use notify_debouncer_full::{new_debouncer, notify::RecursiveMode, DebounceEventResult};

pub struct NotifyFileWatcherProvider {
    debouncer: Arc<Mutex<Option<Debouncer<RecommendedWatcher, FileIdMap>>>>,
    is_running: Arc<AtomicBool>,
    debounce_duration_ms: u64,
}

impl FileWatchProviderTrait for NotifyFileWatcherProvider {
    async fn start_watching_directory_recursively(
        &self,
        path: &Path,
        callback: FileChangeCallback,
    ) -> WatcherResult<()> {
        let debounce_duration = Duration::from_millis(self.debounce_duration_ms);

        let mut debouncer = new_debouncer(
            debounce_duration,
            None,
            move |result: DebounceEventResult| {
                match result {
                    Ok(events) => {
                        for event in events {
                            if let Some(payload) = convert_notify_event_payload(&event.event) {
                                callback(payload);
                            }
                        }
                    }
                    Err(errors) => {
                        eprintln!("[FileWatcher] Error: {:?}", errors);
                    }
                }
            },
        )?;

        debouncer.watcher().watch(path, RecursiveMode::Recursive)?;

        *self.debouncer.lock().await = Some(debouncer);
        self.is_running.store(true, Ordering::SeqCst);

        Ok(())
    }
}
```

### Phase 3: Enhance Logging
**File**: `crates/pt08-http-code-query-server/src/file_watcher_integration_service.rs`

**Add logging at**:
- Line 144-150: Event detection
- Line 164-230: Debounce logic
- Line 195-227: Reindex trigger

**Example**:
```rust
println!("[FileWatcher] Event detected: {:?} {}", event.change_type, file_path.display());
println!("[FileWatcher] Debouncing: waiting {}ms for more changes", debounce_ms);
println!("[FileWatcher] Processing {:?}: {}", change_type, file_path_str);
println!(
    "[FileWatcher] Reindexed {}: +{} entities, -{} entities, +{} edges, -{} edges ({}ms)",
    result.file_path,
    result.entities_added,
    result.entities_removed,
    result.edges_added,
    result.edges_removed,
    result.processing_time_ms
);
```

### Phase 4: Add E2E Tests
**File**: `crates/pt08-http-code-query-server/tests/integration_file_watcher_test.rs` (NEW)

```rust
#[tokio::test]
async fn test_file_watcher_detects_changes() {
    // 1. Start server with test database
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("analysis.db");

    // 2. Create test file
    let test_file = temp_dir.path().join("test.rs");
    std::fs::write(&test_file, "pub fn initial() {}").unwrap();

    // 3. Start server (file watcher auto-starts)
    let server = start_test_server(&db_path).await;

    // 4. Initial reindex
    server.post_incremental_reindex(&test_file).await;

    // 5. Modify file
    std::fs::write(&test_file, "pub fn modified() {}").unwrap();

    // 6. Wait for auto-detection
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 7. Verify event count increased
    let status = server.get_file_watcher_status().await;
    assert!(status.events_processed_total_count >= 1);

    // 8. Verify entity updated
    let entities = server.search_entities("modified").await;
    assert_eq!(entities.len(), 1);
}
```

### Phase 5: Performance Tests
**File**: `crates/pt01-folder-to-cozodb-streamer/benches/file_watcher_bench.rs` (NEW)

```rust
#[bench]
fn bench_file_watcher_latency(b: &mut Bencher) {
    // Measure: Time from file save to callback trigger
    // Target: P99 < 500ms
}

#[bench]
fn bench_debounce_coalescing(b: &mut Bencher) {
    // Measure: 10 rapid edits should trigger 1-2 callbacks
}
```

### Phase 6: Update Documentation
**Files to update**:
1. `README.md`: Add "Always-on File Watching" section
2. `CLAUDE.md`: Update file watching behavior
3. `docs/04-api-guide.md`: Document `/file-watcher-status-check` endpoint

---

## Testing Strategy

### Unit Tests (pt01 crate)
- `test_notify_provider_starts_watcher`: Verify debouncer created
- `test_notify_provider_stops_watcher`: Verify cleanup
- `test_debouncer_fires_callback`: Verify events reach callback
- `test_extension_filtering`: Verify only code files trigger callback

### Integration Tests (pt08 crate)
- `test_watcher_integration_service_starts`: Verify integration layer
- `test_watcher_detects_file_modification`: E2E file modification
- `test_watcher_detects_file_creation`: E2E file creation
- `test_watcher_detects_file_deletion`: E2E file deletion
- `test_watcher_debounces_rapid_changes`: Verify debounce behavior
- `test_watcher_graceful_degradation`: Verify error handling

### E2E Tests (Manual + Automated)
```bash
# Test 1: Basic file change detection
echo "pub fn test() {}" > test.rs
sleep 1
curl http://localhost:7777/file-watcher-status-check | jq '.data.events_processed_total_count'
# Expected: >= 1

# Test 2: Debouncing
for i in {1..10}; do echo "// $i" >> test.rs; done
sleep 1
curl http://localhost:7777/file-watcher-status-check | jq '.data.events_processed_total_count'
# Expected: <= 2

# Test 3: Non-code files ignored for reindex
echo "test" >> README.md
sleep 1
# Expect: Event count increments, but no reindex log
```

### Performance Tests
```bash
# Measure latency (P50, P99)
cargo bench --bench file_watcher_bench

# Measure memory
/usr/bin/time -l parseltongue pt08-http-code-query-server --db "rocksdb:test/analysis.db"

# Measure CPU
top -pid $(pgrep parseltongue)
```

---

## Edge Cases & Risk Mitigation

### Edge Case 1: Symlinks
**Issue**: Watcher may follow symlinks recursively
**Mitigation**: Configure `RecursiveMode::NonRecursive` for symlinks
**Test**: Create symlink to `/`, verify watcher doesn't explode

### Edge Case 2: Large Files (>100MB)
**Issue**: Reindexing 100MB file may timeout
**Mitigation**: Add file size check, skip files > 10MB
**Test**: Create 50MB file, verify skip log

### Edge Case 3: Network Filesystems (NFS, SMB)
**Issue**: `notify` may not work on network mounts
**Mitigation**: Detect unsupported filesystem, fall back to polling
**Test**: Start server in NFS mount, verify graceful degradation

### Edge Case 4: Permission Denied
**Issue**: Watcher can't read directory
**Mitigation**: Log error, set `watcher_running_status_flag: false`
**Test**: `chmod -r .`, verify error logged

### Edge Case 5: Rapid Server Restarts
**Issue**: Old watcher threads may leak
**Mitigation**: Ensure `Drop` trait cleans up debouncer
**Test**: Restart server 100x, verify no thread leak

---

## Rollout Plan

### Development (Feb 1)
1. Implement Phase 1-2 (notify-debouncer-full integration)
2. Manual testing with test files
3. Verify event count increments

### Testing (Feb 1-2)
1. Add Phase 4 E2E tests
2. Add Phase 5 performance benchmarks
3. Run full test suite

### Release (Feb 2)
1. Update docs (Phase 6)
2. Tag v1.4.3
3. Build release binary
4. Publish to GitHub releases

---

## Success Criteria

### Must Have (v1.4.3)
- [ ] File watcher detects changes automatically (events > 0)
- [ ] Incremental reindex triggered on file save
- [ ] Debouncing works (10 rapid edits = 1-2 callbacks)
- [ ] Graceful degradation on watcher failure
- [ ] Logs show all watcher activity
- [ ] E2E tests pass (create, modify, delete)
- [ ] Performance: P99 < 500ms
- [ ] Zero breaking changes

### Nice to Have (Deferred to v1.5.0)
- WebSocket streaming of diffs (apwbd feature)
- Workspace management (apwbd feature)
- Diff analysis endpoint (apwbd feature)
- Temporal coupling analysis (apwbd feature)

---

## Open Questions

1. **Q**: Should we port the entire apwbd file_watcher_service_module (3,400 lines)?
   **A**: No. Use `notify-debouncer-full` crate instead (simpler, battle-tested).

2. **Q**: Should we support `--watch` flag for backwards compatibility?
   **A**: No. v1.4.2 removed it, users expect always-on behavior.

3. **Q**: What if `notify-debouncer-full` also fails?
   **A**: Graceful degradation - log error, manual reindex still works.

4. **Q**: Should we add WebSocket streaming?
   **A**: Defer to v1.5.0 (out of scope for this fix).

---

## References

- **Thesis Document**: [THESIS-PRD-v143-File-Watching.md](./THESIS-PRD-v143-File-Watching.md)
- **Git History**: [GIT-HISTORY-ANALYSIS.md](./GIT-HISTORY-ANALYSIS.md)
- **v1.4.2 PRD**: [PRD-v142-Parseltongue.md](./PRD-v142-Parseltongue.md)
- **notify-debouncer-full**: https://docs.rs/notify-debouncer-full/0.3/
- **Commit 979ffcb7c**: v1.4.2 release (always-on file watching)
- **Commit b21ed137**: v1.4.1 release (added --watch flag)
- **Commit 4329e8f6d**: apwbd branch (complete file watcher with debouncer)

---

**Status**: Ready for Implementation
**Next Step**: Add `notify-debouncer-full` dependency and rewrite `NotifyFileWatcherProvider`
