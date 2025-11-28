# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Parseltongue is a code analysis toolkit that parses codebases into a graph database (CozoDB) for efficient LLM-optimized querying. Core value: 99% token reduction (2-5K tokens vs 500K raw dumps), 31× faster than grep.

**Version**: 1.0.2 (pure analysis system - editing tools removed in v1.0.0)
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

# Check for TODOs/stubs (must be clean before commit)
grep -r "TODO\|STUB\|PLACEHOLDER" --include="*.rs" crates/

# Clean build artifacts (do this regularly)
cargo clean
```

## CLI Usage

```bash
# Ingest codebase into database
parseltongue pt01-folder-to-cozodb-streamer . --db "rocksdb:mycode.db"

# Export dependency graph (progressive disclosure levels)
parseltongue pt02-level00 --where-clause "ALL" --output edges.json --db "rocksdb:mycode.db"  # ~3K tokens
parseltongue pt02-level01 --where-clause "ALL" --output entities.json --db "rocksdb:mycode.db"  # ~30K tokens
parseltongue pt02-level02 --where-clause "ALL" --output typed.json --db "rocksdb:mycode.db"  # ~60K tokens

# Visualize
parseltongue pt07 entity-count --db "rocksdb:mycode.db"
parseltongue pt07 cycles --db "rocksdb:mycode.db"
```

## Workspace Architecture

```
crates/
├── parseltongue/                        # CLI binary - dispatches to tools
├── parseltongue-core/                   # Shared types, traits, storage, tree-sitter parsing
├── pt01-folder-to-cozodb-streamer/      # Tool 1: Ingest codebase → CozoDB
├── pt02-llm-cozodb-to-context-writer/   # Tool 2: Query CozoDB → JSON exports
└── pt07-visual-analytics-terminal/      # Tool 7: Terminal visualizations
```

**Dependency Flow**: `parseltongue` (binary) → `pt01`/`pt02`/`pt07` (tools) → `parseltongue-core` (shared)

## Naming Conventions (Critical)

**FOUR-WORD NAMING**: All function/crate/command names must be exactly 4 words.

```rust
// Functions: underscore-separated
filter_implementation_entities_only()    // ✅
render_box_with_title_unicode()          // ✅
filter_entities()                        // ❌ Too short

// Crates: hyphen-separated
pt01-folder-to-cozodb-streamer           // ✅
pt07-visual-analytics-terminal           // ✅
```

**Pattern**: `verb_constraint_target_qualifier()`

## TDD Workflow

Follow STUB → RED → GREEN → REFACTOR cycle:
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
- **L3 External**: Async/await (Tokio), Serde, CozoDB

## Version Increment Rules

- v0.9.4 → v0.9.5 → ... → v0.9.9 → v1.0.0 (no v0.10.x)
- Each version = ONE complete feature, end-to-end working
- Zero TODOs/stubs in commits
- All tests passing before commit

## Database Format

Always use RocksDB prefix:
```bash
--db "rocksdb:mycode.db"    # ✅
--db "mycode.db"            # ❌
```

## Query Helpers (v0.9.7+)

After exporting JSON, traverse programmatically:
- `find_reverse_dependencies_by_key()` - Blast radius analysis
- `build_call_chain_from_root()` - Execution paths
- `filter_edges_by_type_only()` - Edge filtering
- `collect_entities_in_file_path()` - File search
