# Part 5 — macro-generated items as graph nodes

**State: 5a BUILT, except nested expansion (R4), which stops the run with a typed error.
5b BUILT (section 8). 5c BUILT (section 9). 5d steps 1 to 9 BUILT (section 10). G7 step 10a BUILT (`reads` from consts, statics and types, section 10.15); G7 `uses_type` (10c), the impl member for `<T as Trait>::NAME` (10b) and arguments of builtin macros (`assert!`) are not built.
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
G5  A member reached through a type (Self::NAME, u8::NAME) that name lookup left without a
    target. The type: for Self, the stored self type of the impl that lists the site's owner;
    for any other one-name prefix, its own stored Type lookup at the site. The candidates:
    members named NAME in stored impls of that type, and in stored impls whose self type has
    no stored target. The target is the impl member only when there is exactly one candidate,
    it is in a trait impl of the type with a stored trait, no generic parameter and no
    negation, it has the kind asked for, and it is a graph node. The row carries the lookup
    record either way: candidates, outcome, and what was not read. Section 10.10.

G6  A calls row carries the callee's source text. Its dispatch kind does not say "original"
    for a call written by a macro. The reason for a missing target is the stored one, not a
    catch-all.
G7  Consts, statics, type aliases and structs are sources too, written by a macro or not:
    the type in the declaration and the paths and calls in an initializer or in an
    expression of a type (an array length). An item written in a function sees none of
    the function's parameters or locals.
G8  The stage line's reads_in_expanded_bodies is the number of stored reads rows sited in
    expanded text.
G9  A call whose callee is a path gets its target from the stored name lookup of the whole
    callee path, asked as a Value request in the call's own scope. The target is given only
    when that lookup stores exactly one symbol, of kind Function, that is a graph node, and
    the callee is one name (`f(..)`, `f::<T>(..)`) or its qualifier is stored as a module
    (`m::f(..)`). A qualifier that is a trait or a type names a member: the row keeps the
    record of the whole-path lookup and goes to the type-prefix lookup. Exception: a
    qualifier stored as a builtin type whose whole path has a stored reason takes that
    reason. No request is made for a callee with a `<T as Trait>` anchor, a single name
    bound by a parameter or pattern of the function, or a parenthesised callee.
G10 A callee whose qualifier is stored as a trait and whose whole path stores exactly one
    Function that is a graph node (`Trait::method(x, ..)`) has that function, the trait's
    declaration, as target. The row lists as evidence every function of that name in a
    stored impl of that trait: the impl, the type it is for, and the member. An impl member
    is the target only under G12.
G11 A callee that is one name, or is qualified by a module, and whose path stores exactly
    one Struct that is a graph node (`S(..)`) is a tuple struct's constructor: the struct
    is the target, with its own dispatch kind. A struct found through a trait or a type is
    not a target.
G12 A G10 call whose first argument is the path `self`, written in a method that declares
    `self` by value without a type and is not inside a block, has an impl member as target
    when all of this is stored: the calling method is a member of an impl H for a type
    written as a plain path with a stored target, without generic parameters, not negative,
    not inside a function body; among all stored trait impls that are not shown to be of
    another trait or for another type, exactly one function has the callee's name; that
    function's impl is as plain as H, is of the trait the callee names, and the function
    takes `self` by value; and it is a graph node. The row keeps the G10 candidate list
    and adds the trait's declaration, the member, the impl H and its type. Otherwise the
    G10 row stands.
G13 A method call whose receiver, through any parentheses, is a cast (`(x as T).name(..)`)
    has an impl member as target when all of this is stored: T is one path without generic
    arguments or a type anchor, and its name lookup at the call stores exactly one struct,
    enum, union or builtin; among the stored impls for that type, and those whose type is
    not stored, exactly one has a member `name`; that member is a function taking `self`,
    `&self` or `&mut self`, in a trait impl whose trait is stored, without generic
    parameters, not negative; that trait is the trait of the impl the call is written in,
    that impl is not inside a function body, and the call's nearest item is a function
    with no block above it; and the member is a graph node. The row records the cast's
    type, the candidates, the member, the impl the call is in, and what was not read.
    Otherwise the row has no target and names the condition that stopped it.
```

### 10.3 Steps, predicted and measured on itoa

```text
step  what                                         rules    predicted               measured
1     reads from generated function bodies         G1, G2,  55 reads, 334 edges     55 reads (39 with a target,
                                                   G4, G8                           16 without), 334 edges
2     origin of first and last token, on reads     G3       34 rows gain metadata   34 rows: 29 definition to
                                                                                    definition, 5 call to
                                                                                    definition; 334 edges
3     uses_type from generated signatures          G1-G4    66 uses_type, 366 edges 66 uses_type (28 new with a
                                                                                    target, 4 without), 366 edges
4     calls from generated bodies                  G1-G4,   197 calls, 502 edges    197 calls (3 with a target,
                                                   G6                               194 without), 502 edges
5     trait members through a type                 G5       built: 16 reads and 5 uses_type gained a target
                                                            gain a target
