# Ingestion Error Logging Specification

**Feature**: Persistent error logging for pt01-folder-to-cozodb-streamer
**Version**: 1.0
**Status**: Draft
**Author**: Technical Specification
**Date**: 2026-02-10

---

## 1. Problem Statement

### Current State

The Parseltongue ingestion tool (`pt01-folder-to-cozodb-streamer`) processes source files and extracts code entities. During this process, various errors occur:

- Parse failures (tree-sitter cannot parse malformed code)
- Extraction failures (parser runs but entity extraction fails)
- Unsupported file types (no parser available for language)
- Database insert failures (relation conflicts, constraint violations)
- Filesystem traversal errors (permission denied, broken symlinks)

**Critical Gap**: These errors are detected and collected in `StreamResult.errors: Vec<String>` but are **never persisted to disk**. They print as yellow warnings to stdout during streaming, then vanish when the process exits.

Additionally, several error paths use `eprintln!` without even adding to the error collection, causing silent data loss.

### Impact on Users

1. **No Post-Mortem Diagnostics**: Users cannot review why specific files failed after ingestion completes
2. **Cannot Improve Coverage**: No systematic way to identify patterns in failures (e.g., all SQL files fail, all files in specific directories)
3. **Silent Failures**: Errors printed during long-running ingestion scroll off terminal buffer
4. **No Audit Trail**: Cannot verify if ingestion quality improves between versions
5. **Manual Workaround Required**: Users must redirect stderr to a file manually, which captures all output (not just errors)

### Existing Partial Solution

The HTTP endpoint `/ingestion-coverage-folder-report` writes an `ingestion-errors.txt` file, but it:
- Only lists file names that weren't parsed (no failure reasons)
- Is created after-the-fact via HTTP request (not during ingestion)
- Requires running the HTTP server (not available for pt01-only workflows)
- Uses a simple `[UNPARSED]` categorization without detail

---

## 2. Requirements

### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| **FR-1** | Error log file written automatically during pt01 ingestion | P0 |
| **FR-2** | File placed in workspace directory next to `analysis.db` | P0 |
| **FR-3** | Every error categorized with a prefix tag | P0 |
| **FR-4** | Per-file error detail (file path + failure reason) | P0 |
| **FR-5** | File always created (even with 0 errors) | P1 |
| **FR-6** | Human-readable plain text format | P0 |
| **FR-7** | Header metadata (timestamp, database path, source directory, summary counts) | P1 |
| **FR-8** | Console output shows error count with file reference | P1 |

### Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| **NFR-1** | No performance regression (error collection already occurs) | P0 |
| **NFR-2** | No existing tests broken | P0 |
| **NFR-3** | File size bounded (one line per error, no stack traces) | P1 |
| **NFR-4** | UTF-8 encoding for file paths with non-ASCII characters | P1 |

### Out of Scope (Future Considerations)

- Storing errors in CozoDB relation for HTTP queryability
- `--verbose-errors` CLI flag for detailed console output
- Error log rotation or archival
- Machine-readable formats (JSON, CSV)

---

## 3. Error Categories

Each error line begins with a category prefix in square brackets:

| Category | Prefix | Description | Example Cause |
|----------|--------|-------------|---------------|
| **Parse Error** | `[PARSE_ERROR]` | Tree-sitter could not parse the file | Syntax error, malformed code, language version mismatch |
| **Extraction Failure** | `[EXTRACT_FAIL]` | Parser ran but entity extraction failed | Unexpected AST structure, missing node types |
| **Conversion Failure** | `[CONVERT_FAIL]` | Entity conversion to storage format failed | Invalid UTF-8, path normalization failure |
| **Database Insert** | `[DB_INSERT]` | CozoDB batch insert operation failed | Relation conflict, constraint violation, transaction error |
| **Filesystem Walk** | `[WALK_ERROR]` | Directory traversal error | Permission denied, broken symlink, I/O error |
| **Unsupported Type** | `[UNSUPPORTED]` | File type has no parser | SQL, Markdown, text files without language support |
| **Size Limit** | `[TOO_LARGE]` | File exceeds configured size limit | Files > 1MB (configurable threshold) |

