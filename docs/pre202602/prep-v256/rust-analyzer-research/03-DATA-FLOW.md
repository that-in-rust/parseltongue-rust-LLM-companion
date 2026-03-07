# Rust-Analyzer Data Flow Analysis

**Analysis Date:** 2026-01-29
**Source:** Parseltongue analysis of rust-analyzer codebase

## Overview

Rust-analyzer's data flow is built on a layered architecture where data transformations occur at well-defined boundaries. The system uses **Salsa** for incremental computation, ensuring efficient re-analysis when source files change.

## Core Data Structures

### 1. Virtual File System (VFS)

**Storage:** `Arc<RwLock<(vfs::Vfs, FxHashMap<FileId, LineEndings>)>>`

**Data Flow:**
```
File System
   ↓ (file watcher)
Loader (background thread)
   ↓ (vfs::loader::Message)
GlobalState::handle_vfs_msg()
   ↓ (VFS updates)
vfs::Vfs (in-memory file tree)
   ↓ (FileId mapping)
MemDocs (LSP-managed documents)
   ↓
AnalysisHost (Salsa database)
```

**Key Operations:**
- **File Addition:** New files discovered → VFS assigns FileId → Added to crate graph
- **File Modification:** Change detected → VFS updated → Salsa invalidation → Re-analysis
- **File Removal:** File deleted → VFS removes → Dependent queries invalidated

### 2. MemDocs (In-Memory Documents)

**Purpose:** Track LSP-managed document state

**Fields:**
- `file_path → Document`
- `Document { version, text, rope }`

**LSP Integration:**
```
LSP DidOpen
   ↓
MemDocs::add(path, version, text)
   ↓
VFS override (LSP content takes precedence)

LSP DidChange
   ↓
MemDocs::update(path, version, changes)
   ↓
Apply incremental edits
   ↓
VFS override updated
   ↓
Salsa invalidation

LSP DidClose
   ↓
MemDocs::remove(path)
   ↓
VFS reloads from disk
```

### 3. Crate Graph

**Structure:**
```
ProjectWorkspace[]
   ↓ (cargo metadata + build data)
CrateGraph
   ├─ CrateId[]
   │    ├─ root_file_id: FileId
   │    ├─ edition: Edition
   │    ├─ cfg_options: CfgOptions
   │    ├─ env: Env
   │    └─ dependencies: Vec<Dependency>
   └─ proc_macros: ProcMacroLoadResult
```

**Creation Flow:**
```
fetch_workspaces (cargo metadata)
   ↓
FetchWorkspaceResponse
   ↓
fetch_build_data (build scripts)
   ↓
FetchBuildDataResponse
   ↓
fetch_proc_macros (compile dylibs)
   ↓
FetchProcMacrosResponse
   ↓
switch_workspaces()
   ↓
CrateGraph constructed
   ↓
AnalysisHost.apply_change()
```

## Salsa Query System

### Query Layers

#### Layer 1: Syntax Queries
```
FileId
   ↓ [parse]
SyntaxTree (green tree)
   ↓ [ast operations]
AST nodes
```

**Incremental:** Reparsing reuses unchanged subtrees

#### Layer 2: Name Resolution Queries
```
CrateId
   ↓ [def_map]
DefMap (module tree, imports, macros)
   ↓ [item_tree]
ItemTree (per-file item catalog)
   ↓ [resolve_path]
Resolution (name → DefId)
```

**Incremental:** DefMap recalculation skips unchanged modules

#### Layer 3: Type Inference Queries
```
DefId (function)
   ↓ [infer]
InferenceResult
   ├─ type_of_expr: ExprId → Ty
   ├─ type_of_pat: PatId → Ty
   └─ diagnostics: Vec<InferenceDiagnostic>
```

**Incremental:** Only re-infers changed function bodies

#### Layer 4: HIR Queries
```
DefId
   ↓ [hir_query]
Various HIR information
   ├─ signature
   ├─ body
   ├─ generic_params
   └─ visibility
```

### Query Dependencies

