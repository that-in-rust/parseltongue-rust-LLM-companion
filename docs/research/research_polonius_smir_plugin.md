# Rust Compiler Ecosystem Research: Polonius, Stable MIR, rustc_plugin, and Charon
## For Parseltongue Code-Graph Construction

*Research date: March 3, 2026*

---

## Table of Contents

1. [Polonius — Borrow Check Facts & Graph Edges](#1-polonius)
2. [Flowistry — Information Flow via Ownership](#2-flowistry)
3. [Stable MIR / rustc_public — Stable Compiler API](#3-stable-mir--rustc_public)
4. [rustc_plugin & rustc_utils — Brown University Framework](#4-rustc_plugin--rustc_utils)
5. [Charon — LLBC Format & MIR Extraction](#5-charon)
6. [Summary: Graph Construction Mapping](#6-summary-graph-construction-mapping)

---

## 1. Polonius

**Repository:** https://github.com/rust-lang/polonius  
**Book / Docs:** https://rust-lang.github.io/polonius/

Polonius is the next-generation borrow checker for Rust, implemented as a Datalog engine. It models lifetimes as **origins** — sets of loans — rather than sets of program points. This model is richer and more precise than the existing NLL borrow checker.

### 1.1 Key Conceptual Types (Atoms)

| Atom | Description | Parseltongue Relevance |
|------|-------------|----------------------|
| `origin` | An inference variable representing a set of loans. Corresponds to a lifetime `'a`. | Lifetime node in graph. Edges: origin ⊆ origin (subset_base). |
| `loan` | An abstract borrow created at a `&` or `&mut` expression. One loan per borrow site. | Borrow-edge: tracks what place was borrowed and when. |
| `point` | A location in the CFG. Every statement has a `Start(Stmt)` and `Mid(Stmt)` point. | CFG node; connects statements into control-flow edges. |
| `variable` | A local variable (`Local` in MIR). | Variable node; feeds var_defined_at / var_used_at liveness edges. |
| `path` | A move path — a place or subpath thereof (e.g. `x`, `x.0`, `x.field`). | Ownership paths; child_path gives tree structure of owned data. |

### 1.2 Complete Input Fact Schema

These are the complete fields of `PoloniusFacts` (alias for `AllFacts<RustcFacts>`) as exposed in `rustc_borrowck::polonius::legacy::facts`:

**Control Flow**

| Fact | Signature | Meaning |
|------|-----------|---------|
| `cfg_edge` | `(LocationIndex, LocationIndex)` | Directed edge in the CFG between two program points. |

**Loan Lifecycle**

| Fact | Signature | Meaning |
|------|-----------|---------|
| `loan_issued_at` | `(PoloniusRegionVid, BorrowIndex, LocationIndex)` | Loan `L` created at point `P` inside origin `O`. The origin may reference the loan from `P` onwards. |
| `loan_killed_at` | `(BorrowIndex, LocationIndex)` | A prefix of the borrowed path is assigned at `P`, stopping the loan's propagation. |
| `loan_invalidated_at` | `(LocationIndex, BorrowIndex)` | An action at `P` violates the terms of loan `L` (illegal if `L` is live). |

**Origin / Region Relationships**

| Fact | Signature | Meaning |
|------|-----------|---------|
| `subset_base` | `(PoloniusRegionVid, PoloniusRegionVid, LocationIndex)` | `origin1 ⊆ origin2` at point `P` — the non-transitive outlives constraint. |
| `known_placeholder_subset` | `(PoloniusRegionVid, PoloniusRegionVid)` | Declared `'a: 'b` relationships from function signatures or implied bounds. |
| `placeholder` | `(PoloniusRegionVid, BorrowIndex)` | An origin that is a universal region (e.g. `'static`, or lifetime parameters like `'a`, `'b`). |
| `universal_region` | `Vec<PoloniusRegionVid>` | *(Being phased out)* Same as placeholder but without the associated loan. |

**Liveness**

| Fact | Signature | Meaning |
|------|-----------|---------|
| `origin_live_at` | `(PoloniusRegionVid, LocationIndex)` | The origin appears in the type of a live variable at point `P`. |
| `var_used_at` | `(Local, LocationIndex)` | Variable `v` is used (read) at point `P`. |
| `var_defined_at` | `(Local, LocationIndex)` | Variable `v` is defined (written) at point `P`. |
| `var_dropped_at` | `(Local, LocationIndex)` | Variable `v` is dropped at point `P`. |
| `use_of_var_derefs_origin` | `(Local, PoloniusRegionVid)` | Using variable `v` dereferences origin `O`. |
| `drop_of_var_derefs_origin` | `(Local, PoloniusRegionVid)` | Dropping variable `v` dereferences origin `O`. |

**Move / Path Analysis**

| Fact | Signature | Meaning |
|------|-----------|---------|
| `child_path` | `(MovePathIndex, MovePathIndex)` | Move-path tree: `parent → child` (e.g. `x → x.field`). |
| `path_is_var` | `(MovePathIndex, Local)` | Relates a move-path to its root variable. |
| `path_assigned_at_base` | `(MovePathIndex, LocationIndex)` | A path is initialized/assigned at point `P`. |
| `path_moved_at_base` | `(MovePathIndex, LocationIndex)` | A path is moved (ownership transferred) at point `P`. |
| `path_accessed_at_base` | `(MovePathIndex, LocationIndex)` | A path is accessed at point `P`. |

### 1.3 Derived (Computed) Output Relations

Polonius computes these through its Datalog rules:

| Derived Relation | Meaning |
|-----------------|---------|
| `subset(O1, O2, P)` | Transitive closure of `subset_base` at each point. The full origin ⊆ origin graph at every CFG point. |
| `origin_contains_loan_on_entry(O, L, P)` | Origin `O` contains loan `L` at point `P` (computed via subset propagation + CFG propagation). |
| `loan_live_at(L, P)` | Loan `L` is live at `P` (it's contained in a live origin). |
| `errors(L, P)` | Illegal access: loan `L` is invalidated at `P` while still live. |
| `subset_errors(O1, O2, P)` | Illegal subset relation: undeclared outlives relationship between two placeholder origins. |

### 1.4 How to Access Polonius Facts from rustc

**Flag-based export (filesystem):**
```
RUSTFLAGS="-Znll-facts" cargo build
```
This writes per-function `.facts` files (tab-separated) to `nll-facts/<fn-name>/` for offline Datalog analysis.

**In-process access (rustc_borrowck):**
The `rustc_borrowck::polonius::legacy::facts::PoloniusFacts` struct is populated during borrow checking. Tools can access it via `rustc_driver::Callbacks::after_analysis`. The `rustc_middle::mir::Body` context is available at that phase.

**Analysis variants:**
- `LocationInsensitive` — fast pre-pass, ignores CFG positions of subset constraints
- `Naive` — complete O(n³) Datalog transitive closure; reference implementation  
- `DatafrogOpt` (Hybrid) — production algorithm combining location-insensitive pre-pass with optimized location-sensitive check

### 1.5 Graph Edges Derivable from Polonius Facts

| Graph Edge Type | Source Facts | Trust Level |
|----------------|--------------|-------------|
| **CFG edge** (control flow) | `cfg_edge` | Compiler-verified |
| **Borrow creation** (place → loan) | `loan_issued_at(O, L, P)` | Compiler-verified |
| **Borrow kill** | `loan_killed_at(L, P)` | Compiler-verified |
| **Loan invalidation** (borrow violation site) | `loan_invalidated_at(L, P)` | Compiler-verified |
| **Subset / outlives** (lifetime flows into lifetime) | `subset_base(O1, O2, P)` | Compiler-verified |
| **Origin liveness** | `origin_live_at(O, P)` | Compiler-verified |
| **Variable def/use** | `var_defined_at`, `var_used_at` | Compiler-verified |
| **Ownership move** | `path_moved_at_base(Path, P)` | Compiler-verified |
| **Ownership tree** (struct field reachability) | `child_path(parent, child)` | Compiler-verified |
| **Variable → origin link** | `use_of_var_derefs_origin(Var, O)` | Compiler-verified |
| **Borrow lifetime span** | `origin_contains_loan_on_entry` (derived) | Compiler-verified |

---

## 2. Flowistry

**Repository:** https://github.com/browncel/flowistry  
**Paper:** "Modular Information Flow through Ownership" (Crichton et al., PLDI 2022)  
**Docs:** https://willcrichton.net/flowistry/flowistry/

Flowistry is a modular information-flow analysis for Rust programs. It leverages Rust's ownership type system to perform **function-modular** alias analysis without requiring whole-program analysis. It builds a Program Dependence Graph (PDG) from MIR.

### 2.1 Core Approach: Ownership-Based Alias Analysis

Flowistry replaces traditional pointer analysis with borrow-checker information. The key insight from the paper:

> "Ownership types can be used to soundly and precisely analyze information flow through function calls given only their type signature."

The approach works in four steps:
1. **Loan set computation** — For each place `p`, compute the set of borrows that `p` might alias, using the outlives constraints from the borrow checker.
2. **Subset graph traversal** — Trace from an origin backward through `subset_base` edges to find which loans (= places) an origin can refer to.
3. **Dataflow over MIR** — Propagate information-flow dependencies forward through basic blocks using MIR statements.
4. **PDG construction** — Edges represent data-flow (`x affects y`) and control-flow dependencies (which statements control which branches).

### 2.2 Key APIs

| API | Location | Description |
|-----|----------|-------------|
| `infoflow::compute_flow(tcx, body, def_id)` | `flowistry::infoflow` | Main entry point. Given a MIR `Body`, returns the complete information-flow analysis. |
| `mir::aliases::Aliases` | `flowistry::mir` | The alias analysis structure; maps places to their potential aliased places via borrow checker data. |
| `infoflow::FlowResults` | `flowistry::infoflow` | The computed flow information (data dependency sets at each program point). |

### 2.3 How Flowistry Uses Borrow Checker Data

Flowistry accesses borrow-checker outputs via `rustc_middle::ty::TyCtxt` (not directly via Polonius files). It uses:
- **`body.local_decls`** — type information of locals, which encodes lifetime variables
- **`RegionVid`** — origin/region identifiers from NLL
- **Outlives constraints** from the NLL inference (`region_infer_context`) to determine which places can alias at each program point

The subset graph (`subset_base` / `subset`) is the core alias oracle: tracing backwards from an origin `O` finds all loans in `O`, and each loan points to the exact place expression that was borrowed.

### 2.4 Graph Data Flowistry Enables

| Graph Edge | From Flowistry | Parseltongue Use |
|------------|---------------|-----------------|
| **Data dependency edge** (x → y: x affects y) | `compute_flow` result | Taint analysis, slicing |
| **Control dependency edge** | `BodyExt::control_dependencies()` | Control-flow sensitive slicing |
| **Alias edge** (p1 may-alias p2) | `Aliases` struct | Ownership aliasing, borrow-check provenance |
| **PDG node** (each MIR location) | `FlowResults` | Statement-level dependency graph |

### 2.5 Downstream Users

- **Paralegal** — Uses Flowistry's PDG for security/privacy policy checking
- **Aquascope** — Uses Flowistry for lifetime visualization in IDEs
- **Argus** — Uses Flowistry for type-checking aid and proof search

---

## 3. Stable MIR / rustc_public

**Repository:** https://github.com/rust-lang/rust (compiler/rustc_public/, compiler/rustc_public_bridge/)  
**Nightly Docs:** https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/index.html  
**Project:** https://github.com/rust-lang/project-stable-mir

### 3.1 Background and Status

`rustc_public` (formerly `stable_mir`) is the WIP public compiler API. It was recently renamed from `stable_mir` to `rustc_public` (GSoC 2025) and is being prepared for publication on crates.io. The goal is a versioned, maintained API for third-party tools.

**Architecture:**
- `rustc_public` — user-facing type definitions and APIs (will be on crates.io)
- `rustc_public_bridge` — internal bridge translating between stable and internal rustc types (stays in-tree, not for external use)

**Entry point:** Code must run inside the `run!()` macro (or `run_with_tcx!()` for mixed stable/internal usage). This sets up the thread-local `TyCtxt` context.

```rust
use rustc_public::run;
run!(args, || {
    let local = rustc_public::local_crate();
    let items = rustc_public::all_local_items();
    // ...
})
```

### 3.2 Top-Level Query Functions

| Function | Location | Returns | Description |
|----------|----------|---------|-------------|
| `local_crate()` | `rustc_public` | `Crate` | The currently compiled crate |
| `all_local_items()` | `rustc_public` | `CrateItems` | All items in the local crate that have MIR |
| `entry_fn()` | `rustc_public` | `Option<CrateItem>` | The entry function (`main`, or `start` for no-std) |
| `external_crates(name)` | `rustc_public` | `Vec<Crate>` | External crates by name |
| `find_crates(name)` | `rustc_public` | `Vec<Crate>` | All crates matching a name |
| `all_trait_decls()` | `rustc_public` | `TraitDecls` | All trait declarations |
| `all_trait_impls()` | `rustc_public` | `ImplTraitDecls` | All trait impl declarations |

### 3.3 Core Type Hierarchy

#### `CrateItem` (`rustc_public::CrateItem`)
```rust
pub struct CrateItem(pub DefId);
```
Represents one item in a crate.

| Method | Returns | Description |
|--------|---------|-------------|
| `expect_body()` | `Body` | MIR body (panics if unavailable) |
| `span()` | `Span` | Source span |
| `kind()` | `ItemKind` | fn / static / const / etc. |
| `ty()` | `Ty` | The item's type |
| `is_foreign_item()` | `bool` | Whether it's a foreign (extern) item |
| `requires_monomorphization()` | `bool` | Whether generic parameters remain |
| `all_tool_attrs()` | `Vec<Attribute>` | Tool attributes |

#### `DefId` Specializations
Unlike rustc's single `DefId`, `rustc_public` uses typed variants:

| Type | Meaning |
|------|---------|
| `FnDef` | A function definition |
| `StaticDef` | A static variable definition |
| `InstanceDef` | A monomorphized instance |
| `DefId` (generic) | Base definition identifier |

#### `Ty` and `TyKind` (`rustc_public::ty`)
```rust
// Ty is a lightweight identifier
let ty: Ty = item.ty();
// TyKind is the full structured type (computed on demand)
let kind: TyKind = ty.kind();
```

**Note from Kani:** `TyKind` is NOT cached — calling `ty.kind()` repeatedly is expensive. Cache results manually.

Key types in `rustc_public::ty`:
- `Ty` — opaque type identifier
- `TyKind` — the full enum: `Adt`, `Ref`, `FnDef`, `Tuple`, `Array`, `Slice`, `RawPtr`, `Str`, `Bool`, `Int`, `Uint`, `Float`, `Closure`, `Dynamic`, `Never`, etc.
- `GenericArgs` — list of generic arguments (type params, lifetime params, const params)
- `FnDef` — function definition with generic args

#### `Instance` (`rustc_public::mir::mono::Instance`)
Represents a monomorphized instance of a function. Key distinction: `CrateItem` may be generic; `Instance` is fully instantiated.

```rust
let instance = Instance::try_from(crate_item)?;
let body: Body = instance.body()?;  // fully monomorphized MIR
```

#### `Body` (`rustc_public::mir::Body`)
The full MIR body of a function.

| Field/Method | Type | Description |
|-------------|------|-------------|
| `basic_blocks` | `Vec<BasicBlock>` | All basic blocks |
| `local_decls` | `Vec<LocalDecl>` | Variable declarations (index = Local) |
| `arg_count` | `usize` | Number of arguments |
| (various) | | Span, debug info, etc. |

#### `BasicBlock` (`rustc_public::mir::BasicBlock`)

| Field | Type | Description |
|-------|------|-------------|
| `statements` | `Vec<Statement>` | Non-terminating statements |
| `terminator` | `Terminator` | The block's terminator |
| `is_cleanup` | `bool` | Whether this is a cleanup/drop block |

#### `Statement` and `StatementKind`
- `Assign(Place, Rvalue)` — assignment
- `StorageLive(Local)` / `StorageDead(Local)` — variable storage scope
- `Deinit(Place)` — deinitialize
- `SetDiscriminant(Place, VariantIdx)` — set enum variant
- `FakeRead(FakeReadCause, Place)` — NLL fake read
- `Retag(RetagKind, Place)` — Stacked Borrows retag

#### `Terminator` and `TerminatorKind`
- `Return` — function return
- `Goto { target: BasicBlockIdx }` — unconditional branch
- `SwitchInt { discr: Operand, targets: SwitchTargets }` — conditional branch / match
- `Call { func, args, destination, target, unwind }` — function call
- `Drop { place, target, unwind }` — drop
- `Unreachable` — unreachable code
- `Assert { cond, expected, msg, target, unwind }` — runtime assertion

#### `Place` (`rustc_public::mir::Place`)

| Field | Type | Description |
|-------|------|-------------|
| `local` | `Local` | Root variable |
| `projection` | `Vec<ProjectionElem>` | Field accesses, deref, array index, etc. |

`ProjectionElem` variants: `Deref`, `Field(FieldIdx, Ty)`, `Index(Local)`, `ConstantIndex(...)`, `Subslice(...)`, `Downcast(Option<Symbol>, VariantIdx)`, `OpaqueCast(Ty)`, `Subtype(Ty)`

#### `Rvalue` (`rustc_public::mir::Rvalue`)
- `Use(Operand)` — copy/move
- `Ref(Region, BorrowKind, Place)` — create reference
- `AddressOf(Mutability, Place)` — raw pointer
- `BinaryOp(BinOp, Operand, Operand)` — arithmetic
- `UnaryOp(UnOp, Operand)` — negation, not
- `Aggregate(AggregateKind, Vec<Operand>)` — struct/enum/array construction
- `Discriminant(Place)` — read enum discriminant
- `Len(Place)` — slice/array length
- `Cast(CastKind, Operand, Ty)` — type cast
- `CopyForDeref(Place)` — copy for deref

#### `LocalDecl` (`rustc_public::mir::LocalDecl`)

| Field | Type | Description |
|-------|------|-------------|
| `ty` | `Ty` | Type of the local |
| `span` | `Span` | Source span |
| `mutability` | `Mutability` | `Mut` or `Not` |

### 3.4 What's Accessible vs. What's Missing

**Currently accessible via rustc_public:**
- Full CFG (basic blocks, statements, terminators)
- Type information (Ty, TyKind, generics)
- Function definitions and bodies
- Static and const definitions
- Trait declarations and impls
- Source spans
- MIR projections (field access, deref, etc.)
- Monomorphized instances

**Not yet in rustc_public / still unstable:**
- Polonius / borrow checker facts (remains in `rustc_borrowck`, no stable API)
- Full HIR access (only MIR is exposed)
- Incremental compilation integration
- Creating or mutating MIR items
- Inference context (region/type variables)

### 3.5 Stable MIR vs. rustc_middle Equivalents

| rustc_middle | rustc_public | Notes |
|-------------|-------------|-------|
| `TyCtxt` | Implicit (thread-local via `run!`) | No explicit context parameter needed |
| `DefId` | `DefId` (specialized: `FnDef`, `StaticDef`, etc.) | More type-safe |
| `ty::Ty<'tcx>` | `Ty` | Lightweight identifier; `kind()` for full info |
| `mir::Body<'tcx>` | `Body` | No lifetime parameter; all types already resolved |
| `mir::BasicBlock` | `BasicBlock` | Same concept |
| `mir::Place<'tcx>` | `Place` | No lifetime parameter |
| `tcx.instance_mir(instance)` | `Instance::body()` | Monomorphized directly |
| `tcx.optimized_mir(def_id)` | `CrateItem::expect_body()` | Direct body access |
| `tcx.all_local_trait_impls()` | `all_trait_impls()` | Similar coverage |

### 3.6 Graph Data Accessible through Stable MIR

| Graph Data | API Path | Trust Level |
|-----------|----------|-------------|
| Function call edges | `TerminatorKind::Call { func, .. }` | Compiler-verified |
| CFG edges | `TerminatorKind::{Goto, SwitchInt, Call, ..}` | Compiler-verified |
| Type of each variable | `body.local_decls[local].ty` | Compiler-verified |
| Field access / ownership tree | `Place::projection` elements | Compiler-verified |
| Reference creation | `Rvalue::Ref(region, kind, place)` | Compiler-verified |
| Ownership move | `Rvalue::Use(Operand::Move(place))` | Compiler-verified |
| Aggregate construction | `Rvalue::Aggregate(kind, operands)` | Compiler-verified |
| Trait dispatch target | `TerminatorKind::Call { func: Operand::Constant(FnDef) }` | Compiler-verified |
| Closure capture | `Rvalue::Aggregate(AggregateKind::Closure, ..)` | Compiler-verified |
| Raw pointer creation | `Rvalue::AddressOf(mutability, place)` | Compiler-verified |

---

## 4. rustc_plugin & rustc_utils

**Repository:** https://github.com/cognitive-engineering-lab/rustc_plugin  
**Docs:** https://willcrichton.net/flowistry/rustc_plugin/  
**Authors:** Brown University Cognitive Engineering Lab (Will Crichton et al.)

### 4.1 Overview

`rustc_plugin` is a framework that generalizes the Clippy driver infrastructure to support arbitrary compiler plugins that integrate with Cargo. It handles:
- Sysroot discovery (so rustc can find standard libraries)
- Cargo integration (running on all crates in a workspace, respecting dependencies)
- Argument marshalling between the Cargo-facing binary and the compiler-facing driver binary
- Crate filtering (run on specific files or all crates)

**Architecture:** A plugin requires **two binaries**:
1. `cargo-<name>` — the user-facing CLI (calls `cli_main`)
2. `<name>-driver` — the implementation binary (calls `driver_main`)
3. `lib.rs` — exports a struct implementing `RustcPlugin`

**Current version:** `0.13.0-nightly-2025-03-03` (MaxSRV: rustc 1.86)

### 4.2 The `RustcPlugin` Trait

```rust
pub trait RustcPlugin: Sized {
    // Associated type for plugin-specific serializable arguments
    type Args: Serialize + DeserializeOwned;

    // Required: version string (e.g. "0.1.0")
    fn version(&self) -> Cow<'static, str>;
    
    // Required: name of the driver binary
    fn driver_name(&self) -> Cow<'static, str>;
    
    // Required: parse CLI args; return args + crate filter
    fn args(&self, target_dir: &Utf8Path) -> RustcPluginArgs<Self::Args>;
    
    // Required: core plugin logic (runs inside rustc_driver)
    fn run(
        self,
        compiler_args: Vec<String>,  // rustc args (sysroot, crate, etc.)
        plugin_args: Self::Args,      // your custom args
    ) -> Result<()>;
    
    // Optional: modify `cargo check` invocation
    fn modify_cargo(&self, _cargo: &mut Command, _args: &Self::Args) { ... }
}
```

**Supporting types:**

```rust
pub struct RustcPluginArgs<A> {
    pub args: A,
    pub filter: CrateFilter,
}

pub enum CrateFilter {
    AllCrates,
    CrateContainingFile(PathBuf),
    OnlyCrate(String),
}
```

### 4.3 How it Wraps rustc_driver

Inside `run()`, the plugin uses `rustc_driver` directly:

```rust
fn run(self, compiler_args: Vec<String>, plugin_args: Self::Args) -> Result<()> {
    // Register your rustc_driver::Callbacks
    let mut callbacks = MyCallbacks { args: plugin_args };
    rustc_driver::run_compiler(&compiler_args, &mut callbacks);
    Ok(())
}
```

The `rustc_driver::Callbacks` trait provides hooks:
- `config()` — modify `rustc_interface::Config` before compilation starts
- `after_expansion()` — hook after macro expansion / HIR is available  
- `after_analysis()` — hook after type-checking and MIR generation (most common)

Inside `after_analysis()`, the full `rustc_middle::ty::TyCtxt` is available, giving access to MIR bodies, types, HIR, and all borrow-check results.

### 4.4 `rustc_utils` Extension Traits

All traits are in the `rustc_utils` crate. They extend rustc's built-in types with analysis-friendly methods.

#### `BodyExt<'tcx>` — MIR Body Extensions
Location: `rustc_utils::mir::body`

| Method | Returns | Description |
|--------|---------|-------------|
| `all_returns()` | `impl Iterator<Item=Location>` | All `Return` terminator locations |
| `all_locations()` | `impl Iterator<Item=Location>` | Every statement location in the body |
| `locations_in_block(bb)` | `impl Iterator<Item=Location>` | All locations in a basic block |
| `debug_info_name_map()` | `HashMap<String, Local>` | Map from source variable name → MIR Local |
| `to_string(tcx)` | `Result<String>` | Pretty-print the MIR body |
| `location_to_hir_id(loc)` | `HirId` | Map MIR location back to source HIR node |
| `source_info_to_hir_id(info)` | `HirId` | Map SourceInfo to HIR node |
| `control_dependencies()` | `ControlDependencies<BasicBlock>` | The control-dependence graph (CDG) |
| `async_context(tcx, def_id)` | `Option<Ty<'tcx>>` | For async fns: the future type context |
| `all_places(tcx, def_id)` | `impl Iterator<Item=Place>` | All places reachable in the body |
| `regions_in_args()` | `impl Iterator<Item=Region>` | All region/lifetime variables in argument types |
| `regions_in_return()` | `impl Iterator<Item=Region>` | Region/lifetime variables in return type |

**Key for graph construction:** `control_dependencies()` gives the CDG (which basic blocks control which others), enabling both data-flow and control-flow edge construction.

#### `PlaceExt<'tcx>` — Place/Memory Location Extensions
Location: `rustc_utils::mir::place`

| Method | Returns | Description |
|--------|---------|-------------|
| `make(local, projection, tcx)` | `Place` | Constructor |
| `from_ref(place_ref, tcx)` | `Place` | From a PlaceRef |
| `from_local(local, tcx)` | `Place` | Root place for a local variable |
| `is_arg(body)` | `bool` | Whether this place is a function argument |
| `is_direct(body, tcx)` | `bool` | No dereferences in the projection |
| `refs_in_projection(body, tcx)` | `impl Iterator<...>` | All reference projections along the path |
| `interior_pointers(tcx, body, def_id)` | `HashMap<RegionVid, Vec<(Place, Mutability)>>` | All pointer-typed places accessible through this place, grouped by region |
| `interior_places(tcx, body, def_id)` | `HashSet<Place>` | All subplaces |
| `interior_paths(tcx, body, def_id)` | `HashSet<Place>` | All paths (including through references) |
| `to_string(tcx, body)` | `Option<String>` | Human-readable place string |
| `normalize(tcx, def_id)` | `Place` | Normalize projections |
| `is_source_visible(tcx, body)` | `bool` | Whether this place corresponds to a source-visible variable |

**Key for graph construction:** `interior_pointers` is the critical method — it maps from a place to the set of borrow origins (RegionVids) of all pointers within it. This directly connects Polonius origins to program places for alias analysis.

#### `SpanExt` — Source Span Extensions
Location: `rustc_utils::source_map::span`

| Method | Returns | Description |
|--------|---------|-------------|
| `subtract(child_spans)` | `Vec<Span>` | Remove child spans from this span |
| `as_local(outer)` | `Option<Span>` | Restrict to only the portion within `outer` |
| `overlaps_inclusive(other)` | `bool` | Whether spans overlap (inclusive of endpoints) |
| `trim_end(other)` | `Option<Span>` | Truncate this span at the start of `other` |
| `merge_overlaps(spans)` | `Vec<Span>` | Merge overlapping spans |
| `trim_leading_whitespace(source_map)` | `Option<Vec<Span>>` | Remove leading whitespace (for display) |
| `to_string(tcx)` | `String` | Source text for this span |
| `size()` | `u32` | Byte length of span |

#### `TyExt` — Type Extensions
Location: `rustc_utils::hir::ty`
Extends `Ty` with type-analysis helpers (specific methods not documented in public API).

#### `AdtDefExt` — ADT (struct/enum/union) Extensions
Location: `rustc_utils::mir::adt_def`
Extends `AdtDef` with helper methods for working with algebraic data types.

#### `MutabilityExt` — Mutability Extensions
Location: `rustc_utils::mir::mutability`

#### `OperandExt` — Operand Extensions
Location: `rustc_utils::mir::operand`
Extends `Operand` (Move, Copy, Constant) with convenience methods.

### 4.5 How Flowistry, Aquascope, Paralegal, and Argus Use These

| Tool | rustc_utils Usage | Graph/Analysis Purpose |
|------|------------------|----------------------|
| **Flowistry** | `PlaceExt::interior_pointers` for alias oracle; `BodyExt::control_dependencies` for CDG; `PlaceExt::refs_in_projection` for borrow traversal | PDG construction: data flow + control flow |
| **Aquascope** | `BodyExt::location_to_hir_id` for source mapping; `PlaceExt::is_source_visible` for filtering | Visual lifetime/borrow display in IDE |
| **Paralegal** | Full PDG from Flowistry; uses all `BodyExt` iteration methods | Security policy enforcement via PDG traversal |
| **Argus** | `BodyExt::async_context` for async fn handling; type utilities | Type-checking aid and proof search |

---

## 5. Charon

**Repository:** https://github.com/AeneasVerif/charon  
**Paper:** "Charon: An Analysis Framework for Rust" (Ho et al., 2024) — https://arxiv.org/html/2410.18042v2  
**Library:** `charon-lib` (Rust), `charon-ml` (OCaml)

### 5.1 What Charon Does

Charon runs as a Rust compiler driver (intercepting compilation via `RUSTC_WRAPPER` / rustc callbacks) and extracts the complete contents of a crate plus all its dependencies into a single JSON file (`crate_name.llbc`). It normalizes and cleans up rustc's MIR into a stable, analysis-friendly format.

**Key design decisions:**
- Works at the MIR level (not HIR/THIR) — ground truth of Rust semantics
- Fully resolves traits, generics, and constants (no lazy queries)
- Provides TWO intermediate representations: ULLBC and LLBC

### 5.2 The Two Intermediate Representations

#### ULLBC — Unstructured Low-Level Borrow Calculus
A cleaned-up CFG that preserves MIR's low-level semantics:
- Moves, copies, explicit borrows/reborrows preserved
- Constants represented uniformly (no rustc Steal abstractions)
- Trait instances fully resolved (no lazy lookup)
- Checked operations (`+` that might overflow) represented as a single statement
- Pattern matches restored from discriminant switches to ML-style patterns
- Panics properly identified

Serialized as `.ullbc` JSON files.

#### LLBC — Low-Level Borrow Calculus
A structured AST reconstructed from ULLBC via:
- Demand-set analysis (control-flow reconstruction)
- Natural loop detection
- goto → while/if/match reconstruction

Serialized as `.llbc` JSON files. This is the primary output format for analyses.

### 5.3 Top-Level LLBC JSON Schema

```json
{
  "crate_name": "my_crate",
  "type_decls": [ /* TypeDecl */ ],
  "fun_decls":  [ /* FunDecl */ ],
  "global_decls": [ /* GlobalDecl */ ],
  "trait_decls": [ /* TraitDecl */ ],
  "trait_impls": [ /* TraitImpl */ ]
}
```

Parse with:
```rust
let f = std::fs::File::open("my_crate.llbc")?;
let crate_data: charon_lib::export::CrateData = serde_json::from_reader(f)?;
```

### 5.4 Key Types in charon-lib

#### `CrateData` (top-level)
Contains all five declaration lists above. Each list uses `AllDeclarations` (an ordered map keyed by strongly-typed IDs).

#### Strongly-Typed IDs (vs rustc's untyped DefId)
Charon replaces rustc's single `DefId` with categorical IDs:

| Charon ID Type | Represents |
|---------------|------------|
| `FunDeclId` | A function declaration |
| `TypeDeclId` | A type (struct/enum/union) declaration |
| `GlobalDeclId` | A global/static declaration |
| `TraitDeclId` | A trait declaration |
| `TraitImplId` | A trait implementation |

All IDs carry a fully-qualified, human-readable name (e.g. `std::option::Option`).

#### `FunDecl` — Function Declaration
Key fields:
- `def_id: FunDeclId`
- `name: Name` — fully qualified path
- `signature: FunSig` — argument types, return type, lifetime params
- `body: Option<FunBody>` — absent for extern/opaque functions

#### `FunBody` — Function Body
Contains:
- `locals: Vec<Var>` — local variable declarations with types
- `body: ExprBody` — the actual LLBC body (either `RawCode` for ULLBC or structured `Statement` for LLBC)

#### LLBC `Statement` — Structured AST Nodes
Key variants:
- `Assign(Place, Rvalue)` — assignment
- `FakeRead(Place)` — NLL fake read
- `SetDiscriminant(Place, VariantIdx)` — enum discriminant
- `Drop(Place)` — drop
- `Assert(Assert)` — runtime assertion
- `Call(Call)` — function call (with fully resolved callee)
- `Sequence(Box<Statement>, Box<Statement>)` — sequential composition
- `Switch(Operand, Switch)` — if/match (reconstructed from CFG)
- `Loop(Box<Statement>)` — loop body
- `Break(usize)` — loop break
- `Continue(usize)` — loop continue
- `Return` — return
- `Panic(...)` — panic

#### `TypeDecl` — Type Declaration
Key fields:
- `def_id: TypeDeclId`
- `name: Name`
- `generics: GenericParams`
- `kind: TypeDeclKind` — `Struct(FieldsVec)`, `Enum(VariantsVec)`, `Union(...)`, `Alias(Ty)`, `Opaque`

#### `Place` in Charon
Same concept as MIR `Place` but with resolved types:
- `var_id: VarId` — root local variable
- `projection: Projection` — list of `ProjectionElem`:
  - `ProjDeref` — dereference
  - `ProjField(FieldProjKind, Ty)` — field access with field kind + type
  - `ProjIndex(Operand, Ty, bool)` — array/slice index

#### `Rvalue` in Charon
- `Use(Operand)` — copy/move operand
- `RvRef(Place, BorrowKind)` — create reference
- `UnaryOp(UnOp, Operand)` — unary operation
- `BinaryOp(BinOp, Operand, Operand)` — binary operation
- `Discriminant(Place, TypeDeclId)` — read discriminant
- `Aggregate(AggregateKind, Vec<Operand>)` — struct/enum/array construction
- `Global(GlobalDeclId)` — reference to global
- `Len(Place, Ty, Option<ConstGeneric>)` — slice length

### 5.5 Graph Edges Preserved in Charon Extraction

| Graph Edge | Charon Representation | Preservation |
|-----------|----------------------|-------------|
| **Function call edges** | `Statement::Call { func: FnPtr, .. }` with fully resolved `FnPtr` | Full — trait dispatch resolved |
| **CFG edges** (control flow) | `Statement::Sequence`, `Switch`, `Loop` structure | Full — CFG reconstructed into AST |
| **Data flow** | `Assign(place, rvalue)` — left side is written, right side is read | Full |
| **Borrow creation** | `Rvalue::RvRef(place, BorrowKind)` | Full |
| **Ownership move** | `Operand::Move(place)` | Full |
| **Field access** (ownership tree) | `Place::projection` with `ProjField(FieldProjKind, ty)` | Full |
| **Type inheritance / ADT structure** | `TypeDecl::kind` with fields/variants | Full |
| **Trait impl resolution** | All calls resolved to concrete impl at extraction time | Full — richer than rustc's lazy dispatch |
| **Lifetime/region info** | Present in types (`Ty::Ref(Region, ..)`) but NOT as Polonius facts | Partial — structural only, no flow analysis |

### 5.6 How to Use Charon as a Frontend for Parseltongue

```bash
# Install
cargo install charon-driver

# Extract (run inside the target crate)
charon

# Output: target_crate.llbc
```

```rust
// Consume in Rust
use charon_lib::export::CrateData;
use charon_lib::ast::*;

fn analyze(path: &str) -> CrateData {
    let f = std::fs::File::open(path).unwrap();
    serde_json::from_reader(f).unwrap()
}

fn build_call_graph(data: &CrateData) {
    for fun_decl in data.fun_decls.iter() {
        // Visit all statements looking for Call nodes
        // ...
    }
}
```

**OCaml usage:**
```ocaml
let crate = Charon.load_crate "my_crate.llbc"
```

### 5.7 Charon Limitations vs. Direct rustc Access

| Aspect | Charon | Direct rustc (via rustc_plugin) |
|--------|--------|--------------------------------|
| **Stability** | Alpha software; API breaks planned | Nightly-pinned but well-understood |
| **Borrow check facts** | NOT available (structural lifetime info only) | Full Polonius facts accessible |
| **Trait resolution** | Fully resolved at extraction | Requires manual `tcx.resolve_instance()` |
| **Abstraction level** | Cleaned up, analysis-friendly AST | Raw MIR with all rustc quirks |
| **Unsafe code** | Incomplete support | Full access |
| **Closures** | Incomplete support | Full MIR access |
| **Dynamic dispatch** | Not fully supported | Accessible via `TerminatorKind::Call` |
| **Interior mutability** | Not supported | Accessible (but semantically complex) |
| **Concurrency** | Not supported | Accessible |
| **Dependency crates** | Included in single output file | Requires cross-crate analysis setup |
| **Performance** | ~80s extraction for 20KLOC crate | Incremental with cargo |
| **Language subset** | Safe, sequential, trait bounds | Full Rust (with caveats for `unsafe`) |

---

## 6. Summary: Graph Construction Mapping

This section maps each Parseltongue graph concept to the best API source.

### Entity Nodes

| Entity | Best Source | API Path | Trust Level |
|--------|------------|----------|-------------|
| **Function** | Stable MIR | `all_local_items()` filtered by `ItemKind::Fn` | Compiler-verified |
| **Type** (struct/enum/union) | Stable MIR or Charon | `rustc_public::ty::TyKind::Adt` or `CrateData::type_decls` | Compiler-verified |
| **Variable** (local) | Stable MIR | `body.local_decls[local]` | Compiler-verified |
| **Place** (memory location) | Stable MIR | `Place { local, projection }` | Compiler-verified |
| **Loan** (borrow) | Polonius | `loan_issued_at(origin, loan, point)` | Compiler-verified |
| **Origin** (lifetime set) | Polonius | `AllFacts::placeholder`, `subset_base` origins | Compiler-verified |
| **Trait** | Stable MIR | `all_trait_decls()`, `all_trait_impls()` | Compiler-verified |

### Graph Edges

| Edge Type | Best Source | API / Fact | Trust Level |
|-----------|------------|------------|-------------|
| **Call edge** (fn A calls fn B) | Stable MIR | `TerminatorKind::Call { func: Operand::Constant(FnDef) }` | Compiler-verified |
| **Control flow** (BB → BB) | Stable MIR | `TerminatorKind::{Goto, SwitchInt, Call, Drop}` successors | Compiler-verified |
| **Ownership move** | Stable MIR | `Rvalue::Use(Operand::Move(place))` | Compiler-verified |
| **Borrow creation** | Stable MIR + Polonius | `Rvalue::Ref(..)` + `loan_issued_at` | Compiler-verified |
| **Subset / outlives** | Polonius | `subset_base(O1, O2, P)` | Compiler-verified |
| **Loan liveness** | Polonius | `origin_contains_loan_on_entry` (derived) | Compiler-verified |
| **Data flow** (var A affects var B) | Flowistry | `infoflow::compute_flow(tcx, body, def_id)` | Compiler-verified (sound, 94% precise) |
| **Control dependency** (A controls B) | rustc_utils | `BodyExt::control_dependencies()` | Compiler-verified |
| **Field access** (struct traversal) | Stable MIR | `Place::projection` with `ProjectionElem::Field` | Compiler-verified |
| **Deref** (pointer follow) | Stable MIR | `Place::projection` with `ProjectionElem::Deref` | Compiler-verified |
| **Alias** (p1 may-alias p2) | rustc_utils | `PlaceExt::interior_pointers(tcx, body, def_id)` | Borrow-checker sound |
| **Interior pointer** (field → region) | rustc_utils | `PlaceExt::interior_pointers` → `RegionVid` → `loan_issued_at` | Borrow-checker sound |
| **Variable def** | Polonius | `var_defined_at(var, point)` | Compiler-verified |
| **Variable use** | Polonius | `var_used_at(var, point)` | Compiler-verified |
| **Path assignment** | Polonius | `path_assigned_at_base(path, point)` | Compiler-verified |
| **Path move** | Polonius | `path_moved_at_base(path, point)` | Compiler-verified |
| **Trait impl** | Stable MIR | `all_trait_impls()` | Compiler-verified |

### Recommended Tool Stack for Parseltongue

| Layer | Tool | Role |
|-------|------|------|
| **Foundation** | `rustc_plugin` (Brown) | Cargo integration, compiler entry point |
| **MIR traversal** | `rustc_public` / Stable MIR | Stable API for function bodies, types, CFG |
| **Ownership/alias** | Polonius facts (`-Znll-facts`) | Borrow lifetimes, loan flow, move semantics |
| **Data flow** | Flowistry (`infoflow::compute_flow`) | Full PDG with data + control dependencies |
| **MIR utilities** | `rustc_utils` (BodyExt, PlaceExt) | Control dependencies, place analysis, span mapping |
| **Alternative frontend** | Charon | When full-crate JSON export is preferred over in-process analysis |

### Trust Level Legend

| Level | Meaning |
|-------|---------|
| **Compiler-verified** | Computed by rustc's type/borrow checker; mathematically sound |
| **Borrow-checker sound** | Uses borrow-checker facts; may over-approximate (sound but not always precise) |
| **Heuristic** | Pattern-matching or syntactic analysis; may miss cases |

---

## References

- [Polonius repository](https://github.com/rust-lang/polonius) — rust-lang/polonius
- [Polonius book: Relations](https://rust-lang.github.io/polonius/rules/relations.html) — official relation documentation
- [PoloniusFacts rustc docs](https://doc.rust-lang.org/beta/nightly-rustc/rustc_borrowck/polonius/legacy/facts/type.PoloniusFacts.html) — complete fact schema
- [Polonius revisited (Niko Matsakis, 2023)](https://smallcultfollowing.com/babysteps/blog/2023/09/22/polonius-part-1/)
- [Polonius update blog (Inside Rust, 2023)](https://blog.rust-lang.org/inside-rust/2023/10/06/polonius-update.html)
- [Flowistry paper](https://arxiv.org/abs/2111.13662) — "Modular Information Flow through Ownership"
- [Flowistry docs](https://willcrichton.net/flowistry/flowistry/)
- [rustc_public docs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/index.html)
- [rustc_public::mir docs](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_public/mir/index.html)
- [CrateItem docs](https://doc.rust-lang.org/stable/nightly-rustc/rustc_public/struct.CrateItem.html)
- [Kani StableMIR migration](https://model-checking.github.io/kani/stable-mir.html)
- [GSoC 2025 rustc_public report](https://makai410.dev/posts/gsoc-25-final/)
- [rustc_plugin repo](https://github.com/cognitive-engineering-lab/rustc_plugin)
- [RustcPlugin trait docs](https://willcrichton.net/flowistry/rustc_plugin/trait.RustcPlugin.html)
- [BodyExt trait docs](https://willcrichton.net/flowistry/rustc_utils/mir/body/trait.BodyExt.html)
- [PlaceExt trait docs](https://willcrichton.net/flowistry/rustc_utils/mir/place/trait.PlaceExt.html)
- [SpanExt trait docs](https://willcrichton.net/flowistry/rustc_utils/source_map/span/trait.SpanExt.html)
- [Charon repository](https://github.com/AeneasVerif/charon)
- [Charon paper](https://arxiv.org/html/2410.18042v2) — "Charon: An Analysis Framework for Rust"
- [CHARON/AENEAS pipeline overview](https://www.emergentmind.com/topics/charon-aeneas-pipeline)