### Category Mapping to Code Paths

```rust
// isgl1_generator.rs
tree_sitter::parse() fails          → [PARSE_ERROR]
extract_entities() fails            → [EXTRACT_FAIL]
convert_to_storage() fails          → [CONVERT_FAIL]

// streamer.rs
batch_insert_dependencies() fails   → [DB_INSERT]
batch_insert_entities() fails       → [DB_INSERT]
WalkDir::next() errors              → [WALK_ERROR]

// Language detection
No parser for detected language     → [UNSUPPORTED]
```

---

## 4. File Format

### File Location

```
{workspace_dir}/ingestion-errors.txt
```

Example: `parseltongue20260210143022/ingestion-errors.txt`

### Structure

```
# Parseltongue Ingestion Error Log
# Generated: {ISO 8601 timestamp}
# Database: {db_path}
# Source: {source_directory}
# Total files: {walk_count}, Processed: {success_count}, Errors: {error_count}
#
{error_lines}
```

### Example (With Errors)

```
# Parseltongue Ingestion Error Log
# Generated: 2026-02-10T14:30:22Z
# Database: rocksdb:parseltongue20260210143022/analysis.db
# Source: /Users/amuldotexe/projects/my-app
# Total files: 469, Processed: 151, Errors: 12
#
[PARSE_ERROR] test-fixtures/broken.js: Failed to parse source code
[PARSE_ERROR] test-fixtures/syntax-error.py: Failed to parse source code
[EXTRACT_FAIL] src/legacy/old_api.rs: Entity extraction failed for Rust file
[UNSUPPORTED] schema.sql: No parser available for language: Sql
[UNSUPPORTED] README.md: No parser available for language: Markdown
[WALK_ERROR] .git/objects/pack/tmp_pack_abc123: Permission denied (os error 13)
[WALK_ERROR] node_modules/broken-symlink: Broken symbolic link
[DB_INSERT] Failed to batch insert 3 dependencies: relation conflict on primary key
[CONVERT_FAIL] src/unicode-test/文件.rs: Path normalization failed
[TOO_LARGE] vendor/generated.proto: File size 2.3MB exceeds limit of 1.0MB
[PARSE_ERROR] test-fixtures/incomplete.cpp: Unexpected EOF during parsing
[EXTRACT_FAIL] src/generated/bindings.go: No struct/function declarations found
```

### Example (Zero Errors)

```
# Parseltongue Ingestion Error Log
# Generated: 2026-02-10T14:35:10Z
# Database: rocksdb:parseltongue20260210143510/analysis.db
# Source: /Users/amuldotexe/projects/clean-repo
# Total files: 84, Processed: 84, Errors: 0
#
# No errors encountered during ingestion.
```

### Format Rules

1. **Header lines** prefixed with `#` (shell-style comments)
2. **One error per line** (no multi-line stack traces)
3. **Error format**: `[CATEGORY] {file_path}: {error_message}`
4. **File paths** relative to source directory (if possible) or absolute
5. **Error messages** concise (1-2 sentences max)
6. **Always ends with newline** (POSIX text file standard)

---

## 5. Implementation Approach

### Files to Modify

#### 5.1 `crates/pt01-folder-to-cozodb-streamer/src/isgl1_generator.rs`

**Problem**: Silent error drops via `eprintln!` without adding to error collection.

**Changes**:
1. Add `extraction_warnings: &mut Vec<String>` parameter to all functions that currently use `eprintln!`
2. Replace `eprintln!(...)` with:
   ```rust
   let error_msg = format!("[PARSE_ERROR] {}: {}", file_path, error);
   extraction_warnings.push(error_msg);
   ```
3. Thread `extraction_warnings` through call stack from `process_file_to_entities_sync()`

**Affected Functions**:
- `parse_source_file_to_tree()` - parse failures
- `extract_entities_from_tree()` - extraction failures
- `convert_entity_to_storage()` - conversion failures

#### 5.2 `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs`

**Problem**: Errors collected but not categorized; WalkDir errors not captured.

