# rust-analyzer `ra_ap_*` API Research

> **Crate version at time of research:** `0.0.322` (published ~2026-03-02)  
> **Source:** docs.rs, GitHub rust-lang/rust-analyzer  
> All crates share this version number (ra_ap_hir, ra_ap_hir_def, ra_ap_hir_ty, ra_ap_syntax, ra_ap_ide, ra_ap_base_db, ra_ap_vfs).

---

## Overview: Architectural Layers

```
┌──────────────────────────────────────────────────────────────────┐
│  ra_ap_ide        IDE-level APIs: Analysis, AnalysisHost          │
│  (file positions, text ranges, NavigationTarget, CallItem)        │
├──────────────────────────────────────────────────────────────────┤
│  ra_ap_hir        High-level semantic API                         │
│  (Module, Function, Struct, Enum, Trait, Impl, Type, Semantics)   │
├──────────────────────────────────────────────────────────────────┤
│  ra_ap_hir_ty     Type inference & trait resolution               │
│  (Ty, InferenceResult, TraitEnvironment, TraitImpls)              │
├──────────────────────────────────────────────────────────────────┤
│  ra_ap_hir_def    Definition layer (post-macro, pre-type)         │
│  (ModuleId, FunctionId, StructId, ItemTree, DefMap, Resolver)     │
├──────────────────────────────────────────────────────────────────┤
│  ra_ap_syntax     CST/AST parsing                                 │
│  (SyntaxNode, SyntaxToken, SourceFile, ast::*)                    │
├──────────────────────────────────────────────────────────────────┤
│  ra_ap_base_db    Database traits & file primitives               │
│  (FileId, SourceRoot, CrateGraph, SourceDatabase)                 │
├──────────────────────────────────────────────────────────────────┤
│  ra_ap_vfs        Virtual file system                             │
│  (Vfs, VfsPath, FileId, ChangedFile)                             │
└──────────────────────────────────────────────────────────────────┘
```

The recommended entry point for code-graph construction depends on the use case:
- **Rich semantic graph (compiler-verified):** `ra_ap_hir` via `Semantics<RootDatabase>`
- **Batch analysis / IDE features:** `ra_ap_ide` via `Analysis` / `AnalysisHost`
- **Low-level type queries:** `ra_ap_hir_ty` via `HirDatabase`
- **Syntax-only parsing:** `ra_ap_syntax` via `SourceFile::parse`

---

## 1. `ra_ap_hir` — High-Level Semantic API

> **Docs:** https://docs.rs/ra_ap_hir/latest/ra_ap_hir/  
> **Description:** "HIR provides high-level object-oriented access to Rust code." All types wrap compiler-verified data via the `HirDatabase` query system (salsa).

### 1.1 Core Entry Point: `Semantics<DB>`

```rust
pub struct Semantics<'db, DB>
```

The `Semantics` struct is the **primary API** for going from syntax trees (AST nodes) to semantic information. It bridges `ra_ap_syntax` and `ra_ap_hir`.

**Creation:**
```rust
pub fn new_dyn(db: &dyn HirDatabase) -> Semantics<'_, dyn HirDatabase>
```

**File/Module mapping:**
```rust
pub fn file_to_module_def(&self, file: impl Into<FileId>) -> Option<Module>
pub fn file_to_module_defs(&self, file: impl Into<FileId>) -> impl Iterator<Item = Module>
pub fn hir_file_for(&self, syntax_node: &SyntaxNode) -> HirFileId
pub fn hir_file_to_module_def(&self, file: impl Into<HirFileId>) -> Option<Module>
pub fn hir_file_to_module_defs(&self, file: impl Into<HirFileId>) -> impl Iterator<Item = Module>
```

**AST node → HIR definition conversion:**
```rust
pub fn to_fn_def(&self, f: &Fn) -> Option<Function>
pub fn to_struct_def(&self, s: &Struct) -> Option<Struct>
pub fn to_enum_def(&self, e: &Enum) -> Option<Enum>
pub fn to_trait_def(&self, t: &Trait) -> Option<Trait>
pub fn to_impl_def(&self, i: &Impl) -> Option<Impl>
pub fn to_module_def(&self, m: &Module) -> Option<Module>
pub fn to_adt_def(&self, a: &Adt) -> Option<Adt>
pub fn to_const_def(&self, c: &Const) -> Option<Const>
pub fn to_static_def(&self, s: &Static) -> Option<Static>
pub fn to_macro_def(&self, m: &Macro) -> Option<Macro>
pub fn to_type_alias_def(&self, t: &TypeAlias) -> Option<TypeAlias>
pub fn to_union_def(&self, u: &Union) -> Option<Union>
pub fn to_def<T: ToDef>(&self, src: &T) -> Option<T::Def>
pub fn to_def2<T: ToDef>(&self, src: InFile<&T>) -> Option<T::Def>
```

**Type resolution from syntax:**
```rust
pub fn type_of_expr(&self, expr: &Expr) -> Option<TypeInfo<'db>>
pub fn type_of_pat(&self, pat: &Pat) -> Option<TypeInfo<'db>>
pub fn type_of_binding_in_pat(&self, pat: &IdentPat) -> Option<Type<'db>>
pub fn type_of_self(&self, param: &SelfParam) -> Option<Type<'db>>
pub fn resolve_type(&self, ty: &Type) -> Option<Type<'db>>
```

**Method/field/path resolution:**
```rust
pub fn resolve_method_call(&self, call: &MethodCallExpr) -> Option<Function>
pub fn resolve_method_call_as_callable(&self, call: &MethodCallExpr) -> Option<Callable<'db>>
pub fn resolve_field(&self, field: &FieldExpr) -> Option<Either<Field, TupleField>>
pub fn resolve_record_field(&self, field: &RecordExprField) -> Option<(Field, Option<Local>, Type<'db>)>
pub fn resolve_record_pat_field(&self, field: &RecordPatField) -> Option<(Field, Type<'db>)>
pub fn resolve_path(&self, path: &Path) -> Option<PathResolution>
pub fn resolve_path_per_ns(&self, path: &Path) -> Option<PathResolutionPerNs>
pub fn resolve_trait(&self, path: &Path) -> Option<Trait>
pub fn resolve_macro_call(&self, macro_call: &MacroCall) -> Option<Macro>
pub fn resolve_await_to_poll(&self, await_expr: &AwaitExpr) -> Option<Function>
pub fn resolve_bin_expr(&self, bin_expr: &BinExpr) -> Option<Function>
pub fn resolve_prefix_expr(&self, prefix_expr: &PrefixExpr) -> Option<Function>
pub fn resolve_index_expr(&self, index_expr: &IndexExpr) -> Option<Function>
pub fn resolve_try_expr(&self, try_expr: &TryExpr) -> Option<Function>
pub fn resolve_variant(&self, record_lit: RecordExpr) -> Option<VariantDef>
```

**Scope queries:**
```rust
pub fn scope(&self, node: &SyntaxNode) -> Option<SemanticsScope<'db>>
pub fn scope_at_offset(&self, node: &SyntaxNode, offset: TextSize) -> Option<SemanticsScope<'db>>
```

**Code-graph relevance:**
- `type_of_expr` → typed expression edges (variable → type, call → return type)
- `resolve_method_call` → call edges (caller → callee function)
- `resolve_path` → name resolution edges (path → definition)
- `to_fn_def`, `to_struct_def` → convert syntax nodes to semantic entities

---

### 1.2 `Crate`

```rust
pub struct Crate  // wraps CrateId from base_db
```

**Methods:**
```rust
pub fn all(db: &dyn HirDatabase) -> Vec<Crate>
pub fn root_module(self) -> Module
pub fn modules(self, db: &dyn HirDatabase) -> Vec<Module>
pub fn root_file(self, db: &dyn HirDatabase) -> FileId
pub fn edition(self, db: &dyn HirDatabase) -> Edition
pub fn version(self, db: &dyn HirDatabase) -> Option<String>
pub fn display_name(self, db: &dyn HirDatabase) -> Option<CrateDisplayName>
pub fn dependencies(self, db: &dyn HirDatabase) -> Vec<CrateDependency>
pub fn reverse_dependencies(self, db: &dyn HirDatabase) -> Vec<Crate>
pub fn transitive_reverse_dependencies(self, db: &dyn HirDatabase) -> impl Iterator<Item = Crate>
pub fn origin(self, db: &dyn HirDatabase) -> CrateOrigin
pub fn cfg<'db>(&self, db: &'db dyn HirDatabase) -> &'db CfgOptions
pub fn query_external_importables(
    self, db: &dyn DefDatabase, query: Query
) -> impl Iterator<Item = (Either<ModuleDef, Macro>, Complete)>
```

**Code-graph relevance:**
- `Crate::all(db)` → enumerate all crates in workspace (crate nodes)
- `crate.modules(db)` → enumerate all modules in a crate
- `crate.dependencies(db)` → dependency edges between crates
- `crate.root_module()` → entry point for module traversal

---

### 1.3 `Module`

```rust
pub struct Module  // wraps (CrateId, LocalModuleId)
```

