# Part 4b — `implements` edges from impl headers

Written 2026-11-15 with `rust-executable-specs-spec-first-architecture`, before any code.

## The answer

**State: SPECIFIED, not built. One `implements` row per trait impl header, from the self
type to the trait. On pinned itoa that is 32 rows: 29 with a target and 3 unresolved. The
first target is `u128 --implements--> Unsigned`, the only original header whose trait is in
itoa.**

- **No new table and no inference.** `impl-header-index-table` already stores, per header,
  the resolved self type and the resolved trait, each with its own source range.
- **One contract change.** The graph validator rejects an edge whose source is a builtin
  node. `impl Unsigned for u128` needs exactly that (ARCH rule G11: impl blocks are not
  nodes; the source is the self type).
- **Measured on pinned itoa** (sysroot excluded), 33 impl headers:

```text
headers  where       self type   trait                         row
1        original    u128        Unsigned (itoa)               target
3        original    Buffer      Default, Clone, Copy (core)   unresolved, ExternalSourceMissing
1        original    Buffer      none (inherent impl)          no row
28       macro       12 ints     Integer 12, Sealed 12,        target
                                 Unsigned 4
```

- **Agrees with the expected graph by count**: 32 `implements` rows, 29 from builtins to
  itoa traits and 3 from `Buffer` to `core` traits.

## 1 — Work mode

Spec mode and delivery mode. Component: L1 graph contract, L2 resolver in the syntax
extractor, L3 the CLI facade. The main risk is the validator change letting a builtin
source through for other edge kinds.

## 2 — Executable requirements

**REQ-RUST-047.0 — `implements` edges from impl headers.**

- **R1.** WHEN an impl header outside any function body names a trait and both its self type
  and its trait resolve to graph nodes, THEN one `implements` row SHALL be stored from the
  self type to the trait, with resolution `Extracted`.
- **R2.** WHEN the self type is a builtin, THEN the source SHALL be that builtin's node, and
  one `Builtin` node per builtin name SHALL exist in the graph.
- **R3.** WHEN the trait did not resolve, THEN one unresolved row SHALL be stored from the
  self type with one reason, by the same rule as part 4a R5.
- **R4.** The evidence site of every row SHALL be the trait path's own byte range in the
  header, with the header's expansion provenance.
- **R5.** WHEN the header has no trait (inherent impl) or is a negative impl, THEN no row
  SHALL be stored.
- **R6.** WHEN the self type is not a graph node (unresolved, a type parameter, or an item
  outside the graph), THEN no row SHALL be stored and the stage line SHALL count the header
  in `implements_without_source`.
- **R7.** WHEN a header comes from a macro expansion, THEN R1 to R6 SHALL apply unchanged; the
  site is the range in the expanded text plus the provenance chain.
- **R8.** The validator SHALL accept a builtin source only for `implements`. Every other
  relation with a builtin source SHALL still fail with `InvalidEdgeRecord`.
- **R9.** Two reads of the same store SHALL return identical rows in identical order.

## 3 — Design by layer

- **L1 `parseltongue-core-row-types`.** In `validate_full_graph_input`: allow source
  category `Builtin` when the relation is `Implements`; for such a row, check the site's
  unit against the run's units instead of against the source's unit. No new type.
- **L2 `parseltongue-syntax-span-extractor`.** New module `impl-header-implements-records.rs`:
  - `resolve_impl_header_edges(store, snapshot, input, cancel) -> Result<ImplHeaderEdgeSet, ImplHeaderEdgeError>`.
  - `ImplHeaderEdgeSet { edges, headers_without_source }`.
  - `ImplHeaderEdgeError` is a `thiserror` enum; no free-text variant.
  - Reads header rows through the existing read-store trait; checks cancellation per row.
  - Reuses `read_stored_reason_kind` and the builtin-node collector from part 4a.
- **L3 `parseltongue-cli-dispatcher-binary`.** The facade calls the resolver after the
  signature stage, adds builtin nodes for sources as well as targets, and prints
  `implements=N implements_without_source=N` on the stage line.

## 4 — Verification matrix

```text
req    test                                   type          assertion
R1 R2  test_unsigned_impl_builtin_source      real source   u128 --implements--> Unsigned, Extracted
R3     test_buffer_clone_trait_unresolved     real source   Buffer row for Clone, ExternalSourceMissing
R1     test_trait_impl_header_edge            fixture       item self type, item trait: one row
R2     test_builtin_source_one_node           fixture       two headers on u8: one u8 node
R3     test_unresolved_trait_keeps_reason     fixture       cfg_inactive and ambiguous reasons kept
R4     test_implements_site_trait_range       fixture       site equals the trait path range
R5     test_inherent_negative_no_edge         fixture       inherent and negative impls: zero rows
R6     test_unresolved_self_counted_only      fixture       zero rows, headers_without_source = 1
R7     test_macro_header_edge_provenance      fixture       macro header: row carries the chain
R7     test_itoa_implements_row_counts        real source   32 rows: 29 Extracted, 3 Unresolved
R8     test_builtin_source_implements_only    L1 unit       implements passes; uses_type fails
R9     test_implements_rows_read_twice        fixture       two reads equal
```

Metric for every row: exact equality of source, target or reason, relation and site range.

## 5 — Implementation plan

Each step is one commit of about 200 changed lines, test first.

1. RED: `test_unsigned_impl_builtin_source` on pinned itoa (0 `implements` rows today).
2. L1: validator accepts a builtin source for `implements` only (R8).
3. L2: resolver for original headers with both ends resolved (R1, R2, R4, R5, R9).
4. L3: facade stores the rows and the source builtin nodes; step 1 goes GREEN.
5. Unresolved trait rows and the header count without a source (R3, R6).
6. Macro-expanded headers (R7); `test_itoa_implements_row_counts`.
7. Refactor pass, workspace gates, one run's counts and cost.

## 6 — Quality gate results

Not run. Nothing is built.

## 7 — Open questions

- **Q1. Site of a macro-generated header.** R7 stores the range in the expanded text with the
  provenance chain. The expected graph anchors these rows in the macro definition in
  `src/lib.rs`. Part 5 decides which is canonical; step 6 may move to part 5 if the frozen
  store rejects an expanded-text site.
- **Q2. Site anchor.** The expected graph anchors each row at `impl `, not at the trait path.
  Its anchors are marked not canonical, so part 7 reconciles them to R4.
- **Q3. `Buffer`'s three `core` traits** stay unresolved until part 6 decides how `core`
  targets are stored.
- **Q4. Derived traits.** itoa has no `#[derive]`. `derives` edges are not part of 4b.