**Changes**:
1. **Categorize existing errors** with prefix tags when adding to `errors` vec:
   ```rust
   // Before
   errors.push(format!("Failed to process {}: {}", path, err));

   // After
   errors.push(format!("[EXTRACT_FAIL] {}: {}", path, err));
   ```

2. **Capture WalkDir errors**:
   ```rust
   for entry_result in WalkDir::new(source_path) {
       match entry_result {
           Ok(entry) => { /* process */ },
           Err(err) => {
               errors.push(format!("[WALK_ERROR] {}: {}", err.path().display(), err));
               continue;
           }
       }
   }
   ```

3. **Categorize database errors**:
   ```rust
   if let Err(e) = batch_insert_entities(...) {
       errors.push(format!("[DB_INSERT] Failed to insert entities: {}", e));
   }
   ```

4. **Pass `extraction_warnings` from `isgl1_generator`** into `StreamResult.errors`

#### 5.3 `crates/parseltongue/src/main.rs`

**Problem**: No persistence of errors after streaming completes.

**Changes**:
1. **After streaming completes**, extract workspace directory from `db_path`
2. **Write error log file**:
   ```rust
   let workspace_dir = extract_workspace_dir_from_db_path(&db_path);
   let error_log_path = workspace_dir.join("ingestion-errors.txt");
   write_ingestion_error_log(
       &error_log_path,
       &stream_result.errors,
       &db_path,
       source_path,
       total_files,
       success_count,
   )?;
   ```

3. **Update console output** to reference error log:
   ```rust
   if stream_result.errors.is_empty() {
       println!("✓ Indexing completed (0 errors)");
   } else {
       println!("✓ Indexing completed");
       println!("  Files processed: {}", success_count);
       println!("  Entities created: {}", entity_count);
       println!("  Errors: {} (see {}/ingestion-errors.txt)",
                stream_result.errors.len(),
                workspace_name);
   }
   ```

### Helper Functions

Add to `crates/parseltongue/src/main.rs`:

```rust
fn extract_workspace_dir_from_db_path(db_path: &str) -> PathBuf {
    // "rocksdb:parseltongue20260210/analysis.db" → "parseltongue20260210"
    let path = db_path.strip_prefix("rocksdb:").unwrap_or(db_path);
    PathBuf::from(path).parent().unwrap().to_path_buf()
}

fn write_ingestion_error_log(
    log_path: &Path,
    errors: &[String],
    db_path: &str,
    source_dir: &str,
    total_files: usize,
    processed_files: usize,
) -> anyhow::Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(log_path)?;

    // Header
    writeln!(file, "# Parseltongue Ingestion Error Log")?;
    writeln!(file, "# Generated: {}", chrono::Utc::now().to_rfc3339())?;
    writeln!(file, "# Database: {}", db_path)?;
    writeln!(file, "# Source: {}", source_dir)?;
    writeln!(file, "# Total files: {}, Processed: {}, Errors: {}",
             total_files, processed_files, errors.len())?;
    writeln!(file, "#")?;

    // Errors (or placeholder)
    if errors.is_empty() {
        writeln!(file, "# No errors encountered during ingestion.")?;
    } else {
        for error in errors {
            writeln!(file, "{}", error)?;
        }
    }

    Ok(())
}
```

---

## 6. Console Output Change

### Before

```
✓ Indexing completed
  Workspace: parseltongue20260210143022
  Database: rocksdb:parseltongue20260210143022/analysis.db
  Files processed: 151
  Entities created: 2959
  Dependencies found: 843
```

### After (With Errors)

```
✓ Indexing completed
  Workspace: parseltongue20260210143022
  Database: rocksdb:parseltongue20260210143022/analysis.db
  Files processed: 151
  Entities created: 2959
  Dependencies found: 843
  Errors: 12 (see parseltongue20260210143022/ingestion-errors.txt)
```

### After (Zero Errors)

```
✓ Indexing completed (0 errors)
  Workspace: parseltongue20260210143022
  Database: rocksdb:parseltongue20260210143022/analysis.db
  Files processed: 151
  Entities created: 2959
  Dependencies found: 843
```

