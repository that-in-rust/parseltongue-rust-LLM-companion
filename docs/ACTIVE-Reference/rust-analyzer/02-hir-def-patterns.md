# Idiomatic Rust Patterns: hir-def (Definition Layer)
> Source: rust-analyzer/crates/hir-def
> Purpose: Patterns to guide contributions to rust-analyzer

## Pattern 1: Salsa Interned ID Pattern with Lookup Trait
**File:** `crates/hir-def/src/lib.rs` (lines 221-226, 250-267)
**Category:** Salsa Queries, ID Management
**Description:** Uses a macro-driven pattern to create interned IDs that combine salsa's interning with lookup capabilities. This is the foundational pattern for creating lightweight, copyable IDs that can be looked up to retrieve their full location data.

**Code Example:**
```rust
macro_rules! impl_intern {
    ($id:ident, $loc:ident, $intern:ident, $lookup:ident) => {
        impl_intern_key!($id, $loc);
        impl_intern_lookup!(DefDatabase, $id, $loc, $intern, $lookup);
    };
}

type StructLoc = ItemLoc<ast::Struct>;
impl_intern!(StructId, StructLoc, intern_struct, lookup_intern_struct);

impl StructId {
    pub fn fields(self, db: &dyn DefDatabase) -> &VariantFields {
        VariantFields::firewall(db, self.into())
    }

    pub fn fields_with_source_map(
        self,
        db: &dyn DefDatabase,
    ) -> (Arc<VariantFields>, Arc<ExpressionStoreSourceMap>) {
        VariantFields::query(db, self.into())
    }
}
```

