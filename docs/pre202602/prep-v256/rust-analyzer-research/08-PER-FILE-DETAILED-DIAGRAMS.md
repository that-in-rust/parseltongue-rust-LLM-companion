# Rust-Analyzer: Per-File Detailed Analysis with Mermaid Diagrams

**Generated from Parseltongue Database Queries**
**Database:** 14,852 entities, 92,931 edges

---

## rust-analyzer Crate - Main LSP Server

### File: main_loop.rs (30 entities)

**Purpose:** Core event loop that processes LSP messages and coordinates all subsystems

```mermaid
graph TB
    subgraph "main_loop.rs Structure"
        ML_FN[fn main_loop<br/>Entry point]

        EVENT_ENUM[enum Event<br/>8 variants]
        EVENT_ENUM --> LSP_VAR[Lsp - LSP messages]
        EVENT_ENUM --> TASK_VAR[Task - Background work]
        EVENT_ENUM --> VFS_VAR[Vfs - File changes]
        EVENT_ENUM --> FLY_VAR[Flycheck - Diagnostics]
        EVENT_ENUM --> TEST_VAR[TestResult]
        EVENT_ENUM --> DISC_VAR[DiscoverProject]
        EVENT_ENUM --> FETCH_VAR[FetchWorkspaces]
        EVENT_ENUM --> DEF_VAR[DeferredTask]

        DEFER_ENUM[enum DeferredTask<br/>Database-dependent work]
        DIAG_ENUM[enum DiagnosticsTaskKind]
        DISCOVER_ENUM[enum DiscoverProjectParam]
        PRIME_ENUM[enum PrimeCachesProgress<br/>Begin/Report/End]
        TASK_ENUM[enum Task<br/>Background tasks]

        GS_IMPL[impl GlobalState<br/>Event handling methods]
        GS_IMPL --> RUN_METHOD[fn run<br/>Main event loop]
        GS_IMPL --> NEXT_EVENT[fn next_event<br/>Wait for events]
        GS_IMPL --> HANDLE_EVENT[fn handle_event<br/>Process events]
        GS_IMPL --> HANDLE_TASK[fn handle_task]
        GS_IMPL --> HANDLE_VFS[fn handle_vfs_msg]
        GS_IMPL --> HANDLE_FLY[fn handle_flycheck_msg]
        GS_IMPL --> HANDLE_TEST[fn handle_cargo_test_msg]
        GS_IMPL --> HANDLE_DISC[fn handle_discover_msg]
        GS_IMPL --> HANDLE_DEF[fn handle_deferred_task]
        GS_IMPL --> PRIME_CACHES[fn prime_caches]
        GS_IMPL --> UPDATE_DIAG[fn update_diagnostics]
        GS_IMPL --> UPDATE_TESTS[fn update_tests]
        GS_IMPL --> CLEANUP[fn cleanup_discover_handles]

        ML_FN --> GS_IMPL
    end

    style ML_FN fill:#ff6b6b
    style EVENT_ENUM fill:#ffe66d
    style GS_IMPL fill:#4ecdc4
```

**Data Flow in main_loop.rs:**

```mermaid
sequenceDiagram
    participant Inbox as Inbox Channel
    participant MainLoop as main_loop()
    participant GlobalState
    participant Handlers

    MainLoop->>GlobalState: new(sender, config)
    GlobalState-->>MainLoop: GlobalState instance

    loop Event Loop
        MainLoop->>Inbox: next_event()
        Inbox-->>MainLoop: Event

        alt Event::Lsp
            MainLoop->>Handlers: handle_lsp_message()
        else Event::Task
            MainLoop->>Handlers: handle_task()
        else Event::Vfs
            MainLoop->>Handlers: handle_vfs_msg()
        else Event::Flycheck
            MainLoop->>Handlers: handle_flycheck_msg()
        end

        Handlers->>GlobalState: Update state
        GlobalState->>GlobalState: process_changes()

        alt is_quiescent()
            GlobalState->>GlobalState: Trigger optimizations
            GlobalState->>GlobalState: update_diagnostics()
        end
    end
```

