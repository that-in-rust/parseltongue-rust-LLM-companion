# Parseltongue Test Corpus

Centralized test fixtures for all Parseltongue extraction and analysis tests.

## Naming Convention

Every test folder follows:

```
T{NNN}-word-word-word-word/
```

- **T{NNN}**: 3-digit number from the category band below
- **4 words**: Exactly 4 hyphenated words describing WHY the test exists
- Any LLM reading the folder name should intuitively understand the test's purpose

## Category Bands

| Range | Category |
|-------|----------|
| T001-T019 | Rust entity extraction |
| T020-T039 | Rust dependency edges |
| T040-T059 | Python parsing |
| T060-T079 | JavaScript parsing |
| T080-T099 | TypeScript parsing |
| T100-T119 | Go parsing |
| T120-T139 | Java parsing |
| T140-T159 | C / C++ parsing |
| T160-T179 | Ruby parsing |
| T180-T199 | PHP parsing |
| T200-T219 | C# parsing |
| T220-T239 | Swift parsing |
| T240-T259 | SQL parsing |
| T260-T299 | Cross-language features (ISGL1 keys, cozo escaping, generic sanitization) |
| T300-T349 | Storage infrastructure (CozoDB CRUD, blast radius, batch insert) |
| T350-T399 | E2E pipelines (ingestion, incremental reindex) |
| T400-T449 | HTTP server (file watcher, endpoints) |
| T450-T499 | Edge cases / regressions / known limitations |
| T500+ | Reserved for future |

Numbers within each band are not contiguous -- gaps are left for future tests.

## Folder Contents

Each T-folder contains:

1. **Raw source code files** -- the actual code Parseltongue should parse (`.rs`, `.py`, `.js`, etc.)
2. **EXPECTED.txt** -- structured prose describing what Parseltongue should extract

Nothing else. No snapshots, no JSON schemas, no generated files.

For pure-logic tests (ISGL1 keys, CozoDB ops) that don't parse source files, the T-folder contains only `EXPECTED.txt`.

## EXPECTED.txt Format

```
FIXTURE: T020-rust-async-await-edges
LANGUAGE: Rust
CATEGORY: Dependency Edges
VALIDATES: Async/await expressions create dependency edges to awaited functions

SOURCE FILES:
  - async_basic.rs       (basic await expressions)
  - async_error.rs       (await with ? operator)

EXPECTED ENTITIES:
  async_basic.rs:
    - fn "load_data" (async function, lines ~2-5)

EXPECTED DEPENDENCY EDGES:
  async_basic.rs:
    - load_data -> fetch_data        (await on function call)
    - load_data -> get               (await on method call: client.get())

MINIMUM EDGE COUNTS:
  async_basic.rs: at least 2 edges

KNOWN LIMITATIONS:
  None for this pattern.

RELATED TESTS:
  T064-javascript-async-await-edges (same pattern, JS)
```

### Field Descriptions

| Field | Required | Purpose |
|-------|----------|---------|
| FIXTURE | Yes | T-folder name (must match directory name exactly) |
| LANGUAGE | Yes | Primary language under test |
| CATEGORY | Yes | What aspect is tested (Entity Extraction, Dependency Edges, E2E Pipeline, etc.) |
| VALIDATES | Yes | One-sentence summary of what this test proves |
| SOURCE FILES | If applicable | List of source files with brief descriptions |
| EXPECTED ENTITIES | If applicable | What entities Parseltongue should extract per file |
| EXPECTED DEPENDENCY EDGES | If applicable | What edges should be detected per file |
| MINIMUM EDGE COUNTS | If applicable | Minimum number of edges per file (for automated assertion) |
| KNOWN LIMITATIONS | Optional | Document known gaps -- things we know we do NOT detect |
| RELATED TESTS | Optional | Cross-references to similar tests in other languages/categories |

## How Tests Run

The fixture harness (`crates/parseltongue-core/tests/fixture_harness.rs`) provides:

- `load_fixture_source_file(t_folder, filename)` -- reads source from T-folder
- `validate_fixture_extraction_results(t_folder, filenames)` -- parse + assert

Per-category test files (`t_rust_edge_tests.rs`, `t_python_tests.rs`, etc.) call the harness:

```rust
#[test]
fn t020_rust_async_await_edges() {
    validate_fixture_extraction_results(
        "T020-rust-async-await-edges",
        &["async_basic.rs", "async_error.rs"],
    );
}
```

Run all fixture tests: `cargo test --all`

## Adding a New Test

1. Pick the next available number in the appropriate category band
2. Create `test-fixtures/T{NNN}-word-word-word-word/`
3. Add raw source code files
4. Write `EXPECTED.txt`
5. Add a test function in the appropriate `t_{lang}_tests.rs` file
6. Run `cargo test` to verify
