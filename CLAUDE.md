# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Parseltongue is a code analysis toolkit that parses codebases into a graph database (CozoDB) for efficient LLM-optimized querying. Core value: 99% token reduction (2-5K tokens vs 500K raw dumps), 31x faster than grep.

**Version**: 1.0.3 (HTTP-only architecture)
**Languages Supported**: Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, Ruby, PHP, C#, Swift

## Build and Test Commands

```bash
# Build
cargo build --release

# Run all tests
cargo test --all

# Run tests for specific crate
cargo test -p parseltongue-core
cargo test -p pt01-folder-to-cozodb-streamer
cargo test -p pt08-http-code-query-server

# Check for TODOs/stubs (must be clean before commit)
grep -r "TODO\|STUB\|PLACEHOLDER" --include="*.rs" crates/

# Clean build artifacts (do this regularly)
cargo clean
```

## CLI Usage (HTTP-Only Workflow)

```bash
# Step 1: Ingest codebase (auto-creates parseltongueTIMESTAMP/analysis.db)
parseltongue pt01-folder-to-cozodb-streamer .
# Output shows: Workspace: parseltongue20251201125000
#               Database: rocksdb:parseltongue20251201125000/analysis.db

# Step 2: Start HTTP server using the printed path
parseltongue serve-http-code-backend \
  --db "rocksdb:parseltongue20251201125000/analysis.db" --port 8080

# Step 3: Query via REST API
curl http://localhost:8080/server-health-check-status
curl http://localhost:8080/codebase-statistics-overview-summary
curl http://localhost:8080/code-entities-list-all
curl "http://localhost:8080/code-entities-search-fuzzy?q=handle"
curl "http://localhost:8080/blast-radius-impact-analysis?entity=rust:fn:main&hops=2"
```

**Note**: pt01 always creates a timestamped workspace folder - no `--db` flag needed.

## HTTP Server Endpoints (15 Total)

| Category | Endpoint | Description |
|----------|----------|-------------|
| Core | `/server-health-check-status` | Health check |
| Core | `/codebase-statistics-overview-summary` | Stats summary |
| Core | `/api-reference-documentation-help` | API docs |
| Entity | `/code-entities-list-all` | All entities |
| Entity | `/code-entity-detail-view/{key}` | Entity detail |
| Entity | `/code-entities-search-fuzzy?q=pattern` | Fuzzy search |
| Graph | `/dependency-edges-list-all` | All edges |
| Graph | `/reverse-callers-query-graph?entity=X` | Who calls X? |
| Graph | `/forward-callees-query-graph?entity=X` | What does X call? |
| Analysis | `/blast-radius-impact-analysis?entity=X&hops=N` | Impact analysis |
| Analysis | `/circular-dependency-detection-scan` | Cycle detection |
| Analysis | `/complexity-hotspots-ranking-view?top=N` | Coupling hotspots |
| Analysis | `/semantic-cluster-grouping-list` | Module clusters |
| Advanced | `/smart-context-token-budget?focus=X&tokens=N` | LLM context |
| Advanced | `/temporal-coupling-hidden-deps` | Hidden dependencies |

## Workspace Architecture

```
crates/
├── parseltongue/                        # CLI binary - dispatches to tools
├── parseltongue-core/                   # Shared types, traits, storage, tree-sitter parsing
├── pt01-folder-to-cozodb-streamer/      # Tool 1: Ingest codebase -> CozoDB
└── pt08-http-code-query-server/         # Tool 8: HTTP REST API server
```

**Dependency Flow**: `parseltongue` (binary) -> `pt01`/`pt08` (tools) -> `parseltongue-core` (shared)

## Naming Conventions (Critical)

**FOUR-WORD NAMING**: All function/crate/command names must be exactly 4 words.

```rust
// Functions: underscore-separated
filter_implementation_entities_only()    // Good
render_box_with_title_unicode()          // Good
filter_entities()                        // Bad - Too short

// Crates: hyphen-separated
pt01-folder-to-cozodb-streamer           // Good
pt08-http-code-query-server              // Good
```

**Pattern**: `verb_constraint_target_qualifier()`

## TDD Workflow

Follow STUB -> RED -> GREEN -> REFACTOR cycle:
1. Write failing test first
2. Run test, verify failure
3. Minimal implementation to pass
4. Refactor without breaking tests

## Error Handling

- **Libraries** (`parseltongue-core`): Use `thiserror` for structured errors
- **Applications** (CLI/tools): Use `anyhow` for context

## Layered Architecture

- **L1 Core**: Ownership, traits, Result/Option, RAII (no_std compatible)
- **L2 Standard**: Collections, iterators, Arc/Rc, Send/Sync
- **L3 External**: Async/await (Tokio), Serde, CozoDB, Axum

## Version Increment Rules

- Each version = ONE complete feature, end-to-end working
- Zero TODOs/stubs in commits
- All tests passing before commit

## Database Format

For `serve-http-code-backend`, always use RocksDB prefix:
```bash
--db "rocksdb:parseltongue20251201/analysis.db"    # Good
--db "parseltongue20251201/analysis.db"            # Bad - missing prefix
```

Note: `pt01-folder-to-cozodb-streamer` auto-creates the database - just copy the path it prints.