---

### File: global_state.rs (44 entities)

**Purpose:** Central state management for the LSP server

```mermaid
classDiagram
    class GlobalState {
        +Sender~Message~ sender
        +ReqQueue req_queue
        +Handle~TaskPool~ task_pool
        +Handle~TaskPool~ fmt_pool
        +Arc~Config~ config
        +AnalysisHost analysis_host
        +DiagnosticCollection diagnostics
        +MemDocs mem_docs
        +Arc~RwLock~Vfs~~ vfs
        +Arc~Vec~ProjectWorkspace~~ workspaces
        +OpQueue fetch_workspaces_queue
        +OpQueue fetch_build_data_queue
        +OpQueue fetch_proc_macros_queue
        +OpQueue prime_caches_queue

        +new(sender, config) GlobalState
        +snapshot() GlobalStateSnapshot
        +process_changes() bool
        +switch_workspaces(cause)
        +is_quiescent() bool
        +send_request()
        +publish_diagnostics()
    }

    class GlobalStateSnapshot {
        +Arc~Config~ config
        +Analysis analysis
        +Arc~RwLock~Vfs~~ vfs
        +Arc~Vec~ProjectWorkspace~~ workspaces

        +url_to_file_id(url) FileId
        +file_id_to_url(id) Url
    }

    class AnalysisHost {
        +RootDatabase db

        +apply_change(change)
        +snapshot() Analysis
        +raw_database() RootDatabase
        +trigger_garbage_collection()
    }

    class MemDocs {
        +HashMap~VfsPath Document~ mem_docs

        +get(path) Document
        +set(path, version, text)
        +remove(path)
        +take_changes() bool
    }

    GlobalState --> AnalysisHost
    GlobalState --> MemDocs
    GlobalState --> GlobalStateSnapshot : creates
```

**State Transitions:**

```mermaid
stateDiagram-v2
    [*] --> Loading: new()

    Loading --> Running: VFS loaded
    Running --> Reloading: workspace change
    Reloading --> Running: reload complete

    Running --> Quiescent: no work
    Quiescent --> Running: new event

    Running --> Switching: wants_to_switch
    Switching --> Running: switch_workspaces()

    Quiescent --> [*]: shutdown
```

---

### File: lsp/ext.rs (152 entities)

**Purpose:** Custom LSP protocol extensions specific to rust-analyzer

```mermaid
mindmap
  root((lsp/ext.rs<br/>152 Entities))
    Requests
      AnalyzerStatus
      SyntaxTree
      ExpandMacro
      ViewHir
      ViewMir
      InterpretFunction
      MatchingBrace
      Runnables
      RelatedTests
      OpenDocs
      OpenCargoToml
    Notifications
      PublishDecorations
      DidChangeWorkspace
      CancelFlycheck
      RunFlycheck
      RefreshTestTree
    Types
      Runnable
      RunnableKind
      CargoRunnable
      TestInfo
      Position
      Range
      Location
```

---

## HIR Layer Crates

### File: hir/src/lib.rs

**Purpose:** High-level Intermediate Representation API

```mermaid
graph TB
    subgraph "HIR API Entry Points"
        SEMA[struct Semantics<br/>Bridge to HIR]

        MODULE[struct Module]
        FUNCTION[struct Function]
        STRUCT[struct Struct]
        ENUM[struct Enum]
        TRAIT[struct Trait]
        IMPL[struct Impl]

        TYPE[struct Type]
        CONST[struct Const]
        STATIC[struct Static]

        SEMA --> MODULE
        SEMA --> FUNCTION
        SEMA --> TYPE

        MODULE --> DECLS[declarations()]
        MODULE --> SCOPE[scope()]
        MODULE --> PATH_RES[resolve_path()]

        FUNCTION --> SIG[signature()]
        FUNCTION --> BODY[body()]
        FUNCTION --> PARAMS[params()]

        TYPE --> METHODS[iterate_methods()]
        TYPE --> FIELDS[fields()]
        TYPE --> IMPLS[impls()]
    end

    style SEMA fill:#ffe66d
    style MODULE fill:#fff4a3
    style FUNCTION fill:#ffd97d
    style TYPE fill:#ffb347
```

