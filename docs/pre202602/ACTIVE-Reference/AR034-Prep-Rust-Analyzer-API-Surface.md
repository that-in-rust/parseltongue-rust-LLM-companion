# Prep Doc: rust-analyzer API Surface for Parseltongue v2.0.0

**Date**: 2026-02-16
**Crate**: `rust-llm-03-rust-analyzer` (planned)
**Purpose**: Deep semantic analysis of Rust code via rust-analyzer's `ra_hir` / `ra_ide` internal APIs
**Prerequisite**: Read `Prep-Doc-V200.md` for full v2.0.0 architecture context

---

## 1. rust-analyzer Architecture Overview

rust-analyzer is structured as a set of libraries for analyzing Rust code. It is **not** a monolithic binary --- it is a layered compiler frontend designed for IDE consumption. The layers that matter for us:

### 1.1 Layer Diagram

```
+------------------------------------------------------+
|                    ra_ap_ide                          |  <-- IDE features (completions, goto-def, hover)
|                                                      |      Returns Strings, offsets, text ranges
|   API boundary: AnalysisHost / Analysis              |      POD types, serializable, editor terminology
+------------------------------------------------------+
|                    ra_ap_hir                          |  <-- THIS IS OUR API
|                                                      |      High-level OO access to resolved Rust code
|   API boundary: Function, Impl, Type, Struct, Enum,  |      Each type is self-contained (knows parents)
|   Trait, Closure, Module, Crate                      |      Static, fully resolved view of code
+------------------------------------------------------+
|              ra_ap_hir_ty / ra_ap_hir_def            |  <-- Internal compiler logic
|                                                      |      ECS-style, raw IDs, arenas
|   Type inference, method resolution, name resolution |      Deeply integrated with salsa + chalk
+------------------------------------------------------+
|              ra_ap_syntax (rowan)                     |  <-- Lossless syntax tree (CST)
|                                                      |      Error-resilient parsing
|   Like tree-sitter but with full fidelity            |      Whitespace/comments preserved
+------------------------------------------------------+
|              salsa                                    |  <-- Incremental computation engine
|                                                      |      Key-value store with derived queries
|   Dependency tracking, early cutoff optimization     |      Global version number, durability system
+------------------------------------------------------+
```

### 1.2 The hir Crate --- Our Primary API

The top-level hir crate is the **API boundary**. If you think about "using rust-analyzer as a library," hir is the facade you talk to. It wraps the ECS-style internal API into an OO-flavored API where each type is self-contained and knows its parents and full context.

**Key architectural invariant**: hir provides a **static, fully resolved view** of the code. While internal hir_* crates compute things, hir from the outside looks like an inert data structure. Every method takes a `&dyn HirDatabase` argument (the salsa database) which provides the computation context.

**Key types**:
- `Semantics` --- primary API to bridge syntax trees to HIR. Goes from syntax node to semantic meaning.
- `SemanticsScope` --- the set of visible names at a particular program point.
- `Crate` --- represents a single crate in the crate graph.

### 1.3 The ide Crate --- Higher-Level Alternative

The ide crate builds on hir to provide IDE features. It uses editor terminology (offsets, string labels) rather than compiler terminology (definitions, types). For our use case, we want hir directly because we need the semantic model, not the IDE presentation.

**Key types**:
- `AnalysisHost` --- mutable state to which you apply_change transactionally.
- `Analysis` --- immutable snapshot of the world state. Main entry point for queries.

The separation exists because changes are applied "uniquely" (single AnalysisHost), but you can fork an Analysis and send it to another thread. There is only one AnalysisHost, but there may be several equivalent Analysis instances.

### 1.4 The Salsa Incremental Computation Engine

rust-analyzer uses salsa for incremental, on-demand computation. Think of it as a key-value store that can compute derived values. Every query is K -> V; queries come in two varieties:
- **Inputs**: base facts supplied by the client (file contents, crate graph)
- **Functions**: pure functions that transform inputs into derived data

When an input changes, salsa does **O(1)** work: increments a global version number. Re-validation happens lazily when queries are actually run. **Early cutoff optimization**: if an intermediate value hasn't changed despite its inputs changing (e.g., adding whitespace doesn't change the AST), downstream queries are not recomputed.

**Durability system**: Inputs are categorized by how likely they are to change. Standard library crates get Durability::HIGH so that editing src/lib.rs doesn't trigger re-checking 300ms of std-related queries.

### 1.5 Project Loading: How to Load a Cargo.toml Programmatically

rust-analyzer never does I/O itself. All inputs are passed explicitly via AnalysisHost::apply_change. The loading pipeline:

```
Step 1: cargo metadata + rustc --print sysroot
        --> CargoWorkspace (packages, targets, dependencies)
        --> Sysroot (std, core, alloc crate locations)

Step 2: ProjectWorkspace::load(manifest, &cargo_config, &progress_fn)
        --> ProjectWorkspace (unified representation)

Step 3: Optionally run build scripts
        project_workspace.run_build_scripts(&cargo_config, &progress_fn)
        project_workspace.set_build_scripts(build_scripts)

Step 4: load_workspace(project_workspace, &extra_env, &load_config)
        --> (AnalysisHost, Vfs, Option<ProcMacroClient>)

Step 5: let analysis = host.analysis();
        // Now you can query semantic information
```

