# Rust-Analyzer Folder Structure & Organization

**Visual Guide with Mermaid Diagrams**

## Overall Codebase Structure

```mermaid
graph TB
    ROOT[rust-analyzer Repository]

    ROOT --> CRATES[crates/ - Main Workspace]
    ROOT --> LIB[lib/ - Support Libraries]
    ROOT --> EDITORS[editors/ - Editor Extensions]
    ROOT --> XTASK[xtask/ - Build Tools]

    CRATES --> LSP[rust-analyzer - LSP Server]
    CRATES --> IDE_LAYER[IDE Layer Crates]
    CRATES --> HIR_LAYER[HIR Layer Crates]
    CRATES --> SYNTAX_LAYER[Syntax Layer Crates]
    CRATES --> SUPPORT[Support Crates]

    LIB --> ARENA[la-arena - Arena Allocator]
    LIB --> LINE_IDX[line-index - Line Indexing]
    LIB --> LSP_SVR[lsp-server - LSP Protocol]
    LIB --> SMOL[smol_str - String Interning]

    style ROOT fill:#e1f5ff
    style CRATES fill:#fff4e1
    style LIB fill:#e8f5e9
```

## Main Crates Dependency Layers

```mermaid
graph TB
    subgraph "Layer 1: LSP Protocol"
        RA[rust-analyzer<br/>LSP Server Entry Point]
    end

    subgraph "Layer 2: IDE Features"
        IDE[ide<br/>High-level IDE API]
        IDE_COMP[ide-completion<br/>Code Completion]
        IDE_ASST[ide-assists<br/>Code Actions]
        IDE_DIAG[ide-diagnostics<br/>Diagnostics]
        IDE_SSR[ide-ssr<br/>Structural Search/Replace]
        IDE_DB[ide-db<br/>IDE Database]
    end

    subgraph "Layer 3: HIR (Semantic Analysis)"
        HIR[hir<br/>High-level IR API]
        HIR_DEF[hir-def<br/>Definitions & Name Resolution]
        HIR_TY[hir-ty<br/>Type Inference & Checking]
        HIR_EXP[hir-expand<br/>Macro Expansion]
    end

    subgraph "Layer 4: Syntax"
        SYNTAX[syntax<br/>CST & AST]
        PARSER[parser<br/>Resilient Parser]
        SYN_BR[syntax-bridge<br/>Token Tree Bridge]
        TT[tt<br/>Token Trees]
        MBE[mbe<br/>Macro-by-example]
    end

    subgraph "Layer 5: Foundation"
        BASE_DB[base-db<br/>Salsa Database]
        VFS[vfs<br/>Virtual File System]
        SPAN[span<br/>Source Spans]
        PATHS[paths<br/>Path Utilities]
    end

    RA --> IDE
    RA --> IDE_COMP
    RA --> IDE_ASST
    RA --> IDE_DIAG

    IDE --> HIR
    IDE_COMP --> HIR
    IDE_ASST --> HIR
    IDE_DIAG --> HIR

    HIR --> HIR_DEF
    HIR --> HIR_TY
    HIR_DEF --> HIR_EXP
    HIR_TY --> HIR_DEF

    HIR_DEF --> SYNTAX
    HIR_EXP --> MBE

    SYNTAX --> PARSER
    MBE --> TT
    SYN_BR --> PARSER

    PARSER --> BASE_DB
    HIR_DEF --> BASE_DB
    BASE_DB --> VFS

    style RA fill:#ff6b6b
    style IDE fill:#4ecdc4
    style HIR fill:#ffe66d
    style SYNTAX fill:#95e1d3
    style BASE_DB fill:#a8dadc
```

## Complete Crates Organization

