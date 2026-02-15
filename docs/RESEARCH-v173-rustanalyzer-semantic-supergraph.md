# RESEARCH: v173-rustanalyzer-semantic-supergraph

**Date**: 2026-02-15
**Status**: Research / Pre-Implementation
**Scope**: pt04 crate — rust-analyzer-based deep semantic ingestion for Rust codebases
**Source**: Exploration of https://github.com/rust-lang/rust-analyzer (cloned to CR03/)

---

## Problem Statement

pt01 uses tree-sitter for code parsing. Tree-sitter gives us **syntax** — what the code looks like. For 12 languages, this is the right tradeoff: fast, universal, good enough.

But for Rust specifically, tree-sitter leaves critical semantic information on the table:

| What pt01 extracts (syntax) | What pt01 misses (semantics) |
|---|---|
| Function name: `handle_request` | Resolved types: param is `Arc<dyn Service + Send + Sync>` |
| Calls: `authenticate(req)` | Which impl: `AuthService::authenticate` or `MockAuth::authenticate`? |
| Uses: `HashMap` | Generic args: `HashMap<String, Vec<Box<dyn Error>>>` |
| Implements: `impl T for X` | Trait hierarchy: T requires U requires V |
| Edge: A calls B | Through what path: A calls B via trait method on `Box<dyn T>` |

The dependency graph has edges, but the edges are unlabeled. "A calls B" doesn't tell the LLM whether it's a direct call, a trait method dispatch, a blanket impl, or a closure capture. rust-analyzer knows all of this.

---

## What Is pt04

A new crate: `pt04-rust-semantic-deep-analyzer`. Rust-only. Uses rust-analyzer as a library (not tree-sitter) to produce a **semantic supergraph** that sits on top of pt01's syntax graph.

```
pt01: Fast, 12-language, syntax-level graph
pt04: Deep, Rust-only, semantic-level graph
Both → CozoDB → same HTTP endpoints
```

pt04 doesn't replace pt01. It enriches the graph for Rust code specifically. An LLM querying Parseltongue on a Rust codebase gets both the fast syntax graph AND the deep semantic layer.

---

## rust-analyzer Architecture (What We're Using)

rust-analyzer is organized into layers. pt04 uses three of them:

```
Layer 1: load-cargo
  → Load any Cargo workspace programmatically
  → Returns RootDatabase ready for queries

Layer 2: hir (High-Level Intermediate Representation)
  → Fully resolved types, trait impls, generic bounds
  → Module structure, visibility, imports
  → The semantic model

Layer 3: ide
  → High-level queries: call hierarchy, find references, diagnostics
  → The API surface
```

### Entry Point: Loading a Codebase

```rust
use load_cargo::{load_workspace_at, CargoConfig, LoadCargoConfig};

let (db, vfs, proc_macro_client) = load_workspace_at(
    Path::new("."),
    &CargoConfig::default(),
    &LoadCargoConfig {
        load_out_dirs_from_check: true,
        with_proc_macro_server: ProcMacroServerChoice::Sysroot,
        prefill_caches: true,
    },
    &|msg| println!("{msg}"),
)?;

let host = AnalysisHost::with_database(db);
let analysis = host.analysis();
```

This gives us a fully loaded, type-checked Rust codebase. Every expression has a resolved type. Every method call is resolved to its impl block.

---

## API Surface: What rust-analyzer Exposes

### 1. Type Resolution — `Type<'db>`

The most powerful API. Every expression, binding, parameter, and return type is fully resolved.