**Key types for loading**:
- `ProjectManifest` --- discovered from a path to Cargo.toml
- `CargoConfig` --- controls features, target triple, build script handling
- `LoadCargoConfig` --- controls proc macro loading, cache prefilling
- `ProjectWorkspace` --- the unified project representation
- `CrateGraph` --- DAG of crates. Lower than Cargo's model: each Cargo target is a separate crate. A crate has a root FileId, cfg flags, and dependencies.

**Important**: CrateGraph is described as "spiritually a serialization of rust-project.json." Non-Cargo build systems can provide a rust-project.json instead of Cargo.toml.

---

## 2. Exact API Methods We Need (~20 Methods)

All methods below take &self and db: &dyn HirDatabase unless noted otherwise. The db is the salsa database context.

### 2.1 hir::Function

```rust
// Identity & metadata
fn name(&self, db) -> Name
fn module(&self, db) -> Module
fn visibility(&self, db) -> Visibility

// Signature
fn ret_type(&self, db) -> Type              // Resolved return type (not syntax, actual type)
fn assoc_fn_params(&self, db) -> Vec<Param> // All params including self
fn params_without_self(&self, db) -> Vec<Param>
fn method_params(&self, db) -> Option<Vec<Param>> // None if not a method

// Modifiers
fn is_async(&self, db) -> bool
fn is_unsafe(&self, db) -> bool
fn is_const(&self, db) -> bool
fn has_self_param(&self, db) -> bool

// Generics
fn generic_params(&self, db) -> Vec<GenericParam>  // Type params, lifetime params, const params
```

**What this gives us that tree-sitter cannot**: The ret_type() returns the **resolved** type. For `fn foo() -> impl Iterator<Item = MyStruct>`, tree-sitter sees the syntax `impl Iterator<Item = MyStruct>`. rust-analyzer resolves what MyStruct actually IS --- which module it's from, what traits it implements, its full path. For generic functions, it can tell you the concrete types at call sites.

### 2.2 hir::Param

```rust
fn ty(&self) -> &Type     // Resolved parameter type
fn name(&self) -> Option<Name>
fn pattern_source(&self, db) -> Option<InFile<ast::Pat>>  // Source pattern
```

### 2.3 hir::Impl

```rust
// What is being implemented
fn self_ty(&self, db) -> Type              // The Self type of the impl block
fn trait_(&self, db) -> Option<Trait>      // The trait (None for inherent impls)
fn items(&self, db) -> Vec<AssocItem>      // All associated items (fns, types, consts)

// Query impls for a type
fn all_for_type(db, ty: Type) -> Vec<Impl>    // Static: find all impls for a type
fn all_for_trait(db, trait_: Trait) -> Vec<Impl> // Static: find all impls of a trait

// Modifiers
fn is_negative(&self, db) -> bool
fn is_unsafe(&self, db) -> bool
```

**What this gives us**: For any struct, we can ask "what traits does this implement?" and get back concrete answers. Impl::all_for_type(db, my_struct_type) returns every impl block. impl.trait_() tells us which trait. This is **trait impl resolution** --- something tree-sitter fundamentally cannot do because it requires type inference.

### 2.4 hir::Type

```rust
// Classification
fn as_adt(&self) -> Option<Adt>           // If this is a struct/enum/union, get the Adt
fn as_builtin(&self) -> Option<BuiltinType>
fn as_closure(&self) -> Option<Closure>
fn is_fn_ptr(&self) -> bool
fn is_reference(&self) -> bool
fn is_mutable_reference(&self) -> bool
fn is_unknown(&self) -> bool

// Layout (added in PR #13490)
fn layout(&self, db) -> Result<Layout, LayoutError>  // Size, alignment, field offsets

// Display
fn display(&self, db) -> impl fmt::Display  // Human-readable type name
fn display_truncated(&self, db, max_size: Option<usize>) -> impl fmt::Display
fn display_source_code(&self, db, module: Module, allow_opaque: bool) -> Result<String, DisplaySourceCodeError>

// Traversal
fn autoderef(&self, db) -> impl Iterator<Item = Type>  // Deref chain
fn type_arguments(&self) -> impl Iterator<Item = Type>  // Generic args
fn impls_trait(&self, db, trait_: Trait, args: &[Type]) -> bool
fn normalize_trait_assoc_type(&self, db, args: &[Type], alias: TypeAlias) -> Option<Type>
```

**What this gives us**: Type::layout() returns **size in bytes, alignment, and field offsets** --- data that is impossible to get from syntax alone because it depends on the target platform, repr attributes, and field ordering. Type::display() gives the **fully qualified resolved type name**, not just the syntactic name.

### 2.5 hir::Struct / hir::Enum / hir::Union (via hir::Adt)

These are grouped under the Adt (Algebraic Data Type) enum:

```rust
enum Adt {
    Struct(Struct),
    Enum(Enum),
    Union(Union),
}
```

**Struct methods**:
```rust
fn name(&self, db) -> Name
fn fields(&self, db) -> Vec<Field>
fn ty(&self, db) -> Type                   // Type with unknown generic params
fn generic_params(&self, db) -> Vec<GenericParam>
fn repr(&self, db) -> Option<ReprOptions>  // #[repr(C)], #[repr(packed)], etc.
```

**Enum methods**:
```rust
fn name(&self, db) -> Name
fn variants(&self, db) -> Vec<Variant>
fn generic_params(&self, db) -> Vec<GenericParam>
fn ty(&self, db) -> Type
fn is_data_carrying(&self, db) -> bool
```

**Variant methods** (enum variant):
```rust
fn name(&self, db) -> Name
fn fields(&self, db) -> Vec<Field>
fn parent_enum(&self, db) -> Enum
```