```mermaid
mindmap
  root((rust-analyzer<br/>36 Crates))
    LSP Server
      rust-analyzer
        Main LSP implementation
        Request handlers
        Global state management
    IDE Layer
      ide
        High-level IDE API
        Feature coordination
      ide-completion
        Code completion logic
        Snippet generation
      ide-assists
        Code actions/quick fixes
        Refactoring assists
      ide-diagnostics
        Diagnostic generation
        Error reporting
      ide-db
        Search & indexing
        Symbol resolution
      ide-ssr
        Structural search/replace
        Pattern matching
    HIR Semantic
      hir
        High-level IR API
        Trait system
      hir-def
        Name resolution
        Item definitions
        Module tree
      hir-ty
        Type inference
        Trait solving
        Method resolution
      hir-expand
        Macro expansion
        Attribute processing
    Syntax & Parsing
      syntax
        Concrete syntax tree
        AST nodes
      parser
        Resilient parsing
        Error recovery
      syntax-bridge
        CST to token tree
        Span mapping
      tt
        Token tree representation
      mbe
        Declarative macros
        Pattern matching
    Project Model
      project-model
        Workspace loading
        Cargo integration
      load-cargo
        Cargo workspace parsing
      cfg
        Cfg expression evaluation
      toolchain
        Rust toolchain detection
    Proc Macros
      proc-macro-api
        Proc macro client API
      proc-macro-srv
        Proc macro server
      proc-macro-srv-cli
        CLI for proc macro server
    Foundation
      base-db
        Salsa database
        Query system
      vfs
        Virtual file system
      vfs-notify
        File watching
      span
        Source locations
      paths
        Path normalization
    Utilities
      stdx
        Standard extensions
      profile
        Performance profiling
      test-utils
        Testing utilities
      test-fixture
        Test fixture parsing
      edition
        Rust edition handling
      macros
        Derive macros for RA
      intern
        String interning
```

## File-by-File Structure (Key Crates)

### rust-analyzer Crate

```mermaid
graph LR
    subgraph "rust-analyzer/src/"
        MAIN[main.rs<br/>Entry point]
        MAIN_LOOP[main_loop.rs<br/>Event loop core]
        GLOBAL[global_state.rs<br/>GlobalState struct]
        CONFIG[config.rs<br/>Configuration]
        RELOAD[reload.rs<br/>Workspace reload]

        HANDLERS[handlers/<br/>LSP handlers]
        DIAG[diagnostics/<br/>Diagnostic logic]
        LSP_DIR[lsp/<br/>LSP utilities]

        HANDLERS --> REQ[request.rs<br/>Request handlers]
        HANDLERS --> NOTIF[notification.rs<br/>Notifications]

        MAIN --> MAIN_LOOP
        MAIN_LOOP --> GLOBAL
        GLOBAL --> CONFIG
        GLOBAL --> RELOAD
        MAIN_LOOP --> HANDLERS
    end

    style MAIN fill:#ff6b6b
    style MAIN_LOOP fill:#ff8787
    style GLOBAL fill:#ffa5a5
```

### hir-def Crate (Name Resolution)

```mermaid
graph TB
    subgraph "hir-def/src/"
        LIB[lib.rs<br/>Crate root]

        NAMERES[nameres/<br/>Name resolution]
        NAMERES --> COLLECTOR[collector.rs<br/>Def collection]
        NAMERES --> PATH_RES[path_resolution.rs<br/>Path resolution]
        NAMERES --> DEF_MAP[mod.rs<br/>DefMap]

        BODY_DIR[body/<br/>Function bodies]
        BODY_DIR --> LOWER[lower.rs<br/>HIR lowering]
        BODY_DIR --> SCOPE[scope.rs<br/>Scoping]

        ITEM[item_tree.rs<br/>Item tree]
        ATTRS[attrs.rs<br/>Attributes]
        VISIBILITY[visibility.rs<br/>Visibility]

        LIB --> NAMERES
        LIB --> BODY_DIR
        LIB --> ITEM

        NAMERES --> ITEM
        BODY_DIR --> ITEM
    end

    style NAMERES fill:#ffe66d
    style BODY_DIR fill:#fff4a3
```

### hir-ty Crate (Type Inference)