**Methods:**
```rust
pub fn name(self, db: &dyn HirDatabase) -> Option<Name>
pub fn crate_root(self, db: &dyn HirDatabase) -> Module
pub fn is_crate_root(self) -> bool
pub fn parent(self, db: &dyn HirDatabase) -> Option<Module>
pub fn children(self, db: &dyn HirDatabase) -> impl Iterator<Item = Module>
pub fn path_to_root(self, db: &dyn HirDatabase) -> Vec<Module>
pub fn nearest_non_block_module(self, db: &dyn HirDatabase) -> Module
pub fn declarations(self, db: &dyn HirDatabase) -> Vec<ModuleDef>
pub fn impl_defs(self, db: &dyn HirDatabase) -> Vec<Impl>
pub fn legacy_macros(self, db: &dyn HirDatabase) -> Vec<Macro>
pub fn scope(self, db: &dyn HirDatabase, visible_from: Option<Module>) -> Vec<(Name, ScopeDef)>
pub fn resolve_mod_path(
    &self, db: &dyn HirDatabase, segments: impl IntoIterator<Item = Name>
) -> Option<impl Iterator<Item = ItemInNs>>
pub fn definition_source(self, db: &dyn HirDatabase) -> InFile<ModuleSource>
pub fn definition_source_file_id(self, db: &dyn HirDatabase) -> HirFileId
pub fn as_source_file_id(self, db: &dyn HirDatabase) -> Option<EditionedFileId>
pub fn is_inline(self, db: &dyn HirDatabase) -> bool
pub fn is_mod_rs(self, db: &dyn HirDatabase) -> bool
pub fn find_path(
    self, db: &dyn DefDatabase, item: impl Into<ItemInNs>, cfg: ImportPathConfig
) -> Option<ModPath>
pub fn find_use_path(
    self, db: &dyn DefDatabase, item: impl Into<ItemInNs>,
    prefix_kind: PrefixKind, cfg: ImportPathConfig
) -> Option<ModPath>
```

**Code-graph relevance:**
- `declarations(db)` → all items (functions, structs, enums, traits, type aliases, consts, statics, macros) defined in module
- `impl_defs(db)` → all `impl` blocks in module
- `children(db)` → module hierarchy (parent → child edges)
- `scope(db, ...)` → all visible names including imports

---

### 1.4 `ModuleDef` (enum)

Represents any item that can appear in a module scope:

```rust
pub enum ModuleDef {
    Module(Module),
    Function(Function),
    Adt(Adt),           // Struct | Enum | Union
    Variant(Variant),
    Const(Const),
    Static(Static),
    Trait(Trait),
    TraitAlias(TraitAlias),
    TypeAlias(TypeAlias),
    BuiltinType(BuiltinType),
    Macro(Macro),
}
```

**Code-graph relevance:** Returned by `Module::declarations()` — use `match` to dispatch to each entity type.

---

### 1.5 `Function`

```rust
pub struct Function  // wraps FunctionId
```

**Methods:**
```rust
pub fn name(self, db: &dyn HirDatabase) -> Name
pub fn module(self, db: &dyn HirDatabase) -> Module
pub fn ty(self, db: &dyn HirDatabase) -> Type<'_>
pub fn fn_ptr_type(self, db: &dyn HirDatabase) -> Type<'_>
pub fn ret_type(self, db: &dyn HirDatabase) -> Type<'_>
pub fn ret_type_with_args<'db>(
    self, db: &'db dyn HirDatabase,
    generics: impl Iterator<Item = Type<'db>>
) -> Type<'db>
pub fn async_ret_type<'db>(self, db: &'db dyn HirDatabase) -> Option<Type<'db>>
pub fn has_self_param(self, db: &dyn HirDatabase) -> bool
pub fn self_param(self, db: &dyn HirDatabase) -> Option<SelfParam>
pub fn assoc_fn_params(self, db: &dyn HirDatabase) -> Vec<Param<'_>>
pub fn params_without_self(self, db: &dyn HirDatabase) -> Vec<Param<'_>>
pub fn num_params(self, db: &dyn HirDatabase) -> usize
pub fn method_params(self, db: &dyn HirDatabase) -> Option<Vec<Param<'_>>>
pub fn is_async(self, db: &dyn HirDatabase) -> bool
pub fn is_const(self, db: &dyn HirDatabase) -> bool
pub fn is_varargs(self, db: &dyn HirDatabase) -> bool
pub fn is_unsafe_to_call(
    self, db: &dyn HirDatabase,
    caller: Option<Function>, call_edition: Edition
) -> bool
pub fn has_body(self, db: &dyn HirDatabase) -> bool
pub fn is_test(self, db: &dyn HirDatabase) -> bool
pub fn is_main(self, db: &dyn HirDatabase) -> bool
pub fn is_bench(self, db: &dyn HirDatabase) -> bool
pub fn is_ignore(self, db: &dyn HirDatabase) -> bool
pub fn extern_block(self, db: &dyn HirDatabase) -> Option<ExternBlock>
pub fn returns_impl_future(self, db: &dyn HirDatabase) -> bool
pub fn as_proc_macro(self, db: &dyn HirDatabase) -> Option<Macro>
```

Via `HasVisibility` trait:
```rust
pub fn visibility(&self, db: &dyn HirDatabase) -> Visibility
pub fn is_visible_from(&self, db: &dyn HirDatabase, module: Module) -> bool
```

Via `HasAttrs` trait:
```rust
pub fn attrs(self, db: &dyn HirDatabase) -> AttrsWithOwner
// doc comments via: attrs.doc_exprs() or attrs.by_key(sym::doc)
```

Via `HirDisplay` trait:
```rust
pub fn display<'a>(&'a self, db: &'db dyn HirDatabase, display_target: DisplayTarget) 
    -> HirDisplayWrapper<'_, 'db, Self>
// format!("{}", fn.display(db, target)) → "fn foo(x: i32) -> bool"
```

**`Param` struct:**
```rust
pub fn parent_fn(&self) -> Option<Function>
pub fn index(&self) -> usize
pub fn ty(&self) -> &Type<'db>
pub fn name(&self, db: &dyn HirDatabase) -> Option<Name>
pub fn as_local(&self, db: &dyn HirDatabase) -> Option<Local>
pub fn pattern_source(self, db: &dyn HirDatabase) -> Option<Pat>
```

**Code-graph relevance:**
- `ret_type(db)` + `params_without_self(db)` → full function signature as typed edges
- `visibility(db)` → `Visibility` (pub/pub(crate)/pub(super)/module-private)
- `attrs(db).doc_exprs()` → doc comments for metadata nodes
- `has_self_param(db)` → distinguish methods from free functions
- `is_async`, `is_const`, `is_unsafe_to_call` → function metadata flags

---

### 1.6 `Struct`

```rust
pub fn name(self, db: &dyn HirDatabase) -> Name
pub fn module(self, db: &dyn HirDatabase) -> Module
pub fn fields(self, db: &dyn HirDatabase) -> Vec<Field>
pub fn ty(self, db: &dyn HirDatabase) -> Type<'_>
pub fn ty_params(self, db: &dyn HirDatabase) -> Type<'_>
pub fn constructor_ty(self, db: &dyn HirDatabase) -> Type<'_>
pub fn kind(self, db: &dyn HirDatabase) -> StructKind   // Unit | Tuple | Record
pub fn repr(self, db: &dyn HirDatabase) -> Option<ReprOptions>
pub fn is_unstable(self, db: &dyn HirDatabase) -> bool
// + HasVisibility, HasAttrs, HirDisplay
```

**`Field` struct:**
```rust
pub fn name(&self, db: &dyn HirDatabase) -> Name
pub fn index(&self) -> usize
pub fn ty<'db>(&self, db: &'db dyn HirDatabase) -> TypeNs<'db>
pub fn ty_with_args<'db>(
    &self, db: &'db dyn HirDatabase, generics: impl Iterator<Item = Type<'db>>
) -> Type<'db>
pub fn parent_def(&self, _db: &dyn HirDatabase) -> VariantDef
pub fn layout(&self, db: &dyn HirDatabase) -> Result<Layout, LayoutError>
// + HasVisibility (field-level pub/private)
```

---

### 1.7 `Enum`

```rust
pub fn name(self, db: &dyn HirDatabase) -> Name
pub fn module(self, db: &dyn HirDatabase) -> Module
pub fn variants(self, db: &dyn HirDatabase) -> Vec<Variant>
pub fn num_variants(self, db: &dyn HirDatabase) -> usize
pub fn ty<'db>(self, db: &'db dyn HirDatabase) -> Type<'db>
pub fn variant_body_ty<'db>(self, db: &'db dyn HirDatabase) -> Type<'db>
pub fn is_data_carrying(self, db: &dyn HirDatabase) -> bool
pub fn repr(self, db: &dyn HirDatabase) -> Option<ReprOptions>
pub fn layout(self, db: &dyn HirDatabase) -> Result<Layout, LayoutError>
pub fn is_unstable(self, db: &dyn HirDatabase) -> bool
// + HasVisibility, HasAttrs, HirDisplay
```

**`Variant` struct:**
```rust
pub fn name(self, db: &dyn HirDatabase) -> Name
pub fn module(self, db: &dyn HirDatabase) -> Module
pub fn parent_enum(self, db: &dyn HirDatabase) -> Enum
pub fn fields(self, db: &dyn HirDatabase) -> Vec<Field>
pub fn kind(self, db: &dyn HirDatabase) -> StructKind
pub fn value(self, db: &dyn HirDatabase) -> Option<Expr>          // discriminant expr
pub fn eval(self, db: &dyn HirDatabase) -> Result<i128, ConstEvalError<'_>>
pub fn constructor_ty(self, db: &dyn HirDatabase) -> Type<'_>
pub fn layout(&self, db: &dyn HirDatabase) -> Result<Layout, LayoutError>
pub fn is_unstable(self, db: &dyn HirDatabase) -> bool
```