6     a call whose callee path names one function  G9       -                       30 of 197 calls with a target
7     a trait's method through the trait; a tuple  G10,     -                       41 of 197 calls with a target
      struct's constructor                         G11
8     the impl member when the first argument is   G12      5 rows change target    5 rows: target is the impl
      the method's own self                                                         member; 41 of 197 unchanged
9     the impl member when the receiver is a cast  G13      43 of 197 with a target 43 of 197 calls with a target
      to a stored type
10a   consts, statics and types as reads sources    G7       reads 55 -> 78          78 reads (22 original, 56 in
                                                                                    expanded text); 525 edges
10b   the impl member for <T as Trait>::NAME       G14      2 rows change target    not built
10c   declared and aliased types as uses_type      G7       34 generated, 3 orig    not built
11    arguments of builtin macros (assert!)        -        20 reads sites          not built
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
- **Q19 (closed in step 3).** The real-source test helper `read_edge_site_range` returned a
  byte range without its text. It now returns nothing for a site in expanded text, so a test
  cannot slice `src/lib.rs` with a range of an expansion.
- **Q21.** The 136 generated calls have no target. 18 are bare names of this crate's own
  functions (`divmod100`, `slice_buffer_to_str`); 8 more are in original bodies. The
  namespace stage does not request the callee path of a call (only its prefix), so no
  stored lookup exists for a bare name. 10 are `Unsigned ::fmt(..)`: the prefix is stored
  as the trait, and the lookup does not take a trait as prefix. The expected graph targets
  the impl member (`<uN as Unsigned>::fmt`), which needs the type of the first argument;
  the trait method `Unsigned::fmt` is provable from the path alone. Both are part 5e.
- **Q22.** G6 asks for "the stored reason". For a callee that was never requested there is
  no stored reason. The typed reason stays `TraitSolverNeeded` (the existing rule for a
  call shape the resolver does not handle) and the row's `unresolved_detail_kind` says what
  stopped the lookup. A reason kind of its own would change the core row types; not done.
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

### 10.8 Step 3: `uses_type` from generated function signatures (measured)

```text
rows  source (generated)             site text       target
12    <T as private::Sealed>::write  Self ::Buffer   the Buffer type alias of the same impl
12    <T as private::Sealed>::write  str             builtin str
4     <uN as Unsigned>::fmt          usize           builtin usize
4     <uN as Unsigned>::fmt          Self ::Buffer   none, TraitSolverNeeded (step 5, rule G5)
```

- All 32 rows are sited inside the expanded text of their own node and carry
  `expansion_source_origin`; both tokens of all 32 come from a macro definition.
- A signature row whose file or chain differs from its owner's stops the run with a typed
  error (G2), as for `reads`.
- **Against the expected graph:** it has 100 `uses_type` rows; 66 are stored. The other 34
  start at generated consts and type aliases (step 6).

```text
real-source test, new          test_generated_signature_type_uses failed first on
                               `left: {} right: {4 shapes}`, then passed
fixture test, changed          test_body_owner_rows_skipped (was test_body_and_macro_rows_skipped):
                               an expanded row under an original owner is now a typed error;
                               not seen failing on the old code
real-source test, changed      test_generates_edge_row_counts: 334 -> 366 edges
syntax crate tests             92 + 17 passed
real-source itoa tests         39 passed, 0 failed, 81 s, release, default threads
format (workspace), lint       pass (2 touched crates)
graph                          103 nodes, 366 edges (was 334); still incomplete
workspace gates, run cost      not run for this step; they run at the end of part 5d
```

### 10.9 Step 4: `calls` from generated bodies (measured)

Every `calls` row, original or generated, now carries in its metadata:

```text
key                       value
call_source_text          the whole call as written at the site
callee_source_text        what is called: the callee expression, or the method's name
call_shape_kind           method_call | qualified_path_call | bare_name_call |
                          other_callee_expression
dispatch_kind             how the target was chosen: generic_bound_trait_method |
                          type_path_associated_function | undetermined (no target)
unresolved_detail_kind    only on a row without a target: what stopped the lookup
expansion_source_origin   only on a row sited in expanded text (G3)
```

`dispatch_kind` no longer has the value `original_associated_function`; a target chosen
through a type path is `type_path_associated_function` wherever the call is written.

The 197 rows on pinned itoa, read from the exported edge table (run after step 4b):

```text
rows  written in  call_shape_kind      dispatch_kind                  unresolved_detail_kind        typed reason
104   expansion   method_call          undetermined                   receiver_type_not_determined  TraitSolverNeeded
18    expansion   bare_name_call       undetermined                   callee_path_not_requested     TraitSolverNeeded
10    expansion   qualified_path_call  undetermined                   type_prefix_is_trait          TraitSolverNeeded
4     expansion   qualified_path_call  undetermined                   type_prefix_unresolved        ExternalSourceMissing
36    source      method_call          undetermined                   receiver_type_not_determined  TraitSolverNeeded
10    source      qualified_path_call  undetermined                   member_lookup_unproven        MissingTarget
9     source      bare_name_call       undetermined                   callee_path_not_requested     TraitSolverNeeded
2     source      qualified_path_call  undetermined                   type_prefix_unresolved        ExternalSourceMissing
1     source      qualified_path_call  undetermined                   type_prefix_is_module         TraitSolverNeeded
2     source      qualified_path_call  type_path_associated_function  -                             -
1     source      method_call          generic_bound_trait_method     -                             -
```

