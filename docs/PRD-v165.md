# PRD: Ingestion Diagnostics Coverage Report (v1.6.5)

**Feature**: Ingestion Quality Diagnostics API
**Version**: v1.6.5
**Status**: Draft
**Date**: 2026-02-08
**Author**: Parseltongue Team

---

## Table of Contents

1. [Problem Statement](#1-problem-statement)
2. [Why This Matters for LLM Agents](#2-why-this-matters-for-llm-agents)
3. [Prior Art and Research](#3-prior-art-and-research)
4. [Three Diagnostic Reports](#4-three-diagnostic-reports)
5. [Cross-OS Path Normalization](#5-cross-os-path-normalization)
6. [CozoDB Relation Schemas](#6-cozodb-relation-schemas)
7. [API Design](#7-api-design)
8. [Integration with Existing Pipeline](#8-integration-with-existing-pipeline)
9. [Implementation Phases](#9-implementation-phases)
10. [Acceptance Criteria](#10-acceptance-criteria)
11. [Open Questions](#11-open-questions)

---

## 1. Problem Statement

### The Gap: "What Did Parseltongue Miss?"

After `pt01-folder-to-cozodb-streamer` ingests a codebase, the user sees aggregate statistics:

```
Parallel Streaming Summary:
Total files found: 847
Files processed: 312
Entities created: 2,841 (CODE only)
  - TEST entities: 109 (excluded for optimal LLM context)
Errors encountered: 3
Duration: 1.24s
```

Three questions remain unanswered:

1. **Which files were ignored?** 535 files were skipped -- but which ones and why? Were they all `.json`/`.md` non-code files, or did we miss `.vue`/`.svelte` files that contain real logic?
2. **What test entities were excluded?** 109 test entities were dropped. Which ones? An LLM agent debugging a test failure has zero visibility into what was intentionally excluded.
3. **Did we capture all the code in each parsed file?** A file has 2,000 words but the extracted entities only contain 800 words total. That means 60% of the file content is invisible to the graph. Is the gap imports/comments (expected), or missed function bodies (a real problem)?

### Current State

- **Ignored files**: Silently dropped in `should_process_file()` (`streamer.rs:366-389`). No record kept.
- **Test entities**: Aggregate count in `StreamStats.test_entities_created` (`streamer.rs:68`). Individual entity names/paths are discarded at `streamer.rs:803-807` with `test_count += 1; continue;`.
- **Parse completeness**: No word-count comparison exists. The existing `/ingestion-coverage-folder-report` endpoint counts files-parsed vs files-on-disk, but says nothing about whether the *content* of parsed files was fully captured.

### What This Feature Delivers

A single HTTP endpoint (`GET /ingestion-diagnostics-coverage-report`) that returns all three reports in one response. The data is collected during pt01 ingestion and stored in CozoDB, then queried at endpoint time.

---

## 2. Why This Matters for LLM Agents

Parseltongue's core user is an LLM agent (Claude, GPT, Codex) that queries the code graph to understand a codebase. These agents need to **know what they don't know**:

| Scenario | Without v1.6.5 | With v1.6.5 |
|----------|----------------|-------------|
| Agent debugging a Svelte app | Silently sees zero `.svelte` entities. Assumes there's no frontend. | Sees `.svelte` in ignored files list. Reports: "Parseltongue doesn't parse Svelte yet. 47 `.svelte` files are invisible." |
| Agent investigating test failures | No test entities in the graph. Can't correlate test names to production code. | Queries excluded test entities. Finds `test_payment_flow` was excluded. Reads the file directly for context. |
| Agent reviewing a 500-line file | Sees 3 entities extracted. No way to know if 200 lines or 400 lines were captured. | Sees word count coverage is 45%. Knows over half the file was missed (likely utility functions or nested closures). |

The endpoint enables **self-aware querying** -- agents can programmatically adjust their strategy based on coverage gaps.

---

## 3. Prior Art and Research

### Industry Tools

| Tool | What It Measures | Delivery Mechanism |
|------|-----------------|-------------------|
| **Semgrep** | Tree-sitter parse rate (ERROR nodes / total bytes) | Dashboard metrics, `--metrics` flag |
| **CodeQL** | Database completeness diagnostics | `database export-diagnostics` (SARIF file) |
| **SonarQube** | Lines of code analyzed vs total | `/api/measures` REST endpoint |
| **Sourcegraph** | SCIP indexer coverage stats | Internal telemetry |

**Key finding**: No tool exposes ingestion diagnostics as a user-facing REST endpoint queryable by LLM agents. This is a differentiator for Parseltongue's LLM-agent-first architecture.

### Why Raw Word Count (Not Line Coverage)

Research suggested line coverage or tree-sitter ERROR node counting. The user chose **raw word count** for the following reason:

> "Parse entities as an end-to-end thing -- we do not worry about movement in the page."

Line coverage penalizes blank lines, comments, and import statements -- all of which are legitimately outside entity boundaries. Word count provides a more intuitive "how much content did we capture?" metric:

- `source_word_count`: `source.split_whitespace().count()` on the raw file
- `entity_word_count`: Sum of word counts from all extracted entities' content fields
- `raw_coverage_pct`: `(entity_word_count / source_word_count) * 100`

A file with 72% raw word coverage means 28% of the file's words are outside any extracted entity. But not all of that 28% is a problem -- imports and comments are *expected* to fall outside entity boundaries.

### Dual Coverage Metrics: Raw vs Effective

**The problem with a single coverage number**: A file with 10 import statements and 50 comment lines will always show low raw coverage even if every function body is perfectly captured. This makes it impossible to distinguish "expected gaps" from "real gaps."

**Solution**: Two metrics per file.

| Metric | Formula | What It Measures |
|--------|---------|-----------------|
| `raw_coverage_pct` | `entity_words / source_words × 100` | Total extraction ratio (includes expected gaps) |
| `effective_coverage_pct` | `entity_words / (source_words - import_words - comment_words) × 100` | Extraction ratio of *meaningful* code only |

**Why this works**:
- A file at **72% raw / 96% effective** → healthy. The 28% gap is imports and comments.
- A file at **72% raw / 73% effective** → problem. Almost no imports/comments, so the 28% gap is real missed code.
- A file at **30% raw / 45% effective** → bad. Even after subtracting imports/comments, over half the code is missing.

**The key insight**: `effective_coverage_pct` should approach 90-100% for well-parsed files. If it's below 80%, the parser is missing real code constructs.

### Import Word Counting (Zero New Queries)

Parseltongue's dependency query system (`query_extractor.rs`) already matches import/use/require/include nodes in all 12 languages via tree-sitter. The import byte ranges are already traversed during `execute_dependency_query()` -- we just need to accumulate the word count from those ranges.

**Existing dependency capture patterns by language**:

| Language | Capture Names |
|----------|--------------|
| Rust | `@dependency.use`, `@dependency.use_external` |
| Python | `@dependency.import`, `@dependency.import_from` |
| JavaScript/TypeScript | `@dependency.import`, `@dependency.require` |
| Go | `@dependency.import` |
| Java | `@dependency.import` |
| C/C++ | `@dependency.include` |
| Ruby | `@dependency.require` |
| PHP | `@dependency.use`, `@dependency.require` |
| C# | `@dependency.using` |
| Swift | `@dependency.import` |

**Implementation**: During `execute_dependency_query()`, when a capture name starts with `@dependency`, accumulate the node's byte range. After the loop, compute `import_word_count` from deduplicated byte ranges:

```rust
import_word_count = deduplicated_import_ranges
    .iter()
    .map(|(start, end)| source[*start..*end].split_whitespace().count())
    .sum()
```

Byte range deduplication handles overlapping captures (e.g., `use std::collections::{HashMap, HashSet}` matched by both `@dependency.use` and a sub-pattern).

### Comment Word Counting (AST Root Walk)

After tree-sitter parses a file, walk `tree.root_node()` children to find comment nodes. Each language has specific comment node types:

| Language | Comment Node Types |
|----------|--------------------|
| Rust | `line_comment`, `block_comment` |
| Python | `comment` |
| JavaScript/TypeScript | `comment` |
| Go | `comment` |
| Java | `line_comment`, `block_comment` |
| C/C++ | `comment` |
| Ruby | `comment` |
| PHP | `comment` |
| C# | `comment` |
| Swift | `comment`, `multiline_comment` |

**Implementation**: Walk all descendants of the root node (not just direct children, since comments can be nested inside blocks). For each node whose `kind()` matches a comment type, sum word counts:

```rust
let mut comment_word_count = 0;
let mut cursor = tree.root_node().walk();
// Depth-first traversal
loop {
    let node = cursor.node();
    if is_comment_node(node.kind(), language) {
        comment_word_count += source[node.byte_range()].split_whitespace().count();
    }
    if !cursor.goto_next_sibling() && !cursor.goto_parent() {
        break;
    }
}
```

**Note**: Top-level comments only are counted (comments inside function bodies are already included in `entity_word_count` since the entity content includes the full function body with its internal comments).

---

## 4. Three Diagnostic Reports

### Report 1: Ignored Files by Extension

**What**: Files in the source directory that were skipped because `Language::from_file_path()` returned `None`.

**Data source**: Filesystem walk at query time (same approach as existing `/ingestion-coverage-folder-report`).

**Logic**:
1. Walk the source directory recursively (same WalkDir as pt01)
2. For each file, call `Language::from_file_path(&path)`
3. Files where this returns `None` → ignored
4. Group by file extension, return counts and full file list

**What it reveals**:
- Expected ignores: `.md`, `.json`, `.toml`, `.yaml`, `.lock`, `.gitignore`
- Actionable ignores: `.vue`, `.svelte`, `.graphql`, `.proto` -- file types that contain real logic but are not yet supported

### Report 2: Excluded Test Entities

**What**: Individual test functions/classes that were detected by `TestDetector` and intentionally excluded from the code graph.

**Data source**: New CozoDB relation `TestEntitiesExcluded`, populated during pt01 ingestion.

**Current code** (`streamer.rs:803-807`):
```rust
if matches!(entity_class, EntityClass::TestImplementation) {
    test_count += 1;
    continue; // Don't insert tests into database
}
```

**Change**: Before `continue`, insert entity details into `TestEntitiesExcluded`.

**What it reveals**:
- Which tests exist in the codebase (by name, file, language)
- How many tests per folder/language
- Enables LLM agents to find test entities when debugging test failures

### Report 3: Word Count Coverage Comparison (Dual Metrics)

**What**: Per-file comparison of source word count vs. extracted entity word count, with both raw and effective coverage percentages.

**Data source**: New CozoDB relation `FileWordCoverage`, populated during pt01 ingestion.

**Logic** (during pt01 ingestion, per file):
1. After reading file content: `source_word_count = source.split_whitespace().count()`
2. After extracting entities: `entity_word_count = sum of entity.content.split_whitespace().count()` for all entities in that file
3. During dependency query execution: `import_word_count` accumulated from `@dependency.*` capture byte ranges
4. After tree-sitter parse: `comment_word_count` accumulated from top-level comment node byte ranges
5. Compute dual metrics:
   ```
   raw_coverage_pct = (entity_word_count / source_word_count) × 100
   effective_source = source_word_count - import_word_count - comment_word_count
   effective_coverage_pct = (entity_word_count / effective_source) × 100
   ```
6. Store all fields in `FileWordCoverage` relation

**What it reveals**:
- **Raw coverage < 50%**: Likely has missed constructs OR is import/comment-heavy
- **Effective coverage < 80%**: Real code is being missed (imports/comments already excluded)
- **Raw ≈ 72%, Effective ≈ 96%**: Healthy file -- the gap is expected (imports + comments)
- **Raw ≈ 72%, Effective ≈ 73%**: Problem -- almost no imports/comments, so the gap is real missed code
- **Coverage > 100%**: Overlapping entity content (e.g., nested functions counted in both parent and child)
- The **delta** between raw and effective tells you how import/comment-heavy a file is

---

## 5. Cross-OS Path Normalization

### Problem

Parseltongue runs on macOS, Linux, and Windows. File paths differ:
- macOS/Linux: `src/core/parser.rs`
- Windows: `src\core\parser.rs`

Rust's `std::path::Path` is platform-bound -- a `PathBuf` created on Windows contains backslashes that won't be recognized on Linux. If paths are stored as-is, CozoDB queries break across platforms.

### Industry Standard

All major code analysis tools normalize paths to forward slashes:

| Tool | Approach |
|------|----------|
| **Git** | Always `/` in index, config, `.gitignore` |
| **SCIP** (Sourcegraph) | `relative_path` field always uses `/` |
| **SonarQube** | `component.path` normalized to `/` |
| **Semgrep** | POSIX paths in all output formats |
| **CodeQL** | Forward slashes in database URI scheme |

### Design Decision

1. **Normalize all stored paths to forward slashes** using the `path-slash` crate
2. **Store relative paths only** (strip the workspace root directory)
3. **Split into `folder_path` + `filename`** at the last `/` separator
4. **Root-level files**: `folder_path = ""`, `filename = "Cargo.toml"`

**Implementation**:
```rust
use path_slash::PathExt; // Add to Cargo.toml: path-slash = "0.2"

fn normalize_split_file_path(abs_path: &Path, workspace_root: &Path) -> (String, String) {
    let relative = abs_path.strip_prefix(workspace_root).unwrap_or(abs_path);
    let normalized = relative.to_slash_lossy().to_string();
    match normalized.rsplit_once('/') {
        Some((folder, file)) => (folder.to_string(), file.to_string()),
        None => (String::new(), normalized), // Root-level file
    }
}
```

**Why `folder_path` + `filename` as separate columns**:
- CozoDB queries can `GROUP BY folder_path` without string splitting
- Filter by filename pattern without scanning full paths
- Enables folder-level aggregation in the coverage report

---

## 6. CozoDB Relation Schemas

### Existing Relations (unchanged)

```
CodeGraph {
    ISGL1_key: String =>
    Current_Code: String?, interface_signature: String,
    TDD_Classification: String, file_path: String,
    language: String, entity_type: String, entity_class: String,
    ...
}

DependencyEdges {
    from_key: String, to_key: String, edge_type: String =>
    source_location: String?
}
```

### New Relation: `TestEntitiesExcluded`

```
:create TestEntitiesExcluded {
    entity_name: String,
    folder_path: String,
    filename: String
    =>
    entity_class: String,
    language: String,
    line_start: Int,
    line_end: Int,
    detection_reason: String
}
```

**Composite key**: `(entity_name, folder_path, filename)` -- unique test entity per file.

**Column details**:

| Column | Type | Description |
|--------|------|-------------|
| `entity_name` | String | e.g., `test_parse_rust_function` |
| `folder_path` | String | e.g., `crates/parseltongue-core/src` (forward slashes, relative) |
| `filename` | String | e.g., `parser_tests.rs` |
| `entity_class` | String | Always `TestImplementation` (for now) |
| `language` | String | e.g., `rust`, `python`, `javascript` |
| `line_start` | Int | Start line number |
| `line_end` | Int | End line number |
| `detection_reason` | String | e.g., `test_prefix`, `test_attribute`, `test_decorator` |

### New Relation: `FileWordCoverage`

```
:create FileWordCoverage {
    folder_path: String,
    filename: String
    =>
    language: String,
    source_word_count: Int,
    entity_word_count: Int,
    import_word_count: Int,
    comment_word_count: Int,
    raw_coverage_pct: Float,
    effective_coverage_pct: Float,
    entity_count: Int
}
```

**Composite key**: `(folder_path, filename)` -- one row per parsed file.

**Column details**:

| Column | Type | Description |
|--------|------|-------------|
| `folder_path` | String | e.g., `src/core` (forward slashes, relative) |
| `filename` | String | e.g., `parser.rs` |
| `language` | String | e.g., `rust` |
| `source_word_count` | Int | `source.split_whitespace().count()` on raw file |
| `entity_word_count` | Int | Sum of word counts from all extracted entities |
| `import_word_count` | Int | Word count from import/use/require/include statements (from dependency query captures) |
| `comment_word_count` | Int | Word count from top-level comment nodes (from AST walk) |
| `raw_coverage_pct` | Float | `(entity_word_count / source_word_count) * 100.0` |
| `effective_coverage_pct` | Float | `(entity_word_count / (source_word_count - import_word_count - comment_word_count)) * 100.0` |
| `entity_count` | Int | Number of code entities extracted from this file |

---

## 7. API Design

### Endpoint

```
GET /ingestion-diagnostics-coverage-report
```

Follows the 4-word naming convention: `ingestion-diagnostics-coverage-report`.

### Response Shape

```json
{
  "success": true,
  "endpoint": "/ingestion-diagnostics-coverage-report",
  "data": {
    "summary": {
      "total_files_discovered": 847,
      "files_parsed": 312,
      "files_ignored_by_extension": 535,
      "entities_extracted": 2841,
      "test_entities_excluded": 109,
      "avg_raw_coverage_pct": 72.3,
      "avg_effective_coverage_pct": 91.7
    },
    "ignored_files": {
      "by_extension": {
        ".md": 45,
        ".json": 38,
        ".toml": 12,
        ".yaml": 9,
        ".lock": 3,
        ".txt": 8,
        ".png": 15,
        ".svg": 7
      },
      "total_count": 535,
      "files": [
        "README.md",
        "package.json",
        "Cargo.lock",
        "docs/architecture.md"
      ]
    },
    "excluded_test_entities": {
      "total_count": 109,
      "by_language": {
        "rust": 67,
        "python": 23,
        "javascript": 19
      },
      "by_folder": {
        "crates/parseltongue-core/src": 34,
        "tests": 28,
        "crates/pt01-folder-to-cozodb-streamer/src": 15
      },
      "entities": [
        {
          "entity_name": "test_parse_rust_function",
          "folder_path": "crates/parseltongue-core/src",
          "filename": "parser_tests.rs",
          "language": "rust",
          "line_range": [45, 72],
          "detection_reason": "test_attribute"
        }
      ]
    },
    "word_count_coverage": {
      "avg_raw_coverage_pct": 72.3,
      "avg_effective_coverage_pct": 91.7,
      "files_above_80_pct_effective": 278,
      "files_below_80_pct_effective": 34,
      "files": [
        {
          "folder_path": "src/core",
          "filename": "parser.rs",
          "language": "rust",
          "source_word_count": 1580,
          "entity_word_count": 1203,
          "import_word_count": 245,
          "comment_word_count": 87,
          "raw_coverage_pct": 76.1,
          "effective_coverage_pct": 96.4,
          "entity_count": 12
        }
      ],
      "lowest_effective_coverage_files": [
        {
          "folder_path": "src/generated",
          "filename": "bindings.rs",
          "raw_coverage_pct": 12.4,
          "effective_coverage_pct": 14.1,
          "source_word_count": 8200,
          "entity_word_count": 1017,
          "import_word_count": 980,
          "comment_word_count": 0
        }
      ],
      "highest_effective_coverage_files": [
        {
          "folder_path": "src/core",
          "filename": "types.rs",
          "raw_coverage_pct": 94.7,
          "effective_coverage_pct": 99.5,
          "source_word_count": 420,
          "entity_word_count": 398,
          "import_word_count": 18,
          "comment_word_count": 2
        }
      ]
    }
  },
  "tokens": 1800
}
```

### Query Parameters (optional, future)

| Parameter | Default | Description |
|-----------|---------|-------------|
| `min_coverage` | `0` | Filter word coverage files to those below this threshold |
| `folder` | (all) | Filter to a specific folder path |
| `language` | (all) | Filter to a specific language |

---

## 8. Integration with Existing Pipeline

### pt01 Ingestion Changes

**File**: `crates/pt01-folder-to-cozodb-streamer/src/streamer.rs`

Two insertion points during ingestion:

**1. Test entity collection** (at `streamer.rs:803-807`):

Before:
```rust
if matches!(entity_class, EntityClass::TestImplementation) {
    test_count += 1;
    continue;
}
```

After:
```rust
if matches!(entity_class, EntityClass::TestImplementation) {
    test_count += 1;
    // v1.6.5: Record excluded test entity before skipping
    let (folder, file) = normalize_split_file_path(&file_path, &workspace_root);
    excluded_tests.push(ExcludedTestEntity {
        entity_name: isgl1_key.clone(),
        folder_path: folder,
        filename: file,
        entity_class: "TestImplementation".to_string(),
        language: language.to_string(),
        line_start: entity.line_range.0,
        line_end: entity.line_range.1,
        detection_reason: detect_reason.clone(),
    });
    continue;
}
```

**2. Word count collection** (after entity extraction + dependency queries per file):

```rust
// After all entities extracted for this file
let source_word_count = source.split_whitespace().count();
let entity_word_count: usize = entities_for_file
    .iter()
    .map(|e| e.content.split_whitespace().count())
    .sum();

// Import word count: accumulated during execute_dependency_query()
// (byte ranges from @dependency.* captures, deduplicated)
let import_word_count = deduplicated_import_ranges
    .iter()
    .map(|(start, end)| source[*start..*end].split_whitespace().count())
    .sum::<usize>();

// Comment word count: AST walk for top-level comment nodes
let comment_word_count = count_top_level_comment_words(&tree, &source, &language);

// Dual metrics
let raw_coverage_pct = if source_word_count > 0 {
    (entity_word_count as f64 / source_word_count as f64) * 100.0
} else { 100.0 };

let effective_source = source_word_count
    .saturating_sub(import_word_count)
    .saturating_sub(comment_word_count);
let effective_coverage_pct = if effective_source > 0 {
    (entity_word_count as f64 / effective_source as f64) * 100.0
} else { 100.0 };

let (folder, file) = normalize_split_file_path(&file_path, &workspace_root);
word_coverages.push(FileWordCoverageRow {
    folder_path: folder,
    filename: file,
    language: language.to_string(),
    source_word_count,
    entity_word_count,
    import_word_count,
    comment_word_count,
    raw_coverage_pct,
    effective_coverage_pct,
    entity_count: code_count,
});
```

**3. Batch insert after ingestion completes**:

```rust
// After all files processed, before returning StreamResult
db.insert_test_entities_excluded_batch(&excluded_tests).await?;
db.insert_file_word_coverage_batch(&word_coverages).await?;
```

### CozoDB Schema Changes

**File**: `crates/parseltongue-core/src/storage/cozo_client.rs`

Add two new schema creation methods called from `create_schema()`:
- `create_test_entities_excluded_schema()`
- `create_file_word_coverage_schema()`

Add batch insert methods:
- `insert_test_entities_excluded_batch()`
- `insert_file_word_coverage_batch()`

### HTTP Endpoint Handler

**File**: `crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/ingestion_diagnostics_coverage_handler.rs`

New handler module following existing patterns (see `ingestion_coverage_folder_handler.rs`).

Three data sources in the handler:
1. **Ignored files**: Walk filesystem at query time (same as existing coverage endpoint)
2. **Test entities**: Query `TestEntitiesExcluded` relation
3. **Word coverage**: Query `FileWordCoverage` relation

### Route Registration

**File**: `crates/pt08-http-code-query-server/src/route_definition_builder_module.rs`

Add route:
```rust
.route("/ingestion-diagnostics-coverage-report",
    get(ingestion_diagnostics_coverage_handler::handle_diagnostics_coverage_report))
```

### Dependency Addition

**File**: `crates/pt01-folder-to-cozodb-streamer/Cargo.toml`

```toml
[dependencies]
path-slash = "0.2"
```

---

## 9. Implementation Phases

### Phase 1: CozoDB Schema + Path Normalization

1. Add `path-slash` dependency
2. Implement `normalize_split_file_path()` utility function
3. Create `TestEntitiesExcluded` relation schema
4. Create `FileWordCoverage` relation schema
5. Write unit tests for path normalization (Windows `\`, macOS `/`, root-level files, nested paths)

### Phase 2: pt01 Ingestion Collection

1. Modify test entity exclusion path to collect entity details
2. Add import word count accumulation during `execute_dependency_query()` (uses existing `@dependency.*` captures)
3. Add comment word count via AST root walk after tree-sitter parse
4. Add word count computation with dual metrics (raw + effective) after entity extraction
5. Batch insert both relations after ingestion completes
6. Write integration tests verifying:
   - Both relations are populated
   - `import_word_count > 0` for files with imports
   - `comment_word_count > 0` for files with comments
   - `effective_coverage_pct >= raw_coverage_pct` for all files

### Phase 3: HTTP Endpoint Handler

1. Create handler module with three data-fetching functions
2. Implement filesystem walk for ignored files report
3. Implement CozoDB queries for test entities and word coverage
4. Compose the three reports into the response JSON
5. Register route
6. Write endpoint tests

### Phase 4: End-to-End Verification

1. Clean build + full test suite
2. Ingest a real codebase (Parseltongue itself)
3. Query the endpoint, verify all three reports are populated
4. Verify cross-platform path format in stored data

---

## 10. Acceptance Criteria

### Functional

- [ ] `GET /ingestion-diagnostics-coverage-report` returns HTTP 200 with all three report sections
- [ ] Ignored files section lists all files with unsupported extensions, grouped by extension
- [ ] Excluded test entities section lists all test entities with name, folder_path, filename, language, detection reason
- [ ] Word count coverage section shows per-file dual metrics: `raw_coverage_pct` and `effective_coverage_pct`
- [ ] Import word count is populated from existing `@dependency.*` tree-sitter captures (all 12 languages)
- [ ] Comment word count is populated from AST comment node walk (all 12 languages)
- [ ] `effective_coverage_pct >= raw_coverage_pct` for all files (subtracting words can only increase the ratio)
- [ ] All stored paths use forward slashes regardless of OS
- [ ] All stored paths are relative to workspace root
- [ ] `folder_path` and `filename` are correctly split for all path depths (root, nested, deep)
- [ ] Response includes summary aggregates (total counts, averages for both raw and effective)

### Non-Functional

- [ ] Endpoint responds in < 5 seconds for a 1000-file codebase
- [ ] No regressions in existing pt01 ingestion performance (< 10% overhead from word counting)
- [ ] `cargo test --all` passes with zero failures
- [ ] Zero TODOs/stubs in committed code

### Edge Cases

- [ ] Empty codebase (zero files) returns empty arrays, not errors
- [ ] File with zero words returns `coverage_pct: 100.0`
- [ ] Root-level files have `folder_path: ""`
- [ ] Binary files (images, compiled artifacts) correctly appear in ignored files list
- [ ] Files with only comments/imports show low `raw_coverage_pct` but high `effective_coverage_pct` (expected gap)
- [ ] Files where `import_word_count + comment_word_count >= source_word_count` get `effective_coverage_pct: 100.0` (saturating subtraction)

---

## 11. Open Questions

1. **Should `entity_word_count` include test entities?** Current plan: No -- test entities are excluded from CodeGraph, so their word count should not be part of the coverage metric. This means coverage metrics reflect only CODE entity coverage.

2. **Should the endpoint support pagination for large codebases?** For v1.6.5: No. Return all data. For future: Add `?limit=N&offset=M` if responses exceed reasonable token counts.

3. **Should the `FileWordCoverage` relation be updated during incremental reindex?** For v1.6.5: Only populated during full pt01 ingestion. Incremental reindex (file watcher) does not update word coverage. Future enhancement.

4. **Should comments inside function bodies be double-counted?** Current plan: No. Top-level comments only are counted for `comment_word_count` (comments inside entity bodies are already part of `entity_word_count`). A recursive AST walk would overcount.

5. **Should we count doc-comments separately from regular comments?** For v1.6.5: No distinction. Both `/// doc comments` and `// regular comments` are counted as comments. Future: Could split into `doc_comment_word_count` vs `inline_comment_word_count`.

---

## Appendix A: Import/Comment Coverage -- Design Rationale

### The Problem

Parseltongue extracts named entities (functions, classes, traits, interfaces, etc.) from source files. By design, certain code constructs fall *outside* any entity boundary:

- **Import statements**: `use std::collections::HashMap;`, `import React from 'react';`
- **Top-level comments**: Module-level documentation, license headers
- **Module declarations**: `mod foo;`, `package main`
- **Type aliases**: `type Result<T> = std::result::Result<T, Error>;`
- **Global constants**: `const MAX_RETRIES: u32 = 3;`

These are "expected gaps" -- code that *should* be outside entity boundaries. The original single `coverage_pct` metric conflated expected gaps with real missed code, making it impossible to tell whether 72% coverage was healthy or problematic.

### Why Dual Metrics Solve This

By counting import and comment words separately, we can subtract them from the denominator:

```
effective_source = total_words - import_words - comment_words
effective_coverage = entity_words / effective_source
```

This isolates the *real* coverage: what fraction of meaningful code (excluding boilerplate) did we actually capture?

### Why Zero New Tree-Sitter Queries

The dependency extraction system (`query_extractor.rs`) already walks every import/use/require/include node to build the dependency edge graph. The byte ranges of these nodes are already available during that traversal. We simply accumulate word counts from those ranges -- no additional tree-sitter query patterns needed.

Similarly, tree-sitter already parses the full AST. Walking for comment nodes is a traversal of the existing tree, not a new parse.

### Limitations

- **Module declarations, type aliases, global constants** are NOT subtracted. They are less common and harder to reliably detect across all 12 languages. Future enhancement.
- **Inline comments inside entity bodies** are NOT counted separately (they're already in `entity_word_count`).
- **Multi-line string literals** at module level (e.g., Python docstrings at module scope) are not identified as comments by tree-sitter and would reduce effective coverage. Acceptable edge case.

---

*End of PRD-v165*
