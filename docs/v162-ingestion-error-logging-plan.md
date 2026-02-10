# Plan: Ingestion Error Logging (v1.6.2)

## Context

pt01 already detects and collects errors during ingestion into `StreamResult.errors: Vec<String>`, but this vector is **never written to disk or shown to the user**. Errors print as yellow warnings to console during streaming, then vanish. Additionally, several error paths silently drop errors via `eprintln!` without collecting them. The user has no way to know which files failed or why after ingestion completes.

**Goal**: Write a detailed `ingestion-errors.txt` next to `analysis.db` in the workspace directory, automatically, at the end of every ingestion run.

## What Already Works (Don't Reinvent)

- `StreamResult.errors: Vec<String>` -- already collects most errors (streamer.rs:44-51)
- `stream_file()` collects per-file errors and joins them (streamer.rs:726-859)
- `stream_directory()` collects all file errors (streamer.rs:486-565)
- Workspace dir created at `main.rs:125`: `parseltongue{TIMESTAMP}/`
- Error types with file paths defined in `errors.rs:8-57`

## What's Broken (Silent Drops to Fix)

| Location | What's lost | Fix |
|----------|------------|-----|
| `isgl1_generator.rs:409` | QueryExtractor parse failure -- `eprintln!` only, not collected | Return error info so `stream_file()` can collect it |
| `isgl1_generator.rs:414` | QueryExtractor lock failure -- `eprintln!` only | Same |
| `streamer.rs:511` | WalkDir traversal errors -- `.filter_map(\|e\| e.ok())` silently drops | Collect walk errors into errors Vec |
| `main.rs:163-168` | `StreamResult.errors` field exists but is NEVER printed or written | Write to file + print count |

## Implementation

### Step 1: Fix silent error drops in `isgl1_generator.rs`

Currently `extract_entities()` drops errors via `eprintln!`. Change the function to return extracted entities AND a list of errors/warnings, so the caller (`stream_file`) can collect them.

**Current** (line ~407-414):
```rust
Err(e) => {
    eprintln!("QueryBasedExtractor failed for {:?}: {}", language, e);
    // silently dropped
}
```

**Change**: Instead of `eprintln!`, push error string to a mutable `&mut Vec<String>` parameter, or return errors alongside entities. The simplest approach: add an `extraction_warnings: &mut Vec<String>` parameter to `extract_entities()`, push error messages there, and collect them in `stream_file()`.

### Step 2: Collect WalkDir errors in `streamer.rs`

**Current** (line ~511):
```rust
.filter_map(|e| e.ok())  // silently drops walk errors
```

**Change**: Replace with explicit match that collects walk errors:
```rust
.filter_map(|entry| match entry {
    Ok(e) => Some(e),
    Err(e) => {
        errors.push(format!("[WALK_ERROR] {}", e));
        None
    }
})
```

This requires restructuring the iterator chain slightly since `errors` needs to be mutable. Alternatively, collect walk errors into a separate `walk_errors` Vec and merge later.

### Step 3: Categorize error messages with prefixes

Standardize error messages throughout `stream_file()` and `stream_directory()` with prefixes:

| Prefix | Meaning | Source |
|--------|---------|--------|
| `[PARSE_ERROR]` | Tree-sitter couldn't parse the file | isgl1_generator.rs parse_source() |
| `[EXTRACT_FAIL]` | Entity extraction failed | isgl1_generator.rs extract_entities() |
| `[CONVERT_FAIL]` | Entity conversion failed | streamer.rs parsed_entity_to_code_entity() |
| `[DB_INSERT]` | Database batch insert failed | streamer.rs insert_entities_batch/insert_edges_batch |
| `[WALK_ERROR]` | Filesystem traversal error | streamer.rs WalkDir |
| `[UNSUPPORTED]` | Unsupported file type | errors.rs UnsupportedFileType |
| `[TOO_LARGE]` | File exceeds size limit | errors.rs FileTooLarge |

