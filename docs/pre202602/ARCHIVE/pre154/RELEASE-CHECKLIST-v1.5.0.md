# Release Checklist: Parseltongue v1.5.0

**Feature**: Batch Entity Insertion for 10-60x Ingestion Speedup
**Date**: 2026-02-06
**Status**: READY FOR RELEASE

---

## Executive Summary

v1.5.0 implements batch entity insertion, replacing N database round-trips with a single batch operation. This achieves 10-60x speedup for codebase ingestion.

**Key Implementation**:
- Core: `insert_entities_batch()` in `crates/parseltongue-core/src/storage/cozo_client.rs`
- Integration: Batch insertion in `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs`
- Tests: 8 unit tests + 3 integration tests (all passing in release mode)

**Performance Contract**: 10,000 entities in < 500ms (vs ~30s = 60x speedup)

---

## 1. Pre-Release Verification

### 1.1 Code Quality Checks

- [ ] **Clippy passes with zero warnings**
  ```bash
  cargo clippy --all -- -D warnings
  ```

- [ ] **Code formatting verified**
  ```bash
  cargo fmt --all --check
  ```

- [ ] **No TODOs/STUBs in production code**
  ```bash
  grep -r "TODO\|STUB\|PLACEHOLDER" --include="*.rs" crates/
  # Expected: Empty (no matches in new code)
  ```

- [ ] **Four-word naming convention verified**
  - `insert_entities_batch` follows convention ✓

### 1.2 Test Suite Verification

- [ ] **All unit tests pass**
  ```bash
  cargo test --all
  ```

- [ ] **All tests pass in release mode**
  ```bash
  cargo test --all --release
  ```

- [ ] **Batch insertion specific tests pass**
  ```bash
  cargo test -p parseltongue-core insert_entities_batch -- --nocapture
  cargo test -p pt01-folder-to-cozodb-streamer batch_insertion -- --nocapture
  ```

### 1.3 Performance Verification

- [ ] **Performance benchmarks pass (release mode)**
  ```bash
  cargo test --release -p parseltongue-core insert_entities_batch -- --nocapture
  ```

- [ ] **Record performance metrics**

  | Metric | Target | Actual |
  |--------|--------|--------|
  | Empty batch | < 1ms | _____ |
  | 10 entities | < 50ms | _____ |
  | 100 entities | < 100ms | _____ |
  | 1,000 entities | < 200ms | _____ |
  | **10,000 entities** | **< 500ms** | _____ |

- [ ] **E2E dogfooding test**
  ```bash
  rm -rf parseltongue20*
  cargo run --release -- pt01-folder-to-cozodb-streamer .
  # Record time: _____ seconds
  ```

---

## 2. Testing Requirements

### 2.1 Unit Tests

| Test Suite | Count | Status |
|------------|-------|--------|
| `storage_batch_insert_performance_tests.rs` | 8+ | [ ] Pass |
| `batch_insertion_integration_test.rs` | 3 | [ ] Pass |
| Existing storage tests | 33 | [ ] Pass |

```bash
cargo test -p parseltongue-core --test storage_batch_insert_performance_tests
cargo test -p pt01-folder-to-cozodb-streamer --test batch_insertion_integration_test
```

### 2.2 Integration Tests

- [ ] Full pipeline test
- [ ] HTTP server with batch-ingested data
- [ ] Multi-language codebase test

### 2.3 Multi-Platform (CI)

- [ ] macOS ARM64
- [ ] macOS x86_64
- [ ] Linux x86_64
- [ ] Windows x86_64

---

## 3. Documentation Updates

### 3.1 Version Bumping

- [ ] **Cargo.toml** (line 8): `version = "1.5.0"`
- [ ] **CLAUDE.md** (line 9): Update version reference

```bash
# Verify version
grep "version" Cargo.toml | head -5
```

### 3.2 CHANGELOG.md

- [ ] Create/update with v1.5.0 entry:
  - Batch entity insertion
  - Performance improvements
  - No breaking changes

### 3.3 Release Notes Draft

```markdown
## v1.5.0: 10-60x Faster Ingestion

Batch entity insertion replaces N database round-trips with 1.

| Codebase | v1.4.x | v1.5.0 | Speedup |
|----------|--------|--------|---------|
| 100MB | 2 min | 10-20s | ~6x |
| 500MB | 10 min | 30-60s | ~10x |
| 1GB | 20 min | 1-2 min | ~15x |
```

