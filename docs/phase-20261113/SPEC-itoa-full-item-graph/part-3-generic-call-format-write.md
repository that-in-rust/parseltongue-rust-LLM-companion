# Part 3 — a method call on a generic receiver

**State: NEXT, not started. One decision is needed before code (Q1 below).**

**Target:** `Buffer::format --calls--> private::Sealed::write`, from this source:

```rust
pub fn format<I: Integer>(&mut self, i: I) -> &str {   // src/lib.rs:106
    let string = i.write(unsafe { &mut *buf_ptr });     // src/lib.rs:108
pub trait Integer: private::Sealed { .. }               // src/lib.rs:119
pub trait Sealed: Copy { fn write(self, buf: &mut Self::Buffer) -> &str; }   // :128
```

**What it takes:** three small additions, in this order: the method-call site carries its
receiver and method name; bounds and supertraits become rows; the call resolver walks them.
No type inference is needed, because the receiver is a parameter whose declared type is a
generic parameter.

## 1 — Work mode

Parser and analysis pipeline. Spec Mode plus Delivery Mode. Main risk: emitting an edge
that source does not prove.

## 2 — Executable requirements

**REQ-RUST-042.0 — method call on a generic receiver.**

- **R1** WHEN a method call's receiver is a plain identifier that names a parameter of the
  enclosing function, and that parameter's declared type is a generic parameter of the
  function or its impl, THEN the candidate traits SHALL be that parameter's bounds (inline
  and `where`) and all their supertraits.
- **R2** WHEN exactly one candidate trait declares a method with that name and a `self`
  parameter, THEN the system SHALL store one `calls` edge to that trait method. It SHALL
  never target an implementation and never emit one edge per implementing type.
- **R3** WHEN no candidate or several candidates declare it, THEN the site SHALL be stored
  unresolved with `MissingTarget` or `AmbiguousTarget`.
- **R4** WHEN the receiver identifier is rebound by a `let`, a pattern or a closure parameter
  between the parameter and the call, THEN the site SHALL be stored unresolved.
- **R5** WHEN the receiver is any other expression (field, call, cast, `self`), THEN the site
  SHALL stay unresolved with `TraitSolverNeeded`. That is later work, not this part.
- **R6** WHEN supertraits form a cycle, THEN the walk SHALL terminate and visit each trait once.
- **R7** A method name that matches an unrelated trait or free function SHALL select nothing:
  only traits reached through the bound chain count.
- **R8** The edge SHALL carry the bound chain as dispatch metadata
  (`I`, `Integer`, `private::Sealed`) and the source range of the call.

## 3 — Design by layer

What exists today (read in the code, not assumed):

- The `i.write(..)` site is already a `METHOD_CALL_EXPR` row in `body-site-reference-table`.
  `extract_original_call_syntax` handles only path calls, so it is stored unresolved.
- Bound and supertrait paths outside bodies are submitted to name resolution as signature
  paths. No row links a generic parameter or a trait to them.
- Trait methods are graph nodes contained by their trait.

Additions:

```text
L1  parseltongue-core-row-types
    a bound record: owner item, generic parameter name (or the trait itself for a
    supertrait), the occurrence of the bound's path, where it was written
    (inline | where | supertrait). An enum for the method-call receiver shape.
L3  parseltongue-syntax-span-extractor
    extract_original_call_syntax   also fills receiver text and range, method name,
                                   and the receiver shape, for METHOD_CALL_EXPR
    a new extractor                writes one bound row per bound and per supertrait
    resolve_original_call_edges    new branch: parameter -> generic parameter -> bounds ->
                                   supertraits (visited set) -> trait members by name
L3  parseltongue-turso-store-adapter
    one new table, generic-bound-source-records, owned by signature-type-lowering-stage,
    registered in STAGE_OWNED_TABLE_NAMES
```

No new trait seam: rows are read through the existing `StageTableStorePort`. No change to
DuckDB. Errors extend `OriginalCallResolutionError`; no free-text variant.

## 4 — Verification matrix

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| 042.0 R1 R2 R8 | test_format_sealed_write_call | real source, CLI | one `calls` edge `Buffer::format` to `private::Sealed::write`, range of line 108, bound chain in metadata, read back frozen | exactly 1 edge |
| 042.0 R2 | test_format_sealed_write_call | real source, CLI | no edge from `Buffer::format` to any `write` in an impl | 0 edges |
| 042.0 R1 | test_generic_bound_rows_written | real source, CLI | bound rows for `I: Integer`, `Integer: private::Sealed`, `Sealed: Copy` | 3 rows present |
| 042.0 R3 | test_bound_method_ambiguous_candidates | fixture | two bound traits declare the method | unresolved, `AmbiguousTarget` |
| 042.0 R3 R7 | test_bound_method_name_distractor | fixture | same method name on a trait outside the bound chain | not selected |
| 042.0 R4 | test_local_receiver_shadow_unresolved | fixture | `let i = 0u8; i.write()` inside the function | unresolved |
| 042.0 R5 | test_other_receiver_shapes_unresolved | fixture | field, call and `self` receivers | unresolved, `TraitSolverNeeded` |
| 042.0 R6 | test_cyclic_supertrait_bounds_terminate | fixture | `trait A: B`, `trait B: A` | terminates, each visited once |
| 042.0 | test_trait_rename_changes_identity | real source, mutated copy | rename `Sealed` in a copy of the pinned tree | target identity changes, still 1 edge |
| 045.0 | test_original_member_graph_roundtrip | real source, CLI | existing test still passes; counts move as expected | resolved calls 2 to 3 |

The count in the last row is the expectation; RED measures the real number first.

## 5 — Implementation plan

Each step is its own commit of about 200 lines, with a log entry.

1. STUB and RED: `test_format_sealed_write_call` on pinned itoa. Record how the site is
   stored today and the reason it carries.
2. Method-call syntax evidence in `extract_original_call_syntax`, with fixture tests for R5.
3. Bound rows: record type, extractor, table registration, `test_generic_bound_rows_written`.
4. Resolver branch, with fixture tests for R3, R4, R6, R7.
5. GREEN on the real-source test; add the rename test.
6. REFACTOR, then the gates below, then re-measure the graph counts and one run's cost.

## 6 — Quality gate results

Not run: no code is written. Gates for this part: `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, the bounded workspace test run, the release
build, and the 11 real-source tests plus the new ones.

## 7 — Open questions

- **Q1 (decide before step 4): `Sealed: Copy`.** `Copy` is in `core`. In a run without the
  sysroot its methods are unknown, so "exactly one candidate declares `write`" cannot be
  fully proven.
  - *Strict:* store the site unresolved with `ExternalSourceMissing`. The target is then
    reachable only in a run with the sysroot, which has not finished so far.
  - *Flagged:* emit the edge when exactly one known candidate declares the method, and record
    in the dispatch metadata that one trait in the chain had no source. The final acceptance
    run with the sysroot removes the flag.
  - Recommended: flagged. The run is already marked `incomplete` and cannot be an SOP PASS
    without the sysroot, so nothing is accepted on the weaker proof.
- **Q2: lookup order.** rustc checks methods from a type parameter's bounds before methods of
  other traits in scope. The design relies on it; confirm with a small program compiled by
  rustc before step 4.
- **Q3: `I::MAX_STR_LEN` on line 109** uses the same bound chain and gives a `reads` edge.
  It is left to [part 4](part-4-signatures-other-edge-kinds.md) so this part stays one edge.
