# Coverage Reporting Bug Issues

> Discovered: 2026-02-13 via competitor research analysis
> Affected: `/ingestion-coverage-folder-report` endpoint
> Impact: Reports 41% coverage when actual source coverage is ~83%

---

## Issue 1: Test Exclusion Coverage Inflation

**File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_coverage_folder_handler.rs`

**Problem**: `compute_unparsed_files_list()` (line 409-418) marks test files as `[UNPARSED]` errors because it only checks the `CodeGraph` table. Test files are intentionally excluded from `CodeGraph` during pt01 ingestion and stored in the separate `TestEntitiesExcluded` table -- but the coverage handler never queries that table.

**Impact**: 1,110 test files inflating the error count from ~393 (real) to 1,561 (reported). Coverage drops from ~83% to 41%.

**Fix**: Query `TestEntitiesExcluded` table (pattern exists at `ingestion_diagnostics_coverage_handler.rs:357-387`) and subtract those file paths from the unparsed list.

---

## Issue 2: Init Files Unparsed Miscount

**File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_coverage_folder_handler.rs`

**Problem**: 121 `__init__.py` files are counted as eligible (correct `.py` extension) and marked `[UNPARSED]` when they yield zero entities. Empty or trivial files that parse successfully but produce no extractable entities are indistinguishable from actual parse failures.

**Impact**: 121 false positives in the error count.

**Fix**: Add a `[ZERO_ENTITIES]` category for files that parsed successfully but yielded no entities, separate from `[UNPARSED]` which should mean "parser could not process this file."

---

## Issue 3: Path Normalization Comparison Mismatch

**File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_coverage_folder_handler.rs`

**Problem**: The coverage handler walks the filesystem and gets paths like `./crates/foo.rs` (with `./` prefix), but the database stores paths as `crates/foo.rs` (no prefix). The `PathBuf` exact comparison at line 415 fails for some files due to this inconsistency.

**Impact**: Some successfully parsed files may be falsely reported as unparsed due to path mismatch.

**Fix**: Strip `./` prefix from filesystem-walked paths before comparing against database paths, or use `path_utils.rs:42-54` (`normalize_split_file_path`) for consistent normalization.

---

## Issue 4: Error Log Category Separation

**File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_coverage_folder_handler.rs`

**Problem**: The `ingestion-errors.txt` log file uses a single `[UNPARSED]` tag for all files not found in the database. There is no distinction between:
- Files that actually failed to parse (real errors)
- Test files excluded by design (not errors)
- Files that parsed but yielded zero entities (edge case)

**Impact**: The error log is unusable for diagnosing real parse failures because it's flooded with 1,231 false positives (1,110 tests + 121 init files).

**Fix**: Introduce separate tags:
- `[UNPARSED]` -- actual parsing failures (keep as error)
- `[TEST_EXCLUDED]` -- intentionally excluded test files (informational)
- `[ZERO_ENTITIES]` -- parsed but empty (informational)

Add `test_excluded_count` and `zero_entity_count` fields to the JSON response summary.

---

## References

| Component | File | Lines |
|-----------|------|-------|
| Coverage handler (bug location) | `ingestion_coverage_folder_handler.rs` | 119-260, 340-362, 409-418, 501-530 |
| Diagnostics handler (correct pattern) | `ingestion_diagnostics_coverage_handler.rs` | 357-387 |
| Test exclusion in streamer | `streamer.rs` | 979-994, 1109 |
| TestEntitiesExcluded schema | `cozo_client.rs` | 174-207 |
| ExcludedTestEntity struct | `entities.rs` | 1702-1732 |
| Path normalization utility | `path_utils.rs` | 42-54 |
