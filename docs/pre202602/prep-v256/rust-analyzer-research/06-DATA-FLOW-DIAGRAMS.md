# Rust-Analyzer Data Flow Diagrams

**Visual Guide to How Data Transforms Through the System**

## Complete Data Pipeline

```mermaid
flowchart TD
    subgraph "Input Layer"
        FS[File System<br/>Rust source files]
        LSP_IN[LSP Client<br/>Editor requests]
    end

    subgraph "VFS Layer"
        WATCH[File Watcher<br/>vfs-notify]
        VFS[Virtual File System<br/>In-memory files]
        MEMDOCS[MemDocs<br/>LSP overrides]
    end

    subgraph "Syntax Layer"
        LEX[Lexer<br/>Text → Tokens]
        PARSER[Parser<br/>Tokens → Events]
        GREEN[GreenNode<br/>Immutable CST]
        SYNTAX[SyntaxNode<br/>RedNode wrapper]
        AST[AST<br/>Typed nodes]
    end

    subgraph "Macro Layer"
        TT[Token Trees<br/>tt crate]
        MBE[Macro Expansion<br/>mbe crate]
        PROC[Proc Macros<br/>IPC to server]
    end

    subgraph "HIR Layer"
        ITEM_TREE[ItemTree<br/>File-level items]
        DEF_MAP[DefMap<br/>Module tree]
        NAME_RES[Name Resolution<br/>Path → DefId]
        BODY[Body<br/>Function HIR]
        TYPE_INF[Type Inference<br/>Expression types]
    end

    subgraph "IDE Layer"
        ANALYSIS[Analysis<br/>High-level API]
        FEATURES[IDE Features<br/>Completion, Hover, etc.]
    end

    subgraph "Output Layer"
        LSP_OUT[LSP Responses<br/>To editor]
        DIAG[Diagnostics<br/>Errors & warnings]
    end

    FS --> WATCH
    WATCH --> VFS
    LSP_IN --> MEMDOCS
    MEMDOCS --> VFS

    VFS --> LEX
    LEX --> PARSER
    PARSER --> GREEN
    GREEN --> SYNTAX
    SYNTAX --> AST

    AST --> ITEM_TREE
    AST --> TT

    TT --> MBE
    MBE --> PROC
    PROC --> TT

    TT --> ITEM_TREE

    ITEM_TREE --> DEF_MAP
    DEF_MAP --> NAME_RES
    NAME_RES --> BODY
    BODY --> TYPE_INF

    TYPE_INF --> ANALYSIS
    ITEM_TREE --> ANALYSIS
    DEF_MAP --> ANALYSIS

    ANALYSIS --> FEATURES

    FEATURES --> LSP_OUT
    TYPE_INF --> DIAG
    DIAG --> LSP_OUT

    style FS fill:#e8f5e9
    style LSP_IN fill:#e1f5ff
    style VFS fill:#fff3e0
    style SYNTAX fill:#f3e5f5
    style TYPE_INF fill:#ffe66d
    style FEATURES fill:#4ecdc4
    style LSP_OUT fill:#e1f5ff
```

## Text to Syntax Tree Transformation

```mermaid
flowchart LR
    TEXT["Source Text<br/>fn main() { }"]

    TEXT -->|Lexing| TOKENS["Token Stream<br/>[FN_KW, IDENT(main),<br/>L_PAREN, R_PAREN,<br/>L_CURLY, R_CURLY]"]

    TOKENS -->|Parsing| EVENTS["Parser Events<br/>[Start(FN),<br/>Token(FN_KW),<br/>Start(NAME),<br/>...]"]

    EVENTS -->|Build Tree| GREEN["GreenNode Tree<br/>(Immutable)"]

    GREEN -->|Wrap| RED["SyntaxNode<br/>(with parent pointers)"]

    RED -->|Type Cast| AST_FN["ast::Fn<br/>{name, params, body}"]

    style TEXT fill:#f0f0f0
    style TOKENS fill:#ffebee
    style EVENTS fill:#e1f5ff
    style GREEN fill:#c8e6c9
    style RED fill:#ffccbc
    style AST_FN fill:#bbdefb
```

