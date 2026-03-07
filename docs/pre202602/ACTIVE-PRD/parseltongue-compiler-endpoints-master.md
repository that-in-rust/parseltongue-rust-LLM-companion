# Parseltongue Compiler Endpoints — Master Reference

**Version:** 1.0.0  
**Compiled:** March 3, 2026  
**Purpose:** Single authoritative reference for all compiler APIs that power the Parseltongue "super code-graph" for Rust.

**Parseltongue 5-Phase LLM-Queryable Code-Graph:**

| Phase | Name | Algorithm |
|-------|------|-----------|
| 1 | SEARCH | RRF across symbol / fuzzy / git |
| 2 | ANCHOR | BFS to public interface |
| 3 | CLUSTER | Ego network + PageRank |
| 4 | DISAMBIGUATE | LLM picks |
| 5 | DEEP DIVE | Full subgraph + type flow + CFG |

---

## PART 1: EXECUTIVE SUMMARY

### 1.1 Total APIs/Endpoints Catalogued

| Source | Category | Approximate API Count |
|--------|----------|----------------------|
| `ra_ap_hir` | High-level semantic model (Module, Function, Struct, Enum, Trait, Impl, Type) | ~120 methods |
| `ra_ap_hir_def` | Definition IDs, ItemTree, DefMap, Resolver, ItemScope | ~80 methods |
| `ra_ap_hir_ty` | Type inference, trait resolution, InferenceResult, TraitImpls | ~60 methods |
| `ra_ap_syntax` | CST/AST parsing, SyntaxNode traversal | ~40 methods |
| `ra_ap_ide` | IDE features: Analysis, call hierarchy, symbol search, references | ~50 methods |
| `ra_ap_base_db` / `ra_ap_vfs` | File primitives, CrateGraph, VFS | ~30 methods |
| `TyCtxt` (rustc) | Core query engine: types, traits, impls, HIR, MIR, spans | ~100+ queries |
| `rustc_middle::mir` | MIR Body, CFG, Place, Operand, Rvalue, TerminatorKind | ~80 types/methods |
| `rustc_mir_dataflow` | Dataflow framework, built-in analyses (liveness, borrows, init) | ~40 APIs |
| `rustc_data_structures` | Dominators, graph traits | ~15 APIs |
| Polonius facts | Borrow-check relations: loans, origins, subsets, liveness | 18 input facts + 5 derived |
| Flowistry | Information-flow PDG, alias analysis | ~10 key APIs |
| `rustc_public` (Stable MIR) | Stable API: items, bodies, types, traits | ~40 APIs |
| `rustc_plugin` / `rustc_utils` | Plugin framework + BodyExt, PlaceExt extensions | ~30 APIs |
| Charon (LLBC) | Full-crate JSON extraction: functions, types, traits | ~20 key types |

**Estimated total: ~700+ distinct API endpoints catalogued.**

### 1.2 Five Integration Patterns

| # | Pattern | Stability | Richness | Nightly Required? |
|---|---------|-----------|----------|-------------------|
| 1 | **Direct `rustc_private`** | Breaks every nightly | Maximum (full TyCtxt, MIR, Polonius) | Yes |
| 2 | **`rustc_plugin` (Brown framework)** | Nightly-pinned, managed | Maximum + Cargo integration | Yes |
| 3 | **Charon / LLBC** | Alpha; cleaned AST | High (resolved traits, structured CFG) | Yes (for extraction) |
| 4 | **`rustc_public` (Stable MIR)** | WIP toward crates.io stability | Medium-high (MIR, types, traits; no HIR, no Polonius) | Yes (for now) |
| 5 | **`ra_ap_*` (no compiler dependency)** | Semver'd on crates.io | Medium (HIR-level, no MIR, no borrow-check) | No |

### 1.3 Recommended Approach: Two-Layer Architecture

**Layer 1 — Fast Path (`ra_ap_*`):**
- Runs on stable Rust toolchain
- Provides entity discovery, call hierarchy, symbol search, type relationships, visibility, module tree
- Covers Phases 1–3 (SEARCH, ANCHOR, CLUSTER) with sub-second latency
- Version: `0.0.322+` on crates.io; follows rust-analyzer releases

**Layer 2 — Deep Mode (`rustc_private` via `rustc_plugin`):**
- Requires nightly toolchain with `rustc-dev` component
- Provides MIR-level call graph, data flow, control flow, Polonius borrow facts, monomorphization
- Covers Phase 5 (DEEP DIVE) with full compiler precision
- Fallback: Charon LLBC extraction when deep mode is unavailable

**Graceful degradation:** When Layer 2 is unavailable (wrong nightly, missing component), Parseltongue operates in "fast-only" mode with Layer 1 data. Phases 1–4 remain fully functional; Phase 5 returns partial results with `trust_grade: "heuristic"`.

---

## PART 2: ENTITY EXTRACTION ENDPOINTS

For each entity type, we list extraction APIs across all layers.

### 2.1 Functions

| Property | ra_ap_hir (Fast Path) | rustc TyCtxt (Deep Path) | Stable MIR (Future Path) |
|----------|----------------------|--------------------------|--------------------------|
| **Enumerate all** | `module.declarations(db)` → filter `ModuleDef::Function` | `tcx.hir_body_owners()` filtered by `DefKind::Fn \| DefKind::AssocFn` | `all_local_items()` filtered by `ItemKind::Fn` |
| **Name** | `fn.name(db) -> Name` | `tcx.def_path_str(def_id) -> String` | `CrateItem::name()` |
| **Kind** | `ModuleDef::Function` | `tcx.def_kind(def_id) -> DefKind::Fn / AssocFn` | `CrateItem::kind() -> ItemKind` |
| **Visibility** | `fn.visibility(db) -> Visibility` (via `HasVisibility`) | `tcx.visibility(def_id) -> Visibility<DefId>` | N/A (use internal bridge) |
| **Module path** | `fn.module(db).path_to_root(db)` | `tcx.def_path_str(def_id)` | N/A |
| **File/Span** | `fn.source(db) -> InFile<ast::Fn>` → `.file_id` + `.value.syntax().text_range()` | `tcx.def_span(def_id)` → `sm.lookup_char_pos()` | `CrateItem::span()` |
| **Async/Const/Unsafe** | `fn.is_async(db)`, `fn.is_const(db)`, `fn.is_unsafe_to_call(db, ..)` | `tcx.asyncness(def_id)`, `tcx.constness(def_id)`, `FnHeader::safety` | N/A |
| **Is test** | `fn.is_test(db)` | `attrs.iter().any(\|a\| a.has_name(sym::test))` | `CrateItem::all_tool_attrs()` |
| **Has body** | `fn.has_body(db)` | `tcx.is_mir_available(def_id)` | `CrateItem::expect_body()` (panics if none) |
| **Trust grade** | Verified (compiler semantic model) | Verified (full compiler) | Verified |

**Canonical key fields:** `{ kind: "function", scope: module_path, name: fn_name, file_path, visibility, is_async, is_const, is_unsafe, is_test }`

### 2.2 Structs

| Property | ra_ap_hir (Fast Path) | rustc TyCtxt (Deep Path) | Stable MIR (Future Path) |
|----------|----------------------|--------------------------|--------------------------|
| **Enumerate all** | `module.declarations(db)` → `ModuleDef::Adt(Adt::Struct(s))` | `tcx.hir_free_items()` → `ItemKind::Struct` | `all_local_items()` |
| **Name** | `s.name(db)` | `tcx.def_path_str(def_id)` | via `CrateItem` |
| **Fields** | `s.fields(db) -> Vec<Field>` | `tcx.adt_def(def_id).all_fields()` | via `TyKind::Adt` |
| **Field types** | `field.ty(db) -> Type` | `field.ty(tcx, generic_args)` | `LocalDecl::ty` |
| **Field visibility** | `field.visibility(db)` (via `HasVisibility`) | `field.vis: Visibility<DefId>` | N/A |
| **Generic params** | `GenericDef::from(s).type_or_const_params(db)` | `tcx.generics_of(def_id)` | via `GenericArgs` |
| **Kind (unit/tuple/record)** | `s.kind(db) -> StructKind` | `variant_data.fields()` shape | N/A |
| **Repr** | `s.repr(db)` | `tcx.adt_def(def_id).repr()` | N/A |
| **Trust grade** | Verified | Verified | Verified |

**Canonical key fields:** `{ kind: "struct", scope, name, file_path, visibility, field_count, generic_param_count }`

### 2.3 Enums

| Property | ra_ap_hir (Fast Path) | rustc TyCtxt (Deep Path) |
|----------|----------------------|--------------------------|
| **Enumerate all** | `ModuleDef::Adt(Adt::Enum(e))` | `DefKind::Enum` |
| **Variants** | `e.variants(db) -> Vec<Variant>` | `tcx.adt_def(def_id).variants()` |
| **Variant fields** | `variant.fields(db)` | `variant.fields: IndexVec<FieldIdx, FieldDef>` |
| **Discriminant** | `variant.eval(db)` | `variant.discr: VariantDiscr` |
| **Data-carrying?** | `e.is_data_carrying(db)` | Check variant field counts |
| **Trust grade** | Verified | Verified |

**Canonical key fields:** `{ kind: "enum", scope, name, file_path, visibility, variant_count }`

### 2.4 Traits

