# Rust Compiler Research for Parseltongue (Code Graph + Control Flow + Data Flow)
Date: 2026-03-01
Status: Research Draft

## Scope
You asked whether Parseltongue should use Rust Analyzer or rustc internals, and what parts of rustc are reusable for:
1. code graph
2. control flow graph (CFG)
3. data flow

## Clone Snapshot
- Repo cloned: `https://github.com/rust-lang/rust`
- Local path: `/tmp/rust-compiler-research-20260301-v2`
- Commit analyzed: `82153749` (2026-02-28)

## Short Verdict (Analyzer vs Compiler)
For V200/V216, use a hybrid:
1. Rust Analyzer as default for fast entity graph and query UX.
2. rustc sidecar for deep truth (MIR CFG + selected dataflow) on-demand.

Why:
- rustc gives superior control/data-flow truth.
- rustc internals are heavier and less ergonomic for always-on interactive indexing.
- Rust Analyzer is still the better default latency/ergonomics path for day-to-day retrieval.

So: do not replace Analyzer with rustc. Add rustc as a precision sidecar for "deep mode".

## What rustc Gives You That Analyzer Usually Does Not

### 1) MIR as canonical execution-oriented IR (CFG backbone)
Evidence:
- `compiler/rustc_middle/src/mir/mod.rs` defines `Body` and `BasicBlockData`.
- `compiler/rustc_middle/src/mir/syntax.rs` defines `TerminatorKind`.
- `compiler/rustc_middle/src/mir/terminator.rs` exposes successor/edge APIs.

Relevant pointers:
- `Body`: `compiler/rustc_middle/src/mir/mod.rs:210`
- `BasicBlockData`: `compiler/rustc_middle/src/mir/mod.rs:1290`
- `TerminatorKind`: `compiler/rustc_middle/src/mir/syntax.rs:703`
- `successors()` and `edges()`: `compiler/rustc_middle/src/mir/terminator.rs:455`, `:761`, `:767`

Practical pickup:
- Build CFG edges directly from MIR terminator edges.
- Persist as typed edges:
  - `cfg_next`
  - `cfg_branch_true/false`
  - `cfg_call_return`
  - `cfg_unwind`

### 2) General dataflow framework with fixpoint engine
Evidence:
- `rustc_mir_dataflow` framework (`Analysis` trait, cursor/visitor/fixpoint).

Relevant pointers:
- Framework intro + trait: `compiler/rustc_mir_dataflow/src/framework/mod.rs:1`, `:101`
- `ResultsCursor`: `compiler/rustc_mir_dataflow/src/framework/mod.rs:58`
- Public exports: `compiler/rustc_mir_dataflow/src/lib.rs:19`

Practical pickup:
- Reuse conceptual model for Parseltongue pluggable dataflow passes:
  - transfer function
  - direction (forward/backward)
  - lattice join
  - fixpoint convergence

### 3) Borrow-checker facts for richer data-flow semantics
Evidence:
- Compiler-consumer API to extract borrowck facts and optional Polonius facts.

Relevant pointers:
- `BodyWithBorrowckFacts`: `compiler/rustc_borrowck/src/consumers.rs:85`
- `get_bodies_with_borrowck_facts`: `compiler/rustc_borrowck/src/consumers.rs:123`
- Borrowck dataflow composition: `compiler/rustc_borrowck/src/dataflow.rs:22`

Practical pickup:
- Optional deep diagnostics channel:
  - borrow lifetimes
  - initialized/uninitialized places
  - borrow out-of-scope map
- This is expensive; make it opt-in per entity/query intent.

### 4) HIR containment and parent walk (entity wrapping is cheap)
Evidence:
- `TyCtxt` HIR accessors and parent iterators.

Relevant pointers:
- Parent iterator + owner lookup: `compiler/rustc_middle/src/hir/map.rs:25`, `:103`
- `hir_owner_nodes`: `compiler/rustc_middle/src/hir/map.rs:111`
- `hir_node`, `parent_hir_id`, `hir_enclosing_body_owner`: `compiler/rustc_middle/src/hir/map.rs:131`, `:145`, `:236`

Practical pickup:
- Your Layer-2 “find wrapping entity” maps directly to HIR parent traversal.

### 5) Query-level hooks for MIR/typeck/borrowck/callgraph
Evidence:
- rustc query definitions expose right entry points.

