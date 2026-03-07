# Rust-Analyzer Architecture Overview

**Analysis Date:** 2026-01-29
**Repository:** https://github.com/rust-lang/rust-analyzer
**Analysis Method:** Parseltongue Dependency Graph Generator v1.4.0

## Executive Summary

Rust-analyzer is a sophisticated Language Server Protocol (LSP) implementation for the Rust programming language. The codebase consists of **14,852 code entities** across **92,931 dependency edges** with **zero circular dependencies**, demonstrating excellent architectural hygiene.

## Key Statistics

- **Total Code Entities:** 14,852
- **Dependency Edges:** 92,931
- **Languages:** Rust (primary), Python (tooling)
- **Circular Dependencies:** 0 (excellent architectural health)
- **Files Processed:** 1,446 files
- **Test Entities:** 15,445 (excluded from analysis)

## High-Level Architecture

### Main Entry Point

The architecture centers around the **main_loop** pattern (`crates/rust-analyzer/src/main_loop.rs:41-72`), which establishes the LSP server lifecycle:

```
main_loop(Config, Connection) → GlobalState.run()
```

Key responsibilities:
- Thread priority management (especially on Windows)
- Optional DHAT profiling setup
- Initialization of GlobalState with sender/receiver channels

### Core State Management: GlobalState

The **GlobalState** struct (`crates/rust-analyzer/src/global_state.rs:84-198`) serves as the central orchestrator with 114 lines of fields covering:

#### 1. Communication Layer
- `sender`: LSP message sender
- `req_queue`: Request queue management

#### 2. Concurrent Execution
- `task_pool`: General task execution
- `fmt_pool`: Formatting-specific tasks
- `cancellation_pool`: Cancellation handler threads

#### 3. Analysis Engine
- `analysis_host`: Core analysis infrastructure (AnalysisHost)
- `diagnostics`: Diagnostic collection
- `mem_docs`: In-memory document management
- `source_root_config`: Source root configuration
- `local_roots_parent_map`: Source root hierarchy mapping

#### 4. Proc Macro Support
- `proc_macro_clients`: Procedural macro client connections
- `build_deps_changed`: Build dependency change tracking

#### 5. Flycheck (Diagnostics)
- `flycheck`: Flycheck handles for cargo check integration
- `flycheck_sender/receiver`: Async message channels
- `last_flycheck_error`: Error state tracking

#### 6. Test Explorer
- `test_run_session`: Active test run sessions
- `test_run_sender/receiver`: Test execution communication
- `test_run_remaining_jobs`: Job counter

#### 7. Project Discovery & Loading
- `discover_handles`: Project discovery handles
- `discover_sender/receiver`: Discovery message channels
- `fetch_ws_receiver`: Workspace fetch debouncing

#### 8. Virtual File System (VFS)
- `vfs`: Virtual file system with line ending tracking
- `loader`: VFS loader handle
- `vfs_config_version`: Configuration versioning
- `vfs_done`: Loading completion flag

#### 9. Workspace Management
- `workspaces`: Project workspace metadata
- `crate_graph_file_dependencies`: File dependency tracking
- `detached_files`: Standalone files

#### 10. Operation Queues
- `fetch_workspaces_queue`: Workspace metadata fetching
- `fetch_build_data_queue`: Build data fetching (proc macros, build scripts)
- `fetch_proc_macros_queue`: Proc macro compilation
- `prime_caches_queue`: Cache warming
- `deferred_task_queue`: Database-dependent deferred work

## Layered Architecture

### Layer 1: LSP Protocol Layer
- **Location:** `crates/rust-analyzer/src/`
- **Key Components:** main_loop.rs, handlers/, lsp/
- **Responsibilities:** LSP message handling, protocol translation

### Layer 2: IDE Layer
- **Location:** `crates/ide/`
- **Key Components:** Analysis host, semantic analysis
- **Responsibilities:** High-level IDE features (completion, hover, goto-def)

### Layer 3: HIR (High-Level Intermediate Representation)
- **Location:** `crates/hir/`, `crates/hir-def/`, `crates/hir-ty/`
- **Components:**
  - `hir`: High-level API
  - `hir-def`: Definition resolution
  - `hir-ty`: Type inference and checking
- **Responsibilities:** Semantic understanding of Rust code

### Layer 4: Syntax Layer
- **Location:** `crates/syntax/`, `crates/parser/`
- **Responsibilities:**
  - Parsing Rust source to CST (Concrete Syntax Tree)
  - Resilient error recovery
  - Incremental reparsing

