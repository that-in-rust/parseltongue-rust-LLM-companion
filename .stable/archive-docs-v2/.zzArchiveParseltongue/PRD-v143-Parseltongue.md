# PRD: Parseltongue v1.4.3 - Three Core Requirements

**Version**: 1.4.3
**Status**: Draft
**Created**: 2026-01-31
**Target Release**: 2026-02-02

---

## Executive Summary

Parseltongue v1.4.3 has **exactly three requirements**:

1. **Live watching at super high speed** - File changes detected and reindexed in <100ms
2. **Cross-platform release builds** - macOS (Apple Silicon + Intel) and Linux (x86_64)
3. **All existing APIs working** - 14 endpoints from README must work correctly

**No scope creep.** No new features. Just fix what's broken and ship it.

---

## 1. Live Watching at Super High Speed

### Current State: BROKEN ❌
- File watcher reports "running" but detects **0 events**
- Users believe auto-watch works when it doesn't
- Root cause: `blocking_send` deadlock in notify callback

### Target State: WORKING ✅
- **P50 latency**: < 50ms (file save → reindex start)
- **P99 latency**: < 100ms (match VS Code responsiveness)
- **Event detection**: 100% of file changes captured
- **Debouncing**: 10 rapid edits → 1 reindex (coalescing works)

### Implementation: notify-debouncer-full
Based on research (see PLAN-v143-High-Speed-File-Watching.md):
- Replace broken manual event loop with `notify-debouncer-full` crate
- Battle-tested by watchexec, rust-analyzer, 30M+ repositories
- Correct async event handling (no deadlocks)
- Built-in debouncing (100ms default)

### Acceptance Test
```bash
# Test 1: Basic detection
echo "fn test() {}" > test.rs
sleep 0.2
curl http://localhost:7777/file-watcher-status-check | jq '.data.events_processed_total_count'
# MUST return: >= 1

# Test 2: Performance
time echo "// change" >> test.rs
# MUST complete reindex in < 100ms (P99)

# Test 3: Debouncing
for i in {1..10}; do echo "// $i" >> test.rs; sleep 0.02; done
sleep 0.2
curl http://localhost:7777/file-watcher-status-check | jq '.data.events_processed_total_count'
# MUST return: <= 2 (coalescing worked)
```