| Property | ra_ap_hir (Fast Path) | rustc TyCtxt (Deep Path) |
|----------|----------------------|--------------------------|
| **Enumerate all** | `ModuleDef::Trait(t)` from `module.declarations(db)` | `tcx.all_traits_including_private()` |
| **Associated items** | `t.items(db) -> Vec<AssocItem>` | `tcx.associated_items(trait_id)` |
| **Supertraits** | `t.direct_supertraits(db)` / `t.all_supertraits(db)` | `tcx.predicates_of(trait_id)` → `ClauseKind::Trait` |
| **All implementors** | `Impl::all_for_trait(db, t) -> Vec<Impl>` | `tcx.trait_impls_of(trait_id)` |
| **Is auto trait** | `t.is_auto(db)` | `tcx.trait_def(trait_id).is_auto` |
| **Is unsafe** | `t.is_unsafe(db)` | `tcx.trait_def(trait_id)` safety field |
| **Dyn compatibility** | `t.dyn_compatibility(db)` | `tcx.is_dyn_compatible(trait_id)` |
| **Trust grade** | Verified | Verified |

**Canonical key fields:** `{ kind: "trait", scope, name, file_path, visibility, assoc_item_count, supertrait_count, is_auto, is_unsafe }`

### 2.5 Impl Blocks

| Property | ra_ap_hir (Fast Path) | rustc TyCtxt (Deep Path) |
|----------|----------------------|--------------------------|
| **Enumerate all** | `Impl::all_in_crate(db, krate)` or `module.impl_defs(db)` | `tcx.hir_free_items()` → `ItemKind::Impl` |
| **Self type** | `impl_.self_ty(db) -> Type` | `tcx.impl_self_ty(impl_id)` |
| **Trait (if trait impl)** | `impl_.trait_(db) -> Option<Trait>` | `tcx.impl_trait_header(impl_id)` |
| **Items** | `impl_.items(db) -> Vec<AssocItem>` | `tcx.associated_items(impl_id)` |
| **Is negative** | `impl_.is_negative(db)` | `tcx.impl_polarity(impl_id)` |
| **Trust grade** | Verified | Verified |

### 2.6 Modules

| Property | ra_ap_hir (Fast Path) | rustc TyCtxt (Deep Path) |
|----------|----------------------|--------------------------|
| **Enumerate all** | `krate.modules(db)` | `tcx.hir_module_items(mod_def_id)` |
| **Children** | `module.children(db)` | `tcx.module_children(module_id)` |
| **Parent** | `module.parent(db)` | `tcx.parent_module_from_def_id(def_id)` |
| **Declarations** | `module.declarations(db) -> Vec<ModuleDef>` | `tcx.hir_module_items(mod_def_id)` |
| **Scope (visible names)** | `module.scope(db, visible_from)` | `tcx.module_children(module_id)` |
| **Trust grade** | Verified | Verified |

### 2.7 Type Aliases, Constants, Statics, Macros

| Entity | ra_ap_hir Extraction | rustc Extraction |
|--------|---------------------|------------------|
| **Type Alias** | `ModuleDef::TypeAlias` → `.name(db)`, `.ty(db)` | `DefKind::TyAlias` → `tcx.type_of(def_id)` |
| **Const** | `ModuleDef::Const` → `.name(db)`, `.ty(db)` | `DefKind::Const` → `tcx.type_of(def_id)` |
| **Static** | `ModuleDef::Static` → `.name(db)`, `.ty(db)` | `DefKind::Static { .. }` → `tcx.type_of(def_id)` |
| **Macro** | `ModuleDef::Macro` → `.name(db)`, `.kind(db)` | `DefKind::Macro(MacroKind)` |

---

## PART 3: RELATIONSHIP/EDGE EXTRACTION ENDPOINTS

### 3.1 Call Graph Edges

#### 3.1.1 ra_ap_ide Call Hierarchy (Fast Path)

```rust
// Source: ra_ap_ide::Analysis
// Docs: https://docs.rs/ra_ap_ide/latest/ra_ap_ide/struct.Analysis.html

// Entry point — get NavigationTargets for the function at position
analysis.call_hierarchy(&config, position) -> Cancellable<Option<RangeInfo<Vec<NavigationTarget>>>>

// Incoming callers — who calls this function?
analysis.incoming_calls(&config, position) -> Cancellable<Option<Vec<CallItem>>>

// Outgoing callees — what does this function call?
analysis.outgoing_calls(&config, position) -> Cancellable<Option<Vec<CallItem>>>
```

`CallItem` contains:
- `target: NavigationTarget` — the caller/callee entity
- `ranges: Vec<FileRange>` — exact call site locations

**Limitations:** Text-range based (requires FilePosition). Does not resolve through generic monomorphization. Does not distinguish static/dynamic dispatch.

#### 3.1.2 ra_ap_hir Method Resolution (Fast Path — per-call-site)

```rust
// Resolve a specific method call expression to its target function
semantics.resolve_method_call(&method_call_expr) -> Option<Function>
semantics.resolve_method_call_as_callable(&method_call_expr) -> Option<Callable>

// For operator overloads:
semantics.resolve_bin_expr(&bin_expr) -> Option<Function>
semantics.resolve_prefix_expr(&prefix_expr) -> Option<Function>
semantics.resolve_index_expr(&index_expr) -> Option<Function>
semantics.resolve_try_expr(&try_expr) -> Option<Function> // ? operator

// For await:
semantics.resolve_await_to_poll(&await_expr) -> Option<Function>
```

#### 3.1.3 MIR TerminatorKind::Call (Deep Path — VERIFIED)

**Source:** [`compiler/rustc_middle/src/mir/syntax.rs`](https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/mir/syntax.rs)

```rust
TerminatorKind::Call {
    func: Operand<'tcx>,           // callee function
    args: Box<[Spanned<Operand<'tcx>>]>,
    destination: Place<'tcx>,      // return value target
    target: Option<BasicBlock>,    // successor on normal return
    unwind: UnwindAction,          // unwind path
    fn_span: Span,
}
```

To extract the callee `DefId`:
```rust
let func_ty = func.ty(&body.local_decls, tcx);
match func_ty.kind() {
    TyKind::FnDef(def_id, args) => { /* static dispatch — known target */ }
    TyKind::FnPtr(..) => { /* indirect call — target unknown at compile time */ }
    TyKind::Closure(def_id, args) => { /* closure call */ }
    _ => {}
}
```

#### 3.1.4 Instance Resolution for Monomorphization

```rust
// Source: rustc_middle::ty::Instance
// GitHub: https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/ty/instance.rs

// Resolve a generic function to its concrete monomorphized instance
Instance::try_resolve(tcx, typing_env, def_id, generic_args)
    -> Result<Option<Instance<'tcx>>, ErrorGuaranteed>

// Result classification:
match instance.def {
    InstanceKind::Item(def_id) => { /* Static dispatch: concrete function */ }
    InstanceKind::Virtual(def_id, vtable_idx) => { /* Dynamic dispatch: dyn Trait */ }
    InstanceKind::Intrinsic(def_id) => { /* Compiler intrinsic */ }
    InstanceKind::ClosureOnceShim { call_once, .. } => { /* Closure adapter */ }
    InstanceKind::DropGlue(def_id, ty) => { /* Implicit drop call */ }
    InstanceKind::CloneShim(def_id, ty) => { /* Clone impl */ }
    InstanceKind::VTableShim(def_id) => { /* Vtable dispatch shim */ }
    InstanceKind::ReifyShim(def_id, _) => { /* Function pointer reification */ }
    InstanceKind::FnPtrShim(def_id, ty) => { /* FnTrait for fn pointers */ }
    _ => {}
}
```

#### 3.1.5 Pre-computed Callee List (Shortcut)

```rust
// Fast callee enumeration without iterating MIR blocks:
let callees: &[(DefId, GenericArgsRef)] = tcx.mir_inliner_callees(InstanceKind::Item(def_id));
```

#### 3.1.6 Drop Calls (Implicit)

```rust
TerminatorKind::Drop { place, target, unwind, .. }
// Resolve to destructor:
let place_ty = place.ty(&body.local_decls, tcx).ty;
let instance = Instance::resolve_drop_in_place(tcx, place_ty);
```

#### 3.1.7 Tail Calls

```rust
TerminatorKind::TailCall { func, args, fn_span }
// Same func resolution as Call, but no return target
```

#### 3.1.8 How to Get VERIFIED Call Edges

**Recipe for maximum-fidelity call graph:**

1. Iterate `tcx.hir_body_owners()` — all functions with bodies
2. For each: `tcx.optimized_mir(def_id)` → get MIR body
3. For each `TerminatorKind::Call { func, .. }`:
   - `func.ty(&body.local_decls, tcx).kind()` → get `TyKind::FnDef(callee_id, args)`
   - `Instance::try_resolve(tcx, typing_env, callee_id, args)` → resolve through generics/traits
4. For each `TerminatorKind::Drop { place, .. }`:
   - `Instance::resolve_drop_in_place(tcx, place_ty)` → resolve destructor
5. Mark `InstanceKind::Virtual(..)` as dynamic-dispatch edges
6. Mark `TyKind::FnPtr(..)` as indirect-call edges

**Trust grade: Compiler-verified** for all statically resolvable calls.

---

### 3.2 Type Relationship Edges

#### 3.2.1 Type Usage (Parameters, Returns, Fields)

**Fast path (ra_ap_hir):**
```rust
// Function parameters
let params = fn.params_without_self(db); // Vec<Param>
for p in &params { p.ty() -> &Type }

// Return type
fn.ret_type(db) -> Type

// Struct fields
let fields = struct_.fields(db); // Vec<Field>
for f in &fields { f.ty(db) -> Type }

// Resolve type → entity
ty.as_adt() -> Option<Adt>  // Struct | Enum | Union
ty.type_arguments() -> impl Iterator<Item = Type>  // generic args
```