```rust
impl<'db> Type<'db> {
    // Type identity
    pub fn is_unit(&self) -> bool
    pub fn is_bool(&self) -> bool
    pub fn is_str(&self) -> bool
    pub fn is_never(&self) -> bool
    pub fn is_fn(&self) -> bool
    pub fn is_closure(&self) -> bool
    pub fn is_reference(&self) -> bool
    pub fn is_mutable_reference(&self) -> bool
    pub fn is_raw_ptr(&self) -> bool
    pub fn is_copy(&self, db: &'db dyn HirDatabase) -> bool

    // Decomposition
    pub fn as_reference(&self) -> Option<(Type<'db>, Mutability)>
    pub fn as_adt(&self) -> Option<Adt>
    pub fn as_callable(&self, db: &'db dyn HirDatabase) -> Option<Callable<'db>>
    pub fn as_closure(&self) -> Option<Closure<'db>>
    pub fn as_dyn_trait(&self) -> Option<Trait>

    // Generic arguments
    pub fn type_arguments(&self) -> impl Iterator<Item = Type<'db>>

    // Trait checking
    pub fn impls_trait(&self, db: &'db dyn HirDatabase, trait_: Trait, args: &[Type<'db>]) -> bool
    pub fn impls_fnonce(&self, db: &'db dyn HirDatabase) -> bool

    // Method resolution — resolves `.method()` to the specific impl
    pub fn iterate_method_candidates<T>(
        &self, db: &'db dyn HirDatabase, scope: &SemanticsScope<'_>,
        name: Option<&Name>,
        callback: impl FnMut(Function) -> Option<T>
    ) -> Option<T>

    // All fields with types
    pub fn fields(&self, db: &'db dyn HirDatabase) -> Vec<(Field, Self)>

    // Autoderef chain
    pub fn autoderef(&self, db: &'db dyn HirDatabase) -> impl Iterator<Item = Type<'db>>

    // Associated types
    pub fn normalize_trait_assoc_type(&self, ...) -> Option<Type<'db>>

    // Iterator/Future support
    pub fn iterator_item(self, db: &'db dyn HirDatabase) -> Option<Type<'db>>
    pub fn future_output(self, db: &'db dyn HirDatabase) -> Option<Type<'db>>

    // Memory layout
    pub fn layout(&self, db: &dyn HirDatabase) -> Result<Layout, LayoutError>
}
```

**What this means for pt04**: Every entity in the graph gets fully resolved type information. Not `fn foo(x: T) -> R` but `fn foo(x: Arc<Mutex<HashMap<String, Vec<u8>>>>) -> Result<Response, Box<dyn Error + Send + Sync>>`. The LLM sees real types.

---

### 2. Call Hierarchy — `incoming_calls()` / `outgoing_calls()`

```rust
impl Analysis {
    pub fn call_hierarchy(
        &self, position: FilePosition, config: &CallHierarchyConfig
    ) -> Cancellable<Option<RangeInfo<Vec<NavigationTarget>>>>

    pub fn incoming_calls(
        &self, config: &CallHierarchyConfig, position: FilePosition
    ) -> Cancellable<Option<Vec<CallItem>>>

    pub fn outgoing_calls(
        &self, config: &CallHierarchyConfig, position: FilePosition
    ) -> Cancellable<Option<Vec<CallItem>>>
}
```

**What this means for pt04**: Edges get type-annotated. Not just "A calls B" but:
- "A calls B::process() via trait Handler on type RequestHandler"
- "A calls closure captured from scope with variables [x: &mut String, y: Arc<Config>]"
- "A calls B through dynamic dispatch on Box<dyn Service>"

---

### 3. Trait & Impl Graph

```rust
impl Trait {
    pub fn direct_supertraits(self, db: &dyn HirDatabase) -> Vec<Trait>
    pub fn all_supertraits(self, db: &dyn HirDatabase) -> Vec<Trait>
    pub fn items_with_supertraits(self, db: &dyn HirDatabase) -> Vec<AssocItem>
}

impl Impl {
    pub fn all_in_crate(db: &dyn HirDatabase, krate: Crate) -> Vec<Impl>
    pub fn all_for_type<'db>(db: &'db dyn HirDatabase, ty: Type<'db>) -> Vec<Impl>
    pub fn all_for_trait(db: &dyn HirDatabase, trait_: Trait) -> Vec<Impl>
    pub fn trait_(self, db: &dyn HirDatabase) -> Option<Trait>
    pub fn self_ty(self, db: &dyn HirDatabase) -> Type<'_>
    pub fn items(self, db: &dyn HirDatabase) -> Vec<AssocItem>
    pub fn is_negative(self, db: &dyn HirDatabase) -> bool
    pub fn is_unsafe(self, db: &dyn HirDatabase) -> bool
}
```

**What this means for pt04**: New edge types:
- `ImplementsTrait { entity, trait_, via_blanket: bool }`
- `SupertraitOf { child_trait, parent_trait }`
- `ImplFor { impl_block, target_type, trait_ }`

pt01 can detect `impl Handler for RequestHandler`. pt04 can tell you that `Handler` requires `Service` which requires `Debug + Send + Sync`, and that there are 7 other types that also implement `Handler`, and that there's a blanket impl `impl<T: Service> Handler for Arc<T>`.