**Semantics Usage Pattern:**

```mermaid
sequenceDiagram
    participant User
    participant Semantics
    participant Database
    participant HIR

    User->>Semantics: new(&db)
    Semantics->>Database: Access queries

    User->>Semantics: resolve(name_ref)
    Semantics->>HIR: Look up in scope
    HIR-->>Semantics: Definition
    Semantics-->>User: ModuleDef

    User->>Semantics: type_of_expr(expr)
    Semantics->>Database: infer(function)
    Database-->>Semantics: InferenceResult
    Semantics-->>User: Type
```

---

### File: hir-def/src/nameres/mod.rs

**Purpose:** Name resolution and DefMap construction

```mermaid
graph TD
    subgraph "Name Resolution Pipeline"
        CRATE[CrateId]
        CRATE --> COLLECT[DefCollector]

        COLLECT --> RESOLVE_IMPORTS[resolve imports<br/>use statements]
        COLLECT --> EXPAND_MACROS[expand macros<br/>macro_rules!, proc macros]
        COLLECT --> RESOLVE_ITEMS[resolve items<br/>functions, types, etc.]

        RESOLVE_IMPORTS --> DEF_MAP[DefMap]
        EXPAND_MACROS --> DEF_MAP
        RESOLVE_ITEMS --> DEF_MAP

        DEF_MAP --> MODULES[Module tree]
        DEF_MAP --> SCOPE[Scope per module]
        DEF_MAP --> PRELUDE[Preludes]
        DEF_MAP --> EXTERNS[Extern crates]
    end

    style COLLECT fill:#ffe66d
    style DEF_MAP fill:#4ecdc4
```

---

### File: hir-ty/src/infer/mod.rs

**Purpose:** Type inference algorithm

```mermaid
flowchart TD
    FN_BODY[Function Body HIR]

    FN_BODY --> INFER_CTX[InferenceContext::new]

    INFER_CTX --> INFER_BODY[infer_body]

    INFER_BODY --> INFER_EXPR[infer_expr]
    INFER_BODY --> INFER_PAT[infer_pat]
    INFER_BODY --> INFER_STMT[infer_stmt]

    INFER_EXPR --> COLLECT_CONST[Collect constraints<br/>Literals, calls, etc.]

    COLLECT_CONST --> UNIFY[Unification<br/>Solve type variables]

    UNIFY --> COERCE[Coercion<br/>Apply implicit conversions]

    COERCE --> RESULT[InferenceResult<br/>type_of_expr<br/>type_of_pat]

    style INFER_CTX fill:#ffe66d
    style UNIFY fill:#ff6b6b
    style RESULT fill:#4ecdc4
```

---

## IDE Layer Crates

### File: ide/src/lib.rs

**Purpose:** High-level IDE feature coordinator

```mermaid
graph LR
    subgraph "IDE Features API"
        ANALYSIS[struct Analysis]

        ANALYSIS --> COMPL[completions]
        ANALYSIS --> HOVER[hover]
        ANALYSIS --> GOTO[goto_definition]
        ANALYSIS --> REFS[find_all_refs]
        ANALYSIS --> RENAME[rename]
        ANALYSIS --> DIAG[diagnostics]
        ANALYSIS --> HIGHLIGHT[syntax_highlighting]
        ANALYSIS --> INLAY[inlay_hints]
        ANALYSIS --> CODELENS[code_lens]
        ANALYSIS --> SIGNATURE[signature_help]
        ANALYSIS --> SYMBOLS[symbol_search]
        ANALYSIS --> STRUCTURE[file_structure]
        ANALYSIS --> RUNNABLES[runnables]
    end

    style ANALYSIS fill:#4ecdc4
```

