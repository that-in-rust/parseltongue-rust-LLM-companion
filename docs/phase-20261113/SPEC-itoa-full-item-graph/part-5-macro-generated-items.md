# Part 5 — macro-generated items as graph nodes

**State: SPECIFIED for 5a, no code. Rules R2, R3 and R5 decided by the user on 2026-11-15
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
- **Predicted, not measured:** after 5a, 103 nodes (58 + 45), 88 `contains` rows
  (43 + 45) and 45 `generates` rows; 285 edges in all (195 + 90).
- **What the expected graph has for these items:** 45 `contains` rows, and 202 edges that
  start at a generated item (80 `calls`, 66 `uses_type`, 56 `reads`). 4 more end at one.

## Sub-parts

```text
part  target                                              rows on itoa (predicted)   state
5a    generated items as nodes, contained, with their     45 nodes, 45 contains      SPECIFIED
      generating call
5b    labels: impl header and macro call text in the      58 + 45 labels             not specified
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
- **R4 (nested expansion).** WHEN the chain has more than one call, THEN the container SHALL
  come from the first call of the chain, the one written in a source file.
- **R5 (generating call; decided: both forms).** Each generated node SHALL name the macro
  call node that produced it (the last call of its chain) in two ways: a field on the node
  row, and one `generates` row from the call node to the item, sited at the call.
  `generates` is a new relation kind; it is never collapsed and never counted as a
  dependency between items.
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
R10      test_generated_item_node_counts          real source   itoa-source-acceptance-tests
R11      test_generated_nodes_read_twice          real source   itoa-source-acceptance-tests
R1 R8    test_expanded_item_projected_node        fixture       original-item-projection-tests
R4       test_nested_expansion_outer_container    fixture       original-item-projection-tests
R6       test_expanded_text_digest_mismatch       fixture       original-item-projection-tests
R7       test_inactive_expanded_item_skipped      fixture       original-item-projection-tests
R1       test_expanded_body_item_evidence         fixture       original-item-projection-tests
R9       test_generated_item_node_counts          real source   itoa-source-acceptance-tests
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

Tests that pin today's behaviour and will change in 5a or 5d (named by the second review,
not yet read by me): `original-item-projection-tests.rs` lines 203 to 210,
`test_expanded_body_read_counted`, `test_expanded_header_keeps_chain`,
`test_itoa_reads_row_counts`.

## 6 — Quality gate results

None yet.

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
- **Q4. The expected graph has no macro call or definition nodes.** It gains them when it
  is repaired, per the user's decision.

Second opinion (Rule 5): GLM-5.3 through pi, max reasoning, read-only, on this design
before it was written. Record:
`docs/devin-research/1794745800-second-opinion-glm-5-3-max-part-5-macro-items-design.md`.