- The 136 generated rows by callee text: 104 method calls, 10 `slice_buffer_to_str`, 8
  `divmod100`, 10 `Unsigned ::fmt` (a call through the trait's path), 4
  `mem ::size_of ::<Self >`.
- The 9 bare names in original bodies: 4 `divmod100`, 2 `div_rem_1e16`, 2 `enc_16lsd::<..>`,
  1 `DecimalPairs` (a tuple struct used as a constructor).
- All 136 are sited inside the expanded text of their own node and carry
  `expansion_source_origin`. A call whose chain differs from its owner's stops the run with
  a typed error (G2).
- **No new target.** Resolved calls stay at 3. See Q21.
- **Against the expected graph:** it has 102 grouped `calls` rows (its summary file). A
  row without a target cannot match one, so the matched count is unchanged by this step.

**Step 4b (same day): two things step 4 got wrong, corrected.**

- I wrote that the 10 generated `fmt` calls are `Self::fmt` / `uN::fmt` and that their prefix
  has no stored target. Read from the stored path rows: the callee is `Unsigned ::fmt`, and
  its prefix `Unsigned` is stored as resolved, to the trait. The lookup stops because it
  accepts only a struct, enum, union or builtin as prefix. The detail is now
  `type_prefix_is_trait`; the one `u128_ext::mulhi` call is `type_prefix_is_module`.
- G6, "the stored reason": 5 calls whose shape the lookup does not handle
  (4 `mem ::size_of ::<Self >`, 1 `MaybeUninit::<u8>::uninit`) have a prefix that was looked
  up and is stored as `external_source_missing`. Their typed reason was the catch-all
  `TraitSolverNeeded`; it is now `ExternalSourceMissing`, with detail
  `type_prefix_unresolved`.
- Calls by typed reason after 4b: 178 `TraitSolverNeeded`, 10 `MissingTarget`,
  6 `ExternalSourceMissing`, 3 with a target.

```text
real-source test, new          test_generated_body_call_rows failed first on a missing
                               call_source_text, then passed; for 4b it failed on
                               fmt type_prefix_unresolved / size_of callee_path_not_requested
                               against the corrected expectation, then passed
fixture test, new              test_call_metadata_site_mismatch: metadata of a resolved and an
                               unsupported call; a site in another text than its owner is a
                               typed error. Passed on first run (written after the code)
real-source test, changed      test_generates_edge_row_counts: 366 -> 502 edges
syntax crate tests             93 + 17 passed
real-source itoa tests         40 passed, 0 failed, 88 s after 4b, release, default threads
format (workspace), lint       pass (2 touched crates)
graph                          103 nodes, 502 edges (was 366); still incomplete
workspace gates, run cost      not run for this step; they run at the end of part 5d
```

### 10.10 Step 5: a member reached through a type (measured)

Rule G5 as built. It replaces the earlier wording ("a trait of this crate declares NAME ...
through a stored supertrait row"), which the build showed to be narrower than needed: the
lookup goes by type and name over the stored impl headers, and walks no supertrait.

```text
what                     how it is proven
the type, for Self       the stored self type of the impl header whose member list holds the
                         site's owner. A function nested in a method is in no member list, so
                         it has no Self; the lookup does not walk up.
the type, other prefix   a new Type request for the one-name qualifier of a two-segment body
                         value path (context body_value_type_prefix), in the site's scope.
                         No builtin name is matched by text.
candidates               members named NAME in stored impls of the type, plus stored impls
                         whose self type has no stored target (generic and blanket impls)
target                   exactly one candidate; trait impl of the type; stored trait; no
                         generic parameter; not negative; kind CONST (reads) or TYPE_ALIAS
                         (uses_type); the member is a graph node
not checked              whether the trait is in scope at the site
```

The target is the impl member (`<u8 as Integer>::MAX_STR_LEN`), not the trait's declaration.

Every row the lookup ran for carries `type_relative_member_evidence`:

```text
self_type_source_text, self_type_builtin_name, self_type_identity
member_lookup_outcome_kind   single_trait_impl_member | no_stored_impl_member |
                             single_member_unproven | several_stored_impl_members
member_candidate_impl_rows   per candidate: impl header, trait text, flags, member kind
impl_trait_source_text, impl_header_occurrence   (only with a target)
sources_not_read             builtin type: its inherent items, and trait impls for it in crates
                             not read. Type of this crate: blanket trait impls in crates not read.
```

Measured on pinned itoa at `705efeae5` (release, 3 runs: 15.7 to 20.4 s, 344 to 360 MB peak,
16 MB store). Totals unchanged: 103 nodes, 502 edges.

```text
rows  relation   site                  written in  target
5     reads      Self ::MAX_STR_LEN    expansion   <iN as Integer>::MAX_STR_LEN
5     reads      uN ::MAX_STR_LEN      expansion   <uN as Integer>::MAX_STR_LEN
5     reads      u128::MAX_STR_LEN     source      <u128 as Integer>::MAX_STR_LEN
1     reads      i128::MAX_STR_LEN     source      <i128 as Integer>::MAX_STR_LEN
4     uses_type  Self ::Buffer         expansion   <uN as private::Sealed>::Buffer
1     uses_type  Self::Buffer          source      <u128 as private::Sealed>::Buffer
```

- `reads`: 55 of 55 have a target (was 39). `uses_type`: 62 of 66 (was 57); the other 4 are
  `MaybeUninit`, in `core`.
- Path rows: 12 new prefix requests (11 builtins, 1 type parameter `I`), counted apart from
  the 202 signature and import paths.
- Predicted 4 `uses_type` rows; measured 5. The fifth is in the source-written `fmt` for u128.
- `uses_type` runs the lookup only for the prefix `Self`; `reads` for any one-name prefix.
  itoa has no `u8::Alias` in a signature.

Tests: `test_type_relative_member_targets` (real source; failed first on
`u128::MAX_STR_LEN has no target`), 8 tests in `type-relative-member-tests.rs` and
`test_value_prefix_path_listed`. The 9 fixture tests passed on first run; four mutations of
the code (unknown self type not a candidate, kind not checked, inherent impl accepted, prefix
of any path requested) each made at least one fail.

Open:

- **Q23.** No pipeline-level fixture for: an impl inside a function body (the member is not a
  node), `Self` in a closure against `Self` in a nested function, and a prefix that is a
  `use` alias. Each is covered only by reading the code.
- **Q18, still open.** The rule cannot see an inherent item or a trait impl in `core`. The row
  says so; the target stands on itoa compiling.
- The generated `Unsigned ::fmt(..)` calls are not in this step (Q21).

### 10.11 Step 6: a call whose callee path names one function (measured)

Rule G9. Built at the commit that adds this section.

**Cause removed.** The callee path of a call was never sent to name lookup; only its type
prefix was. Now each call's whole callee path is one more stored path request, context kind
`body_call_callee_path`, Value namespace. Pinned itoa stores 56 of them (path table: 258 rows,
was 202).

```text
 what name lookup stores for the 56 callee paths            rows  row's outcome
 divmod100                Function                            12  target (8 generated, 4 original)
 slice_buffer_to_str      Function                            10  target (generated)
 div_rem_1e16             Function                             2  target
 enc_16lsd::<..>          Function                             2  target
 u128_ext::mulhi          Function, qualifier Module           1  target
 Unsigned ::fmt           Function, qualifier Trait           10  no target: qualifier is a trait (Q21)
 DecimalPairs             Struct                               1  no target: callee_is_not_function (Q26)
 Buffer::new              no target, no reason; Struct         2  target by the member lookup of part 3
 u128::from               no target, no reason; Builtin        9  no target: member lookup unproven
 str::from_utf8_unchecked external source missing; Builtin     1  no target: external source missing
 mem ::size_of, hint::unreachable_unchecked, MaybeUninit::<u8>::uninit
                          external source missing              6  no target: qualifier unresolved
```

**Measured (three release runs on pinned itoa).** 103 nodes and 502 edges, both unchanged.
`resolved_calls` 30 of 197 (was 3). Wall 16.8 to 19.1 s, peak 357 to 376 MB, store 16 MB.
The graph is still marked incomplete.

**Row metadata.** A targeted row has `dispatch_kind: callee_path_function`. Every call whose
callee path has a stored lookup carries `callee_path_lookup_evidence`: the stored state and
reason, the kinds of the symbols found, the qualifier's kind, and the outcome
(`single_function_node`, `callee_qualifier_not_module`, `callee_is_not_function`,
`callee_path_unresolved`, `callee_path_ambiguous`, `callee_target_not_node`). So the 10
`Unsigned ::fmt` rows already say that the path names one function through a trait.

**Finding (Q24).** itoa has `use core::str;`, so `str` in `str::from_utf8_unchecked` is the
module `core::str`. The stored Type lookup of the qualifier `str` says builtin type. The
Value lookup of the whole path says external source missing, which is right. The row takes
the whole path's reason (the exception in G9). The qualifier lookup itself is not fixed: an
import whose source was not read does not shadow a builtin type name in the name lookup.

**Tests.** Real source: `test_callee_path_function_targets` (failed first: `slice_buffer_to_str`
without a target; then passed) pins the 27 targets by caller kind and the lookup record of
all 56 calls; `test_sysroot_reached_names_reasoned` pins what name lookup stores for the 56
paths. Fixtures: `test_request_call_callee_path`, `test_callee_path_block_scope` (the request
is made in the block's scope, where a block-local `fn` is the binding),
`test_bare_callee_names_function`, `test_bare_callee_untargeted_reasons`,
`test_qualified_callee_needs_module`, `test_struct_qualifier_keeps_member`,
`test_shadowed_qualifier_blocks_callee`. The fixture tests were written after the code and
passed or failed only on wrong expectations of mine, so they are weak as first-run evidence.
Nine deliberate breaks of the code were tried; seven made a test fail at once, two did not
(ignoring the shadowing flag; letting an ambiguous qualified path decide) and a test was
added for each.

**Open questions.**

- **Q24.** The Type lookup of a qualifier named like a builtin (`str`) ignores an import of
  that name whose source was not read. Fix belongs in name lookup. No wrong target on itoa.
- **Q25.** `crate::f(..)`, `self::f(..)`, `super::f(..)` and generic arguments on a
  non-final segment are not covered by a test, and whether their qualifier is stored as a
  module is not checked. itoa has none; such a call can only lose a target, not gain a
  wrong one.
- **Q26.** `DecimalPairs(..)` is a tuple struct's constructor. Name lookup stores the struct.
  Decided for the next step under the maximum-information rule: the row targets the struct
  node with its own dispatch kind.
- **Q21, decided for the next step.** `Unsigned ::fmt(..)`: the path alone proves the trait's
  method declaration. That is the target; the impl members that could run are evidence, and
  one of them becomes the target only when the receiver's type is stored.
- A local binding named like the qualifier (`fn caller(m: u8) { m::f() }`) blocks the lookup,
  although Rust would still resolve `m` as a module. Conservative; loses a target only.

### 10.12 Step 7: a trait's method through the trait, and a tuple struct's constructor (measured)

Rules G10 and G11. Built at the commit that adds this section. Both were decided in step 6
(Q21, Q26) under the maximum-information rule.

```text
 callee            rows  target                     dispatch_kind             row's outcome
 Unsigned ::fmt      10  Unsigned::fmt (the trait's trait_path_method         single_trait_method
                         declaration)
 DecimalPairs         1  DecimalPairs (the struct)  tuple_struct_constructor  single_struct_node
```

**Measured (three release runs on pinned itoa).** 103 nodes and 502 edges, both unchanged.
`resolved_calls` 41 of 197 (was 30). Wall 18.4 to 22.7 s, peak 354 to 369 MB, store 16 MB.
The graph is still marked incomplete.

**Row metadata (G10).** Each of the 10 rows carries `trait_method_lookup_evidence`:
`trait_member_outcome_kind: trait_method_declaration`, `receiver_type_stored_flag: false`,
`sources_not_read` (impls of the trait in crates whose source was not read), and
`trait_member_candidate_rows`. On itoa each row lists the same five candidates: `fmt` in
the impls of `Unsigned` for `u8`, `u16`, `u32`, `u64` (generated) and `u128` (original).
Each candidate row has the impl header's occurrence key, the impl's self type text, whether
that self type is stored as resolved, whether the impl has generic parameters, and the
member's occurrence key.

**What the 10 calls are (counted in the stored rows).** Five are `Unsigned::fmt(self, buf)`
in the `write` of an unsigned type (`u8` to `u128`); five are
`Unsigned::fmt(self.unsigned_abs(), ..)` in the `write` of a signed type (`i8` to `i128`).

**Not built: the impl member as target.** For the five `Unsigned::fmt(self, buf)` calls the
first argument is the `self` parameter of a method in an impl for a stored type, so the
impl member that runs is provable from source without type inference. No rule reads the
first argument yet, so these rows also stop at the declaration (Q27). The other five need
the return type of `unsigned_abs`, which is in `core`.

**Tests.** Real source: `test_trait_path_call_targets` (failed first: "`<u8 as
private::Sealed>::write` calls Unsigned::fmt without a target"; then passed) pins the 11
targets, their dispatch kinds and the five candidate types. Changed to measured values:
`test_callee_path_function_targets` (two outcomes), `test_generated_body_call_rows` (the
10 `fmt` rows have a target; two more dispatch kinds). Fixtures:
`test_trait_method_candidate_rows` (only functions of that name in resolved impls of that
trait), `test_tuple_struct_callee_target` (a struct that is, and is not, a graph node),
`test_qualified_callee_needs_module` (a function through a trait is a target; a struct or
a const through a trait is not; a function through a struct goes to the member lookup; a
struct through a module is a target), `test_bare_callee_untargeted_reasons` (a const is
not a call target). The fixture tests were written after the code. Six deliberate breaks of
the code each made a fixture test fail at once
(a struct through a trait accepted; any non-module qualifier treated as a trait; candidates
ignoring the trait, the member kind, or the trait's stored state; the graph-node filter
removed).

**Open questions.**

- **Q27.** `Trait::method(self, ..)` where `self` is the method's own `self` parameter in an
  impl for a stored type: the impl member is provable. Built in step 8 (10.13, G12).
- **Q28.** A tuple enum variant called as a function (`E::V(..)`) and a unit struct or
  variant used as a value are not covered. itoa has none.
- **Q29.** A struct found in the Value namespace is taken as a tuple struct. A struct with
  named fields has no Value binding in Rust; whether name lookup ever stores one for it is
  not checked by a test.
- **Q30.** The candidate list is not narrowed by the impl's generic parameters or by
  negative impls; each candidate row carries the generic-parameter flag instead. itoa has
  neither on `Unsigned`.

### 10.13 Step 8: the impl member when the first argument is the method's own `self` (measured)

Rule G12 (closes Q27). Built at the commit that adds this section.

```text
 callee                               rows  target                     dispatch_kind           row's outcome
 Unsigned ::fmt (self , buf )            5  <uN as Unsigned>::fmt      trait_path_impl_method  single_impl_member_by_self
                                            (uN = u8 u16 u32 u64 u128)
 Unsigned ::fmt (self .unsigned_abs ()   5  Unsigned::fmt (the trait's trait_path_method       trait_method_declaration
   , ..)                                    declaration), unchanged
```

**Measured (three release runs on pinned itoa).** 103 nodes, 502 edges and `resolved_calls`
41 of 197, all unchanged: the five rows had a target before and have a more exact one now.
Wall 14.8 to 18.1 s, peak 339 to 365 MB, store 16 MB. The exported edge table holds 5 rows
with `trait_path_impl_method` and 5 with `receiver_impl_header_occurrence`. The graph is
still marked incomplete.

**Why the member is proven, not inferred.** The call is written in `fn write(self, ..)` of
`impl private::Sealed for uN`. `self` there has the type the impl is for. That impl's type
is stored with a target. `Trait::name(self, ..)` takes `self` by value, so no reference or
deref step changes the type. So the function that runs is `name` in the impl of the trait
for that same stored type, when there is exactly one.

**What is stored where.**

- Call syntax (`call_syntax_evidence`): `call_first_argument_self_flag`, true only for the
  bare path `self` as first argument where the nearest `fn` declares `self` or `mut self`
  and has no block above it.
- Impl header member rows: `member_self_parameter_kind`, one of `value`, `reference`,
  `typed`, or null when the function has no `self`. `member_self_parameter_flag` stays.
- The row's `trait_method_lookup_evidence`, when a member is chosen:
  `trait_member_outcome_kind: single_impl_member_by_self`, `trait_member_candidate_rows`
  (the G10 list, unchanged), `selected_member_occurrence_key`,
  `trait_method_declaration_identity`, `receiver_type_stored_flag: true`,
  `receiver_self_type_identity`, `receiver_self_type_source_text` (`u8`),
  `receiver_impl_header_occurrence` (the impl that holds the calling method), and
  `sources_not_read`.

**Changed after the second opinion** (record:
`docs/devin-research/1794782315-second-opinion-glm-5-3-max-part-5d-step-eight-self-argument-member.md`).
The first build counted only impls whose trait was stored with a plain path. With
`trait T<A = u8>`, `impl T<u16> for u8` and `impl T for u8`, the first impl's trait is not
stored (it has generic arguments), so one candidate was left and `T::<u16>::f(self)` would
have been given the wrong member. Now every stored trait impl counts unless its trait or its
type is stored as another one, and two candidates give no member. This case is pinned on
header rows in a fixture, not on real source. Also from the review: the row names the impl
that holds the calling method and the chosen member; a member written `r#name` matches.

**Tests.** Real source: `test_trait_path_call_targets` (failed first: all ten rows were
`trait_path_method trait_method_declaration false`; then passed) pins the five impl
members, the five declaration rows and the type text of each chosen row. Changed to the
measured value: `test_generated_body_call_rows` (one more dispatch kind; it failed on the
first full run). Fixtures, in `self-argument-member-tests.rs`, written after the code:
`test_own_self_argument_flag` (14 call shapes), `test_self_argument_member_rules` (header
rows: each condition of G12 removed one at a time, on the calling method's impl and on the
candidate's), `test_trait_call_self_member` (three runs of the call producer on real
source text: member chosen; another first argument; member not a graph node). Eighteen
deliberate breaks of the code each made one of these fail
(`/home/ubuntu/phase5d-work/s8-mutations.txt`).

**Open questions.**

- **Q31.** A callee whose trait qualifier has generic arguments (`T::<u16>::f(self)`) is not
  rejected by G12 itself; the rule relies on "exactly one candidate among all impls that may
  be of this trait for this type". The reviewer asked for an explicit guard. Not added: with
  one such impl, generic arguments cannot select another. Not tested on real source.
- **Q32.** An impl written inside a `const _: () = { .. }` block at module level gets no
  member (the block rule); it keeps the G10 row. itoa's five `const _` blocks hold no impl.
- **Q33.** The five `Unsigned::fmt(self.unsigned_abs(), ..)` rows need the return type of
  `unsigned_abs`, which is in `core`. They stay on the trait's declaration until part 6.
- **Q34.** Next rule proposed by the review: a method call whose receiver is a cast
  (`(self as $primitive).write(buf)`) has the cast's written type as receiver type.
  Closed: built in step 9 (10.14, G13).

### 10.14 Step 9: the impl member when the receiver is a cast to a stored type (measured)

Rule G13 (closes Q34). Built at `6e78660c0` and `a9e8cdbf1`.

```text
 call                         written in                      target                           dispatch_kind
 (self as u64 ) .write (buf ) <usize as private::Sealed>::write <u64 as private::Sealed>::write cast_receiver_impl_method
 (self as i64 ) .write (buf ) <isize as private::Sealed>::write <i64 as private::Sealed>::write cast_receiver_impl_method
```

**Measured (three release runs on pinned itoa).** 103 nodes and 502 edges, unchanged;
`resolved_calls` 43 of 197 (was 41). Wall 19.2 to 21.4 s, peak 350 to 359 MB, store
16.8 MB. The exported edge table holds 2 rows with `cast_receiver_impl_method`. The graph
is still marked incomplete.

**Method calls, counted in the stored rows before this step.** Of 141 method calls, three
have a target inside itoa: `i.write(..)` (already resolved by the generic bound) and these
two. The other 138 call a method of `core` and wait on part 6.

**Why the member is proven, not inferred.**

- The receiver's type is written in the source: a cast expression has the type it casts
  to. No inference is used. The cast's path is sent to name lookup at the call, so a local
  item that shadows `u64` is found there, as the compiler would find it.
- The trait is in scope. A method of a trait can be called only where the trait is in
  scope. `private::Sealed` is not imported in `src/lib.rs`; the call compiles because the
  trait of an impl is in scope in the bodies of that impl's members. So the rule requires
  the chosen member's trait to be the trait of the impl the call is written in.
- A call written in a function or a const nested in the body is left out: such an item has
  the module's scope, not the impl's.
- One candidate only. Every stored impl for the type, and every stored impl whose type is
  not stored (a generic, a reference, a blanket impl), counts. Two members with the name
  give no target.

**What is not read, and is said on the row.** For a builtin such as `u64`, an inherent
method `write` in `core`, or an impl for `u64` of another trait in an unread crate, would
change the answer. `core` is not read in this phase. The row gives the target and lists
both under `sources_not_read` (the same rule as `u8::MAX_STR_LEN`, G5). Decided under the
maximum-information rule; the second opinion agreed (`u64` has no inherent `write`, and
withholding the target would remove a true edge).

**What is stored where.**

- Method receiver evidence (`method_receiver_evidence`), on every method call:
  `receiver_shape_kind: cast_expression`; `receiver_cast_type_evidence` (the type's byte
  range in its file, its text, and `type_simple_path_flag`), null for other receivers;
  `receiver_member_body_flag`, true when the call's nearest item is a function with no
  block above it.
- Name lookup: one more request per cast receiver with a simple path, context
  `body_cast_receiver_type`, type namespace. Stored path rows on itoa: 204 (was 202), 166
  resolved (was 164).
- The row's `cast_receiver_lookup_evidence`: `self_type_source_text` and
  `self_type_identity` (the cast's type), `member_candidate_impl_rows`,
  `member_lookup_outcome_kind`, `impl_header_occurrence` and `impl_trait_source_text` of
  the chosen member's impl, `selected_member_occurrence_key`,
  `member_self_parameter_kind`, `enclosing_impl_header_occurrence`,
  `enclosing_impl_trait_identity`, `enclosing_impl_trait_source_text`,
  `enclosing_impl_self_type_identity`, `enclosing_impl_self_type_source_text` (`usize`),
  `call_member_body_flag`, `trait_scope_proof_kind` (`enclosing_impl_trait`, or null),
  and `sources_not_read`.
- A row without a target has `dispatch_kind: undetermined` and `unresolved_detail_kind`,
  one of `cast_type_not_stored`, `cast_member_unproven`, `cast_member_not_method`,
  `cast_trait_scope_unproven`, `cast_member_not_node`. It keeps the same evidence.

**Second opinion** (record:
`docs/devin-research/1794784173-second-opinion-glm-5-3-max-part-5d-step-nine-cast-receiver.md`).
It found no compiling program within read source for which the rule gives a wrong member.
Adopted: three missing fixture cases (the member is not a graph node; the one candidate is
a negative impl; the call names the method as `r#name`), and the enclosing impl's trait
and type written on the row. Its example for a nested `fn` (the trait is not in scope
there) was not compiled here; the rule excludes that position either way.

**Tests.** Real source: `test_cast_receiver_call_targets` (failed first: no row had a cast
receiver, 0 passed 1 failed; then passed) pins both rows, their dispatch kind, the cast's
type, the outcome, the trait and two unread sources. Changed to measured values:
`test_generated_body_call_rows` (one more dispatch kind; 102 method calls without a
target, 2 with), `test_sysroot_reached_names_reasoned` (204 path rows, 166 resolved),
`test_capture_generic_receiver_evidence` (two more fields). Fixtures, in
`cast-receiver-member-tests.rs`, written after the code and passing on first run:
`test_cast_receiver_syntax_evidence` (10 receiver shapes), `test_cast_receiver_path_request`
(which paths are sent to name lookup), `test_cast_receiver_member_rules` (header rows:
each condition of G13 removed one at a time), `test_cast_receiver_call_member` (two runs of
the call producer on source text: member chosen; member not a graph node). Sixteen
deliberate breaks of the code; fifteen failed a fixture test at once, one (any item
instead of a function as the nearest item) passed until a case was added
(`/home/ubuntu/phase5d-work/s9-mutations.txt`).

**Open questions.**

- **Q35.** A call written in an associated const of the impl (`const C: u8 = (1 as
  u8).m();`) gets no target, though the trait is in scope there. A miss, not a wrong
  target. itoa has none.
- **Q36.** The trait-in-scope proof covers only the enclosing impl's trait. A trait in
  scope by `use`, by the prelude, or by being declared in the module is not yet used as
  proof; such a call keeps `cast_trait_scope_unproven`.
- **Q37.** Receivers whose type can be read from itoa's own source, by the review's count:
  `buf[offset]`, `DECIMAL_PAIRS.0`, `self.bytes`, `string`, and `self` in a method of an
  impl for a stored type. Their methods (`MaybeUninit::write`, `get_unchecked`, `len`,
  `as_mut_ptr`, `unsigned_abs`) are in `core`: the receiver type can be recorded now,
  the target waits on part 6. Not provable without `core`: `try_into`, `expect`,
  `unwrap`, `ilog10`, `cast`, `rev`.

### 10.15 Step 10a: consts, statics and types as `reads` sources (measured)

Rule G7, `reads` half. Built at `ed55f4a63`.

**Cause found (measured on the step 9 store).** The stored site table already held every
path expression owned by a generated const or type alias. No `reads` row came out because
the value-path request was only made for a path inside a function body
(`check_body_value_path`). The `reads` producer has no filter on the owner's kind.

**Rule as built.**

```text
A path expression is a value-path request when it is written in any item: a function body,
the initializer of a const or static, or an expression of a type (array length, enum
discriminant). Unchanged: not a call's callee, and its single name is not a local binding.
Local bindings are looked for in the nearest item the path is written in. A const, static
or struct written in a function does not see the function's parameters or locals; a let in
a const's own block still binds.
A path in an attribute of the file is in no item and is not requested.
```

**Measured on pinned itoa (3 release runs, executable `576bb091…ced1e`).**

```text
nodes=103 edges=525 (was 502) reads=78 (was 55) reads_in_expanded_bodies=56 (was 34)
calls=197 resolved_calls=43 type_uses=66 implements=32 imports=4   completeness=incomplete
wall 17.0 to 19.4 s, peak 371 to 391 MB, store 17,358,848 bytes

new rows  source                               read                           target
10        <T as Integer>::MAX_STR_LEN (gen.)   uN::MAX / iN::MAX              none: no stored impl of the
                                                                              builtin declares MAX; inherent
                                                                              items in core were not read
10        <T as private::Sealed>::Buffer (gen.) Self::MAX_STR_LEN (array len) <T as Integer>::MAX_STR_LEN (G5)
2         <usize|isize as Integer>::MAX_STR_LEN <u64|i64 as Integer>::MAX_STR_LEN
                                                                              Integer::MAX_STR_LEN, the
                                                                              trait's declaration
1         struct Buffer (original)             i128::MAX_STR_LEN (array len)  <i128 as Integer>::MAX_STR_LEN
```

- The 10 rows without a target keep the reason `TraitSolverNeeded` and carry the G5
  lookup record (`no_stored_impl_member`, and the two unread sources). They are not
  marked "external source missing": a blanket impl or a trait default in read source is
  not enumerated by the lookup, so "it is in `core`" is not proven. Part 6 names them.
- The 2 qualified-path rows are true but less exact than the expected graph, which has
  the impl member `<u64 as Integer>::MAX_STR_LEN`. Name lookup drops the type anchor.
  Choosing the member needs name requests for the anchor type and the trait path, which
  are not stored today. That is step 10b (rule G14).
- **Side effect, checked.** The 11 array-length paths were requested before as
  `signature` paths (type-or-value namespace, unresolved, no reason). They are
  `body_value_path` requests now. `signature` path rows on itoa go from 204 to 193;
  11 more one-name type prefixes are requested (`u8` … `i128`). `uses_type` is
  unchanged at 66.

**Tests.** `test_item_initializer_read_rows` (itoa, all 23 rows by source, text and
target; failed first with 0 rows), `test_item_initializer_paths_listed` (fixture; failed
first on a struct in a function seeing the function's parameter). 12 pins in 7 older
itoa tests and 2 fixture tests were changed to the measured values, each after reading
the assertion. 45 of 45 itoa tests pass (136 s), 119 + 17 syntax crate tests, format,
lint on the two touched crates.

**Deliberate breaks.** Four, each fails a test: value paths only in functions; bindings
looked for in the nearest function; bindings in the nearest function, const or static;
no item check at all (this one survived until the file-attribute case was added).

**Not done.** No `todo!()` stub: the change is to two existing predicates, no new
signature. No second opinion yet for this step; it is taken with 10b and 10c.

- **Q38.** A path in an attribute of an item (`#[a = B] fn f() {}`) is requested as a
  value path of that item. Not valid Rust after expansion; no test.
- **Q39.** The 20 expected `reads` of `core` consts in `const _` blocks
  (`assert!($Signed::MIN < 0, ..)`) need the arguments of a builtin macro parsed. Step 11.
- **Q40.** Calls in a const initializer (`uN::MAX.ilog10()`) are stored sites; whether
  the `calls` producer emits them was not measured in this step (`calls` stayed 197,
  which already includes them or not). Measure in 10c.