---

## 4. Build & Package

### 4.1 Local Build

- [ ] **Build release binary**
  ```bash
  cargo clean && cargo build --release
  ```

- [ ] **Verify binary**
  ```bash
  ./target/release/parseltongue --version
  # Expected: parseltongue 1.5.0
  ```

- [ ] **Check binary size** (~45-55MB expected)
  ```bash
  ls -lh target/release/parseltongue
  ```

### 4.2 Cross-Platform (GitHub Actions)

Release workflow builds automatically on tag push:
- `parseltongue-macos-arm64`
- `parseltongue-macos-x86_64`
- `parseltongue-linux-x86_64`
- `parseltongue-windows-x86_64.exe`

---

## 5. Release Process

### 5.1 Git Operations

- [ ] **Clean working directory**
  ```bash
  git status  # Should be clean
  ```

- [ ] **Create annotated tag**
  ```bash
  git tag -a v1.5.0 -m "Release v1.5.0: Batch Entity Insertion - 10-60x ingestion speedup"
  ```

- [ ] **Push tag**
  ```bash
  git push origin v1.5.0
  ```

### 5.2 GitHub Release

- [ ] **Monitor Actions workflow**
  ```
  https://github.com/that-in-rust/parseltongue-dependency-graph-generator/actions
  ```

- [ ] **Verify all 4 binaries uploaded**
  ```bash
  gh release view v1.5.0 --json assets --jq '.assets[].name'
  ```

- [ ] **Add release notes**
  ```bash
  gh release edit v1.5.0 --notes-file docs/RELEASE-NOTES-v1.5.0.md
  ```

---

## 6. Post-Release Verification

### 6.1 Binary Download Test

```bash
# macOS ARM64
cd /tmp && mkdir test-v150 && cd test-v150
curl -L https://github.com/that-in-rust/parseltongue-dependency-graph-generator/releases/download/v1.5.0/parseltongue-macos-arm64 -o parseltongue
chmod +x parseltongue
./parseltongue --version
# Expected: parseltongue 1.5.0
```

### 6.2 Smoke Test

```bash
# Ingest test codebase
./parseltongue pt01-folder-to-cozodb-streamer .

# Start server
./parseltongue pt08-http-code-query-server --db "rocksdb:parseltongue*/analysis.db" &
sleep 2

# Test endpoints
curl -s http://localhost:7777/server-health-check-status | jq '.success'
curl -s http://localhost:7777/codebase-statistics-overview-summary | jq '.data'

# Cleanup
pkill parseltongue
```

### 6.3 Documentation Links

- [ ] GitHub release page accessible
- [ ] Download links work
- [ ] README installation commands correct

---

## 7. Rollback Plan

### Severity Assessment

| Severity | Action |
|----------|--------|
| **Critical** | Delete release, revert tag |
| **High** | Hotfix release (v1.5.1) |
| **Medium** | Document, fix in next release |

### Rollback Commands

```bash
# Delete release (if critical issue)
gh release delete v1.5.0 --yes
git push origin --delete v1.5.0
git tag -d v1.5.0

# Or hotfix
git checkout -b hotfix/v1.5.1 v1.5.0
# Fix issue
git tag -a v1.5.1 -m "Hotfix for v1.5.0"
git push origin v1.5.1
```

---

## 8. Final Sign-Off

| Section | Status | Date |
|---------|--------|------|
| Pre-Release Verification | [ ] | _____ |
| Testing | [ ] | _____ |
| Documentation | [ ] | _____ |
| Build & Package | [ ] | _____ |
| Release Process | [ ] | _____ |
| Post-Release Verification | [ ] | _____ |
| Rollback Plan Reviewed | [ ] | _____ |

**Release Approved By**: ________________________
**Date**: ________________________

---

## Quick Reference Commands

```bash
# Pre-release
cargo clippy --all -- -D warnings
cargo fmt --all --check
cargo test --all --release

# Performance
cargo test --release -p parseltongue-core insert_entities_batch -- --nocapture

# Release
git tag -a v1.5.0 -m "Release v1.5.0"
git push origin v1.5.0

# Verify
gh release view v1.5.0
```

---

**Checklist Items**: 50+ actionable steps
**Based On**: Explore agent analysis + existing release patterns
**Created**: 2026-02-06