---

### 1.8 `Trait`

```rust
pub fn lang(db: &dyn HirDatabase, krate: Crate, name: &Name) -> Option<Trait>
pub fn name(self, db: &dyn HirDatabase) -> Name
pub fn module(self, db: &dyn HirDatabase) -> Module
pub fn items(self, db: &dyn HirDatabase) -> Vec<AssocItem>
pub fn items_with_supertraits(self, db: &dyn HirDatabase) -> Vec<AssocItem>
pub fn function(self, db: &dyn HirDatabase, name: impl PartialEq<Name>) -> Option<Function>
pub fn direct_supertraits(self, db: &dyn HirDatabase) -> Vec<Trait>
pub fn all_supertraits(self, db: &dyn HirDatabase) -> Vec<Trait>
pub fn is_auto(self, db: &dyn HirDatabase) -> bool
pub fn is_unsafe(&self, db: &dyn HirDatabase) -> bool
pub fn type_or_const_param_count(&self, db: &dyn HirDatabase, count_required_only: bool) -> usize
pub fn dyn_compatibility(&self, db: &dyn HirDatabase) -> Option<DynCompatibilityViolation>
pub fn complete(self, db: &dyn HirDatabase) -> Complete
// + HasVisibility, HasAttrs, HirDisplay
```

**`AssocItem` enum:**
```rust
pub enum AssocItem {
    Function(Function),
    Const(Const),
    TypeAlias(TypeAlias),
}
```

**Code-graph relevance:**
- `direct_supertraits(db)` / `all_supertraits(db)` → supertrait edges
- `items(db)` → all associated items (functions, const, type aliases) as child edges
- `Impl::all_for_trait(db, trait_)` → find all implementors

---

### 1.9 `Impl`

```rust
pub fn all_in_crate(db: &dyn HirDatabase, krate: Crate) -> Vec<Impl>
pub fn all_in_module(db: &dyn HirDatabase, module: Module) -> Vec<Impl>
pub fn all_for_type<'db>(db: &'db dyn HirDatabase, _: Type<'db>) -> Vec<Impl>
pub fn all_for_trait(db: &dyn HirDatabase, trait_: Trait) -> Vec<Impl>
pub fn trait_(self, db: &dyn HirDatabase) -> Option<Trait>     // None = inherent impl
pub fn trait_ref(self, db: &dyn HirDatabase) -> Option<TraitRef<'_>>
pub fn self_ty(self, db: &dyn HirDatabase) -> Type<'_>
pub fn items(self, db: &dyn HirDatabase) -> Vec<AssocItem>
pub fn is_negative(self, db: &dyn HirDatabase) -> bool
pub fn is_unsafe(self, db: &dyn HirDatabase) -> bool
pub fn module(self, db: &dyn HirDatabase) -> Module
pub fn as_builtin_derive_path(self, db: &dyn HirDatabase) -> Option<InMacroFile<Path>>
pub fn check_orphan_rules(self, db: &dyn HirDatabase) -> bool
```

**Code-graph relevance:**
- `Impl::all_for_trait(db, t)` → find all types that implement trait `t` (implementation edges)
- `Impl::all_for_type(db, ty)` → find all impls on a type (inherent + trait)
- `trait_(db)` → if `Some(t)`, this is a trait impl edge (Type → Trait)
- `self_ty(db)` → the implementing type node
- `items(db)` → concrete implementations of the trait's methods

---

### 1.10 `Type`

The `Type` struct represents a resolved HIR type with full generic substitution.

**Type inspection:**
```rust
pub fn is_unit(&self) -> bool
pub fn is_bool(&self) -> bool
pub fn is_str(&self) -> bool
pub fn is_never(&self) -> bool
pub fn is_reference(&self) -> bool
pub fn is_mutable_reference(&self) -> bool
pub fn as_reference(&self) -> Option<(Type<'db>, Mutability)>
pub fn is_slice(&self) -> bool
pub fn is_tuple(&self) -> bool
pub fn is_array(&self) -> bool
pub fn is_closure(&self) -> bool
pub fn as_closure(&self) -> Option<Closure<'db>>
pub fn is_fn(&self) -> bool
pub fn is_raw_ptr(&self) -> bool
pub fn is_unknown(&self) -> bool
pub fn is_copy(&self, db: &'db dyn HirDatabase) -> bool
pub fn as_adt(&self) -> Option<Adt>                           // → Struct | Enum | Union
pub fn as_builtin(&self) -> Option<BuiltinType>
pub fn as_dyn_trait(&self) -> Option<Trait>
pub fn as_type_param(&self, _db: &'db dyn HirDatabase) -> Option<TypeParam>
pub fn as_callable(&self, db: &'db dyn HirDatabase) -> Option<Callable<'db>>
pub fn as_impl_traits(&self, db: &'db dyn HirDatabase) -> Option<impl Iterator<Item = Trait>>
pub fn strip_references(&self) -> Self
pub fn type_arguments(&self) -> impl Iterator<Item = Type<'db>> + '_   // generic args
```

**Struct/tuple fields:**
```rust
pub fn fields(&self, db: &'db dyn HirDatabase) -> Vec<(Field, Self)>
pub fn tuple_fields(&self, _db: &'db dyn HirDatabase) -> Vec<Self>
pub fn as_array(&self, db: &'db dyn HirDatabase) -> Option<(Self, usize)>
```

**Trait queries:**
```rust
pub fn impls_trait(&self, db: &'db dyn HirDatabase, trait_: Trait, args: &[Type<'db>]) -> bool
pub fn impls_iterator(self, db: &'db dyn HirDatabase) -> bool
pub fn impls_fnonce(&self, db: &'db dyn HirDatabase) -> bool
pub fn applicable_inherent_traits(&self, db: &'db dyn HirDatabase) -> impl Iterator<Item = Trait>
pub fn env_traits(&self, db: &'db dyn HirDatabase) -> impl Iterator<Item = Trait>
pub fn as_associated_type_parent_trait(&self, db: &'db dyn HirDatabase) -> Option<Trait>
```

**Type unification/coercion:**
```rust
pub fn could_unify_with(&self, db: &'db dyn HirDatabase, other: &Type<'db>) -> bool
pub fn could_coerce_to(&self, db: &'db dyn HirDatabase, to: &Type<'db>) -> bool
```

**Code-graph relevance:**
- `as_adt()` → resolve type to struct/enum/union entity
- `type_arguments()` → generic instantiation edges
- `impls_trait(db, t, args)` → verify trait satisfaction (compiler-verified)
- `fields(db)` → struct field type edges

---

### 1.11 `Callable` and `CallableKind`

```rust
pub fn kind(&self) -> CallableKind<'db>    // Function(Function) | Closure | FnPtr | Other
pub fn params(&self) -> Vec<Param<'db>>
pub fn return_type(&self) -> Type<'db>
pub fn receiver_param(&self, db: &'db dyn HirDatabase) -> Option<(SelfParam, Type<'db>)>
pub fn n_params(&self) -> usize
pub fn ty(&self) -> &Type<'db>
```

---

### 1.12 Key Traits

| Trait | Key Methods | Implementors |
|-------|-------------|--------------|
| `HasAttrs` | `attrs(db) -> AttrsWithOwner` | Function, Struct, Enum, Trait, Impl, Variant, Field, Module, ... |
| `HasVisibility` | `visibility(db) -> Visibility`, `is_visible_from(db, module) -> bool` | Function, Struct, Enum, Trait, Field, Const, Static, TypeAlias |
| `HasSource` | `source(db) -> InFile<T::Ast>` | Most HIR items — returns the source syntax node |
| `HirDisplay` | `display(db, target)`, `display_truncated(db, max, target)`, `display_source_code(db, mod, allow_opaque)` | Function, Type, Struct, Trait, ... |
| `HasCrate` | `krate(db) -> Crate` | All HIR items |

**`Visibility` enum values:**
```rust
Public          // pub
PubCrate        // pub(crate)
Module(Module)  // pub(super) / pub(in path)
Private         // private (no qualifier)
```

**`AttrsWithOwner` — doc comments and attributes:**
```rust
attrs.by_key(sym::doc)          // doc comment strings
attrs.doc_exprs()               // Iterator<Item=DocExpr>
attrs.cfg()                     // Option<CfgExpr> - cfg condition
attrs.cfgs()                    // Iterator<Item=CfgExpr>
attrs.is_test() / is_bench()    // test/bench markers
attrs.repr()                    // #[repr(...)] options
attrs.derive_macro_invocs(...)  // (via ItemScope)
```

---

### 1.13 `GenericDef` and Generic Parameters

```rust
pub enum GenericDef {
    Function(Function),
    Adt(Adt),
    Trait(Trait),
    TypeAlias(TypeAlias),
    Impl(Impl),
    ...
}

// Methods:
pub fn params(self, db: &dyn HirDatabase) -> Vec<GenericParam>
pub fn lifetime_params(self, db: &dyn HirDatabase) -> Vec<LifetimeParam>
pub fn type_or_const_params(self, db: &dyn HirDatabase) -> Vec<TypeOrConstParam>
```

---

### 1.14 `SemanticsScope`

Encapsulates the set of visible names at a particular program point.

