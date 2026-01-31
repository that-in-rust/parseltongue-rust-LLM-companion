# Parseltongue Architecture Overview

**Analysis Date**: 2026-01-31
**Database**: rocksdb:parseltongue20260131154912/analysis.db
**Codebase Version**: 1.4.2

## Executive Summary

Parseltongue is a code analysis toolkit that parses codebases into a graph database (CozoDB) for efficient LLM-optimized querying. Core value: **99% token reduction** (2-5K tokens vs 500K raw dumps), **31x faster than grep**.

**Codebase Statistics**:
- 230 code entities
- 3 test entities
- 3,867 dependency edges
- Primary language: Rust
- Supports: Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, Ruby, PHP, C#, Swift

## High-Level Architecture

Parseltongue follows a **3-tier layered architecture**:

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI Binary (parseltongue)                │
│                    Dispatches to tools                       │
└─────────────────────┬───────────────────┬───────────────────┘
                      │                   │
        ┌─────────────▼──────────┐  ┌────▼────────────────────┐
        │ pt01-folder-to-cozodb  │  │ pt08-http-code-query    │
        │ -streamer              │  │ -server                  │
        │ (Ingestion Tool)       │  │ (HTTP REST API Server)   │
        └─────────────┬──────────┘  └────┬────────────────────┘
                      │                   │
                      └─────────┬─────────┘
                                │
                    ┌───────────▼────────────┐
                    │  parseltongue-core     │
                    │  (Shared Components)   │
                    │  - Tree-sitter parsing │
                    │  - Entity model        │
                    │  - CozoDB storage      │
                    └────────────────────────┘
```

### Crate Responsibilities

#### 1. **parseltongue** (CLI Binary)
- Entry point for all operations
- Dispatches commands to pt01 and pt08 tools
- Located: `crates/parseltongue/`

#### 2. **pt01-folder-to-cozodb-streamer** (Ingestion Tool)
- **Purpose**: Ingest codebase into CozoDB
- **Key modules**:
  - `streamer`: Directory traversal and processing
  - `file_watcher`: Real-time file monitoring
  - `isgl1_generator`: Entity key generation
  - `test_detector`: Classify TEST vs CODE entities
  - `lsp_client`: Language Server Protocol metadata
  - `cli`: Command-line interface
- **Output**: Creates timestamped workspace (e.g., `parseltongue20260131154912/analysis.db`)

#### 3. **pt08-http-code-query-server** (HTTP REST API)
- **Purpose**: Query and analyze stored codebase
- **14 HTTP endpoints** across 5 categories:
  - Core: health check, statistics, API docs
  - Entity: list, detail, fuzzy search
  - Graph: dependency edges, reverse callers, forward callees
  - Analysis: blast radius, circular dependencies, complexity hotspots, semantic clustering
  - Advanced: smart context for LLM token budgets
- **Key modules**:
  - `http_endpoint_handler_modules/`: 14 handler modules (one per endpoint)
  - `http_server_startup_runner`: Server lifecycle management
  - `file_watcher_integration_service`: Real-time update integration
  - `incremental_reindex_core_logic`: Incremental update engine
  - `port_selection`: Dynamic port binding

#### 4. **parseltongue-core** (Shared Library)
- **Purpose**: Common infrastructure for all tools
- **Key components**:
  - `storage/cozo_client.rs`: CozoDB interface (CozoDbStorage)
  - `query_extractor.rs`: Tree-sitter-based parsing (QueryBasedExtractor)
  - `entities.rs`: CodeEntity, DependencyEdge models
  - `entity_class_specifications.rs`: CODE vs TEST classification
  - `temporal.rs`: Temporal state tracking
  - `serializers/`: JSON and TOON output formats
  - `error.rs`: Structured error handling (thiserror)

## Layered Architecture Philosophy

Following Rust best practices, Parseltongue uses a **3-layer abstraction model**:

- **L1 Core**: Ownership, traits, Result/Option, RAII (no_std compatible)
- **L2 Standard**: Collections, iterators, Arc/Rc, Send/Sync
- **L3 External**: Async/await (Tokio), Serde, CozoDB, Axum

This ensures:
- Clear dependency boundaries
- Testable components
- Minimal coupling between crates

## Dependency Flow

```
parseltongue (binary)
    ├── pt01-folder-to-cozodb-streamer (tool)
    │   └── parseltongue-core (shared)
    └── pt08-http-code-query-server (tool)
        └── parseltongue-core (shared)
```

**Critical**: parseltongue-core is a **shared dependency**, not a parent crate. Tools depend on core, core does NOT depend on tools.

## Data Storage

- **Database**: CozoDB with RocksDB backend
- **Schema**:
  - `code_entities`: Primary entity table (key, file_path, entity_type, entity_class, language, etc.)
  - `dependency_edges`: Relationship graph (from_key, to_key, edge_type, source_location)
  - `file_hash_cache`: Incremental reindexing optimization
- **Entity Keys**: ISGL1 format - `language:entity_type:name:file_path:lines`
  - Example: `rust:fn:handle_server_health_check_status:__crates_pt08-http-code-query-server_src_http_endpoint_handler_modules_server_health_check_handler_rs:34-51`

## Naming Conventions

**FOUR-WORD NAMING**: All function/crate/command names must be exactly 4 words.

- Functions: `underscore_separated` (e.g., `filter_implementation_entities_only`)
- Crates: `hyphen-separated` (e.g., `pt01-folder-to-cozodb-streamer`)
- Pattern: `verb_constraint_target_qualifier()`

## Key Design Decisions

1. **HTTP-only architecture**: v1.4.2 removed standalone CLI querying, all queries via HTTP
2. **Always-on file watching**: Real-time codebase monitoring for incremental updates
3. **Token-optimized serialization**: TOON format for minimal LLM token usage
4. **Multi-language support**: 12 languages via tree-sitter grammars
5. **Temporal state tracking**: Entities track changes over time

## Performance Characteristics

From complexity hotspot analysis:
- Most coupled entities: `new`, `unwrap`, `to_string`, `Ok`, `Some` (stdlib usage)
- Hotspot functions:
  - `test_complete_cycle_graph_state`: 37 outbound dependencies
  - `create_schema`: 40 inbound dependencies
  - `create_dependency_edges_schema`: 55 inbound dependencies

Top coupling indicates heavy use of:
- Builder patterns (`new`, `builder`)
- Error handling (`unwrap`, `Ok`)
- Iterators (`iter`, `collect`, `map`)
- String operations (`to_string`, `clone`)

## Next Steps

See subsequent documents for:
- 02_CONTROL_FLOW.md - Detailed execution paths
- 03_DATA_FLOW.md - Entity lifecycle and transformations
- 04_MODULE_DEEP_DIVE.md - Per-module analysis