## File Change to Re-Analysis

```mermaid
sequenceDiagram
    participant User
    participant Editor as LSP Client
    participant VFS as Virtual FS
    participant Salsa as Salsa DB
    participant Parser
    participant HIR as HIR Layers
    participant IDE as IDE Features

    User->>Editor: Edit file
    Editor->>VFS: DidChangeTextDocument

    VFS->>VFS: Update file content
    VFS->>VFS: Compute file diff

    VFS->>Salsa: file_text changed
    Salsa->>Salsa: Invalidate:<br/>- parse(file)<br/>- item_tree(file)<br/>- def_map(module)<br/>- infer(affected fns)

    Editor->>IDE: textDocument/completion

    IDE->>Salsa: Get parse(file)
    Salsa->>Parser: Re-parse (cached invalid)
    Parser-->>Salsa: New SyntaxTree
    Salsa-->>IDE: SyntaxTree

    IDE->>Salsa: Get infer(function)
    Salsa->>HIR: Re-infer (cached invalid)
    HIR-->>Salsa: InferenceResult
    Salsa-->>IDE: Type information

    IDE->>IDE: Generate completions
    IDE-->>Editor: CompletionList
```

## Workspace to CrateGraph Transformation

```mermaid
flowchart TD
    CARGO_TOML[Cargo.toml<br/>Package manifest]

    CARGO_TOML -->|cargo metadata| METADATA["CargoWorkspace<br/>{packages, targets,<br/>dependencies}"]

    METADATA --> BUILD["Build Scripts<br/>cargo build<br/>--message-format=json"]

    BUILD --> BUILD_DATA["BuildDataResult<br/>{OUT_DIR, cfgs,<br/>env vars}"]

    METADATA --> PROC_MACRO["Compile Proc Macros<br/>Build dylibs"]

    PROC_MACRO --> PROC_CLIENTS["ProcMacroClient[]<br/>IPC handles"]

    METADATA --> CRATE_GRAPH_BUILD[Build CrateGraph]
    BUILD_DATA --> CRATE_GRAPH_BUILD
    PROC_CLIENTS --> CRATE_GRAPH_BUILD

    CRATE_GRAPH_BUILD --> CRATE_GRAPH["CrateGraph<br/>{crates, dependencies,<br/>proc_macros}"]

    CRATE_GRAPH --> SALSA_APPLY[Apply to Salsa DB]

    SALSA_APPLY --> ANALYSIS_READY[Analysis Ready]

    style CARGO_TOML fill:#fff3e0
    style METADATA fill:#e1f5ff
    style BUILD_DATA fill:#c8e6c9
    style PROC_CLIENTS fill:#ffccbc
    style CRATE_GRAPH fill:#ffe66d
    style ANALYSIS_READY fill:#4ecdc4
```

## Name Resolution Data Flow

```mermaid
flowchart TD
    SOURCE[Source File]
    SOURCE --> PARSE[Parse]
    PARSE --> AST[AST]

    AST --> ITEM_TREE[ItemTree<br/>Catalog of items]

    ITEM_TREE --> COLLECT[DefCollector]

    COLLECT --> RESOLVE_IMPORTS[Resolve use statements]
    RESOLVE_IMPORTS --> RESOLVE_MACROS[Expand macros]
    RESOLVE_MACROS --> RESOLVE_ITEMS[Resolve items]

    RESOLVE_ITEMS --> DEF_MAP["DefMap<br/>{modules, scope,<br/>prelude, externs}"]

    DEF_MAP --> PATH_RESOLVE["Path Resolution<br/>foo::bar::Baz<br/>↓<br/>DefId"]

    PATH_RESOLVE --> DEFINITION["Definition<br/>{kind, module,<br/>visibility}"]

    style SOURCE fill:#f0f0f0
    style ITEM_TREE fill:#e1f5ff
    style DEF_MAP fill:#ffe66d
    style DEFINITION fill:#4ecdc4
```