```rust
pub fn module(&self) -> Module
pub fn krate(&self) -> Crate
pub fn containing_function(&self) -> Option<Function>
pub fn process_all_names(&self, f: &mut dyn FnMut(Name, ScopeDef))
pub fn visible_traits(&self) -> VisibleTraits
pub fn can_use_trait_methods(&self, t: Trait) -> bool
pub fn speculative_resolve(&self, ast_path: &Path) -> Option<PathResolution>
pub fn resolve_mod_path(&self, path: &ModPath) -> impl Iterator<Item = ItemInNs>
pub fn extern_crates(&self) -> impl Iterator<Item = (Name, Module)> + '_
pub fn generic_def(&self) -> Option<GenericDef>
pub fn is_visible(&self, db: &dyn DefDatabase, visibility: Visibility) -> bool
```

---

## 2. `ra_ap_hir_def` — Definition Layer

> **Docs:** https://docs.rs/ra_ap_hir_def/latest/ra_ap_hir_def/  
> **Description:** "Everything between macro expansion and type inference." Defines the canonical IDs for all items.

### 2.1 Core ID Types

These are the stable identifiers used throughout the system. They are all `Copy + Eq + Hash`.

| ID Type | What It Identifies |
|---------|--------------------|
| `FunctionId` | A function definition |
| `StructId` | A struct definition |
| `EnumId` | An enum definition |
| `EnumVariantId` | A single enum variant |
| `TraitId` | A trait definition |
| `ImplId` | An impl block |
| `TypeAliasId` | A type alias |
| `ConstId` | A const definition |
| `StaticId` | A static definition |
| `FieldId` | A struct/enum field |
| `TypeParamId` | A type parameter |
| `LifetimeParamId` | A lifetime parameter |
| `BlockId` | An anonymous block item scope |
| `Macro2Id`, `MacroRulesId`, `ProcMacroId` | Macro definitions |
| `ExternBlockId`, `ExternCrateId` | extern items |

**Key enum IDs:**
```rust
pub enum DefWithBodyId {
    FunctionId(FunctionId),
    StaticId(StaticId),
    ConstId(ConstId),
    ...
}

pub enum ModuleDefId {
    ModuleId(ModuleId),
    FunctionId(FunctionId),
    AdtId(AdtId),        // StructId | EnumId | UnionId
    EnumVariantId(EnumVariantId),
    ConstId(ConstId),
    StaticId(StaticId),
    TraitId(TraitId),
    TraitAliasId(TraitAliasId),
    TypeAliasId(TypeAliasId),
    BuiltinType(BuiltinType),
    MacroId(MacroId),
}

pub enum AdtId {
    StructId(StructId),
    UnionId(UnionId),
    EnumId(EnumId),
}

pub enum GenericDefId {
    FunctionId(FunctionId),
    AdtId(AdtId),
    TraitId(TraitId),
    TraitAliasId(TraitAliasId),
    TypeAliasId(TypeAliasId),
    ImplId(ImplId),
    ...
}
```

---

### 2.2 `ItemTree`

The `ItemTree` is a simplified, macro-expanded AST containing only items (no expressions). One `ItemTree` per `HirFileId`.

```rust
pub struct ItemTree  // in ra_ap_hir_def::item_tree

// Key item types in ItemTree:
pub struct Function    // function item in the tree
pub struct Struct      // struct item
pub struct Enum        // enum item
pub struct Trait       // trait item
pub struct Impl        // impl block
pub struct TypeAlias   // type alias
pub struct Const       // const
pub struct Static      // static
pub struct Mod         // module
pub struct Use         // use declaration
pub struct ExternCrate // extern crate
pub struct ExternBlock // extern { } block

// Each item carries:
// - AstId<T>  → back-reference to syntax
// - RawVisibility  → unresolved visibility
// - Name
// - generic params (as indices)
```

**Access via `DefDatabase`:**
```rust
fn file_item_tree(&self, file_id: HirFileId) -> &ItemTree
```

---

### 2.3 `DefMap` — Module-Level Name Resolution

```rust
pub struct DefMap  // in ra_ap_hir_def::nameres
```

Contains the results of (early) name resolution for a crate.

**Methods:**
```rust
// Module enumeration
pub fn modules(&self) -> impl Iterator<Item = (ModuleId, &ModuleData)> + '_
pub fn modules_for_file<'a>(
    &'a self, db: &'a dyn DefDatabase, file_id: FileId
) -> impl Iterator<Item = ModuleId> + 'a
pub fn root_module_id(&self) -> ModuleId
pub fn crate_root(&self, db: &dyn DefDatabase) -> ModuleId
pub fn parent(&self) -> Option<ModuleId>
pub fn containing_module(&self, local_mod: ModuleId) -> Option<ModuleId>

// Feature flags
pub fn is_no_std(&self) -> bool
pub fn is_no_core(&self) -> bool
pub fn is_unstable_feature_enabled(&self, feature: &Symbol) -> bool

// Macros
pub fn fn_as_proc_macro(&self, id: FunctionId) -> Option<ProcMacroId>
pub fn proc_macro_as_fn(&self, id: ProcMacroId) -> Option<FunctionId>

// Debug
pub fn dump(&self, db: &dyn DefDatabase) -> String
pub fn krate(&self) -> Crate
```

**`ModuleData`** (from `modules()` iterator) contains the module's `ItemScope`.

---

### 2.4 `ItemScope` — Per-Module Name Registry

```rust
pub struct ItemScope  // in ra_ap_hir_def::item_scope
```

**Methods:**
```rust
pub fn entries(&self) -> impl Iterator<Item = (&Name, PerNs)> + '_
pub fn declarations(&self) -> impl Iterator<Item = ModuleDefId> + '_
pub fn types(&self) -> impl Iterator<Item = (&Name, Item<ModuleDefId, ImportOrExternCrate>)> + '_
pub fn values(&self) -> impl Iterator<Item = (&Name, Item<ModuleDefId, ImportOrGlob>)> + '_
pub fn macros(&self) -> impl Iterator<Item = (&Name, Item<MacroId, ImportOrExternCrate>)> + '_
pub fn impls(&self) -> impl ExactSizeIterator<Item = ImplId> + '_
pub fn trait_impls(&self) -> impl Iterator<Item = ImplId> + '_
pub fn inherent_impls(&self) -> impl Iterator<Item = ImplId> + '_
pub fn imports(&self) -> impl Iterator<Item = ImportId> + '_
pub fn extern_crate_decls(&self) -> impl ExactSizeIterator<Item = ExternCrateId> + '_
pub fn get(&self, name: &Name) -> PerNs
pub fn all_macro_calls(&self) -> impl Iterator<Item = MacroCallId> + '_
pub fn legacy_macros(&self) -> impl Iterator<Item = (&Name, &[MacroId])> + '_
pub fn derive_macro_invocs(&self) -> impl Iterator<Item = ...> + '_
pub fn attr_macro_invocs(&self) -> impl Iterator<Item = (AstId<Item>, MacroCallId)> + '_
```

---

### 2.5 `Resolver` — Name Resolution Façade

```rust
pub struct Resolver  // in ra_ap_hir_def::resolver
```

**Key methods:**
```rust
pub fn resolve_path_in_type_ns(
    &self, db: &dyn DefDatabase, path: &Path
) -> Option<(TypeNs, Option<usize>, Option<ImportOrExternCrate>)>
pub fn resolve_path_in_type_ns_fully(
    &self, db: &dyn DefDatabase, path: &Path
) -> Option<TypeNs>
pub fn resolve_path_in_value_ns(
    &self, db: &dyn DefDatabase, path: &Path, hygiene_id: HygieneId
) -> Option<ResolveValueResult>
pub fn resolve_path_in_value_ns_fully(
    &self, db: &dyn DefDatabase, path: &Path, hygiene: HygieneId
) -> Option<ValueNs>
pub fn resolve_known_trait(&self, db: &dyn DefDatabase, path: &ModPath) -> Option<TraitId>
pub fn resolve_known_struct(&self, db: &dyn DefDatabase, path: &ModPath) -> Option<StructId>
pub fn resolve_known_enum(&self, db: &dyn DefDatabase, path: &ModPath) -> Option<EnumId>
pub fn resolve_visibility(
    &self, db: &dyn DefDatabase, visibility: &RawVisibility
) -> Option<Visibility>
pub fn names_in_scope(
    &self, db: &dyn DefDatabase
) -> IndexMap<Name, SmallVec<[ScopeDef; 1]>, FxBuildHasher>
pub fn traits_in_scope(&self, db: &dyn DefDatabase) -> FxHashSet<TraitId>
pub fn module(&self) -> ModuleId
pub fn def_map(&self) -> &DefMap
pub fn generic_params(&self) -> Option<&GenericParams>
pub fn body_owner(&self) -> Option<DefWithBodyId>
pub fn impl_def(&self) -> Option<ImplId>
pub fn is_visible(&self, db: &dyn DefDatabase, visibility: Visibility) -> bool
```

---

### 2.6 `DefDatabase` Queries

The `DefDatabase` trait (underlying query system via salsa) exposes:

```rust
fn file_item_tree(&self, file_id: HirFileId) -> &ItemTree
fn macro_def(&self, m: MacroId) -> MacroDefId

// Signatures (lightweight, no body)
fn function_signature(&self, e: FunctionId) -> Arc<FunctionSignature>
fn struct_signature(&self, struct_: StructId) -> Arc<StructSignature>
fn enum_signature(&self, e: EnumId) -> Arc<EnumSignature>
fn trait_signature(&self, trait_: TraitId) -> Arc<TraitSignature>
fn impl_signature(&self, impl_: ImplId) -> Arc<ImplSignature>
fn type_alias_signature(&self, e: TypeAliasId) -> Arc<TypeAliasSignature>
fn const_signature(&self, e: ConstId) -> Arc<ConstSignature>
fn static_signature(&self, e: StaticId) -> Arc<StaticSignature>

// Body and scopes
fn body(&self, def: DefWithBodyId) -> Arc<Body>
fn body_with_source_map(&self, def: DefWithBodyId) -> (Arc<Body>, Arc<BodySourceMap>)
fn expr_scopes(&self, def: DefWithBodyId) -> Arc<ExprScopes>

// Generics
fn generic_params(&self, def: GenericDefId) -> Arc<GenericParams>

// Visibility
fn field_visibilities(&self, var: VariantId) -> Arc<ArenaMap<LocalFieldId, Visibility>>
fn assoc_visibility(&self, def: AssocItemId) -> Visibility

// Import map for external queries
fn import_map(&self, krate: Crate) -> Arc<ImportMap>

// Crate features
fn crate_notable_traits(&self, krate: Crate) -> Option<&[TraitId]>
fn crate_supports_no_std(&self, crate_id: Crate) -> bool
```

---

## 3. `ra_ap_hir_ty` — Type Inference and Trait Resolution

> **Docs:** https://docs.rs/ra_ap_hir_ty/latest/ra_ap_hir_ty/  
> **Description:** "The type system. Used for completion, hover, and assists."

### 3.1 Core Type: `Ty`

```rust
pub type Ty = Ty<Interner>        // from Chalk (ra_ap_hir_ty re-exports)
pub type TyKind = TyKind<Interner>
pub type Substitution = Substitution<Interner>
pub type TraitRef = TraitRef<Interner>
pub type AliasTy = AliasTy<Interner>
```

`TyKind` variants (key ones for code-graph):
```
Adt(AdtId, Substitution)           → struct/enum/union with generics
FnDef(FunctionId, Substitution)    → named function  
Closure(ClosureId, Substitution)   → closure
FnPtr(...)                         → fn pointer
Tuple(len, Substitution)           → tuple
Scalar(Scalar)                     → int/float/bool/char
Array(Ty, Const)                   → [T; N]
Slice(Ty)                          → [T]
Ref(Lifetime, Ty, Mutability)      → &T / &mut T
Raw(Mutability, Ty)                → *const T / *mut T
Dyn(DynTy)                         → dyn Trait
OpaqueType(OpaqueTyId, Subst)      → impl Trait
Alias(AliasTy)                     → type alias projection
Placeholder(TypeVarId)             → unresolved type parameter
BoundVar(BoundVar)                 → bound variable in binder
InferenceVar(InferenceVar, Kind)   → inference variable
Never                              → !
Error                              → type error
```

### 3.2 `TyExt` Trait (extension methods on `Ty`)

```rust
fn is_unit(&self) -> bool
fn is_integral(&self) -> bool
fn is_scalar(&self) -> bool
fn is_floating_point(&self) -> bool
fn is_never(&self) -> bool
fn is_unknown(&self) -> bool
fn contains_unknown(&self) -> bool
fn is_union(&self) -> bool
fn as_adt(&self) -> Option<(AdtId, &Substitution)>
fn as_builtin(&self) -> Option<BuiltinType>
fn as_tuple(&self) -> Option<&Substitution>
fn as_closure(&self) -> Option<ClosureId>
fn as_fn_def(&self, db: &dyn HirDatabase) -> Option<FunctionId>
fn as_reference(&self) -> Option<(&Ty, Lifetime, Mutability)>
fn as_raw_ptr(&self) -> Option<(&Ty, Mutability)>
fn as_generic_def(&self, db: &dyn HirDatabase) -> Option<GenericDefId>
fn callable_def(&self, db: &dyn HirDatabase) -> Option<CallableDefId>
fn callable_sig(&self, db: &dyn HirDatabase) -> Option<CallableSig>
fn strip_references(&self) -> &Ty
fn dyn_trait(&self) -> Option<TraitId>
fn impl_trait_bounds(&self, db: &dyn HirDatabase) -> Option<Vec<QuantifiedWhereClause>>
fn associated_type_parent_trait(&self, db: &dyn HirDatabase) -> Option<TraitId>
fn equals_ctor(&self, other: &Ty) -> bool
```

---

### 3.3 `InferenceResult`

The output of `HirDatabase::infer(def: DefWithBodyId)` — a map from expressions/patterns to their inferred types.

```rust
pub struct InferenceResult

// Type lookups
pub fn type_of_expr_or_pat(&self, id: ExprOrPatId) -> Option<&Ty>
pub fn type_of_expr_with_adjust(&self, id: ExprId) -> Option<&Ty>   // post-adjustment
pub fn type_of_pat_with_adjust(&self, id: PatId) -> Option<&Ty>

// Type mismatches (type errors)
pub fn type_mismatch_for_expr(&self, expr: ExprId) -> Option<&TypeMismatch>
pub fn type_mismatches(&self) -> impl Iterator<Item = (ExprOrPatId, &TypeMismatch)>

// Resolution (at lower level than Semantics)
pub fn method_resolution(&self, expr: ExprId) -> Option<(FunctionId, Substitution)>
pub fn field_resolution(&self, expr: ExprId) -> Option<Either<FieldId, TupleFieldId>>
pub fn variant_resolution_for_expr(&self, id: ExprId) -> Option<VariantId>
pub fn assoc_resolutions_for_expr(&self, id: ExprId) -> Option<(AssocItemId, Substitution)>

// Adjustment info
pub fn expr_adjustment(&self, id: ExprId) -> Option<&[Adjustment]>
pub fn pat_adjustment(&self, id: PatId) -> Option<&[Ty]>

// Closure captures
pub fn closure_info(&self, closure: &ClosureId) -> &(Vec<CapturedItem>, FnTrait)

pub fn is_erroneous(&self) -> bool
pub fn diagnostics(&self) -> &[InferenceDiagnostic]
```

**Access pattern:**
```rust
let infer: Arc<InferenceResult> = db.infer(def_with_body_id);
// Then map ExprId → Ty for each expression in the body
```

---

### 3.4 `TraitEnvironment`

The set of where-clauses assumed to be true at a given point:

```rust
pub fn empty(krate: Crate) -> Arc<Self>
pub fn new(
    krate: Crate,
    block: Option<BlockId>,
    traits_from_clauses: Box<[(Ty, TraitId)]>,
    env: Environment<Interner>
) -> Arc<Self>
pub fn traits_in_scope_from_clauses(&self, ty: Ty) -> impl Iterator<Item = TraitId> + '_
```

---

### 3.5 `TraitImpls` — Finding All Implementations

```rust
pub struct TraitImpls  // in ra_ap_hir_ty::method_resolution

// Static constructors
pub fn for_crate(db: &dyn HirDatabase, krate: Crate) -> &Arc<TraitImpls>
pub fn for_crate_and_deps(db: &dyn HirDatabase, krate: Crate) -> &Box<[Arc<TraitImpls>]>

// Finding impls
pub fn blanket_impls(&self, for_trait: TraitId) -> &[ImplId]
pub fn for_trait(&self, trait_: TraitId, callback: impl FnMut(Either<&[ImplId], &[BuiltinDeriveImplId]>))
pub fn for_trait_and_self_ty(
    &self, trait_: TraitId, self_ty: &SimplifiedType
) -> (&[ImplId], &[BuiltinDeriveImplId])
pub fn has_impls_for_trait_and_self_ty(&self, trait_: TraitId, self_ty: &SimplifiedType) -> bool
```

---

### 3.6 `InherentImpls` — Inherent (non-trait) Impls

```rust
pub struct InherentImpls  // in ra_ap_hir_ty::method_resolution

pub fn for_crate(db: &dyn HirDatabase, krate: Crate) -> &InherentImpls
pub fn for_self_ty(&self, self_ty: &SimplifiedType) -> &[ImplId]
pub fn for_each_crate_and_block(
    db: &dyn HirDatabase, krate: Crate, block: Option<BlockId>,
    for_each: &mut dyn FnMut(&InherentImpls)
)
```

---

### 3.7 Method Resolution Functions

```rust
// in ra_ap_hir_ty::method_resolution
pub fn iterate_method_candidates_dyn(...)   // iterate all applicable methods for a type
pub fn iterate_path_candidates(...)         // iterate all path-accessible items
pub fn implements_trait(
    ty: &Canonical<Ty>, db: &dyn HirDatabase,
    env: Arc<TraitEnvironment>, krate: Crate, trait_: TraitId
) -> bool
pub fn implements_trait_unique(...) -> bool
pub fn is_dyn_method(...) -> bool
pub fn lookup_impl_const(...) -> Option<(DefWithBodyId, Substitution)>
pub fn def_crates(
    db: &dyn HirDatabase, ty: &Ty, krate: Crate
) -> Option<ArrayVec<Crate, 2>>             // crates where impls may live
```

---

### 3.8 Key `HirDatabase` Queries for Type Work

