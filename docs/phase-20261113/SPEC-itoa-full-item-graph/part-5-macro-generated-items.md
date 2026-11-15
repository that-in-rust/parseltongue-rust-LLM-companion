# Part 5 — macro-generated items as graph nodes

**State: 5a BUILT, except nested expansion (R4), which stops the run with a typed error.
5b BUILT (section 8). 5c BUILT (section 9). 5d SPECIFIED, steps 1 and 2 of 6 BUILT (section 10).
5e is not specified. Rules R2, R3 and R5 decided by the user on 2026-11-15
("most explicit for an agent reading our artifact").**

## The answer

The store already holds every macro-generated item of itoa; one SQL filter keeps them out
of the graph. Part 5 lets them in, in five sub-parts, starting with nodes and containment.

- **Measured** on one release run at commit `499e04559` (store exported with
  `export-published-stage-rows`):
  - `span-reference-text-table` has 148 rows; 93 come from expanded text.
  - Of those 93, 45 are active items that are not impl blocks: 17 `CONST`, 16 `FN`,
    12 `TYPE_ALIAS`. The expected graph has 45 generated nodes of the same kinds
    (5 `const _`, 12 `MAX_STR_LEN`, 12 `Buffer`, 12 `write`, 4 `fmt`).
  - The other 48: 28 active impl blocks (records, not nodes), and 20 spans under
    `#[cfg]` that is off (the four `impl_Integer_size!` calls for 16 and 32 bit).
  - Each generated span already has an identity: its unit, the path
    `.parseltongue/macro-source/<call digest>/expanded.rs`, its byte range in the expanded
    text, its kind, and the chain of call digests.
  - The filter is `original-item-graph-projection.sql` line 11 (chain length = 0).
- **Measured after 5a** at commit `bbdb98524` (3 release runs): 103 nodes, 88 `contains`
  rows, 45 `generates` rows, 285 edges in all. The prediction was the same.
  - 11 of the 15 expanded calls write an active item: five `impl_Integer!` (7 each), two
    `impl_Integer_size!` (3 each), four `impl_Unsigned!` (1 each). The other four
    `impl_Integer_size!` calls write only items under a `#[cfg]` that is off.
  - All 15 expanded calls are one level deep. itoa has no nested expansion.
- **What the expected graph has for these items:** 45 `contains` rows, and 202 edges that
  start at a generated item (80 `calls`, 66 `uses_type`, 56 `reads`). 4 more end at one.

## Sub-parts

```text
part  target                                              rows on itoa (predicted)   state
5a    generated items as nodes, contained, with their     45 nodes, 45 contains,     BUILT (measured)
      generating call                                     45 generates
5b    labels: impl header and macro call text in the      103 labels, all distinct   BUILT (measured)
      node's path text
5c    `expands`: macro call -> macro definition          15 rows                    BUILT (measured)
5d    generated bodies and signatures as edge sources:    up to 202 rows             step 1 of 6 BUILT
      reads, then uses_type, then calls
5e    first chain: <u128 as Sealed>::write ->             3 call rows                not specified
      <u128 as Unsigned>::fmt -> div_rem_1e16 -> mulhi
```

## 1 — Work mode

Brownfield. `derive_original_graph_input` and its SQL are extended; no new crate. The
resolvers of parts 2 to 4 are not touched in 5a.

## 2 — Executable requirements (5a)

**REQ-RUST-046.0 — macro-generated items as nodes.**

- **R1 (which items).** WHEN a span has a non-empty expansion chain, its `#[cfg]` is
  active, its kind is one the graph has for original items, and it is not inside a function
  body or an initializer, THEN the graph SHALL hold one node for it. Impl blocks stay
  records.
- **R2 (identity; decided).** The node identity SHALL be the stored occurrence key: unit,
  expansion path relative to the package root, byte range in the expanded text, kind, chain.
  Two calls of one macro give two nodes.
- **R3 (container; decided).** WHEN a generated item is at the top of an expansion or is
  a member of a generated impl, THEN one `contains` row SHALL go from the file area or
  inline module that holds the outermost macro call to the item. The macro call node is not
  its container. (Same rule as original impl members, REQ-RUST-043.0.)