---

### File: ide-completion/src/lib.rs

**Purpose:** Code completion engine

```mermaid
flowchart TB
    POSITION[Cursor Position]

    POSITION --> CONTEXT[CompletionContext::new]

    CONTEXT --> CLASSIFY{Classify Context}

    CLASSIFY --> DOT[Dot Completion<br/>foo.|]
    CLASSIFY --> PATH[Path Completion<br/>foo::|]
    CLASSIFY --> KEYWORD[Keyword Completion]
    CLASSIFY --> ATTR[Attribute Completion<br/>#[|]]

    DOT --> DOT_COMP[completions/dot.rs]
    PATH --> PATH_COMP[completions/use_.rs]
    KEYWORD --> KEYWORD_COMP[completions/keyword.rs]
    ATTR --> ATTR_COMP[completions/attribute.rs]

    DOT_COMP --> RENDER[Rendering]
    PATH_COMP --> RENDER
    KEYWORD_COMP --> RENDER
    ATTR_COMP --> RENDER

    RENDER --> FUNC_RENDER[render/function.rs]
    RENDER --> STRUCT_RENDER[render/struct_literal.rs]
    RENDER --> PATTERN_RENDER[render/pattern.rs]

    FUNC_RENDER --> ITEMS[CompletionItem[]]
    STRUCT_RENDER --> ITEMS
    PATTERN_RENDER --> ITEMS

    style CONTEXT fill:#ffe66d
    style RENDER fill:#4ecdc4
    style ITEMS fill:#95e1d3
```

---

## Syntax Layer Crates

### File: syntax/src/lib.rs

**Purpose:** Concrete Syntax Tree implementation

```mermaid
classDiagram
    class SyntaxNode {
        <<Red Node>>
        +kind() SyntaxKind
        +text_range() TextRange
        +parent() SyntaxNode
        +children() Iterator
        +first_child() SyntaxNode
        +last_child() SyntaxNode
        +siblings() Iterator
        +descendants() Iterator
        +ancestors() Iterator
    }

    class GreenNode {
        <<Immutable>>
        +kind() SyntaxKind
        +text_len() TextSize
        +children() &[GreenElement]
    }

    class SyntaxToken {
        +kind() SyntaxKind
        +text() &str
        +text_range() TextRange
        +parent() SyntaxNode
        +next_sibling_or_token()
        +prev_sibling_or_token()
    }

    class SourceFile {
        <<AST Root>>
        +parse(text) Parse
        +items() Iterator~Item~
        +errors() Vec~SyntaxError~
    }

    SyntaxNode --> GreenNode : wraps
    SyntaxToken --> GreenNode : wraps
    SourceFile --> SyntaxNode : contains
```

---

### File: parser/src/lib.rs

**Purpose:** Resilient incremental parser

```mermaid
flowchart LR
    TEXT[Source Text]

    TEXT --> LEXER[Lexer<br/>Text → Tokens]

    LEXER --> TOKENS[Token Stream<br/>SyntaxKind[]]

    TOKENS --> PARSER[Parser<br/>Grammar rules]

    PARSER --> EVENTS[Parser Events<br/>Start/Finish/Token]

    EVENTS --> TREE_SINK[TreeSink<br/>Build tree]

    TREE_SINK --> GREEN[GreenNode<br/>Immutable CST]

    GREEN --> SYNTAX[SyntaxNode<br/>With parent pointers]

    style LEXER fill:#ffccbc
    style PARSER fill:#ffe66d
    style GREEN fill:#c8e6c9
    style SYNTAX fill:#4ecdc4
```

---

## Foundation Crates

### File: base-db/src/lib.rs

**Purpose:** Salsa database foundation