```rust
fn infer(&self, def: DefWithBodyId) -> Arc<InferenceResult>
fn trait_solve(
    &self, krate: Crate, block: Option<BlockId>,
    goal: Canonical<InEnvironment<Goal<Interner>>>
) -> Option<Solution<Interner>>
fn trait_environment(&self, def: GenericDefId) -> Arc<TraitEnvironment>
fn trait_environment_for_body(&self, def: DefWithBodyId) -> Arc<TraitEnvironment>
fn trait_impls_in_crate(&self, krate: Crate) -> Arc<TraitImpls>
fn trait_impls_in_deps(&self, krate: Crate) -> Arc<[Arc<TraitImpls>]>
fn inherent_impls_in_crate(&self, krate: Crate) -> Arc<InherentImpls>
fn callable_item_signature(&self, def: CallableDefId) -> Binders<CallableSig>
fn generic_predicates(&self, def: GenericDefId) -> GenericPredicates
fn field_types(&self, var: VariantId) -> Arc<ArenaMap<Idx<FieldData>, Binders<Ty>>>
fn impl_self_ty(&self, def: ImplId) -> Binders<Ty>
fn impl_trait(&self, def: ImplId) -> Option<Binders<TraitRef>>
fn normalize_projection(
    &self, projection: ProjectionTy<Interner>, env: Arc<TraitEnvironment>
) -> Ty<Interner>
fn lookup_impl_method(
    &self, env: Arc<TraitEnvironment>,
    func: FunctionId, fn_subst: Substitution<Interner>
) -> (FunctionId, Substitution<Interner>)
```

---

## 4. `ra_ap_syntax` — AST Parsing

> **Docs:** https://docs.rs/ra_ap_syntax/latest/ra_ap_syntax/  
> **Description:** "Syntax Tree library for rust-analyzer." Uses a lossless CST (Concrete Syntax Tree) where every character is preserved.

### 4.1 Core Type Aliases

```rust
pub type SyntaxNode = rowan::SyntaxNode<RustLanguage>
pub type SyntaxToken = rowan::SyntaxToken<RustLanguage>
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<RustLanguage>
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<RustLanguage>
pub type SyntaxNodePtr = rowan::ast::AstNodePtr<RustLanguage>
```

`SyntaxNode` is backed by the `rowan` crate — all traversal methods come from `rowan::SyntaxNode`:
```rust
// Navigation
fn parent(&self) -> Option<SyntaxNode>
fn children(&self) -> SyntaxNodeChildren
fn children_with_tokens(&self) -> SyntaxElementChildren
fn descendants(&self) -> impl Iterator<Item = SyntaxNode>
fn descendants_with_tokens(&self) -> impl Iterator<Item = SyntaxElement>
fn ancestors(&self) -> impl Iterator<Item = SyntaxNode>
fn first_child(&self) -> Option<SyntaxNode>
fn last_child(&self) -> Option<SyntaxNode>
fn first_token(&self) -> Option<SyntaxToken>
fn last_token(&self) -> Option<SyntaxToken>
fn next_sibling(&self) -> Option<SyntaxNode>
fn prev_sibling(&self) -> Option<SyntaxNode>

// Metadata
fn kind(&self) -> SyntaxKind
fn text_range(&self) -> TextRange
fn text(&self) -> SyntaxText
fn index(&self) -> usize

// Preorder traversal
fn preorder(&self) -> Preorder
fn preorder_with_tokens(&self) -> PreorderWithTokens
```

---

### 4.2 Parsing

```rust
// Parse a complete source file
pub fn parse(text: &str, edition: Edition) -> Parse<SourceFile>

// Parse<T> result:
pub fn tree(&self) -> T          // typed root node
pub fn syntax_node(&self) -> SyntaxNode
pub fn errors(&self) -> &[SyntaxError]
pub fn ok(self) -> Result<T, Vec<SyntaxError>>
```

---

### 4.3 `ast::SourceFile`

```rust
pub fn parse(text: &str, edition: Edition) -> Parse<SourceFile>

// Via AstNode trait:
pub fn syntax(&self) -> &SyntaxNode

// Via HasModuleItem trait:
pub fn items(&self) -> AstChildren<Item>    // top-level items

// Via HasAttrs:
pub fn attrs(&self) -> AstChildren<Attr>

// Via HasDocComments:
pub fn doc_comments(&self) -> DocCommentIter
```

---

### 4.4 Key AST Node Types (`ast::*`)

All AST nodes implement `AstNode` (giving access to `syntax()`) and optionally trait interfaces:

| AST Node | Key Access Traits |
|----------|-------------------|
| `ast::Fn` | `HasName`, `HasAttrs`, `HasVisibility`, `HasGenericParams` |
| `ast::Struct` | `HasName`, `HasAttrs`, `HasVisibility`, `HasGenericParams` |
| `ast::Enum` | `HasName`, `HasAttrs`, `HasVisibility`, `HasGenericParams` |
| `ast::Trait` | `HasName`, `HasAttrs`, `HasVisibility`, `HasGenericParams`, `HasTypeBounds` |
| `ast::Impl` | `HasAttrs`, `HasGenericParams` |
| `ast::Module` | `HasName`, `HasAttrs`, `HasVisibility` |
| `ast::TypeAlias` | `HasName`, `HasAttrs`, `HasVisibility`, `HasGenericParams` |
| `ast::Use` | `HasAttrs`, `HasVisibility` |
| `ast::CallExpr` | `HasArgList` |
| `ast::MethodCallExpr` | `HasArgList`, `HasGenericArgs` |
| `ast::Param` | `HasAttrs` |
| `ast::RecordField` | `HasName`, `HasAttrs`, `HasVisibility` |

**`AstNode` trait:**
```rust
pub trait AstNode {
    fn can_cast(kind: SyntaxKind) -> bool where Self: Sized;
    fn cast(syntax: SyntaxNode) -> Option<Self> where Self: Sized;
    fn syntax(&self) -> &SyntaxNode;
}
```

**`match_ast!` macro** — pattern match a `SyntaxNode` against typed AST nodes:
```rust
match_ast! {
    match node {
        ast::Fn(it) => { /* handle function */ },
        ast::Struct(it) => { /* handle struct */ },
        _ => {}
    }
}
```

---

### 4.5 `algo` Module — Tree Algorithms

```rust
// in ra_ap_syntax::algo
pub fn find_node_at_offset<N: AstNode>(syntax: &SyntaxNode, offset: TextSize) -> Option<N>
pub fn find_node_at_range<N: AstNode>(syntax: &SyntaxNode, range: TextRange) -> Option<N>
pub fn ancestors_at_offset(syntax: &SyntaxNode, offset: TextSize) -> impl Iterator<Item = SyntaxNode>
pub fn least_common_ancestor(u: &SyntaxNode, v: &SyntaxNode) -> Option<SyntaxNode>
pub fn neighbor<T: AstNode>(node: &T, direction: Direction) -> Option<T>
pub fn non_trivia_sibling(element: SyntaxElement, direction: Direction) -> Option<SyntaxElement>
pub fn previous_non_trivia_token(token: SyntaxToken) -> Option<SyntaxToken>
pub fn skip_trivia_token(token: SyntaxToken, direction: Direction) -> Option<SyntaxToken>
pub fn has_errors(node: &SyntaxNode) -> bool
```

---

## 5. `ra_ap_ide` — IDE Features Layer

> **Docs:** https://docs.rs/ra_ap_ide/latest/ra_ap_ide/  
> **Description:** "ide-centric APIs. Operates with files and text ranges, returns Strings suitable for display."

### 5.1 `AnalysisHost` — World State

```rust
pub struct AnalysisHost

pub fn new(lru_capacity: Option<u16>) -> AnalysisHost
pub fn with_database(db: RootDatabase) -> AnalysisHost
pub fn apply_change(&mut self, change: ChangeWithProcMacros)
pub fn analysis(&self) -> Analysis           // immutable snapshot
pub fn raw_database(&self) -> &RootDatabase
pub fn raw_database_mut(&mut self) -> &mut RootDatabase
```

---

### 5.2 `Analysis` — Query Snapshot

All methods return `Cancellable<T>` = `Result<T, Cancelled>`. Cancelled if the world state is updated while a query is running.

**Setup / indexing:**
```rust
pub fn from_single_file(text: String) -> (Analysis, FileId)  // quick single-file setup
pub fn fetch_crates(&self) -> Cancellable<FxIndexSet<CrateInfo>>
pub fn parallel_prime_caches<F>(&self, num_worker_threads: usize, cb: F) -> Cancellable<()>
pub fn file_text(&self, file_id: FileId) -> Cancellable<Arc<str>>
pub fn parse(&self, file_id: FileId) -> Cancellable<SourceFile>
pub fn view_item_tree(&self, file_id: FileId) -> Cancellable<String>
pub fn view_hir(&self, position: FilePosition) -> Cancellable<String>
```

**Navigation / definitions:**
```rust
pub fn goto_definition(
    &self, position: FilePosition, config: &GotoDefinitionConfig<'_>
) -> Cancellable<Option<RangeInfo<Vec<NavigationTarget>>>>

pub fn goto_declaration(
    &self, position: FilePosition, config: &GotoDefinitionConfig<'_>
) -> Cancellable<Option<RangeInfo<Vec<NavigationTarget>>>>

pub fn goto_implementation(
    &self, config: &GotoImplementationConfig, position: FilePosition
) -> Cancellable<Option<RangeInfo<Vec<NavigationTarget>>>>

pub fn goto_type_definition(
    &self, position: FilePosition
) -> Cancellable<Option<RangeInfo<Vec<NavigationTarget>>>>
```