**Why This Matters for Contributors:** This pattern appears dozens of times in hir-def. When adding new definition types (like a hypothetical `TraitAliasId`), you must follow this exact pattern: define a `Loc` type using `ItemLoc` or `AssocItemLoc`, use `impl_intern!` to wire up salsa interning, and add helper methods to access related data. Understanding this pattern is critical for extending rust-analyzer's type system.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Architectural/Performance
**Rust-Specific Insight:** This pattern leverages Rust's macro system combined with salsa's query framework to create zero-cost ID types. The IDs are `Copy` (just integers under the hood) but carry type safety through the type parameter. The lookup trait pattern enables bidirectional mapping (ID ↔ Location) with automatic memoization via salsa's interning, preventing duplicate allocations for identical locations.
**Contribution Tip:** When adding a new definition type (e.g., `impl_intern!(TraitAliasId, TraitAliasLoc, intern_trait_alias, lookup_intern_trait_alias)`), ensure the helper methods follow the naming convention: simple accessors return references to cached data, while `_with_source_map` variants return tuples for IDE features. Check that the `Loc` type includes both the container (ModuleId) and AstId for proper source mapping.
**Common Pitfalls:** Forgetting to implement helper methods on the ID type (like `fields()`) leads to verbose call sites. Not using the macro pattern and hand-rolling implementations risks breaking salsa's invalidation tracking. Using `Box` or `Arc` for IDs when they should be `Copy` hurts performance.
**Related Patterns in Ecosystem:** Similar patterns in salsa-based projects (chalk for trait solving), proc-macro crates using interned symbols (syn's Symbol), and game engines with entity-component-systems (ECS) using type-safe entity IDs.

---

## Pattern 2: Generic ItemLoc with AstId Container Pattern
**File:** `crates/hir-def/src/lib.rs` (lines 116-186)
**Category:** AST-to-HIR Bridge, Location Storage
**Description:** Uses a generic struct `ItemLoc<N: AstIdNode>` to store the location of any syntax node, consisting of a container (ModuleId) and an AstId. Implements all standard traits manually to avoid unnecessary trait bounds while supporting any AST node type.

**Code Example:**
```rust
#[derive(Debug)]
pub struct ItemLoc<N: AstIdNode> {
    pub container: ModuleId,
    pub id: AstId<N>,
}

impl<N: AstIdNode> Clone for ItemLoc<N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<N: AstIdNode> Copy for ItemLoc<N> {}

impl<N: AstIdNode> PartialEq for ItemLoc<N> {
    fn eq(&self, other: &Self) -> bool {
        self.container == other.container && self.id == other.id
    }
}

impl<N: AstIdNode> Eq for ItemLoc<N> {}

impl<N: AstIdNode> Hash for ItemLoc<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.container.hash(state);
        self.id.hash(state);
    }
}

impl<N: AstIdNode> HasModule for ItemLoc<N> {
    #[inline]
    fn module(&self, _db: &dyn DefDatabase) -> ModuleId {
        self.container
    }
}
```

**Why This Matters for Contributors:** This pattern demonstrates how to write generic wrapper types that work with salsa while maintaining full control over trait implementations. The manual trait impls avoid requiring unnecessary bounds on `N` (like `N: Clone`), which would fail for many AST node types. When creating similar location-tracking types, follow this pattern exactly.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Structural/Performance
**Rust-Specific Insight:** This showcases advanced trait bound hygiene—manually implementing standard traits (Clone, PartialEq, Hash) without propagating bounds to the generic parameter `N`. Rust's derive macros would add `N: Clone` bound, but AST nodes often don't implement Clone. By manually delegating to the actual fields (container, id), we get Copy semantics (trivial bitwise copy) without requiring `N: Copy`. The `#[inline]` on `module()` ensures the delegation compiles away.
**Contribution Tip:** When creating new generic location types, copy this exact impl pattern. Use `impl<N: AstIdNode> Clone for YourLoc<N>` with manual delegation, not `#[derive(Clone)]`. Always implement `HasModule` for salsa-queryable types. Test that your type is `Copy` even when `N` is not—this is critical for performance.
**Common Pitfalls:** Adding unnecessary bounds like `N: Clone + Eq` when deriving traits. Forgetting `Copy` implementation (it's just a marker trait when Clone is correct). Not implementing `HasModule`, causing compilation failures in resolver contexts. Using references instead of owned AstId (AstId is already just a u32 wrapper).
**Related Patterns in Ecosystem:** Similar bound-minimal generic patterns in petgraph (Graph<N, E> doesn't require N: Clone), serde's internal types, and nalgebra's generic matrix types that avoid propagating numeric trait bounds unnecessarily.

---

## Pattern 3: Salsa Database Trait Pattern with Query Groups
**File:** `crates/hir-def/src/db.rs` (lines 34-258)
**Category:** Salsa Queries, Database Architecture
**Description:** Defines database traits with `#[query_group]` attribute for both interning (`InternDatabase`) and actual queries (`DefDatabase`). Separates concerns: interning queries vs. data queries vs. transparent queries, each with appropriate salsa attributes.

**Code Example:**
```rust
#[query_group::query_group(InternDatabaseStorage)]
pub trait InternDatabase: RootQueryDb {
    #[salsa::interned]
    fn intern_struct(&self, loc: StructLoc) -> StructId;

    #[salsa::interned]
    fn intern_enum(&self, loc: EnumLoc) -> EnumId;
}

#[query_group::query_group]
pub trait DefDatabase: InternDatabase + ExpandDatabase + SourceDatabase {
    #[salsa::tracked]
    fn struct_signature(&self, struct_: StructId) -> Arc<StructSignature> {
        self.struct_signature_with_source_map(struct_).0
    }

    #[salsa::invoke(StructSignature::query)]
    fn struct_signature_with_source_map(
        &self,
        struct_: StructId,
    ) -> (Arc<StructSignature>, Arc<ExpressionStoreSourceMap>);

    #[salsa::transparent]
    #[salsa::invoke(GenericParams::new)]
    fn generic_params(&self, def: GenericDefId) -> Arc<GenericParams>;
}
```

**Why This Matters for Contributors:** This layered database trait pattern is how rust-analyzer achieves incremental computation. `#[salsa::tracked]` caches results, `#[salsa::transparent]` doesn't create a cache layer (useful for cheap conversions), and `#[salsa::invoke]` specifies a custom query function. When adding queries, you must choose the correct attribute and follow the naming convention: simple queries return just the data, while `_with_source_map` variants return both data and AST mapping.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Architectural/Performance
**Rust-Specific Insight:** This pattern exploits Rust's trait system to create a compile-time query graph with fine-grained invalidation. The trait inheritance (`DefDatabase: InternDatabase + ExpandDatabase`) establishes query dependencies. The `#[salsa::transparent]` attribute is crucial—it makes queries invisible to the dependency graph, preventing cache pollution for trivial transforms. The separation between interning (InternDatabase) and computation (DefDatabase) follows the principle of least privilege: most code only needs read access, not interning rights.
**Contribution Tip:** When adding a query, decide: (1) Is it pure lookup? Use `#[salsa::transparent]`. (2) Does it compute from other queries? Use `#[salsa::tracked]`. (3) Does it need custom computation logic? Add `#[salsa::invoke(YourType::query_fn)]`. Always provide both `foo()` and `foo_with_source_map()` variants for HIR construction queries—the IDE layer needs source maps, but type checking doesn't.
**Common Pitfalls:** Using `tracked` for cheap conversions (pollutes the dependency graph). Forgetting `transparent` for getters that just delegate (e.g., `generic_params` delegates to `GenericParams::new`). Adding queries without considering invalidation cascades—always think "what inputs affect this output?" Not returning Arc for large structures (forces expensive clones).
**Related Patterns in Ecosystem:** Similar incremental computation patterns in turbopack/swc (using turbo-tasks), buck2's dice framework, and any build system with fine-grained caching (bazel's ActionGraph). The transparent query pattern mirrors Haskell's `INLINE` pragma.

---

## Pattern 4: ItemTree as an Invalidation Barrier
**File:** `crates/hir-def/src/item_tree.rs` (lines 1-291, 423-441)
**Category:** Incremental Compilation, IR Design
**Description:** ItemTree is a simplified AST containing only items (no bodies), stored per HirFileId. It's designed as an "invalidation barrier" - changes inside function bodies don't invalidate the ItemTree, preventing cascading re-computation of name resolution.

**Code Example:**
```rust
/// The item tree of a source file.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ItemTree {
    top_level: Box<[ModItemId]>,
    top_attrs: AttrsOrCfg,
    attrs: FxHashMap<FileAstId<ast::Item>, AttrsOrCfg>,
    vis: ItemVisibilities,
    big_data: FxHashMap<FileAstId<ast::Item>, BigModItem>,
    small_data: FxHashMap<FileAstId<ast::Item>, SmallModItem>,
}

mod_items! {
ModItemId ->
    Const in small_data -> ast::Const,
    Enum in small_data -> ast::Enum,
    Function in small_data -> ast::Fn,
    Impl in small_data -> ast::Impl,
    Struct in small_data -> ast::Struct,
    // ... more items
}

#[salsa_macros::tracked(returns(deref))]
pub(crate) fn file_item_tree_query(db: &dyn DefDatabase, file_id: HirFileId) -> Arc<ItemTree> {
    let ctx = lower::Ctx::new(db, file_id);
    let syntax = db.parse_or_expand(file_id);
    let item_tree = match_ast! {
        match syntax {
            ast::SourceFile(file) => ctx.lower_module_items(&file),
            ast::MacroItems(items) => ctx.lower_module_items(&items),
            // ...
        }
    };
    Arc::new(item_tree)
}
```

**Why This Matters for Contributors:** ItemTree is critical to rust-analyzer's performance. It ensures that typing in a function body only invalidates that function's Body, not the entire DefMap or name resolution. When adding new item types or modifying lowering, preserve this property: ItemTree should only contain signature-level information, never expression bodies. The `mod_items!` macro shows the standard pattern for adding new item types to the tree.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Architectural/Performance
**Rust-Specific Insight:** This is the architectural masterstroke of rust-analyzer. By separating "what exists" (ItemTree: signatures, names, visibility) from "what it does" (Body: expression HIR), the system creates a natural invalidation boundary. The use of `FxHashMap<FileAstId, BigModItem>` for large items (impls, enums) and a separate map for small items optimizes memory layout. The `#[salsa_macros::tracked(returns(deref))]` attribute means salsa returns `&Arc<ItemTree>` directly, avoiding ref-counting overhead in hot paths.
**Contribution Tip:** When adding a new item kind (e.g., trait aliases), extend the `mod_items!` macro with an entry like `TraitAlias in small_data -> ast::TraitAlias`. Add the lowering logic in `lower::Ctx::lower_module_items()`. Crucially, only extract data visible to name resolution—generic parameter names yes, default values no. Run `cargo test -p hir-def` and verify that editing function bodies doesn't invalidate your new item type.
**Common Pitfalls:** Including expression data in ItemTree (breaks invalidation barrier). Storing AstPtr instead of FileAstId (loses file association). Not using the `mod_items!` macro and hand-rolling storage (inconsistent access patterns). Forgetting to handle your new item in `DefCollector` (name resolution won't see it). Not considering cfg-disabled items (use `AttrsOrCfg` pattern).
**Related Patterns in Ecosystem:** Similar staged-lowering patterns in rustc (HIR vs MIR vs LLVM IR), Swift compiler (parse → type-check → SIL), and query-based compilers (Kythe for cross-references without full type-checking). The invalidation barrier concept mirrors incrementalization in build systems (Shake, Redo).

---

## Pattern 5: ExpressionStore Split Pattern (Expr-Only + Types)
**File:** `crates/hir-def/src/expr_store.rs` (lines 97-125, 218-266)
**Category:** Memory Optimization, Arena Allocation
**Description:** Splits data storage into `ExpressionOnlyStore` (expressions, patterns, bindings) and shared type/lifetime arenas. Most stores (like generics) don't need expression data, so wrapping it in `Option<Box<>>` saves memory when unused.

**Code Example:**
```rust
#[derive(Debug, PartialEq, Eq)]
struct ExpressionOnlyStore {
    exprs: Arena<Expr>,
    pats: Arena<Pat>,
    bindings: Arena<Binding>,
    labels: Arena<Label>,
    binding_owners: FxHashMap<BindingId, ExprId>,
    block_scopes: Box<[BlockId]>,
    ident_hygiene: FxHashMap<ExprOrPatId, HygieneId>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ExpressionStore {
    expr_only: Option<Box<ExpressionOnlyStore>>,
    pub types: Arena<TypeRef>,
    pub lifetimes: Arena<LifetimeRef>,
}

#[derive(Debug, Eq, Default)]
pub struct ExpressionStoreBuilder {
    pub exprs: Arena<Expr>,
    pub pats: Arena<Pat>,
    pub bindings: Arena<Binding>,
    pub labels: Arena<Label>,
    pub lifetimes: Arena<LifetimeRef>,
    pub types: Arena<TypeRef>,
    // ... source maps and diagnostics
}

impl ExpressionStoreBuilder {
    pub fn finish(self) -> (ExpressionStore, ExpressionStoreSourceMap) {
        // Conditionally wrap expr_only data in Option<Box<>>
        // to save memory when no expressions exist
    }
}
```

**Why This Matters for Contributors:** This pattern shows how rust-analyzer optimizes for common cases: generic parameter lists contain types but rarely expressions. By splitting the store and using `Option<Box<>>`, we avoid allocating expression arenas for type-only contexts. When adding new HIR data structures, consider similar splits based on usage patterns. The builder pattern with `finish()` is the standard way to construct these split structures.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★☆
**Pattern Classification:** Performance/Structural
**Rust-Specific Insight:** This exploits Rust's zero-cost Option representation—`Option<Box<T>>` is the same size as a nullable pointer. The split recognizes that 90% of HIR contexts (trait bounds, type aliases, const generics) never have expressions. Arena<T> allocation is cheap (~24 bytes overhead per arena), but multiplied by thousands of items, it matters. The builder pattern accumulates in flat arenas during construction, then conditionally boxes in `finish()`, giving a single allocation decision point.
**Contribution Tip:** When extending ExpressionStore with new data (e.g., inline const blocks), decide: does this appear in type signatures (lifetimes, types) or only in bodies (exprs, pats)? Add to the appropriate section. Use the builder during lowering, calling `finish()` exactly once when complete. For new HIR constructs, measure before splitting—profile memory usage with `cargo test --release` and check Arena allocations. The 80/20 rule applies: only split if >50% of cases don't need the data.
**Common Pitfalls:** Over-splitting into too many optional sections (complicates access). Not using Box (just Option<ExpressionOnlyStore> is huge). Forgetting to update both the store and builder when adding fields. Accessing `.expr_only.unwrap()` without checking (panics for type-only contexts—use pattern matching or `as_ref()`). Not updating PartialEq implementation (causes test failures).
**Related Patterns in Ecosystem:** Similar conditional-storage patterns in servo's style system (rule trees store selectors but not always computed values), rustc's Ty representation (interned types with optional adt_def), and game engines (entity components as sparse sets). The builder-to-immutable pattern mirrors typed-builder crate.

---

## Pattern 6: DefMap with Layered Resolution (DefMap + LocalDefMap)
**File:** `crates/hir-def/src/nameres.rs` (lines 118-206, 275-289)
**Category:** Name Resolution, Incremental Compilation
**Description:** Splits definition map into `DefMap` (stable, external-facing) and `LocalDefMap` (contains crate-local data like extern prelude). This prevents invalidation cascades when adding dependencies to a crate.

**Code Example:**
```rust
/// Parts of the def map that are only needed when analyzing code in the same crate.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct LocalDefMap {
    /// The extern prelude which contains all root modules of external crates that are in scope.
    extern_prelude: FxIndexMap<Name, (ModuleId, Option<ExternCrateId>)>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DefMap {
    krate: Crate,
    block: Option<BlockInfo>,
    pub root: ModuleId,
    pub modules: ModulesMap,
    prelude: Option<(ModuleId, Option<UseId>)>,
    macro_use_prelude: FxHashMap<Name, (MacroId, Option<ExternCrateId>)>,
    derive_helpers_in_scope: FxHashMap<AstId<ast::Item>, Vec<(Name, MacroId, ...)>>,
    pub macro_def_to_macro_id: FxHashMap<ErasedAstId, MacroId>,
    diagnostics: Vec<DefDiagnostic>,
    data: Arc<DefMapCrateData>,
}

impl std::ops::Index<ModuleId> for DefMap {
    type Output = ModuleData;
    fn index(&self, id: ModuleId) -> &ModuleData {
        self.modules.get(&id)
            .unwrap_or_else(|| panic!("ModuleId not found in ModulesMap"))
    }
}
```

**Why This Matters for Contributors:** This split prevents a common performance problem: when you add a dependency to `Cargo.toml`, only `LocalDefMap` (with extern prelude) needs invalidation, not the entire `DefMap`. Always put crate-local data (extern prelude, crate-specific settings) in `LocalDefMap`, and keep `DefMap` stable across dependency changes. This is critical for IDE responsiveness.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Architectural/Performance
**Rust-Specific Insight:** This is an invalidation-minimization pattern leveraging salsa's dependency tracking. The `extern_prelude` (which crates are available) changes when you add dependencies, but module-internal name resolution (what names exist in each module) doesn't. By separating them, salsa only re-runs queries depending on LocalDefMap when Cargo.toml changes, not the much larger DefMap. The Index impl makes DefMap transparent—`def_map[module_id]` works naturally while keeping ModulesMap internal.
**Contribution Tip:** When adding crate-level configuration (e.g., edition, feature flags, lang items), ask: "Does this change when I add a dependency or only when I edit code?" If the former, put it in LocalDefMap. If the latter, DefMap. Always access LocalDefMap through a separate query (`crate_def_map_local()`), not embedded in DefMap—this maintains the invalidation boundary. Test by adding a dependency and verifying DefMap queries don't re-run.
**Common Pitfalls:** Putting dependency-affected data in DefMap (massive invalidation cascade). Merging LocalDefMap into DefMap for "convenience" (destroys the performance benefit). Not using Arc for DefMap (gets cloned frequently). Forgetting to update both when adding module-scope features. Not maintaining the Index impl when changing ModulesMap structure.
**Related Patterns in Ecosystem:** Similar staged invalidation in rustc (crate metadata vs. MIR), buck2's dice (fine-grained deps), and Vite/webpack (module graph vs. bundle graph). The pattern generalizes to any compiler with "project-wide metadata" vs. "per-module analysis."

---

## Pattern 7: Resolver with Scope Stack Pattern
**File:** `crates/hir-def/src/resolver.rs` (lines 44-94, 136-275)
**Category:** Name Resolution, Scope Handling
**Description:** Uses a stack of scopes (`Vec<Scope>`) processed in reverse order. Each scope type (ExprScope, GenericParams, BlockScope) implements its own resolution logic. The resolver delegates to scopes until a match is found.

**Code Example:**
```rust
#[derive(Debug, Clone)]
pub struct Resolver<'db> {
    /// The stack of scopes, where the inner-most scope is the last item.
    scopes: Vec<Scope<'db>>,
    module_scope: ModuleItemMap<'db>,
}

#[derive(Debug, Clone)]
enum Scope<'db> {
    /// All the items and imported names of a module
    BlockScope(ModuleItemMap<'db>),
    /// Brings the generic parameters of an item into scope
    GenericParams { def: GenericDefId, params: Arc<GenericParams> },
    /// Local bindings
    ExprScope(ExprScope),
    /// Macro definition inside bodies
    MacroDefScope(MacroDefId),
}

impl<'db> Resolver<'db> {
    pub fn resolve_path_in_type_ns(
        &self,
        db: &dyn DefDatabase,
        path: &Path,
    ) -> Option<(TypeNs, Option<usize>, Option<ImportOrExternCrate>)> {
        let first_name = path.segments().first()?;

        // Try each scope in reverse order (innermost first)
        for scope in self.scopes() {
            match scope {
                Scope::GenericParams { params, def } => {
                    if *first_name == sym::Self_ {
                        return Some((TypeNs::SelfType(impl_), ...));
                    }
                    if let Some(id) = params.find_type_by_name(first_name, *def) {
                        return Some((TypeNs::GenericParam(id), ...));
                    }
                }
                Scope::BlockScope(m) => {
                    if let Some(res) = m.resolve_path_in_type_ns(db, path) {
                        return Some(res);
                    }
                }
                // ... more scope types
            }
        }
        self.module_scope.resolve_path_in_type_ns(db, path)
    }
}
```

**Why This Matters for Contributors:** The scope stack pattern is how rust-analyzer handles nested scopes (function parameters shadow module items, generic params shadow outer scopes). When adding new scope types (e.g., for inline const blocks), create a new `Scope` variant and implement resolution logic for it. The reverse iteration (`scopes()`) ensures inner scopes shadow outer ones. This pattern is replicated in value, type, and lifetime resolution.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Behavioral/Structural
**Rust-Specific Insight:** This is a textbook interpreter pattern adapted for name resolution. The Vec<Scope> represents the environment stack, and reverse iteration implements lexical scoping naturally. Using an enum for scope variants lets each scope type (ExprScope, GenericParams, BlockScope) carry its own data and resolution logic. The borrowing discipline is critical—the resolver borrows from DefDatabase, so the scopes hold `&'db` references, preventing accidental mutations while resolving.
**Contribution Tip:** When adding a new scope kind (e.g., `Scope::InlineConstBlock`), add the variant with necessary data (e.g., `{ body_id: BodyId, owner: DefId }`). Implement resolution for all namespaces (type_ns, value_ns, lifetime_ns) in the appropriate `resolve_path_in_*_ns` methods. Add scope-pushing logic in `Resolver::for_expr()` or wherever the new scope appears. Test shadowing behavior—inner scopes should always win.
**Common Pitfalls:** Forgetting to implement resolution for all three namespaces (type/value/macro). Iterating scopes in the wrong order (forward instead of reverse = outer scopes shadow inner). Not handling `Self` specially in impl/trait contexts. Leaking scope data across salsa query boundaries (Scope contains references, not owned data). Not cleaning up scopes when popping context (memory leaks in recursive resolution).
**Related Patterns in Ecosystem:** Similar scope-chain patterns in JavaScript engines (V8's context chain), Python's LEGB rule implementation, and any language with lexical scoping. Rustc's Rib-based resolver, chalk's environment stacks, and interpreters following SICP's environment model.

---

## Pattern 8: PerNs (Per-Namespace) Resolution Pattern
**File:** `crates/hir-def/src/per_ns.rs` (lines 1-165)
**Category:** Name Resolution, Namespace Handling
**Description:** Uses a struct containing three optional items (types, values, macros) to represent that Rust allows the same name in different namespaces. Provides combinators for filtering, merging, and extracting per-namespace data.

**Code Example:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Item<Def, Import = ImportId> {
    pub def: Def,
    pub vis: Visibility,
    pub import: Option<Import>,
}

pub type TypesItem = Item<ModuleDefId, ImportOrExternCrate>;
pub type ValuesItem = Item<ModuleDefId, ImportOrGlob>;
pub type MacrosItem = Item<MacroId, ImportOrExternCrate>;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct PerNs {
    pub types: Option<TypesItem>,
    pub values: Option<ValuesItem>,
    pub macros: Option<MacrosItem>,
}

impl PerNs {
    pub fn types(def: ModuleDefId, vis: Visibility, import: Option<ImportOrExternCrate>) -> PerNs {
        PerNs { types: Some(Item { def, vis, import }), values: None, macros: None }
    }

    pub fn both(types: ModuleDefId, values: ModuleDefId, vis: Visibility, ...) -> PerNs {
        PerNs {
            types: Some(Item { def: types, vis, import }),
            values: Some(Item { def: values, vis, ... }),
            macros: None,
        }
    }

    pub fn filter_visibility(self, mut f: impl FnMut(Visibility) -> bool) -> PerNs {
        PerNs {
            types: self.types.filter(|def| f(def.vis)),
            values: self.values.filter(|def| f(def.vis)),
            macros: self.macros.filter(|def| f(def.vis)),
        }
    }

    pub fn or(self, other: PerNs) -> PerNs {
        PerNs {
            types: self.types.or(other.types),
            values: self.values.or(other.values),
            macros: self.macros.or(other.macros),
        }
    }
}
```

**Why This Matters for Contributors:** PerNs encodes Rust's namespace rules at the type level. When resolving names, always return `PerNs`, not individual items. The `or()` combinator merges results from different scopes, `filter_visibility()` applies visibility rules, and `both()` handles items like structs (both type and constructor). Never manually construct separate type/value/macro results; use PerNs to keep them together and maintain import tracking.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Structural/Behavioral
**Rust-Specific Insight:** This pattern uses Rust's type system to prevent a category of bugs—forgetting to check a namespace. Instead of three separate lookups (types_map.get(), values_map.get(), macros_map.get()), PerNs bundles them. The combinator methods (or, filter_visibility, take_types) provide point-free style composition, avoiding verbose match statements. The Item<Def, Import> inner type is clever—it tracks not just the definition but also how it was imported (direct import vs. glob), crucial for cycle detection and diagnostic messages.
**Contribution Tip:** When writing resolution code, always return PerNs::default() for "not found" rather than None. Use the builder methods (PerNs::types(), PerNs::both()) rather than constructing the struct directly—they ensure you don't forget visibility or import tracking. When merging scopes, use `.or()`—it implements the shadowing rule (prefer local scope). For filtering, chain combinators: `per_ns.filter_visibility(|v| ...).take_types()`.
**Common Pitfalls:** Returning only types when a name exists in multiple namespaces (e.g., struct Foo also has constructor Foo). Forgetting to track imports (breaks "find all references"). Using `PerNs { types: Some(...), values: None, macros: None }` instead of `PerNs::types(...)` (verbose and error-prone). Not propagating visibility correctly (allows seeing private items). Assuming PerNs is always non-empty (check with `.is_none()`).
**Related Patterns in Ecosystem:** Similar multi-namespace patterns in TypeScript (value/type/namespace), C++ (type/value/template), and OCaml (value/type/module). The combinator style mirrors Option/Result APIs. Rustc uses separate Res types per namespace; PerNs unifies them.

---

## Pattern 9: AttrFlags Bitflags Pattern
**File:** `crates/hir-def/src/attrs.rs` (lines 1-200)
**Category:** Memory Optimization, Attribute Handling
**Description:** Uses bitflags to compactly store presence/absence of attributes. Most attributes are binary flags; only a few need full data extraction (via separate queries). Wrapper methods check flags before calling expensive queries.

**Code Example:**
```rust
bitflags! {
    pub struct AttrFlags: u64 {
        const IS_DEPRECATED = 1 << 0;
        const IS_UNSTABLE = 1 << 1;
        const HAS_CFG = 1 << 2;
        const HAS_REPR = 1 << 3;
        const FUNDAMENTAL = 1 << 4;
        const LANG_ITEM = 1 << 5;
        const IS_DOC_HIDDEN = 1 << 6;
        const HAS_DOC_ALIASES = 1 << 7;
        // ... many more flags
    }
}

fn match_attr_flags(attr_flags: &mut AttrFlags, attr: Meta) -> ControlFlow<Infallible> {
    match attr {
        Meta::Path { path } => {
            match path.segments[0].text() {
                "fundamental" => attr_flags.insert(AttrFlags::FUNDAMENTAL),
                "no_std" => attr_flags.insert(AttrFlags::IS_NO_STD),
                _ => {}
            }
        },
        Meta::TokenTree { path, tt } => {
            match path.segments[0].text() {
                "deprecated" => attr_flags.insert(AttrFlags::IS_DEPRECATED),
                "repr" => attr_flags.insert(AttrFlags::HAS_REPR),
                "doc" => extract_doc_tt_attr(attr_flags, tt),
                _ => {}
            }
        },
        // ...
    }
    ControlFlow::Continue(())
}

// Wrapper pattern: check flag before calling expensive query
impl StructSignature {
    pub fn repr(&self, db: &dyn DefDatabase, id: StructId) -> Option<ReprOptions> {
        if self.flags.contains(StructFlags::HAS_REPR) {
            AttrFlags::repr(db, id.into())  // Only query if flag is set
        } else {
            None
        }
    }
}
```

**Why This Matters for Contributors:** This pattern saves memory (64 bits vs. storing full attribute data) and prevents expensive queries when attributes are absent. Always define a bitflag first, then add a separate query for extracting attribute data. The wrapper pattern (`if flag { query() } else { None }`) prevents salsa from tracking the query when the attribute isn't present, improving incremental performance. This pattern appears in AttrFlags, StructFlags, EnumFlags, etc.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Performance/Structural
**Rust-Specific Insight:** This is a brilliant application of the bitflags crate for sparse boolean data. Most items have 0-2 attributes; storing them in a u64 bitset is vastly more efficient than HashMap<String, AttributeValue>. The key innovation is the two-tier query pattern: flags are always computed (cheap), but attribute payloads (like #[repr(C, align(8))]) are only queried when the flag is set. The ControlFlow<Infallible> return in `match_attr_flags` is a type-safe way to signal early exit without Result overhead.
**Contribution Tip:** When adding a new attribute (e.g., `#[track_caller]`), extend the AttrFlags bitfield with a new constant, add a match arm in `match_attr_flags()` to set the flag, and create a separate `#[salsa::tracked]` query to extract data if needed. Use the wrapper pattern in the signature type: `if self.flags.contains(TRACK_CALLER) { query_track_caller_data(db, id) } else { None }`. This prevents salsa from creating dependencies when the attribute is absent.
**Common Pitfalls:** Forgetting to set the flag during attribute parsing (query will never run). Calling the full query without checking the flag first (creates unnecessary salsa edges). Using more than 64 flags (need AttrFlags2). Not handling multi-word attributes correctly (e.g., `#[doc(hidden)]` has both HAS_DOC and IS_DOC_HIDDEN). Comparing bitflags with `==` instead of `.contains()`.
**Related Patterns in Ecosystem:** Similar bitflag optimizations in rustc (ty::TypeFlags), Linux kernel (capability sets), and game engines (component masks). The two-tier query pattern mirrors lazy evaluation in Haskell (WHNF vs. full normalization) and database indices (bitmap index + detail fetch).

---

## Pattern 10: DynMap for Heterogeneous AST Mappings
**File:** `crates/hir-def/src/dyn_map.rs` (lines 1-200)
**Category:** Type-Safe Heterogeneous Collections, AST-to-HIR Mapping
**Description:** Uses a type-level key pattern to store multiple hash maps of different types in a single container. Keys define the map type at compile time, ensuring type safety without generics on the container.

**Code Example:**
```rust
pub struct Key<K, V, P = (K, V)> {
    _phantom: PhantomData<(K, V, P)>,
}

impl<K, V, P> Key<K, V, P> {
    pub(crate) const fn new() -> Key<K, V, P> {
        Key { _phantom: PhantomData }
    }
}

pub trait Policy {
    type K;
    type V;
    fn insert(map: &mut DynMap, key: Self::K, value: Self::V);
    fn get<'a>(map: &'a DynMap, key: &Self::K) -> Option<&'a Self::V>;
}

#[derive(Default)]
pub struct DynMap {
    pub(crate) map: Map,  // Type-erased anymap
}

impl<P: Policy> Index<Key<P::K, P::V, P>> for DynMap {
    type Output = KeyMap<Key<P::K, P::V, P>>;
    fn index(&self, _key: Key<P::K, P::V, P>) -> &Self::Output {
        // Safe due to #[repr(transparent)]
        unsafe { std::mem::transmute(self) }
    }
}

// Usage:
pub const STRUCT: Key<ast::Struct, StructId> = Key::new();
pub const FUNCTION: Key<ast::Fn, FunctionId> = Key::new();

let mut map = DynMap::new();
map[STRUCT].insert(struct_ast_ptr, struct_id);
map[FUNCTION].insert(fn_ast_ptr, fn_id);
```

**Why This Matters for Contributors:** DynMap allows storing multiple AST-to-ID mappings in one structure without generic type parameters. Each constant key defines its own map type. When adding new item types, define a new key constant (e.g., `pub const TRAIT_ALIAS: Key<ast::TraitAlias, TraitAliasId> = Key::new()`). The Policy trait and transmute pattern ensure type safety at compile time while using a type-erased container at runtime. This is critical for mapping syntax back to HIR.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★☆
**Pattern Classification:** Structural/Advanced
**Rust-Specific Insight:** This is a type-safe heterogeneous container using compile-time keys and runtime type erasure. The Key<K, V, P> is zero-sized (PhantomData), so keys are compile-time constants. The Index impl performs a transmute that's safe due to #[repr(transparent)]—DynMap and KeyMap<K> have identical representation, but the KeyMap<K> wrapper provides type-safe access. The Policy trait abstracts over storage strategies (e.g., single value vs. Vec<V> for one-to-many mappings).
**Contribution Tip:** When adding a new AST-to-HIR mapping, define a key constant in the keys module: `pub const YOUR_ITEM: Key<ast::YourItem, YourItemId> = Key::new();`. Use it like `dyn_map[YOUR_ITEM].insert(ast_ptr, id)`. If you need one-to-many mappings (one AST node → multiple IDs), define a custom Policy with Vec storage. Always use the key constant, never construct Key directly at call sites—the const folding ensures zero runtime cost.
**Common Pitfalls:** Defining keys as `static` instead of `const` (prevents const propagation). Not understanding the transmute safety invariant (breaks if DynMap's layout changes). Using DynMap for frequently-changing data (better to use a plain HashMap<TypeId, Box<dyn Any>>). Trying to iterate over all maps in DynMap (impossible by design—type-erased). Not using #[repr(transparent)] on wrapper types.
**Related Patterns in Ecosystem:** Similar heterogeneous map patterns in bevy_ecs (component storage), any-map crate, rustc's TypedArena with multiple element types, and anymap crate. The compile-time key pattern is used in tower's Service layers. TypeScript's mapped types provide similar type-level safety.

---

## Pattern 11: Signature Queries with Source Map Split
**File:** `crates/hir-def/src/signatures.rs` (lines 1-200)
**Category:** Salsa Queries, AST-to-HIR Mapping
**Description:** Every signature type has two queries: one returning just the signature (tracked/cached), and one returning `(Signature, SourceMap)`. The tracked query delegates to the full query and extracts the first element, avoiding duplicate computation.

**Code Example:**
```rust
#[derive(Debug, PartialEq, Eq)]
pub struct StructSignature {
    pub name: Name,
    pub generic_params: Arc<GenericParams>,
    pub store: Arc<ExpressionStore>,
    pub flags: StructFlags,
    pub shape: FieldsShape,
}

impl StructSignature {
    pub fn query(db: &dyn DefDatabase, id: StructId)
        -> (Arc<Self>, Arc<ExpressionStoreSourceMap>)
    {
        let loc = id.lookup(db);
        let source = loc.source(db);
        let attrs = AttrFlags::query(db, id.into());

        let (store, generic_params, source_map) = lower_generic_params(
            db, loc.container, id.into(), file_id,
            source.generic_param_list(), source.where_clause()
        );

        (
            Arc::new(StructSignature { generic_params, store, flags, shape, name }),
            Arc::new(source_map),
        )
    }
}

// In db.rs:
#[salsa::tracked]
fn struct_signature(&self, struct_: StructId) -> Arc<StructSignature> {
    self.struct_signature_with_source_map(struct_).0
}

#[salsa::invoke(StructSignature::query)]
fn struct_signature_with_source_map(
    &self,
    struct_: StructId,
) -> (Arc<StructSignature>, Arc<ExpressionStoreSourceMap>);
```

**Why This Matters for Contributors:** This pattern prevents duplicate work: the IDE layer often needs source maps (to jump to definitions), but type checking doesn't. The `#[salsa::tracked]` query caches just the signature, while the full query returns both. The tracked query extracts `.0` from the full query, so there's only one computation. When adding new signature types, always follow this pattern: implement `YourSignature::query()` returning a tuple, then add two database methods (one tracked, one with invoke).

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Performance/Architectural
**Rust-Specific Insight:** This exploits salsa's memoization to avoid duplicating computation while providing two APIs. The tracked query (`struct_signature`) is the common case—it caches the Arc<StructSignature> and only recomputes when inputs change. The invoke query (`struct_signature_with_source_map`) is the full version. By making the tracked query call the full version and extract `.0`, salsa ensures only one actual computation happens. Arc is critical here—cloning Arc<T> is cheap (just increment refcount), so returning the same Arc from both queries is essentially free.
**Contribution Tip:** When implementing a new signature type (e.g., TraitAliasSignature), structure it as: (1) Define the signature struct with all semantic data. (2) Implement `TraitAliasSignature::query(db, id) -> (Arc<Self>, Arc<SourceMap>)` doing the lowering. (3) Add to DefDatabase: one query with `#[salsa::tracked] fn trait_alias_signature() -> Arc<TraitAliasSignature> { self.trait_alias_signature_with_source_map().0 }` and one with `#[salsa::invoke(TraitAliasSignature::query)]`. Always return Arc to enable sharing.
**Common Pitfalls:** Implementing the signature query as tracked instead of invoke (duplicate computation). Not using Arc (expensive clones). Returning different data from the two queries (breaks the invariant that one is a projection of the other). Forgetting to update both queries when adding signature fields. Not making the SourceMap separate (couples IDE and type-checking).
**Related Patterns in Ecosystem:** Similar split-API patterns in rustc (queries returning (data, spans) tuples), databases (query with/without EXPLAIN), and GraphQL (fields with resolvers that return related data). The Arc-based sharing mirrors flyweight pattern and string interning.

---

## Pattern 12: Visibility Resolution with Module Hierarchy Walking
**File:** `crates/hir-def/src/visibility.rs` (lines 1-150)
**Category:** Visibility Checking, Module Traversal
**Description:** Implements visibility checking by walking the module hierarchy using DefMap, checking if `from_module` is a descendant of `to_module`. Handles both regular modules and block modules with special care for block expression boundaries.

**Code Example:**
```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Visibility {
    Module(ModuleId, VisibilityExplicitness),
    PubCrate(Crate),
    Public,
}

impl Visibility {
    pub fn is_visible_from(self, db: &dyn DefDatabase, from_module: ModuleId) -> bool {
        let to_module = match self {
            Visibility::Module(m, _) => m,
            Visibility::PubCrate(krate) => return from_module.krate(db) == krate,
            Visibility::Public => return true,
        };

        if from_module == to_module {
            return true;
        }

        if from_module.krate(db) != to_module.krate(db) {
            return false;
        }

        let def_map = from_module.def_map(db);
        Self::is_visible_from_def_map_(db, def_map, to_module, from_module)
    }

    fn is_visible_from_def_map_(
        db: &dyn DefDatabase,
        def_map: &DefMap,
        mut to_module: ModuleId,
        mut from_module: ModuleId,
    ) -> bool {
        // Walk up from from_module to see if we reach to_module
        loop {
            if from_module == to_module {
                return true;
            }
            match def_map[from_module].parent {
                Some(parent) => from_module = parent,
                None => {
                    match def_map.parent() {
                        Some(module) => {
                            def_map = module.def_map(db);
                            from_module = module;
                        }
                        None => return false,
                    }
                }
            }
        }
    }
}
```

**Why This Matters for Contributors:** This pattern handles Rust's complex visibility rules including `pub(in path)`, `pub(crate)`, and `pub(super)`. The key insight is that visibility is a tree relationship: an item is visible if you're in a descendant module. The algorithm walks up from `from_module` checking ancestors. When adding new module-like constructs (like inline const blocks), ensure they properly integrate with this visibility walking. The special handling of block modules (checking `def_map.parent()`) is critical for correctness.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Behavioral/Algorithmic
**Rust-Specific Insight:** This implements the Rust visibility model as a tree-traversal problem. The enum design (Visibility::Module vs PubCrate vs Public) maps directly to Rust's syntax and provides O(1) checks for the common cases (Public always visible, different crates always invisible). The hierarchical walk handles the subtle case of nested modules and block expressions—DefMap has a parent chain that must be followed. VisibilityExplicitness tracks whether `pub(in path)` was explicit or inferred, important for diagnostics.
**Contribution Tip:** When adding syntax for visibility (e.g., `pub(friend)` for a hypothetical feature), extend the Visibility enum with a new variant and implement the `is_visible_from` logic for it. Test edge cases: same module (always visible), different crates (check PubCrate), nested blocks (ensure parent chain walking works), and glob imports (visibility of the glob itself matters). Use `def_map[module_id].parent` to walk the tree, falling back to `def_map.parent()` for block expressions.
**Common Pitfalls:** Not handling block modules (they have separate DefMaps). Forgetting the krate check early (cross-crate visibility can return early). Infinite loops if the module tree has cycles (shouldn't happen, but defensive coding helps). Not preserving VisibilityExplicitness (breaks error messages). Comparing ModuleIds from different DefMaps without normalizing.
**Related Patterns in Ecosystem:** Similar tree-traversal patterns in rustc (def_path walking), file system ACLs (permission inheritance), DOM event bubbling (capture/bubble phases), and CSS specificity (ancestor selector matching). The early-exit optimization mirrors short-circuit evaluation in logic programming.

---

## Pattern 13: ItemScope with Separate Import Tracking
**File:** `crates/hir-def/src/item_scope.rs` (lines 1-200)
**Category:** Name Resolution, Import Handling
**Description:** ItemScope stores items in per-namespace maps (types/values/macros), but tracks imports separately in `use_imports_*` maps. This allows distinguishing between locally-declared items and imported items, which is crucial for re-exports and glob resolution.

**Code Example:**
```rust
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ItemScope {
    /// Defs visible in this scope (includes both declarations and imports)
    types: FxIndexMap<Name, TypesItem>,
    values: FxIndexMap<Name, ValuesItem>,
    macros: FxIndexMap<Name, MacrosItem>,
    unresolved: FxHashSet<Name>,

    /// Declarations (not imports)
    declarations: ThinVec<ModuleDefId>,

    impls: ThinVec<(ImplId, bool)>,
    unnamed_consts: ThinVec<ConstId>,
    unnamed_trait_imports: ThinVec<(TraitId, Item<()>)>,

    /// Separate tracking of import resolutions
    use_imports_types: FxHashMap<ImportOrExternCrate, ImportOrDef>,
    use_imports_values: FxHashMap<ImportOrGlob, ImportOrDef>,
    use_imports_macros: FxHashMap<ImportOrExternCrate, ImportOrDef>,

    use_decls: ThinVec<UseId>,
    extern_crate_decls: ThinVec<ExternCrateId>,

    /// Legacy textual scope macros
    legacy_macros: FxHashMap<Name, SmallVec<[MacroId; 2]>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Item<Def, Import = ImportId> {
    pub def: Def,
    pub vis: Visibility,
    pub import: Option<Import>,
}
```

**Why This Matters for Contributors:** The dual storage (items + import tracking) is essential for name resolution. When you resolve `use foo::bar`, you need to know where `bar` came from (local declaration vs. glob import). The `use_imports_*` maps track this. When implementing import resolution, always update both: add to `types`/`values`/`macros` for visibility, and add to `use_imports_*` for provenance tracking. This pattern also explains why `Item<Def, Import>` exists: it bundles the definition with optional import information.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Structural/Behavioral
**Rust-Specific Insight:** This is a sophisticated data structure that separates "what names exist" (types/values/macros maps) from "how they got here" (use_imports_* maps). The separation is critical for glob expansion—when you `pub use inner::*`, you need to re-export only the imported items from inner, not items declared in the current module. The `Item<Def, Import>` type is a sum-type encoding: `import: Option<ImportId>` tells you if this was imported (Some) or locally declared (None). ThinVec for declarations optimizes the common case (few declarations per module).
**Contribution Tip:** When handling import resolution (e.g., for `use` statements), update both data structures: (1) Insert into `types`/`values`/`macros` with `Item { def, vis, import: Some(import_id) }`. (2) Track the resolution in `use_imports_types[import_id] = ImportOrDef::Def(def)`. For glob imports, mark them as `ImportOrDef::Import(glob_source)` to enable re-export chains. When resolving names, check both maps—`types.get(name)` for the definition, `use_imports_types` for import tracking.
**Common Pitfals:** Only updating one of the maps (breaks "find all references" or glob expansion). Not distinguishing `ImportOrDef::Def` vs `ImportOrDef::Import` (flattens import chains, losing diagnostic info). Using Vec instead of ThinVec for rarely-populated fields (wastes memory). Forgetting to handle `unnamed_trait_imports` (for trait method resolution). Not marking glob imports specially in `use_imports_values` (type: `ImportOrGlob`).
**Related Patterns in Ecosystem:** Similar provenance-tracking patterns in package managers (direct vs. transitive dependencies), build systems (source files vs. generated files), and version control (authored vs. cherry-picked commits). The dual-map structure mirrors database indices (data table + index tables).

---

## Pattern 14: HasModule Trait for Universal Module Access
**File:** `crates/hir-def/src/lib.rs` (lines 994-1207)
**Category:** Trait Design, Module Navigation
**Description:** Defines a `HasModule` trait implemented by all definition IDs, providing uniform access to the containing module. Uses manual implementations for associated items (which delegate to container.module()) vs. top-level items (which store module directly).

**Code Example:**
```rust
pub trait HasModule {
    /// Returns the enclosing module this thing is defined within.
    fn module(&self, db: &dyn DefDatabase) -> ModuleId;

    /// Returns the crate this thing is defined within.
    #[inline]
    fn krate(&self, db: &dyn DefDatabase) -> Crate {
        self.module(db).krate(db)
    }
}

// Generic impl for items with ItemLoc
impl<N, ItemId> HasModule for ItemId
where
    N: AstIdNode,
    ItemId: Lookup<Database = dyn DefDatabase, Data = ItemLoc<N>> + Copy,
{
    #[inline]
    fn module(&self, db: &dyn DefDatabase) -> ModuleId {
        self.lookup(db).container
    }
}

// Manual impl for associated items (requires indirection through container)
impl HasModule for FunctionId {
    #[inline]
    fn module(&self, db: &dyn DefDatabase) -> ModuleId {
        self.lookup(db).container.module(db)
    }
}

impl HasModule for AdtId {
    fn module(&self, db: &dyn DefDatabase) -> ModuleId {
        match *self {
            AdtId::StructId(it) => it.module(db),
            AdtId::UnionId(it) => it.module(db),
            AdtId::EnumId(it) => it.module(db),
        }
    }
}
```

**Why This Matters for Contributors:** HasModule provides uniform navigation regardless of definition type. When adding new ID types, implement HasModule appropriately: for items in modules use the generic impl via ItemLoc; for associated items (in traits/impls) implement manually with container indirection; for sum types (like AdtId) delegate to each variant. This trait is pervasive in visibility checking, path resolution, and crate navigation. Getting it right is critical for name resolution to work.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Structural/Creational
**Rust-Specific Insight:** This is a textbook extension trait pattern providing uniform interface across heterogeneous types. The generic impl for `ItemId: Lookup<Data = ItemLoc<N>>` is powerful—it automatically provides HasModule for any ID type that lookups to an ItemLoc, avoiding boilerplate. The manual impls for associated items (FunctionId in traits/impls) show when indirection is needed. The `#[inline]` annotations ensure the trait calls optimize to direct field access.
**Contribution Tip:** When adding a new ID type: (1) If it's a top-level item (in a module), use `type YourId = Lookup<ItemLoc<ast::YourItem>>` and the generic impl works automatically. (2) If it's an associated item (in a trait/impl), implement manually: `impl HasModule for YourId { fn module(&self, db) -> ModuleId { self.lookup(db).container.module(db) } }`. (3) For sum types (e.g., `YourEnumId { A | B | C }`), delegate: `match self { A(it) => it.module(db), ... }`. Always use `#[inline]`.
**Common Pitfalls:** Not implementing HasModule for a new ID type (compilation error in resolver). Forgetting the indirection for associated items (returns trait/impl ID instead of containing module). Not inlining (adds unnecessary function call overhead). Implementing for the Loc type instead of the ID type (wrong abstraction level). Not handling all variants in sum-type delegation.
**Related Patterns in Ecosystem:** Similar uniform-interface patterns in trait objects (dyn Trait), iterator adapters (impl Iterator for all), and From/Into blanket impls. The generic impl mirrors Rust's coherence rules. Comparable to Java's interface default methods and Haskell's type classes with default implementations.

---

## Pattern 15: Arena-based ID Types with Type Safety
**File:** `crates/hir-def/src/hir.rs` (lines 40-77, 198-200) and `crates/hir-def/src/expr_store.rs` (lines 100-123)
**Category:** Arena Allocation, Type-Safe IDs
**Description:** Uses `la_arena::Idx<T>` to create strongly-typed IDs for arena-allocated data. Each ID type is a newtype alias, preventing mixing of different ID types while maintaining zero-cost abstractions.

**Code Example:**
```rust
pub type BindingId = Idx<Binding>;
pub type ExprId = Idx<Expr>;
pub type PatId = Idx<Pat>;
pub type LabelId = Idx<Label>;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ExprOrPatId {
    ExprId(ExprId),
    PatId(PatId),
}

impl ExprOrPatId {
    pub fn as_expr(self) -> Option<ExprId> {
        match self {
            Self::ExprId(v) => Some(v),
            _ => None,
        }
    }

    pub fn is_expr(&self) -> bool {
        matches!(self, Self::ExprId(_))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ExpressionOnlyStore {
    exprs: Arena<Expr>,
    pats: Arena<Pat>,
    bindings: Arena<Binding>,
    labels: Arena<Label>,
    binding_owners: FxHashMap<BindingId, ExprId>,
    ident_hygiene: FxHashMap<ExprOrPatId, HygieneId>,
}
```

**Why This Matters for Contributors:** Arena-based IDs are foundational to HIR. Each `Idx<T>` is just a `u32` wrapper, but the type parameter prevents you from using a `PatId` where an `ExprId` is expected. When building HIR nodes, allocate in arenas (`exprs.alloc(expr)` returns `ExprId`) and store IDs, never references. The `ExprOrPatId` enum shows how to handle cases where either type is valid (like destructuring assignments). This pattern appears in Body, GenericParams, and every other arena-based data structure.

### Rust Expert Commentary
**Idiomatic Rating:** ★★★★★
**Pattern Classification:** Structural/Performance
**Rust-Specific Insight:** Arena allocation is a key Rust pattern for tree-like data structures. Instead of `Box<Expr>` (scattered heap allocations, pointer chasing), arenas allocate all Expr nodes contiguously in a Vec. `Idx<T>` is type-safe indexing—it's just `u32` internally but statically prevents `exprs[pat_id]` type errors. The zero-cost abstraction is perfect: no runtime overhead vs raw indices, but compile-time safety. The `ExprOrPatId` sum type handles Rust's "either pattern or expression" cases (like in `let` bindings).
**Contribution Tip:** When building HIR, always use arenas for recursive types. Create the arena (`Arena::<Expr>::new()`), allocate nodes (`let id = exprs.alloc(Expr::Block(...))`), and store IDs in parent nodes. For union types (expr-or-pat, type-or-const), define a sum enum like `ExprOrPatId`. Provide accessor methods (`as_expr()`, `is_expr()`) for ergonomic pattern matching. Never store references—arenas don't support borrowing while mutating.
**Common Pitfalls:** Storing `&Expr` instead of `ExprId` (lifetime errors). Mixing IDs from different arenas (runtime panic or wrong data). Not using Idx<T> (loses type safety—`u32` indices are error-prone). Trying to mutate arena while holding IDs (safe but requires mutable lookups). Creating circular references via IDs (possible but usually indicates design issue—HIR should be acyclic).
**Related Patterns in Ecosystem:** Similar arena patterns in rustc (HIR, MIR, Ty interning), servo (script and style tree arenas), salsa (interned data), and game engines (entity-component-systems with generational indices). The type-safe ID pattern mirrors newtype indices in ECS frameworks like hecs and bevy.

---

## Summary for Contributors

The hir-def crate implements the definition and name resolution layer between macro expansion (hir-expand) and type inference (hir-ty). Key architectural principles:

1. **Incremental by Design**: ItemTree and DefMap are carefully designed as invalidation barriers
2. **Memory Optimization**: Bitflags for attributes, split stores for expressions vs types, Arc for sharing
3. **ID-based Architecture**: Everything uses interned IDs (StructId, FunctionId, etc.) via salsa
4. **Per-Namespace Resolution**: PerNs encodes Rust's type/value/macro namespaces
5. **Signature/Body Split**: Signatures (without bodies) allow name resolution without type inference

When contributing:
- Follow the impl_intern! pattern for new definition types
- Use salsa::tracked for cached queries, transparent for zero-cost
- Maintain ItemTree as signature-only (no expression bodies)
- Always return PerNs from resolution, never raw items
- Implement HasModule for new ID types
- Use arena-based IDs (Idx<T>) for HIR data structures

---

## Key Patterns for Contributors: Quick Reference

### Essential Patterns (Must Know)
1. **Salsa Interned ID Pattern** - Foundation for all definition types
2. **ItemTree as Invalidation Barrier** - Core to performance
3. **PerNs Resolution** - Rust's namespace model encoded
4. **Signature/SourceMap Split** - API efficiency pattern

### Performance-Critical Patterns
5. **Bitflags for Attributes** - Memory optimization (64 bits vs. full storage)
6. **ExpressionStore Split Pattern** - Conditional allocation (Option<Box<>>)
7. **DefMap/LocalDefMap Split** - Invalidation minimization
8. **Arena-based IDs** - Cache-friendly allocation + type safety

### Navigation Patterns
9. **HasModule Trait** - Uniform module access across all IDs
10. **Resolver with Scope Stack** - Lexical scoping implementation
11. **Visibility Resolution** - Tree traversal for access checking

### Data Structure Patterns
12. **DynMap for Heterogeneous Mappings** - Type-safe AST-to-HIR
13. **ItemScope with Import Tracking** - Dual storage (items + provenance)
14. **Generic ItemLoc** - Bound-minimal generic wrappers

### Query Patterns
15. **Database Trait with Query Groups** - Layered salsa architecture

---

## Contribution Readiness Checklist

### Before Adding a New Definition Type (e.g., TraitAliasId)

#### 1. Location Type Setup
- [ ] Define `type TraitAliasLoc = ItemLoc<ast::TraitAlias>` (or AssocItemLoc if associated)
- [ ] Use `impl_intern!(TraitAliasId, TraitAliasLoc, intern_trait_alias, lookup_intern_trait_alias)`
- [ ] Verify `TraitAliasId` is Copy and implements Lookup trait

#### 2. ItemTree Integration
- [ ] Add entry to `mod_items!` macro: `TraitAlias in small_data -> ast::TraitAlias`
- [ ] Implement lowering in `lower::Ctx::lower_module_items()`
- [ ] Store only signature-level data (no bodies, no default implementations)
- [ ] Handle cfg attributes via `AttrsOrCfg`
- [ ] Test that editing unrelated code doesn't invalidate ItemTree

#### 3. Signature Query Setup
- [ ] Create `TraitAliasSignature` struct with semantic data
- [ ] Implement `TraitAliasSignature::query(db, id) -> (Arc<Self>, Arc<SourceMap>)`
- [ ] Add to DefDatabase:
  - `#[salsa::tracked] fn trait_alias_signature() -> Arc<TraitAliasSignature>`
  - `#[salsa::invoke(TraitAliasSignature::query)] fn trait_alias_signature_with_source_map()`
- [ ] Ensure tracked query extracts `.0` from full query

#### 4. Name Resolution Integration
- [ ] Add to `ModuleDefId` enum if it's a module-level item
- [ ] Implement `HasModule` for `TraitAliasId`
- [ ] Update `PerNs` handling if needed (usually types namespace)
- [ ] Add to `ItemScope` storage (types/values/macros)
- [ ] Handle in `DefCollector::collect_*` methods

#### 5. DynMap Key (for AST mapping)
- [ ] Define `pub const TRAIT_ALIAS: Key<ast::TraitAlias, TraitAliasId> = Key::new()`
- [ ] Use in source map construction: `dyn_map[TRAIT_ALIAS].insert(ast_ptr, id)`

#### 6. Attribute Handling
- [ ] Add bitflags to AttrFlags if needed (e.g., `HAS_MARKER_ATTR`)
- [ ] Implement flag detection in `match_attr_flags()`
- [ ] Create separate query for extracting attribute data
- [ ] Use wrapper pattern: `if flags.contains(FLAG) { query() } else { None }`

#### 7. Testing Checklist
- [ ] Write ItemTree lowering test: `#[test] fn lower_trait_alias()`
- [ ] Test name resolution: can you reference the trait alias?
- [ ] Test visibility: pub vs. private trait aliases
- [ ] Test incremental: editing bodies doesn't invalidate
- [ ] Test with generics, where clauses, attributes
- [ ] Test cross-crate usage (if applicable)

### Code Quality Standards

#### Salsa Query Guidelines
- [ ] Use `#[salsa::tracked]` for cached, expensive queries
- [ ] Use `#[salsa::transparent]` for cheap getters/conversions
- [ ] Use `#[salsa::invoke]` when query logic is complex
- [ ] Always return `Arc<T>` for large structures
- [ ] Provide both `foo()` and `foo_with_source_map()` for HIR construction

#### Memory Optimization
- [ ] Use bitflags for boolean attributes (not HashMap<String, bool>)
- [ ] Use `ThinVec` for rarely-populated collections
- [ ] Use `Arc` for shared data (signatures, generic params)
- [ ] Use `Option<Box<>>` for conditionally-needed large data
- [ ] Use `FxHashMap` instead of `HashMap` (faster for int/ID keys)

#### Type Safety
- [ ] Never use raw `u32` indices—always `Idx<T>`
- [ ] Implement standard traits manually when derive adds unwanted bounds
- [ ] Use newtypes for domain concepts (ModuleId, not just u32)
- [ ] Define sum types (enums) for "either/or" cases (ExprOrPatId)
- [ ] Always `#[inline]` on trivial trait method impls

#### API Design
- [ ] Return `PerNs` from resolution, not individual items
- [ ] Implement `HasModule` for all ID types
- [ ] Follow naming: `foo()` for simple, `foo_with_source_map()` for IDE
- [ ] Use builder pattern for complex construction (ExpressionStoreBuilder)
- [ ] Provide combinator methods (filter_visibility, take_types, or)

### Common Anti-Patterns to Avoid

❌ **Don't**: Store expression bodies in ItemTree
✅ **Do**: Keep ItemTree signature-only for invalidation barrier

❌ **Don't**: Use `#[salsa::tracked]` for cheap conversions
✅ **Do**: Use `#[salsa::transparent]` for zero-cost delegation

❌ **Don't**: Return individual type/value/macro items separately
✅ **Do**: Always return `PerNs` to maintain namespace unity

❌ **Don't**: Derive traits on generic types without checking bounds
✅ **Do**: Manually implement to avoid propagating unnecessary bounds

❌ **Don't**: Use `Vec<bool>` or `HashMap<String, bool>` for attributes
✅ **Do**: Use bitflags (u64) for compact storage

❌ **Don't**: Clone large structures (signatures, bodies)
✅ **Do**: Wrap in Arc and clone the Arc (refcount increment)

❌ **Don't**: Store `&Expr` references
✅ **Do**: Use arena-based `ExprId` indices

❌ **Don't**: Mix IDs from different arenas
✅ **Do**: Use type-safe `Idx<Expr>` vs `Idx<Pat>`

❌ **Don't**: Put dependency-affected data in DefMap
✅ **Do**: Split into DefMap (stable) and LocalDefMap (dep-affected)

❌ **Don't**: Forget to implement HasModule
✅ **Do**: Implement for all ID types (manual or via generic impl)

---

## Pattern Relationship Map

```
Foundational Layer (Must understand first):
├─ Pattern 1: Salsa Interned ID
├─ Pattern 2: Generic ItemLoc
├─ Pattern 15: Arena-based IDs
└─ Pattern 3: Database Trait Pattern

Invalidation Architecture (Performance critical):
├─ Pattern 4: ItemTree as Barrier
├─ Pattern 6: DefMap/LocalDefMap Split
└─ Pattern 5: ExpressionStore Split

Name Resolution Core:
├─ Pattern 8: PerNs Resolution
├─ Pattern 7: Resolver with Scope Stack
├─ Pattern 12: Visibility Resolution
└─ Pattern 13: ItemScope with Import Tracking

Data Access Patterns:
├─ Pattern 11: Signature/SourceMap Split
├─ Pattern 9: AttrFlags Bitflags
├─ Pattern 10: DynMap Heterogeneous Storage
└─ Pattern 14: HasModule Trait

Integration Glue:
└─ All patterns work together in DefCollector/DefMap construction
```

---

## Learning Path for New Contributors

### Week 1: Foundation
1. Read Patterns 1, 2, 15 (ID systems)
2. Study Pattern 3 (Salsa queries)
3. Build mental model: "Everything is an interned ID"

### Week 2: Name Resolution
4. Read Patterns 7, 8, 13 (Resolver, PerNs, ItemScope)
5. Study Pattern 12 (Visibility)
6. Trace a name resolution query end-to-end

### Week 3: Performance Architecture
7. Read Patterns 4, 6 (Invalidation barriers)
8. Study Pattern 5, 9 (Memory optimization)
9. Run benchmarks, understand salsa profiling

### Week 4: Advanced Patterns
10. Read Patterns 10, 11, 14 (DynMap, Signatures, HasModule)
11. Review the anti-patterns section
12. Attempt a small contribution (add tests, fix bugs)

### Week 5: First Feature
13. Pick a small feature (e.g., support new attribute)
14. Follow the contribution checklist
15. Submit PR, iterate on feedback

---

## Debugging Tips

### When Name Resolution Fails
1. Check ItemTree: did your item get lowered?
2. Check DefCollector: is it being collected?
3. Check ItemScope: is it in the right namespace?
4. Check Visibility: is it accessible from the use site?
5. Check PerNs: are you returning all three namespaces?

### When Incremental Compilation is Slow
1. Verify ItemTree doesn't contain body data
2. Check DefMap/LocalDefMap split is maintained
3. Profile with `SALSA_LOG=1` environment variable
4. Look for unnecessary `#[salsa::tracked]` queries
5. Check if you're cloning large structures (use Arc)

### When Tests Fail
1. Run `cargo test -p hir-def --test test_name`
2. Check if you updated all necessary maps (types, values, macros)
3. Verify HasModule is implemented
4. Check if DynMap key is defined
5. Ensure signature query returns both data and source map

---

## Resources for Deeper Understanding

### Salsa Framework
- [Salsa Book](https://salsa-rs.github.io/salsa/)
- Key concept: Memoization with fine-grained dependencies
- Query groups = trait + storage derivation

### Rust Namespaces
- RFC 2126: Path clarity (type vs value namespace)
- Understand: type namespace (structs, enums, traits, type aliases)
- Understand: value namespace (functions, constants, constructors)
- Understand: macro namespace (macros)

### Arena Allocation
- [la-arena crate](https://docs.rs/la-arena)
- Benefits: cache locality, bulk deallocation, stable indices
- Drawback: can't remove individual items (append-only)

### Incremental Compilation Theory
- Self-adjusting computation (Acar et al.)
- Build systems à la carte (Mokhov et al.)
- Adapton: adaptive memoization

---

**Final Note**: The patterns in hir-def represent years of refinement for IDE-grade performance. When in doubt, follow existing code structure exactly. The patterns exist for reasons that may not be obvious until you profile at scale. Measure before optimizing, but respect the existing optimization patterns—they were learned through hard-won experience.