---

### 4. Function Signatures (Deep)

```rust
impl Function {
    pub fn name(self, db: &dyn HirDatabase) -> Name
    pub fn ty(self, db: &dyn HirDatabase) -> Type<'_>
    pub fn ret_type(self, db: &dyn HirDatabase) -> Type<'_>
    pub fn async_ret_type<'db>(self, db: &'db dyn HirDatabase) -> Option<Type<'db>>
    pub fn params_without_self(self, db: &dyn HirDatabase) -> Vec<Param<'_>>
    pub fn method_params(self, db: &dyn HirDatabase) -> Option<Vec<Param<'_>>>
    pub fn has_self_param(self, db: &dyn HirDatabase) -> bool
    pub fn self_param(self, db: &dyn HirDatabase) -> Option<SelfParam>
    pub fn is_unsafe_to_call(&self, db: &dyn HirDatabase) -> bool
}
```

**What this means for pt04**: Entity metadata goes from:
```
name: "handle_request", params: ["req", "state"], return: "impl IntoResponse"
```
to:
```
name: "handle_request"
params: [
  { name: "req", type: "axum::extract::Request<Body>", is_self: false },
  { name: "state", type: "axum::extract::State<Arc<AppState>>", is_self: false }
]
return: "axum::response::Response<Body>"
async: true
unsafe: false
visibility: pub(crate)
generic_params: []
where_clauses: []
```

---

### 5. Generics & Lifetime Bounds

```rust
impl TypeParam {
    pub fn name(self, db: &dyn HirDatabase) -> Name
    pub fn ty(self, db: &dyn HirDatabase) -> Type<'_>
    pub fn default(self, db: &dyn HirDatabase) -> Option<Type<'_>>
    pub fn trait_bounds(self, db: &dyn HirDatabase) -> Vec<Trait>
}

impl LifetimeParam {
    pub fn name(self, db: &dyn HirDatabase) -> Name
}
```

**What this means for pt04**: Generic bounds become edges:
```
fn process<T: Handler + Send + Sync, E: Into<Box<dyn Error>>>(handler: T) -> Result<(), E>
```
produces:
- `TypeParam T bounded_by [Handler, Send, Sync]`
- `TypeParam E bounded_by [Into<Box<dyn Error>>]`

These bounds are invisible to tree-sitter.

---

### 6. Macro Expansion

```rust
impl Analysis {
    pub fn expand_macro(&self, position: FilePosition) -> Cancellable<Option<ExpandedMacro>>
    pub fn view_hir(&self, position: FilePosition) -> Cancellable<String>
    pub fn view_mir(&self, position: FilePosition) -> Cancellable<String>
}

impl Semantics {
    pub fn expand_macro_call(&self, macro_call: &ast::MacroCall) -> Option<InFile<SyntaxNode>>
    pub fn expand_attr_macro(&self, item: &ast::Item) -> Option<ExpandResult<InFile<SyntaxNode>>>
    pub fn expand_derive_macro(&self, attr: &ast::Attr) -> Option<Vec<InFile<SyntaxNode>>>
}
```

**What this means for pt04**: Macros are the biggest hole in tree-sitter parsing. `#[derive(Debug, Clone, Serialize)]` generates code that tree-sitter can't see. proc macros like `#[tokio::main]` transform the entire function. rust-analyzer expands all of these, making the generated code visible to the graph.

A `#[derive(Serialize)]` generates an `impl Serialize for MyType` with method calls to `serializer.serialize_struct()`, `serialize_field()`, etc. pt04 can include these in the dependency graph. pt01 can't even see them.

---

### 7. Closure Capture Analysis

```rust
impl Closure<'db> {
    pub fn captured_items(&self, db: &'db dyn HirDatabase) -> Vec<ClosureCapture<'db>>
    pub fn capture_types(&self, db: &'db dyn HirDatabase) -> Vec<Type<'db>>
    pub fn fn_trait(&self, db: &dyn HirDatabase) -> FnTrait  // Fn, FnMut, FnOnce
}

impl ClosureCapture<'db> {
    pub fn local(&self) -> Local
    pub fn kind(&self) -> CaptureKind  // SharedRef, MutableRef, Move
    pub fn display_place(&self, db: &dyn HirDatabase) -> String
}
```