**References:**
```rust
pub fn find_all_refs(
    &self, position: FilePosition, config: &FindAllRefsConfig<'_>
) -> Cancellable<Option<Vec<ReferenceSearchResult>>>
```

**Call hierarchy:**
```rust
pub fn call_hierarchy(
    &self, position: FilePosition, config: &CallHierarchyConfig<'_>
) -> Cancellable<Option<RangeInfo<Vec<NavigationTarget>>>>

pub fn incoming_calls(
    &self, config: &CallHierarchyConfig<'_>, position: FilePosition
) -> Cancellable<Option<Vec<CallItem>>>

pub fn outgoing_calls(
    &self, config: &CallHierarchyConfig<'_>, position: FilePosition
) -> Cancellable<Option<Vec<CallItem>>>
```

**Symbol search:**
```rust
pub fn symbol_search(
    &self, query: Query, limit: usize
) -> Cancellable<Vec<NavigationTarget>>
```

**Module hierarchy:**
```rust
pub fn parent_module(&self, position: FilePosition) -> Cancellable<Vec<NavigationTarget>>
pub fn child_modules(&self, position: FilePosition) -> Cancellable<Vec<NavigationTarget>>
```

**Crate queries:**
```rust
pub fn crates_for(&self, file_id: FileId) -> Cancellable<Vec<Crate>>
pub fn crate_root(&self, crate_id: Crate) -> Cancellable<FileId>
pub fn crate_edition(&self, crate_id: Crate) -> Cancellable<Edition>
pub fn relevant_crates_for(&self, file_id: FileId) -> Cancellable<Vec<Crate>>
pub fn transitive_rev_deps(&self, crate_id: Crate) -> Cancellable<Vec<Crate>>
pub fn is_proc_macro_crate(&self, crate_id: Crate) -> Cancellable<bool>
pub fn is_crate_no_std(&self, crate_id: Crate) -> Cancellable<bool>
```

**Structure (file outline):**
```rust
pub fn file_structure(
    &self, config: &FileStructureConfig, file_id: FileId
) -> Cancellable<Vec<StructureNode>>
```

**Hover/signature (rich metadata):**
```rust
pub fn hover(
    &self, config: &HoverConfig<'_>, range: FileRange
) -> Cancellable<Option<RangeInfo<HoverResult>>>

pub fn signature_help(
    &self, position: FilePosition
) -> Cancellable<Option<SignatureHelp>>

pub fn inlay_hints(
    &self, config: &InlayHintsConfig<'_>,
    file_id: FileId, range: Option<TextRange>
) -> Cancellable<Vec<InlayHint>>
```

**Diagnostics:**
```rust
pub fn syntax_diagnostics(
    &self, config: &DiagnosticsConfig, file_id: FileId
) -> Cancellable<Vec<Diagnostic>>
pub fn semantic_diagnostics(
    &self, config: &DiagnosticsConfig,
    resolve: AssistResolveStrategy, file_id: FileId
) -> Cancellable<Vec<Diagnostic>>
```

**Static index (for LSIF/SCIP output):**
```rust
pub fn moniker(
    &self, position: FilePosition
) -> Cancellable<Option<RangeInfo<Vec<MonikerResult>>>>
```

---

### 5.3 Key Return Types

**`NavigationTarget`** — represents any navigable code element:
```rust
pub struct NavigationTarget {
    pub file_id: FileId,
    pub full_range: TextRange,
    pub focus_range: Option<TextRange>,
    pub name: SmolStr,
    pub kind: Option<SymbolKind>,
    pub container_name: Option<SmolStr>,
    pub description: Option<String>,
    pub docs: Option<Documentation>,
    pub alias: Option<SmolStr>,
}
pub fn focus_or_full_range(&self) -> TextRange
```

**`CallItem`** — one node in a call hierarchy:
```rust
pub struct CallItem {
    pub target: NavigationTarget,   // the callee/caller
    pub ranges: Vec<FileRange>,     // call sites
}
```

**`ReferenceSearchResult`** — results of find-all-references:
```rust
pub struct ReferenceSearchResult {
    pub declaration: Option<Declaration>,
    pub references: IntMap<FileId, Vec<(TextRange, ReferenceCategory)>>,
}
```

**`StructureNode`** — file outline item:
```rust
pub struct StructureNode {
    pub parent: Option<usize>,
    pub label: String,
    pub navigation_range: TextRange,
    pub node_range: TextRange,
    pub kind: StructureNodeKind,
    pub detail: Option<String>,
    pub deprecated: bool,
}
```

**`StaticIndex`** — full static analysis for LSIF/SCIP:
```rust
pub fn compute<'a>(
    analysis: &'a Analysis, vendored_libs_config: VendoredLibrariesConfig<'_>
) -> StaticIndex<'a>
// Contains:
//   files: Vec<StaticIndexedFile>
//   tokens: TokenStore (symbol → type/definition info)
```

---

## 6. `ra_ap_base_db` — File and Database Primitives

> **Docs:** https://docs.rs/ra_ap_base_db/latest/ra_ap_base_db/  
> **Description:** "base_db defines basic database traits. The concrete DB is defined by ide."

### 6.1 `FileId`

```rust
pub struct FileId(u32)  // opaque handle to a file in Vfs
```

A `FileId` is stable for the lifetime of a `Vfs` state. It is the primitive connection between `ra_ap_vfs` and the salsa database.

---

### 6.2 `CrateGraph` — Project Model

```rust
pub struct CrateGraph  // "turns text files into Rust crates"
```

Built at project load time. Maps sets of `FileId` → `CrateId` with dependencies and cfg flags.

**`CrateData`** fields:
```rust
pub root_file_id: FileId
pub edition: Edition
pub display_name: Option<CrateDisplayName>
pub cfg_options: CfgOptions
pub potential_cfg_options: Option<CfgOptions>
pub env: Env
pub dependencies: Vec<Dependency>
pub origin: CrateOrigin
pub is_proc_macro: bool
pub channel: Option<ReleaseChannel>
```

**`Dependency`:**
```rust
pub struct Dependency {
    pub crate_id: CrateId,
    pub name: CrateName,
    pub prelude: bool,
}
```

---

### 6.3 `SourceDatabase` Trait

The base salsa database trait. `RootDatabase` in `ra_ap_ide` implements this:

```rust
pub trait SourceDatabase: FileLoader {
    // Query: file text
    fn file_text(&self, file_id: FileId) -> Arc<str>    // via FileTextQuery
    fn parse_errors(&self, file_id: FileId) -> ...       // via ParseErrorsQuery

    // Query: crate graph
    fn crate_graph(&self) -> Arc<CrateGraph>             // via CrateGraphQuery

    // Query: source root
    fn source_root(&self, id: SourceRootId) -> Arc<SourceRoot>   // via SourceRootQuery
    fn file_source_root(&self, id: FileId) -> SourceRootId
}
```

**`SourceRoot`** — a directory watched for changes (maps to one crate root or library root):
```rust
pub struct SourceRoot {
    pub is_library: bool,
    // Contains a FileSet: FileId ↔ VfsPath mappings
}
```

**`FileLoader` trait:**
```rust
pub trait FileLoader {
    fn file_text(&self, file_id: FileId) -> Arc<str>
    fn resolve_path(&self, path: AnchoredPath<'_>) -> Option<FileId>
    fn relevant_crates(&self, file_id: FileId) -> Arc<FxHashSet<CrateId>>
}
```

---

## 7. `ra_ap_vfs` — Virtual File System

> **Docs:** https://docs.rs/ra_ap_vfs/latest/ra_ap_vfs/  
> **Description:** "Virtual File System — maps paths to FileIds and tracks changes."

### 7.1 `Vfs`

```rust
pub struct Vfs

pub fn file_id(&self, path: &VfsPath) -> Option<(FileId, FileExcluded)>
pub fn file_path(&self, file_id: FileId) -> &VfsPath
pub fn iter(&self) -> impl Iterator<Item = (FileId, &VfsPath)> + '_
pub fn set_file_contents(&mut self, path: VfsPath, contents: Option<Vec<u8>>) -> bool
pub fn take_changes(&mut self) -> IndexMap<FileId, ChangedFile, ...>
pub fn exists(&self, file_id: FileId) -> bool
pub fn insert_excluded_file(&mut self, path: VfsPath)
```

### 7.2 `VfsPath`

```rust
pub struct VfsPath  // either AbsPath or virtual path

// Construction:
VfsPath::new_real_path(abs_path: AbsPathBuf) -> Self
VfsPath::new_virtual_path(path: String) -> Self

// Access:
pub fn as_path(&self) -> Option<&AbsPath>
```

### 7.3 `ChangedFile`

```rust
pub struct ChangedFile {
    pub file_id: FileId,
    pub kind: ChangeKind,  // Create | Modify | Delete
}
```

---

## 8. Code-Graph Construction Patterns

### Pattern 1: Full Crate Semantic Graph (High-Level)