## Type Inference Data Flow

```mermaid
flowchart TD
    FN_BODY[Function Body AST]

    FN_BODY --> LOWER["Lower to HIR<br/>{exprs, pats,<br/>statements}"]

    LOWER --> INFER_CTX[Create InferenceContext]

    INFER_CTX --> COLLECT_CONSTRAINTS["Collect Constraints<br/>- Literals → concrete types<br/>- Function calls → signatures<br/>- Method calls → trait impls"]

    COLLECT_CONSTRAINTS --> UNIFY["Unification<br/>Solve type variables"]

    UNIFY --> COERCION["Apply Coercions<br/>- & to &mut<br/>- T to dyn Trait<br/>- Array to slice"]

    COERCION --> INFER_RESULT["InferenceResult<br/>{type_of_expr,<br/>type_of_pat,<br/>diagnostics}"]

    INFER_RESULT --> METHOD_RES["Method Resolution<br/>Find impl blocks"]

    METHOD_RES --> FINAL_TYPES[Final Type Information]

    style FN_BODY fill:#f0f0f0
    style INFER_CTX fill:#e1f5ff
    style UNIFY fill:#ffe66d
    style INFER_RESULT fill:#4ecdc4
    style FINAL_TYPES fill:#95e1d3
```

## Completion Data Generation

```mermaid
flowchart LR
    CURSOR[Cursor Position]

    CURSOR --> CONTEXT["CompletionContext<br/>{token, semantic_scope,<br/>expected_type}"]

    CONTEXT --> DETERMINE{Completion Kind}

    DETERMINE -->|foo.| DOT_COMP[Dot Completion<br/>Methods & fields]
    DETERMINE -->|use foo::| PATH_COMP[Path Completion<br/>Modules & items]
    DETERMINE -->|fn | KEYWORD_COMP[Keyword Completion]
    DETERMINE -->|"let x = | EXPR_COMP[Expression Completion]

    DOT_COMP --> RECEIVER_TY[Get receiver type]
    RECEIVER_TY --> METHODS[List methods<br/>via trait impls]
    METHODS --> FIELDS[List fields]

    PATH_COMP --> SCOPE[Get current scope]
    SCOPE --> VISIBLE[List visible items]

    KEYWORD_COMP --> KEYWORDS[Rust keywords]

    EXPR_COMP --> EXPECTED[Expected type]
    EXPECTED --> COMPATIBLE[Find compatible items]

    METHODS --> ITEMS[CompletionItem[]]
    FIELDS --> ITEMS
    VISIBLE --> ITEMS
    KEYWORDS --> ITEMS
    COMPATIBLE --> ITEMS

    ITEMS --> SCORE[Score & Rank<br/>- Relevance<br/>- Fuzzy match<br/>- Deprecation]

    SCORE --> SNIPPETS[Add Snippets<br/>- Function calls<br/>- Struct literals]

    SNIPPETS --> FINAL[Final CompletionList]

    style CURSOR fill:#f0f0f0
    style CONTEXT fill:#e1f5ff
    style ITEMS fill:#ffe66d
    style FINAL fill:#4ecdc4
```

## Diagnostic Generation Flow