Each message includes the file path: `[PARSE_ERROR] src/broken.rs: Failed to parse source code`

### Step 4: Write `ingestion-errors.txt` in `main.rs`

After `stream_directory()` returns, write `StreamResult.errors` to disk. This is the minimal change -- just a few lines added after the existing result handling.

**Location**: `main.rs` after line ~163 where `result` is received.

```rust
// Write ingestion error log
let error_log_path = format!("{}/ingestion-errors.txt", workspace_dir);
let mut error_file = std::fs::File::create(&error_log_path)?;
writeln!(error_file, "# Parseltongue Ingestion Error Log")?;
writeln!(error_file, "# Generated: {}", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"))?;
writeln!(error_file, "# Database: {}", workspace_db_path)?;
writeln!(error_file, "# Source: {}", source_path)?;
writeln!(error_file, "# Total files: {}, Processed: {}, Errors: {}",
    result.total_files, result.processed_files, result.errors.len())?;
writeln!(error_file, "#")?;
for error in &result.errors {
    writeln!(error_file, "{}", error)?;
}
```

Also print error summary to console:
```rust
if !result.errors.is_empty() {
    println!("  Errors: {} (see {})", result.errors.len(), error_log_path);
}
```

**File is ALWAYS written** -- even with 0 errors (header only). This ensures the user can always find it and confirm the run completed.

### Step 5: Also update the parallel path

The parallel streaming function `stream_directory_with_parallel_rayon()` (streamer.rs:570-712) follows the same error collection pattern. Apply the same prefix categorization there. The file writing in main.rs handles both paths since it operates on the returned `StreamResult`.

## Files to Modify

| File | Change |
|------|--------|
| `crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs` | Add `extraction_warnings` param to `extract_entities()`, collect errors instead of `eprintln!` |
| `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs` | Categorize errors with prefixes, collect WalkDir errors, pass warnings vec to extract_entities |
| `crates/parseltongue/src/main.rs` | Write `ingestion-errors.txt` after streaming, print error count with file path |

**No new files needed.** This is a fix to existing error handling, not a new feature.

## Example Output

`parseltongue20260210/ingestion-errors.txt`:
```
# Parseltongue Ingestion Error Log
# Generated: 2026-02-10T14:30:22
# Database: rocksdb:parseltongue20260210/analysis.db
# Source: .
# Total files: 469, Processed: 151, Errors: 12
#
[PARSE_ERROR] test-fixtures/v151-edge-bug-repro/app.js: Failed to parse source code
[PARSE_ERROR] test-fixtures/v151-edge-bug-repro/modules.rb: Failed to parse source code
[EXTRACT_FAIL] test-fixtures/v156-sql-test/schema.sql: QueryBasedExtractor failed for Sql: No parser available
[CONVERT_FAIL] src/legacy.rs: Failed to convert entity rust:fn:old_api: Key generation error
[WALK_ERROR] .git/objects/pack/tmp_pack_abc: Permission denied
[DB_INSERT] Failed to batch insert 3 dependencies: relation conflict
```

Console output change:
```
✓ Indexing completed
  Files processed: 151
  Entities created: 2959
  Errors: 12 (see parseltongue20260210/ingestion-errors.txt)
```

## Verification

1. `cargo build --release` -- builds clean
2. `cargo test -p pt01-folder-to-cozodb-streamer` -- existing tests pass
3. Run ingestion on this repo:
   ```bash
   ./target/release/parseltongue pt01-folder-to-cozodb-streamer .
   ```
4. Verify `parseltongue{TIMESTAMP}/ingestion-errors.txt` exists
5. Verify file contains header with correct counts
6. Verify error messages have category prefixes
7. Verify console shows error count with file path
8. Verify: error count in file matches "Errors encountered: N" from streaming output
9. Run on a clean project with 0 errors -- verify file still created with header only