- **R4 (nested expansion; NOT BUILT).** WHEN the chain has more than one call, THEN the
  container SHALL come from the first call of the chain, the one written in a source file.
  Today the generating call (R5) is found only when it is written in a source file. A call
  written by another macro has no node yet, so the projection stops with
  `InvalidSourceRow` and writes no graph. It does not guess a call. See Q5.
- **R5 (generating call; decided: both forms).** Each generated node SHALL name the macro
  call node that produced it (the last call of its chain) in two ways: a field on the node
  row, and one `generates` row from the call node to the item, sited at the call.
  `generates` is a new relation kind; it is never collapsed and never counted as a
  dependency between items. The validator rejects a generated node without a call, a
  source-written node with one, a named call that is not a `MacroInvocation` node, and a
  `generates` row whose source or site is not the call its target names.
- **R6 (evidence).** The node's source text SHALL equal the stored expanded text at its
  range, checked by digest; a mismatch SHALL be a typed error, not a skipped row.
- **R7 (inactive).** WHEN the span's `#[cfg]` is off, THEN no node and no row.
- **R8 (owner kind).** A member of a generated impl SHALL have owner kind `Impl` and SHALL
  be checked against its row in `impl-header-index-table`, as original members are.
- **R9 (counts).** The stage line SHALL print how many generated items became nodes and how
  many were left out, by reason.
- **R10 (unchanged rows).** The 58 nodes and 195 edges stored today SHALL be unchanged.
- **R11 (repeat).** Two reads of one store SHALL give the same nodes and rows.

Errors: the existing `OriginalSourceProjectionError`; no new variant unless R6 needs one.

## 3 — Design by layer

- **L1 `parseltongue-core-row-types`.** Nothing for R1 to R4: `validate_full_graph_input`
  rejects an expansion chain only for the older top-level-containment coverage (line 682).
  R5 as a field adds one optional field to `HldNodeInputRecord`.
- **L3 `parseltongue-duckdb-analysis-adapter`.**
  - SQL: a second span view for chain length > 0, joined to
    `macro-expanded-item-table` on the first call digest to get the call site's file and
    lexical container. The ancestors view joins on equal file paths, so it cannot be reused
    as it is.
  - Rust: `build_original_item_identity` checks text against a real file's bytes. A
    generated item is checked against `macro_expanded_source_text`.
  - The stored path of an expanded span is absolute (measured:
    `/tmp/4d-perf/itoa2/.parseltongue/...`). The node identity uses the relative form.
- **Known limit, same as original items:** the unit key holds the Cargo package id, which
  holds the checkout path. So an identity is stable between runs of one checkout, not
  between checkouts.

## 4 — Verification matrix (5a)

```text
rule     test                                     kind          where
R1 R2    test_generated_const_node_contained      real source   itoa-source-acceptance-tests
R3       test_generated_const_node_contained      real source   itoa-source-acceptance-tests
R1 R7    test_generated_item_node_counts          real source   itoa-source-acceptance-tests
R5       test_generated_node_names_call           real source   itoa-source-acceptance-tests
R5       test_generates_edge_row_counts           real source   itoa-source-acceptance-tests
R5       test_generated_node_call_required        unit          generated-node-validation-tests
R5       test_generating_call_node_kind           unit          generated-node-validation-tests
R5       test_original_node_call_rejected         unit          generated-node-validation-tests
R5       test_generates_edge_call_accepted        unit          generated-node-validation-tests
R5       test_generates_edge_site_checked         unit          generated-node-validation-tests
R5       test_generates_edge_source_checked       unit          generated-node-validation-tests
R10      test_generated_item_node_counts          real source   itoa-source-acceptance-tests
R11      test_generated_nodes_read_twice          real source   itoa-source-acceptance-tests
R1 R8    test_expanded_item_projected_node        fixture       generated-item-projection-tests
R4       test_nested_expansion_call_rejected      fixture       generated-item-projection-tests
         (pins the typed rejection; R4 itself has no passing test)
R6       test_expanded_text_digest_mismatch       fixture       generated-item-projection-tests
R7       test_inactive_expanded_item_skipped      fixture       generated-item-projection-tests
R1       test_expanded_body_item_evidence         fixture       generated-item-projection-tests
R1       test_reject_orphan_generated_span        fixture       original-item-projection-tests
R9       (no test) stage line stage=generated-item-source-inventory, read on one run:
         impl-record 28, inactive 20, included 45
```

