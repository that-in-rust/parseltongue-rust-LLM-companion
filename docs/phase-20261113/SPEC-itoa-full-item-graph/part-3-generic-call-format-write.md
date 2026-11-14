# Part 3 — a method call on a generic receiver

**State: BUILT for one receiver shape. The target edge is stored from pinned source and the
workspace gates pass at `b1115a5f4`, with one test skipped as before. Two things are not done:
type parameters of an enclosing impl or trait, and a rename test on real source.**

**Target:** `Buffer::format --calls--> private::Sealed::write`, from this source:

```rust
pub fn format<I: Integer>(&mut self, i: I) -> &str {   // src/lib.rs:106
    let string = i.write(unsafe { &mut *buf_ptr });     // src/lib.rs:108
pub trait Integer: private::Sealed { .. }               // src/lib.rs:119
pub trait Sealed: Copy { fn write(self, buf: &mut Self::Buffer) -> &str; }   // :128
```

**What it took:** three small additions, in this order: the method-call site carries its
receiver and method name; bound rows carry what they bound; the call resolver walks them.
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
- **R9** WHEN a trait in the chain has no source in this run and exactly one known candidate
  declares the method, THEN the edge SHALL still be stored, and its dispatch metadata SHALL
  name each trait whose source was missing. WHEN no known candidate declares it and a trait
  in the chain has no source, THEN the site SHALL be unresolved with `ExternalSourceMissing`.

## 3 — Design by layer

What exists today (read in the code, not assumed):

- The `i.write(..)` site is already a `METHOD_CALL_EXPR` row in `body-site-reference-table`.
  `extract_original_call_syntax` handles only path calls, so it is stored unresolved.
- Bound and supertrait paths outside bodies are submitted to name resolution as signature
  paths. No row links a generic parameter or a trait to them.
- Trait methods are graph nodes contained by their trait.

Additions:

```text
L3  parseltongue-syntax-span-extractor
    extract_original_call_syntax   BUILT. Each METHOD_CALL_EXPR row carries
                                   method_receiver_evidence: receiver shape, method name,
                                   the parameter's declared type, type-parameter owner,
                                   rebinding flag
    extract_type_bound_syntax      BUILT. Each TYPE_BOUND row carries bound_syntax_evidence:
                                   what is bounded (type parameter, supertrait, where
                                   subject) and the trait path with its source range
    resolve_original_call_edges    NOT BUILT. New branch: parameter -> type parameter ->
                                   bounds -> supertraits (visited set) -> trait members

No new table and no new record type. `field-signature-detail-table` already had one row per
`TYPE_BOUND`, owned by its function or trait, so the bound evidence is added to that row
instead of the separate table first planned here.
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
| 042.0 R5 | test_other_receiver_shapes_unresolved | fixture | field, reference and cast receivers; a type parameter of the enclosing trait | unresolved, `TraitSolverNeeded` |
| 042.0 R1 | test_resolve_bound_supertrait_method | fixture | inline and `where` bounds both reach the supertrait's method | 1 edge each |
| 042.0 R1 | test_bound_method_missing_target | fixture | no bound trait declares the method; method without `self`; no bound | unresolved, `MissingTarget` |
| 042.0 R9 | test_format_sealed_write_call | real source, CLI, sysroot excluded | the edge's metadata names `Copy` as a trait without source | 1 gap entry |
| 042.0 R9 | test_missing_source_gap_recorded | fixture | same, on a fixture; `?Sized` and lifetime bounds are skipped | 1 gap entry |
| 042.0 R9 | test_missing_source_only_unresolved | fixture | the method is declared by no known trait and one chain trait has no source | unresolved, `ExternalSourceMissing` |
| 042.0 R6 | test_cyclic_supertrait_bounds_terminate | fixture | `trait A: B`, `trait B: A` | terminates, each visited once |
| 042.0 | test_trait_rename_changes_identity | fixture | traits and method renamed | still 1 edge, to the renamed method |
| 045.0 | test_original_member_graph_roundtrip | real source, CLI | existing test still passes | pass |

All of these exist in code and pass. Two differences from the plan: the rename test runs on a
fixture, not on a mutated copy of the pinned tree, and it does not compare identities before
and after. Resolved calls went from 2 to 3 in the measured run; no test asserts that count.

## 5 — Implementation plan

Each step is its own commit of about 200 lines, with a log entry.

1. DONE `9a007b8ec`. RED: `test_format_sealed_write_call` on pinned itoa; the site was stored
   unresolved with `TraitSolverNeeded`.
2. DONE `408eb28a5`. Method-call syntax evidence, four fixture tests.
3. DONE `f80c3932e`. Bound evidence on the existing `TYPE_BOUND` rows.
4. DONE `ded07eeeb`. Resolver branch; the real-source test is GREEN.
5. DONE `4940980c0`, `b1115a5f4`. Ten resolver fixture tests.
6. Gates run (section 6). No refactor was made: the resolver is one function and its index.
   Counts and cost were measured once, just before step 4's final reshape.

## 6 — Quality gate results

Run at `b1115a5f4` with the four commands in `AGENTS.md`:

```text
format    pass
lint      pass (release, workspace, all targets, all features, warnings denied)
tests     442 passed, 0 failed, 4 ignored, in 21 test binaries; 3 min 39 s
          skipped as before: test_lang_items_in_sysroot (open defect, not a waiver)
build     pass (release, workspace, all targets)
real source   13 of 13 itoa tests pass inside that run, 114.56 s
```

- The four ignored tests are three manual store profiles and one subprocess fixture.
- Steps 2 to 5 were pushed after gates on the two touched crates only; the workspace gates
  were run once, after step 5.
- Only the real-source test has a recorded failing run. The fixture tests were written with or
  after the code; four deliberate one-line breakages of the resolver each failed one test.
- Memory: 355 MB peak for the largest single process of one real-source run. Whole process
  tree was not measured.

Left open by this part:

- A type parameter of the enclosing impl or trait (`impl<I: Integer> W { fn run(i: I) .. }`)
  stays unresolved. No call in pinned itoa has been shown to need it.
- `where Self: Trait` bounds and bounds on associated types are not read.
- A bound with generic arguments (`I: Into<u8>`) makes the call unresolved.

## 7 — Open questions

- **Q1: `Sealed: Copy`. Decided 2026-11-14: store the edge and name the gap (R9).** `Copy` is
  in `core`; without the sysroot its methods are unknown, so "exactly one candidate declares
  `write`" is not fully proven. Codebases are expected to be partial, so dropping the edge
  would lose what source does show. If `Copy` also declared `write`, rustc would reject the
  call as ambiguous, so the edge is correct whenever the code compiles. The rule is general
  and recorded in the [PRD](../PRD-itoa-full-item-graph-scope.md), "Decisions in force".
- **Q2: lookup order. Answered.** rustc checks methods from a type parameter's bounds before
  methods of other traits in scope, so traits outside the bound chain never compete (R7).
  Confirmed 2026-11-14 with rustc 1.99.1: with `I: Integer`, `Integer: Sealed`, and a second
  trait in scope whose blanket impl also has `write`, `i.write()` compiles and calls
  `Sealed::write`.
- **Q3: `I::MAX_STR_LEN` on line 109** uses the same bound chain and gives a `reads` edge.
  It is left to [part 4](part-4-signatures-other-edge-kinds.md) so this part stays one edge.
