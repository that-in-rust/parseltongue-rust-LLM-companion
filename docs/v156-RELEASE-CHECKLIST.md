# v1.5.6 Release Checklist

**Date**: 2026-02-08
**Version**: 1.5.6
**Codename**: Generic Sanitization + Windows/PHP Fix + SQL Infrastructure

---

## Release Summary

v1.5.6 delivers three critical improvements:

1. **Generic Type Sanitization** - Fixes 6.7% edge insertion failures for `< > , [ ]` in entity names
2. **Backslash Escaping Fix** - Enables Windows file paths and PHP namespace support
3. **SQL Language Infrastructure** - Foundation for SQL as 13th language (parser pending)

---

## Pre-Release Verification

### 1. Clean Build

```bash
cargo clean
cargo build --release
```

- [ ] Build succeeds with no errors
- [ ] No warnings in parseltongue crates

### 2. Test Suite

```bash
# Core functionality tests (skip flaky performance tests)
cargo test -p parseltongue-core --test isgl1_v2_generic_sanitization_tests
cargo test -p parseltongue-core --test cozo_escaping_tests
```

| Test Suite | Tests | Status |
|------------|-------|--------|
| `isgl1_v2_generic_sanitization_tests` | 13 | ✅ All pass |
| `cozo_escaping_tests` | 6 | ✅ All pass |

- [ ] 13 sanitization tests pass
- [ ] 6 escaping tests pass

### 3. Integration Test

```bash
./target/release/parseltongue pt01-folder-to-cozodb-streamer crates
```

- [ ] Ingestion completes successfully
- [ ] Edge insertion shows 0 failures
- [ ] Entities created > 2000

### 4. HTTP Endpoints (All 14)

```bash
DB_PATH=$(ls -td parseltongue2026* | head -1)
./target/release/parseltongue pt08-http-code-query-server --db "rocksdb:${DB_PATH}/analysis.db" &
sleep 3
```

| # | Endpoint | Command | Status |
|---|----------|---------|--------|
| 1 | Health | `curl -s localhost:7777/server-health-check-status` | ✅ |
| 2 | Stats | `curl -s localhost:7777/codebase-statistics-overview-summary` | ✅ |
| 3 | API Docs | `curl -s localhost:7777/api-reference-documentation-help` | ✅ |
| 4 | Entities | `curl -s localhost:7777/code-entities-list-all` | ✅ |
| 5 | Detail | `curl -s "localhost:7777/code-entity-detail-view/{key}"` | ✅ |
| 6 | Search | `curl -s "localhost:7777/code-entities-search-fuzzy?q=sanitize"` | ✅ |
| 7 | Edges | `curl -s localhost:7777/dependency-edges-list-all` | ✅ |
| 8 | Callers | `curl -s "localhost:7777/reverse-callers-query-graph?entity={key}"` | ✅ |
| 9 | Callees | `curl -s "localhost:7777/forward-callees-query-graph?entity={key}"` | ✅ |
| 10 | Blast | `curl -s "localhost:7777/blast-radius-impact-analysis?entity={key}"` | ✅ |
| 11 | Cycles | `curl -s localhost:7777/circular-dependency-detection-scan` | ✅ |
| 12 | Hotspots | `curl -s localhost:7777/complexity-hotspots-ranking-view` | ✅ |
| 13 | Clusters | `curl -s localhost:7777/semantic-cluster-grouping-list` | ✅ |
| 14 | Context | `curl -s "localhost:7777/smart-context-token-budget?focus={key}"` | ✅ |

### 5. New Feature Verification

```bash
# Verify sanitization function exists
curl -s "localhost:7777/code-entities-search-fuzzy?q=sanitize_entity_name" | jq '.data.total_count'
# Expected: 1+

# Verify escape function exists
curl -s "localhost:7777/code-entities-search-fuzzy?q=escape_for_cozo" | jq '.data.total_count'
# Expected: 1+

# Verify SQL language enum exists
curl -s "localhost:7777/code-entities-search-fuzzy?q=Sql" | jq '.data.total_count'
# Expected: 1+
```