**Field methods**:
```rust
fn name(&self, db) -> Name
fn ty(&self, db) -> Type                   // Resolved field type
fn visibility(&self, db) -> Visibility
```

**What this gives us**: For `struct Foo<T> { bar: T }`, tree-sitter sees field type as T. rust-analyzer resolves what T actually IS at each usage site. Combined with Type::layout(), we get field offsets and padding --- critical for memory layout analysis and unsafe code auditing.

### 2.6 hir::Trait

```rust
fn name(&self, db) -> Name
fn items(&self, db) -> Vec<AssocItem>              // Required and provided methods
fn super_traits(&self, db) -> Vec<Trait>           // Direct supertraits
fn all_super_traits(&self, db) -> Vec<Trait>       // Transitive closure of supertraits
fn generic_params(&self, db) -> Vec<GenericParam>
fn is_auto(&self, db) -> bool                      // auto trait (Send, Sync)
fn is_unsafe(&self, db) -> bool                    // unsafe trait
fn is_object_safe(&self, db) -> bool
```

**What this gives us**: The full supertrait hierarchy. For `trait Handler: Service + Clone + Send`, tree-sitter sees the syntax bounds. rust-analyzer resolves the **transitive closure** --- if Service: Debug, then Handler transitively requires Debug too. This is critical for understanding trait obligation chains.

### 2.7 hir::Closure

```rust
// Accessed via Type::as_closure()
fn captured_items(&self, db) -> Vec<ClosureCapture>
fn fn_trait(&self, db) -> FnTrait  // Fn, FnMut, or FnOnce
```

**ClosureCapture** contains:
```rust
fn local(&self) -> Local           // The captured variable
fn kind(&self) -> CaptureKind     // ByRef(Shared/Mut/UniqueImmutable) or ByValue
fn display_place(&self, db) -> String  // Human-readable capture description
```

**What this gives us**: For `let closure = move |x| foo.bar.baz()`, tree-sitter sees a closure expression with move. rust-analyzer tells us **exactly** which variables are captured, whether by reference or by value, and which specific fields of a struct are captured (RFC 2229 precise captures). This is critical for understanding data flow through closures and for ownership/lifetime analysis.

### 2.8 Generic Parameters and Bounds

```rust
// GenericParam is an enum:
enum GenericParam {
    TypeParam(TypeParam),
    LifetimeParam(LifetimeParam),
    ConstParam(ConstParam),
}

// TypeParam methods:
fn name(&self, db) -> Name
fn default(&self, db) -> Option<Type>
fn trait_bounds(&self, db) -> Vec<Trait>  // The bounds on this type param

// LifetimeParam methods:
fn name(&self, db) -> Name
```

**What this gives us**: For `fn foo<T: Display + Clone + 'static>(x: T)`, tree-sitter parses the bound syntax. rust-analyzer resolves the full trait objects behind Display and Clone, including their supertraits and associated types. It also resolves where clauses: `where T: Into<String>` --- tree-sitter sees Into<String> as syntax; rust-analyzer knows Into<String> is the specific trait from core::convert with the String type argument.

---

## 3. Reference Implementations to Study

### 3.1 cargo-modules (PRIMARY REFERENCE)

**Repository**: https://github.com/regexident/cargo-modules
**What it does**: Visualizes a Rust crate's internal module/item structure as a tree or graph.
**Why it matters**: It is the closest existing tool to what rust-llm-03 will do. It uses the full ra_ap_* stack.

**Dependencies** (from Cargo.toml, all pinned to =0.0.289):
```toml
ra_ap_base_db = "=0.0.289"
ra_ap_cfg = "=0.0.289"
ra_ap_hir = "=0.0.289"
ra_ap_hir_def = "=0.0.289"
ra_ap_hir_ty = "=0.0.289"
ra_ap_ide = "=0.0.289"
ra_ap_ide_db = "=0.0.289"
ra_ap_load-cargo = "=0.0.289"
ra_ap_paths = "=0.0.289"
ra_ap_proc_macro_api = "=0.0.289"
ra_ap_project_model = "=0.0.289"
ra_ap_syntax = "=0.0.289"
ra_ap_vfs = "=0.0.289"
```

**Key pattern**: Exact version pinning (=0.0.289) across ALL ra_ap crates. This is mandatory because the internal APIs change between releases and all crates must be the same version.

**Build time optimization** (from their Cargo.toml):
```toml
[profile.dev-opt]
inherits = "dev"
opt-level = 2

[profile.dev.package."ra_ap_hir"]
inherits = "dev-opt"
[profile.dev.package."ra_ap_hir_def"]
inherits = "dev-opt"
[profile.dev.package."ra_ap_hir_ty"]
inherits = "dev-opt"
[profile.dev.package."ra_ap_ide"]
inherits = "dev-opt"
[profile.dev.package."ra_ap_ide_db"]
inherits = "dev-opt"
[profile.dev.package."ra_ap_load-cargo"]
inherits = "dev-opt"
```

This compiles the ra_ap crates at opt-level = 2 even in dev mode --- critical because rust-analyzer's salsa queries are unusably slow at opt-level = 0.

**What to study in source**:
1. How it calls load_workspace to set up the analysis database
2. How it traverses Module -> items -> Function/Struct/Enum/Trait
3. How it handles multi-crate workspaces
4. Error handling patterns for broken/incomplete code

### 3.2 cargo-semver-checks (CORRECTED --- Does NOT Use ra_hir)

**Repository**: https://github.com/obi1kenobi/cargo-semver-checks
**What it actually does**: Uses **rustdoc JSON** (not rust-analyzer) to detect semver violations.

