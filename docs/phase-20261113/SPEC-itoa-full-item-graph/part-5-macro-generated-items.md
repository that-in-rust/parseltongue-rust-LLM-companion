# Part 5 — macro-generated items as graph nodes

**State: 5a BUILT, except nested expansion (R4), which stops the run with a typed error.
5b SPECIFIED (section 8). 5c to 5e are not specified. Rules R2, R3 and R5 decided by the user on 2026-11-15
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
5b    labels: impl header and macro call text in the      103 labels, all distinct   SPECIFIED (section 8)
      node's path text
5c    `expands`: macro call -> macro definition          15 rows                    not specified
5d    generated bodies and signatures as edge sources:    up to 202 rows             not specified
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

**State: SPECIFIED.** Numbers marked *measured* come from one release run at `4ea998560`
(store `/home/ubuntu/phase5b-work/store.db`); everything else is a rule.

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
```

Examples on itoa (predicted until the test passes):

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
test_unnamed_const_label_text      fixture      L5 and L7: `const _`, `const _ #call`, `r#`, kind fallback
```

### 8.4 Plan

1. Stub `item-node-label-text.rs` in the DuckDB adapter; failing fixture tests.
2. Failing real-source tests.
3. Code; both projections call the one helper.
4. The two 5a tests that find `const _` by the suffix `::_` find it by kind.
5. Gates, three measured runs, log, PRD.

### 8.5 Open questions (5b)

- **Q7. Expected-graph labels differ in form.** They carry no crate and file prefix
  (`u128_ext::mulhi`), and write `const _ #impl_Integer(i8,u8)`. Part 7 decides how labels
  are compared; the expected graph is repaired to the stored form there.
- **Q8. No typed link from an impl member to its impl header.** The label segment is the
  only one. A node field naming the header (`node_impl_header_reference`) is in the backlog.
- **Q9. Q2 (range in the macro definition) moves to 5d**, where generated bodies need it.

Second opinion (Rule 5): GLM-5.3 through pi, max reasoning, read-only, before this section
was written. Record: `docs/devin-research/1794768300-second-opinion-glm-5-3-max-part-5b-node-labels.md`.