### Layer 5: Foundation
- **Location:** `lib/`, various support crates
- **Components:**
  - `vfs`: Virtual file system
  - `salsa`: Incremental computation framework
  - `tt`: Token tree representation
  - `base-db`: Database foundation

## Data Flow Patterns

### 1. Document Change Flow
```
User Edit → VFS Update → Salsa Invalidation → Re-analysis → Diagnostics
```

### 2. Completion Request Flow
```
LSP Request → main_loop → GlobalState → AnalysisHost → IDE Layer → HIR Query → Response
```

### 3. Workspace Loading Flow
```
Discovery → Cargo Metadata → Build Scripts → Proc Macros → Crate Graph → Analysis Ready
```

## Concurrency Model

Rust-analyzer uses multiple concurrency strategies:

1. **Message Passing:** Channels for async communication
2. **Thread Pools:** task_pool, fmt_pool, cancellation_pool
3. **Lock-Free Queries:** Salsa provides query memoization
4. **Snapshots:** GlobalStateSnapshot for read-only access

## Key Design Patterns

### 1. Query-Based Architecture (Salsa)
- Incremental computation
- Automatic caching and invalidation
- Dependency tracking

### 2. Event Loop Pattern
- Central main_loop processes events
- Non-blocking request handling
- Deferred task execution

### 3. Layered Abstraction
- Clear separation between syntax, semantics, and IDE features
- Each layer has well-defined APIs

### 4. Resilient Parsing
- Syntax layer recovers from errors
- Partial ASTs enable IDE features even with invalid code

## Complexity Hotspots

Top 10 most-coupled entities (by inbound connections):

1. **Some** (2,476 callers) - Option construction
2. **new** (2,002 callers) - Constructor pattern
3. **check_assist** (1,720 callers) - Test infrastructure
4. **check** (1,647 callers) - Validation functions
5. **map** (1,647 callers) - Functional transformations
6. **into** (1,458 callers) - Type conversions
7. **clone** (1,455 callers) - Data cloning
8. **syntax** (1,345 callers) - Syntax tree access
9. **iter** (1,086 callers) - Iterator creation
10. **kind** (959 callers) - Type discrimination

## Module Organization

### Core LSP Server
- `crates/rust-analyzer/` - Main LSP server
- `crates/rust-analyzer/src/main_loop.rs` - Event loop
- `crates/rust-analyzer/src/global_state.rs` - State management
- `crates/rust-analyzer/src/handlers/` - LSP request handlers
- `crates/rust-analyzer/src/reload.rs` - Workspace reload logic

### IDE Features
- `crates/ide/` - IDE feature implementations
- `crates/ide-completion/` - Code completion
- `crates/ide-assists/` - Code actions/assists
- `crates/ide-diagnostics/` - Diagnostic generation
- `crates/ide-db/` - IDE database and helpers

### Semantic Analysis
- `crates/hir/` - High-level IR API
- `crates/hir-def/` - Definition and name resolution
- `crates/hir-ty/` - Type inference and checking
- `crates/hir-expand/` - Macro expansion

### Syntax and Parsing
- `crates/parser/` - Resilient parser
- `crates/syntax/` - Syntax tree and CST
- `crates/syntax-bridge/` - Bridge between parser and syntax tree
- `crates/tt/` - Token tree representation

### Project Model
- `crates/project-model/` - Workspace and crate graph
- `crates/load-cargo/` - Cargo workspace loading
- `crates/cfg/` - Configuration expression evaluation

### Support Libraries
- `crates/vfs/` - Virtual file system
- `crates/vfs-notify/` - File watching
- `crates/paths/` - Path utilities
- `crates/profile/` - Performance profiling

## Architectural Strengths

1. **Zero Circular Dependencies:** Clean module boundaries
2. **Incremental Computation:** Salsa-based query system minimizes redundant work
3. **Resilient Error Handling:** Continues providing features even with invalid code
4. **Modular Design:** Clear separation of concerns
5. **Extensive Testing:** 15,445 test entities

## Performance Characteristics

- **Thread Priority Optimization:** Main loop runs at elevated priority on Windows
- **Incremental Analysis:** Only reanalyzes changed portions
- **Snapshot-based Queries:** Read-only queries don't block writers
- **Debounced File Watching:** VFS changes are debounced to avoid thrashing
- **Lazy Crate Graph Construction:** Workspace loading is two-phase (metadata first, build data second)

## Next Steps

See the following documents for detailed analysis:
- `02-CONTROL-FLOW.md` - Detailed control flow patterns
- `03-DATA-FLOW.md` - Data flow and state transitions
- `04-KEY-COMPONENTS.md` - Deep dive into major components