**What this means for pt04**: Closures create invisible dependencies. A closure that captures `&mut database` creates a dependency on whatever `database` is. pt01 can't see this. pt04 can say:
```
closure at line 45 captures:
  - database: &mut DatabaseConnection (MutableRef)
  - config: Arc<Config> (SharedRef)
  - user_id: String (Move)
implements: FnMut
```

---

### 8. Module Visibility

```rust
impl Module {
    pub fn declarations(self, db: &dyn HirDatabase) -> Vec<ModuleDef>
    pub fn scope(&self, db: &dyn HirDatabase, visible_from: Option<Module>) -> ModuleScope
    pub fn find_path(self, db: &dyn HirDatabase, item: impl Into<ItemInNs>, config: FindPathConfig) -> Option<ModPath>
    pub fn path_to_root(self, db: &dyn HirDatabase) -> Vec<Module>
    pub fn children(self, db: &dyn HirDatabase) -> impl Iterator<Item = Module>
}
```

**What this means for pt04**: Visibility becomes a graph property. "Can module A access function B?" is answerable. This enables:
- Detecting unnecessary `pub` (function is public but only called from same module)
- Detecting visibility violations (using `pub(crate)` when `pub(super)` would suffice)
- Mapping the actual access graph (who CAN call what, not just who DOES call what)

---

### 9. Memory Layout

```rust
impl Analysis {
    pub fn get_recursive_memory_layout(
        &self, position: FilePosition
    ) -> Cancellable<Option<RecursiveMemoryLayout>>
}

impl Type<'db> {
    pub fn layout(&self, db: &dyn HirDatabase) -> Result<Layout, LayoutError>
}
```

**What this means for pt04**: Performance analysis. "This struct is 128 bytes with 40 bytes padding." "This enum variant is 256 bytes because of the largest variant." Feed into SQALE or a custom performance debt metric.

---

### 10. Diagnostics

```rust
impl Analysis {
    pub fn syntax_diagnostics(&self, config: &DiagnosticsConfig, file_id: FileId) -> Cancellable<Vec<Diagnostic>>
    pub fn semantic_diagnostics(&self, config: &DiagnosticsConfig, resolve: AssistResolveStrategy, file_id: FileId) -> Cancellable<Vec<Diagnostic>>
    pub fn full_diagnostics(&self, config: &DiagnosticsConfig, resolve: AssistResolveStrategy, file_id: FileId) -> Cancellable<Vec<Diagnostic>>
}
```

**What this means for pt04**: rust-analyzer finds issues that tree-sitter can't: unused imports, unreachable code, type mismatches, missing trait implementations. These become annotations on graph entities.

---

## pt04 Graph vs pt01 Graph

### Entity Comparison

| Field | pt01 (tree-sitter) | pt04 (rust-analyzer) |
|---|---|---|
| name | `handle_request` | `handle_request` |
| entity_type | `function` | `function` |
| language | `rust` | `rust` |
| params | `["req", "state"]` (names only) | `[{name: "req", type: "Request<Body>"}, {name: "state", type: "State<Arc<AppState>>"}]` |
| return_type | `"impl IntoResponse"` (syntax) | `"Response<Body>"` (resolved) |
| generic_params | `["T"]` (names only) | `[{name: "T", bounds: ["Handler", "Send", "Sync"]}]` |
| async | unknown | `true` |
| unsafe | unknown | `false` |
| visibility | unknown | `pub(crate)` |
| macro_generated | unknown | `false` |
| token_count | from `bpe-openai` | from `bpe-openai` |

### Edge Comparison

| Edge Type | pt01 | pt04 |
|---|---|---|
| Calls | `A → B` | `A → B via Trait T on Type X (dynamic dispatch)` |
| Uses | `A uses HashMap` | `A uses HashMap<String, Vec<Result<T, E>>>` |
| Implements | `impl T for X` | `impl T for X` + supertraits + blanket impls |
| ClosureCaptures | not detected | `closure captures [db: &mut Conn, config: Arc<Config>]` |
| BoundedBy | not detected | `T: Handler + Send + Sync` |
| VisibleFrom | not detected | `pub(crate)` — who CAN access this |
| DerivedFrom | not detected | `#[derive(Serialize)]` generated `impl Serialize` |
| SupertraitOf | not detected | `Handler: Service + Debug` |

### Coverage Comparison

