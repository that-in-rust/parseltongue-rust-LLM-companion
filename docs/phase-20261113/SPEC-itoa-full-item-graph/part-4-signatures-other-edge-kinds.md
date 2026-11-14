# Part 4 — signatures and the other edge kinds

**State: 4a steps 1 to 4 of 7 BUILT (edges to items, `Self` included). 4b to 4d are stubs.**

This part adds the four edge kinds that have no producer. They are picked one at a time:

```text
4a  uses_type from signatures   REQ-RUST-018.2   item targets and Self built; builtins, unresolved next
4b  implements from impl headers                 stub
4c  reads of consts and statics                  stub
4d  imports from use nodes                       stub
```

**4a target:** `Buffer::new --uses_type--> Buffer`, from this source:

```rust
pub fn new() -> Buffer {            // src/lib.rs:98
```

**What it takes:** no new table and no type inference. Name resolution already stores one
resolved row per type path in a signature; signature rows already exist per parameter,
return type, field and bound. The missing piece is the join: which path belongs to which
item, and whether the path is a type.

## 1 — Work mode

Parser and graph work in a CLI pipeline. Delivery Mode with Reliability Mode: the risk is a
wrong or duplicated edge, not ambiguity about what is wanted.

## 2 — Executable requirements

**REQ-RUST-018.2 — type uses in signatures.** "Signature row" means a parameter, self
parameter, return type, struct or tuple field, generic parameter or bound row of an item that
is a graph node and was written in original source.

- **R1.** WHEN a type path inside a signature row resolves to exactly one item of the
  analysed crates, THEN the graph SHALL store one `uses_type` edge from the row's owner to
  that item, with the path's own source range as its site.
- **R2.** WHEN the path is `Self` inside an impl, THEN the target SHALL be the item the impl
  header's self type resolves to. WHEN that self type is unresolved, THEN the site SHALL be
  stored unresolved with the header's reason.
- **R3.** WHEN the path resolves to a builtin type, THEN the graph SHALL store the edge to
  that builtin's identity, and one node of category `Builtin` per builtin name used.
- **R4.** WHEN the path resolves to a type parameter, or is `Self` inside a trait
  declaration, THEN no row SHALL be stored: it names no item.
- **R5.** WHEN name resolution left the path unresolved, THEN one unresolved `uses_type` row
  SHALL be stored with one reason: `ExternalSourceMissing` when resolution recorded missing
  source, `AmbiguousTarget` for several targets, `TraitSolverNeeded` otherwise.
- **R6.** WHEN a path inside a signature row is an expression, not a type (an array length,
  a const generic argument), THEN it SHALL NOT produce a `uses_type` row.
- **R7.** WHEN the same owner names the same type at two sites, THEN two edges SHALL be
  stored. Nested paths (`MaybeUninit<u8>`) are separate sites.
- **R8.** WHEN the row's owner is inside a function body, or the row came from a macro
  expansion, THEN no row SHALL be stored by this part (macro rows: part 5).
- **R9.** WHEN the same stored rows are read twice, THEN the edges SHALL be identical and in
  the same order.

## 3 — Design by layer

```text
L1  parseltongue-core-row-types          no change: HldEdgeInputRecord, UsesType, Builtin exist
L2  parseltongue-syntax-span-extractor   extract_signature_type_paths   (syntax, pure)
                                         resolve_signature_type_edges   (reads stored rows)
L3  parseltongue-cli-dispatcher-binary   full-graph-checkpoint-facade adds the edges
    stores                               no change
```

- **Syntax (pure).** Each signature row gains `type_path_source_rows`: the source range and
  text of every `PATH_TYPE` under it, and whether it is a bare `Self`. Paths under an
  expression are not listed (R6).
- **Resolver.** `resolve_signature_type_edges(store, source_scope, input, cancel)`, the same
  shape as `resolve_original_call_edges`. It reads `field-signature-detail-table`,
  `path-resolution-site-records` and `impl-header-index-table` through the existing
  `StageTableStorePort` trait, so fixtures can stand in for the store.
- **Errors.** One `thiserror` enum, `SignatureTypeEdgeError`: cancelled, store, identity,
  invalid evidence field, scope mismatch. No free-text variant.
- **No guessing.** The resolver never looks a name up itself; it only joins rows that name
  resolution and the impl-header stage already wrote.