---

## 7. Integration with Coverage Endpoint

### Current State

The HTTP endpoint `/ingestion-coverage-folder-report` currently:
1. Walks the source directory
2. Identifies files that should be parseable but aren't in the database
3. Writes `{workspace}/ingestion-errors.txt` with `[UNPARSED]` entries

### Conflict Resolution

The pt01 error log and coverage endpoint both want to write `ingestion-errors.txt` to the same location.

### Proposed Solution: Append Model

**Option A: Coverage Endpoint Appends** (Recommended)

1. pt01 writes error log during ingestion (as specified)
2. Coverage endpoint **appends** its findings if the file exists:
   ```rust
   let mut file = OpenOptions::new()
       .create(true)
       .append(true)
       .open(log_path)?;

   writeln!(file, "\n# Coverage Analysis Findings")?;
   writeln!(file, "# Generated: {}", timestamp)?;
   for unparsed in unparsed_files {
       writeln!(file, "[UNPARSED] {}: File not found in database", unparsed)?;
   }
   ```

**Option B: Separate Files**

- pt01 writes: `ingestion-errors.txt`
- Coverage writes: `ingestion-coverage.txt`

**Recommendation**: Option A (append model) provides a unified error log while keeping concerns separated by section headers.

### Coverage Endpoint Changes

Update `crates/pt08-http-code-query-server/src/endpoints.rs`:

```rust
// In generate_ingestion_coverage_report_file()

// Check if pt01 error log exists
let log_path = workspace_dir.join("ingestion-errors.txt");
let mut file = if log_path.exists() {
    // Append to existing log
    OpenOptions::new().append(true).open(&log_path)?
} else {
    // Create new log with header
    let mut f = File::create(&log_path)?;
    writeln!(f, "# Parseltongue Ingestion Error Log")?;
    writeln!(f, "# Generated: {}", chrono::Utc::now().to_rfc3339())?;
    writeln!(f, "#")?;
    f
};

// Write coverage findings
writeln!(file, "\n# Coverage Analysis")?;
writeln!(file, "# Generated: {}", chrono::Utc::now().to_rfc3339())?;
for unparsed in unparsed_files {
    writeln!(file, "[UNPARSED] {}: File not found in database", unparsed)?;
}
```

---

## 8. Acceptance Criteria

### Must Have (P0)

- [ ] **AC-1**: After `pt01-folder-to-cozodb-streamer` completes, `{workspace}/ingestion-errors.txt` exists
- [ ] **AC-2**: File contains header with timestamp, database path, source directory, and summary counts
- [ ] **AC-3**: Every error has a category prefix (`[PARSE_ERROR]`, `[DB_INSERT]`, etc.)
- [ ] **AC-4**: Error count in file matches console output
- [ ] **AC-5**: Zero-error runs create the file with "No errors" message
- [ ] **AC-6**: All existing tests pass (`cargo test --all`)
- [ ] **AC-7**: No performance regression (error collection overhead < 1% of total time)

### Should Have (P1)

- [ ] **AC-8**: Console output shows error count with file path reference
- [ ] **AC-9**: File paths are relative to source directory when possible
- [ ] **AC-10**: Coverage endpoint appends to existing error log (not overwrites)
- [ ] **AC-11**: UTF-8 file paths render correctly (tested with non-ASCII characters)

### Test Cases

| Test ID | Scenario | Expected Outcome |
|---------|----------|------------------|
| **TC-1** | Ingest clean repo (zero errors) | Log file created with "No errors" message |
| **TC-2** | Ingest with parse errors | Log contains `[PARSE_ERROR]` entries |
| **TC-3** | Ingest with unsupported files | Log contains `[UNSUPPORTED]` entries |
| **TC-4** | Ingest with permission errors | Log contains `[WALK_ERROR]` entries |
| **TC-5** | Ingest with DB insert failures | Log contains `[DB_INSERT]` entries |
| **TC-6** | Run coverage endpoint after pt01 | Appends `[UNPARSED]` entries without overwriting |
| **TC-7** | Console output validation | Error count matches file, path printed correctly |