```
file_text (input)
   ↓
parse
   ↓
item_tree ← macro_expand ← parse_macro
   ↓
def_map
   ↓
resolve_path
   ↓
infer ← lower_ty ← generic_predicates
   ↓
method_resolution
   ↓
completions / hover / goto_definition
```

**Invalidation Propagation:**
1. User edits file
2. `file_text` query invalidated
3. Salsa invalidates dependent queries
4. Next query access re-executes
5. Unchanged results short-circuit propagation

## LSP Message Flow

### Request: Completion

```
LSP client: textDocument/completion
   ↓
lsp_server::Message::Request
   ↓
on_new_request(request)
   ↓
handlers::handle_completion
   ↓
snapshot = GlobalState::snapshot()
   ↓
spawn task with snapshot
   ↓ (background thread)
ide::completions(db, position)
   ↓ (Salsa queries)
CompletionContext::new()
   ├─ Resolve name at cursor
   ├─ Determine completion kind
   └─ Gather semantic info
   ↓
complete_X() (various completers)
   ├─ complete_dot()
   ├─ complete_path()
   ├─ complete_keyword()
   └─ ...
   ↓
Vec<CompletionItem>
   ↓
to_proto::completion_item()
   ↓
lsp_types::CompletionItem[]
   ↓
LSP response
```

### Request: Hover

```
LSP client: textDocument/hover
   ↓
handlers::handle_hover
   ↓
snapshot
   ↓
ide::hover(db, position)
   ↓
Determine target
   ├─ Token at cursor
   ├─ Resolve to definition
   └─ Gather type info
   ↓
HoverResult
   ├─ markup: String (markdown)
   ├─ actions: Vec<HoverAction>
   └─ range: TextRange
   ↓
to_proto::hover()
   ↓
lsp_types::Hover
```

### Request: Goto Definition

```
LSP client: textDocument/definition
   ↓
handlers::handle_goto_definition
   ↓
ide::goto_definition(db, position)
   ↓
Resolve name
   ↓
NavigationTarget
   ├─ file_id
   ├─ range
   └─ name
   ↓
to_proto::location()
   ↓
lsp_types::Location[]
```

## Diagnostic Flow

### Pull Diagnostics

```
LSP client: textDocument/diagnostic
   ↓
handlers::handle_document_diagnostic
   ↓
snapshot
   ↓
ide::diagnostics(db, file_id)
   ↓ (Salsa queries)
Collect diagnostics
   ├─ Syntax errors
   ├─ Name resolution errors
   ├─ Type errors
   ├─ Unused items
   └─ ...
   ↓
Vec<Diagnostic>
   ↓
to_proto::diagnostic()
   ↓
lsp_types::Diagnostic[]
```

### Push Diagnostics (Flycheck)

```
File save
   ↓
DidSaveTextDocument
   ↓
Trigger flycheck
   ↓
FlycheckHandle::restart()
   ↓ (spawns cargo check)
CargoCheckHandle
   ↓ (streams JSON output)
FlycheckMessage::AddDiagnostic
   ↓
handle_flycheck_msg()
   ↓
DiagnosticCollection::set_native_diagnostics()
   ↓
publish_diagnostics()
   ↓
lsp_types::PublishDiagnosticsParams
```

## Workspace Loading

### Phase 1: Metadata

```
fetch_workspaces_queue.request_op()
   ↓
spawn task
   ↓
cargo metadata --format-version 1
   ↓ (parse JSON)
CargoWorkspace
   ├─ packages: Vec<Package>
   ├─ targets: Vec<Target>
   └─ workspace_root: PathBuf
   ↓
FetchWorkspaceResponse
   ↓
handle_task(FetchWorkspaces)
   ↓
wants_to_switch = Some(cause)
   ↓ (on next vfs_done)
switch_workspaces(cause)
```

### Phase 2: Build Data