```rust
// 1. Set up database
let host = AnalysisHost::default();
// ... apply_change with CrateGraph + file contents ...
let db = host.raw_database();

// 2. Iterate all crates
let crates = Crate::all(db);
for krate in &crates {
    // 3. Iterate all modules
    for module in krate.modules(db) {
        // 4. Enumerate all items
        for module_def in module.declarations(db) {
            match module_def {
                ModuleDef::Function(f) => {
                    // Entity: function node
                    let name = f.name(db);
                    let visibility = f.visibility(db);
                    let attrs = f.attrs(db);
                    let ret_ty = f.ret_type(db);
                    let params = f.params_without_self(db);
                    // params[i].ty() → typed parameter edges
                }
                ModuleDef::Adt(Adt::Struct(s)) => {
                    let fields = s.fields(db);
                    // fields[i].ty(db) → field type edges
                }
                ModuleDef::Trait(t) => {
                    let impls = Impl::all_for_trait(db, t);
                    // impls → implementation edges
                }
                _ => {}
            }
        }
        // 5. Enumerate impl blocks
        for impl_ in module.impl_defs(db) {
            let self_ty = impl_.self_ty(db);
            let trait_ = impl_.trait_(db);  // Some → trait impl edge
            let items = impl_.items(db);
        }
    }
}
```

### Pattern 2: Call Graph via IDE Layer

```rust
let analysis = host.analysis();
let config = CallHierarchyConfig::default();

// For a function at a known position:
let outgoing = analysis.outgoing_calls(&config, position)?;
// outgoing: Vec<CallItem> where each CallItem.target is the callee

let incoming = analysis.incoming_calls(&config, position)?;
// incoming: Vec<CallItem> where each CallItem.target is the caller
```

### Pattern 3: Find All Implementations of a Trait

```rust
// Using ra_ap_hir:
let impls = Impl::all_for_trait(db, trait_);
for impl_ in impls {
    let implementing_type = impl_.self_ty(db);
    // implementing_type.as_adt() → get the concrete type
}

// Using ra_ap_hir_ty (lower level):
let trait_impls = TraitImpls::for_crate_and_deps(db, krate);
for arc in trait_impls.iter() {
    arc.for_trait(trait_id, |impls| {
        for impl_id in impls { /* ... */ }
    });
}

// Using ra_ap_ide:
let nav = analysis.goto_implementation(config, position)?.unwrap();
// nav.info: Vec<NavigationTarget> — each is an implementing type/method
```

### Pattern 4: Resolve Type of Any Expression

```rust
// Via ra_ap_hir Semantics (preferred for syntax → semantic):
let semantics = Semantics::new_dyn(db);
let ty: Option<TypeInfo> = semantics.type_of_expr(&ast_expr);
if let Some(ty_info) = ty {
    let original = ty_info.original;  // Type<'db>
    if let Some(adt) = original.as_adt() {
        // adt is Struct | Enum | Union
    }
}

// Via ra_ap_hir_ty InferenceResult (for body-level iteration):
let infer = db.infer(def_with_body_id);
let ty: Option<&Ty> = infer.type_of_expr_or_pat(expr_or_pat_id);
```

### Pattern 5: Extract Doc Comments and Attributes

```rust
let attrs = function.attrs(db);  // AttrsWithOwner

// Doc comments:
for doc_expr in attrs.doc_exprs() { /* doc text */ }
// or via by_key:
let doc_attr = attrs.by_key(sym::doc);

// cfg conditions:
if let Some(cfg) = attrs.cfg() { /* CfgExpr */ }

// Derive macros (on struct/enum):
// Access via ItemScope::derive_macro_invocs(...)

// Arbitrary attribute:
let deprecated = attrs.by_key(sym::deprecated);
```

---

## 9. API Summary Table

| Goal | Primary API | Crate | Compiler-Verified? |
|------|-------------|-------|--------------------|
| Enumerate all crates | `Crate::all(db)` | ra_ap_hir | ✓ |
| Enumerate all modules | `krate.modules(db)` | ra_ap_hir | ✓ |
| Enumerate module items | `module.declarations(db)` | ra_ap_hir | ✓ |
| Function signature | `fn.ret_type(db)`, `fn.params_without_self(db)` | ra_ap_hir | ✓ |
| Function visibility | `fn.visibility(db)` | ra_ap_hir | ✓ |
| Struct fields + types | `struct.fields(db)`, `field.ty(db)` | ra_ap_hir | ✓ |
| Enum variants | `enum.variants(db)`, `variant.fields(db)` | ra_ap_hir | ✓ |
| Trait items | `trait.items(db)` | ra_ap_hir | ✓ |
| Supertrait chain | `trait.all_supertraits(db)` | ra_ap_hir | ✓ |
| Trait implementors | `Impl::all_for_trait(db, t)` | ra_ap_hir | ✓ |
| Type → trait check | `ty.impls_trait(db, t, args)` | ra_ap_hir | ✓ |
| Resolve expression type | `semantics.type_of_expr(expr)` | ra_ap_hir | ✓ |
| Resolve method call | `semantics.resolve_method_call(call)` | ra_ap_hir | ✓ |
| Resolve path | `semantics.resolve_path(path)` | ra_ap_hir | ✓ |
| Type inference (batch) | `db.infer(def)` → `InferenceResult` | ra_ap_hir_ty | ✓ |
| All impls in crate | `TraitImpls::for_crate(db, krate)` | ra_ap_hir_ty | ✓ |
| Go to definition | `analysis.goto_definition(pos, cfg)` | ra_ap_ide | ✓ |
| Go to implementation | `analysis.goto_implementation(cfg, pos)` | ra_ap_ide | ✓ |
| Find all references | `analysis.find_all_refs(pos, cfg)` | ra_ap_ide | ✓ |
| Call hierarchy | `analysis.incoming_calls(cfg, pos)` / `outgoing_calls` | ra_ap_ide | ✓ |
| Symbol search | `analysis.symbol_search(query, limit)` | ra_ap_ide | ✓ |
| File outline | `analysis.file_structure(cfg, file_id)` | ra_ap_ide | ✓ |
| Doc comments | `item.attrs(db).doc_exprs()` | ra_ap_hir | ✓ |
| Generic parameters | `generic_def.type_or_const_params(db)` | ra_ap_hir | ✓ |
| Names in scope | `semantics_scope.process_all_names(f)` | ra_ap_hir | ✓ |
| Parse source file | `SourceFile::parse(text, edition)` | ra_ap_syntax | Syntax only |
| AST traversal | `node.descendants()`, `match_ast!` | ra_ap_syntax | Syntax only |
| ItemTree (macro-exp) | `db.file_item_tree(file_id)` | ra_ap_hir_def | ✓ |
| Name resolution | `Resolver::resolve_path_in_type_ns(db, path)` | ra_ap_hir_def | ✓ |
| Field visibility map | `db.field_visibilities(variant_id)` | ra_ap_hir_def | ✓ |
| Crate dependency graph | `krate.dependencies(db)` | ra_ap_hir | ✓ |
| Load files into DB | `Vfs::set_file_contents(path, contents)` | ra_ap_vfs | N/A |
| Path → FileId | `vfs.file_id(path)` | ra_ap_vfs | N/A |
| Set up crate graph | `CrateGraph` + `ChangeWithProcMacros` | ra_ap_base_db | N/A |

---

## 10. Setup Skeleton

```rust
use ra_ap_vfs::{Vfs, VfsPath};
use ra_ap_base_db::{CrateGraph, FileId, SourceRoot, SourceRootId};
use ra_ap_ide::{AnalysisHost, FileChange};
use ra_ap_hir::{Crate, Semantics};
use ra_ap_syntax::Edition;

// 1. Create host
let mut host = AnalysisHost::default();

// 2. Create VFS and load files
let mut vfs = Vfs::default();
vfs.set_file_contents(
    VfsPath::new_real_path(abs_path_buf),
    Some(source_text.into_bytes())
);
let changes = vfs.take_changes();

// 3. Build CrateGraph
let mut crate_graph = CrateGraph::default();
// crate_graph.add_crate_root(file_id, edition, display_name, cfg_options, ...)

// 4. Apply change to host
let mut change = ra_ap_ide::Change::new();
for (file_id, changed_file) in changes {
    change.change_file(file_id, Some(file_content));
}
change.set_crate_graph(crate_graph);
host.apply_change(change.into());

// 5. Query
let db = host.raw_database();
let analysis = host.analysis();

// Use Semantics for HIR access:
let semantics = Semantics::new_dyn(db as &dyn ra_ap_hir::db::HirDatabase);

// Use analysis for IDE features:
let refs = analysis.find_all_refs(position, &config);
```

---

## Sources

- [ra_ap_hir docs.rs](https://docs.rs/ra_ap_hir/latest/ra_ap_hir/)
- [ra_ap_hir_def docs.rs](https://docs.rs/ra_ap_hir_def/latest/ra_ap_hir_def/)
- [ra_ap_hir_ty docs.rs](https://docs.rs/ra_ap_hir_ty/latest/ra_ap_hir_ty/)
- [ra_ap_syntax docs.rs](https://docs.rs/ra_ap_syntax/latest/ra_ap_syntax/)
- [ra_ap_ide docs.rs](https://docs.rs/ra_ap_ide/latest/ra_ap_ide/)
- [ra_ap_base_db docs.rs](https://docs.rs/ra_ap_base_db/latest/ra_ap_base_db/)
- [ra_ap_vfs docs.rs](https://docs.rs/ra_ap_vfs/latest/ra_ap_vfs/)
- [rust-analyzer GitHub](https://github.com/rust-lang/rust-analyzer)
- [crates.io ra_ap_hir](https://crates.io/crates/ra_ap_hir)
