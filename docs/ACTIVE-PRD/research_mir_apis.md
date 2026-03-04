# Rust Compiler MIR APIs: Control Flow & Data Flow Analysis Reference

**Source**: `github.com/rust-lang/rust` (main branch, March 2026)  
**Crates covered**: `rustc_middle` (mir), `rustc_mir_dataflow`, `rustc_data_structures`  
**Purpose**: Complete API reference for building a code-graph via MIR control flow and data flow analysis

---

## Table of Contents

1. [MIR Body (`rustc_middle/src/mir/mod.rs`)](#1-mir-body)
2. [Control Flow Graph](#2-control-flow-graph)
3. [Data Flow — Places, Operands, Rvalues](#3-data-flow--places-operands-rvalues)
4. [MIR Dataflow Framework (`rustc_mir_dataflow`)](#4-mir-dataflow-framework)
5. [Control Flow Utilities — Dominators, Loops](#5-control-flow-utilities)
6. [Call Graph Construction](#6-call-graph-construction)
7. [MIR Visitors](#7-mir-visitors)

---

## 1. MIR Body

**Source**: `compiler/rustc_middle/src/mir/mod.rs`  
**GitHub**: https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/mir/mod.rs

### Getting a MIR Body

```rust
// Optimized, post-inlining MIR (runtime code)
let body: &Body<'tcx> = tcx.optimized_mir(def_id);

// Pre-optimization MIR (before any passes)
let body: &Body<'tcx> = tcx.mir_built(def_id).borrow();

// MIR for a monomorphized instance (handles generics)
let body: &Body<'tcx> = tcx.instance_mir(instance.def);

// Promoted constants (inline constants lifted to statics)
let body: &Body<'tcx> = tcx.promoted_mir(def_id)[promoted_index];
```

### `Body<'tcx>` struct

```rust
pub struct Body<'tcx> {
    pub basic_blocks: BasicBlocks<'tcx>,         // CFG: all basic blocks
    pub source_scopes: IndexVec<SourceScope, SourceScopeData<'tcx>>,
    pub local_decls: IndexVec<Local, LocalDecl<'tcx>>,
    pub user_type_annotations: CanonicalUserTypeAnnotations<'tcx>,
    pub arg_count: usize,                         // args are locals 1..=arg_count
    pub spread_arg: Option<Local>,
    pub var_debug_info: Vec<VarDebugInfo<'tcx>>,
    pub span: Span,
    pub required_consts: Vec<ConstOperand<'tcx>>,
    pub is_polymorphic: bool,
    pub injection_phase: Option<MirPhase>,
    pub tainted_by_errors: Option<ErrorGuaranteed>,
    pub coroutine: Option<Box<CoroutineInfo<'tcx>>>,
    pub phase: MirPhase,
    pub pass_count: usize,
    pub source: MirSource<'tcx>,
}
```

### Key `Body<'tcx>` Methods

```rust
impl<'tcx> Body<'tcx> {
    // Local variable iteration
    pub fn local_kind(&self, local: Local) -> LocalKind
    pub fn args_iter(&self) -> impl Iterator<Item = Local>         // _1.._arg_count
    pub fn vars_and_temps_iter(&self) -> impl Iterator<Item = Local>
    pub fn return_ty(&self) -> Ty<'tcx>
    
    // Location helpers
    pub fn terminator_loc(&self, bb: BasicBlock) -> Location
    pub fn stmt_at(&self, loc: Location) -> Either<&Statement<'tcx>, &Terminator<'tcx>>
    pub fn locations(&self) -> impl Iterator<Item = Location>      // all statement+terminator locations
    
    // Indexing: body[bb] -> &BasicBlockData<'tcx>
    // body.basic_blocks.len() -> number of blocks
}
```

### Special Locals

```rust
// Return place is always local 0
pub const RETURN_PLACE: Local = Local::from_u32(0);
// Argument locals: 1..=body.arg_count
// Temporaries and user variables: body.arg_count+1..
```

### `LocalDecl<'tcx>` struct

```rust
pub struct LocalDecl<'tcx> {
    pub mutability: Mutability,
    pub local_info: ClearCrossCrate<Box<LocalInfo<'tcx>>>,
    pub ty: Ty<'tcx>,
    pub user_ty: Option<Box<UserTypeProjections>>,
    pub source_info: SourceInfo,  // { span, scope: SourceScope }
}
```

### `LocalInfo<'tcx>` variants

```rust
pub enum LocalInfo<'tcx> {
    User(BindingForm<'tcx>),                    // user-declared variable
    StaticRef { def_id: DefId, is_thread_local: bool },
    ConstRef { def_id: DefId },
    AggregateTemp,                              // temporary for aggregate construction
    BlockTailTemp(BlockTailInfo),
    DerefTemp,                                  // temporary for deref coercion
    FakeBorrow,                                 // two-phase borrow placeholder
    Boring,                                     // compiler-generated temp
}
```

### `LocalKind` enum

```rust
pub enum LocalKind {
    Temp,           // compiler-generated temporary
    Arg,            // function argument (locals 1..=arg_count)
    ReturnPointer,  // local 0
}
```

### `SourceScopeData<'tcx>` struct

```rust
pub struct SourceScopeData<'tcx> {
    pub span: Span,
    pub parent_scope: Option<SourceScope>,
    pub inlined: Option<(Instance<'tcx>, Span)>,   // present if inlined from another fn
    pub inlined_parent_scope: Option<SourceScope>,
    pub local_data: ClearCrossCrate<SourceScopeLocalData>,
}
```

### `MirPhase` enum

```rust
pub enum MirPhase {
    Built,
    Analysis(AnalysisPhase),
    Runtime(RuntimePhase),
}
```

---

## 2. Control Flow Graph

**Sources**:  
- `compiler/rustc_middle/src/mir/mod.rs` — BasicBlock, BasicBlockData  
- `compiler/rustc_middle/src/mir/basic_blocks.rs` — BasicBlocks struct  
- `compiler/rustc_middle/src/mir/syntax.rs` — Statement, Terminator  
- `compiler/rustc_middle/src/mir/terminator.rs` — Terminator methods  
- `compiler/rustc_middle/src/mir/traversal.rs` — traversal utilities

### `BasicBlock` and `BasicBlockData`

```rust
// BasicBlock is a newtype index: rustc_index::newtype_index!
// body[bb] -> &BasicBlockData<'tcx>

pub struct BasicBlockData<'tcx> {
    pub statements: Vec<Statement<'tcx>>,
    pub terminator: Option<Terminator<'tcx>>,
    pub is_cleanup: bool,    // true if this is an unwind cleanup block
}

impl<'tcx> BasicBlockData<'tcx> {
    pub fn terminator(&self) -> &Terminator<'tcx>
    pub fn terminator_mut(&mut self) -> &mut Terminator<'tcx>
    pub fn retain_statements<F: FnMut(&mut Statement<'tcx>) -> bool>(&mut self, f: F)
    pub fn expand_statements<F, I>(&mut self, f: F)
}
```

### `BasicBlocks<'tcx>` struct

```rust
// Wraps IndexVec<BasicBlock, BasicBlockData<'tcx>> + lazy CFG caches
pub struct BasicBlocks<'tcx> {
    basic_blocks: IndexVec<BasicBlock, BasicBlockData<'tcx>>,
    cache: Cache,  // holds predecessors, dominators, reverse_postorder — lazily computed
}

impl<'tcx> BasicBlocks<'tcx> {
    // CFG structure (lazily cached)
    pub fn dominators(&self) -> &Dominators<BasicBlock>
    pub fn predecessors(&self) -> &Predecessors
    //   Predecessors = IndexVec<BasicBlock, SmallVec<[BasicBlock; 4]>>
    pub fn reverse_postorder(&self) -> &[BasicBlock]
    
    // Mutation — invalidates cache
    pub fn as_mut(&mut self) -> &mut IndexVec<BasicBlock, BasicBlockData<'tcx>>
    
    // Mutation — preserves CFG cache (safe only if structure doesn't change)
    pub fn as_mut_preserves_cfg(&mut self) -> &mut IndexVec<BasicBlock, BasicBlockData<'tcx>>
    
    pub fn invalidate_cfg_cache(&mut self)
    pub fn len(&self) -> usize
    pub fn iter_enumerated(&self) -> impl Iterator<Item = (BasicBlock, &BasicBlockData<'tcx>)>
    
    // Implements graph traits
    // graph::DirectedGraph, graph::Successors, graph::Predecessors
}
```

### `Statement<'tcx>` struct

```rust
pub struct Statement<'tcx> {
    pub source_info: SourceInfo,
    pub kind: StatementKind<'tcx>,
}
```

### `StatementKind<'tcx>` variants

```rust
pub enum StatementKind<'tcx> {
    Assign(Box<(Place<'tcx>, Rvalue<'tcx>)>),
    FakeRead(Box<(FakeReadCause, Place<'tcx>)>),
    SetDiscriminant { place: Box<Place<'tcx>>, variant_index: VariantIdx },
    StorageLive(Local),
    StorageDead(Local),
    Retag(RetagKind, Box<Place<'tcx>>),
    PlaceMention(Box<Place<'tcx>>),
    AscribeUserType(Box<(Place<'tcx>, UserTypeProjection)>, Variance),
    Coverage(CoverageKind),
    Intrinsic(Box<NonDivergingIntrinsic<'tcx>>),
    ConstEvalCounter,
    Nop,
    BackwardIncompatibleDropHint { place: Box<Place<'tcx>>, reason: BackwardIncompatibleDropReason },
}

// Key helpers:
impl<'tcx> StatementKind<'tcx> {
    pub fn as_assign(&self) -> Option<&(Place<'tcx>, Rvalue<'tcx>)>
}
```

### `Terminator<'tcx>` struct

```rust
pub struct Terminator<'tcx> {
    pub source_info: SourceInfo,
    pub kind: TerminatorKind<'tcx>,
}
```

### `TerminatorKind<'tcx>` variants

```rust
pub enum TerminatorKind<'tcx> {
    // Unconditional jump
    Goto { target: BasicBlock },
    
    // Conditional branch (switch on integer value)
    SwitchInt { discr: Operand<'tcx>, targets: SwitchTargets },
    
    // Unwind paths
    UnwindResume,
    UnwindTerminate(UnwindTerminateReason),
    
    // Normal exit
    Return,
    Unreachable,
    
    // Drop a place (may call destructor)
    Drop {
        place: Place<'tcx>,
        target: BasicBlock,
        unwind: UnwindAction,
        replace: bool,
        drop: Option<BasicBlock>,           // custom drop shim block
        async_fut: Option<BasicBlock>,      // async drop block
    },
    
    // Function call — most important for call graph
    Call {
        func: Operand<'tcx>,                // the function being called
        args: Box<[Spanned<Operand<'tcx>>]>,
        destination: Place<'tcx>,           // where the return value goes
        target: Option<BasicBlock>,         // Some(bb) = normal return, None = diverging
        unwind: UnwindAction,
        call_source: CallSource,
        fn_span: Span,
    },
    
    // Tail call (no return target — calling convention optimization)
    TailCall {
        func: Operand<'tcx>,
        args: Box<[Spanned<Operand<'tcx>>]>,
        fn_span: Span,
    },
    
    // Assertion (e.g. bounds check, overflow check)
    Assert {
        cond: Operand<'tcx>,
        expected: bool,
        msg: Box<AssertMessage<'tcx>>,
        target: BasicBlock,
        unwind: UnwindAction,
    },
    
    // Coroutine yield
    Yield {
        value: Operand<'tcx>,
        resume: BasicBlock,
        resume_arg: Place<'tcx>,
        drop: Option<BasicBlock>,
    },
    CoroutineDrop,
    
    // Analysis artifacts (not real CFG edges)
    FalseEdge { real_target: BasicBlock, imaginary_target: BasicBlock },
    FalseUnwind { real_target: BasicBlock, unwind: UnwindAction },
    
    // Inline assembly
    InlineAsm {
        template: &'tcx [InlineAsmTemplatePiece],
        operands: Box<[InlineAsmOperand<'tcx>]>,
        options: InlineAsmOptions,
        line_spans: &'tcx [Span],
        targets: Box<[BasicBlock]>,
        unwind: UnwindAction,
    },
}
```

### `Terminator<'tcx>` Methods

```rust
impl<'tcx> Terminator<'tcx> {
    // Iterate all successor basic blocks
    pub fn successors(&self) -> Successors<'_>
    //   Successors implements DoubleEndedIterator<Item = BasicBlock>
    
    // Mutable successor iteration (for rewriting)
    pub fn successors_mut(&mut self, f: impl FnMut(&mut BasicBlock))
    
    // Unwind action if any
    pub fn unwind(&self) -> Option<&UnwindAction>
    pub fn unwind_mut(&mut self) -> Option<&mut UnwindAction>
    
    // Typed edge description (used by dataflow framework)
    pub fn edges(&self) -> TerminatorEdges<'_, 'tcx>
}

pub enum TerminatorEdges<'mir, 'tcx> {
    None,                                   // Return, Unreachable
    Single(BasicBlock),                     // Goto, Assert (success), Drop (no unwind)
    Double(BasicBlock, BasicBlock),         // two distinct successors
    AssignOnReturn {
        return_: Box<[BasicBlock]>,
        cleanup: Option<BasicBlock>,
        place: CallReturnPlaces<'mir, 'tcx>,
    },
    SwitchInt {
        targets: &'mir SwitchTargets,
        discr: &'mir Operand<'tcx>,
    },
}
```

### `SwitchTargets`

```rust
pub struct SwitchTargets {
    values: SmallVec<[u128; 1]>,
    targets: SmallVec<[BasicBlock; 2]>,
    // targets[0..n-1] correspond to values[0..n-1]
    // targets[n] is the "otherwise" branch (fallthrough)
}

impl SwitchTargets {
    // Iterate (value, target_bb) pairs (not including otherwise)
    pub fn iter(&self) -> impl Iterator<Item = (u128, BasicBlock)>
    
    // The fallthrough / otherwise branch
    pub fn otherwise(&self) -> BasicBlock
    
    // Lookup target for a specific integer value
    pub fn target_for_value(&self, value: u128) -> BasicBlock
    
    // All targets including otherwise
    pub fn all_targets(&self) -> &[BasicBlock]
    pub fn all_targets_mut(&mut self) -> &mut [BasicBlock]
    
    // All values (does NOT include otherwise)
    pub fn all_values(&self) -> &[SwitchTargetValue]
    
    // Single target: true if all branches point to same block
    pub fn as_static_if(&self) -> Option<(u128, BasicBlock, BasicBlock)>
}
```

### `Location` struct

```rust
pub struct Location {
    pub block: BasicBlock,
    pub statement_index: usize,
    // statement_index == block.statements.len() means the terminator
}

impl Location {
    pub const START: Location = Location { block: START_BLOCK, statement_index: 0 };
    pub fn is_before_entry(&self) -> bool
    pub fn dominates(&self, other: Location, dominators: &Dominators<BasicBlock>) -> bool
    pub fn successor_within_block(&self) -> Location
}
```

### CFG Traversal (`compiler/rustc_middle/src/mir/traversal.rs`)

```rust
use rustc_middle::mir::traversal;

// DFS preorder (discovery order, suitable for forward analysis seed)
pub fn preorder(body: &Body<'_>) -> Preorder<'_, '_>
// also: pub fn reachable(body) -> Preorder  (same thing)

// DFS postorder (completion order)
pub fn postorder(body: &Body<'_>) -> Postorder<'_, '_>

// Reverse postorder — canonical linear order for forward dataflow
pub fn reverse_postorder(body: &Body<'_>)
    -> impl Iterator<Item = (BasicBlock, &BasicBlockData<'_>)>
// (uses body.basic_blocks.reverse_postorder() cache)

// Reachability as bitset — O(1) membership test
pub fn reachable_as_bitset(body: &Body<'_>) -> DenseBitSet<BasicBlock>

// Monomorphic reachability — skips dead SwitchInt branches for known instance
pub fn mono_reachable<'a, 'tcx>(
    body: &'a Body<'tcx>,
    tcx: TyCtxt<'tcx>,
    instance: Instance<'tcx>,
) -> MonoReachable<'a, 'tcx>
```

### START_BLOCK constant

```rust
pub const START_BLOCK: BasicBlock = BasicBlock::from_u32(0);
```

---

## 3. Data Flow — Places, Operands, Rvalues

**Source**: `compiler/rustc_middle/src/mir/syntax.rs`  
**GitHub**: https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/mir/syntax.rs

### `Place<'tcx>` struct

```rust
pub struct Place<'tcx> {
    pub local: Local,
    pub projection: &'tcx List<PlaceElem<'tcx>>,
    // Empty projection = just the local (base case)
}

// PlaceElem<'tcx> = ProjectionElem<Local, Ty<'tcx>>
pub enum ProjectionElem<V, T> {
    Deref,                                  // *place
    Field(FieldIdx, T),                     // place.field (with type T)
    Index(V),                               // place[local]
    ConstantIndex { offset: u64, min_length: u64, from_end: bool },
    Subslice { from: u64, to: u64, from_end: bool },
    Downcast(Option<Symbol>, VariantIdx),   // place as Variant
    OpaqueCast(T),
    UnwrapUnsafeBinder(T),
}
```

### Key `Place<'tcx>` methods

```rust
impl<'tcx> Place<'tcx> {
    pub fn from(local: Local) -> Self       // construct bare local place
    pub fn as_local(&self) -> Option<Local> // Some if no projections
    pub fn as_ref(&self) -> PlaceRef<'tcx>
    pub fn is_indirect(&self) -> bool       // true if any Deref in projection
    pub fn is_indirect_first_projection(&self) -> bool
    pub fn local_or_deref_local(&self) -> Option<Local>
    pub fn ty<D: HasLocalDecls<'tcx>>(&self, local_decls: &D, tcx: TyCtxt<'tcx>) -> PlaceTy<'tcx>
    pub fn iter_projections(&self) -> impl Iterator<Item = (PlaceRef<'tcx>, PlaceElem<'tcx>)>
    pub fn project_deeper(&self, more_projections: &[PlaceElem<'tcx>], tcx: TyCtxt<'tcx>) -> Self
}

pub struct PlaceTy<'tcx> {
    pub ty: Ty<'tcx>,
    pub variant_index: Option<VariantIdx>,
}
impl<'tcx> PlaceTy<'tcx> {
    pub fn projection_ty(&self, tcx: TyCtxt<'tcx>, elem: PlaceElem<'tcx>) -> PlaceTy<'tcx>
}
```

### `Operand<'tcx>` enum

```rust
pub enum Operand<'tcx> {
    Copy(Place<'tcx>),              // reads the place (does NOT move out)
    Move(Place<'tcx>),              // moves out of the place
    Constant(Box<ConstOperand<'tcx>>),
    RuntimeChecks(RuntimeChecks),   // instrumentation artifact
}

impl<'tcx> Operand<'tcx> {
    pub fn place(&self) -> Option<Place<'tcx>>    // Some for Copy/Move
    pub fn constant(&self) -> Option<&ConstOperand<'tcx>>
    pub fn is_move(&self) -> bool
    pub fn ty<D: HasLocalDecls<'tcx>>(&self, local_decls: &D, tcx: TyCtxt<'tcx>) -> Ty<'tcx>
    pub fn to_copy(&self) -> Self   // converts Move -> Copy
    pub fn function_handle(tcx, def_id, args, span) -> Self  // make fn ptr operand
}

pub struct ConstOperand<'tcx> {
    pub span: Span,
    pub user_ty: Option<UserTypeAnnotationIndex>,
    pub const_: Const<'tcx>,
}
```

### `Rvalue<'tcx>` enum

```rust
pub enum Rvalue<'tcx> {
    Use(Operand<'tcx>),                             // simple use/copy/move
    Repeat(Operand<'tcx>, Const<'tcx>),             // [operand; count]
    Ref(Region<'tcx>, BorrowKind, Place<'tcx>),     // &place or &mut place
    RawPtr(RawPtrKind, Place<'tcx>),                // &raw const/mut place
    Len(Place<'tcx>),                               // slice/array length
    Cast(CastKind, Operand<'tcx>, Ty<'tcx>),        // type cast
    BinaryOp(BinOp, Box<(Operand<'tcx>, Operand<'tcx>)>),
    UnaryOp(UnOp, Operand<'tcx>),
    Discriminant(Place<'tcx>),                      // discriminant value of enum
    Aggregate(Box<AggregateKind<'tcx>>, IndexVec<FieldIdx, Operand<'tcx>>),
    CopyForDeref(Place<'tcx>),                      // deref coercion copy
    ThreadLocalRef(DefId),                          // &thread_local!
    WrapUnsafeBinder(Operand<'tcx>, Ty<'tcx>),
}

pub enum BorrowKind {
    Shared,
    Fake(FakeBorrowKind),
    Mut { kind: MutBorrowKind },
}

pub enum AggregateKind<'tcx> {
    Array(Ty<'tcx>),
    Tuple,
    Adt(DefId, VariantIdx, GenericArgsRef<'tcx>, Option<UserTypeAnnotationIndex>, Option<FieldIdx>),
    Closure(DefId, GenericArgsRef<'tcx>),
    Coroutine(DefId, GenericArgsRef<'tcx>),
    CoroutineClosure(DefId, GenericArgsRef<'tcx>),
    RawPtr(Ty<'tcx>, Mutability),
}
```

### Reads and Writes — Tracking Def-Use

For def-use analysis, use `PlaceContext` from the MIR visitor (see §7):

```rust
// Reading a place:
//   Operand::Copy(place)  -> visit_place with NonMutatingUse(Copy)
//   Operand::Move(place)  -> visit_place with NonMutatingUse(Move)
//   Rvalue::Ref(_, _, place) -> visit_place with NonMutatingUse(SharedBorrow) or MutatingUse(Borrow)
//   Rvalue::Discriminant(place) -> NonMutatingUse(Inspect)

// Writing to a place:
//   StatementKind::Assign((lhs, _)) -> visit_place with MutatingUse(Store)
//   TerminatorKind::Call { destination, .. } -> MutatingUse(Call)
//   TerminatorKind::Yield { resume_arg, .. } -> MutatingUse(Yield)

// Def-Use classification via liveness.rs (DefUse enum):
pub enum DefUse {
    Def,           // full write to local (no projection, or projection through pointer)
    Use,           // any read of local
    PartialWrite,  // write to projection of local (still a use for liveness)
    NonUse,        // debug info only
}

impl DefUse {
    pub fn for_place(place: Place<'_>, context: PlaceContext) -> DefUse
    pub fn apply(state: &mut DenseBitSet<Local>, place: Place<'_>, context: PlaceContext)
}
```

### `CallReturnPlaces<'_, 'tcx>` — Places written by calls

```rust
pub enum CallReturnPlaces<'a, 'tcx> {
    Call(Place<'tcx>),
    Yield(Place<'tcx>),
    InlineAsm(&'a [InlineAsmOperand<'tcx>]),
}

impl<'tcx> CallReturnPlaces<'_, 'tcx> {
    pub fn for_each(&self, f: impl FnMut(Place<'tcx>))
}
```

---

## 4. MIR Dataflow Framework

**Source**: `compiler/rustc_mir_dataflow/src/framework/mod.rs`  
**GitHub**: https://github.com/rust-lang/rust/blob/main/compiler/rustc_mir_dataflow/src/framework/mod.rs

### Overview

The framework is a fixpoint iterator over the CFG. It supports both **forward** and **backward** analyses. The domain is any type implementing `JoinSemiLattice`. Instantiation pattern:

```rust
use rustc_mir_dataflow::Analysis;  // brings iterate_to_fixpoint into scope

let results = MyAnalysis::new(...)
    .iterate_to_fixpoint(tcx, body, Some("pass_name"))
    .into_results_cursor(body);
```

### `Analysis<'tcx>` trait

```rust
pub trait Analysis<'tcx> {
    // Required associated types
    type Domain: Clone + JoinSemiLattice;
    type Direction: Direction = Forward;       // default: forward analysis
    type SwitchIntData = !;                    // set to a type only if you need per-edge SwitchInt effects

    // Required associated const
    const NAME: &'static str;                 // short identifier, used for debugging

    // Required methods
    fn bottom_value(&self, body: &mir::Body<'tcx>) -> Self::Domain;
    fn initialize_start_block(&self, body: &mir::Body<'tcx>, state: &mut Self::Domain);
    fn apply_primary_statement_effect(
        &self,
        state: &mut Self::Domain,
        statement: &mir::Statement<'tcx>,
        location: Location,
    );

    // Optional methods (all have default no-op implementations)
    fn apply_early_statement_effect(
        &self,
        _state: &mut Self::Domain,
        _statement: &mir::Statement<'tcx>,
        _location: Location,
    ) {}
    fn apply_early_terminator_effect(
        &self,
        _state: &mut Self::Domain,
        _terminator: &mir::Terminator<'tcx>,
        _location: Location,
    ) {}
    fn apply_primary_terminator_effect<'mir>(
        &self,
        _state: &mut Self::Domain,
        terminator: &'mir mir::Terminator<'tcx>,
        _location: Location,
    ) -> TerminatorEdges<'mir, 'tcx> {
        terminator.edges()   // default: just propagate edges unchanged
    }
    fn apply_call_return_effect(
        &self,
        _state: &mut Self::Domain,
        _block: BasicBlock,
        _return_places: CallReturnPlaces<'_, 'tcx>,
    ) {}
    fn get_switch_int_data(
        &self,
        _block: mir::BasicBlock,
        _discr: &mir::Operand<'tcx>,
    ) -> Option<Self::SwitchIntData> { None }
    fn apply_switch_int_edge_effect(
        &self,
        _data: &mut Self::SwitchIntData,
        _state: &mut Self::Domain,
        _value: SwitchTargetValue,
        _targets: &mir::SwitchTargets,
    ) { unreachable!() }

    // Extension method — do not override
    fn iterate_to_fixpoint<'mir>(
        self,
        tcx: TyCtxt<'tcx>,
        body: &'mir mir::Body<'tcx>,
        pass_name: Option<&'static str>,
    ) -> Results<'tcx, Self>
    where
        Self: Sized,
        Self::Domain: DebugWithContext<Self>;
}
```

### `JoinSemiLattice` and `MaybeReachable`

```rust
pub trait JoinSemiLattice {
    // Join other into self; returns true if self changed
    fn join(&mut self, other: &Self) -> bool;
}

// Wrapper that adds an Unreachable "bottom" element
pub enum MaybeReachable<S> {
    Unreachable,
    Reachable(S),
}
// Implements JoinSemiLattice, GenKill<T>
```

### Direction markers

```rust
pub struct Forward;
pub struct Backward;

// Used as Analysis::Direction. The framework iterates in RPO for Forward,
// and reverse of RPO for Backward.
```

### `GenKill<T>` trait

```rust
pub trait GenKill<T> {
    fn gen_(&mut self, elem: T);              // add element (set bit)
    fn kill(&mut self, elem: T);              // remove element (clear bit)
    fn gen_all(&mut self, elems: impl IntoIterator<Item = T>);
    fn kill_all(&mut self, elems: impl IntoIterator<Item = T>);
}

// Implementations: DenseBitSet<T>, MixedBitSet<T>, MaybeReachable<S>
```

### `Results<'tcx, A>` struct

```rust
pub struct Results<'tcx, A: Analysis<'tcx>> {
    pub analysis: A,
    pub entry_states: IndexVec<BasicBlock, A::Domain>,  // domain value at block entry
}

impl<'tcx, A: Analysis<'tcx>> Results<'tcx, A> {
    pub fn into_results_cursor<'mir>(self, body: &'mir Body<'tcx>) -> ResultsCursor<'mir, 'tcx, A>
    pub fn borrow_results_cursor<'mir>(&'mir self, body: &'mir Body<'tcx>) -> ResultsCursor<'mir, 'tcx, A>
    pub fn entry_state_for_block(&self, block: BasicBlock) -> &A::Domain
}
```

### `ResultsCursor<'mir, 'tcx, A>` struct

```rust
// Source: compiler/rustc_mir_dataflow/src/framework/cursor.rs
// Random-access inspection of dataflow results at any MIR location.
pub struct ResultsCursor<'mir, 'tcx, A: Analysis<'tcx>> { /* private */ }

impl<'mir, 'tcx, A: Analysis<'tcx>> ResultsCursor<'mir, 'tcx, A> {
    // Construction
    pub fn new_owning(body: &'mir mir::Body<'tcx>, results: Results<'tcx, A>) -> Self
    pub fn new_borrowing(body: &'mir mir::Body<'tcx>, results: &'mir Results<'tcx, A>) -> Self
    
    // Query current state
    pub fn get(&self) -> &A::Domain
    pub fn body(&self) -> &'mir mir::Body<'tcx>
    pub fn analysis(&self) -> &A
    
    // Seek to a block boundary (O(1) + cost of re-applying effects)
    pub fn seek_to_block_start(&mut self, block: BasicBlock)
    // Forward: state before first statement
    // Backward: state propagated to predecessors (ignoring edge effects)
    
    pub fn seek_to_block_end(&mut self, block: BasicBlock)
    // Forward: state propagated to successors (ignoring edge effects)
    // Backward: state before terminator
    
    // Seek to just before a statement's primary effect
    pub fn seek_before_primary_effect(&mut self, target: Location)
    // Early effect IS applied; primary is NOT

    // Seek to just after a statement's primary effect
    pub fn seek_after_primary_effect(&mut self, target: Location)
    // Both early and primary effects are applied

    // Apply a custom effect without seeking (marks state as needing reset)
    pub fn apply_custom_effect(&mut self, f: impl FnOnce(&A, &mut A::Domain))
}
```

### `ResultsVisitor` trait

```rust
pub trait ResultsVisitor<'tcx, A: Analysis<'tcx>> {
    type Domain = A::Domain;
    fn visit_block_start(&mut self, state: &A::Domain) {}
    fn visit_block_end(&mut self, state: &A::Domain) {}
    fn visit_statement_before_primary_effect(
        &mut self, results: &mut Results<'tcx, A>,
        state: &A::Domain, statement: &mir::Statement<'tcx>, location: Location,
    ) {}
    fn visit_statement_after_primary_effect(
        &mut self, results: &mut Results<'tcx, A>,
        state: &A::Domain, statement: &mir::Statement<'tcx>, location: Location,
    ) {}
    fn visit_terminator_before_primary_effect(
        &mut self, results: &mut Results<'tcx, A>,
        state: &A::Domain, terminator: &mir::Terminator<'tcx>, location: Location,
    ) {}
    fn visit_terminator_after_primary_effect(
        &mut self, results: &mut Results<'tcx, A>,
        state: &A::Domain, terminator: &mir::Terminator<'tcx>, location: Location,
    ) {}
}

// Visiting functions
pub fn visit_results<'tcx, A>(
    body: &mir::Body<'tcx>,
    blocks: impl IntoIterator<Item = BasicBlock>,
    results: &mut Results<'tcx, A>,
    vis: &mut impl ResultsVisitor<'tcx, A>,
)
pub fn visit_reachable_results<'tcx, A>(
    body: &mir::Body<'tcx>,
    results: &mut Results<'tcx, A>,
    vis: &mut impl ResultsVisitor<'tcx, A>,
)
```

---

## 5. Built-in Dataflow Analyses

**Source**: `compiler/rustc_mir_dataflow/src/impls/`  
**GitHub**: https://github.com/rust-lang/rust/tree/main/compiler/rustc_mir_dataflow/src/impls

### `MaybeInitializedPlaces<'a, 'tcx>`

```rust
// Tracks which MovePathIndexes (place paths) are *possibly* initialized.
// Domain: MaybeReachable<MixedBitSet<MovePathIndex>>
// Direction: Forward

pub struct MaybeInitializedPlaces<'a, 'tcx> {
    tcx: TyCtxt<'tcx>,
    body: &'a Body<'tcx>,
    move_data: &'a MoveData<'tcx>,
    exclude_inactive_in_otherwise: bool,  // builder option
    skip_unreachable_unwind: bool,        // builder option
}

impl<'a, 'tcx> MaybeInitializedPlaces<'a, 'tcx> {
    pub fn new(tcx: TyCtxt<'tcx>, body: &'a Body<'tcx>, move_data: &'a MoveData<'tcx>) -> Self
    pub fn exclude_inactive_in_otherwise(mut self) -> Self
    pub fn skipping_unreachable_unwind(mut self) -> Self
    // Check if a drop's unwind is dead (used internally)
    pub fn is_unwind_dead(&self, place: mir::Place<'tcx>, state: &<Self as Analysis<'tcx>>::Domain) -> bool
}
// Implements Analysis<'tcx>

// To check if a specific place is definitely initialized, intersect with
// MaybeUninitializedPlaces (both must agree).
```

### `MaybeUninitializedPlaces<'a, 'tcx>`

```rust
// Dual of MaybeInitializedPlaces — tracks possibly-uninitialized places.
// Domain: MixedBitSet<MovePathIndex>  (MaybeUninitializedPlacesDomain)
// Direction: Forward

pub struct MaybeUninitializedPlaces<'a, 'tcx> { /* tcx, body, move_data, flags */ }

impl<'a, 'tcx> MaybeUninitializedPlaces<'a, 'tcx> {
    pub fn new(tcx, body, move_data) -> Self
    pub fn mark_inactive_variants_as_uninit(mut self) -> Self
    pub fn include_inactive_in_otherwise(mut self) -> Self
    pub fn skipping_unreachable_unwind(mut self, unreachable_unwind: DenseBitSet<BasicBlock>) -> Self
}
```

### `EverInitializedPlaces<'a, 'tcx>`

```rust
// Tracks places that have EVER been initialized (without an intervening StorageDead).
// Used to detect double-initialization of immutable locals.
// Domain: MixedBitSet<InitIndex>  (EverInitializedPlacesDomain)
// Direction: Forward

pub struct EverInitializedPlaces<'a, 'tcx> {
    body: &'a Body<'tcx>,
    move_data: &'a MoveData<'tcx>,
}
impl<'a, 'tcx> EverInitializedPlaces<'a, 'tcx> {
    pub fn new(body: &'a Body<'tcx>, move_data: &'a MoveData<'tcx>) -> Self
}
```

### `MaybeBorrowedLocals`

```rust
// Tracks which locals have a live pointer/reference to them.
// Ignores fake borrows. Used for alias analysis (especially coroutines).
// Domain: DenseBitSet<Local>
// Direction: Forward

pub struct MaybeBorrowedLocals;

impl MaybeBorrowedLocals {
    // Utility: get all locals ever borrowed in the body (single pass, no fixpoint)
    pub fn borrowed_locals(body: &Body<'_>) -> DenseBitSet<Local>
}
```

### `MaybeLiveLocals`

```rust
// Standard liveness analysis.
// A local is "live" if it might be read before the next write.
// Field-insensitive: partial write to projection counts as a USE.
// Domain: DenseBitSet<Local>
// Direction: Backward

pub struct MaybeLiveLocals;
// NOTE: combine with MaybeBorrowedLocals for full liveness through references
```

### `MaybeTransitiveLiveLocals<'a>`

```rust
// Like MaybeLiveLocals but skips dead stores.
// Designed for dead-store elimination.
// Domain: DenseBitSet<Local>
// Direction: Backward

pub struct MaybeTransitiveLiveLocals<'a> {
    always_live: &'a DenseBitSet<Local>,
    debuginfo_locals: &'a DenseBitSet<Local>,
}
impl<'a> MaybeTransitiveLiveLocals<'a> {
    pub fn new(always_live: &'a DenseBitSet<Local>, debuginfo_locals: &'a DenseBitSet<Local>) -> Self
    pub fn can_be_removed_if_dead<'tcx>(
        stmt_kind: &StatementKind<'tcx>,
        always_live: &DenseBitSet<Local>,
        debuginfo_locals: &'a DenseBitSet<Local>,
    ) -> Option<Place<'tcx>>
}
```

### `MaybeStorageLive<'a>` / `MaybeStorageDead<'a>`

```rust
// Tracks which locals have live storage at each point
// (between StorageLive and StorageDead).
// Domain: DenseBitSet<Local>
// Direction: Forward

pub struct MaybeStorageLive<'a> {
    always_live_locals: Cow<'a, DenseBitSet<Local>>,
}
impl<'a> MaybeStorageLive<'a> {
    pub fn new(always_live_locals: Cow<'a, DenseBitSet<Local>>) -> Self
}

pub struct MaybeStorageDead<'a> { /* dual */ }

// Helper: find locals without StorageLive/Dead annotations
pub fn always_storage_live_locals(body: &Body<'_>) -> DenseBitSet<Local>
```

### `MaybeRequiresStorage<'mir, 'tcx>`

```rust
// Determines whether each local *requires* storage at each location
// (storage that can be observed via pointer/reference).
// Combines MaybeBorrowedLocals with move-tracking.
// Domain: DenseBitSet<Local>
// Direction: Forward

pub struct MaybeRequiresStorage<'mir, 'tcx> {
    borrowed_locals: RefCell<BorrowedLocalsResults<'mir, 'tcx>>,
}
impl<'mir, 'tcx> MaybeRequiresStorage<'mir, 'tcx> {
    pub fn new(borrowed_locals: ResultsCursor<'mir, 'tcx, MaybeBorrowedLocals>) -> Self
}
```

### `MoveData<'tcx>` — Move paths for place tracking

```rust
// Used by MaybeInitializedPlaces, MaybeUninitializedPlaces, EverInitializedPlaces
use rustc_mir_dataflow::move_paths::{MoveData, HasMoveData, MovePathIndex, InitIndex, LookupResult};

pub struct MoveData<'tcx> {
    pub move_paths: IndexVec<MovePathIndex, MovePath<'tcx>>,
    pub moves: IndexVec<MoveOutIndex, MoveOut>,
    pub loc_map: IndexVec<Location, SmallVec<[MoveOutIndex; 4]>>,
    pub rev_lookup: MovePathLookup<'tcx>,
    pub inits: IndexVec<InitIndex, Init>,
    pub init_loc_map: IndexVec<Location, SmallVec<[InitIndex; 4]>>,
    pub init_path_map: IndexVec<MovePathIndex, SmallVec<[InitIndex; 1]>>,
}

impl<'tcx> MoveData<'tcx> {
    // Build from a body
    pub fn gather_moves(body: &Body<'tcx>, tcx: TyCtxt<'tcx>, typing_env: TypingEnv<'tcx>)
        -> Result<MoveData<'tcx>, (MoveData<'tcx>, Vec<MoveError<'tcx>>)>
}

// Lookup a place in the move path table
pub enum LookupResult {
    Exact(MovePathIndex),   // found exact match
    Parent(MovePathIndex),  // found parent (sub-path not tracked)
}
impl MovePathLookup<'tcx> {
    pub fn find(&self, place: PlaceRef<'tcx>) -> LookupResult
    pub fn find_local(&self, local: Local) -> Option<MovePathIndex>
}
```

### Typical Dataflow Analysis Usage Pattern

```rust
use rustc_mir_dataflow::Analysis;
use rustc_mir_dataflow::impls::{MaybeLiveLocals, MaybeBorrowedLocals};

fn analyze<'tcx>(tcx: TyCtxt<'tcx>, body: &mir::Body<'tcx>) {
    // 1. Run the analysis to fixpoint
    let results = MaybeLiveLocals
        .iterate_to_fixpoint(tcx, body, Some("my_pass"))
        .into_results_cursor(body);

    // 2. Inspect state at specific locations using the cursor
    let mut cursor = results;
    for (bb, bb_data) in body.basic_blocks.iter_enumerated() {
        // State at block entry
        cursor.seek_to_block_start(bb);
        let entry_state = cursor.get();
        
        // State after each statement
        for (idx, _stmt) in bb_data.statements.iter().enumerate() {
            let loc = Location { block: bb, statement_index: idx };
            cursor.seek_after_primary_effect(loc);
            let state_after = cursor.get();
            // state_after.contains(local) -> local is live here
        }
    }
}
```

---

## 6. Control Flow Utilities — Dominators, Loops

### Dominator Tree

**Source**: `compiler/rustc_data_structures/src/graph/dominators/mod.rs`  
**GitHub**: https://github.com/rust-lang/rust/blob/main/compiler/rustc_data_structures/src/graph/dominators/mod.rs  
**Algorithm**: Lengauer-Tarjan with Georgiadis path compression

```rust
use rustc_data_structures::graph::dominators::{dominators, Dominators};

// Build dominator tree for any ControlFlowGraph
pub fn dominators<G: ControlFlowGraph>(g: &G) -> Dominators<G::Node>

// Get dominators for a MIR body (cached in BasicBlocks)
let doms: &Dominators<BasicBlock> = body.basic_blocks.dominators();
```

### `Dominators<Node: Idx>` struct

```rust
pub struct Dominators<Node: Idx> { /* kind: Kind<Node> */ }

impl<Node: Idx> Dominators<Node> {
    // Is node reachable from the start node?
    pub fn is_reachable(&self, node: Node) -> bool
    
    // The unique immediate dominator of node (None for the start node)
    pub fn immediate_dominator(&self, node: Node) -> Option<Node>
    
    // Does a dominate b?
    // Panics if b is unreachable.
    // O(1) using DFS timestamps
    pub fn dominates(&self, a: Node, b: Node) -> bool
}
```

### Dominator tree traversal

```rust
// Walk the dominator chain (from node up to root)
let mut node = some_bb;
while let Some(idom) = body.basic_blocks.dominators().immediate_dominator(node) {
    println!("{node:?} is dominated by {idom:?}");
    node = idom;
}

// Check if location `a` dominates location `b` in the same/different blocks
impl Location {
    pub fn dominates(&self, other: Location, dominators: &Dominators<BasicBlock>) -> bool {
        if self.block == other.block {
            self.statement_index <= other.statement_index
        } else {
            dominators.dominates(self.block, other.block)
        }
    }
}
```

### Post-dominators

The compiler does not have a dedicated post-dominator API. To compute post-dominators:

```rust
// Post-dominator = dominator on the reversed CFG.
// Use rustc_data_structures::graph::dominators::dominators() on a reversed graph.
// Reversed graph: implement ControlFlowGraph where successors() returns predecessors
// and predecessors() returns successors of the original.
//
// Or use the BasicBlocks predecessors() and build a custom reversed structure.
use rustc_data_structures::graph::dominators::dominators as dominators_fn;

struct ReversedCFG<'a, 'tcx> {
    body: &'a Body<'tcx>,
    // conceptual start = all exit blocks (Return, UnwindResume)
}
```

### Loop Detection

**Source**: `compiler/rustc_middle/src/mir/loops.rs`  
**GitHub**: https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/mir/loops.rs

```rust
use rustc_middle::mir::loops::maybe_loop_headers;

// Returns a DenseBitSet<BasicBlock> of approximate loop headers.
// A loop header is a block that is not yet visited when its predecessor is visited in postorder.
// This is a HEURISTIC — exact loop headers require dominator computation.
pub fn maybe_loop_headers(body: &Body<'_>) -> DenseBitSet<BasicBlock>

// Usage:
let loop_headers = maybe_loop_headers(body);
for bb in body.basic_blocks.indices() {
    if loop_headers.contains(bb) {
        // bb is a likely loop header
    }
}
```

### Precise Loop Detection (using dominators)

```rust
// A back edge is an edge (A -> B) where B dominates A (i.e., B is a loop header of A's loop).
// A "natural loop" consists of all nodes that can reach B going backwards without going through B.

fn find_back_edges(body: &Body<'_>) -> Vec<(BasicBlock, BasicBlock)> {
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

### Control Dependencies

Control dependencies are not directly provided. Compute from dominance frontier:

```rust
// A node X is control-dependent on Y if:
//   1. There exists a CFG path Y -> ... -> X that does not pass through post_idom(Y)
//   2. Y has at least 2 successors
// 
// Algorithm:
// 1. Compute post-dominators (reverse CFG dominators)
// 2. For each edge (A -> B) where B does not post-dominate A:
//    Walk up post-dominator tree from B to post_idom(A), marking each node
//    as control-dependent on A.
```

---

## 7. Call Graph Construction

**Sources**:  
- `compiler/rustc_middle/src/ty/instance.rs` — Instance, InstanceKind  
- `compiler/rustc_middle/src/mir/syntax.rs` — TerminatorKind::Call  
**GitHub**:  
- https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/ty/instance.rs

### Extracting Calls from Terminators

```rust
// In your CFG traversal or visitor:
for (bb, bb_data) in body.basic_blocks.iter_enumerated() {
    let term = bb_data.terminator();
    match &term.kind {
        TerminatorKind::Call { func, args, destination, target, fn_span, .. } => {
            // func: Operand<'tcx> — the callee
            // args: Box<[Spanned<Operand<'tcx>>]> — arguments
            // destination: Place<'tcx> — return value location
            // target: Option<BasicBlock> — None if diverging function (-> !)
            
            // Get the type of func to determine callee
            let func_ty = func.ty(&body.local_decls, tcx);
            match func_ty.kind() {
                ty::FnDef(def_id, args) => {
                    // Monomorphic or generic function call
                    // Resolve to a concrete Instance:
                    if let Ok(Some(instance)) = Instance::try_resolve(
                        tcx,
                        typing_env,
                        *def_id,
                        args,
                    ) {
                        // instance.def: InstanceKind
                        // instance.args: GenericArgsRef<'tcx>
                    }
                }
                ty::FnPtr(..) => {
                    // Dynamic dispatch through function pointer — callee unknown
                    // Track as an indirect call
                }
                ty::Closure(def_id, args) => {
                    // Closure call — resolve via Instance::resolve_closure
                    let instance = Instance::resolve_closure(
                        tcx, *def_id, args, ty::ClosureKind::FnOnce
                    );
                }
                _ => {}
            }
        }
        
        TerminatorKind::TailCall { func, args, fn_span } => {
            // Same pattern as Call but no return target
        }
        
        TerminatorKind::Drop { place, .. } => {
            // May call destructor — resolve drop glue
            let place_ty = place.ty(&body.local_decls, tcx).ty;
            let instance = Instance::resolve_drop_in_place(tcx, place_ty);
            // instance.def = InstanceKind::DropGlue(def_id, Some(ty))
        }
        
        _ => {}
    }
}
```

### `Instance<'tcx>` struct

```rust
pub struct Instance<'tcx> {
    pub def: InstanceKind<'tcx>,
    pub args: GenericArgsRef<'tcx>,
}

impl<'tcx> Instance<'tcx> {
    // Primary resolution: monomorphize a generic function
    pub fn try_resolve(
        tcx: TyCtxt<'tcx>,
        typing_env: TypingEnv<'tcx>,
        def_id: DefId,
        args: GenericArgsRef<'tcx>,
    ) -> Result<Option<Instance<'tcx>>, ErrorGuaranteed>
    // Returns None if the trait method has no concrete impl (e.g. trait object call)
    // Returns Some(instance) with instance.def = InstanceKind::Virtual for dyn dispatch
    
    pub fn expect_resolve(tcx, typing_env, def_id, args, span) -> Instance<'tcx>
    
    // For function pointers
    pub fn resolve_for_fn_ptr(tcx, typing_env, def_id, args) -> Option<Instance<'tcx>>
    
    // For closures
    pub fn resolve_closure(
        tcx: TyCtxt<'tcx>,
        def_id: DefId,
        args: GenericArgsRef<'tcx>,
        requested_kind: ty::ClosureKind,
    ) -> Instance<'tcx>
    
    // For drop glue
    pub fn resolve_drop_in_place(tcx: TyCtxt<'tcx>, ty: Ty<'tcx>) -> Instance<'tcx>
    
    // Get MIR with generics instantiated
    pub fn instantiate_mir_and_normalize_erasing_regions<T>(
        &self,
        tcx: TyCtxt<'tcx>,
        typing_env: TypingEnv<'tcx>,
        value: EarlyBinder<'tcx, T>,
    ) -> T
    where T: TypeFoldable<TyCtxt<'tcx>>
}
```

### `InstanceKind<'tcx>` variants

```rust
pub enum InstanceKind<'tcx> {
    // A regular Rust function/method (may be generic)
    Item(DefId),
    
    // A compiler intrinsic (e.g., std::intrinsics::size_of)
    Intrinsic(DefId),
    
    // Wrapper for calling a fn through a vtable in a direct manner
    VTableShim(DefId),
    
    // Wrapper to reify a function as a function pointer
    ReifyShim(DefId, Option<ReifyReason>),
    
    // Wrapper for calling FnTrait::call on a function type
    FnPtrShim(DefId, Ty<'tcx>),
    
    // DYNAMIC DISPATCH — the callee is unknown at compile time
    // def_id: the trait method, idx: vtable index
    Virtual(DefId, usize),
    
    // Wrapper for calling FnOnce on a closure
    ClosureOnceShim { call_once: DefId, track_caller: bool },
    
    // Drop glue for a type
    DropGlue(DefId, Option<Ty<'tcx>>),
    
    // Clone shim for Copy types used in Clone
    CloneShim(DefId, Ty<'tcx>),
    
    // Turns a function pointer into an address
    FnPtrAddrShim(DefId, Ty<'tcx>),
    
    // Async drop glue
    AsyncDropGlue(DefId, Ty<'tcx>),
}

impl<'tcx> InstanceKind<'tcx> {
    pub fn def_id(&self) -> DefId
    pub fn is_inline(&self, tcx: TyCtxt<'tcx>) -> bool
}
```

### Handling trait method dispatch

```rust
// Pattern: resolving a trait method call
fn resolve_call_target<'tcx>(
    tcx: TyCtxt<'tcx>,
    typing_env: TypingEnv<'tcx>,
    def_id: DefId,
    substs: GenericArgsRef<'tcx>,
) -> CallTarget {
    match Instance::try_resolve(tcx, typing_env, def_id, substs) {
        Ok(Some(instance)) => match instance.def {
            InstanceKind::Item(def_id) => CallTarget::Static(def_id, instance.args),
            InstanceKind::Virtual(def_id, vtable_idx) => {
                // dyn dispatch — receiver type is a trait object
                // Cannot statically determine the callee
                CallTarget::Dynamic { trait_method: def_id, vtable_idx }
            }
            InstanceKind::Intrinsic(def_id) => CallTarget::Intrinsic(def_id),
            InstanceKind::DropGlue(def_id, ty) => CallTarget::DropGlue(def_id, ty),
            InstanceKind::ClosureOnceShim { call_once, .. } => CallTarget::Static(call_once, instance.args),
            InstanceKind::CloneShim(def_id, ty) => CallTarget::Clone(def_id, ty),
            // ... other shims
        },
        Ok(None) => CallTarget::Unknown,  // no impl found (e.g. extern fn / default)
        Err(_) => CallTarget::Error,
    }
}
```

### `TypingEnv` — Required for instance resolution

```rust
use rustc_middle::ty::TypingEnv;

// For monomorphized code (post-specialization):
let typing_env = TypingEnv::fully_monomorphized();

// For a specific function body:
let typing_env = tcx.typing_env(tcx.param_env(def_id));
// or: TypingEnv::post_analysis(tcx, def_id)
```

---

## 8. MIR Visitors

**Source**: `compiler/rustc_middle/src/mir/visit.rs`  
**GitHub**: https://github.com/rust-lang/rust/blob/main/compiler/rustc_middle/src/mir/visit.rs

### Overview

The visitor traits are generated by the `make_mir_visitor!` macro, producing two versions:
- `Visitor` — immutable, takes `&Body<'tcx>`
- `MutVisitor` — mutable, takes `&mut Body<'tcx>`, requires `fn tcx(&self) -> TyCtxt<'tcx>`

### `Visitor<'tcx>` trait (immutable)

```rust
pub trait Visitor<'tcx>: Sized {
    // === Top-level entry points ===
    fn visit_body(&mut self, body: &Body<'tcx>) {
        self.super_body(body);
    }
    
    // === Block and statement level ===
    fn visit_basic_block_data(&mut self, block: BasicBlock, data: &BasicBlockData<'tcx>)
    fn visit_source_scope_data(&mut self, scope_data: &SourceScopeData<'tcx>)
    fn visit_statement(&mut self, statement: &Statement<'tcx>, location: Location)
    fn visit_assign(&mut self, place: &Place<'tcx>, rvalue: &Rvalue<'tcx>, location: Location)
    fn visit_terminator(&mut self, terminator: &Terminator<'tcx>, location: Location)
    fn visit_assert_message(&mut self, msg: &AssertMessage<'tcx>, location: Location)
    
    // === Rvalues and operands ===
    fn visit_rvalue(&mut self, rvalue: &Rvalue<'tcx>, location: Location)
    fn visit_operand(&mut self, operand: &Operand<'tcx>, location: Location)
    fn visit_place(&mut self, place: &Place<'tcx>, context: PlaceContext, location: Location)
    fn visit_projection(&mut self, place: PlaceRef<'tcx>, context: PlaceContext, location: Location)
    fn visit_projection_elem(&mut self, place: PlaceRef<'tcx>, elem: &PlaceElem<'tcx>, context: PlaceContext, location: Location)
    fn visit_constant(&mut self, constant: &ConstOperand<'tcx>, location: Location)
    fn visit_const_operand(&mut self, constant: &ConstOperand<'tcx>, location: Location)
    fn visit_ty(&mut self, ty: Ty<'tcx>, ty_context: TyContext)
    fn visit_region(&mut self, region: Region<'tcx>, location: Location)
    
    // === Locals and types ===
    fn visit_local(&mut self, local: Local, context: PlaceContext, location: Location)
    fn visit_local_decl(&mut self, local: Local, local_decl: &LocalDecl<'tcx>)
    fn visit_source_info(&mut self, source_info: &SourceInfo)
    fn visit_span(&mut self, span: &Span)
    
    // === Debug info ===
    fn visit_var_debug_info(&mut self, var_debug_info: &VarDebugInfo<'tcx>)
    
    // === Convenience ===
    // visit a specific location, calling visit_statement or visit_terminator as appropriate
    fn visit_location(&mut self, body: &Body<'tcx>, location: Location)
    
    // Super methods (default implementations — call these to recurse normally)
    fn super_body(&mut self, body: &Body<'tcx>)
    fn super_basic_block_data(&mut self, block: BasicBlock, data: &BasicBlockData<'tcx>)
    fn super_statement(&mut self, statement: &Statement<'tcx>, location: Location)
    fn super_terminator(&mut self, terminator: &Terminator<'tcx>, location: Location)
    fn super_assign(&mut self, place: &Place<'tcx>, rvalue: &Rvalue<'tcx>, location: Location)
    fn super_rvalue(&mut self, rvalue: &Rvalue<'tcx>, location: Location)
    fn super_operand(&mut self, operand: &Operand<'tcx>, location: Location)
    fn super_place(&mut self, place: &Place<'tcx>, context: PlaceContext, location: Location)
    fn super_local_decl(&mut self, local: Local, local_decl: &LocalDecl<'tcx>)
    // ... and more super_ methods for every visit_ method
}
```

### `MutVisitor<'tcx>` trait (mutable)

```rust
pub trait MutVisitor<'tcx>: Sized {
    // Required: provide access to TyCtxt for place projections
    fn tcx(&self) -> TyCtxt<'tcx>;
    
    // Same visit_ methods as Visitor but taking &mut references
    fn visit_body(&mut self, body: &mut Body<'tcx>)
    fn visit_place(&mut self, place: &mut Place<'tcx>, context: PlaceContext, location: Location)
    fn visit_local(&mut self, local: &mut Local, context: PlaceContext, location: Location)
    // ... (same structure as Visitor)
}
```

### `PlaceContext` enum — how a place is used

```rust
pub enum PlaceContext {
    NonMutatingUse(NonMutatingUseContext),
    MutatingUse(MutatingUseContext),
    NonUse(NonUseContext),
}

pub enum NonMutatingUseContext {
    Inspect,        // PlaceMention, FakeRead
    Copy,           // Operand::Copy
    Move,           // Operand::Move
    SharedBorrow,   // Rvalue::Ref(_, Shared, _)
    FakeBorrow,     // Rvalue::Ref(_, Fake(_), _)
    RawBorrow,      // Rvalue::RawPtr(_, _)
    Projection,     // intermediate place in projection path
    PlaceMention,
}

pub enum MutatingUseContext {
    Store,          // StatementKind::Assign lhs
    SetDiscriminant,
    Deinit,
    Call,           // TerminatorKind::Call destination
    TailCall,
    Yield,          // TerminatorKind::Yield resume_arg
    Drop,           // TerminatorKind::Drop place
    Borrow,         // Rvalue::Ref(_, Mut, _)
    RawBorrow,      // Rvalue::RawPtr (mutable)
    Retag,          // StatementKind::Retag
    Projection,     // intermediate place in mutable projection path
    AsmOutput,      // InlineAsm output
}

pub enum NonUseContext {
    StorageLive,    // StatementKind::StorageLive
    StorageDead,    // StatementKind::StorageDead
    AscribeUserTy(Variance),
    VarDebugInfo,
}
```

### `VisitPlacesWith<F>` — utility visitor

```rust
// Visits all places (and their locals) in a body with a closure.
pub struct VisitPlacesWith<F>(F);

impl<F: FnMut(Local, PlaceContext, Location)> Visitor<'_> for VisitPlacesWith<F> {
    fn visit_local(&mut self, local: Local, context: PlaceContext, location: Location) {
        (self.0)(local, context, location)
    }
}

// Usage: collect all reads/writes to each local
let mut reads: HashMap<Local, Vec<Location>> = HashMap::new();
VisitPlacesWith(|local, context, location| {
    if matches!(context, PlaceContext::NonMutatingUse(_)) {
        reads.entry(local).or_default().push(location);
    }
}).visit_body(body);
```

### Common visitor patterns

```rust
// Pattern 1: Collect all call sites
struct CallCollector<'tcx> {
    calls: Vec<(Location, Operand<'tcx>, Vec<Operand<'tcx>>)>,
}

impl<'tcx> Visitor<'tcx> for CallCollector<'tcx> {
    fn visit_terminator(&mut self, terminator: &Terminator<'tcx>, location: Location) {
        if let TerminatorKind::Call { func, args, .. } = &terminator.kind {
            self.calls.push((location, func.clone(), args.iter().map(|a| a.node.clone()).collect()));
        }
        self.super_terminator(terminator, location);
    }
}

// Pattern 2: Track def-use for each local
struct DefUseCollector {
    defs: IndexVec<Local, Vec<Location>>,
    uses: IndexVec<Local, Vec<Location>>,
}

impl<'tcx> Visitor<'tcx> for DefUseCollector {
    fn visit_local(&mut self, local: Local, context: PlaceContext, location: Location) {
        match context {
            PlaceContext::MutatingUse(MutatingUseContext::Store | MutatingUseContext::Call) => {
                self.defs[local].push(location);
            }
            PlaceContext::NonMutatingUse(_) => {
                self.uses[local].push(location);
            }
            _ => {}
        }
    }
}

// Pattern 3: Count all places read per location
struct ReadPlacesAtLocation<'tcx> {
    target_loc: Location,
    reads: Vec<Place<'tcx>>,
}
impl<'tcx> Visitor<'tcx> for ReadPlacesAtLocation<'tcx> {
    fn visit_place(&mut self, place: &Place<'tcx>, context: PlaceContext, location: Location) {
        if location == self.target_loc && matches!(context, PlaceContext::NonMutatingUse(_)) {
            self.reads.push(*place);
        }
    }
}
```

---

## Appendix A: Module Structure & Crate Layout

```
compiler/rustc_middle/src/mir/
├── mod.rs              Body, LocalDecl, SourceScopeData, LocalKind, START_BLOCK
├── syntax.rs           StatementKind, TerminatorKind, Place, Operand, Rvalue, BorrowKind
├── basic_blocks.rs     BasicBlocks, Predecessors, CFG cache
├── terminator.rs       Terminator methods, successors(), edges(), SwitchTargets
├── traversal.rs        preorder, postorder, reverse_postorder, reachable_as_bitset, mono_reachable
├── visit.rs            Visitor, MutVisitor, PlaceContext, VisitPlacesWith
├── loops.rs            maybe_loop_headers()
└── query.rs            ConstQualifs, CoroutineLayout

compiler/rustc_middle/src/ty/
└── instance.rs         Instance, InstanceKind, try_resolve, resolve_closure, resolve_drop_in_place

compiler/rustc_data_structures/src/graph/
└── dominators/mod.rs   Dominators, dominators(), immediate_dominator(), dominates()

compiler/rustc_mir_dataflow/src/
├── framework/
│   ├── mod.rs          Analysis trait, GenKill, JoinSemiLattice, MaybeReachable, iterate_to_fixpoint
│   ├── cursor.rs       ResultsCursor, seek_before/after_primary_effect
│   ├── direction.rs    Forward, Backward, Direction
│   ├── lattice.rs      JoinSemiLattice impls, MaybeReachable
│   ├── results.rs      Results, EntryStates
│   └── visitor.rs      ResultsVisitor, visit_results, visit_reachable_results
├── impls/
│   ├── initialized.rs  MaybeInitializedPlaces, MaybeUninitializedPlaces, EverInitializedPlaces
│   ├── borrowed_locals.rs  MaybeBorrowedLocals, borrowed_locals()
│   ├── liveness.rs     MaybeLiveLocals, MaybeTransitiveLiveLocals, DefUse
│   └── storage_liveness.rs  MaybeStorageLive, MaybeStorageDead, MaybeRequiresStorage,
│                             always_storage_live_locals()
└── move_paths/         MoveData, MovePathIndex, InitIndex, LookupResult
```

---

## Appendix B: Key Imports Cheat Sheet

```rust
// Core MIR types
use rustc_middle::mir::{
    Body, BasicBlock, BasicBlockData, BasicBlocks,
    Local, LocalDecl, LocalKind, LocalInfo,
    Place, PlaceElem, PlaceRef, PlaceTy,
    Operand, ConstOperand,
    Rvalue, BorrowKind, AggregateKind,
    Statement, StatementKind,
    Terminator, TerminatorKind, TerminatorEdges,
    SwitchTargets, CallReturnPlaces,
    Location, SourceInfo, SourceScope, SourceScopeData,
    START_BLOCK, RETURN_PLACE,
    MirPhase, MirSource,
};

// Traversal
use rustc_middle::mir::traversal::{self, preorder, postorder, reverse_postorder, reachable_as_bitset};

// Visitor
use rustc_middle::mir::visit::{Visitor, MutVisitor, PlaceContext, NonMutatingUseContext, MutatingUseContext, NonUseContext};

// Instance resolution
use rustc_middle::ty::{Instance, InstanceKind, TyCtxt, TypingEnv};
use rustc_middle::ty::instance::Instance;

// Dominators
use rustc_data_structures::graph::dominators::{dominators, Dominators};
use rustc_middle::mir::BasicBlocks;  // .dominators() method via cached field

// Dataflow framework
use rustc_mir_dataflow::Analysis;
use rustc_mir_dataflow::{Forward, Backward};
use rustc_mir_dataflow::{JoinSemiLattice, MaybeReachable};
use rustc_mir_dataflow::{Results, ResultsCursor};
use rustc_mir_dataflow::{ResultsVisitor, visit_results, visit_reachable_results};
use rustc_mir_dataflow::GenKill;

// Built-in analyses
use rustc_mir_dataflow::impls::{
    MaybeInitializedPlaces,
    MaybeUninitializedPlaces,
    EverInitializedPlaces,
    MaybeBorrowedLocals,
    MaybeLiveLocals,
    MaybeTransitiveLiveLocals,
    MaybeStorageLive, MaybeStorageDead,
    MaybeRequiresStorage,
    always_storage_live_locals,
    borrowed_locals,
};

// Move paths (needed for initialization analyses)
use rustc_mir_dataflow::move_paths::{MoveData, HasMoveData, MovePathIndex, InitIndex, LookupResult};

// Index types
use rustc_index::{IndexVec, Idx};
use rustc_index::bit_set::{DenseBitSet, MixedBitSet};

// Loops
use rustc_middle::mir::loops::maybe_loop_headers;
```

---

## Appendix C: Code-Graph Construction Recipe

Here is a complete algorithm sketch for building a code-graph from MIR:

```rust
pub struct CodeGraph<'tcx> {
    // Nodes: one per (DefId, GenericArgs) pair
    nodes: HashMap<Instance<'tcx>, FunctionNode<'tcx>>,
}

pub struct FunctionNode<'tcx> {
    // CFG
    blocks: Vec<BasicBlock>,
    edges: Vec<(BasicBlock, BasicBlock)>,       // successor edges
    back_edges: Vec<(BasicBlock, BasicBlock)>,  // loop back edges
    
    // Data flow per block
    entry_live: IndexVec<BasicBlock, DenseBitSet<Local>>,
    
    // Calls
    call_sites: Vec<CallSite<'tcx>>,
}

pub struct CallSite<'tcx> {
    location: Location,
    callee: CallTarget<'tcx>,
    args: Vec<Operand<'tcx>>,
    destination: Place<'tcx>,
}

fn build_code_graph<'tcx>(tcx: TyCtxt<'tcx>, root: Instance<'tcx>) {
    let mut worklist = vec![root];
    let mut visited = HashSet::new();
    let mut graph = CodeGraph::default();

    while let Some(instance) = worklist.pop() {
        if !visited.insert(instance) { continue; }
        
        // 1. Get MIR body
        let body = tcx.instance_mir(instance.def);
        
        // 2. Build CFG edges
        let predecessors = body.basic_blocks.predecessors();
        let dominators = body.basic_blocks.dominators();
        
        let mut node = FunctionNode::default();
        for (bb, data) in body.basic_blocks.iter_enumerated() {
            for succ in data.terminator().successors() {
                node.edges.push((bb, succ));
                // Detect back edges (loops)
                if dominators.dominates(succ, bb) {
                    node.back_edges.push((bb, succ));
                }
            }
        }
        
        // 3. Run liveness analysis
        let live_results = MaybeLiveLocals
            .iterate_to_fixpoint(tcx, body, None);
        for bb in body.basic_blocks.indices() {
            node.entry_live[bb] = live_results.entry_states[bb].clone();
        }
        
        // 4. Collect call sites and build call graph
        let mut call_visitor = CallCollector::new(tcx, instance, body);
        call_visitor.visit_body(body);
        
        for call in call_visitor.calls {
            if let CallTarget::Static(callee_instance) = call.callee {
                worklist.push(callee_instance);
            }
            node.call_sites.push(call);
        }
        
        graph.nodes.insert(instance, node);
    }
}
```

---

*Research conducted from GitHub source: https://github.com/rust-lang/rust  
See also: [MIR chapter in rustc-dev-guide](https://rustc-dev-guide.rust-lang.org/mir/index.html)*