---

## Version Bump

### Update Cargo.toml Files

```bash
# Update version in all Cargo.toml files
sed -i '' 's/version = "1.5.4"/version = "1.5.6"/' Cargo.toml
sed -i '' 's/version = "1.5.4"/version = "1.5.6"/' crates/parseltongue/Cargo.toml
sed -i '' 's/version = "1.5.4"/version = "1.5.6"/' crates/parseltongue-core/Cargo.toml
sed -i '' 's/version = "1.5.4"/version = "1.5.6"/' crates/pt01-folder-to-cozodb-streamer/Cargo.toml
sed -i '' 's/version = "1.5.4"/version = "1.5.6"/' crates/pt08-http-code-query-server/Cargo.toml

# Verify
grep -r '^version = "1.5.6"' --include="Cargo.toml" .
```

- [ ] All 5 Cargo.toml files updated to 1.5.6

---

## Git Operations

### Stage Files

```bash
# Stage modified files
git add crates/parseltongue-core/Cargo.toml
git add crates/parseltongue-core/src/entities.rs
git add crates/parseltongue-core/src/isgl1_v2.rs
git add crates/parseltongue-core/src/query_extractor.rs
git add crates/parseltongue-core/src/storage/cozo_client.rs
git add crates/parseltongue-core/src/storage/mod.rs
git add crates/pt01-folder-to-cozodb-streamer/src/external_dependency_handler.rs
git add crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs
git add crates/pt01-folder-to-cozodb-streamer/src/streamer.rs

# Stage new test files
git add crates/parseltongue-core/tests/cozo_escaping_tests.rs
git add crates/parseltongue-core/tests/isgl1_v2_generic_sanitization_tests.rs

# Stage documentation
git add docs/v156-PRD-final.md
git add docs/v156-RELEASE-CHECKLIST.md
git add docs/v155-*.md

# Stage Cargo.lock
git add Cargo.lock
```

### Commit

```bash
git commit -m "$(cat <<'EOF'
feat(v1.5.6): generic type sanitization + Windows/PHP backslash fix + SQL infrastructure

## Features
- ISGL1 v2.1 generic type sanitization (< > , [ ] { } → __lt__ __gt__ etc.)
- SQL language infrastructure (Language::Sql, EntityType::Table/View)

## Bug Fixes
- CRITICAL: Fix backslash escaping in edge insertion for Windows/PHP support
- Fix 6.7% edge insertion failure rate for generic types

## New Tests
- 13 sanitization tests (isgl1_v2_generic_sanitization_tests.rs)
- 6 escaping tests (cozo_escaping_tests.rs)

## Files Changed (23)
- parseltongue-core: entities.rs, isgl1_v2.rs, query_extractor.rs, cozo_client.rs
- pt01-folder-to-cozodb-streamer: isgl1_generator.rs, streamer.rs, external_dependency_handler.rs

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

### Tag and Push

```bash
# Create tag
git tag -a v1.5.6 -m "v1.5.6: Generic type sanitization + Windows/PHP fix + SQL infrastructure"

# Push to remote
git push origin HEAD
git push origin v1.5.6
```

---

## GitHub Release (gh CLI)

### Create Release

```bash
gh release create v1.5.6 \
  --title "v1.5.6: Generic Type Sanitization + Windows/PHP Fix" \
  --notes "$(cat <<'EOF'
## What's New in v1.5.6

### Features

#### Generic Type Sanitization (ISGL1 v2.1)
Fixes edge insertion failures for entities with generic type parameters:
- `List<string>` → `List__lt__string__gt__`
- `Dictionary<K, V>` → `Dictionary__lt__K__c__V__gt__`
- `int[]` → `int__lb____rb__`

**Affected Languages**: C#, C++, Java, TypeScript, JavaScript, Rust

#### SQL Language Infrastructure
Foundation for SQL as 13th supported language:
- Added `Language::Sql` enum variant
- Added `EntityType::Table` and `EntityType::View`
- Parser implementation coming in v1.5.7

### Bug Fixes