**Architecture**:
1. rustdoc generates JSON describing a crate's public API
2. Trustfall query engine processes that JSON
3. Declarative lint queries detect semver violations

**Why this matters for us**: It represents an **alternative approach** to Rust semantic analysis. Instead of loading the full compiler frontend (rust-analyzer), it uses rustdoc's already-computed output. Trade-offs:
- Rustdoc JSON: simpler, faster, but only public API, no type layouts, no closure captures
- rust-analyzer HIR: full semantic model, but heavier, slower to load, internal API

**Decision for v2.0.0**: We need closure captures, type layouts, and trait impl resolution for ALL items (not just public API). Rustdoc JSON is insufficient. We use rust-analyzer.

### 3.3 Cratographer (MCP Tool)

An MCP tool for AI agents that uses ra_ap_ide for "full semantic understanding of Rust code." Demonstrates that the ra_ap_* crates are viable for production tool use, not just internal rust-analyzer consumption.

### 3.4 ink-analyzer (Smart Contract Analyzer)

**Repository**: https://github.com/ink-analyzer/ink-analyzer
**What it does**: Semantic analysis of ink! smart contracts for Substrate.
**What it uses**: ra_ap_syntax for lossless, error-resilient parsing. Has an architecture similar to ra_ap_ide with notions of persistent state, change over time, immutable snapshots, and cancellable transactions.

**Key insight**: ink-analyzer chose ra_ap_syntax over syn specifically because ra_ap_syntax generates a lossless syntax tree with error nodes for invalid code, allowing analysis to continue on the rest of the valid code. This error resilience is critical for IDE-like tools.

### 3.5 rust-clippy (DIFFERENT APPROACH --- Uses rustc Internals)

**What it uses**: rustc compiler internals directly (not rust-analyzer).
**Why not this approach**: rustc internals are tightly coupled to specific nightly compiler versions. rust-analyzer's ra_ap_* crates are published on crates.io with a stable-ish API. Clippy requires a specific nightly toolchain per version; we need to work with stable Rust.

---

## 4. Version Pinning Strategy

### 4.1 Current Latest Version

As of February 2026, the latest ra_ap_hir version is **0.0.290** on crates.io. The ra_ap_* crates follow a rapid release cadence tied to rust-analyzer development.

### 4.2 Pinning Strategy

**Exact version pinning is mandatory**. All ra_ap_* crates must be the same version:

```toml
# In Cargo.toml for rust-llm-03-rust-analyzer
[dependencies]
ra_ap_hir = "=0.0.290"
ra_ap_ide = "=0.0.290"
ra_ap_ide_db = "=0.0.290"
ra_ap_load-cargo = "=0.0.290"
ra_ap_paths = "=0.0.290"
ra_ap_project_model = "=0.0.290"
ra_ap_syntax = "=0.0.290"
ra_ap_vfs = "=0.0.290"
```

The = prefix is critical. Without it, Cargo may resolve different ra_ap_* crates to different versions, which will cause compilation failures because the internal types are not compatible across versions.

### 4.3 Why crates.io Published Crates (Not Git Dependency)

The ra_ap_* crates are **automatically published** from the rust-analyzer repository to crates.io. Using crates.io is preferred over git dependencies because:

1. **Reproducible builds**: crates.io versions are immutable
2. **Cargo.lock stability**: git dependencies can break when the upstream repo force-pushes or reorganizes
3. **Compilation caching**: crates.io dependencies are cached by Cargo; git dependencies are re-fetched
4. **Ecosystem compatibility**: other crates in our workspace can depend on the same version

A git dependency would look like this (NOT recommended):
```toml
# DON'T DO THIS --- fragile, slow, non-reproducible
[dependencies]
hir = { git = "https://github.com/rust-lang/rust-analyzer", tag = "2025-12-29" }
```

### 4.4 When and Why to Update

**Update when**:
1. A bug fix in rust-analyzer affects our analysis correctness
2. New API methods are added that we need (e.g., better closure capture analysis)
3. Performance improvements in salsa or type inference
4. Security fixes

**Update process**:
1. Change ALL ra_ap_* versions simultaneously in Cargo.toml
2. Run full test suite --- internal API changes may break our code
3. Check the rust-analyzer changelog for breaking changes
4. Pin to the new version only after all tests pass

**Update cadence**: Quarterly, unless a critical fix is needed. The ra_ap_* crates publish frequently (weekly), but most releases are incremental and don't affect our usage.

---

## 5. Feature Flag Setup

### 5.1 The Core Problem

The ra_ap_* dependency tree is massive. It pulls in salsa, chalk (trait solver), rowan (syntax trees), and dozens of internal crates. This adds significant compile time. Most users of parseltongue analyze multi-language codebases and don't need Rust-specific deep analysis. The rust-analyzer bridge must be **optional**.

### 5.2 Cargo Feature Flag Design

In the **workspace-level** Cargo.toml:
```toml
[workspace.features]
rust-analyzer = []
```

In rust-llm-03-rust-analyzer/Cargo.toml:
```toml
[package]
name = "rust-llm-03-rust-analyzer"

[dependencies]
ra_ap_hir = "=0.0.290"
ra_ap_ide = "=0.0.290"
ra_ap_load-cargo = "=0.0.290"
ra_ap_project_model = "=0.0.290"
ra_ap_paths = "=0.0.290"
ra_ap_syntax = "=0.0.290"
ra_ap_vfs = "=0.0.290"
# ... all ra_ap_* deps
```