```mermaid
flowchart TD
    subgraph "Syntax Diagnostics"
        PARSE_ERRORS[Parser Errors<br/>Invalid syntax]
    end

    subgraph "Name Resolution Diagnostics"
        UNRESOLVED_NAME[Unresolved Name]
        UNRESOLVED_IMPORT[Unresolved Import]
        UNRESOLVED_MACRO[Unresolved Macro]
    end

    subgraph "Type Diagnostics"
        TYPE_MISMATCH[Type Mismatch]
        MISSING_FIELDS[Missing Fields]
        PRIVATE_FIELD[Private Field Access]
        TRAIT_BOUNDS[Unsatisfied Trait Bounds]
    end

    subgraph "HIR Diagnostics"
        UNUSED_VAR[Unused Variable]
        UNUSED_IMPORT[Unused Import]
        MISSING_MATCH_ARM[Missing Match Arm]
    end

    subgraph "Flycheck Diagnostics"
        CARGO_CHECK[cargo check output<br/>Compiler diagnostics]
        CLIPPY[clippy output<br/>Lint warnings]
    end

    PARSE_ERRORS --> COLLECT[DiagnosticCollection]
    UNRESOLVED_NAME --> COLLECT
    UNRESOLVED_IMPORT --> COLLECT
    UNRESOLVED_MACRO --> COLLECT
    TYPE_MISMATCH --> COLLECT
    MISSING_FIELDS --> COLLECT
    PRIVATE_FIELD --> COLLECT
    TRAIT_BOUNDS --> COLLECT
    UNUSED_VAR --> COLLECT
    UNUSED_IMPORT --> COLLECT
    MISSING_MATCH_ARM --> COLLECT
    CARGO_CHECK --> COLLECT
    CLIPPY --> COLLECT

    COLLECT --> DEDUPE[Deduplicate<br/>Priority: Flycheck > Native]

    DEDUPE --> FIX_GEN["Generate Fixes<br/>- Add missing field<br/>- Import item<br/>- Remove unused"]

    FIX_GEN --> TO_LSP[Convert to LSP Diagnostic]

    TO_LSP --> PUBLISH[publish_diagnostics]

    PUBLISH --> CLIENT[LSP Client]

    style COLLECT fill:#ff6b6b
    style FIX_GEN fill:#4ecdc4
    style CLIENT fill:#e1f5ff
```

## Token to Semantic Token Mapping

```mermaid
flowchart LR
    TOKEN[SyntaxToken<br/>from parse tree]

    TOKEN --> CLASSIFY{Classify Token}

    CLASSIFY -->|Identifier| RESOLVE[Resolve semantics<br/>via HIR]
    CLASSIFY -->|Keyword| KEYWORD_TYPE[Token: keyword]
    CLASSIFY -->|Literal| LITERAL_TYPE[Token: number/string]
    CLASSIFY -->|Comment| COMMENT_TYPE[Token: comment]

    RESOLVE --> DEF_KIND{Definition Kind}

    DEF_KIND -->|Function| FN_TOKEN["Token: function<br/>Modifiers: async, unsafe"]
    DEF_KIND -->|Parameter| PARAM_TOKEN["Token: parameter<br/>Modifiers: mutable"]
    DEF_KIND -->|Local| LOCAL_TOKEN["Token: variable<br/>Modifiers: mutable"]
    DEF_KIND -->|Field| FIELD_TOKEN["Token: property"]
    DEF_KIND -->|Type| TYPE_TOKEN["Token: type/struct/enum"]
    DEF_KIND -->|Trait| TRAIT_TOKEN["Token: interface"]
    DEF_KIND -->|Macro| MACRO_TOKEN["Token: macro"]

    KEYWORD_TYPE --> SEMANTIC[SemanticToken]
    LITERAL_TYPE --> SEMANTIC
    COMMENT_TYPE --> SEMANTIC
    FN_TOKEN --> SEMANTIC
    PARAM_TOKEN --> SEMANTIC
    LOCAL_TOKEN --> SEMANTIC
    FIELD_TOKEN --> SEMANTIC
    TYPE_TOKEN --> SEMANTIC
    TRAIT_TOKEN --> SEMANTIC
    MACRO_TOKEN --> SEMANTIC

    SEMANTIC --> ENCODE[Delta Encode<br/>Position offsets]

    ENCODE --> LSP_TOKENS[LSP SemanticTokens]

    style TOKEN fill:#f0f0f0
    style RESOLVE fill:#ffe66d
    style SEMANTIC fill:#4ecdc4
    style LSP_TOKENS fill:#e1f5ff
```

## Hover Information Assembly