| Metric | pt01 | pt04 |
|---|---|---|
| Languages | 12 | Rust only |
| Coverage per file | ~70% (misses macros, gaps) | ~99% (macros expanded) |
| Type accuracy | Syntax only | Fully resolved |
| Speed | ~ms per file | ~seconds per crate (full type check) |
| Dependencies | tree-sitter grammars | rust-analyzer + Cargo |

---

## CozoDB Schema Extensions

pt04 would add new relations alongside existing ones:

```
# Resolved type information per entity
SemanticTypes {
    ISGL1_key: String =>
    resolved_return_type: String,
    resolved_params: String,        // JSON array of {name, type}
    generic_bounds: String,          // JSON array of {param, bounds[]}
    visibility: String,              // pub, pub(crate), pub(super), private
    is_async: Bool,
    is_unsafe: Bool,
    is_const: Bool,
    macro_generated: Bool,
}

# Trait implementation graph
TraitImpls {
    impl_key: String =>
    trait_name: String,
    self_type: String,
    is_blanket: Bool,
    is_negative: Bool,
}

# Supertrait hierarchy
SupertraitEdges {
    child_trait: String,
    parent_trait: String =>
}

# Closure capture information
ClosureCaptures {
    closure_key: String,
    captured_var: String =>
    capture_kind: String,           // SharedRef, MutableRef, Move
    captured_type: String,
    fn_trait: String,               // Fn, FnMut, FnOnce
}

# Typed call edges (superset of DependencyEdges)
TypedCallEdges {
    from_key: String,
    to_key: String =>
    call_kind: String,              // Direct, TraitMethod, DynDispatch, ClosureInvoke
    via_trait: String?,
    receiver_type: String?,
}

# Memory layout
TypeLayouts {
    ISGL1_key: String =>
    size_bytes: Int,
    alignment: Int,
    padding_bytes: Int,
}
```

---

## Implementation Plan

### Dependencies

```toml
[dependencies]
# rust-analyzer crates (from crates.io or git)
ide = { git = "https://github.com/rust-lang/rust-analyzer" }
hir = { git = "https://github.com/rust-lang/rust-analyzer" }
load-cargo = { git = "https://github.com/rust-lang/rust-analyzer" }
ide-db = { git = "https://github.com/rust-lang/rust-analyzer" }
vfs = { git = "https://github.com/rust-lang/rust-analyzer" }

# Our crates
parseltongue-core = { path = "../parseltongue-core" }
```

### Workflow

```
1. Load Cargo workspace via load_workspace_at()
2. Get AnalysisHost + Analysis snapshot
3. Iterate all crates → modules → declarations
4. For each entity:
   a. Extract resolved types (params, return, generics, bounds)
   b. Extract visibility, async/unsafe/const flags
   c. Check if macro-generated
   d. Compute memory layout (optional)
5. For each function body:
   a. Extract outgoing calls with type context
   b. Extract closure captures
   c. Extract trait method dispatch info
6. For each trait:
   a. Extract supertrait hierarchy
   b. Find all implementations (including blanket)
7. For each impl block:
   a. Link to trait and self_type
   b. Extract associated items
8. Write all to CozoDB (SemanticTypes, TraitImpls, TypedCallEdges, etc.)
```

### Performance Concern

rust-analyzer does full type checking. This is slower than tree-sitter parsing:
- tree-sitter: ~1ms per file
- rust-analyzer: ~1-5s per crate (full type check with macro expansion)

For a 50-crate workspace, pt04 might take 1-3 minutes vs pt01's seconds. This is acceptable because:
- pt04 runs once at ingestion, not on every query
- The semantic depth justifies the cost
- Users opt into pt04 specifically for deep Rust analysis

### CLI Integration

```bash
# Syntax graph (fast, all languages)
parseltongue pt01-folder-to-cozodb-streamer .

# Semantic supergraph (Rust only, deep)
parseltongue pt04-rust-semantic-deep-analyzer .

# Both (full graph)
parseltongue pt01-folder-to-cozodb-streamer . && parseltongue pt04-rust-semantic-deep-analyzer .
```

pt04 writes to the same CozoDB database as pt01. The semantic data supplements the syntax data. HTTP endpoints automatically see both layers.

---

## What This Enables for LLMs

### Before (pt01 only)

LLM asks: "What does `handle_request` depend on?"