### Performance Benchmarks

Run on reference codebase (Parseltongue itself, ~150 files):

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Ingestion time | 250ms | < 260ms | < 4% increase |
| Memory usage | ~40MB | ~40MB | No change |
| File I/O | 1 write (DB) | 2 writes (DB + log) | +1 file |

---

## 9. Future Considerations

### Phase 2: Database Storage

Store errors in a CozoDB relation for queryability:

```datalog
?[error_id, category, file_path, message, timestamp] <- [[...]]
```

Benefits:
- HTTP endpoint: `GET /ingestion-errors-query?category=PARSE_ERROR`
- Filter by category, file pattern, timestamp
- Aggregate statistics (most common error types)

### Phase 3: Verbose Console Mode

Add `--verbose-errors` flag to print full error detail during streaming:

```bash
parseltongue pt01-folder-to-cozodb-streamer . --verbose-errors
```

Behavior:
- Print each error immediately (not just at end)
- Include stack traces for unexpected errors
- Color-coded by severity (red = critical, yellow = warning)

### Phase 4: Error Recovery

Add `--continue-on-error` flag with configurable thresholds:

```bash
parseltongue pt01-folder-to-cozodb-streamer . --continue-on-error --max-errors 100
```

Behavior:
- Stop ingestion if error count exceeds threshold
- Prevents runaway failures (e.g., broken tree-sitter grammar)
- Write partial database + error log

### Phase 5: Error Analytics

Add `pt-error-analysis` tool to analyze error logs:

```bash
parseltongue pt-error-analysis parseltongue20260210/ingestion-errors.txt

# Output:
# Error Summary:
#   [PARSE_ERROR]: 45 files
#   [UNSUPPORTED]: 23 files
#   [WALK_ERROR]: 2 files
#
# Top Failed Directories:
#   test-fixtures/: 30 errors
#   vendor/: 15 errors
```

---

## 10. Implementation Checklist

### Phase 1: Core Implementation

- [ ] Modify `isgl1_generator.rs`: Add `extraction_warnings` parameter
- [ ] Modify `streamer.rs`: Categorize errors, capture WalkDir errors
- [ ] Modify `main.rs`: Write error log file after streaming
- [ ] Add `extract_workspace_dir_from_db_path()` helper
- [ ] Add `write_ingestion_error_log()` helper
- [ ] Update console output to show error count
- [ ] Test on clean repository (zero errors)
- [ ] Test on repository with known errors

### Phase 2: Coverage Integration

- [ ] Update coverage endpoint to append (not overwrite)
- [ ] Add section header for coverage findings
- [ ] Test coverage endpoint after pt01 run

### Phase 3: Testing & Validation

- [ ] Run full test suite (`cargo test --all`)
- [ ] Manual test: Ingest Parseltongue repository
- [ ] Manual test: Ingest test-fixtures directory
- [ ] Manual test: Verify UTF-8 path handling
- [ ] Performance benchmark (compare before/after)
- [ ] Update documentation (CLAUDE.md, README.md)

### Phase 4: Release

- [ ] Code review
- [ ] Merge to main branch
- [ ] Tag release version
- [ ] Update changelog

---

## 11. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| **File path conflicts** (coverage endpoint) | High | Medium | Implement append model with section headers |
| **Performance regression** | Low | Medium | Error collection already occurs; only adds file write |
| **UTF-8 encoding issues** | Medium | Low | Use `std::fs::write()` with explicit UTF-8 |
| **Disk space concerns** | Low | Low | Log size bounded by file count (1 line per error) |
| **Breaking existing workflows** | Low | High | Always create file (even with 0 errors) for consistency |

---

## 12. Dependencies

### External Crates

- `chrono` - Already in dependencies (ISO 8601 timestamp formatting)
- No new dependencies required

### Internal Dependencies

- `parseltongue-core` - Error types, storage traits
- `pt01-folder-to-cozodb-streamer` - Streaming logic
- `pt08-http-code-query-server` - Coverage endpoint integration