```mermaid
flowchart TD
    HOVER_POS[Hover Position]

    HOVER_POS --> FIND_TOKEN[Find token at position]
    FIND_TOKEN --> TOKEN[SyntaxToken]

    TOKEN --> RESOLVE[Resolve to definition<br/>via HIR]

    RESOLVE --> DEF[Definition]

    DEF --> GATHER_INFO[Gather Information]

    GATHER_INFO --> TYPE_INFO[Type Information<br/>Display type signature]
    GATHER_INFO --> DOC_COMMENT[Documentation<br/>Parse doc comments]
    GATHER_INFO --> ATTRS[Attributes<br/>deprecated, etc.]
    GATHER_INFO --> VALUE_INFO[Value Information<br/>const values]

    TYPE_INFO --> FORMAT[Format as Markdown]
    DOC_COMMENT --> FORMAT
    ATTRS --> FORMAT
    VALUE_INFO --> FORMAT

    FORMAT --> ACTIONS[Generate Actions]

    ACTIONS --> GOTO_ACTION["goto_type_action<br/>Go to Type Definition"]
    ACTIONS --> RUNNABLE_ACTION["runnable_action<br/>Run/Debug buttons"]

    GOTO_ACTION --> HOVER_RESULT[HoverResult]
    RUNNABLE_ACTION --> HOVER_RESULT

    HOVER_RESULT --> TO_LSP[Convert to LSP Hover]

    TO_LSP --> LSP_HOVER["lsp_types::Hover<br/>{contents, range, actions}"]

    style HOVER_POS fill:#f0f0f0
    style GATHER_INFO fill:#ffe66d
    style FORMAT fill:#4ecdc4
    style LSP_HOVER fill:#e1f5ff
```

## Inlay Hints Data Flow

```mermaid
flowchart TD
    FILE_RANGE[File + Range]

    FILE_RANGE --> INFER[Run type inference<br/>for range]

    INFER --> TRAVERSE[Traverse syntax tree<br/>in range]

    TRAVERSE --> EXPR{Expression Type}

    EXPR -->|let stmt| TYPE_HINT["Type Hint<br/>let x: u32 = "]
    EXPR -->|fn call| PARAM_HINT["Parameter Hint<br/>foo(↓name: value)"]
    EXPR -->|method chain| CHAIN_HINT["Chaining Hint<br/>.map()↓: Iterator"]
    EXPR -->|closure| CLOSURE_HINT["Closure Hint<br/>|x|↓: i32 { }"]

    TYPE_HINT --> SHOULD_SHOW{Should show?}
    PARAM_HINT --> SHOULD_SHOW
    CHAIN_HINT --> SHOULD_SHOW
    CLOSURE_HINT --> SHOULD_SHOW

    SHOULD_SHOW -->|Type obvious| SKIP
    SHOULD_SHOW -->|Config disabled| SKIP
    SHOULD_SHOW -->|Yes| CREATE_HINT

    CREATE_HINT[Create InlayHint]

    CREATE_HINT --> HINTS[Vec<InlayHint>]

    HINTS --> TO_LSP[Convert to LSP format]

    TO_LSP --> LSP_HINTS["lsp_types::InlayHint[]<br/>{position, label, kind}"]

    SKIP[Don't create hint]

    style FILE_RANGE fill:#f0f0f0
    style INFER fill:#ffe66d
    style CREATE_HINT fill:#4ecdc4
    style LSP_HINTS fill:#e1f5ff
```

## Memory Layout of Key Structures