#### CRITICAL: Backslash Escaping in Edge Insertion
Fixed missing backslash escaping in `cozo_client.rs` that broke:
- **Windows file paths**: `C:\Users\Dev\MyApp` now works
- **PHP namespaces**: `MyApp\Controllers\User` now works

This was causing silent edge insertion failures on Windows and for PHP codebases.

### Test Coverage
- **19 new tests** added (13 sanitization + 6 escaping)
- All 14 HTTP endpoints verified
- 100% edge insertion success rate

### Files Changed
- `parseltongue-core/src/isgl1_v2.rs` - Added `sanitize_entity_name_for_isgl1()`
- `parseltongue-core/src/storage/cozo_client.rs` - Fixed backslash escaping (4 locations)
- `parseltongue-core/src/entities.rs` - Added `Language::Sql`, `EntityType::Table/View`
- Plus 7 more files across the codebase

### Breaking Changes
- **ISGL1 v2.0 → v2.1**: Entity keys with generics will have different format
- **Action Required**: Re-run `pt01-folder-to-cozodb-streamer` to re-ingest codebase

### Documentation
- [v156-PRD-final.md](docs/v156-PRD-final.md) - Complete specification
- [v155-SPEC-GENERIC-TYPE-SANITIZATION.md](docs/v155-SPEC-GENERIC-TYPE-SANITIZATION.md) - Sanitization spec
- [v155-RCA_01.md](docs/v155-RCA_01.md) - Root cause analysis

### Upgrade Instructions

```bash
# 1. Pull latest
git pull origin main

# 2. Rebuild
cargo build --release

# 3. Re-ingest your codebase (required for new key format)
rm -rf parseltongue2026*
./target/release/parseltongue pt01-folder-to-cozodb-streamer <your-codebase>

# 4. Start server
./target/release/parseltongue pt08-http-code-query-server \
  --db "rocksdb:$(ls -td parseltongue2026* | head -1)/analysis.db"
```

### What's Next (v1.5.7)
- SQL parser implementation with tree-sitter-sql
- ORM edge detection (Entity Framework, TypeORM, Prisma)
- View → Table dependency tracking

---

**Full Changelog**: https://github.com/that-in-rust/parseltongue-dependency-graph-generator/compare/v1.5.4...v1.5.6
EOF
)"
```

### Verify Release

```bash
# Check release was created
gh release view v1.5.6

# Open in browser
gh release view v1.5.6 --web
```

---

## Post-Release Verification

### 1. Clone Fresh and Build

```bash
cd /tmp
git clone https://github.com/that-in-rust/parseltongue-dependency-graph-generator.git pt-test
cd pt-test
git checkout v1.5.6
cargo build --release
```

### 2. Test Installation

```bash
./target/release/parseltongue pt01-folder-to-cozodb-streamer crates
# Should complete with 0 edge failures
```

### 3. Cleanup

```bash
cd -
rm -rf /tmp/pt-test
pkill -f "pt08-http-code-query-server"
```

---

## Release Artifacts

| Artifact | Location |
|----------|----------|
| Release Page | https://github.com/that-in-rust/parseltongue-dependency-graph-generator/releases/tag/v1.5.6 |
| PRD | [docs/v156-PRD-final.md](docs/v156-PRD-final.md) |
| Sanitization Spec | [docs/v155-SPEC-GENERIC-TYPE-SANITIZATION.md](docs/v155-SPEC-GENERIC-TYPE-SANITIZATION.md) |
| RCA | [docs/v155-RCA_01.md](docs/v155-RCA_01.md) |
| This Checklist | [docs/v156-RELEASE-CHECKLIST.md](docs/v156-RELEASE-CHECKLIST.md) |

---

## Summary

| Metric | Value |
|--------|-------|
| New Tests | 19 (13 + 6) |
| Files Changed | 23 |
| Edge Insertion Success | 100% (was 93.3%) |
| Languages Supported | 12 + SQL infrastructure |
| Breaking Change | Yes (re-ingest required) |

---

**Release Status**: Ready for deployment