```mermaid
graph TB
    subgraph "hir-ty/src/"
        LIB[lib.rs<br/>Type system]

        INFER[infer/<br/>Type inference]
        INFER --> UNIFY[unify.rs<br/>Unification]
        INFER --> COERCE[coerce.rs<br/>Coercion]
        INFER --> EXPR[expr.rs<br/>Expression typing]

        METHOD[method_resolution.rs<br/>Method resolution]

        LOWER[lower.rs<br/>Type lowering]

        AUTODEREF[autoderef.rs<br/>Auto-deref]

        NEXT[next_solver/<br/>New trait solver]

        LIB --> INFER
        LIB --> METHOD
        LIB --> LOWER

        METHOD --> AUTODEREF
        INFER --> METHOD
    end

    style INFER fill:#4ecdc4
    style METHOD fill:#7fdbda
```

### ide Crate (IDE Features)

```mermaid
graph TB
    subgraph "ide/src/"
        LIB[lib.rs<br/>IDE API]

        HOVER[hover/<br/>Hover info]
        GOTO[goto_definition.rs<br/>Go to definition]
        REFS[references.rs<br/>Find references]
        RENAME[rename.rs<br/>Rename refactoring]

        HIGHLIGHT[syntax_highlighting/<br/>Syntax highlighting]

        INLAY[inlay_hints/<br/>Inlay hints]

        RUNNABLES[runnables.rs<br/>Runnable detection]

        LIB --> HOVER
        LIB --> GOTO
        LIB --> REFS
        LIB --> RENAME
        LIB --> HIGHLIGHT
        LIB --> INLAY
    end

    style LIB fill:#95e1d3
    style HOVER fill:#b8f5e6
```

## Data Flow Between Folders

```mermaid
flowchart TD
    FS[File System]

    VFS_N[vfs-notify<br/>File Watcher]
    VFS[vfs<br/>Virtual FS]

    PARSER[parser<br/>Parsing]
    SYNTAX[syntax<br/>CST/AST]

    HIR_EXP[hir-expand<br/>Macro Expansion]
    HIR_DEF[hir-def<br/>Name Resolution]
    HIR_TY[hir-ty<br/>Type Inference]

    IDE[ide<br/>Features]

    RA[rust-analyzer<br/>LSP Server]

    CLIENT[LSP Client<br/>Editor]

    FS -->|File Changes| VFS_N
    VFS_N -->|Events| VFS
    VFS -->|File Content| PARSER

    PARSER -->|Syntax Tree| SYNTAX
    SYNTAX -->|AST| HIR_EXP

    HIR_EXP -->|Expanded Code| HIR_DEF
    HIR_DEF -->|Definitions| HIR_TY

    HIR_TY -->|Type Info| IDE
    HIR_DEF -->|Name Info| IDE

    IDE -->|IDE Data| RA
    RA <-->|LSP Messages| CLIENT

    style FS fill:#e8f5e9
    style CLIENT fill:#e1f5ff
    style RA fill:#ff6b6b
    style IDE fill:#4ecdc4
    style HIR_TY fill:#ffe66d
    style VFS fill:#f0f0f0
```

## Lib Directory Structure

```mermaid
graph TB
    subgraph "lib/ - Support Libraries"
        ARENA[la-arena<br/>Arena-based allocation<br/>Index-based data structures]
        LINE[line-index<br/>UTF-8 line indexing<br/>Offset ↔ Line:Col conversion]
        LSP[lsp-server<br/>LSP protocol implementation<br/>Message handling]
        SMOL[smol_str<br/>Small string optimization<br/>String interning]
        TEXT[text-size<br/>Text offset types<br/>Type-safe sizes]
        UNGRAMMAR[ungrammar<br/>Grammar specification<br/>For parser generation]
    end

    style ARENA fill:#bbdefb
    style LINE fill:#c8e6c9
    style LSP fill:#ffccbc
    style SMOL fill:#f8bbd0
    style TEXT fill:#d1c4e9
    style UNGRAMMAR fill:#ffe0b2
```

## Project Model Flow