```json
{
  "callees": [
    {"to_key": "rust:fn:authenticate:...", "edge_type": "Calls"},
    {"to_key": "rust:fn:parse_body:...", "edge_type": "Calls"},
    {"to_key": "rust:fn:db_query:...", "edge_type": "Calls"}
  ]
}
```

LLM knows the names but not the types, not the dispatch mechanism, not the trait hierarchy.

### After (pt01 + pt04)

```json
{
  "callees": [
    {
      "to_key": "rust:fn:authenticate:...",
      "edge_type": "Calls",
      "call_kind": "TraitMethod",
      "via_trait": "AuthService",
      "receiver_type": "Arc<dyn AuthService + Send + Sync>",
      "param_types": ["&Request<Body>"],
      "return_type": "Result<User, AuthError>"
    },
    {
      "to_key": "rust:fn:parse_body:...",
      "edge_type": "Calls",
      "call_kind": "Direct",
      "param_types": ["Request<Body>"],
      "return_type": "Result<serde_json::Value, ParseError>"
    },
    {
      "to_key": "rust:fn:db_query:...",
      "edge_type": "Calls",
      "call_kind": "TraitMethod",
      "via_trait": "DatabasePool",
      "receiver_type": "&Pool<Postgres>",
      "param_types": ["&str", "&[&(dyn ToSql + Sync)]"],
      "return_type": "Result<Vec<Row>, sqlx::Error>"
    }
  ],
  "closure_captures": [
    {
      "closure_at": "line 52",
      "captures": [
        {"var": "db_pool", "type": "&Pool<Postgres>", "kind": "SharedRef"},
        {"var": "user", "type": "User", "kind": "Move"}
      ],
      "fn_trait": "FnOnce"
    }
  ]
}
```

The LLM now knows:
- `authenticate` is called through dynamic dispatch on a trait object
- `db_query` uses sqlx's PostgreSQL pool
- There's a closure capturing the DB pool by shared reference and moving the user
- The return type is `Result<Vec<Row>, sqlx::Error>` not just "some result"

This is the difference between "A calls B" and "A calls B.process() through Arc<dyn Service> with arguments of type Request<Body>, returning Result<Response, Box<dyn Error + Send + Sync>>."

---

## Open Questions

1. **rust-analyzer as a library dependency**: rust-analyzer publishes crates to crates.io but they're tightly coupled and version-locked. Pinning to a specific commit may be necessary. Build times will be significant (~5-10 min for first compile).

2. **proc-macro support**: Loading proc macros requires a running proc-macro server. Do we ship one? Use the sysroot one? Skip proc macros for faster ingestion?

3. **Incremental updates**: rust-analyzer uses Salsa for incremental computation. Can we reuse this for file watcher updates, or do we re-analyze the full crate on every change?

4. **CozoDB schema merging**: pt01 and pt04 both write to the same database. How do we handle entity key conflicts? Use the same ISGL1 keys (pt04 enriches pt01 entities) or separate key spaces?

5. **Binary size**: rust-analyzer is large. Will pt04 significantly increase the Parseltongue binary? Should it be a separate binary?

6. **Non-Cargo Rust**: Some Rust projects don't use Cargo (embedded, kernel modules). Do we support these? rust-analyzer has `rust-project.json` for non-Cargo projects.

---

## Summary

| Decision | Choice | Rationale |
|---|---|---|
| Approach | rust-analyzer as library | Full semantic analysis, not just syntax |
| Scope | Rust only | rust-analyzer only supports Rust |
| Relationship to pt01 | Supplements, doesn't replace | pt01 handles 12 languages; pt04 adds depth for Rust |
| Key APIs | Type<'db>, Impl, Trait, Semantics, load_workspace_at | Cover types, traits, calls, macros, visibility |
| New CozoDB relations | 6 (SemanticTypes, TraitImpls, SupertraitEdges, ClosureCaptures, TypedCallEdges, TypeLayouts) | Capture semantic information pt01 can't |
| Performance | 1-3 min for 50-crate workspace | Acceptable for one-time deep analysis |
| Binary | Separate from pt01 | rust-analyzer dependency is large |

---

**Last Updated**: 2026-02-15
**Key Insight**: tree-sitter gives you the skeleton. rust-analyzer gives you the nervous system.
**Next Step**: Prototype loading a small Cargo workspace and extracting typed call edges.