Relevant pointers:
- Root analysis query: `compiler/rustc_middle/src/queries.rs:413`
- `mir_built`: `compiler/rustc_middle/src/queries.rs:651`
- `optimized_mir`: `compiler/rustc_middle/src/queries.rs:737`
- `typeck`: `compiler/rustc_middle/src/queries.rs:1249`
- `mir_borrowck`: `compiler/rustc_middle/src/queries.rs:1266`
- `mir_callgraph_cyclic`: `compiler/rustc_middle/src/queries.rs:1320`
- `mir_inliner_callees`: `compiler/rustc_middle/src/queries.rs:1331`

Practical pickup:
- Seed call/dependency edges with compiler-proven callgraph queries where available.

### 6) Driver integration points (where to run extraction)
Evidence:
- `Callbacks` trait and `after_analysis` hook in driver.

Relevant pointers:
- `Callbacks`: `compiler/rustc_driver_impl/src/lib.rs:118`
- `after_analysis`: `compiler/rustc_driver_impl/src/lib.rs:142`
- `run_compiler`: `compiler/rustc_driver_impl/src/lib.rs:170`
- call to `analysis(())`: `compiler/rustc_driver_impl/src/lib.rs:325`
- callback invocation: `compiler/rustc_driver_impl/src/lib.rs:331`
- interface entrypoint: `compiler/rustc_interface/src/interface.rs:388`

Practical pickup:
- Run extraction after rustc analysis completes (guarantees semantic data availability).

### 7) Span and line mapping you can trust
Evidence:
- `SourceMap` and line/snippet lookup.

Relevant pointers:
- `SourceMap`: `compiler/rustc_span/src/source_map.rs:200`
- `lookup_char_pos`: `compiler/rustc_span/src/source_map.rs:430`
- `lookup_line`: `compiler/rustc_span/src/source_map.rs:437`
- `span_to_location_info`: `compiler/rustc_span/src/source_map.rs:480`

Practical pickup:
- Canonical pointer emission:
  - `file_path|||entity_key|||start_line|||end_line`

### 8) Reusable graph algorithms for your graph engine
Evidence:
- rustc graph toolkit includes SCC and dominators for CFG/graph analytics.

Relevant pointers:
- Graph traits: `compiler/rustc_data_structures/src/graph/mod.rs:14`
- Dominators: `compiler/rustc_data_structures/src/graph/dominators/mod.rs:1`, `:41`
- Tarjan SCCs: `compiler/rustc_data_structures/src/graph/scc/mod.rs:1`, `:73`

Practical pickup:
- For control-flow intelligence:
  - dominator tree
  - SCC condensation graph
  - cycle-aware ranking

### 9) Incremental identity design lessons from DepNode
Evidence:
- dep graph nodes are kind + fingerprint, stable across sessions.

Relevant pointers:
- DepNode design docs: `compiler/rustc_middle/src/dep_graph/dep_node.rs:1`
- DepNode struct: `compiler/rustc_middle/src/dep_graph/dep_node.rs:94`

Practical pickup:
- Strengthen Parseltongue identity model with fingerprint side-channel:
  - canonical key + stable hash/fingerprint for staleness and cross-run linking.

## What NOT to Pull Into V200 Core
1. Full rustc query dep-graph runtime as canonical product graph.
   - It is compiler-internal incremental plumbing, not user-facing code architecture graph.
2. Full borrowck/polonius by default.
   - Useful, but too heavy and unstable for baseline query latency.
3. Full rustc driver replacement.
   - Keep as sidecar extractor mode.

## A Practical Architecture for Parseltongue

### Mode A (Default Fast Mode)
- Analyzer/tree-sitter path
- outputs:
  - entities
  - dependency/call edges (best-effort)
  - span pointers

### Mode B (rustc Deep Mode, on-demand)
- rustc sidecar on selected crate/function set
- outputs:
  - MIR CFG edges
  - selected dataflow facts
  - higher-confidence call/data-flow slices

### Merge Rule
- Canonical identity remains Parseltongue key:
  - `language|||kind|||scope|||name|||file_path|||discriminator`
- rustc facts attach as evidence/verified layers with provenance.

## Suggested First 4 Implementable Steps
1. Add rustc deep-mode prototype using `Callbacks::after_analysis`.
2. Extract MIR CFG for one function into Parseltongue edge schema.
3. Attach `SourceMap`-based line spans for every extracted node/edge.
4. Add one opt-in dataflow pass (`MaybeLiveLocals`) and expose as query packet.

## Strategic Recommendation
If the goal is LLM-ready context at scale:
1. Keep Analyzer for breadth + speed.
2. Add rustc for depth + correctness where it matters.
3. Route by query intent:
   - explain/search: Analyzer-first
   - refactor-risk, aliasing, lifecycle, tricky control/data-flow: rustc deep mode

This gives you PMF-friendly latency while preserving a path to compiler-grade truth.