```mermaid
sequenceDiagram
    participant Workspace
    participant project-model
    participant load-cargo
    participant cfg
    participant proc-macro-api
    participant base-db

    Workspace->>project-model: Load project
    project-model->>load-cargo: Read Cargo.toml
    load-cargo->>load-cargo: cargo metadata
    load-cargo-->>project-model: CargoWorkspace

    project-model->>cfg: Parse cfg expressions
    cfg-->>project-model: CfgOptions

    project-model->>proc-macro-api: Load proc macros
    proc-macro-api->>proc-macro-api: Compile dylibs
    proc-macro-api-->>project-model: ProcMacroClient

    project-model->>base-db: Build CrateGraph
    base-db-->>Workspace: Analysis ready
```

## Syntax Parsing Flow

```mermaid
flowchart LR
    SOURCE[Source Code]

    LEX[Lexer<br/>in parser crate]
    PARSE[Parser<br/>Resilient parsing]
    GREEN[GreenNode<br/>Immutable tree]
    RED[RedNode<br/>SyntaxNode]
    AST[AST Nodes<br/>Typed wrappers]

    SOURCE -->|Text| LEX
    LEX -->|Tokens| PARSE
    PARSE -->|Events| GREEN
    GREEN -->|Wrap| RED
    RED -->|Cast| AST

    style SOURCE fill:#f0f0f0
    style GREEN fill:#c8e6c9
    style RED fill:#ffccbc
    style AST fill:#bbdefb
```

## Summary of Key Directories

| Directory | Purpose | Key Files | ELI5 |
|-----------|---------|-----------|------|
| `rust-analyzer/` | LSP server entry point | `main.rs`, `main_loop.rs`, `global_state.rs` | The "front desk" that talks to your editor |
| `ide/` | IDE feature implementations | `hover.rs`, `goto_definition.rs`, `completion.rs` | The "brains" that provide smart code features |
| `hir/` | High-level IR | `lib.rs`, `semantics.rs` | Simplified view of your code for analysis |
| `hir-def/` | Name resolution | `nameres/`, `body/`, `item_tree.rs` | Figures out what names mean |
| `hir-ty/` | Type inference | `infer/`, `method_resolution.rs` | Figures out types of expressions |
| `hir-expand/` | Macro expansion | `builtin_fn_macro.rs`, `proc_macro.rs` | Expands macros into regular code |
| `syntax/` | Syntax tree | `ast/`, `ptr.rs` | Parse tree structure |
| `parser/` | Parsing | `grammar/`, `lib.rs` | Converts text to tree |
| `base-db/` | Database | `lib.rs`, `input.rs` | Salsa query system |
| `vfs/` | Virtual file system | `lib.rs`, `loader.rs` | In-memory file tracking |
| `project-model/` | Workspace | `workspace.rs`, `cargo_workspace.rs` | Understands Cargo projects |
| `ide-completion/` | Code completion | `completions/`, `render/` | Suggests code as you type |
| `ide-assists/` | Quick fixes | `handlers/` | Code actions and refactorings |
| `ide-diagnostics/` | Error checking | `handlers/` | Shows errors and warnings |
| `proc-macro-api/` | Proc macro client | `msg.rs`, `process.rs` | Talks to proc macro server |
| `proc-macro-srv/` | Proc macro server | `dylib.rs`, `lib.rs` | Runs proc macros |

## Directory Interaction Patterns

```mermaid
graph LR
    subgraph "User Action"
        EDIT[Edit File]
    end

    subgraph "File System Layer"
        VFS_N[vfs-notify]
        VFS[vfs]
    end

    subgraph "Syntax Layer"
        PARSER[parser]
        SYNTAX[syntax]
    end

    subgraph "Semantic Layer"
        HIR[hir-*]
    end

    subgraph "IDE Layer"
        IDE[ide-*]
    end

    subgraph "LSP Layer"
        RA[rust-analyzer]
    end

    EDIT --> VFS_N
    VFS_N --> VFS
    VFS --> PARSER
    PARSER --> SYNTAX
    SYNTAX --> HIR
    HIR --> IDE
    IDE --> RA

    style EDIT fill:#ffebee
    style RA fill:#ff6b6b
```

This folder structure shows how rust-analyzer is organized as a **layered architecture** where each layer has clear responsibilities and dependencies flow downward.