In the **binary crate** (rust-llm/Cargo.toml):
```toml
[dependencies]
rust-llm-03-rust-analyzer = { path = "../rust-llm-03-rust-analyzer", optional = true }

[features]
default = []
rust-analyzer = ["dep:rust-llm-03-rust-analyzer"]
```

### 5.3 Conditional Compilation in the Pipeline

```rust
// In rust-llm (binary) or rust-llm-04 (reasoning engine)

pub fn analyze_codebase(path: &Path) -> AnalysisResult {
    // Step 1: Always run tree-sitter extraction (fast, all languages)
    let syntactic_facts = rust_llm_01::extract_facts(path);

    // Step 2: Always run cross-language edge detection
    let cross_lang_edges = rust_llm_02::detect_boundaries(&syntactic_facts);

    // Step 3: Conditionally run rust-analyzer (slow, Rust only)
    #[cfg(feature = "rust-analyzer")]
    let semantic_facts = rust_llm_03::extract_semantic_facts(path);

    #[cfg(not(feature = "rust-analyzer"))]
    let semantic_facts = Vec::new();

    // Step 4: Feed everything into Ascent reasoning engine
    rust_llm_04::reason(syntactic_facts, cross_lang_edges, semantic_facts)
}
```

### 5.4 Build Commands

```bash
# Fast build --- no rust-analyzer (default)
cargo build --release
# Compiles in ~30 seconds. Analyzes all 12 languages syntactically.

# Full build --- with rust-analyzer
cargo build --release --features rust-analyzer
# Compiles in ~5-10 minutes first time. Adds deep Rust semantic analysis.
```

### 5.5 Runtime Detection

The binary should report whether it was compiled with rust-analyzer support:

```rust
pub fn capabilities() -> Vec<&'static str> {
    let mut caps = vec!["tree-sitter", "cross-language", "ascent-reasoning"];

    #[cfg(feature = "rust-analyzer")]
    caps.push("rust-analyzer-semantic");

    caps
}
```

This surfaces in /server-health-check-status or equivalent endpoint so users know what analysis depth is available.

---

## 6. What Data We Get That Tree-Sitter CANNOT Provide

This is the entire justification for the rust-llm-03 crate. Tree-sitter gives us syntax; rust-analyzer gives us semantics.

### 6.1 Resolved Type Names

| Scenario | Tree-sitter sees | rust-analyzer resolves |
|---|---|---|
| `fn foo() -> T` | T (identifier) | std::collections::HashMap<String, Vec<u8>> |
| `let x = bar();` | No type at all | Result<HttpResponse, anyhow::Error> |
| `impl<T: Clone> Foo<T>` | T, Clone, Foo as syntax | T is bounded by core::clone::Clone, Foo is crate::models::Foo |
| `use std::io::Result` | Import path | core::result::Result<T, std::io::Error> (type alias resolved) |

**Impact on our graph**: Instead of edges like fn:foo -> type:T (meaningless), we get fn:foo -> type:std::collections::HashMap (actionable). Type alias chains are fully resolved.

### 6.2 Trait Implementation Resolution

Tree-sitter can see `impl Display for MyStruct { ... }` in the same file. But:

| Scenario | Tree-sitter | rust-analyzer |
|---|---|---|
| `#[derive(Clone, Debug)]` | Sees the attribute text | Knows MyStruct implements Clone and Debug with generated methods |
| Blanket impl `impl<T: Display> ToString for T` | Cannot see (different crate) | Knows MyStruct implements ToString because it implements Display |
| `impl From<MyError> for anyhow::Error` | Only if in same file | Finds all From impls across the entire crate graph |
| Auto traits (Send, Sync) | Cannot determine | Knows whether a type is Send/Sync based on its fields |

**Impact**: Impl::all_for_type(db, my_struct_type) gives the **complete** set of trait implementations, including derived, blanket, and auto trait impls. This is critical for understanding API contracts.

### 6.3 Type Layouts (Size, Alignment, Padding)

```rust
// For: struct Foo { a: u8, b: u64, c: u16 }

// Tree-sitter: sees field declarations, types as syntax. Knows nothing about size.
// rust-analyzer: Type::layout() returns:
//   size: 16 bytes
//   alignment: 8 bytes
//   field offsets: a=8, b=0, c=10 (reordered by compiler for optimal packing)
//   padding: 5 bytes between c and end
```

This data is computed using the same layout algorithm as rustc, ported to rust-analyzer in PR #13490 (by HKalbasi). It respects #[repr(C)], #[repr(packed)], #[repr(align(N))], and target-specific sizes.

**Impact**: Enables memory layout visualization, padding analysis, cache-line optimization hints --- features no other code analysis tool provides at the source level.

### 6.4 Closure Capture Analysis

```rust
let name = String::from("hello");
let data = vec![1, 2, 3];
let config = Config { verbose: true, level: 5 };

let closure = move || {
    println!("{}", name);           // captures name by value
    process(&data);                 // captures data by value (move)
    if config.verbose { log(); }    // captures config.verbose by value (RFC 2229 precise)
};
```

Tree-sitter sees: a closure expression with move keyword. Nothing about what is captured.

rust-analyzer via Closure::captured_items(db) returns:
1. name --- CaptureKind::ByValue
2. data --- CaptureKind::ByValue
3. config.verbose --- CaptureKind::ByValue (precise capture of single field)

And closure.fn_trait(db) returns FnOnce (because it moves captured values).