---

## 13. Success Metrics

### Quantitative

- **Error log file created**: 100% of pt01 runs
- **Category coverage**: All 7 error types used in practice
- **Performance overhead**: < 5% increase in ingestion time
- **User adoption**: Error log referenced in bug reports

### Qualitative

- Users can diagnose ingestion failures without re-running
- Bug reports include error log excerpts (improved debuggability)
- Coverage improvements trackable over time (version N vs N+1)

---

## Appendix A: Error Message Guidelines

### Do's

- **Be specific**: "Failed to parse source code" → "Unexpected token '}' at line 42"
- **Include context**: File path, line number (if available)
- **Use category prefixes**: Enables filtering and grouping
- **Keep concise**: One line per error (no stack traces)

### Don'ts

- **Avoid generic messages**: "An error occurred"
- **No user-facing jargon**: "AST node type mismatch" → "Entity extraction failed"
- **No sensitive data**: Absolute paths may leak usernames (use relative paths)
- **No redundancy**: Category prefix implies context (don't repeat in message)

### Examples

```
✅ [PARSE_ERROR] src/main.rs: Unexpected token '}' at line 42
❌ [PARSE_ERROR] /Users/alice/projects/app/src/main.rs: Tree-sitter parse error: unexpected node type TSNodeKindRBrace at byte offset 1234

✅ [UNSUPPORTED] schema.sql: No parser available for language: Sql
❌ [ERROR] schema.sql: Unsupported file type detected by language detector module

✅ [WALK_ERROR] node_modules/.bin/symlink: Broken symbolic link
❌ [WALK_ERROR] /Users/alice/projects/app/node_modules/.bin/symlink: std::io::Error: os error 2: No such file or directory (kind: NotFound)
```

---

## Appendix B: Reference Implementation Pseudocode

### Error Log Writer

```rust
struct IngestionErrorLog {
    timestamp: DateTime<Utc>,
    db_path: String,
    source_dir: String,
    total_files: usize,
    processed_files: usize,
    errors: Vec<CategorizedError>,
}

struct CategorizedError {
    category: ErrorCategory,
    file_path: String,
    message: String,
}

enum ErrorCategory {
    ParseError,       // [PARSE_ERROR]
    ExtractFail,      // [EXTRACT_FAIL]
    ConvertFail,      // [CONVERT_FAIL]
    DbInsert,         // [DB_INSERT]
    WalkError,        // [WALK_ERROR]
    Unsupported,      // [UNSUPPORTED]
    TooLarge,         // [TOO_LARGE]
}

impl IngestionErrorLog {
    fn write_to_file(&self, path: &Path) -> anyhow::Result<()> {
        let mut file = File::create(path)?;

        // Header
        writeln!(file, "# Parseltongue Ingestion Error Log")?;
        writeln!(file, "# Generated: {}", self.timestamp.to_rfc3339())?;
        writeln!(file, "# Database: {}", self.db_path)?;
        writeln!(file, "# Source: {}", self.source_dir)?;
        writeln!(file, "# Total files: {}, Processed: {}, Errors: {}",
                 self.total_files, self.processed_files, self.errors.len())?;
        writeln!(file, "#")?;

        // Errors
        if self.errors.is_empty() {
            writeln!(file, "# No errors encountered during ingestion.")?;
        } else {
            for error in &self.errors {
                writeln!(file, "[{}] {}: {}",
                         error.category.as_str(),
                         error.file_path,
                         error.message)?;
            }
        }

        Ok(())
    }
}

impl ErrorCategory {
    fn as_str(&self) -> &'static str {
        match self {
            Self::ParseError => "PARSE_ERROR",
            Self::ExtractFail => "EXTRACT_FAIL",
            Self::ConvertFail => "CONVERT_FAIL",
            Self::DbInsert => "DB_INSERT",
            Self::WalkError => "WALK_ERROR",
            Self::Unsupported => "UNSUPPORTED",
            Self::TooLarge => "TOO_LARGE",
        }
    }
}
```

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-02-10 | Technical Specification | Initial draft |

---

**End of Specification**