**Deep path (rustc):**
```rust
// Function signature
let sig = tcx.fn_sig(def_id).skip_binder().skip_binder();
sig.inputs()  // &[Ty<'tcx>]
sig.output()  // Ty<'tcx>

// Struct fields with type
let adt_def = tcx.adt_def(def_id);
for field in adt_def.all_fields() {
    field.ty(tcx, generic_args) -> Ty<'tcx>
}

// Match type → entity
match ty.kind() {
    TyKind::Adt(adt_def, args) => adt_def.did()  // DefId of struct/enum/union
    TyKind::FnDef(def_id, args) => /* function item type */
    TyKind::Ref(region, inner_ty, mutability) => /* reference type */
    TyKind::Param(param_ty) => param_ty.name  // generic parameter T
    _ => {}
}
```

#### 3.2.2 Trait Implementations

**Fast path:**
```rust
// All impls of a trait
Impl::all_for_trait(db, trait_) -> Vec<Impl>

// All impls for a type
Impl::all_for_type(db, ty) -> Vec<Impl>

// Edges: impl.self_ty(db) ——implements——> impl.trait_(db)
```

**Deep path:**
```rust
// All impls of a trait (including cross-crate)
let impls: &TraitImpls = tcx.trait_impls_of(trait_def_id);

// Local impls only
let local_impls = tcx.all_local_trait_impls(()); // HashMap<TraitDefId, Vec<LocalDefId>>

// For a specific impl: what trait + type?
let header = tcx.impl_trait_header(impl_def_id); // Option<ImplTraitHeader>
let self_ty = tcx.impl_self_ty(impl_def_id);     // Binders<Ty>
```

#### 3.2.3 Trait Bounds and Where Clauses

**Fast path:**
```rust
// Generic params with bounds
let params = GenericDef::from(fn_or_type).type_or_const_params(db);
// Trait bounds accessible via type inference context
```

**Deep path:**
```rust
let predicates = tcx.predicates_of(def_id);
for (predicate, span) in predicates.predicates {
    match predicate.kind().skip_binder() {
        ClauseKind::Trait(trait_pred) => {
            // T: Trait edge
            let trait_def_id = trait_pred.trait_ref.def_id;
            let self_ty = trait_pred.self_ty();
        }
        ClauseKind::TypeOutlives(outlives) => { /* T: 'a */ }
        ClauseKind::Projection(proj) => { /* <T as Trait>::Assoc = U */ }
        _ => {}
    }
}
```

#### 3.2.4 Generic Instantiation Edges

**Fast path:**
```rust
ty.type_arguments() -> impl Iterator<Item = Type>
// e.g., HashMap<String, Vec<i32>> → [String, Vec<i32>]
```

**Deep path:**
```rust
match ty.kind() {
    TyKind::Adt(adt_def, generic_args) => {
        for arg in generic_args.iter() {
            match arg.unpack() {
                GenericArgKind::Type(ty) => { /* type argument */ }
                GenericArgKind::Lifetime(region) => { /* lifetime argument */ }
                GenericArgKind::Const(c) => { /* const argument */ }
            }
        }
    }
    _ => {}
}
```

#### 3.2.5 Associated Types

**Fast path:**
```rust
let assoc_items = trait_.items(db); // Vec<AssocItem>
for item in assoc_items {
    match item {
        AssocItem::TypeAlias(ta) => { /* associated type */ }
        _ => {}
    }
}
```

**Deep path:**
```rust
let assoc_items = tcx.associated_items(trait_id);
for item in assoc_items.in_definition_order() {
    if item.kind == AssocKind::Type {
        // Associated type
    }
}

// Resolve projection: <T as Trait>::Assoc → concrete type
tcx.normalize_projection(projection_ty, trait_env) -> Ty
```

---

### 3.3 Module/Scope Edges

#### 3.3.1 Module Containment Hierarchy

**Fast path:**
```rust
// Module tree
module.children(db) -> impl Iterator<Item = Module>    // child modules
module.parent(db) -> Option<Module>                     // parent module
module.path_to_root(db) -> Vec<Module>                  // full chain to crate root
module.is_crate_root() -> bool

// Items in module
module.declarations(db) -> Vec<ModuleDef>               // all items
module.impl_defs(db) -> Vec<Impl>                       // all impl blocks
```

**Deep path:**
```rust
tcx.module_children(module_id)                          // all children of module
tcx.parent_module_from_def_id(def_id) -> LocalModDefId  // parent module
```

#### 3.3.2 Use/Import Relationships

**Fast path:**
```rust
// ItemScope gives all imports
let scope = module.scope(db, visible_from);
// scope: Vec<(Name, ScopeDef)> — includes imported names

// ItemScope (lower level via hir_def):
item_scope.imports() -> impl Iterator<Item = ImportId>
item_scope.entries() -> impl Iterator<Item = (&Name, PerNs)>
```

**Deep path:**
```rust
// HIR Use items
match item.kind {
    ItemKind::Use(use_path, use_kind) => {
        // use_path: the path being imported
        // use_kind: Star (glob), Named, or ListItem
    }
    _ => {}
}
```

#### 3.3.3 Re-exports and Visibility

**Fast path:**
```rust
// Visibility detection
fn.visibility(db) -> Visibility  // Public | PubCrate | Module(_) | Private
fn.is_visible_from(db, from_module) -> bool

// Find canonical import path
module.find_path(db, item, cfg) -> Option<ModPath>
module.find_use_path(db, item, prefix_kind, cfg) -> Option<ModPath>
```

**Deep path:**
```rust
let vis: Visibility<DefId> = tcx.visibility(def_id);
match vis {
    Visibility::Public => { /* pub */ }
    Visibility::Restricted(module_def_id) => {
        // pub(crate), pub(super), pub(in path)
        // module_def_id identifies the restricting module
    }
}
```

#### 3.3.4 Crate Dependency Edges

**Fast path:**
```rust
Crate::all(db) -> Vec<Crate>                                    // all workspace crates
krate.dependencies(db) -> Vec<CrateDependency>                   // direct deps
krate.reverse_dependencies(db) -> Vec<Crate>                     // reverse deps
krate.transitive_reverse_dependencies(db) -> impl Iterator<Item = Crate>
```

**Deep path:**
```rust
let all_crates: &[CrateNum] = tcx.crates(());
for krate in all_crates {
    let name = tcx.crate_name(*krate);
}
```

---

### 3.4 Data Flow Edges

#### 3.4.1 Def-Use Chains from MIR

**Source:** [`compiler/rustc_middle/src/mir/visit.rs`](https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/mir/visit.rs)

```rust
// Track definitions and uses of each local variable via MIR Visitor
impl<'tcx> Visitor<'tcx> for DefUseCollector {
    fn visit_local(&mut self, local: Local, context: PlaceContext, location: Location) {
        match context {
            PlaceContext::MutatingUse(MutatingUseContext::Store | MutatingUseContext::Call) => {
                // DEF: local is written here
            }
            PlaceContext::NonMutatingUse(NonMutatingUseContext::Copy | NonMutatingUseContext::Move) => {
                // USE: local is read here
            }
            _ => {}
        }
    }
}
```

#### 3.4.2 Place/Operand Tracking

**Source:** [`compiler/rustc_middle/src/mir/syntax.rs`](https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/mir/syntax.rs)

```rust
// Place = local variable + projection chain
Place { local: Local, projection: &[PlaceElem] }

// Key PlaceElem variants for data flow:
ProjectionElem::Field(idx, ty)    // struct field access
ProjectionElem::Deref             // dereference pointer/reference
ProjectionElem::Index(local)      // array/slice indexing
ProjectionElem::Downcast(name, variant_idx)  // enum variant

// Operand types indicating data movement:
Operand::Copy(place)    // reads without consuming
Operand::Move(place)    // reads and invalidates
Operand::Constant(..)   // constant value
```

#### 3.4.3 Assignment Flow Through Basic Blocks

```rust
// Assignments: StatementKind::Assign(Box<(Place, Rvalue)>)
// The LHS Place is written (DEF), the RHS Rvalue is read (USE)
//
// Key Rvalue variants for data flow:
Rvalue::Use(Operand)                        // simple copy/move
Rvalue::Ref(Region, BorrowKind, Place)      // creates reference to place
Rvalue::BinaryOp(BinOp, (Operand, Operand)) // combines two values
Rvalue::Aggregate(AggregateKind, operands)  // struct/enum construction
Rvalue::Cast(CastKind, Operand, Ty)         // type conversion
```

#### 3.4.4 Flowistry-Level Information Flow