**Impact**: This enables:
- Ownership flow tracking through closures
- Detecting when closures accidentally capture more than needed
- Understanding the Fn/FnMut/FnOnce trait implementation of each closure
- Data flow analysis for security auditing (what data can a closure access?)

### 6.5 Lifetime Information

While rust-analyzer's lifetime inference is not as complete as rustc's, it does resolve lifetime parameters and their relationships:

- Named lifetimes on function signatures ('a, 'static)
- Lifetime bounds (T: 'a)
- Lifetime elision results (what the compiler infers)

Tree-sitter sees lifetime syntax; rust-analyzer resolves what it **means** for data validity.

### 6.6 Macro Expansion (Seeing Through Derive Macros)

rust-analyzer expands macros and analyzes the expanded code:

```rust
#[derive(Serialize, Deserialize)]
struct Config {
    port: u16,
    host: String,
}
```

Tree-sitter sees: an attribute #[derive(Serialize, Deserialize)] and the struct fields.
rust-analyzer: expands the derive macro, generates the impl Serialize for Config and impl Deserialize for Config blocks, type-checks them, and makes the generated methods available in the semantic model.

**Macro expansion pipeline**:
1. macro_rules! macros: expanded by rust-analyzer's built-in engine
2. Procedural macros: expanded by loading the proc macro .dylib and calling it via proc_macro_srv
3. The expanded code is then parsed, resolved, and type-checked like regular code

**Known limitation**: When a proc macro crashes or is unavailable, rust-analyzer falls back to treating the macro output as {unknown}. This can cascade --- if #[derive(Serialize)] fails to expand, rust-analyzer won't see any Serialize impl for the type. Our code must handle this gracefully.

**Impact**: We can see the **actual** trait implementations generated by derive macros, not just the attribute text. This means #[derive(Clone)] generates a real Clone impl that appears in our dependency graph.

### 6.7 Summary Table

| Capability | Tree-sitter | rust-analyzer | Value for v2.0.0 |
|---|---|---|---|
| Resolved type names | No | Yes | Accurate dependency edges |
| Trait impl resolution | Partial (same file) | Complete (crate graph) | Full API contract understanding |
| Type layouts | No | Yes | Memory analysis, unsafe auditing |
| Closure captures | No | Yes (RFC 2229) | Ownership flow, data flow analysis |
| Lifetime resolution | Syntax only | Partial semantic | Data validity analysis |
| Macro expansion | No | Yes (proc macro) | See through derive macros |
| Generic instantiation | No | Yes | Concrete type at call sites |
| Auto trait analysis | No | Yes | Send/Sync safety verification |
| Where clause resolution | Syntax only | Full semantic | Complete trait obligation chains |
| Type alias resolution | No | Yes | True underlying types |

---

## 7. Build Time Impact and Mitigation Strategies

### 7.1 The Problem

rust-analyzer is famously slow to compile. The ra_ap_* dependency tree includes:
- salsa (incremental computation)
- chalk (trait solver)
- rowan (syntax trees)
- dozens of internal crates (hir_def, hir_ty, hir_expand, base_db, cfg, tt, mbe, ...)

**Expected first-time build**: 5-10 minutes (release mode on a modern machine).
**Expected incremental rebuild** (after changing our code, not ra_ap deps): seconds (Cargo caches compiled dependencies).

For comparison, rust-analyzer itself notes that "both ra_hir and ra_ide_api are really slow to compile, which makes the fix & test loop rather frustrating" (GitHub issue #1987).

### 7.2 Mitigation Strategy 1: Feature Flag (Primary)

As described in Section 5, making the entire rust-llm-03 crate optional behind a feature flag means the **default build** never touches the ra_ap crates. Only users who explicitly opt in (--features rust-analyzer) pay the compilation cost.

### 7.3 Mitigation Strategy 2: Dev Profile Optimization

Following cargo-modules' pattern, compile ra_ap crates at opt-level = 2 even in dev mode. Without this, salsa's queries are unreasonably slow at opt-level = 0:

```toml
[profile.dev-opt]
inherits = "dev"
opt-level = 2

# Apply to all ra_ap crates
[profile.dev.package."ra_ap_hir"]
inherits = "dev-opt"
[profile.dev.package."ra_ap_hir_def"]
inherits = "dev-opt"
[profile.dev.package."ra_ap_hir_ty"]
inherits = "dev-opt"
[profile.dev.package."ra_ap_ide"]
inherits = "dev-opt"
[profile.dev.package."ra_ap_ide_db"]
inherits = "dev-opt"
[profile.dev.package."ra_ap_load-cargo"]
inherits = "dev-opt"
[profile.dev.package."ra_ap_base_db"]
inherits = "dev-opt"
[profile.dev.package."ra_ap_syntax"]
inherits = "dev-opt"
```

### 7.4 Mitigation Strategy 3: Separate Workspace Member

The rust-llm-03-rust-analyzer crate is its own workspace member with its own dependency tree. Cargo only compiles it when something in the workspace depends on it. If we structure the feature flag correctly, cargo build (without --features rust-analyzer) will skip the entire crate and its dependencies.

### 7.5 Mitigation Strategy 4: Crate Boundary Isolation

The rust-llm-03 crate should have a **thin, stable public API**:

```rust
// rust-llm-03's public interface
pub struct SemanticFact { ... }  // Uses types from rust-llm-core, NOT ra_ap types

pub fn extract_semantic_facts(
    cargo_toml_path: &Path,
    progress: &dyn Fn(String),
) -> Result<Vec<SemanticFact>, Error>
```

No ra_ap_* types leak into the public API. This means changes to the ra_ap API only affect rust-llm-03 internally. Other crates in the workspace never need to know about salsa, chalk, or rowan.

### 7.6 Mitigation Strategy 5: Compilation Caching

- **sccache**: Shared compilation cache. Since ra_ap crates are deterministic builds from crates.io, they can be cached across CI runs and developer machines.
- **cargo-chef**: For Docker builds, pre-compile dependencies in a separate layer so ra_ap crates are built once and cached.
- **Cargo.lock**: Always commit Cargo.lock to ensure deterministic dependency resolution.

### 7.7 Expected Build Time Budget

| Scenario | Without rust-analyzer | With rust-analyzer |
|---|---|---|
| First clean build (release) | ~30s | ~5-10 min |
| Incremental rebuild (our code changed) | ~2-5s | ~2-5s |
| Incremental rebuild (ra_ap version bumped) | N/A | ~5-10 min |
| CI build (with sccache warm) | ~30s | ~1-2 min |

The key insight: the build time cost is **one-time** per ra_ap version. After the initial compilation, Cargo caches the compiled ra_ap crates and only recompiles our code.

---

## 8. Programmatic Usage Pattern (Pseudocode)

Putting it all together, here is how rust-llm-03 will use the ra_ap APIs:

```rust
use ra_ap_hir::{self as hir, Adt, AssocItem, Crate, HirDatabase, Module, Semantics};
use ra_ap_ide::{AnalysisHost, RootDatabase};
use ra_ap_load_cargo::{LoadCargoConfig, ProcMacroServerChoice, load_workspace};
use ra_ap_paths::AbsPathBuf;
use ra_ap_project_model::{CargoConfig, ProjectManifest, ProjectWorkspace};

pub fn extract_semantic_facts(cargo_toml: &Path) -> Result<Vec<SemanticFact>> {
    // 1. Discover and load the project
    let manifest_path = AbsPathBuf::assert(cargo_toml.canonicalize()?);
    let manifest = ProjectManifest::from_manifest_file(manifest_path)?;

    let cargo_config = CargoConfig::default();
    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: true,
        prefill_caches: false,
        with_proc_macro_server: ProcMacroServerChoice::Sysroot,
    };

    // 2. Load workspace into analysis host
    let mut workspace = ProjectWorkspace::load(manifest, &cargo_config, &|_| {})?;
    let build_scripts = workspace.run_build_scripts(&cargo_config, &|_| {})?;
    workspace = workspace.set_build_scripts(build_scripts);

    let (host, vfs, _proc_macro) = load_workspace(
        workspace, &cargo_config.extra_env, &load_config
    )?;

    let db = host.raw_database();
    let mut facts = Vec::new();

    // 3. Iterate over all crates in the workspace (skip dependencies)
    for krate in Crate::all(db) {
        if !is_workspace_crate(&krate, db) { continue; }

        // 4. Walk module tree
        visit_module(krate.root_module(db), db, &mut facts)?;
    }

    Ok(facts)
}

fn visit_module(module: Module, db: &RootDatabase, facts: &mut Vec<SemanticFact>) {
    for item in module.declarations(db) {
        match item {
            hir::ModuleDef::Function(func) => {
                let ret_type = func.ret_type(db);
                let is_async = func.is_async(db);
                let is_unsafe = func.is_unsafe(db);
                let params = func.assoc_fn_params(db);

                facts.push(SemanticFact::Function {
                    name: func.name(db).to_string(),
                    return_type: ret_type.display(db).to_string(),
                    is_async,
                    is_unsafe,
                    param_types: params.iter()
                        .map(|p| p.ty().display(db).to_string())
                        .collect(),
                    generic_bounds: extract_bounds(&func, db),
                });
            }

            hir::ModuleDef::Adt(adt) => match adt {
                Adt::Struct(s) => {
                    let fields: Vec<_> = s.fields(db).iter().map(|f| {
                        FieldInfo {
                            name: f.name(db).to_string(),
                            ty: f.ty(db).display(db).to_string(),
                            visibility: format!("{:?}", f.visibility(db)),
                        }
                    }).collect();

                    // Get layout if available
                    let layout = s.ty(db).layout(db).ok().map(|l| LayoutInfo {
                        size: l.size.bytes(),
                        align: l.align.abi.bytes(),
                    });

                    // Find all trait impls
                    let impls = hir::Impl::all_for_type(db, s.ty(db));
                    let traits: Vec<String> = impls.iter()
                        .filter_map(|i| i.trait_(db))
                        .map(|t| t.name(db).to_string())
                        .collect();

                    facts.push(SemanticFact::Struct {
                        name: s.name(db).to_string(),
                        fields,
                        layout,
                        implemented_traits: traits,
                    });
                }
                Adt::Enum(e) => { /* similar for variants */ }
                Adt::Union(u) => { /* similar for fields */ }
            },

            hir::ModuleDef::Trait(t) => {
                let supers = t.all_super_traits(db)
                    .iter()
                    .map(|s| s.name(db).to_string())
                    .collect();

                facts.push(SemanticFact::Trait {
                    name: t.name(db).to_string(),
                    super_traits: supers,
                    items: t.items(db).len(),
                    is_auto: t.is_auto(db),
                    is_unsafe: t.is_unsafe(db),
                    is_object_safe: t.is_object_safe(db),
                });
            }

            hir::ModuleDef::Module(submodule) => {
                visit_module(submodule, db, facts);
            }

            _ => {} // Const, Static, TypeAlias, BuiltinType, Macro
        }
    }

    // 5. Also visit impl blocks in this module
    for impl_block in module.impl_defs(db) {
        let self_ty = impl_block.self_ty(db).display(db).to_string();
        let trait_name = impl_block.trait_(db).map(|t| t.name(db).to_string());

        for item in impl_block.items(db) {
            if let AssocItem::Function(func) = item {
                // Extract method info with resolved types
                // ...
            }
        }
    }
}
```

---

## 9. Risk Assessment and Open Questions

### 9.1 API Stability Risk

The ra_ap_* crates are **internal APIs** published on crates.io. They have no stability guarantees. Methods can be renamed, signatures can change, types can be reorganized between releases.

**Mitigation**:
- Exact version pinning (=0.0.290)
- Thin wrapper API that isolates ra_ap types from the rest of the codebase
- Update on our schedule, not theirs (quarterly)
- Comprehensive test suite that catches API breakage immediately

### 9.2 Proc Macro Expansion Reliability

Proc macros are expanded by loading the .dylib and calling it. This can fail if:
- The proc macro crate hasn't been compiled yet
- The proc macro panics on incomplete code
- The proc macro server crashes

**Mitigation**: Always handle {unknown} types gracefully. If a derive macro fails to expand, our analysis should degrade gracefully (lose those trait impls) rather than crash.

### 9.3 Memory Usage

rust-analyzer's salsa database holds the entire crate graph in memory. For large projects (100+ crates, millions of lines), this can use 2-4 GB of RAM.

**Mitigation**: The rust-llm-03 crate runs as a one-shot analysis, not a persistent server. After extracting all semantic facts, we drop the AnalysisHost and free the memory. The extracted facts (our SemanticFact structs) are much smaller than the full salsa database.

### 9.4 Analysis Time

Loading and analyzing a large Cargo workspace takes 10-60 seconds depending on size.

**Mitigation**: Run rust-analyzer analysis in parallel with tree-sitter extraction. Tree-sitter is fast (sub-second for most projects); rust-analyzer is slow. Start both at ingest time, merge results when both complete.

### 9.5 Open Questions

1. **Which ra_ap version to pin for v2.0.0 launch?** Current latest is 0.0.290. Should we test with this, or wait for the next stable batch?
2. **Do we need ra_ap_hir_ty directly?** Or is everything accessible through the hir facade? cargo-modules pulls in hir_ty --- we should check if they need it for something the facade doesn't expose.
3. **Build script handling**: Should we run build scripts during analysis? Required for crates that generate code (e.g., prost, sqlx), but adds time and complexity.
4. **Proc macro loading**: ProcMacroServerChoice::Sysroot vs ProcMacroServerChoice::None? Loading proc macros makes #[derive(...)] work but adds time and can crash. Start without, add later?
5. **Target triple**: rust-analyzer needs to know the target for type layouts. Default to host triple? Allow configuration?

---

## 10. Sources and References

### Architecture and Documentation
- [rust-analyzer Architecture Guide](https://rust-analyzer.github.io/book/contributing/architecture.html)
- [rust-analyzer Contributing Guide](https://rust-analyzer.github.io/book/contributing/guide.html)
- [Durable Incrementality (Salsa blog post)](https://rust-analyzer.github.io/blog/2023/07/24/durable-incrementality.html)
- [Fast Rust Builds (matklad)](https://matklad.github.io/2021/09/04/fast-rust-builds.html)

### API Documentation
- [ra_ap_hir on docs.rs](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/)
- [ra_ap_ide on docs.rs](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/)
- [hir crate internal docs](https://rust-lang.github.io/rust-analyzer/hir/index.html)
- [hir::Function](https://rust-lang.github.io/rust-analyzer/hir/struct.Function.html)
- [hir::Impl](https://rust-lang.github.io/rust-analyzer/hir/struct.Impl.html)
- [hir::Type](https://rust-lang.github.io/rust-analyzer/hir/struct.Type.html)
- [hir::Closure](https://rust-lang.github.io/rust-analyzer/hir/struct.Closure.html)
- [hir::Variant](https://rust-lang.github.io/rust-analyzer/hir/struct.Variant.html)

### Reference Implementations
- [cargo-modules (uses ra_ap_hir)](https://github.com/regexident/cargo-modules)
- [cargo-semver-checks (uses rustdoc JSON)](https://github.com/obi1kenobi/cargo-semver-checks)
- [ink-analyzer (uses ra_ap_syntax)](https://github.com/ink-analyzer/ink-analyzer)
- [rust-analyzer repository](https://github.com/rust-lang/rust-analyzer)

### Key PRs
- [PR #13490: Compute data layout of types](https://github.com/rust-lang/rust-analyzer/pull/13490)
- [PR #14470: Compute closure captures](https://github.com/rust-lang/rust-analyzer/pull/14470)
- [PR #20039: Fix closure capturing for let exprs](https://github.com/rust-lang/rust-analyzer/pull/20039)
- [PR #20144: Add load_workspace_into_db](https://github.com/rust-lang/rust-analyzer/pull/20144)

### crates.io
- [ra_ap_hir on crates.io](https://crates.io/crates/ra_ap_hir) (latest: 0.0.290)
- [ra_ap_ide on crates.io](https://crates.io/crates/ra_ap_ide)
- [ra_ap_load-cargo on crates.io](https://crates.io/crates/ra_ap_load-cargo)
- [ra_ap_project_model on crates.io](https://crates.io/crates/ra_ap_project_model)
- [salsa on GitHub](https://github.com/salsa-rs/salsa)
