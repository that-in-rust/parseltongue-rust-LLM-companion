# rustc Internals Research: APIs for Building a Rich Code Graph

**Research Date:** March 2026  
**Sources:** [nightly rustc docs](https://doc.rust-lang.org/nightly/nightly-rustc/), [rustc dev guide](https://rustc-dev-guide.rust-lang.org/), [GitHub: rust-lang/rust](https://github.com/rust-lang/rust)

---

## Table of Contents

1. [TyCtxt<'tcx> — The Central Query Engine](#1-tyctxttcx--the-central-query-engine)
2. [DefId and LocalDefId](#2-defid-and-localdefid)
3. [HIR (High-level IR)](#3-hir-high-level-ir)
4. [Type System](#4-type-system)
5. [Span and SourceMap](#5-span-and-sourcemap)
6. [rustc_driver Entry Points](#6-rustc_driver-entry-points)
7. [MIR for Call Graph Analysis](#7-mir-for-call-graph-analysis)
8. [Complete Working Driver Example](#8-complete-working-driver-example)
9. [Graph Data Summary by Query](#9-graph-data-summary-by-query)

---

## 1. TyCtxt<'tcx> — The Central Query Engine

**Location:** `rustc_middle/src/ty/context.rs`  
**Docs:** https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html

```rust
pub struct TyCtxt<'tcx> {
    gcx: &'tcx GlobalCtxt<'tcx>,
}
```

`TyCtxt` is the central data structure of the compiler. It wraps `GlobalCtxt` and provides access to all compiler queries. It is obtained via the `after_analysis` callback in `rustc_driver::Callbacks`. Everything is accessed through `tcx.query_name(key)`.

### 1.1 Core Type Queries

| Query | Signature | Graph Data Enabled |
|-------|-----------|-------------------|
| `type_of` | `fn(impl IntoQueryParam<DefId>) -> EarlyBinder<'tcx, Ty<'tcx>>` | Type of any definition |
| `type_of_opaque` | `fn(impl IntoQueryParam<DefId>) -> Result<EarlyBinder<'tcx, Ty<'tcx>>, CyclePlaceholder>` | Opaque type (impl Trait) hidden type |
| `fn_sig` | `fn(impl IntoQueryParam<DefId>) -> EarlyBinder<'tcx, PolyFnSig<'tcx>>` | Function signature (args + return) |
| `adt_def` | `fn(impl IntoQueryParam<DefId>) -> AdtDef<'tcx>` | Struct/enum/union definition |
| `trait_def` | `fn(impl IntoQueryParam<DefId>) -> &'tcx TraitDef` | Trait definition |
| `generics_of` | `fn(impl IntoQueryParam<DefId>) -> &'tcx Generics` | Generic parameters |
| `predicates_of` | `fn(impl IntoQueryParam<DefId>) -> GenericPredicates<'tcx>` | Where-clauses and bounds |
| `explicit_predicates_of` | `fn(impl IntoQueryParam<DefId>) -> GenericPredicates<'tcx>` | Explicit predicates only |
| `inferred_outlives_of` | `fn(impl IntoQueryParam<DefId>) -> &'tcx [(Clause<'tcx>, Span)]` | Inferred outlives bounds |
| `variances_of` | `fn(impl IntoQueryParam<DefId>) -> &'tcx [Variance]` | Variance of type parameters |
| `param_env` | `fn(impl IntoQueryParam<DefId>) -> ParamEnv<'tcx>` | ParamEnv for trait solving |
| `constness` | `fn(impl IntoQueryParam<DefId>) -> Constness` | Whether item is const |
| `asyncness` | `fn(impl IntoQueryParam<DefId>) -> Asyncness` | Whether function is async |
| `def_kind` | `fn(impl IntoQueryParam<DefId>) -> DefKind` | Kind of definition |
| `def_span` | `fn(impl IntoQueryParam<DefId>) -> Span` | Span of definition |
| `def_ident_span` | `fn(impl IntoQueryParam<DefId>) -> Option<Span>` | Span of identifier only |
| `visibility` | `fn(impl IntoQueryParam<DefId>) -> Visibility<DefId>` | Visibility (pub/crate/etc) |
| `attrs_for_def` | `fn(impl IntoQueryParam<DefId>) -> &'tcx [Attribute]` | All attributes on definition |
| `codegen_fn_attrs` | `fn(impl IntoQueryParam<DefId>) -> &'tcx CodegenFnAttrs` | Codegen-relevant attributes |
| `fn_arg_idents` | `fn(impl IntoQueryParam<DefId>) -> &'tcx [Option<Ident>]` | Argument names |
| `coroutine_kind` | `fn(impl IntoQueryParam<DefId>) -> Option<CoroutineKind>` | Coroutine kind (async, gen, etc) |

### 1.2 Trait and Impl Queries

| Query | Signature | Graph Data Enabled |
|-------|-----------|-------------------|
| `trait_impls_of` | `fn(impl IntoQueryParam<DefId>) -> &'tcx TraitImpls` | All impls of a trait |
| `all_local_trait_impls` | `fn(()) -> &'tcx FxIndexMap<DefId, Vec<LocalDefId>>` | All trait→impls map for local crate |
| `local_trait_impls` | `fn(impl IntoQueryParam<DefId>) -> &'tcx [LocalDefId]` | Local impls of a specific trait |
| `implementations_of_trait` | `fn((CrateNum, DefId)) -> &'tcx [(DefId, Option<SimplifiedType>)]` | Impls of a trait in a given crate |
| `impl_trait_header` | `fn(impl IntoQueryParam<DefId>) -> Option<ImplTraitHeader<'tcx>>` | Trait + args for `impl Trait for Type` |
| `impl_parent` | `fn(impl IntoQueryParam<DefId>) -> Option<DefId>` | Parent of impl (for specialization) |
| `impl_subject` | `fn(impl IntoQueryParam<DefId>) -> EarlyBinder<'tcx, ImplSubject<'tcx>>` | What the impl is for |
| `impl_item_implementor_ids` | `fn(impl IntoQueryParam<DefId>) -> &'tcx DefIdMap<DefId>` | Maps impl items to trait items |
| `inherent_impls` | `fn(impl IntoQueryParam<DefId>) -> &'tcx [DefId]` | Inherent impls for a type |
| `crate_inherent_impls` | `fn(()) -> (&'tcx CrateInherentImpls, Result<...>)` | All inherent impls in crate |
| `trait_impls_in_crate` | `fn(CrateNum) -> &'tcx [DefId]` | All trait impls in a crate |
| `associated_item` | `fn(impl IntoQueryParam<DefId>) -> AssocItem` | A single associated item |
| `associated_items` | `fn(impl IntoQueryParam<DefId>) -> &'tcx AssocItems` | All associated items of trait/impl |
| `associated_item_def_ids` | `fn(impl IntoQueryParam<DefId>) -> &'tcx [DefId]` | DefIds of all assoc items |
| `specialization_graph_of` | `fn(impl IntoQueryParam<DefId>) -> Result<&'tcx Graph, ErrorGuaranteed>` | Specialization hierarchy |
| `specializes` | `fn((DefId, DefId)) -> bool` | Whether impl1 specializes impl2 |
| `codegen_select_candidate` | `fn(PseudoCanonicalInput<'tcx, TraitRef<'tcx>>) -> Result<&'tcx ImplSource<'tcx, ()>, ...>` | Select impl for codegen |
| `vtable_entries` | `fn(TraitRef<'tcx>) -> &'tcx [VtblEntry<'tcx>]` | Vtable layout for a trait |
| `own_existential_vtable_entries` | `fn(impl IntoQueryParam<DefId>) -> &'tcx [DefId]` | Own vtable entries |
| `resolve_instance_raw` | `fn(PseudoCanonicalInput<'tcx, (DefId, GenericArgsRef<'tcx>)>) -> Result<Option<Instance<'tcx>>, ...>` | Resolve to monomorphized instance |

### 1.3 HIR Access Queries

| Query | Signature | Graph Data Enabled |
|-------|-----------|-------------------|
| `hir_crate` | `fn(()) -> &'tcx Crate<'tcx>` | Full HIR crate (use sparingly) |
| `hir_crate_items` | `fn(()) -> &'tcx ModuleItems` | All items in entire crate |
| `hir_module_items` | `fn(LocalModDefId) -> &'tcx ModuleItems` | Items in a specific module |
| `hir_free_items` | `fn(()) -> impl Iterator<Item = ItemId>` | Free (top-level) items |
| `hir_item` | `fn(ItemId) -> &'tcx Item<'tcx>` | Item by ItemId |
| `hir_trait_item` | `fn(TraitItemId) -> &'tcx TraitItem<'tcx>` | Trait item |
| `hir_impl_item` | `fn(ImplItemId) -> &'tcx ImplItem<'tcx>` | Impl item |
| `hir_foreign_item` | `fn(ForeignItemId) -> &'tcx ForeignItem<'tcx>` | Foreign (extern) item |
| `hir_body` | `fn(BodyId) -> &'tcx Body<'tcx>` | Function body |
| `hir_body_owners` | `fn(()) -> impl Iterator<Item = LocalDefId>` | All body-owning DefIds |
| `local_def_id_to_hir_id` | `fn(impl IntoQueryParam<LocalDefId>) -> HirId` | LocalDefId → HirId |
| `hir_node_by_def_id` | `fn(LocalDefId) -> Node<'tcx>` | HIR node for a def |
| `hir_attrs` | `fn(HirId) -> &'tcx [Attribute]` | Attributes at a HIR node |
| `hir_span` | `fn(HirId) -> Span` | Span of a HIR node |
| `hir_fn_decl_by_hir_id` | `fn(HirId) -> Option<&'tcx FnDecl<'tcx>>` | Function decl at HIR id |
| `hir_fn_sig_by_hir_id` | `fn(HirId) -> Option<&'tcx FnSig<'tcx>>` | FnSig at HIR id |
| `hir_enclosing_body_owner` | `fn(HirId) -> LocalDefId` | Enclosing function |
| `hir_body_owner_kind` | `fn(DefId) -> BodyOwnerKind` | What kind of body owner |
| `hir_body_const_context` | `fn(LocalDefId) -> Option<ConstContext>` | Const context for body |
| `hir_def_path` | `fn(LocalDefId) -> DefPath` | Human-readable def path |
| `hir_def_key` | `fn(LocalDefId) -> DefKey` | DefKey for lookup |
| `hir_parent_iter` | `fn(HirId) -> impl Iterator<Item = (HirId, Node<'tcx>)>` | Ancestor chain |
| `hir_parent_id_iter` | `fn(HirId) -> impl Iterator<Item = HirId>` | Ancestor HirIds |
| `hir_get_if_local` | `fn(DefId) -> Option<Node<'tcx>>` | HIR node if local def |
| `hir_get_module` | `fn(LocalModDefId) -> (&'tcx Mod<'tcx>, Span, HirId)` | Module contents |
| `hir_get_generics` | `fn(LocalDefId) -> Option<&'tcx Generics<'tcx>>` | HIR generics |
| `hir_krate_attrs` | `fn(()) -> &'tcx [Attribute]` | Crate-level attributes |
| `source_span` | `fn(impl IntoQueryParam<LocalDefId>) -> Span` | Full absolute span of definition |

### 1.4 MIR Queries

| Query | Signature | Graph Data Enabled |
|-------|-----------|-------------------|
| `optimized_mir` | `fn(impl IntoQueryParam<DefId>) -> &'tcx Body<'tcx>` | Optimized MIR (for codegen/call graph) |
| `mir_built` | `fn(impl IntoQueryParam<LocalDefId>) -> &'tcx Steal<Body<'tcx>>` | Initial MIR before optimization |
| `mir_for_ctfe` | `fn(impl IntoQueryParam<DefId>) -> &'tcx Body<'tcx>` | MIR for const eval |
| `mir_promoted` | `fn(impl IntoQueryParam<LocalDefId>) -> (&'tcx Steal<Body>, &'tcx Steal<IndexVec<Promoted, Body>>)` | MIR + promoted constants |
| `promoted_mir` | `fn(impl IntoQueryParam<DefId>) -> &'tcx IndexVec<Promoted, Body<'tcx>>` | Promoted constant MIR |
| `mir_keys` | `fn(()) -> &'tcx FxIndexSet<LocalDefId>` | All DefIds that have MIR |
| `mir_shims` | `fn(InstanceKind<'tcx>) -> &'tcx Body<'tcx>` | MIR for compiler-generated shims |
| `mir_callgraph_cyclic` | `fn(impl IntoQueryParam<LocalDefId>) -> &'tcx UnordSet<LocalDefId>` | Functions in call cycle |
| `mir_inliner_callees` | `fn(InstanceKind<'tcx>) -> &'tcx [(DefId, GenericArgsRef<'tcx>)]` | Callee list for inliner |
| `is_mir_available` | `fn(impl IntoQueryParam<DefId>) -> bool` | Whether MIR is available |
| `is_ctfe_mir_available` | `fn(impl IntoQueryParam<DefId>) -> bool` | Whether const-eval MIR available |
| `typeck` | `fn(impl IntoQueryParam<LocalDefId>) -> &'tcx TypeckResults<'tcx>` | Type-check results for a body |

### 1.5 Iterating All DefIds

```rust
// Primary: iterate all body-owning definitions (functions, closures, etc.)
for def_id in tcx.hir_body_owners() {
    // def_id: LocalDefId
    let kind = tcx.def_kind(def_id);
    let ty = tcx.type_of(def_id).skip_binder();
}

// All free (top-level) items as HIR
for id in tcx.hir_free_items() {
    let item = tcx.hir_item(id);
    let def_id = id.owner_id.def_id; // LocalDefId
}

// All items in the crate (returns ModuleItems)
let all_items = tcx.hir_crate_items(());

// Iterate local DefIds (lower level)
for def_id in tcx.iter_local_def_id() { /* LocalDefId */ }

// All traits (including private)
for trait_id in tcx.all_traits_including_private() { /* DefId */ }

// All traits visible to user
for trait_id in tcx.visible_traits() { /* DefId */ }
```

### 1.6 Type Resolution and Normalization

```rust
// Get type for any definition
let ty: EarlyBinder<'tcx, Ty<'tcx>> = tcx.type_of(def_id);
let ty: Ty<'tcx> = ty.skip_binder(); // without substitution
let ty: Ty<'tcx> = ty.instantiate(tcx, args); // with generic args substituted

// Get function signature
let sig: EarlyBinder<'tcx, PolyFnSig<'tcx>> = tcx.fn_sig(def_id);
let poly_sig: PolyFnSig<'tcx> = sig.skip_binder();
let sig: FnSig<'tcx> = poly_sig.skip_binder(); // remove late-bound lifetimes

// Typeck results for HIR body (expression types)
let typeck = tcx.typeck(local_def_id);
let expr_ty: Ty<'tcx> = typeck.node_type(hir_id);
let expr_ty_adjusted: Ty<'tcx> = typeck.expr_ty_adjusted(expr);

// Normalize erasing regions (common for codegen work)
let norm_ty = tcx.normalize_erasing_regions(typing_env, ty);

// Resolve instance (monomorphization)
let instance = Instance::resolve(tcx, typing_env, def_id, generic_args)
    .ok().flatten();

// Resolve instance for codegen
let instance = tcx.resolve_instance_raw(PseudoCanonicalInput { ... });
```

### 1.7 Trait Resolution Queries

```rust
// All impls of a trait
let impls: &TraitImpls = tcx.trait_impls_of(trait_def_id);

// All local trait impls as a map: trait DefId -> Vec<LocalDefId>
let map = tcx.all_local_trait_impls(());

// Impls of a trait in a specific crate
let impls = tcx.implementations_of_trait((crate_num, trait_def_id));

// Check if an impl specializes another
let does_specialize = tcx.specializes((impl_id1, impl_id2));

// Iterate relevant impls for a type+trait
tcx.for_each_relevant_impl(trait_def_id, self_ty, |impl_def_id| { ... });

// Select concrete impl for codegen
let impl_source = tcx.codegen_select_candidate(input);

// Get trait this impl is for
let trait_ref: Option<EarlyBinder<TraitRef>> = tcx.impl_trait_header(impl_def_id)
    .map(|h| h.trait_ref);
```

### 1.8 DefPath and Debugging

```rust
// Human-readable path string: "my_crate::module::Foo::bar"
let path_str: String = tcx.def_path_str(def_id);
let debug_str: String = tcx.def_path_debug_str(def_id);

// Structured DefPath
let def_path: DefPath = tcx.def_path(def_id);
let def_path: DefPath = tcx.hir_def_path(local_def_id);

// Crate info
let crate_name: Symbol = tcx.crate_name(crate_num);
let all_crates: &[CrateNum] = tcx.crates(());

// Parent module
let parent_mod: LocalModDefId = tcx.parent_module_from_def_id(local_def_id);

// Visibility
let vis: Visibility<DefId> = tcx.visibility(def_id);
// vis.is_public(), vis.is_accessible_from(module, tcx)
```

### 1.9 Stability and Deprecation

```rust
let stability: Option<Stability> = tcx.lookup_stability(def_id);
let const_stability = tcx.lookup_const_stability(def_id);
let deprecation: Option<Deprecation> = tcx.lookup_deprecation(def_id);
let is_notable = tcx.is_doc_notable_trait(def_id);
```

### 1.10 Layout and ABI

```rust
// Get memory layout
let layout = tcx.layout_of(PseudoCanonicalInput { typing_env, value: ty });
if let Ok(tyl) = layout {
    tyl.size, tyl.align, tyl.abi // AbiAndPrefAlign
}

// Function ABI
let fn_abi = tcx.fn_abi_of_instance(input);
```

---

## 2. DefId and LocalDefId

**Location:** `rustc_span/src/def_id.rs`  
**Docs:** https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/def_id/struct.DefId.html

### 2.1 DefId Structure

```rust
#[repr(C)]
pub struct DefId {
    pub index: DefIndex,  // Index within the crate
    pub krate: CrateNum,  // Which crate
}
```

A `DefId` uniquely identifies any definition anywhere in the full crate graph (including dependencies). It combines a crate index and a definition index.

### 2.2 DefId Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `is_local` | `fn(self) -> bool` | True if defined in current crate |
| `as_local` | `fn(self) -> Option<LocalDefId>` | Convert to LocalDefId if local |
| `expect_local` | `fn(self) -> LocalDefId` | Convert, panic if not local |
| `is_crate_root` | `fn(self) -> bool` | True if this is a crate root |
| `as_crate_root` | `fn(self) -> Option<CrateNum>` | CrateNum if crate root |
| `is_top_level_module` | `fn(self) -> bool` | True if top-level module |

### 2.3 LocalDefId

```rust
pub struct LocalDefId {
    pub local_def_index: DefIndex,
}
```

`LocalDefId` is a `DefId` known to be in the current crate. It is used in all HIR queries (they only work on local definitions).

**Conversions:**
```rust
// LocalDefId → DefId
let def_id: DefId = local_def_id.to_def_id();
let def_id: DefId = DefId::from(local_def_id);

// DefId → LocalDefId
let local: Option<LocalDefId> = def_id.as_local();
let local: LocalDefId = def_id.expect_local(); // panics if not local

// LocalDefId ↔ HirId
let hir_id: HirId = tcx.local_def_id_to_hir_id(local_def_id);
let local_def_id: LocalDefId = hir_id.owner.def_id;
```

### 2.4 DefKind — All Variants

**Docs:** https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/def/enum.DefKind.html

```rust
pub enum DefKind {
    // Types
    Mod,            // mod foo { ... }
    Struct,         // struct Foo { ... } (Ctor is DefKind::Ctor)
    Union,          // union Foo { ... }
    Enum,           // enum Foo { ... }
    Variant,        // Foo::Bar (Ctor is DefKind::Ctor)
    Trait,          // trait Foo { ... }
    TyAlias,        // type Foo = Bar
    ForeignTy,      // extern type T (from extern block)
    TraitAlias,     // trait IntIterator = Iterator<Item = i32>
    AssocTy,        // associated type in trait/impl
    TyParam,        // T in struct Vec<T>

    // Values
    Fn,             // fn foo() { ... }
    Const,          // const FOO: u32 = 1
    ConstParam,     // N in struct Foo<const N: usize>
    Static { safety: Safety, mutability: Mutability, nested: bool },
    Ctor(CtorOf, CtorKind), // constructor: Foo { } or Foo( )
    AssocFn,        // associated fn in trait/impl
    AssocConst,     // associated const in trait/impl

    // Macros
    Macro(MacroKind),

    // Other
    ExternCrate,    // extern crate foo
    Use,            // use foo::bar
    ForeignMod,     // extern { ... } block
    AnonConst,      // [u8; 1 + 2] — the 1+2 part
    InlineConst,    // const { 1 + 2 }
    OpaqueTy,       // impl Trait
    Field,          // field in struct/enum/union
    LifetimeParam,  // 'a in struct Foo<'a>
    GlobalAsm,      // global_asm!
    Impl { of_trait: bool }, // impl Block or impl Trait for Type
    Closure,        // closure, coroutine, or coroutine-closure
    SyntheticCoroutineBody, // synthetic body for async closure
}
```

```rust
// Common usage
let kind: DefKind = tcx.def_kind(def_id);
match kind {
    DefKind::Fn | DefKind::AssocFn => { /* function */ }
    DefKind::Struct | DefKind::Enum | DefKind::Union => { /* ADT */ }
    DefKind::Trait => { /* trait */ }
    DefKind::Impl { of_trait } => { /* impl block */ }
    _ => {}
}
```

### 2.5 DefPath

```rust
pub struct DefPath {
    pub data: Vec<DisambiguatedDefPathData>,
    pub krate: CrateNum,
}

impl DefPath {
    // Get path as string without crate prefix (no TyCtxt needed)
    pub fn to_string_no_crate_verbose(&self) -> String
    // Filename-friendly version without crate prefix
    pub fn to_filename_friendly_no_crate(&self) -> String
}
```

**DefPathData variants** (components of a path):

```rust
pub enum DefPathData {
    CrateRoot,
    Impl,
    ForeignMod,
    Use,
    GlobalAsm,
    TypeNs(Symbol),       // type namespace: struct/enum/trait names
    ValueNs(Symbol),      // value namespace: fn/const/static names
    MacroNs(Symbol),      // macro namespace
    LifetimeNs(Symbol),   // lifetime names
    Closure,
    Ctor,
    AnonConst,
    OpaqueTy,
    OpaqueLifetime(Symbol),
    AnonAssocTy(Symbol),
    SyntheticCoroutineBody,
    NestedStatic,
}
```

```rust
// Getting DefPath from DefId via TyCtxt
let def_path: DefPath = tcx.def_path(def_id);           // any DefId
let def_path: DefPath = tcx.hir_def_path(local_def_id); // local only

// Human-readable (most common)
let path_str: String = tcx.def_path_str(def_id);
// → "std::collections::HashMap::new"
```

---

## 3. HIR (High-level IR)

**Location:** `rustc_hir/src/hir.rs`  
**Docs:** https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/hir/enum.ItemKind.html

### 3.1 HirId

```rust
pub struct HirId {
    pub owner: OwnerId,          // The owning definition (LocalDefId wrapper)
    pub local_id: ItemLocalId,   // Local ID within that owner
}

pub struct OwnerId {
    pub def_id: LocalDefId,
}
```

`HirId` uniquely identifies a node in the HIR of the current crate. More stable than raw indices for incremental compilation.

**Conversions:**
```rust
// HirId → LocalDefId (owner)
let local_def_id: LocalDefId = hir_id.owner.def_id;

// LocalDefId → HirId
let hir_id: HirId = tcx.local_def_id_to_hir_id(local_def_id);

// Get HIR node
let node: Node<'tcx> = tcx.hir_node_by_def_id(local_def_id);
let node: Node<'tcx> = tcx.hir_node(hir_id); // via map

// Parent
let parent_hir_id: HirId = tcx.parent_hir_id(hir_id);
let parent_node: Node<'tcx> = tcx.parent_hir_node(hir_id);
```

### 3.2 ItemKind — All Variants

| Variant | Data | Description |
|---------|------|-------------|
| `ExternCrate(Option<Symbol>)` | Original name if renamed | `extern crate foo` |
| `Use(&'hir UsePath, UseKind)` | Path + use kind | `use foo::bar::*` |
| `Static(&'hir Ty, Mutability, BodyId)` | Type, mutability, body | `static X: T = ...` |
| `Const(&'hir Ty, &'hir Generics, BodyId)` | Type, generics, body | `const X: T = ...` |
| `Fn { sig, generics, body, has_body }` | FnSig, generics, body | Function declaration |
| `Macro(&'hir MacroDef, MacroKind)` | Macro definition | `macro_rules!` or `macro` |
| `Mod(&'hir Mod)` | Module contents | `mod foo { ... }` |
| `ForeignMod { abi, items }` | ABI + foreign items | `extern "C" { ... }` |
| `GlobalAsm { asm, fake_body }` | Inline asm | `global_asm!` |
| `TyAlias(&'hir Ty, &'hir Generics)` | Aliased type, generics | `type Foo = Bar<u8>` |
| `Enum(EnumDef, &'hir Generics)` | Variants, generics | `enum Foo<A, B> { ... }` |
| `Struct(VariantData, &'hir Generics)` | Fields, generics | `struct Foo<A> { ... }` |
| `Union(VariantData, &'hir Generics)` | Fields, generics | `union Foo { ... }` |
| `Trait(IsAuto, Safety, &'hir Generics, GenericBounds, &'hir [TraitItemRef])` | Auto/safety/bounds/items | `trait Foo { ... }` |
| `TraitAlias(&'hir Generics, GenericBounds)` | Generics, bounds | `trait IntIter = Iterator<Item=i32>` |
| `Impl(&'hir Impl)` | Full impl block | `impl<A> Trait for Foo { ... }` |

```rust
// Accessing item kind
let item: &Item<'tcx> = tcx.hir_item(item_id);
match &item.kind {
    ItemKind::Fn { sig, generics, body, .. } => {
        let fn_decl: &FnDecl = sig.decl;
        let inputs: &[Ty] = fn_decl.inputs;
        let output: &FnRetTy = &fn_decl.output;
        let body: &Body = tcx.hir_body(*body);
    }
    ItemKind::Struct(variant_data, generics) => {
        let fields: &[FieldDef] = variant_data.fields();
    }
    ItemKind::Impl(impl_block) => {
        let trait_ref: Option<&TraitRef> = impl_block.of_trait.as_ref();
        let self_ty: &Ty = impl_block.self_ty;
        let items: &[ImplItemRef] = impl_block.items;
    }
    _ => {}
}
```

### 3.3 Key HIR Types

**Item:**
```rust
pub struct Item<'hir> {
    pub ident: Ident,
    pub owner_id: OwnerId,
    pub kind: ItemKind<'hir>,
    pub span: Span,
    pub vis_span: Span,
}
```

**TraitItem:**
```rust
pub struct TraitItem<'hir> {
    pub ident: Ident,
    pub owner_id: OwnerId,
    pub generics: &'hir Generics<'hir>,
    pub kind: TraitItemKind<'hir>,
    pub span: Span,
    pub defaultness: Defaultness,
}
// TraitItemKind: Const(Ty, Option<BodyId>), Fn(FnSig, TraitFn), Type(bounds, Option<Ty>)
```

**ImplItem:**
```rust
pub struct ImplItem<'hir> {
    pub ident: Ident,
    pub owner_id: OwnerId,
    pub generics: &'hir Generics<'hir>,
    pub kind: ImplItemKind<'hir>,
    pub span: Span,
    pub vis_span: Span,
    pub defaultness: Defaultness,
}
// ImplItemKind: Const(Ty, BodyId), Fn(FnSig, BodyId), Type(Ty)
```

**ForeignItem:**
```rust
pub struct ForeignItem<'hir> {
    pub ident: Ident,
    pub owner_id: OwnerId,
    pub kind: ForeignItemKind<'hir>,
    pub span: Span,
    pub vis_span: Span,
}
// ForeignItemKind: Fn(FnDecl, &[Ident], Generics), Static(Ty, Mutability, Safety), Type
```

**FnSig (HIR):**
```rust
pub struct FnSig<'hir> {
    pub header: FnHeader,     // constness, asyncness, safety, abi
    pub decl: &'hir FnDecl<'hir>,
    pub span: Span,
}

pub struct FnDecl<'hir> {
    pub inputs: &'hir [Ty<'hir>],    // parameter types
    pub output: FnRetTy<'hir>,        // return type
    pub c_variadic: bool,
    pub implicit_self: ImplicitSelfKind,
    pub lifetime_elision_allowed: bool,
}
```

**Body:**
```rust
pub struct Body<'hir> {
    pub params: &'hir [Param<'hir>],
    pub value: &'hir Expr<'hir>,   // function body expression
    pub coroutine_kind: Option<CoroutineKind>,
}
```

### 3.4 HIR Traversal

#### Method 1: Query-based (Incremental-friendly, Preferred)
```rust
// Iterate all items
for item_id in tcx.hir_free_items() {
    let item: &Item<'tcx> = tcx.hir_item(item_id);
    let def_id: LocalDefId = item_id.owner_id.def_id;
}

// Iterate all body owners (functions, closures, constants)
for def_id in tcx.hir_body_owners() {
    let kind = tcx.def_kind(def_id);
    if matches!(kind, DefKind::Fn | DefKind::AssocFn | DefKind::Closure) {
        let body = tcx.hir_body_owned_by(def_id);
    }
}

// Visit all item-likes in crate
tcx.hir_visit_all_item_likes_in_crate(&mut my_visitor);

// Walk top-level module with nested visitor
tcx.hir_walk_toplevel_module(&mut my_visitor);
```

#### Method 2: intravisit::Visitor (Deep AST Traversal)
```rust
use rustc_hir::intravisit::{self, Visitor, NestedFilter};

struct MyVisitor<'tcx> {
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> Visitor<'tcx> for MyVisitor<'tcx> {
    type NestedFilter = NestedFilter::OnlyBodies; // or NestedFilter::All

    fn nested_visit_map(&self) -> Self::Map {
        self.tcx.hir()
    }

    fn visit_item(&mut self, item: &'tcx hir::Item<'tcx>) {
        // Process item
        intravisit::walk_item(self, item); // recurse
    }

    fn visit_fn(&mut self, fk: FnKind, decl: &FnDecl, body_id: BodyId, _: Span, id: LocalDefId) {
        // Process function
        intravisit::walk_fn(self, fk, decl, body_id, id);
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        intravisit::walk_expr(self, expr);
    }
}
```

**All available `walk_*` functions:**
`walk_anon_const`, `walk_arm`, `walk_assoc_item_constraint`, `walk_block`, `walk_body`,
`walk_const_arg`, `walk_enum_def`, `walk_expr`, `walk_expr_field`, `walk_field_def`,
`walk_fn`, `walk_fn_decl`, `walk_fn_ret_ty`, `walk_foreign_item`, `walk_foreign_item_ref`,
`walk_generic_arg`, `walk_generic_args`, `walk_generic_param`, `walk_generics`,
`walk_ident`, `walk_impl_item`, `walk_impl_item_ref`, `walk_inline_asm`,
`walk_item`, `walk_lifetime`, `walk_local`, `walk_mod`, `walk_opaque_ty`,
`walk_param`, `walk_param_bound`, `walk_pat`, `walk_path`, `walk_path_segment`,
`walk_poly_trait_ref`, `walk_qpath`, `walk_stmt`, `walk_struct_def`, `walk_trait_item`,
`walk_trait_item_ref`, `walk_trait_ref`, `walk_ty`, `walk_use`, `walk_variant`,
`walk_where_predicate`

### 3.5 HIR Map Methods (via tcx.hir())
```rust
// Expect specific kinds
tcx.hir().expect_item(local_def_id) -> &'hir Item
tcx.hir().expect_impl_item(local_def_id) -> &'hir ImplItem
tcx.hir().expect_trait_item(local_def_id) -> &'hir TraitItem
tcx.hir().expect_foreign_item(owner_id) -> &'hir ForeignItem
tcx.hir().expect_expr(hir_id) -> &'hir Expr
tcx.hir().expect_opaque_ty(local_def_id) -> &'hir OpaqueTy

// Attributes
tcx.hir().attrs(hir_id) -> &'hir [Attribute]

// Span and name
tcx.hir().span(hir_id) -> Span
tcx.hir().span_with_body(hir_id) -> Span
tcx.hir().name(hir_id) -> Symbol
tcx.hir().opt_name(hir_id) -> Option<Symbol>
tcx.hir().ident(hir_id) -> Ident

// Foreign ABI
tcx.hir().get_foreign_abi(hir_id) -> ExternAbi

// Function output
tcx.hir().get_fn_output(local_def_id) -> Option<&'hir FnRetTy>
```

### 3.6 Attribute and Visibility Access

```rust
// Access attributes on any HIR node
let attrs: &[Attribute] = tcx.hir_attrs(hir_id);
// or on any definition
let attrs: &[Attribute] = tcx.attrs_for_def(def_id);

// Check specific attributes
for attr in attrs {
    if attr.has_name(sym::inline) { ... }
    if attr.has_name(sym::derive) { ... }
}

// Visibility
let vis: Visibility<DefId> = tcx.visibility(def_id);
match vis {
    Visibility::Public => {},
    Visibility::Restricted(module_def_id) => {},
}
```

### 3.7 HIR → Type System Bridge

```rust
// From a function's LocalDefId, get its type-checked results
let typeck_results: &TypeckResults<'tcx> = tcx.typeck(local_def_id);

// Get the type of any HIR expression
let expr_ty: Ty<'tcx> = typeck_results.node_type(expr.hir_id);

// Get the type of a pattern
let pat_ty: Ty<'tcx> = typeck_results.pat_ty(pat);

// Get the type of a node
let ty: Ty<'tcx> = typeck_results.node_type(hir_id);

// TypeckResults also provides:
// .type_dependent_def(hir_id) -> Option<(DefKind, DefId)>  -- method resolution
// .qpath_res(qpath, hir_id) -> Res                          -- path resolution
// .adjustments() -> &LocalTableInContext<Vec<Adjustment>>   -- coercions
```

---

## 4. Type System

**Location:** `rustc_middle/src/ty/`  
**Docs:** https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/

### 4.1 Ty<'tcx> and TyKind

```rust
// Ty is a thin wrapper (always use Ty, never TyKind directly)
pub struct Ty<'tcx>(Interned<'tcx, WithCachedTypeInfo<TyKind<'tcx>>>);

// Access the kind
let kind: &TyKind<'tcx> = ty.kind();

// Common type constructors via tcx
tcx.types.bool, tcx.types.char, tcx.types.i32, tcx.types.f64
tcx.types.unit    // ()
tcx.types.never   // !
tcx.types.str_    // str
```

### 4.2 TyKind — All Variants

| Variant | Data | Represents |
|---------|------|------------|
| `Bool` | — | `bool` |
| `Char` | — | `char` |
| `Int(IntTy)` | `IntTy` | `i8, i16, i32, i64, i128, isize` |
| `Uint(UintTy)` | `UintTy` | `u8, u16, u32, u64, u128, usize` |
| `Float(FloatTy)` | `FloatTy` | `f32, f64` |
| `Adt(AdtDef<'tcx>, GenericArgsRef<'tcx>)` | ADT def + args | Struct, enum, union |
| `Foreign(DefId)` | DefId | `extern type T` |
| `Str` | — | `str` (unsized) |
| `Array(Ty<'tcx>, Const<'tcx>)` | element type + length | `[T; N]` |
| `Pat(Ty<'tcx>, Pat<'tcx>)` | type + pattern | Pattern newtype |
| `Slice(Ty<'tcx>)` | element type | `[T]` (unsized) |
| `RawPtr(Ty<'tcx>, Mutability)` | pointee + mut | `*mut T` / `*const T` |
| `Ref(Region<'tcx>, Ty<'tcx>, Mutability)` | lifetime + type + mut | `&'a T` / `&'a mut T` |
| `FnDef(DefId, GenericArgsRef<'tcx>)` | fn DefId + args | Function item type |
| `FnPtr(Binder<FnSigTys>, FnHeader)` | signature | `fn(i32) -> bool` |
| `UnsafeBinder(UnsafeBinderInner)` | — | Unsafe binder (lifetime erasure) |
| `Dynamic(BoundExistentialPredicates, Region, DynKind)` | predicates + lifetime | `dyn Trait + Send + 'a` |
| `Closure(DefId, GenericArgsRef)` | closure def + args | Closure type |
| `CoroutineClosure(DefId, GenericArgsRef)` | coroutine-closure def + args | `async` closure |
| `Coroutine(DefId, GenericArgsRef)` | coroutine def + args | Coroutine / async fn body |
| `CoroutineWitness(DefId, GenericArgsRef)` | — | Types stored in coroutine |
| `Never` | — | `!` (never type) |
| `Tuple(Tys)` | list of types | `(i32, bool)` |
| `Alias(AliasTyKind, AliasTy)` | kind + def+args | Projection / opaque / type alias |
| `Param(ParamTy)` | param index + name | Generic `T` |
| `Bound(DebruijnIndex, BoundTy)` | binder depth + var | `for<'a>` bound variable |
| `Placeholder(PlaceholderTy)` | — | During trait solving |
| `Infer(InferTy)` | inference var | During type checking |
| `Error(ErrorGuaranteed)` | — | Error propagation |

```rust
// Common pattern matching
match ty.kind() {
    TyKind::Adt(adt_def, generic_args) => {
        let is_struct = adt_def.is_struct();
        let is_enum = adt_def.is_enum();
        let is_union = adt_def.is_union();
        let def_id = adt_def.did();
        for variant in adt_def.variants() { ... }
    }
    TyKind::Ref(region, inner_ty, mutability) => { ... }
    TyKind::FnDef(def_id, args) => {
        let sig = tcx.fn_sig(*def_id).instantiate(tcx, args);
    }
    TyKind::Tuple(tys) => {
        for ty in tys.iter() { ... }
    }
    TyKind::Param(param_ty) => {
        let name = param_ty.name; // Symbol
        let index = param_ty.index;
    }
    _ => {}
}
```

### 4.3 AdtDef — Struct/Enum/Union Definitions

```rust
pub struct AdtDef<'tcx>(Interned<'tcx, AdtDefData>);
```

| Method | Signature | Description |
|--------|-----------|-------------|
| `did` | `fn(self) -> DefId` | Definition ID |
| `is_struct` | `fn(self) -> bool` | True for structs |
| `is_enum` | `fn(self) -> bool` | True for enums |
| `is_union` | `fn(self) -> bool` | True for unions |
| `variants` | `fn(self) -> &'tcx IndexSlice<VariantIdx, VariantDef>` | All variants |
| `variant` | `fn(self, idx: VariantIdx) -> &'tcx VariantDef` | Single variant by index |
| `non_enum_variant` | `fn(self) -> &'tcx VariantDef` | For struct/union (single variant) |
| `variant_with_id` | `fn(self, vid: DefId) -> &'tcx VariantDef` | Variant by DefId |
| `variant_index_with_id` | `fn(self, vid: DefId) -> VariantIdx` | Index by DefId |
| `all_fields` | `fn(self) -> impl Iterator<Item = &'tcx FieldDef>` | All fields across all variants |
| `variant_descr` | `fn(self) -> &'static str` | "struct", "enum", or "union" |

**VariantDef fields:**
```rust
pub struct VariantDef {
    pub def_id: DefId,
    pub ctor: Option<(CtorKind, DefId)>,
    pub name: Symbol,
    pub discr: VariantDiscr,
    pub fields: IndexVec<FieldIdx, FieldDef>,
    pub flags: VariantFlags,
}

// Methods:
variant.ident(tcx) -> Ident
variant.ctor_kind() -> Option<CtorKind>
variant.ctor_def_id() -> Option<DefId>
variant.is_field_list_non_exhaustive() -> bool
```

**FieldDef fields:**
```rust
pub struct FieldDef {
    pub did: DefId,
    pub name: Symbol,
    pub vis: Visibility<DefId>,
    pub safety: Safety,
    pub value: Option<DefId>, // default value (if any)
}

// Get field type (with generic substitution)
field.ty(tcx, generic_args) -> Ty<'tcx>
```

### 4.4 Generics

```rust
pub struct Generics {
    pub parent: Option<DefId>,          // Parent item's generics (for associated items)
    pub parent_count: usize,            // Number of params from parent
    pub own_params: Vec<GenericParamDef>, // This item's own parameters
    pub param_def_id_to_index: FxHashMap<DefId, u32>,
    pub has_self: bool,                 // True for trait methods (implicit Self param)
    pub has_late_bound_regions: Option<Span>,
}

pub struct GenericParamDef {
    pub name: Symbol,
    pub def_id: DefId,
    pub index: u32,
    pub pure_wrt_drop: bool,
    pub kind: GenericParamDefKind,
}

// GenericParamDefKind variants:
// Lifetime, Type { has_default, synthetic }, Const { has_default, is_host_effect, synthetic }
```

```rust
// Getting generics
let generics: &Generics = tcx.generics_of(def_id);
for param in &generics.own_params {
    match param.kind {
        GenericParamDefKind::Type { .. } => { /* type param */ }
        GenericParamDefKind::Lifetime => { /* lifetime param */ }
        GenericParamDefKind::Const { .. } => { /* const param */ }
    }
}

// Getting predicates / where-clauses
let predicates: GenericPredicates<'tcx> = tcx.predicates_of(def_id);
for (predicate, span) in predicates.predicates {
    match predicate.kind().skip_binder() {
        ClauseKind::Trait(trait_pred) => {
            let trait_def_id = trait_pred.trait_ref.def_id;
            let self_ty = trait_pred.self_ty();
        }
        ClauseKind::TypeOutlives(outlives) => { ... }
        ClauseKind::RegionOutlives(outlives) => { ... }
        _ => {}
    }
}
```

### 4.5 TraitRef and Predicates

```rust
// TraitRef: represents `Type: Trait<Args>`
pub struct TraitRef<'tcx> {
    pub def_id: DefId,              // The trait
    pub args: GenericArgsRef<'tcx>, // Generic args [Self, Arg1, Arg2, ...]
}

impl TraitRef<'tcx> {
    pub fn self_ty(self) -> Ty<'tcx>  // Self type
    pub fn args(self) -> GenericArgsRef<'tcx>
}

// Predicates
pub enum ClauseKind<'tcx> {
    Trait(TraitPredicate<'tcx>),                // T: Trait
    RegionOutlives(RegionOutlivesPredicate<'tcx>), // 'a: 'b
    TypeOutlives(TypeOutlivesPredicate<'tcx>),   // T: 'a
    Projection(ProjectionPredicate<'tcx>),       // <T as Trait>::Assoc = U
    ConstArgHasType(Const<'tcx>, Ty<'tcx>),
    WellFormed(GenericArg<'tcx>),
    ConstEvaluatable(Const<'tcx>),
}
```

### 4.6 Instance<'tcx> — Monomorphized Functions

```rust
pub struct Instance<'tcx> {
    pub def: InstanceKind<'tcx>,         // What kind of instance
    pub args: GenericArgsRef<'tcx>,      // Concrete generic arguments
}
```

**InstanceKind variants** (see also §7.3):

| Variant | Description |
|---------|-------------|
| `Item(DefId)` | User-defined fn, closure, coroutine |
| `Intrinsic(DefId)` | Compiler intrinsic |
| `VTableShim(DefId)` | Vtable shim for unsized self |
| `ReifyShim(DefId, Option<ReifyReason>)` | fn() pointer shim |
| `FnPtrShim(DefId, Ty<'tcx>)` | FnTrait impl for fn pointers |
| `Virtual(DefId, usize)` | Dynamic dispatch (vtable slot) |
| `ClosureOnceShim { call_once, track_caller }` | FnOnce for FnMut closure |
| `DropGlue(DefId, Option<Ty<'tcx>>)` | `drop_in_place` |
| `CloneShim(DefId, Ty<'tcx>)` | Compiler-generated Clone |
| `ThreadLocalShim(DefId)` | Thread-local accessor |
| `AsyncDropGlueCtorShim(DefId, Ty<'tcx>)` | Async drop |

```rust
// Create an instance
let instance = Instance::mono(tcx, def_id);   // monomorphic
let instance = Instance::new_raw(def_id, args);

// Resolve: find concrete impl for a trait method
let result = Instance::try_resolve(tcx, typing_env, def_id, generic_args);
// or via query:
let result = tcx.resolve_instance_raw(PseudoCanonicalInput { typing_env, value: (def_id, args) });

// Get symbol name for codegen
let sym = tcx.symbol_name(instance);

// Get ABI
let abi = tcx.fn_abi_of_instance(PseudoCanonicalInput { typing_env, value: (instance, extra_args) });

// Check if should codegen locally
let local = tcx.should_codegen_locally(instance);
```

### 4.7 GenericArgsRef (SubstsRef)

```rust
type GenericArgsRef<'tcx> = &'tcx List<GenericArg<'tcx>>;

pub enum GenericArgKind<'tcx> {
    Lifetime(Region<'tcx>),
    Type(Ty<'tcx>),
    Const(Const<'tcx>),
}
```

```rust
// Iterate generic args
for arg in generic_args.iter() {
    match arg.unpack() {
        GenericArgKind::Type(ty) => { ... }
        GenericArgKind::Lifetime(region) => { ... }
        GenericArgKind::Const(c) => { ... }
    }
}

// Substitute generics into a type
let instantiated_ty = ty.instantiate(tcx, generic_args);
```

### 4.8 AssocItem — Associated Items

```rust
pub struct AssocItem {
    pub def_id: DefId,
    pub kind: AssocKind,                    // Type, Fn, or Const
    pub container: AssocItemContainer,      // TraitContainer or ImplContainer
    pub trait_item_def_id: Option<DefId>,   // Corresponding trait item (for impls)
}

// Methods:
assoc_item.name(tcx) -> Symbol / Ident
assoc_item.visibility(tcx) -> Visibility<DefId>
assoc_item.container_id(tcx) -> DefId
assoc_item.trait_container(tcx) -> Option<DefId>  // Some if in trait
assoc_item.impl_container(tcx) -> Option<DefId>   // Some if in impl
assoc_item.is_fn() -> bool
assoc_item.is_type() -> bool
assoc_item.is_method() -> bool  // fn with self parameter
```

```rust
// Get all associated items for a trait or impl
let assoc_items: &AssocItems = tcx.associated_items(def_id);
for item in assoc_items.in_definition_order() {
    match item.kind {
        AssocKind::Fn => { /* method or associated function */ }
        AssocKind::Type => { /* associated type */ }
        AssocKind::Const => { /* associated constant */ }
    }
}
```

---

## 5. Span and SourceMap

**Location:** `rustc_span/src/lib.rs`  
**Docs:** https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/struct.Span.html

### 5.1 Span Structure

```rust
// Span is a compressed handle (not directly inspectable — use data())
pub struct Span { /* opaque */ }

pub struct SpanData {
    pub lo: BytePos,
    pub hi: BytePos,
    pub ctxt: SyntaxContext,
    pub parent: Option<LocalDefId>,
}

pub struct BytePos(pub u32);  // Absolute byte offset into source
```

### 5.2 Span Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `lo` | `fn(self) -> BytePos` | Start byte position |
| `hi` | `fn(self) -> BytePos` | End byte position |
| `data` | `fn(self) -> SpanData` | Full span data |
| `ctxt` | `fn(self) -> SyntaxContext` | Syntax context (macro expansion) |
| `parent` | `fn(self) -> Option<LocalDefId>` | Enclosing definition |
| `from_expansion` | `fn(self) -> bool` | True if from macro expansion |
| `source_callsite` | `fn(self) -> Span` | Original macro callsite span |
| `is_visible` | `fn(self, sm: &SourceMap) -> bool` | Not from external macro |
| `shrink_to_lo` | `fn(self) -> Span` | Empty span at start |
| `shrink_to_hi` | `fn(self) -> Span` | Empty span at end |
| `to` | `fn(self, end: Span) -> Span` | Merge two spans |
| `between` | `fn(self, end: Span) -> Span` | Gap between spans |
| `macro_backtrace` | `fn(self) -> impl Iterator<Item = ExpnData>` | Expansion chain |
| `in_derive_expansion` | `fn(self) -> bool` | From `#[derive]` |
| `is_rust_2015` | `fn(self) -> bool` | Edition check |

### 5.3 SourceMap — Converting Span to File/Line/Column

**Docs:** https://doc.rust-lang.org/nightly/nightly-rustc/rustc_span/source_map/struct.SourceMap.html

```rust
// Get SourceMap from TyCtxt
let sm: &SourceMap = tcx.sess.source_map();

// Convert BytePos to file + line + column (Loc)
let loc: Loc = sm.lookup_char_pos(span.lo());
// loc.file: Arc<SourceFile>
// loc.line: usize  (1-based)
// loc.col: CharPos (0-based)
// loc.col_display: usize

// Get full location info for a span
let (file, line_lo, col_lo, line_hi, col_hi) = sm.span_to_location_info(span);
// file: Option<Arc<SourceFile>>
// lines and cols: usize (1-based)

// Get source file for a position
let source_file: Arc<SourceFile> = sm.lookup_source_file(byte_pos);

// Get source text for a span
let snippet: Result<String, _> = sm.span_to_snippet(span);

// Get filename
let filename: FileName = sm.span_to_filename(span);

// Convert to displayable string
let loc_str: String = sm.span_to_string(span, FileNameDisplayPreference::Remapped);
let diag_str: String = sm.span_to_diagnostic_string(span);

// Line lookup
let line_result = sm.lookup_line(byte_pos);
// → Ok(SourceFileAndLine { sf, line }) or Err(Arc<SourceFile>)

// Get byte offset within file
let local = sm.lookup_byte_offset(byte_pos);
// → SourceFileAndBytePos { sf, pos: BytePos }

// All source files
let files: MappedReadGuard<MonotonicVec<Arc<SourceFile>>> = sm.files();

// Check multiline
let is_multi = sm.is_multiline(span);

// Get lines info
let lines: FileLinesResult = sm.span_to_lines(span);
```

### 5.4 SourceFile

```rust
pub struct SourceFile {
    pub name: FileName,             // file path
    pub src: Option<Lrc<String>>,   // source text (if available)
    pub start_pos: BytePos,
    pub source_len: RelativeBytePos,
    // ...
}
```

### 5.5 Getting File/Line/Column — Practical Pattern

```rust
fn span_to_location(tcx: TyCtxt, span: Span) -> Option<(String, usize, usize)> {
    let sm = tcx.sess.source_map();
    if span.is_dummy() { return None; }
    let loc = sm.lookup_char_pos(span.lo());
    let filename = loc.file.name.prefer_remapped_unconditionaly().to_string();
    Some((filename, loc.line, loc.col.to_usize() + 1))
}

// Or use span_to_location_info for start+end
let (file_opt, line_start, col_start, line_end, col_end) =
    tcx.sess.source_map().span_to_location_info(def_span);
```

---

## 6. rustc_driver Entry Points

**Crate:** `rustc_driver`  
**Docs:** https://doc.rust-lang.org/nightly/nightly-rustc/rustc_driver/index.html  
**Dev Guide:** https://rustc-dev-guide.rust-lang.org/rustc-driver/intro.html

### 6.1 Callbacks Trait

```rust
// Location: rustc_driver (re-exported from rustc_driver_impl)
pub trait Callbacks {
    /// Called before creating the compiler instance.
    /// Use to configure: file loader, target, crate types, etc.
    fn config(&mut self, _config: &mut Config) {}

    /// Called after parsing the crate root (submodules not yet parsed).
    /// krate: mutable access to AST
    fn after_crate_root_parsing(
        &mut self,
        _compiler: &Compiler,
        _krate: &mut Crate,
    ) -> Compilation { Compilation::Continue }

    /// Called after macro expansion and name resolution.
    /// tcx: full TyCtxt available (no type info yet)
    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &Compiler,
        _tcx: TyCtxt<'tcx>,
    ) -> Compilation { Compilation::Continue }

    /// Called after type checking and borrow checking.
    /// tcx: full TyCtxt with complete type information
    /// *** THIS IS WHERE YOU BUILD YOUR CODE GRAPH ***
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &Compiler,
        _tcx: TyCtxt<'tcx>,
    ) -> Compilation { Compilation::Continue }
}
```

**Compilation enum:**
```rust
pub enum Compilation {
    Continue,   // Continue normal compilation
    Stop,       // Stop after this phase (don't emit artifacts)
}
```

### 6.2 run_compiler — Primary Entry Point

```rust
/// Run the compiler with given args and callbacks.
/// This is the primary entry point for custom rustc drivers.
pub fn run_compiler(at_args: &[String], callbacks: &mut (dyn Callbacks + Send))
```

### 6.3 Config — Compiler Configuration

```rust
// rustc_interface::interface::Config
pub struct Config {
    pub opts: config::Options,
    pub crate_cfg: Vec<String>,
    pub crate_check_cfg: CheckCfg,
    pub input: Input,           // source file or string
    pub output_dir: Option<PathBuf>,
    pub output_file: Option<OutFileName>,
    pub file_loader: Option<Box<dyn FileLoader + Send + Sync>>,
    pub locale_resources: &'static [&'static str],
    pub lint_caps: FxHashMap<lint::LintId, lint::Level>,
    pub parse_sess_created: Option<Box<dyn FnOnce(&mut ParseSess) + Send>>,
    pub hash_untracked_state: Option<Box<dyn FnOnce(&Session, &mut StableHasher) + Send>>,
    pub register_lints: Option<Box<dyn Fn(&Session, &mut LintStore) + Send + Sync>>,
    pub override_queries: Option<fn(&Session, &mut Providers)>,
    pub make_codegen_backend: Option<Box<dyn FnOnce(&config::Options) -> Box<dyn CodegenBackend + Send> + Send>>,
    pub registry: errors::registry::Registry,
    pub ice_file: Option<PathBuf>,
    pub expanded_args: Vec<String>,
}
```

### 6.4 Compiler Struct (rustc_interface)

```rust
// rustc_interface::interface::Compiler
// Provides access to session and source map before TyCtxt is available
impl Compiler {
    pub fn session(&self) -> &Session
    // Session provides: source_map(), diagnostic handler, etc.
}

// Session methods used in Callbacks
compiler.session().source_map()          // &SourceMap
compiler.session().diagnostic()          // DiagCtxt
compiler.session().opts                  // Options
compiler.session().target                // Target spec
compiler.session().features()            // Feature flags
```

### 6.5 Utility Functions

```rust
// Install ICE (Internal Compiler Error) hook
rustc_driver::install_ice_hook(DEFAULT_BUG_REPORT_URL, |_| ());

// Install Ctrl+C handler
rustc_driver::install_ctrlc_handler();

// Catch fatal errors and return exit code
let exit = rustc_driver::catch_with_exit_code(|| run_compiler(&args, &mut cbs));

// Initialize logger
rustc_driver::init_rustc_env_logger(&early_dcx);
```

---

## 7. MIR for Call Graph Analysis

**Location:** `rustc_middle/src/mir/`  
**Docs:** https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/mir/struct.Body.html

### 7.1 MIR Body Structure

```rust
pub struct Body<'tcx> {
    pub basic_blocks: BasicBlocks<'tcx>,     // CFG: Vec<BasicBlockData>
    pub local_decls: IndexVec<Local, LocalDecl<'tcx>>, // variables (local 0 = return)
    pub arg_count: usize,                    // # of function arguments
    pub var_debug_info: Vec<VarDebugInfo<'tcx>>, // debug variable names
    pub source_scopes: IndexVec<SourceScope, SourceScopeData<'tcx>>,
    pub span: Span,                          // overall function span
    pub source: MirSource<'tcx>,             // which DefId/InstanceKind this is
    pub is_polymorphic: bool,                // has uninstatiated type params
    pub coroutine: Option<Box<CoroutineInfo<'tcx>>>,
    // ...
}

pub struct BasicBlockData<'tcx> {
    pub statements: Vec<Statement<'tcx>>,
    pub terminator: Option<Terminator<'tcx>>,
    pub is_cleanup: bool,  // cleanup/unwind block
}
```

### 7.2 TerminatorKind — Call Graph Relevant Variants

```rust
pub enum TerminatorKind<'tcx> {
    /// Normal function call — PRIMARY for call graph edges
    Call {
        func: Operand<'tcx>,                    // callee (can be Constant or Copy(_))
        args: Box<[Spanned<Operand<'tcx>>]>,    // arguments
        destination: Place<'tcx>,               // where return value goes
        target: Option<BasicBlock>,             // successor on normal return
        unwind: UnwindAction,                   // unwind path on panic
        call_source: CallSource,                // HIR call or operator
        fn_span: Span,                          // span of call expression
    },
    /// Tail call (no target/unwind)
    TailCall {
        func: Operand<'tcx>,
        args: Box<[Spanned<Operand<'tcx>>]>,
        fn_span: Span,
    },
    /// Drop glue — implicit drop_in_place call
    Drop {
        place: Place<'tcx>,
        target: BasicBlock,
        unwind: UnwindAction,
        replace: bool,
    },
    Goto { target: BasicBlock },
    SwitchInt { discr: Operand<'tcx>, targets: SwitchTargets },
    Return,
    Unreachable,
    Assert { cond, expected, msg, target, unwind },
    Yield { value, resume, resume_arg, drop },
    // ...
}
```

### 7.3 Extracting Call Graph from MIR

```rust
fn extract_callees<'tcx>(tcx: TyCtxt<'tcx>, def_id: DefId) 
    -> Vec<(DefId, GenericArgsRef<'tcx>)> 
{
    if !tcx.is_mir_available(def_id) { return vec![]; }
    let body: &Body<'tcx> = tcx.optimized_mir(def_id);
    let mut callees = vec![];
    
    for bb_data in body.basic_blocks.iter() {
        if let Some(terminator) = &bb_data.terminator {
            if let TerminatorKind::Call { func, .. } = &terminator.kind {
                // Try to get the callee DefId
                if let Some(Operand::Constant(const_op)) = Some(func) {
                    if let ConstantKind::Val(_, ty) | ty = const_op.const_.ty() {
                        if let TyKind::FnDef(callee_id, args) = ty.kind() {
                            callees.push((*callee_id, args));
                        }
                    }
                }
            }
        }
    }
    callees
}

// Simpler: use the inliner's precomputed callee list
let callees: &[(DefId, GenericArgsRef)] = tcx.mir_inliner_callees(instance_kind);
```

### 7.4 LocalDecl for Type Info

```rust
pub struct LocalDecl<'tcx> {
    pub mutability: Mutability,
    pub local_info: ClearCrossCrate<Box<LocalInfo<'tcx>>>,
    pub ty: Ty<'tcx>,              // TYPE of this local variable
    pub user_ty: Option<...>,
    pub source_info: SourceInfo,   // span + scope
}

// Local 0 = return value
// Locals 1..=arg_count = arguments
// Locals arg_count+1.. = temporaries

let return_ty: Ty<'tcx> = body.local_decls[RETURN_PLACE].ty;
for (i, arg) in body.local_decls.iter().enumerate().take(body.arg_count + 1).skip(1) {
    println!("arg {}: {:?}", i, arg.ty);
}
```

---

## 8. Complete Working Driver Example

**Source:** https://rustc-dev-guide.rust-lang.org/rustc-driver/interacting-with-the-ast.html

```rust
// rust-toolchain.toml: set nightly
// Cargo.toml: [package] rustc-private = true
#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

use rustc_driver::{Compilation, run_compiler};
use rustc_interface::interface::{Compiler, Config};
use rustc_middle::ty::TyCtxt;
use rustc_hir::def::DefKind;

struct CodeGraphCallbacks {
    pub graph: Vec<(String, String)>, // (definer, callee) edges
}

impl rustc_driver::Callbacks for CodeGraphCallbacks {
    fn config(&mut self, config: &mut Config) {
        // Optional: set file loader, disable codegen, etc.
        config.opts.output_types = rustc_session::config::OutputTypes::new(&[]);
    }

    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        // 1. Iterate all local body owners
        for local_def_id in tcx.hir_body_owners() {
            let def_id = local_def_id.to_def_id();
            let kind = tcx.def_kind(def_id);
            
            // 2. Get name
            let name = tcx.def_path_str(def_id);
            
            // 3. Get span
            let span = tcx.def_span(def_id);
            let (file, line, col) = {
                let sm = tcx.sess.source_map();
                let loc = sm.lookup_char_pos(span.lo());
                (
                    loc.file.name.prefer_remapped_unconditionaly().to_string(),
                    loc.line,
                    loc.col.to_usize() + 1,
                )
            };

            // 4. Get type
            if matches!(kind, DefKind::Fn | DefKind::AssocFn) {
                let sig = tcx.fn_sig(def_id).skip_binder().skip_binder();
                
                // 5. Get MIR and extract callees
                if tcx.is_mir_available(def_id) {
                    let body = tcx.optimized_mir(def_id);
                    for bb in body.basic_blocks.iter() {
                        if let Some(term) = &bb.terminator {
                            if let rustc_middle::mir::TerminatorKind::Call { func, .. } = &term.kind {
                                // Extract callee from func operand
                                if let Some(callee_ty) = func.ty(&body.local_decls, tcx).kind().as_fn_def() {
                                    let callee_name = tcx.def_path_str(callee_ty.0);
                                    self.graph.push((name.clone(), callee_name));
                                }
                            }
                        }
                    }
                }

                // 6. HIR for attributes and visibility
                let hir_id = tcx.local_def_id_to_hir_id(local_def_id);
                let attrs = tcx.hir_attrs(hir_id);
                let vis = tcx.visibility(def_id);
            }
        }
        
        Compilation::Stop // Don't emit object code
    }
}

fn main() {
    rustc_driver::install_ice_hook("https://your.repo/issues", |_| ());
    let exit = rustc_driver::catch_with_exit_code(|| {
        let args: Vec<String> = std::env::args().collect();
        run_compiler(&args, &mut CodeGraphCallbacks { graph: vec![] });
    });
    std::process::exit(exit);
}
```

### 8.1 Cargo.toml Setup

```toml
[package]
name = "my-rustc-plugin"
version = "0.1.0"

[dependencies]
# Pin exact nightly versions
rustc_driver = { path = "" }  # handled by rust-toolchain.toml
```

```toml
# rust-toolchain.toml
[toolchain]
channel = "nightly-YYYY-MM-DD"
components = ["rustc-dev", "llvm-tools-preview"]
```

### 8.2 Minimal Cargo.toml Metadata

```toml
[package.metadata.rust-analyzer]
rustc_private = true
```

---

## 9. Graph Data Summary by Query

This table maps each graph construction task to the exact rustc query to use.

| Graph Element | Query | Return Type |
|---------------|-------|-------------|
| All nodes (functions) | `tcx.hir_body_owners()` | `impl Iterator<Item=LocalDefId>` |
| All items (any kind) | `tcx.hir_free_items()` | `impl Iterator<Item=ItemId>` |
| Node name | `tcx.def_path_str(def_id)` | `String` |
| Node kind | `tcx.def_kind(def_id)` | `DefKind` |
| Node span | `tcx.def_span(def_id)` | `Span` |
| Node type | `tcx.type_of(def_id).skip_binder()` | `Ty<'tcx>` |
| Function signature | `tcx.fn_sig(def_id).skip_binder()` | `PolyFnSig<'tcx>` |
| Generic params | `tcx.generics_of(def_id)` | `&Generics` |
| Where-clauses | `tcx.predicates_of(def_id)` | `GenericPredicates` |
| Struct/enum fields | `tcx.adt_def(def_id).all_fields()` | `impl Iterator<FieldDef>` |
| Trait items | `tcx.associated_items(trait_id)` | `&AssocItems` |
| Impl for trait | `tcx.impl_trait_header(impl_id)` | `Option<ImplTraitHeader>` |
| All impls of trait | `tcx.trait_impls_of(trait_id)` | `&TraitImpls` |
| Inherent impls | `tcx.inherent_impls(type_id)` | `&[DefId]` |
| Call edges (MIR) | `tcx.optimized_mir(fn_id)` | `&Body<'tcx>` |
| Call edges (fast) | `tcx.mir_inliner_callees(instance)` | `&[(DefId, GenericArgsRef)]` |
| Visibility | `tcx.visibility(def_id)` | `Visibility<DefId>` |
| Attributes | `tcx.attrs_for_def(def_id)` | `&[Attribute]` |
| Source location | `sm.span_to_location_info(span)` | `(file, line, col, ...)` |
| Source text | `sm.span_to_snippet(span)` | `Result<String, _>` |
| Closure captures | `tcx.upvars_mentioned(def_id)` | `Option<&FxIndexMap<...>>` |
| Type params expressed | `tcx.type_of(def_id)` + `.kind() == Param` | — |
| All crate names | `tcx.crates(())` + `tcx.crate_name(krate)` | `&[CrateNum]` |
| MIR available? | `tcx.is_mir_available(def_id)` | `bool` |
| Vtable layout | `tcx.vtable_entries(trait_ref)` | `&[VtblEntry]` |
| Symbol name (codegen) | `tcx.symbol_name(instance)` | `SymbolName` |
| Stability | `tcx.lookup_stability(def_id)` | `Option<Stability>` |
| Deprecation | `tcx.lookup_deprecation(def_id)` | `Option<Deprecation>` |
| Monomorphized instance | `Instance::try_resolve(tcx, env, id, args)` | `Result<Option<Instance>>` |
| Type check results | `tcx.typeck(local_def_id)` | `&TypeckResults` |
| Expression type | `typeck.node_type(hir_id)` | `Ty<'tcx>` |
| HIR → DefPath | `tcx.hir_def_path(local_def_id)` | `DefPath` |
| Module children | `tcx.module_children(module_id)` | `&[ModChild]` |
| Parent module | `tcx.parent_module_from_def_id(def_id)` | `LocalModDefId` |
| Lang item | `tcx.require_lang_item(LangItem::Add, span)` | `DefId` |
| All traits | `tcx.all_traits_including_private()` | `impl Iterator<DefId>` |

---

## Key Notes for Code Graph Implementation

1. **Use `hir_body_owners()` as the primary iterator** — it gives all functions, closures, and constants that have bodies.

2. **Always check `tcx.is_mir_available(def_id)`** before calling `tcx.optimized_mir(def_id)` — not all items have MIR (e.g., external functions without bodies, items from other crates if not inlineable).

3. **`LocalDefId` vs `DefId`** — HIR queries only work with `LocalDefId`. Use `def_id.as_local()` to check locality before HIR access.

4. **Prefer query-based traversal over `Visitor`** for incremental compilation compatibility. Use `hir_free_items()`, `hir_body_owners()`, and their siblings.

5. **`EarlyBinder`** — many queries return `EarlyBinder<Ty>`. Call `.skip_binder()` to get the type before substituting generics, or `.instantiate(tcx, args)` to substitute.

6. **For call graph edges**: Use `tcx.mir_inliner_callees(InstanceKind::Item(def_id))` for a precomputed list, or iterate `TerminatorKind::Call` in the MIR body for full control.

7. **`Compilation::Stop`** in `after_analysis` prevents the compiler from emitting object files, which speeds up analysis-only tools significantly.

8. **Access `SourceMap` via** `tcx.sess.source_map()` or `compiler.session().source_map()` — it is the single source of file/line/column information.

9. **`typeck` results** contain expression-level type information that `type_of` does not provide. Use `tcx.typeck(local_def_id).node_type(hir_id)` to get the type of any HIR expression or pattern.

10. **Trait impls**: `tcx.all_local_trait_impls(())` returns a `HashMap<TraitDefId, Vec<LocalImplId>>` — use this to build the full trait-impl bipartite graph efficiently.
