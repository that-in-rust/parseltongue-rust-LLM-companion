# Part 4b — `implements` edges from impl headers

Written 2026-11-15 with `rust-executable-specs-01`, before any code.

## The answer

**State: BUILT (all 7 steps, 2026-11-15). One `implements` row per trait impl header, from
the self type to the trait. Pinned itoa stores 32 rows: 29 with a target and 3 unresolved,
including the 28 macro-generated headers. The first target was
`u128 --implements--> Unsigned`, the only original header whose trait is in itoa.**

- **Stored by one run:** 58 nodes and 170 edges (43 `contains`, 61 `calls`, 34 `uses_type`,
  32 `implements`). `implements_without_source` is 0. The graph is still `incomplete`.
- **The 32 rows:** 12 to `Integer`, 12 to `private::Sealed`, 5 to `Unsigned`, and 3 from
  `Buffer` unresolved with `ExternalSourceMissing` (`Default`, `Clone`, `Copy`).
- **Macro-generated headers stayed in this part.** The frozen store accepts a site in
  expanded text, so the 28 did not move to part 5.

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
R4                                                          site equals the range of `Unsigned`
R3     test_buffer_clone_trait_unresolved     real source   Buffer: Clone, Copy, Default, ExternalSourceMissing
R1 R4  test_trait_impl_header_edge            fixture       item self type, item trait: one row at the trait path
R2     test_builtin_source_two_headers        fixture       u8 and u16 each source one row
R2     test_builtin_source_one_node           fixture       two headers on u8: one u8 node; graph validates
R3     test_unresolved_trait_keeps_reason     fixture       stored reason kept; MissingTarget outside the graph
R5     test_inherent_negative_no_edge         fixture       inherent and negative impls: zero rows
R6     test_unresolved_self_counted_only      fixture       zero rows, headers_without_source = 2
R7     test_expanded_header_keeps_chain       fixture       expanded header: same row, path and chain kept
R7     test_macro_header_edge_provenance      real source   28 rows; each site is the trait path in expanded text
R1 R2  test_itoa_implements_row_counts        real source   32 rows: 12 + 12 + 5 with a target, 3 unresolved
R8     test_builtin_source_implements_only    L1 unit       implements passes; uses_type, calls, contains fail
R9     test_implements_rows_read_twice        real source   two reads of the frozen graph are equal, no duplicate
```

Metric for every row: exact equality of source, target or reason, relation and site range.

Changes from the plan: `test_implements_site_trait_range` was not written as its own test;
R4 is asserted inside the three tests marked R4 and in `test_macro_header_edge_provenance`.
`test_expanded_header_keeps_chain` is the fixture test for R7, and the two macro tests and
the read-twice test run on real source instead of a fixture.

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

Run 2026-11-15 at commit `7e2a2ef87`, the four `AGENTS.md` commands.

```text
cargo fmt --all -- --check                                   pass
cargo clippy --release --workspace --all-targets -D warnings pass
cargo test --release --workspace --all-targets               470 passed, 0 failed, 4 ignored
cargo build --release --workspace --all-targets              pass
```

- `test_lang_items_in_sysroot` is skipped, as before, so this is not a full PASS.
- The 22 real-source itoa tests pass inside that run (232.7 s on their own).
- **Cost of one run** (3 runs, sysroot excluded, 2 workers, executable sha256
  `5be64c7b…41d825f3`): 13.0 to 13.7 s, 369 to 371 MB peak over the whole process tree,
  9.5 MB store. Memory is resident memory summed over the tree, sampled about every 18 ms;
  the SOP asks for 10 ms.
- **Test first:** every test had a failing run before its code, with these exceptions.
  - `test_inherent_negative_no_edge` passed on the stub; it asserts an absence.
  - `test_implements_rows_read_twice`: its equality check never failed. Its first version
    also required the rows to be in the Rust record type's sort order, which failed; that
    check was replaced by a no-duplicate check (see Q5).
  - The `MissingTarget` case of `test_unresolved_trait_keeps_reason` was added after its code.
  - `test_expanded_header_keeps_chain` was written after the code and shown to fail with
    the old skip restored.
  - The expanded-text slice check in `test_macro_header_edge_provenance` was added after
    its code; the test's row count had failed before (0 of 28).
- **Refactor pass:** none needed.

## 7 — Open questions

- **Q1. Site of a macro-generated header.** Built as R7 says: the site is the range in the
  expanded text, under the path `.parseltongue/macro-source/<digest>/expanded.rs`, with a
  one-entry provenance chain. The expected graph anchors these rows in the macro definition
  in `src/lib.rs`. Part 5 decides which is canonical.
- **Q2. Site anchor.** The expected graph anchors each row at `impl `, not at the trait path.
  Its anchors are marked not canonical, so part 7 reconciles them to R4.
- **Q3. `Buffer`'s three `core` traits** stay unresolved until part 6 decides how `core`
  targets are stored.
- **Q4. Derived traits.** itoa has no `#[derive]`. `derives` edges are not part of 4b.
- **Q5. Row order.** Two reads return the same order, but it is the store's row-key order,
  not the order of the Rust record type. Nothing depends on the difference today.
- **Q6. Identity across directories.** A row's identity includes the workspace's absolute
  path (through the package id), so two runs in different directories do not give equal
  rows. Part 7 must settle this before any cross-run comparison.