### Files to Change
1. `crates/pt01-folder-to-cozodb-streamer/Cargo.toml` - Add `notify-debouncer-full = "0.3"`
2. `crates/pt01-folder-to-cozodb-streamer/src/file_watcher.rs` - Replace `RecommendedWatcher` with `Debouncer`
3. `crates/pt08-http-code-query-server/src/http_server_startup_runner.rs` - **Add missing 6 languages** (C, C++, Ruby, PHP, C#, Swift)
4. `crates/pt08-http-code-query-server/src/file_watcher_integration_service.rs` - Add comprehensive logging

### Critical Fix: All 12 Languages
**Current bug**: File watcher only monitors 6/12 languages!

**Fix**: Update `http_server_startup_runner.rs` line 342-349:
```rust
let extensions = vec![
    // Currently monitored (6 languages)
    "rs".to_string(),      // Rust
    "py".to_string(),      // Python
    "js".to_string(),      // JavaScript
    "ts".to_string(),      // TypeScript
    "go".to_string(),      // Go
    "java".to_string(),    // Java

    // MISSING - Add these (6 languages)
    "c".to_string(),       // C
    "h".to_string(),       // C header
    "cpp".to_string(),     // C++
    "cc".to_string(),      // C++ alternate
    "cxx".to_string(),     // C++ alternate
    "hpp".to_string(),     // C++ header
    "rb".to_string(),      // Ruby
    "php".to_string(),     // PHP
    "cs".to_string(),      // C#
    "swift".to_string(),   // Swift
];
```

### Success Criteria
- [ ] Events > 0 when files change
- [ ] P99 latency < 100ms
- [ ] Debouncing works (10 edits = 1-2 callbacks)
- [ ] **All 12 languages monitored** (.rs .py .js .ts .go .java .c .h .cpp .hpp .rb .php .cs .swift)
- [ ] Logs show all watcher activity
- [ ] E2E tests pass

---

## 2. Cross-Platform Release Builds

### Current State: Manual Downloads Only
- Users must download from GitHub releases
- Only macOS universal binary available
- No Linux builds
- No automated CI/CD

### Target State: Automated Releases
Build matrix:
- **macOS**:
  - Apple Silicon (aarch64-apple-darwin)
  - Intel (x86_64-apple-darwin)
  - Universal binary (both architectures)
- **Linux**:
  - x86_64 (x86_64-unknown-linux-gnu)
  - musl (x86_64-unknown-linux-musl) - static binary

### Implementation: GitHub Actions
Create `.github/workflows/release.yml`:

```yaml
name: Release Builds

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: aarch64-apple-darwin
            os: macos-14
            name: parseltongue-macos-arm64
          - target: x86_64-apple-darwin
            os: macos-13
            name: parseltongue-macos-x64
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            name: parseltongue-linux-x64
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            name: parseltongue-linux-x64-musl

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Create artifact
        run: |
          cp target/${{ matrix.target }}/release/parseltongue ${{ matrix.name }}
          chmod +x ${{ matrix.name }}

      - name: Upload to release
        uses: softprops/action-gh-release@v1
        with:
          files: ${{ matrix.name }}
```

### Acceptance Test
```bash
# macOS (Apple Silicon)
curl -L https://github.com/.../parseltongue-macos-arm64 -o parseltongue
chmod +x parseltongue
./parseltongue --version
# MUST show: parseltongue 1.4.3

# macOS (Intel)
curl -L https://github.com/.../parseltongue-macos-x64 -o parseltongue
# Same test

# Linux
curl -L https://github.com/.../parseltongue-linux-x64 -o parseltongue
# Same test
```

### Success Criteria
- [ ] GitHub Actions workflow exists
- [ ] Builds triggered on tag push (v1.4.3)
- [ ] 4 binaries uploaded to release
- [ ] All binaries executable and show correct version

---

## 3. All Existing APIs Working

### Current State: Mixed ✅❌
From README.md, these 14 endpoints MUST work:

| Endpoint | Status | Issue |
|----------|--------|-------|
| `/server-health-check-status` | ✅ Working | None |
| `/codebase-statistics-overview-summary` | ❌ BROKEN | Returns null counts |
| `/code-entities-search-fuzzy` | ⚠️ Inconsistent | Works for "handle", fails for "main" |
| `/reverse-callers-query-graph` | ✅ Working | None |
| `/blast-radius-impact-analysis` | ✅ Working | None |
| `/smart-context-token-budget` | ❌ BROKEN | Returns 0 entities always |
| `/code-entities-list-all` | ✅ Working | None |
| `/code-entity-detail-view` | ✅ Working | None |
| `/forward-callees-query-graph` | ✅ Working | None |
| `/dependency-edges-list-all` | ✅ Working | None |
| `/circular-dependency-detection-scan` | ✅ Working | None |
| `/complexity-hotspots-ranking-view` | ⚠️ Returns externals | Shows "unknown:0-0" |
| `/semantic-cluster-grouping-list` | ✅ Working | None |
| `/api-reference-documentation-help` | ✅ Working | None |

**Additional endpoints** (not in README, but exposed):
- `/incremental-reindex-file-update` (POST) - ✅ Working perfectly
- `/file-watcher-status-check` (GET) - ✅ Working

### Target State: 100% Working
All 14 README endpoints MUST return valid data:
- No null values in critical fields
- No "unknown:0-0" entities
- Consistent search results
- Smart context returns entities

### Fixes Required

#### Fix 1: Codebase Statistics (NULL counts)
**File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/codebase_statistics_overview_handler.rs`

**Issue**: Returns `null` for entity/edge counts

**Fix**: Ensure counts are populated from database
```rust
// Query total entities
let total_entities = storage.raw_query(
    "?[count(key)] := *CodeGraph{key}"
).await?;

// Extract count value correctly
let count = total_entities.rows[0][0].as_i64().unwrap_or(0);
```

#### Fix 2: Smart Context (0 entities)
**File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/smart_context_token_budget_handler.rs`

**Issue**: Returns 0 entities always

**Root cause**: Query logic broken or entity filtering too aggressive

**Fix**: Debug query, verify entities returned

#### Fix 3: Fuzzy Search (Inconsistent)
**File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/code_entities_fuzzy_search_handler.rs`

**Issue**: Works for "handle", fails for "main"

**Fix**: Check query logic, ensure case-insensitive matching

### Acceptance Test
Run full endpoint test suite:

```bash
# Test 1: Health check
curl http://localhost:7777/server-health-check-status
# Expected: {"success": true, "data": {...}}

# Test 2: Statistics (MUST NOT BE NULL)
curl http://localhost:7777/codebase-statistics-overview-summary | jq '.data.total_entity_count'
# Expected: > 0 (not null)

# Test 3: Fuzzy search
curl "http://localhost:7777/code-entities-search-fuzzy?q=main"
# Expected: At least 1 result

# Test 4: Smart context (MUST RETURN ENTITIES)
curl "http://localhost:7777/smart-context-token-budget?focus=rust:fn:main&tokens=2000"
# Expected: entities_included > 0 (not 0)

# Test 5: Blast radius
curl "http://localhost:7777/blast-radius-impact-analysis?entity=rust:fn:main&hops=2"
# Expected: total_affected > 0

# Test 6-14: Run remaining endpoints
# All MUST return valid JSON with success: true
```

### Success Criteria
- [ ] All 14 README endpoints return valid data
- [ ] No null values in critical fields
- [ ] No "unknown:0-0" entities in results
- [ ] Smart context returns > 0 entities
- [ ] Fuzzy search works for common terms

---

## Implementation Timeline

### Day 1 (Feb 1): Fix File Watching
**Duration**: 6-8 hours

1. Add `notify-debouncer-full` dependency (5 min)
2. Rewrite `NotifyFileWatcherProvider` (2 hours)
3. Add comprehensive logging (30 min)
4. Write E2E tests (2 hours)
5. Run performance benchmarks (1 hour)
6. Verify P99 < 100ms (30 min)

**Go/No-Go**: If tests pass and P99 < 100ms, proceed. Otherwise, debug.

### Day 2 (Feb 2): Fix Broken APIs
**Duration**: 4-6 hours

1. Fix codebase statistics (1 hour)
2. Fix smart context (2 hours)
3. Fix fuzzy search (1 hour)
4. Test all 14 endpoints (1 hour)
5. Document any remaining issues (30 min)

**Go/No-Go**: If all 14 endpoints return valid data, proceed. Otherwise, iterate.

### Day 2 (Feb 2): Setup CI/CD
**Duration**: 2-4 hours

1. Create `.github/workflows/release.yml` (1 hour)
2. Test workflow on feature branch (1 hour)
3. Tag v1.4.3 and trigger release (10 min)
4. Verify all 4 binaries uploaded (30 min)
5. Test downloads on macOS/Linux (1 hour)

**Go/No-Go**: If all binaries download and run, ship. Otherwise, fix workflow.

---

## Out of Scope (Deferred to v1.5.0)

**NOT in v1.4.3**:
- WebSocket streaming (apwbd feature)
- Diff analysis endpoint (apwbd feature)
- Workspace management (apwbd feature)
- Temporal coupling (apwbd feature)
- Embedded React frontend (apwbd feature)
- Incremental tree-sitter parsing (optimization)
- GraphQL endpoint
- Authentication/authorization
- Batch operations

**Why defer?** v1.4.3 is a **bug fix release**. Focus on fixing what's broken, not adding features.

---

## Success Criteria Summary

### Must Have (v1.4.3)
- [ ] **Req 1**: File watcher detects changes (events > 0, P99 < 100ms)
- [ ] **Req 2**: Release builds for macOS (arm64 + x64) and Linux (x64 + musl)
- [ ] **Req 3**: All 14 README endpoints return valid data (no nulls, no 0 entities)

### Nice to Have (Deferred)
- Incremental parsing (v1.5.0)
- WebSocket streaming (v1.5.0)
- Workspace management (v1.5.0)

---

## References

- **Thesis Document**: [THESIS-PRD-v143-File-Watching.md](./THESIS-PRD-v143-File-Watching.md) - 61KB root cause analysis
- **Research Plan**: [PLAN-v143-High-Speed-File-Watching.md](./PLAN-v143-High-Speed-File-Watching.md) - Best-in-class architectures
- **README**: [../README.md](../README.md) - All 14 API endpoints
- **v1.4.2 Commit**: `979ffcb7c` - Always-on file watching (broken)

---

## Testing Checklist

### File Watching Tests
- [ ] Basic detection (edit file → events > 0)
- [ ] Performance (P99 < 100ms)
- [ ] Debouncing (10 edits → 1-2 callbacks)
- [ ] **Extension filtering (all 12 languages: .rs .py .js .ts .go .java .c .h .cpp .hpp .rb .php .cs .swift)**
- [ ] Test each language: Create .c file → verify event detected
- [ ] Graceful degradation (watcher failure → log error)

### API Tests (All 14 Endpoints)
- [ ] `/server-health-check-status`
- [ ] `/codebase-statistics-overview-summary` (fix null counts)
- [ ] `/code-entities-search-fuzzy` (fix inconsistent results)
- [ ] `/reverse-callers-query-graph`
- [ ] `/blast-radius-impact-analysis`
- [ ] `/smart-context-token-budget` (fix 0 entities)
- [ ] `/code-entities-list-all`
- [ ] `/code-entity-detail-view`
- [ ] `/forward-callees-query-graph`
- [ ] `/dependency-edges-list-all`
- [ ] `/circular-dependency-detection-scan`
- [ ] `/complexity-hotspots-ranking-view`
- [ ] `/semantic-cluster-grouping-list`
- [ ] `/api-reference-documentation-help`

### Build Tests
- [ ] macOS arm64 build works
- [ ] macOS x64 build works
- [ ] Linux x64 build works
- [ ] Linux musl build works
- [ ] All binaries show correct version

---

**Status**: Ready for Implementation
**Next Step**: Add `notify-debouncer-full` dependency and fix file watcher
**Target Release**: February 2, 2026