## 5 — Implementation plan (5a)

1. RED, real source: `test_generated_const_node_contained`. One `Constant` node named `_`
   whose chain names the call `impl_Integer!(i8, u8)`, with one `contains` row from the
   file area of `src/lib.rs`. This is the smallest slice: 1 node, 1 row.
2. STUB then RED on fixtures for R1, R4, R6, R7, R8.
3. GREEN: the SQL view and the identity for generated items; step 1 passes.
4. GREEN: all 45; `test_generated_item_node_counts` (45 nodes by kind, 20 inactive, 58 old
   nodes unchanged).
5. R5: the node field, then the `generates` relation kind and its 45 rows. Each is its
   own RED and GREEN.
6. REFACTOR, workspace gates, cost of one run, this file and the log.

All six steps are done, except that step 2 was not done as written: the fixture tests
were written after the code and passed on first run, and R4 has a rejection test only.

Tests that pin today's behaviour and will change in 5a or 5d (named by the second review,
not yet read by me): `original-item-projection-tests.rs` lines 203 to 210,
`test_expanded_body_read_counted`, `test_expanded_header_keeps_chain`,
`test_itoa_reads_row_counts`.

## 6 — Quality gate results

Measured at commit `bbdb98524`, release build, one thread.

```text
gate                                   result
cargo fmt --all -- --check             pass
clippy, workspace, -D warnings         pass
cargo build --release --workspace      pass
workspace tests                        514 passed, 0 failed, 4 ignored
                                       (test_lang_items_in_sysroot skipped: not a full PASS)
real-source itoa tests                 33 passed, 461 s
adapter tests                          175 passed
one run, 3 times                       13.7 to 15.9 s; 359 to 378 MB peak; 12.88 MB store
stored                                 103 nodes, 285 edges, marked incomplete
```

Tests that passed on first run and so were not seen failing: `test_generated_item_node_counts`,
`test_generated_nodes_read_twice`, and the five fixture tests in
`generated-item-projection-tests`.

## 7 — Open questions

- **Q1 (closed). How does a generated node name its call?** Both: a `generates` edge and a
  node field (R5). The expected graph gains the same 45 rows when it is repaired.
- **Q2. Where is the range in the macro definition stored?** The expected graph sites a
  generated node in the definition body, so five nodes share one range. Production sites
  it in the expanded text. `macro_token_origin_rows` maps one to the other. Part 7 cannot
  compare ranges without it. Lean: a side table keyed by node identity, built in 5b; the
  evidence key is not widened.
- **Q3. Expanded files have no "file missing" or "item without owner" check** in
  `hld-original-source-projection.sql` (second review; not yet read by me).
- **Q5. Nested expansion.** R4 is not built. To build it, a macro call written inside an
  expansion must itself be a node (identity: its range in the expanded text and its chain),
  generated by the call before it. Then the last call of any chain is a node and R5 holds
  for every depth. itoa has no such call, so this waits until a repo needs it.
- **Q6. `contains` and `generates` say different things and both stay.** The file area
  contains the item (where a reader finds it); the call generates it (what to edit).
- **Q4. The expected graph has no macro call or definition nodes.** It gains them when it
  is repaired, per the user's decision.

Second opinion (Rule 5): GLM-5.3 through pi, max reasoning, read-only, on this design
before it was written. Record:
`docs/devin-research/1794745800-second-opinion-glm-5-3-max-part-5-macro-items-design.md`.

## 8 — Part 5b: readable node labels (REQ-RUST-050.0)

**State: BUILT at `47d4d05e0`.** The numbers in 8.1 are measured on one release run at
`4ea998560`, before the change; the numbers in 8.6 are measured after it.

### 8.1 The problem (measured)

`node_qualified_path_text` is the one text a reader sees for a node. Today it is
`<cargo package id>::<file>::<module and trait names>::<declared name, or the syntax kind>`.
On itoa only 45 of the 103 labels are distinct:

```text
nodes  label today                      what they are
15     ...::src/lib.rs::MACRO_CALL      the 15 macro calls
3      ...::src/lib.rs::USE             the 3 use items
5      ...::src/lib.rs::_               5 generated `const _`
12     ...::src/lib.rs::MAX_STR_LEN     12 generated impl consts
12     ...::src/lib.rs::Buffer          12 generated impl types
12     ...::src/lib.rs::write           12 generated impl functions
5      ...::src/lib.rs::fmt             1 original and 4 generated impl functions
4      ...::new, format, default, clone original impl members, no impl named
```

The prefix is `path+file:///<checkout>/itoa#1.0.18`, which changes with the checkout.
An impl block is not a node, so the label is the only place a reader sees which impl a
member belongs to.

### 8.2 Rules

The label is display text. It is not the identity, is never parsed by production code, and
need not be unique. Every part of it is copied from stored source text; nothing is guessed.

```text
L1  Prefix: <crate target name>::<file path>, then the names of enclosing modules and
    traits, as today. The crate target name (`itoa`) replaces the package id; the package
    id stays in the node identity. A file area reads <crate>::<file>::<file-area>.
L2  Impl member, original or generated: one segment from its impl header before the name.
    Trait impl: <SelfText as TraitText>. Inherent impl: SelfText. Both are the header's
    own source text (`type_source_text`), normalized by L6.
    A header with no readable self text gives the segment IMPL (the syntax kind).
L3  Macro call: the last segment is the call's own source text, normalized by L6.
L4  Use item: the last segment is its own source text, normalized by L6.
L5  A const with no name: `const _`. If a macro call wrote it: `const _ #` followed by
    that call's L3 segment. Other generated items do not repeat the call text; they carry
    the call as a node field and a `generates` row.
L6  Text normalization, in order: drop leading lines that start with `//`; turn every
    whitespace run into one space; drop spaces next to `::`; drop one trailing `;`;
    if the result is longer than 160 characters keep the first 160 and add ` …`.
L7  Any other item: its declared name with a leading `r#` removed, or the syntax kind if
    it has none (unchanged).
L8  A builtin type node (`u8`, `str`) keeps its bare name (unchanged; it has no file).
```

Examples on itoa (measured, read back from the stored table):

```text
itoa::src/lib.rs::<u8 as private::Sealed>::write
itoa::src/lib.rs::<Buffer as Default>::default
itoa::src/lib.rs::Buffer::new
itoa::src/lib.rs::impl_Integer!(i8, u8)
itoa::src/lib.rs::use core::mem::{self, MaybeUninit}
itoa::src/lib.rs::const _ #impl_Integer!(i8, u8)
```

### 8.3 Verification matrix (5b)

```text
test                               level        pins
test_node_labels_all_distinct      real itoa    103 labels pairwise distinct; none contains file:///;
                                                macro call and use labels equal the file's own bytes
test_impl_member_label_segment     real itoa    exact labels of original and generated impl members;
                                                `const _` labels end with their call's segment
test_impl_header_label_segment     fixture      L2: trait impl, inherent impl, expander spacing, no self text
test_source_text_label_rule        fixture      L6: comments, whitespace, `::`, `;`, the 160 cut
test_unnamed_const_label_text      fixture      L3, L4, L5, L7: call, use, `const _`, `const _ #call`, `r#`, kind fallback
test_label_crate_name_prefix       fixture      L1: crate target name first, the package id nowhere
```

### 8.4 Plan

1. Stub `item-node-label-text.rs` in the DuckDB adapter; failing fixture tests.
2. Failing real-source tests.
3. Code; both projections call the one helper.
4. The two 5a tests that find `const _` by the suffix `::_` find it by kind.
5. Gates, three measured runs, log, PRD.

### 8.5 Open questions (5b)

- **Q10. L2 on a negative impl or an impl whose trait text is unreadable gives `IMPL`.**
  Only the no-self-text case has a test. itoa has neither.
- **Q11. The 160 cut can make two long labels equal.** Labels need not be unique (8.2), and
  itoa's longest label is 88 characters, so the cut is tested on a fixture only.

- **Q7. Expected-graph labels differ in form.** They carry no crate and file prefix
  (`u128_ext::mulhi`), and write `const _ #impl_Integer(i8,u8)`. Part 7 decides how labels
  are compared; the expected graph is repaired to the stored form there.
- **Q8. No typed link from an impl member to its impl header.** The label segment is the
  only one. A node field naming the header (`node_impl_header_reference`) is in the backlog.