```
fetch_build_data_queue.request_op()
   ↓
spawn task
   ↓
Run build scripts
   ↓ (captures OUT_DIR, env vars, cfgs)
BuildDataResult
   ├─ out_dirs: HashMap<PackageId, PathBuf>
   ├─ cfgs: HashMap<PackageId, Vec<CfgFlag>>
   └─ envs: HashMap<PackageId, Vec<(String, String)>>
   ↓
FetchBuildDataResponse
   ↓
Update crate graph
```

### Phase 3: Proc Macros

```
fetch_proc_macros_queue.request_op()
   ↓
spawn task
   ↓
Compile proc macro crates
   ↓ (cargo build --message-format=json)
Collect dylib paths
   ↓
Load proc macro servers
   ↓ (spawn proc-macro-srv processes)
ProcMacroLoadResult
   ↓
Update crate graph proc macros
   ↓
Re-expand macros
```

## Change Processing

### VFS Change Application

```
handle_vfs_msg(vfs::loader::Message)
   ↓
match message {
   Loaded { files } => {
      for (path, contents) in files {
         vfs.set_file_contents(path, contents)
      }
   }
   Progress { .. } => {
      // Update progress UI
   }
}
   ↓
process_changes()
```

### Salsa Change Application

```
process_changes() -> bool (state_changed)
   ↓
let changed_files = vfs.take_changes()
   ↓
For each changed FileId:
   ├─ Update source_root
   ├─ Invalidate file_text query
   └─ Track change
   ↓
analysis_host.apply_change(change)
   ↓ (Salsa invalidates dependencies)
return !changed_files.is_empty()
```

## Snapshot Mechanism

### Creating Snapshots

```
GlobalState::snapshot() -> GlobalStateSnapshot
```

**Purpose:** Provide read-only database access for background tasks

**Contents:**
```rust
GlobalStateSnapshot {
    config: Arc<Config>,
    workspaces: Arc<Vec<ProjectWorkspace>>,
    analysis: Analysis,  // Salsa snapshot
    vfs: Arc<RwLock<(vfs::Vfs, ..)>>,
    // ... other Arc-cloned fields
}
```

**Usage:**
```
Main thread (GlobalState)
   ↓
Create snapshot
   ↓
Spawn task with snapshot
   ↓ (background thread)
Query Analysis
   ├─ Read-only Salsa access
   ├─ No blocking of main thread
   └─ Stale data acceptable
```

## Configuration Flow

```
LSP: workspace/didChangeConfiguration
   ↓
on_notification(DidChangeConfiguration)
   ↓
self.config.update(new_config)
   ↓
Changes may trigger:
   ├─ Workspace reload (if cargo settings changed)
   ├─ Flycheck restart (if check settings changed)
   ├─ Feature recomputation (if features changed)
   └─ Client refresh (if display settings changed)
```

## Macro Expansion Data Flow

```
Source file with macro call
   ↓ [parse]
SyntaxTree
   ↓ [find macro calls]
MacroCallId
   ↓ [macro_expand]
Resolve macro definition
   ↓
match MacroKind {
   BuiltIn => expand_builtin(...)
   Declarative => mbe::expand(...)
   ProcMacro => proc_macro_server.expand(...)
}
   ↓
tt::TokenTree (expanded tokens)
   ↓ [parse_as_X]
Expanded SyntaxTree
   ↓ [item_tree/body_lowering]
Integrate into HIR
```

### Proc Macro Expansion

```
MacroCallId (proc macro)
   ↓
Lookup server for crate
   ↓
ProcMacroClient::expand(subtree)
   ↓ (IPC to proc-macro-srv process)
proc_macro_srv::expand()
   ↓ (loads dylib, calls expander)
TokenStream → TokenStream
   ↓ (serialized back)
tt::TokenTree
```

## Test Explorer Data Flow

```
File changes
   ↓
update_tests() (if enabled)
   ↓
discover_tests_in_module()
   ↓ (Salsa queries)
Find test functions
   ├─ #[test] annotations
   ├─ #[cfg(test)] modules
   └─ Doctest comments
   ↓
TestNode tree
   ↓
to_proto::test_item()
   ↓
lsp_ext::TestItem[]
   ↓
Publish to client
```