**Source:** [Flowistry](https://github.com/browncel/flowistry) — `flowistry::infoflow`

```rust
// Compute full information-flow PDG for a function body
let flow = infoflow::compute_flow(tcx, body, def_id);
// flow: FlowResults — data dependency sets at each program point

// Alias analysis (ownership-based, borrow-checker-sound)
let aliases = Aliases::compute(tcx, body, def_id);
// aliases: maps places to their potentially aliased places via borrow checker data
```

**Downstream tools:**
- `rustc_utils::BodyExt::control_dependencies()` → Control Dependence Graph (CDG)
- `rustc_utils::PlaceExt::interior_pointers(tcx, body, def_id)` → maps places to borrow origins

---

### 3.5 Control Flow Edges

#### 3.5.1 BasicBlock Successor/Predecessor

**Source:** [`compiler/rustc_middle/src/mir/basic_blocks.rs`](https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/mir/basic_blocks.rs)

```rust
// Successors — from terminator
terminator.successors() -> impl Iterator<Item = BasicBlock>
terminator.edges() -> TerminatorEdges<'_, 'tcx>  // typed edge info

// Predecessors — cached in BasicBlocks
body.basic_blocks.predecessors() -> &IndexVec<BasicBlock, SmallVec<[BasicBlock; 4]>>

// Reverse postorder — canonical iteration order for forward analysis
body.basic_blocks.reverse_postorder() -> &[BasicBlock]
```

#### 3.5.2 Branch Conditions (SwitchInt)

```rust
TerminatorKind::SwitchInt { discr: Operand, targets: SwitchTargets }

// SwitchTargets methods:
targets.iter() -> impl Iterator<Item = (u128, BasicBlock)>  // (value, target) pairs
targets.otherwise() -> BasicBlock                            // fallthrough/default branch
targets.target_for_value(value) -> BasicBlock               // lookup specific branch
targets.all_targets() -> &[BasicBlock]                      // all targets
targets.as_static_if() -> Option<(u128, BasicBlock, BasicBlock)>  // if-like pattern
```

#### 3.5.3 Loop Detection

**Source:** [`compiler/rustc_middle/src/mir/loops.rs`](https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/mir/loops.rs)

```rust
// Heuristic loop header detection
maybe_loop_headers(body) -> DenseBitSet<BasicBlock>

// Precise loop detection via dominator back edges:
fn find_back_edges(body: &Body) -> Vec<(BasicBlock, BasicBlock)> {
    let doms = body.basic_blocks.dominators();
    let mut back_edges = vec![];
    for (bb, data) in body.basic_blocks.iter_enumerated() {
        for succ in data.terminator().successors() {
            if doms.dominates(succ, bb) {
                back_edges.push((bb, succ));  // (tail, header)
            }
        }
    }
    back_edges
}
```

#### 3.5.4 Dominator/Post-Dominator Relationships

**Source:** [`compiler/rustc_data_structures/src/graph/dominators/mod.rs`](https://github.com/rust-lang/rust/blob/main/compiler/rustc_data_structures/src/graph/dominators/mod.rs)

```rust
// Dominator tree (cached in BasicBlocks)
let doms: &Dominators<BasicBlock> = body.basic_blocks.dominators();

doms.is_reachable(node) -> bool
doms.immediate_dominator(node) -> Option<BasicBlock>
doms.dominates(a, b) -> bool  // O(1) using DFS timestamps

// Location-level dominance
location_a.dominates(location_b, &doms) -> bool

// Post-dominators: compute by reversing CFG and running dominators()
// (no built-in API — requires custom reversed graph)
```

#### 3.5.5 Error Paths (Panic, Result Propagation)

```rust
// Assertion checks (bounds, overflow, division by zero):
TerminatorKind::Assert { cond, expected, msg, target, unwind }
// msg: AssertMessage variants (BoundsCheck, Overflow, DivisionByZero, etc.)

// Explicit unwind paths:
// UnwindAction::Cleanup(bb) — jump to cleanup block on panic
// UnwindAction::Continue — continue unwinding
// UnwindAction::Unreachable — UB if reached
// UnwindAction::Terminate — abort on double panic

// Cleanup blocks:
bb_data.is_cleanup  // true for unwind/cleanup blocks
```

#### 3.5.6 CFG Traversal Utilities

```rust
// Source: compiler/rustc_middle/src/mir/traversal.rs
traversal::preorder(body) -> Preorder       // DFS discovery order
traversal::postorder(body) -> Postorder     // DFS completion order
traversal::reverse_postorder(body)          // canonical forward-analysis order
traversal::reachable_as_bitset(body) -> DenseBitSet<BasicBlock>  // O(1) membership
```

---

### 3.6 Ownership/Borrow Edges

#### 3.6.1 Polonius Facts

**Source:** [rust-lang/polonius](https://github.com/rust-lang/polonius) — `rustc_borrowck::polonius::legacy::facts`

**Input facts (compiler-generated):**

| Fact | Signature | Edge Type |
|------|-----------|-----------|
| `loan_issued_at` | `(Origin, Loan, Point)` | Borrow creation: origin O contains loan L from point P |
| `loan_killed_at` | `(Loan, Point)` | Borrow invalidation: prefix of borrowed path reassigned |
| `loan_invalidated_at` | `(Point, Loan)` | Illegal use of loan (borrow violation site) |
| `subset_base` | `(Origin1, Origin2, Point)` | Outlives: origin1 ⊆ origin2 at point P |
| `known_placeholder_subset` | `(Origin1, Origin2)` | Declared `'a: 'b` from signatures |
| `origin_live_at` | `(Origin, Point)` | Origin appears in type of live variable at P |
| `var_used_at` | `(Variable, Point)` | Variable read at point |
| `var_defined_at` | `(Variable, Point)` | Variable written at point |
| `var_dropped_at` | `(Variable, Point)` | Variable dropped at point |
| `child_path` | `(PathParent, PathChild)` | Ownership tree: parent → child (e.g. `x → x.field`) |
| `path_is_var` | `(Path, Variable)` | Root variable of a move path |
| `path_assigned_at_base` | `(Path, Point)` | Path initialization |
| `path_moved_at_base` | `(Path, Point)` | Ownership transfer |
| `path_accessed_at_base` | `(Path, Point)` | Path access |

**Derived (computed by Polonius engine):**

| Relation | Meaning |
|----------|---------|
| `subset(O1, O2, P)` | Transitive closure of `subset_base` at each CFG point |
| `origin_contains_loan_on_entry(O, L, P)` | Origin O contains loan L at point P |
| `loan_live_at(L, P)` | Loan L is live at P |
| `errors(L, P)` | Borrow violation detected |

**Access method:**
```bash
# Export to filesystem:
RUSTFLAGS="-Znll-facts" cargo build
# Per-function .facts files written to nll-facts/<fn-name>/

# In-process via rustc_driver::Callbacks::after_analysis
```

#### 3.6.2 Move Semantics Tracking

```rust
// From MIR:
Operand::Move(place)  // ownership transfer
Operand::Copy(place)  // copy (for Copy types)

// From Polonius:
path_moved_at_base(path, point)     // ownership transfer at point
path_assigned_at_base(path, point)  // re-initialization
child_path(parent, child)           // ownership tree structure
```

#### 3.6.3 Lifetime Relationships

```rust
// From Polonius:
subset_base(O1, O2, P)              // O1 outlives O2 at point P
known_placeholder_subset(O1, O2)    // declared 'a: 'b
use_of_var_derefs_origin(Var, O)    // using Var dereferences origin O
drop_of_var_derefs_origin(Var, O)   // dropping Var dereferences origin O
```

#### 3.6.4 Mutable vs Immutable Borrow Tracking

```rust
// From MIR Rvalue:
Rvalue::Ref(region, BorrowKind::Shared, place)     // &place (immutable borrow)
Rvalue::Ref(region, BorrowKind::Mut { .. }, place)  // &mut place (mutable borrow)
Rvalue::Ref(region, BorrowKind::Fake(_), place)     // two-phase borrow placeholder

// From PlaceContext in MIR visitor:
NonMutatingUseContext::SharedBorrow   // immutable borrow
MutatingUseContext::Borrow            // mutable borrow
```

---

### 3.7 Trait Dispatch Edges

#### 3.7.1 Static Dispatch Resolution

```rust
// Via Instance resolution:
let instance = Instance::try_resolve(tcx, typing_env, def_id, args)?;
match instance.def {
    InstanceKind::Item(concrete_def_id) => {
        // Fully resolved static dispatch — concrete function known
    }
    _ => {}
}

// Via ra_ap_hir_ty:
db.lookup_impl_method(env, func_id, fn_subst) -> (FunctionId, Substitution)
```

#### 3.7.2 Dynamic Dispatch (dyn Trait)

```rust
// Vtable entries for a trait:
tcx.vtable_entries(trait_ref) -> &[VtblEntry]
// VtblEntry variants: MetadataSize, MetadataAlign, Method(DefId, SubstsRef), Vacant, TraitVPtr(...)

// Own entries (not including supertraits):
tcx.own_existential_vtable_entries(trait_def_id) -> &[DefId]

// Instance resolution returns Virtual for dyn dispatch:
InstanceKind::Virtual(def_id, vtable_idx)
// def_id: the trait method
// vtable_idx: index into vtable
```

#### 3.7.3 Specialization Relationships

```rust
// Does impl1 specialize impl2?
tcx.specializes((impl_id1, impl_id2)) -> bool

// Full specialization hierarchy:
tcx.specialization_graph_of(trait_def_id) -> Result<&Graph, ErrorGuaranteed>

// Parent impl (in specialization chain):
tcx.impl_parent(impl_def_id) -> Option<DefId>
```

#### 3.7.4 Blanket Impl Resolution

```rust
// ra_ap_hir_ty:
let trait_impls = TraitImpls::for_crate_and_deps(db, krate);
trait_impls.blanket_impls(trait_id) -> &[ImplId]

// rustc:
let impls = tcx.trait_impls_of(trait_def_id);
// Filter for blanket impls: those where self_ty is a type parameter
```

---

## PART 4: METADATA EXTRACTION ENDPOINTS

### 4.1 Signatures & Types

#### Function Signatures

**Fast path:**
```rust
fn.ret_type(db) -> Type
fn.params_without_self(db) -> Vec<Param>
fn.self_param(db) -> Option<SelfParam>
fn.async_ret_type(db) -> Option<Type>  // inner type for async fn
fn.display(db, target) -> HirDisplayWrapper  // "fn foo(x: i32) -> bool"
```

**Deep path:**
```rust
let sig = tcx.fn_sig(def_id).skip_binder().skip_binder();
sig.inputs()   // &[Ty<'tcx>] — parameter types
sig.output()   // Ty<'tcx> — return type
tcx.fn_arg_idents(def_id) // &[Option<Ident>] — parameter names
```

#### Generic Parameters and Bounds

**Fast path:**
```rust
let params = GenericDef::from(item).type_or_const_params(db);
let lifetime_params = GenericDef::from(item).lifetime_params(db);
```

**Deep path:**
```rust
let generics = tcx.generics_of(def_id);
generics.own_params           // Vec<GenericParamDef>
generics.has_self             // true for trait methods

let predicates = tcx.predicates_of(def_id);  // where clauses
```

#### Lifetime Annotations

**Fast path:** `GenericDef::from(item).lifetime_params(db) -> Vec<LifetimeParam>`

**Deep path:**
```rust
let generics = tcx.generics_of(def_id);
for param in &generics.own_params {
    if let GenericParamDefKind::Lifetime = param.kind { /* lifetime param */ }
}
```

### 4.2 Visibility & Scope

**Fast path (`ra_ap_hir`):**
```rust
// Visibility enum:
Visibility::Public          // pub
Visibility::PubCrate        // pub(crate)
Visibility::Module(Module)  // pub(super) / pub(in path)
Visibility::Private         // no qualifier

item.visibility(db) -> Visibility          // via HasVisibility trait
item.is_visible_from(db, module) -> bool   // check from specific module

// Enclosing module:
fn.module(db) -> Module
module.path_to_root(db) -> Vec<Module>     // full module path
```

**Deep path (rustc):**
```rust
let vis: Visibility<DefId> = tcx.visibility(def_id);
vis.is_public()
vis.is_accessible_from(module_def_id, tcx)

// Field-level visibility:
let field_vis = tcx.field_visibilities(variant_id);  // ArenaMap<FieldId, Visibility>
```

### 4.3 Complexity Metrics (Derivable from Compiler Data)

| Metric | How to Derive | Source |
|--------|--------------|--------|
| **Fan-in** (incoming calls) | Count `incoming_calls` results or reverse-index MIR call graph | `ra_ap_ide::incoming_calls` or MIR `TerminatorKind::Call` |
| **Fan-out** (outgoing calls) | Count `outgoing_calls` results or count `Call` terminators in MIR | `ra_ap_ide::outgoing_calls` or MIR traversal |
| **Cyclomatic complexity** | `basic_blocks.len() - edges + 2` or count `SwitchInt` terminators + 1 | `body.basic_blocks` from MIR |
| **Type complexity** | Count generic params + trait bounds + nested type depth | `tcx.generics_of` + `tcx.predicates_of` |
| **Nesting depth** | Count nested `SourceScope` parents | `body.source_scopes` parent chain |

### 4.4 Source Location

**Fast path:**
```rust
// HasSource trait: item.source(db) -> InFile<T::Ast>
let in_file = fn.source(db);
let file_id: HirFileId = in_file.file_id;
let ast_node = in_file.value;
let text_range: TextRange = ast_node.syntax().text_range();

// File path:
vfs.file_path(file_id) -> &VfsPath
```

**Deep path:**
```rust
let span = tcx.def_span(def_id);
let sm = tcx.sess.source_map();

// Full location info:
let (file, line_lo, col_lo, line_hi, col_hi) = sm.span_to_location_info(span);

// Source text:
let snippet: Result<String> = sm.span_to_snippet(span);

// Filename:
let filename = sm.span_to_filename(span);
```

### 4.5 Attributes & Annotations

**Fast path (`ra_ap_hir`):**
```rust
let attrs: AttrsWithOwner = item.attrs(db);  // via HasAttrs trait

// Doc comments:
attrs.doc_exprs() -> Iterator<Item = DocExpr>
attrs.by_key(sym::doc)

// Cfg conditions:
attrs.cfg() -> Option<CfgExpr>

// Specific checks:
fn.is_test(db)       // #[test]
fn.is_bench(db)      // #[bench]
fn.is_async(db)      // async fn
fn.is_const(db)      // const fn
fn.is_unsafe_to_call(db, ..)  // unsafe fn
```

**Deep path (rustc):**
```rust
let attrs = tcx.attrs_for_def(def_id);
for attr in attrs {
    attr.has_name(sym::test)     // #[test]
    attr.has_name(sym::cfg)      // #[cfg(...)]
    attr.has_name(sym::inline)   // #[inline]
    attr.has_name(sym::derive)   // #[derive(...)]
    attr.has_name(sym::doc)      // doc comments
}

// Codegen attributes:
let codegen_attrs = tcx.codegen_fn_attrs(def_id);
// inline, no_mangle, link_section, etc.

// Unsafe detection:
// HIR: FnHeader::safety
// DefKind::Static { safety, mutability, nested }

// Extern ABI:
// HIR: FnHeader::abi (Abi::Rust, Abi::C, etc.)
```

---

## PART 5: GRAPH ALGORITHM FUEL

Map each algorithm to the EXACT APIs that provide its data.

| Algorithm | Data Needed | Primary API (Fast Path) | Deep API (Nightly) |
|-----------|-------------|------------------------|-------------------|
| **RRF Search** | Symbol names, paths, kinds | `analysis.symbol_search(query, limit)` → `Vec<NavigationTarget>` | `tcx.hir_body_owners()` + `tcx.def_path_str(def_id)` + `tcx.def_kind(def_id)` |
| **BFS Anchor** | Visibility, containment | `fn.visibility(db)` via `HasVisibility` + `module.declarations(db)` | `tcx.visibility(def_id)` + `hir::ItemKind` visibility |
| **Ego Network** | Call graph 1-2 hops | `analysis.incoming_calls(cfg, pos)` + `analysis.outgoing_calls(cfg, pos)` | MIR `TerminatorKind::Call` + `Instance::try_resolve()` |
| **PageRank** | Weighted call graph | Derived from `CallItem.ranges.len()` (call-site count as weight) | Derived from MIR call count per edge |
| **Leiden Communities** | Call + type + module edges | Derived from call hierarchy + `Type.as_adt()` + `module.children()` | Derived from MIR calls + `TyKind::Adt` + module tree |
| **k-Core Decomposition** | Call graph degrees | Derived from fan-in + fan-out counts | Derived from MIR call graph |
| **SCC (Tarjan)** | Call + impl edges | Derived from call + `Impl::all_for_trait()` | `tcx.mir_callgraph_cyclic(def_id)` + `tcx.trait_impls_of()` |
| **Blast Radius (BFS)** | Transitive callers/callees | Repeated `incoming_calls` / `outgoing_calls` | MIR transitive closure over `TerminatorKind::Call` |
| **Complexity Hotspots** | Fan-in, fan-out, type complexity | Derived from call hierarchy + `GenericDef::params()` | MIR `basic_blocks.len()` + `tcx.generics_of()` + `tcx.predicates_of()` |
| **Coupling/Cohesion (CK)** | Type usage, field access | `Type.as_adt()` + `Struct.fields()` + `Field.ty()` | MIR `Place::projection` `Field` elements + `TyKind::Adt` |
| **Entropy** | Branch distribution, type variety | Derived from `analysis.file_structure()` | MIR `SwitchInt` target counts + `local_decls` type diversity |
| **SQALE Debt** | Complexity + coverage + coupling | Combined metrics from above | Combined metrics from MIR + `tcx` |

---

## PART 6: PARSELTONGUE API MAPPING

Map each Parseltongue endpoint to the compiler APIs that power it.

### GET /code-entities-list-all

**Purpose:** Enumerate all code entities in the workspace.

| Source | API |
|--------|-----|
| Fast path | `Crate::all(db)` → `krate.modules(db)` → `module.declarations(db)` → match on `ModuleDef` variants |
| Deep path | `tcx.hir_body_owners()` + `tcx.hir_free_items()` + `tcx.def_kind(def_id)` |
| Stable MIR | `all_local_items()` |

### GET /code-entities-search-fuzzy

**Purpose:** Fuzzy search for entities by name.

| Source | API |
|--------|-----|
| Fast path | `analysis.symbol_search(Query::new(text), limit)` → `Vec<NavigationTarget>` |
| Deep path | `tcx.hir_body_owners()` + `tcx.def_path_str(def_id).contains(query)` |
| External | `Crate::query_external_importables(db, query)` for cross-crate search |

### GET /code-entity-detail-view

**Purpose:** Full metadata for a single entity.

| Source | API |
|--------|-----|
| Fast path | Entity-specific: `fn.name(db)`, `fn.ret_type(db)`, `fn.visibility(db)`, `fn.attrs(db)`, `fn.module(db)`, `fn.source(db)`, `fn.is_async(db)`, etc. |
| Deep path | `tcx.def_path_str(def_id)` + `tcx.fn_sig(def_id)` + `tcx.visibility(def_id)` + `tcx.attrs_for_def(def_id)` + `tcx.def_span(def_id)` + `sm.span_to_snippet(span)` |
| Type info | `tcx.generics_of(def_id)` + `tcx.predicates_of(def_id)` |

### GET /dependency-edges-list-all

**Purpose:** All edges (call, type, module, impl) between entities.

| Source | API |
|--------|-----|
| Call edges (fast) | `analysis.outgoing_calls(cfg, pos)` for each function |
| Call edges (deep) | MIR `TerminatorKind::Call` traversal of `tcx.optimized_mir(def_id)` |
| Type edges | `fn.ret_type(db)` / `fn.params_without_self(db)` → `ty.as_adt()` |
| Impl edges | `Impl::all_for_trait(db, t)` / `tcx.trait_impls_of(trait_id)` |
| Module edges | `module.children(db)` / `module.parent(db)` |
| Crate edges | `krate.dependencies(db)` |

### GET /reverse-callers-query-graph

**Purpose:** All callers of a given function (who calls me?).

| Source | API |
|--------|-----|
| Fast path | `analysis.incoming_calls(&config, position)` → `Vec<CallItem>` |
| Deep path | Reverse-index MIR call graph (iterate all body owners, find calls to target `DefId`) |
| Shortcut | `analysis.find_all_refs(position, config)` → `ReferenceSearchResult` (includes call sites) |

### GET /forward-callees-query-graph

**Purpose:** All functions called by a given function (what do I call?).

| Source | API |
|--------|-----|
| Fast path | `analysis.outgoing_calls(&config, position)` → `Vec<CallItem>` |
| Deep path | `tcx.optimized_mir(def_id)` → iterate `TerminatorKind::Call` |
| Shortcut | `tcx.mir_inliner_callees(InstanceKind::Item(def_id))` → `&[(DefId, GenericArgsRef)]` |

### GET /blast-radius-impact-analysis

**Purpose:** Transitive impact of changes to a function.

| Source | API |
|--------|-----|
| Fast path | Recursive BFS over `analysis.incoming_calls()` (callers of callers) |
| Deep path | Transitive closure over MIR call graph + `tcx.trait_impls_of()` for trait dispatch paths |
| Reverse deps | `krate.transitive_reverse_dependencies(db)` for crate-level impact |

### GET /circular-dependency-detection-scan

**Purpose:** Detect circular call chains and mutual dependencies.

| Source | API |
|--------|-----|
| Fast path | Tarjan SCC on call graph built from `outgoing_calls` |
| Deep path | `tcx.mir_callgraph_cyclic(def_id)` → `&UnordSet<LocalDefId>` (functions in call cycle) |
| Module level | Build directed graph from `module.children()` + `use` imports, detect SCCs |

### GET /complexity-hotspots-ranking-view

**Purpose:** Rank entities by composite complexity score.

| Source | API |
|--------|-----|
| Fan-in/out | Count results from `incoming_calls` / `outgoing_calls` |
| Cyclomatic | MIR `body.basic_blocks.len()` - edge_count + 2; or count `SwitchInt` + 1 |
| Type complexity | `tcx.generics_of(def_id).own_params.len()` + `tcx.predicates_of(def_id).predicates.len()` |
| Nesting | Source scope depth from `body.source_scopes` |

### GET /semantic-cluster-grouping-list

**Purpose:** Group related entities into semantic clusters.

| Source | API |
|--------|-----|
| Module clustering | `module.declarations(db)` + `module.impl_defs(db)` |
| Type affinity | `Impl::all_for_type(db, ty)` — all impls on a type form a cluster |
| Trait families | `trait.all_supertraits(db)` + `Impl::all_for_trait(db, t)` |
| Call clusters | Ego-network from `outgoing_calls` / `incoming_calls` |

### GET /strongly-connected-components-analysis

**Purpose:** Find SCCs in the call/dependency graph.

| Source | API |
|--------|-----|
| Fast path | Tarjan on call graph from `outgoing_calls` |
| Deep path | MIR call graph + `tcx.mir_callgraph_cyclic(def_id)` |
| Trait cycles | `tcx.trait_impls_of()` → detect mutual impl chains |

### GET /technical-debt-sqale-scoring

**Purpose:** SQALE-based technical debt score.

| Source | Inputs |
|--------|--------|
| Complexity | Cyclomatic from MIR + fan-in/out from call graph |
| Coupling | Type usage cross-module: `fn.ret_type(db).as_adt()` → check if ADT is in different module |
| Coverage | Test functions: `fn.is_test(db)` ratio |
| Duplication | Heuristic from source snippets: `sm.span_to_snippet(span)` |
| Documentation | `attrs.doc_exprs()` presence/absence |

### GET /kcore-decomposition-layering-analysis

**Purpose:** k-core layers of the call graph.

| Source | API |
|--------|-----|
| Degree computation | Fan-in + fan-out from call graph edges |
| Graph source | `outgoing_calls` (fast) or MIR `TerminatorKind::Call` (deep) |

### GET /centrality-measures-entity-ranking

**Purpose:** PageRank, betweenness, closeness centrality.

| Source | API |
|--------|-----|
| Call graph | `outgoing_calls` / `incoming_calls` for adjacency |
| Weighted edges | `CallItem.ranges.len()` as edge weight (call frequency) |
| Deep graph | Full MIR call graph with `Instance::try_resolve` for monomorphization |

### GET /entropy-complexity-measurement-scores

**Purpose:** Information entropy of code structure.

| Source | API |
|--------|-----|
| Branch entropy | MIR `SwitchTargets.all_targets().len()` distribution per function |
| Type entropy | Diversity of `body.local_decls[local].ty` across locals |
| Control entropy | Distribution of terminator kinds across basic blocks |

### GET /coupling-cohesion-metrics-suite

**Purpose:** CK-style coupling and cohesion metrics.

| Source | API |
|--------|-----|
| Efferent coupling (Ce) | Outgoing type references: `fn.ret_type(db).as_adt()` in external modules |
| Afferent coupling (Ca) | Incoming references: `analysis.find_all_refs(pos, config)` from other modules |
| LCOM | Method-field access matrix from `Impl::all_for_type(db, ty)` → `impl.items(db)` |
| Field access tracking | MIR `Place::projection` with `Field(..)` elements |

### GET /leiden-community-detection-clusters

**Purpose:** Community detection on the code graph.

| Source | API |
|--------|-----|
| Multi-layer graph | Call edges + type edges + module containment + impl edges |
| Call edges | `outgoing_calls` / MIR calls |
| Type edges | `fn.params_without_self(db)` / `fn.ret_type(db)` → `as_adt()` |
| Module edges | `module.children(db)` / `module.parent(db)` |
| Impl edges | `Impl::all_for_trait(db, t)` + `impl.self_ty(db).as_adt()` |

---

## PART 7: INTEGRATION ARCHITECTURE RECOMMENDATION

### 7.1 Two-Layer Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    PARSELTONGUE SERVER                       │
│                                                             │
│  ┌──────────────────────┐    ┌────────────────────────────┐ │
│  │  LAYER 1: FAST PATH  │    │  LAYER 2: DEEP MODE        │ │
│  │  (ra_ap_* crates)    │    │  (rustc_plugin + nightly)   │ │
│  │                      │    │                             │ │
│  │  • Entity discovery  │    │  • MIR call graph           │ │
│  │  • Call hierarchy    │    │  • Data flow analysis       │ │
│  │  • Symbol search     │    │  • Control flow graph       │ │
│  │  • Type resolution   │    │  • Polonius borrow facts    │ │
│  │  • Module tree       │    │  • Instance resolution      │ │
│  │  • Visibility        │    │  • Monomorphization         │ │
│  │  • Attributes        │    │  • Dataflow framework       │ │
│  │                      │    │  • Dominator tree           │ │
│  │  Phases: 1,2,3,4     │    │                             │ │
│  │  Latency: <100ms     │    │  Phase: 5 (DEEP DIVE)      │ │
│  │  Toolchain: stable   │    │  Latency: 1-30s             │ │
│  └──────────────────────┘    │  Toolchain: nightly          │ │
│                              └────────────────────────────┘ │
│                                                             │
│  ┌──────────────────────────────────────────────────────────┐│
│  │  ALTERNATIVE DEEP PATH: Charon/LLBC                     ││
│  │  • JSON extraction of full crate (offline)              ││
│  │  • Structured AST with resolved traits                  ││
│  │  • No in-process nightly dependency                     ││
│  │  • Use when deep mode is unavailable                    ││
│  └──────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### 7.2 When to Use Each Layer

| Scenario | Use Layer 1 | Use Layer 2 | Use Charon |
|----------|-------------|-------------|------------|
| Entity listing, search, outline | ✓ | | |
| Call hierarchy (1-2 hops) | ✓ | | |
| Visibility/scope analysis | ✓ | | |
| Type relationships (ADT → field types) | ✓ | | |
| Trait impl discovery | ✓ | ✓ (cross-crate) | |
| Full transitive call graph | | ✓ | ✓ |
| Monomorphized dispatch resolution | | ✓ | ✓ |
| Data flow / def-use chains | | ✓ | |
| Borrow/lifetime analysis | | ✓ (Polonius) | |
| Control flow graph with dominators | | ✓ | |
| Information-flow PDG | | ✓ (Flowistry) | |
| Cyclomatic complexity from MIR | | ✓ | ✓ |
| Offline batch analysis | | | ✓ |

### 7.3 How Charon/LLBC Serves as Alternative Deep Path

**When to use Charon instead of direct rustc:**
1. When nightly toolchain is not available or not desired in CI
2. When cross-language analysis is needed (Charon has OCaml bindings via `charon-ml`)
3. When a stable JSON format is preferred over in-process Rust APIs
4. When trait dispatch must be fully resolved upfront (Charon resolves all dispatch at extraction time)

**Charon limitations vs. direct rustc:**
- No Polonius borrow-check facts (structural lifetime info only)
- Incomplete support for closures, unsafe code, dynamic dispatch
- Alpha stability — API changes expected
- Extraction time: ~80s for 20KLOC crate (vs. incremental rustc)

**Usage:**
```bash
# Extract
charon              # produces target_crate.llbc

# Consume
let data: CrateData = serde_json::from_reader(File::open("target_crate.llbc")?)?;
for fun in &data.fun_decls { /* walk LLBC statements for call sites */ }
```

### 7.4 Version Coupling Strategy

**Layer 1 (`ra_ap_*`):**
- Published on crates.io with semver (`0.0.322+`)
- Track rust-analyzer releases (typically weekly)
- Pin to specific version in `Cargo.toml`; update quarterly
- Breaking changes are rare within a major rust-analyzer release cycle

**Layer 2 (`rustc_private`):**
- Pin to specific nightly date in `rust-toolchain.toml`
- Support strategy: N (current nightly) and N-1 (previous month's nightly)
- Use `rustc_plugin` (version `0.13.0-nightly-2025-03-03`) for Cargo integration
- Require `components = ["rustc-dev", "llvm-tools-preview"]`

**Graceful degradation strategy:**
```
IF nightly toolchain available AND rustc-dev component present:
    Use Layer 2 for full analysis
ELSE IF Charon LLBC file exists AND is fresh:
    Use Charon output for deep analysis
ELSE:
    Use Layer 1 only
    Mark Phase 5 results as trust_grade: "heuristic"
```

### 7.5 Graceful Degradation When Deep Mode Unavailable

| Endpoint | Full Mode (L1+L2) | Degraded Mode (L1 only) |
|----------|-------------------|------------------------|
| `/code-entities-list-all` | Full with MIR availability flags | Full (no MIR flags) |
| `/code-entities-search-fuzzy` | Full | Full |
| `/code-entity-detail-view` | Full with MIR metrics | Partial — no MIR metrics |
| `/forward-callees-query-graph` | MIR-verified call graph | ra_ap_ide call hierarchy (text-based) |
| `/reverse-callers-query-graph` | MIR-verified reverse graph | ra_ap_ide incoming_calls |
| `/blast-radius-impact-analysis` | Transitive MIR closure | BFS on ra_ap_ide calls (less precise) |
| `/complexity-hotspots-ranking-view` | MIR cyclomatic + type complexity | Fan-in/out only |
| `/entropy-complexity-measurement-scores` | MIR branch distribution | Unavailable (returns empty) |
| `/coupling-cohesion-metrics-suite` | MIR field access tracking | Type-level coupling only |

---

## PART 8: QUICK REFERENCE TABLES

### 8.1 All Key Types — One-Line Descriptions

#### ra_ap_* Types

| Type | Crate | Description |
|------|-------|-------------|
| `Semantics<'db, DB>` | `ra_ap_hir` | Bridge from syntax nodes to semantic (HIR) information |
| `Crate` | `ra_ap_hir` | A Rust crate in the workspace |
| `Module` | `ra_ap_hir` | A module within a crate |
| `ModuleDef` | `ra_ap_hir` | Enum of all items that can appear in a module scope |
| `Function` | `ra_ap_hir` | A function definition (wraps `FunctionId`) |
| `Struct` | `ra_ap_hir` | A struct definition |
| `Enum` | `ra_ap_hir` | An enum definition |
| `Variant` | `ra_ap_hir` | An enum variant |
| `Trait` | `ra_ap_hir` | A trait definition |
| `Impl` | `ra_ap_hir` | An impl block (inherent or trait) |
| `Type<'db>` | `ra_ap_hir` | A resolved HIR type with full generic substitution |
| `Field` | `ra_ap_hir` | A struct/enum field |
| `Callable<'db>` | `ra_ap_hir` | A callable entity (function, closure, fn pointer) |
| `Visibility` | `ra_ap_hir` | Visibility level (Public, PubCrate, Module, Private) |
| `AttrsWithOwner` | `ra_ap_hir` | Attributes on a HIR item including doc comments |
| `SemanticsScope<'db>` | `ra_ap_hir` | Set of visible names at a particular program point |
| `AnalysisHost` | `ra_ap_ide` | Mutable world state; apply changes, get `Analysis` snapshots |
| `Analysis` | `ra_ap_ide` | Immutable query snapshot for IDE features |
| `NavigationTarget` | `ra_ap_ide` | A navigable code element (file, range, name, kind) |
| `CallItem` | `ra_ap_ide` | One node in a call hierarchy (target + call site ranges) |
| `ReferenceSearchResult` | `ra_ap_ide` | Find-all-references result (declaration + reference locations) |
| `StructureNode` | `ra_ap_ide` | File outline item (label, range, kind, detail) |
| `InferenceResult` | `ra_ap_hir_ty` | Map from expressions/patterns to inferred types |
| `TraitImpls` | `ra_ap_hir_ty` | All implementations of a trait (blanket, specific, builtin) |
| `InherentImpls` | `ra_ap_hir_ty` | Inherent (non-trait) impls for types in a crate |
| `DefMap` | `ra_ap_hir_def` | Module-level name resolution results for a crate |
| `ItemScope` | `ra_ap_hir_def` | Per-module name registry (types, values, macros, imports) |
| `Resolver` | `ra_ap_hir_def` | Name resolution facade (resolve paths in type/value namespace) |
| `ItemTree` | `ra_ap_hir_def` | Macro-expanded item list for a file |
| `FileId` | `ra_ap_base_db` | Opaque handle to a file in VFS |
| `CrateGraph` | `ra_ap_base_db` | Maps file sets → crate IDs with dependencies |
| `Vfs` | `ra_ap_vfs` | Virtual file system mapping paths to file IDs |
| `SyntaxNode` | `ra_ap_syntax` | A node in the lossless concrete syntax tree |

#### rustc Types

| Type | Crate | Description |
|------|-------|-------------|
| `TyCtxt<'tcx>` | `rustc_middle` | Central query engine wrapping all compiler state |
| `DefId` | `rustc_span` | Globally unique definition identifier (crate + index) |
| `LocalDefId` | `rustc_span` | A `DefId` known to be in the current crate |
| `HirId` | `rustc_hir` | Unique identifier for a node in the current crate's HIR |
| `DefKind` | `rustc_hir` | Kind of definition (Fn, Struct, Trait, Impl, etc.) |
| `Ty<'tcx>` | `rustc_middle` | An interned type (access kind via `.kind()`) |
| `TyKind<'tcx>` | `rustc_middle` | Full type variant (Adt, Ref, FnDef, Tuple, etc.) |
| `AdtDef<'tcx>` | `rustc_middle` | Struct/enum/union definition with variants and fields |
| `Instance<'tcx>` | `rustc_middle` | A monomorphized function instance (def + concrete generic args) |
| `InstanceKind<'tcx>` | `rustc_middle` | Kind of instance (Item, Virtual, DropGlue, etc.) |
| `Body<'tcx>` (MIR) | `rustc_middle` | MIR function body (basic blocks, locals, source scopes) |
| `BasicBlock` | `rustc_middle` | Index into the CFG (newtype around u32) |
| `BasicBlockData<'tcx>` | `rustc_middle` | Statements + terminator for one basic block |
| `Statement<'tcx>` | `rustc_middle` | A non-terminating MIR instruction |
| `Terminator<'tcx>` | `rustc_middle` | Block-ending instruction (Call, Goto, SwitchInt, Return, etc.) |
| `Place<'tcx>` | `rustc_middle` | A memory location (local + projection chain) |
| `Operand<'tcx>` | `rustc_middle` | A value (Copy, Move, or Constant) |
| `Rvalue<'tcx>` | `rustc_middle` | Right-hand side of an assignment (Use, Ref, BinaryOp, Aggregate, etc.) |
| `Location` | `rustc_middle` | A point in MIR (block + statement index) |
| `Span` | `rustc_span` | Source code range (compressed byte positions + syntax context) |
| `SourceMap` | `rustc_span` | Maps `Span` → file/line/column |
| `Dominators<N>` | `rustc_data_structures` | Dominator tree with O(1) dominance queries |
| `TypingEnv<'tcx>` | `rustc_middle` | Environment for trait resolution and instance resolution |
| `GenericArgsRef<'tcx>` | `rustc_middle` | Concrete generic arguments (type, lifetime, const) |
| `TraitRef<'tcx>` | `rustc_middle` | `Type: Trait<Args>` — a trait reference |
| `AssocItem` | `rustc_middle` | An associated item (fn, type, or const) in a trait/impl |
| `TypeckResults<'tcx>` | `rustc_middle` | Type-check results for a function body (expression types) |
| `MoveData<'tcx>` | `rustc_mir_dataflow` | Move path tracking for initialization/liveness analysis |
| `ResultsCursor` | `rustc_mir_dataflow` | Random-access cursor into dataflow analysis results |

#### Polonius/Flowistry/Charon Types

| Type | Source | Description |
|------|--------|-------------|
| `PoloniusFacts` | `rustc_borrowck` | All input facts for the Polonius borrow checker |
| `FlowResults` | Flowistry | Information-flow dependency sets at each program point |
| `Aliases` | Flowistry | Alias analysis mapping places to potentially aliased places |
| `ControlDependencies` | `rustc_utils` | Control dependence graph (which BBs control which) |
| `CrateData` | `charon_lib` | Complete crate extraction (functions, types, traits) in JSON |
| `FunDecl` | `charon_lib` | A function declaration with optional LLBC body |
| `TypeDecl` | `charon_lib` | A type declaration (struct/enum/union/alias) |

### 8.2 All Key Queries — Return Types

#### ra_ap_ide Analysis Queries

| Query | Return Type |
|-------|-------------|
| `analysis.symbol_search(query, limit)` | `Cancellable<Vec<NavigationTarget>>` |
| `analysis.goto_definition(pos, cfg)` | `Cancellable<Option<RangeInfo<Vec<NavigationTarget>>>>` |
| `analysis.goto_implementation(cfg, pos)` | `Cancellable<Option<RangeInfo<Vec<NavigationTarget>>>>` |
| `analysis.find_all_refs(pos, cfg)` | `Cancellable<Option<Vec<ReferenceSearchResult>>>` |
| `analysis.incoming_calls(cfg, pos)` | `Cancellable<Option<Vec<CallItem>>>` |
| `analysis.outgoing_calls(cfg, pos)` | `Cancellable<Option<Vec<CallItem>>>` |
| `analysis.file_structure(cfg, file_id)` | `Cancellable<Vec<StructureNode>>` |
| `analysis.hover(cfg, range)` | `Cancellable<Option<RangeInfo<HoverResult>>>` |
| `analysis.parent_module(pos)` | `Cancellable<Vec<NavigationTarget>>` |
| `analysis.crates_for(file_id)` | `Cancellable<Vec<Crate>>` |

#### rustc TyCtxt Queries

| Query | Return Type |
|-------|-------------|
| `tcx.def_kind(def_id)` | `DefKind` |
| `tcx.def_span(def_id)` | `Span` |
| `tcx.def_path_str(def_id)` | `String` |
| `tcx.visibility(def_id)` | `Visibility<DefId>` |
| `tcx.type_of(def_id)` | `EarlyBinder<Ty<'tcx>>` |
| `tcx.fn_sig(def_id)` | `EarlyBinder<PolyFnSig<'tcx>>` |
| `tcx.adt_def(def_id)` | `AdtDef<'tcx>` |
| `tcx.generics_of(def_id)` | `&Generics` |
| `tcx.predicates_of(def_id)` | `GenericPredicates<'tcx>` |
| `tcx.trait_impls_of(def_id)` | `&TraitImpls` |
| `tcx.all_local_trait_impls(())` | `&FxIndexMap<DefId, Vec<LocalDefId>>` |
| `tcx.associated_items(def_id)` | `&AssocItems` |
| `tcx.impl_trait_header(def_id)` | `Option<ImplTraitHeader<'tcx>>` |
| `tcx.attrs_for_def(def_id)` | `&[Attribute]` |
| `tcx.optimized_mir(def_id)` | `&Body<'tcx>` |
| `tcx.mir_inliner_callees(instance)` | `&[(DefId, GenericArgsRef)]` |
| `tcx.mir_callgraph_cyclic(def_id)` | `&UnordSet<LocalDefId>` |
| `tcx.is_mir_available(def_id)` | `bool` |
| `tcx.vtable_entries(trait_ref)` | `&[VtblEntry]` |
| `tcx.typeck(local_def_id)` | `&TypeckResults<'tcx>` |
| `tcx.hir_body_owners()` | `impl Iterator<Item=LocalDefId>` |
| `tcx.hir_free_items()` | `impl Iterator<Item=ItemId>` |
| `tcx.module_children(def_id)` | `&[ModChild]` |
| `tcx.crates(())` | `&[CrateNum]` |

### 8.3 Import Paths for Everything

#### ra_ap_* Import Paths

```rust
// Core semantic model
use ra_ap_hir::{
    Semantics, Crate, Module, ModuleDef, Function, Struct, Enum, Variant,
    Trait, Impl, Type, Field, Callable, CallableKind, Param, SelfParam,
    Adt, Const, Static, TypeAlias, Macro, AssocItem, GenericDef,
    GenericParam, TypeParam, LifetimeParam, TypeOrConstParam,
    SemanticsScope, Visibility, HasVisibility, HasAttrs, HasSource,
    HirDisplay, HasCrate, AttrsWithOwner,
};

// Definition layer
use ra_ap_hir_def::{
    FunctionId, StructId, EnumId, TraitId, ImplId, TypeAliasId,
    ConstId, StaticId, ModuleId, FieldId,
    DefWithBodyId, ModuleDefId, AdtId, GenericDefId,
    DefMap, ItemScope, Resolver, ItemTree,
    DefDatabase, Body, BodySourceMap, ExprScopes, GenericParams,
};

// Type inference
use ra_ap_hir_ty::{
    Ty, TyKind, TyExt, Substitution, TraitRef,
    InferenceResult, TraitImpls, InherentImpls, TraitEnvironment,
    HirDatabase, CallableDefId, CallableSig,
};

// Syntax
use ra_ap_syntax::{
    SyntaxNode, SyntaxToken, SourceFile, AstNode,
    ast, TextRange, TextSize, Edition,
    algo::{find_node_at_offset, find_node_at_range},
    match_ast,
};

// IDE features
use ra_ap_ide::{
    AnalysisHost, Analysis, FilePosition, FileRange,
    NavigationTarget, CallItem, ReferenceSearchResult, StructureNode,
    HoverConfig, HoverResult, SignatureHelp, InlayHint,
    Query, StaticIndex,
};

// Base database
use ra_ap_base_db::{
    FileId, SourceRoot, SourceRootId, CrateGraph, CrateData,
    Dependency, SourceDatabase, FileLoader, CrateOrigin,
};

// Virtual file system
use ra_ap_vfs::{Vfs, VfsPath, ChangedFile, ChangeKind};
```

#### rustc Import Paths

```rust
#![feature(rustc_private)]

// Driver
extern crate rustc_driver;
extern crate rustc_interface;

// Core types
use rustc_middle::ty::{TyCtxt, Ty, TyKind, AdtDef, Instance, InstanceKind};
use rustc_middle::ty::{GenericArgsRef, GenericArgKind, TraitRef, TypingEnv};
use rustc_middle::ty::{Visibility, AssocItem, AssocKind, Generics, GenericPredicates};

// MIR
use rustc_middle::mir::{
    Body, BasicBlock, BasicBlockData, BasicBlocks,
    Local, LocalDecl, LocalKind,
    Place, PlaceElem, Operand, ConstOperand,
    Rvalue, BorrowKind, AggregateKind,
    Statement, StatementKind,
    Terminator, TerminatorKind, SwitchTargets,
    Location, SourceInfo, SourceScope,
    START_BLOCK, RETURN_PLACE,
};

// MIR traversal
use rustc_middle::mir::traversal::{preorder, postorder, reverse_postorder, reachable_as_bitset};

// MIR visitor
use rustc_middle::mir::visit::{Visitor, MutVisitor, PlaceContext};
use rustc_middle::mir::visit::{NonMutatingUseContext, MutatingUseContext};

// HIR
use rustc_hir::def::DefKind;
use rustc_hir::{Item, ItemKind, TraitItem, ImplItem, FnSig, FnDecl};

// Span
use rustc_span::{Span, BytePos, DefId, LocalDefId};
use rustc_span::source_map::SourceMap;

// Dominators
use rustc_data_structures::graph::dominators::{dominators, Dominators};

// Dataflow
use rustc_mir_dataflow::Analysis;
use rustc_mir_dataflow::{Results, ResultsCursor};
use rustc_mir_dataflow::impls::{
    MaybeLiveLocals, MaybeBorrowedLocals,
    MaybeInitializedPlaces, MaybeUninitializedPlaces,
    MaybeStorageLive, always_storage_live_locals,
};
use rustc_mir_dataflow::move_paths::{MoveData, MovePathIndex};

// Index types
use rustc_index::{IndexVec, Idx};
use rustc_index::bit_set::DenseBitSet;

// Loops
use rustc_middle::mir::loops::maybe_loop_headers;
```

#### Stable MIR / rustc_public Import Paths

```rust
use rustc_public::{local_crate, all_local_items, entry_fn, all_trait_decls, all_trait_impls};
use rustc_public::{CrateItem, Crate, DefId, FnDef, StaticDef};
use rustc_public::ty::{Ty, TyKind, GenericArgs};
use rustc_public::mir::{Body, BasicBlock, Statement, Terminator, TerminatorKind};
use rustc_public::mir::{Place, Rvalue, Operand, LocalDecl, ProjectionElem};
use rustc_public::mir::mono::Instance;
use rustc_public::run;
```

#### rustc_plugin / rustc_utils Import Paths

```rust
use rustc_plugin::{RustcPlugin, RustcPluginArgs, CrateFilter, cli_main, driver_main};
use rustc_utils::mir::body::BodyExt;
use rustc_utils::mir::place::PlaceExt;
use rustc_utils::source_map::span::SpanExt;
```

#### Charon Import Paths

```rust
use charon_lib::export::CrateData;
use charon_lib::ast::{FunDecl, TypeDecl, TraitDecl, TraitImpl, GlobalDecl};
use charon_lib::ast::{FunDeclId, TypeDeclId, TraitDeclId, TraitImplId, GlobalDeclId};
use charon_lib::ast::{Statement, Rvalue, Place, Operand, AggregateKind};
```

---

## Sources

This document was compiled from four primary research files:

1. **ra_ap_* API Research** — [ra_ap_hir docs](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/), [ra_ap_ide docs](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/), [rust-analyzer GitHub](https://github.com/rust-lang/rust-analyzer)
2. **rustc Internals Research** — [nightly rustc docs](https://doc.rust-lang.org/nightly/nightly-rustc/), [rustc dev guide](https://rustc-dev-guide.rust-lang.org/), [GitHub rust-lang/rust](https://github.com/rust-lang/rust)
3. **MIR APIs Research** — [MIR mod.rs](https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/mir/mod.rs), [MIR syntax.rs](https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/mir/syntax.rs), [dataflow framework](https://github.com/rust-lang/rust/blob/main/compiler/rustc_mir_dataflow/src/framework/mod.rs), [dominators](https://github.com/rust-lang/rust/blob/main/compiler/rustc_data_structures/src/graph/dominators/mod.rs)
4. **Polonius / Stable MIR / rustc_plugin / Charon Research** — [Polonius repo](https://github.com/rust-lang/polonius), [rustc_public docs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/index.html), [rustc_plugin repo](https://github.com/cognitive-engineering-lab/rustc_plugin), [Charon repo](https://github.com/AeneasVerif/charon), [Flowistry docs](https://willcrichton.net/flowistry/flowistry/)