- **Q9. Q2 (range in the macro definition) moves to 5d**, where generated bodies need it.

### 8.6 Quality gate results (5b, measured at `47d4d05e0`)

```text
labels read back from the stored table      103, 103 distinct, 0 containing file:///
                                            (90 item and file-area labels, 13 builtin types)
fixture tests, new                          4 passed; all 4 failed on the stub first
real-source tests, new                      2 passed; both failed first (45 distinct of 103;
                                            no `itoa::src/lib.rs::<file-area>` label)
adapter tests                               179 passed, 0 failed
real-source itoa tests                      35 passed, 0 failed, 448 s, release, one thread
format, lint (adapter and CLI crates)       pass
one run, 3 times                            13.9 to 18.8 s, 336 to 353 MB peak, 12.9 MB store
graph                                       103 nodes, 285 edges, unchanged; still incomplete
workspace gates                             not run for 5b; they run at the end of part 5
```

Three older tests named a node by its old label and were changed to the new form:
`test_generated_item_node_counts` and `test_generated_node_names_call` (real source), and
`test_preserve_known_syntax_kinds` (fixture; its macro call no longer has an invented name).

Second opinion (Rule 5): GLM-5.3 through pi, max reasoning, read-only, before this section
was written. Record: `docs/devin-research/1794768300-second-opinion-glm-5-3-max-part-5b-node-labels.md`.

## 9 — Part 5c: `expands`, macro call to macro definition (REQ-RUST-051.0)

**State: BUILT** at `b0733521f` (section 9.6). Numbers in 9.1 are measured on the stored
tables of one release run at `47d4d05e0`; 9.3 states the prediction and the measured result.

### 9.1 What is stored today (measured)

```text
MacroInvocation nodes                           15, all written in src/lib.rs
MacroDefinition nodes                           3: impl_Integer, impl_Integer_size, impl_Unsigned
legacy-macro-scope-records (published)          94 rows; one row is one lookup of a macro name
                                                in textual scope: the site, the name, and the
                                                definition in scope there, or null
  rows whose site is the path of one of the     15 distinct, each with exactly one definition,
  15 calls                                      and that definition is one of the 3 nodes
  calls per definition                          impl_Integer 5, impl_Integer_size 6, impl_Unsigned 4
macro-expanded-item-table, state expanded       15 rows, each with the arm matched (all 0) and,
                                                per output token, where it was copied from
```

So no new producer is needed: the graph step joins two stored tables.

### 9.2 Rules

```text
E1  A tenth relation kind, expands: macro call node -> macro definition node. It is a
    structural relation like generates; it is not a dependency and is never collapsed.
E2  Every macro call node that is written in a source file has exactly one expands row.
E3  Site: the whole macro call, the same site as its generates rows.
E4  Target: the definition that the stored textual-scope lookup for the call's macro path
    names. A call has one stored lookup per scope revision; the row with the highest
    revision is the one in force, as in the existing name resolver. Two lookups at that
    revision naming different definitions are a typed error. (Revised when built: the first
    wording required all revisions to agree.)
E5  Cross-check: every output token the expansion row says was copied from a definition
    must lie inside the target definition's byte range, in the same file. A mismatch is a
    typed error, never a row.
E6  A call with no definition in scope and no expansion gets one unresolved row. The reason
    is the one stored on its expansion row (ExternalSourceMissing for a macro in core), or
    MissingTarget when no row exists.
E7  A call that was expanded but has no stored lookup naming a definition (a macro called
    by a path such as crate::m!) stops the run with a typed error. Not built; itoa has none.
E8  The row's metadata is JSON: {"macro_name_text": .., "macro_matched_arm_index": N}; the
    index is present only when the call was expanded. It tells a reader which arm to edit.
E9  The validator rejects an expands row whose source is not a macro call node, whose
    target is not a macro definition node, or whose site is not the source call.
E10 The stage line prints expands=N.
```

A call written by a macro (nested expansion) is not a node (section 2, R4), so it has no row.

### 9.3 Predicted on itoa, and measured

15 `expands` rows, all with a target, split 5 / 6 / 4; arm index 0 on all 15; 300 edges
(285 + 15); nodes unchanged at 103. The expected graph has no macro nodes, so these 15 rows
match nothing in it until part 7 repairs it.