## Semantic Tokens Data Flow

```
LSP: textDocument/semanticTokens/full
   ↓
handlers::handle_semantic_tokens_full
   ↓
Check semantic_tokens_cache
   ↓ (if miss)
ide::semantic_tokens(db, file_id)
   ↓
Traverse syntax tree
   ↓
For each token:
   ├─ Resolve to semantic info
   ├─ Determine token type
   ├─ Determine modifiers
   └─ Record range
   ↓
Vec<SemanticToken>
   ↓
Encode as delta stream
   ↓
Cache result
   ↓
lsp_types::SemanticTokens
```

## Code Action Data Flow

```
LSP: textDocument/codeAction
   ↓
handlers::handle_code_action
   ↓
snapshot
   ↓
ide_assists::assists(db, range)
   ↓
For each registered assist:
   ├─ Check applicability
   ├─ Compute edits (if applicable)
   └─ Generate assist
   ↓
ide_diagnostics::diagnostics(db, file_id)
   ↓
Extract fixes from diagnostics
   ↓
Combine assists + fixes
   ↓
Vec<CodeAction>
   ↓
to_proto::code_action()
   ↓
lsp_types::CodeActionOrCommand[]
```

## Inlay Hints Data Flow

```
LSP: textDocument/inlayHint
   ↓
handlers::handle_inlay_hints
   ↓
ide::inlay_hints(db, file_id, range)
   ↓
Infer types for range
   ↓
For each applicable location:
   ├─ Type hints (let x = ...)
   ├─ Parameter hints (func(↓param: arg))
   ├─ Chaining hints (a.b().c()↓)
   ├─ Closure return hints (|x| ↓{...})
   ├─ Binding mode hints (match &x ↓{})
   └─ Generic parameter hints (func::<↓T>())
   ↓
Vec<InlayHint>
   ↓
to_proto::inlay_hint()
   ↓
lsp_types::InlayHint[]
```

## Optimization Strategies

### 1. Incremental Computation (Salsa)
- Memoizes query results
- Tracks dependencies
- Invalidates only affected queries
- Short-circuits on unchanged results

### 2. Lazy Evaluation
- Queries computed on-demand
- Unused code paths not analyzed
- Defers expensive operations

### 3. Caching
- Semantic tokens cached per file
- Parse trees reused across queries
- Type inference results memoized

### 4. Snapshot Isolation
- Background queries use snapshots
- No locking of main database
- Stale reads acceptable for IDE features

### 5. Batching
- VFS changes accumulated
- Diagnostics published in batches
- Multiple edits combined

### 6. Debouncing
- Workspace fetches debounced
- File watcher events coalesced
- Avoid redundant operations

## Data Persistence

### What's Persisted
- None (in-memory only)

### Why No Persistence
- Salsa queries are deterministic
- Full rebuild from source is fast enough
- Simplifies consistency model
- No serialization overhead

### Consequence
- Clean startup every launch
- No stale cache bugs
- Workspace loading required on start

## Memory Management

### Reference Counting
- `Arc<>` for shared ownership
- `workspaces`, `config`, `vfs` all Arc-wrapped
- Snapshots cheaply cloned

### Salsa GC
- `trigger_garbage_collection()` removes unused query results
- Runs when idle
- Revision-based tracking

### VFS Memory
- Files stored in-memory
- Line endings cached
- Rope data structure for efficient edits

## Summary

Rust-analyzer's data flow is characterized by:

1. **Layered Transformations:** Source → Syntax → Name Resolution → Types → IDE Features
2. **Incremental Computation:** Salsa minimizes redundant work
3. **Snapshot Isolation:** Background queries don't block
4. **Event-Driven Updates:** File changes trigger targeted re-analysis
5. **Caching & Memoization:** Expensive computations reused
6. **Batch Processing:** Similar events coalesced

The system maintains **strong consistency** for correctness while achieving **low latency** through clever caching and incremental techniques.