```mermaid
graph TB
    subgraph "GlobalState (Stack/Arc)"
        GS[GlobalState]
        GS --> SENDER[sender: Sender]
        GS --> CONFIG[config: Arc<Config>]
        GS --> ANALYSIS[analysis_host: AnalysisHost]
        GS --> VFS_ARC[vfs: Arc<RwLock<Vfs>>]
        GS --> WORKSPACES[workspaces: Arc<Vec<...>>]
    end

    subgraph "Salsa Database (Heap)"
        ANALYSIS --> DB[RootDatabase]
        DB --> FILE_TEXT[file_text: HashMap<FileId, Arc<String>>]
        DB --> PARSE_MEMO[parse: Memoized results]
        DB --> INFER_MEMO[infer: Memoized results]
    end

    subgraph "VFS (Shared)"
        VFS_ARC --> VFS_DATA[Vfs Data]
        VFS_DATA --> FILES[files: HashMap<FileId, Vec<u8>>]
        VFS_DATA --> PATHS[paths: HashMap<VfsPath, FileId>]
    end

    subgraph "Snapshot (Cheap Clone)"
        GS -->|snapshot| SNAP[GlobalStateSnapshot]
        SNAP --> CONFIG_CLONE[config: Arc clone]
        SNAP --> ANALYSIS_SNAP[analysis: Analysis snapshot]
        SNAP --> VFS_CLONE[vfs: Arc clone]
    end

    style GS fill:#ff6b6b
    style DB fill:#ffe66d
    style VFS_DATA fill:#4ecdc4
    style SNAP fill:#95e1d3
```

## Data Transformation Layers Summary

```mermaid
flowchart TD
    L1["Layer 1: Raw Text<br/>String, &str"]
    L2["Layer 2: Tokens<br/>SyntaxKind, TextRange"]
    L3["Layer 3: Syntax Tree<br/>SyntaxNode, GreenNode"]
    L4["Layer 4: AST<br/>Typed nodes: ast::Fn, ast::Expr"]
    L5["Layer 5: Token Trees<br/>tt::TokenTree (for macros)"]
    L6["Layer 6: ItemTree<br/>Per-file item catalog"]
    L7["Layer 7: DefMap<br/>Module structure, name resolution"]
    L8["Layer 8: HIR Body<br/>Expression & pattern HIR"]
    L9["Layer 9: Types<br/>Ty, TraitRef, InferenceResult"]
    L10["Layer 10: IDE Data<br/>CompletionItem, Diagnostic, etc."]
    L11["Layer 11: LSP Messages<br/>JSON-RPC over stdio"]

    L1 -->|Lexing| L2
    L2 -->|Parsing| L3
    L3 -->|Wrapping| L4
    L4 -->|Macro input| L5
    L4 -->|Lowering| L6
    L6 -->|Collection| L7
    L7 -->|Lowering| L8
    L8 -->|Inference| L9
    L9 -->|Feature impl| L10
    L10 -->|Serialization| L11

    style L1 fill:#f0f0f0
    style L5 fill:#ffccbc
    style L7 fill:#ffe66d
    style L9 fill:#fff4a3
    style L10 fill:#4ecdc4
    style L11 fill:#e1f5ff
```

## Key Takeaways

### Data Flow Principles

1. **Layered Transformations**: Each layer produces a higher-level representation
2. **Immutability**: Most data structures are immutable (GreenNode, Arc<T>)
3. **Memoization**: Salsa caches expensive computations
4. **Incremental**: Only re-compute what changed
5. **Shared Ownership**: Arc enables cheap cloning for snapshots

### Transformation Characteristics

| From | To | Crate | Incremental? | Cached? |
|------|----|----|-------------|---------|
| Text | Tokens | parser | Yes (reparsing) | No |
| Tokens | CST | parser | Yes | Yes (parse query) |
| CST | AST | syntax | N/A (wrapping) | N/A |
| AST | ItemTree | hir-def | Yes (per file) | Yes |
| ItemTree | DefMap | hir-def | Yes (per module) | Yes |
| AST | Body | hir-def | Yes (per fn) | Yes |
| Body | Types | hir-ty | Yes (per fn) | Yes |
| HIR | IDE Data | ide-* | Depends | Partial |

### Memory Patterns

- **Interning**: Small strings (SmolStr), paths stored once
- **Arena allocation**: Syntax nodes, HIR elements use arena
- **Reference counting**: Arc<T> for shared immutable data
- **Copy-on-write**: VFS tracks changes, applies in batch