Measured at `b0733521f`, read back from the stored edge table: 15 `expands` rows, all with a
target, 5 to `impl_Integer`, 6 to `impl_Integer_size`, 4 to `impl_Unsigned`; arm index 0 on
all 15; 103 nodes; 300 edges. The prediction held.

### 9.4 Tests

```text
test                                kind         proves
test_expands_edge_row_counts        real source  E1-E5, E8: 15 rows, 5/6/4, site, arm 0, 300 edges
test_expands_row_endpoint_kinds     fixture      E9: wrong source kind, wrong target kind, wrong site
test_expands_unresolved_call_row    fixture      E2, E6: a call with no definition gets one unresolved row
test_expands_token_origin_mismatch  fixture      E5: a copied token outside the definition stops the run
test_expands_call_definition_row    fixture      E2-E4, E8: one call, one definition, one row with arm index
test_expands_latest_scope_revision  fixture      E4: the lookup at the highest scope revision names the target
test_expands_pathless_expansion_rejected  fixture  E7: an expanded call with no stored lookup stops the run
```

The last three were added when the part was built.

### 9.5 Open questions (5c)

- **Q12.** E7: macros called by path. Needs the resolved binding stored per call.
- **Q13.** A builtin or `include!` call at item position: unresolved by E6; no fixture yet.
- **Q14.** Closed when built: the lookup at the highest scope revision is used (E4), so a
  later definition that shadows an earlier one is followed, not rejected.
- **Q15.** The validator checks one row at a time (E9). It does not check that every macro
  call node has exactly one `expands` row, or the shape of the metadata; the real-source
  test pins both on itoa only.
- **Q16.** E5 is checked in the producer, not in the validator, so a graph written by
  another producer is not covered by it.

Second opinion (Rule 5): GLM-5.3 through pi, max reasoning, read-only:
[record](../../devin-research/1794770900-second-opinion-glm-5-3-max-part-5c-expands-edge.md).

### 9.6 Quality gate results (5c, measured at `b0733521f`)

```text
expands rows read back from the stored table  15, all with a target: 5 / 6 / 4 by definition;
                                              arm index 0 on all 15; site = the macro call
fixture tests, new (validator)                1 passed (5 cases); failed on the stub first
fixture tests, new (adapter)                  5 passed on first run. Checked by mutation: with
                                              the revision filter and the token check disabled,
                                              test_expands_latest_scope_revision and
                                              test_expands_token_origin_mismatch fail. The other
                                              three were not seen failing.
real-source test, new                         test_expands_edge_row_counts passed; failed first on
                                              `not yet implemented: REQ-RUST-051.0`
core tests                                    22 passed
adapter tests                                 184 passed, 0 failed
Turso adapter tests                           71 passed, 0 failed, 3 ignored
real-source itoa tests                        36 passed, 0 failed, 80 s, release, default threads
format (workspace), lint (4 touched crates)   pass
one run, 3 times                              14.9 to 15.8 s, 343 to 361 MB peak, 13.1 MB store
graph                                         103 nodes, 300 edges (was 285); still incomplete
workspace gates                               not run for 5c; they run at the end of part 5
```

One defect found on the way: the first SQL read the call's compilation unit from a JSON
field that is empty in the stored lookup rows, so no call joined to its lookup and the run
stopped with the E7 typed error. The unit is now read from the lookup's site key.

## 10 — Part 5d: generated items as edge sources (REQ-RUST-052.0)

**State: SPECIFIED; step 1 BUILT** (section 10.6). Section 10.1 is measured on stored tables of
runs at `255163ea0` with the filters named there lifted by hand and then restored.

### 10.1 What is stored today (measured)

```text
producer        filter that drops rows from expanded text                         rows with it lifted
reads           body-value-read-records.rs, the expansion-chain test on a site    +34: 24 with a target,
                                                                                  10 without
uses_type       signature-type-edge-records.rs, the same test on a signature row  +32: 28 with a target,
                                                                                  4 without
calls           original-call-resolution-records.rs, the same test on a call site +136, all without a
                                                                                  target
all three lifted                                                                  502 edges (300 + 202)
```

- The stored tables already hold the sites: `body-site-reference-table` and
  `field-signature-detail-table` have rows for expanded text, each with its owner, its range
  in the expanded text and the chain of call digests. Name resolution has run on them.