## 4 — Verification matrix

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| 018.2 R1 | test_new_return_type_edge | real source, CLI | `Buffer::new --uses_type--> Buffer`, site is `Buffer` on line 98, read back frozen | exactly 1 edge |
| 018.2 R2 | test_clone_self_type_edge | real source, CLI | `<Buffer as Clone>::clone --uses_type--> Buffer`, site is `Self` on line 88 | exactly 1 edge |
| 018.2 R3 | test_signature_builtin_type_edges | real source, CLI | `private::Sealed::write --uses_type--> str`; one `Builtin` node for `str` | 1 edge, 1 node |
| 018.2 R5 | test_signature_external_type_unresolved | real source, CLI, sysroot excluded | `MaybeUninit` in `enc_16lsd` is stored unresolved | `ExternalSourceMissing` |
| 018.2 R1 R7 | test_signature_paths_listed_once | fixture | params, return, fields, bounds; nested generic arguments | one entry per type path |
| 018.2 R6 | test_expression_paths_not_types | fixture | `[u8; N::LEN]` and `Foo<{ LEN }>` | no entry for the expression |
| 018.2 R4 | test_type_parameter_no_edge | fixture | `fn f<T>(t: T)`; `Self` in a trait method | 0 rows |
| 018.2 R2 | test_self_unresolved_header_reason | fixture | `Self` in an impl whose self type has no source | unresolved, header's reason |
| 018.2 R7 | test_repeated_type_two_edges | fixture | `fn f(a: S, b: S)` | 2 edges, 2 sites |
| 018.2 R8 | test_body_and_macro_rows_skipped | fixture | item inside a function body; row with an expansion chain | 0 rows |
| 045.0 | test_original_member_graph_roundtrip | real source, CLI | existing test still passes | pass |

## 5 — Implementation plan

Each step is its own commit of about 200 lines, with a log entry and the workspace gates
before its push.

1. RED: `test_new_return_type_edge` on pinned itoa. Record what is stored today.
2. Syntax: `type_path_source_rows` on signature rows, with the R6 and R7 fixture tests first.
3. Resolver for targets that are items (R1, R4, R7, R8, R9) and the facade call. Step 1 GREEN.
4. `Self` through the impl header (R2).
5. Builtin targets and their nodes (R3). Needs Q1.
6. Unresolved rows with reasons (R5).
7. Refactor, gates, re-measure counts and one run's cost (whole process tree).

## 6 — Quality gate results

**After step 3**, on pinned itoa with the sysroot excluded:

- Workspace gates pass: format, lint, release build; 450 tests passed, 0 failed, 4 ignored;
  `test_lang_items_in_sysroot` skipped as before. The 14 real-source tests pass (140.9 s).
- The graph holds 45 nodes and 111 edges (was 104): 7 `uses_type` edges, all to items.
- All 7 are rows of the expected graph: `Buffer::new` and `default` to `Buffer`;
  `Buffer::format` and `Unsigned` to `Integer`; `Integer` to `private::Sealed`;
  `Sealed::write` and `Unsigned::fmt` to `private::Sealed::Buffer` (written `Self::Buffer`).
- Not stored yet, by step: 1 `Self` (step 4), 21 builtin sites (step 5), 6 unresolved
  sites (step 6: 4 `MaybeUninit`/`Copy` with missing source, `Self::Buffer` in
  `impl Unsigned for u128`).
- Correction: this part first said 9 edges to items. Two of those nine were impl-header
  paths, which belong to 4b. The signature count is 7.
- Not measured: one run's cost.

**After step 4:** `<Buffer as Clone>::clone --uses_type--> Buffer` is stored, with the site
on `Self`; `test_clone_self_type_edge` passes. Gates pass: 452 tests passed, 0 failed,
4 ignored; the 15 real-source tests pass (138.3 s). The edge counts were not re-read; one
more edge than step 3 is expected (112), since itoa has one `Self` in an original signature.
A `Self` whose impl header is unresolved stores nothing until step 6.

## 7 — Open questions

- **Q1: builtin targets need nodes. Lean: yes, category `Builtin`.** The validator rejects an
  edge whose target is not a node. The expected graph counts 73 item nodes and treats
  builtins as targets only, so the comparison must count nodes by category. Needed at step 5.
- **Q2: OD-8, non-function sources.** Fields and bounds make a struct or trait the source of
  `uses_type` (`Buffer --uses_type--> u8`). The backlog leans yes and the expected graph
  already has these rows, so R1 includes them.
- **Q3: declared types of consts and statics** (`MAX_STR_LEN: usize`) have no signature row
  today. Not in 4a; they need a row first.
- **Q4: `Self::Buffer` inside an impl** is left unresolved by name resolution with no reason.
  4a stores it as `TraitSolverNeeded` (R5) rather than resolving the projection.

## Stubs: 4b to 4d

- **4b `implements`.** One edge per impl header from the self type to the trait. Impl headers
  already resolve both. Open: the source when the self type is a builtin (`impl Unsigned for
  u128`), and trait targets in `core` (part 6).
- **4c `reads`.** Paths to consts and statics in bodies and initializers
  (`I::MAX_STR_LEN`, `DECIMAL_PAIRS.0`). Part of REQ-RUST-032.0.
- **4d `imports`.** One edge per `use` node to what it names. All four in itoa point into
  `core` (part 6).

REQ-RUST-032.0 (two tables, every item a node, every proven dependency one uncollapsed
edge) is met only when 4a to 4d and parts 5 to 7 are.