```mermaid
graph TB
    subgraph "Database Layers"
        ROOT_DB[RootDatabase<br/>Main database]

        ROOT_DB --> SOURCE_DB[SourceDatabase<br/>File contents]
        ROOT_DB --> EXPAND_DB[ExpandDatabase<br/>Macro expansion]
        ROOT_DB --> DEF_DB[DefDatabase<br/>Definitions]
        ROOT_DB --> HIR_DB[HirDatabase<br/>Type inference]

        SOURCE_DB --> FILE_TEXT[file_text: FileId → String]
        SOURCE_DB --> FILE_SOURCE_ROOT[file_source_root]
        SOURCE_DB --> SOURCE_ROOT_CRATES[source_root_crates]

        EXPAND_DB --> PARSE_MACRO[parse_macro_expansion]
        EXPAND_DB --> MACRO_EXPAND[macro_expand]

        DEF_DB --> ITEM_TREE[item_tree]
        DEF_DB --> DEF_MAP[crate_def_map]
        DEF_DB --> BODY[body]

        HIR_DB --> INFER[infer]
        HIR_DB --> TY[ty]
        HIR_DB --> CALLABLE_SIG[callable_item_signature]
    end

    style ROOT_DB fill:#a8dadc
    style SOURCE_DB fill:#e1f5ff
    style DEF_DB fill:#ffe66d
    style HIR_DB fill:#ff6b6b
```

---

### File: vfs/src/lib.rs

**Purpose:** Virtual File System

```mermaid
classDiagram
    class Vfs {
        +HashMap~VfsPath FileId~ path_map
        +Vec~FileState~ file_set

        +set_file_contents(path, contents)
        +file_contents(id) &[u8]
        +file_path(id) &VfsPath
        +take_changes() Vec~ChangedFile~
    }

    class VfsPath {
        +as_path() PathBuf
        +normalize()
    }

    class FileId {
        +0 u32
    }

    class Loader {
        <<trait>>
        +load(entries)
        +set_contents(path, bytes)
    }

    Vfs --> VfsPath
    Vfs --> FileId
    Loader ..> Vfs : updates
```

---

## Project Model Crates

### File: project-model/src/workspace.rs

**Purpose:** Cargo workspace loading

```mermaid
sequenceDiagram
    participant Workspace
    participant Cargo
    participant BuildData
    participant ProcMacro
    participant CrateGraph

    Workspace->>Cargo: cargo metadata
    Cargo-->>Workspace: CargoWorkspace

    Workspace->>BuildData: Run build scripts
    BuildData-->>Workspace: OUT_DIR, cfgs, env

    Workspace->>ProcMacro: Compile proc macros
    ProcMacro-->>Workspace: dylib paths

    Workspace->>CrateGraph: to_crate_graph()

    CrateGraph->>CrateGraph: Add crates
    CrateGraph->>CrateGraph: Add dependencies
    CrateGraph->>CrateGraph: Attach proc macros

    CrateGraph-->>Workspace: Complete CrateGraph
```

---

## Summary: Entity Counts by Key Files

| File | Entities | Purpose |
|------|----------|---------|
| lsp/ext.rs | 152 | LSP protocol extensions |
| global_state.rs | 44 | Central state management |
| main_loop.rs | 30 | Event loop core |
| diagnostics.rs | 22 | Diagnostic coordination |
| reload.rs | 20 | Workspace reload logic |
| mem_docs.rs | 12 | LSP document tracking |

## Data Flow Summary

```mermaid
graph LR
    FS[File System] --> VFS[VFS]
    VFS --> PARSE[Parser]
    PARSE --> SYNTAX[Syntax Tree]
    SYNTAX --> HIR_DEF[hir-def<br/>Name Resolution]
    HIR_DEF --> HIR_TY[hir-ty<br/>Type Inference]
    HIR_TY --> IDE[IDE Features]
    IDE --> LSP[LSP Server]
    LSP --> CLIENT[Editor Client]

    style VFS fill:#fff3e0
    style SYNTAX fill:#f3e5f5
    style HIR_TY fill:#ffe66d
    style IDE fill:#4ecdc4
    style LSP fill:#ff6b6b
```

This analysis was generated purely from Parseltongue database queries without using grep or glob.