- The 24 reads with a target are `DECIMAL_PAIRS`, 6 in each generated `<uN as Unsigned>::fmt`.
  The 10 without are `Self::MAX_STR_LEN` and `uN::MAX_STR_LEN` in generated `write` bodies.
- The 4 signature rows without a target are `Self::Buffer` in `impl Unsigned for uN`:
  `Buffer` is declared by `private::Sealed`, a supertrait.
- All 136 calls fall into one catch-all that gives `TraitSolverNeeded`; their metadata has no
  callee text and says `original_associated_function`.
- `macro-expanded-item-table` stores, per output token of an expansion, the byte range it was
  copied from in the source file and whether that is the macro definition or the call.
- The expected graph has 202 rows from generated sources: 76 `reads` sites (56 grouped),
  66 `uses_type`, 136 `calls` sites (80 grouped). The three producers give 34, 32 and 136
  of them with the filters lifted (the call producer already reads const initializers); the
  other 42 `reads` sites and 34 `uses_type` rows start at generated consts and type aliases.

### 10.2 Rules

```text
G1  A node written by a macro expansion is an edge source on the same terms as a node written
    in a source file: reads and calls from its body, uses_type from its signature.
G2  Site: the occurrence in the expanded text (the virtual path, the range there, the digest
    of the site's text, the chain of call digests). A site and its owner are in the same
    text; a site whose file or chain differs from its owner's is a typed error, never a row.
    The site is not moved to the source file: the digest is of the text the edge read, and
    no range of the source file has to hold that text (u8::MAX_STR_LEN is one token from
    the call and two from the definition).
G3  A row sited in expanded text carries, in its metadata, where its first token and its
    last token were copied from: file, byte range, and definition or call. The two are
    stored apart, never merged into one range. This is where an agent edits.
G4  Targets follow the existing rules. A path that name resolution left without a target is
    one row with a reason and its source text.
G5  A member reached through a type (Self::NAME, u8::NAME): the target is a trait member only
    when a trait of this crate declares NAME and a stored impl header proves the type
    implements that trait, directly or through a stored supertrait row. The row records what
    was not read (the inherent items of the type). No trait of this crate declaring NAME
    (MIN, MAX, BITS, ilog10): the rule does not fire.
G6  A calls row carries the callee's source text. Its dispatch kind does not say "original"
    for a call written by a macro. The reason for a missing target is the stored one, not a
    catch-all.
G7  Generated consts and type aliases are sources too: the type in the declaration and the
    paths and calls in an initializer.
G8  The stage line's reads_in_expanded_bodies is the number of stored reads rows sited in
    expanded text.
```

### 10.3 Steps, predicted and measured on itoa

```text
step  what                                         rules    predicted               measured
1     reads from generated function bodies         G1, G2,  55 reads, 334 edges     55 reads (39 with a target,
                                                   G4, G8                           16 without), 334 edges
2     origin of first and last token, on reads     G3       34 rows gain metadata   34 rows: 29 definition to
                                                                                    definition, 5 call to
                                                                                    definition; 334 edges
3     uses_type from generated signatures          G1-G4    66 uses_type, 366 edges not built
4     calls from generated bodies                  G1-G4,   197 calls, 502 edges    not built
                                                   G6
5     trait members through a type                 G5       16 reads and 4 uses_type not built
                                                            gain a target
6     generated consts and type aliases as sources G7       up to 42 reads sites,   not built
                                                            34 uses_type
```

Step 2 was planned after step 4; the second opinion moved it forward so steps 3 and 4 land
with it.

### 10.4 Tests

```text
test                                kind         proves
test_generated_fmt_static_reads     real source  G1, G2, G4: each of the 4 generated fmt has 6 reads of
                                                 DECIMAL_PAIRS sited inside its own expanded text; 55
                                                 reads; 10 generated rows without a target
test_expanded_site_owner_mismatch   fixture      G2: a site whose chain differs from its owner's stops
                                                 the run
test_itoa_reads_row_counts          real source  changed: counts the 21 rows sited in original text
test_reads_rows_read_twice          real source  changed: 55 rows, read twice, none twice
```

Step 2:

```text
test                                kind         proves
test_generated_read_source_origins  real source  G3: each of the 34 generated reads names a first and a
                                                 last token in src/lib.rs whose text there starts and
                                                 ends the site's text; 24 DECIMAL_PAIRS from the
                                                 definition; uN::MAX_STR_LEN starts in the call and ends
                                                 in the definition; original rows carry no origin
test_site_token_origin_pair         unit         G3: first and last token found apart; a one-token site
                                                 names it twice
test_site_origin_missing_rejected   unit         G3: no stored expansion, no token inside, or a site
                                                 edge that cuts a token is a typed error
```

`test_expanded_body_read_counted` (REQ-RUST-048.0 R7, "counted and left to part 5") is
removed: R7 is superseded by G1.

### 10.5 Open questions (5d)

- **Q17.** Step 6: do the stored tables hold value-path and call sites for const initializers
  and type aliases? If not, step 6 changes the namespace stage, not only a producer.
- **Q18.** G5 rests on the rule that an inherent item shadows a trait item. `core` is not
  read, so "u8 has no inherent MAX_STR_LEN" is not proven; the row says so. Not checked
  against rustc here.
- **Q19.** The real-source test helper `read_edge_site_range` returns a byte range without
  its file. One test compares ranges alone; steps 3 and 4 must make it compare the file too.
- **Q20.** Expanded text prints `Self ::MAX_STR_LEN` with a space. The row's source text keeps
  the expanded form; no fixture pins it yet.

Second opinion (Rule 5): GLM-5.3 through pi, max reasoning, read-only:
[record](../../devin-research/1794772360-second-opinion-glm-5-3-max-part-5d-generated-edge-sources.md).

### 10.6 Quality gate results (5d step 1)

```text
real-source test, new          test_generated_fmt_static_reads failed first on
                               `<u8 as Unsigned>::fmt left: 0 right: 6`, then passed
fixture test, new              test_expanded_site_owner_mismatch failed first, then passed
syntax crate tests             90 + 17 passed
real-source itoa tests         37 passed, 0 failed, 83 s, release, default threads
format (workspace), lint       pass (2 touched crates)
graph                          103 nodes, 334 edges (was 300); still incomplete
workspace gates, run cost      not run for this step; they run at the end of part 5d
```

### 10.7 Step 2: where a generated site was copied from (measured)

The metadata of a row sited in expanded text has one more key. Read back from the stored
edge table for `u8::MAX_STR_LEN` in the `write` generated by `impl_Integer!(i8, u8)`:

```text
"expansion_source_origin": {
  "site_first_token_origin": { "token_source_relative_path": "src/lib.rs",
                               "token_source_byte_range": [5925, 5927],      text there: u8
                               "token_source_origin_kind": "invocation" },
  "site_last_token_origin":  { "token_source_relative_path": "src/lib.rs",
                               "token_source_byte_range": [5419, 5430],      text there: MAX_STR_LEN
                               "token_source_origin_kind": "definition" }
}
```

```text
rows  site text            first token            last token
24    DECIMAL_PAIRS        definition             definition (same token; 6 places in impl_Unsigned)
5     Self ::MAX_STR_LEN   definition, Self       definition, MAX_STR_LEN
5     uN ::MAX_STR_LEN     call, u8 .. u128       definition, MAX_STR_LEN
```

- **Source:** `macro-expanded-item-table`, the stored origin of every output token. Nothing
  is recomputed.
- **Rule for a site:** the tokens whose output range lies inside the site. A site with no
  stored expansion, with no token inside, or whose edge cuts a token stops the run with a
  typed error.
- **Not built:** the origin on `generates`, `contains` and `expands` rows (their site is the
  macro call in the source file already), and on `uses_type` and `calls` (steps 3 and 4).

```text
real-source test, new          test_generated_read_source_origins failed first on
                               `left: Null right: "src/lib.rs"`, then passed
unit tests, new                2 failed first on `not yet implemented: REQ-RUST-052.0 G3`, then passed
syntax crate tests             92 + 17 passed
real-source itoa tests         38 passed, 0 failed, 91 s, release, default threads
format (workspace), lint       pass (2 touched crates)
graph                          103 nodes, 334 edges (unchanged); still incomplete
workspace gates, run cost      not run for this step; they run at the end of part 5d
```
