# Part 4d — `imports` edges from `use` items

Written 2026-11-15 with `rust-executable-specs-01`, before any code. Reviewed first by
GLM-5.3 (Rule 5):
[record](../../devin-research/1794730674-second-opinion-glm-5-3-max-part-4d-imports-design.md).
The built code was reviewed again before commit:
[record](../../devin-research/1794740417-second-opinion-glm-5-3-max-part-4d-diff-review.md).

## The answer

**State: BUILT (2026-11-15, commit `5dabc3279`). One `imports` row per name a `use` item
imports, from the `use` node to what the name resolves to. On pinned itoa that is 4 rows,
all into `core`, so all 4 are stored unresolved with reason `ExternalSourceMissing` and
their full path text until part 6 names `core` targets. Measured total after 4d: 195 edges
(191 + 4), as predicted. Against the expected graph: 0 of 4 rows match by target, 4 of 4 by
source node, 2 of 4 by exact site range (Q1, Q4).**

The measurements below were taken before any code, and are what the design started from.

Measured on one run of pinned itoa (commit `fc0ebe4e0`, sysroot excluded, exported with
`export-published-stage-rows`):

```text
fact                                                     where
3 active use items are nodes, kind UseStatement          item-effective-visibility-records
  byte ranges [1995,2010] [2011,2046] [2047,2061]
2 cfg-inactive use items (no_panic) are not nodes        src/lib.rs:59-60, src/u128_ext.rs:1-2
8 import request rows                                    syntax-namespace-input-imports
  4 of kind Import, 4 of kind Signature (trait bounds)
the 4 Import rows:  core::hint            [1999,2009]
                    core::mem::self       [2027,2031]
                    core::mem::MaybeUninit [2033,2044]
                    core::str             [2051,2060]
no row anywhere says what an import resolved to          all 5 name-resolution tables read
a resolved import shows only as a binding whose          module-scope-binding-table
  binding_import_evidence_keys hold the import site        (9 of 140 bindings, all Signature)
```

The gap is therefore in the analysis stage, not in syntax: the import requests exist and
are resolved inside DuckDB, but their outcome is not written out.

## 1 — Work mode

Spec Mode and Delivery Mode. Component: the DuckDB name-resolution stage (one new output
table) and a new edge resolver in `parseltongue-syntax-span-extractor`. Main risk: a wrong
reason or a wrong source node, not performance.

## 2 — Executable requirements

REQ-RUST-049.0. Scope: `use` items in original source text of pinned itoa.

```text
R1. WHEN name resolution finishes, THEN it SHALL write one row per import request of kind
    Import to import-resolution-site-records: the import site, the state (resolved,
    ambiguous, unresolved), the resolved targets, and the unresolved reason. The reason
    external_source_missing SHALL come from the same SQL rule that path requests use
    (REQ-RUST-038.1), not from a second copy of it.
R2. WHEN an import site lies inside the byte range of exactly one UseStatement node of the
    same unit and file with no expansion chain, THEN that node SHALL be the source of the
    site's imports rows.
R3. WHEN the import resolved and a target is a graph node, THEN one imports row SHALL be
    stored per distinct target, Extracted. One site may have more than one row (a name in
    the type and the macro namespace).
R4. WHEN the import resolved and no target is a graph node, THEN one unresolved row SHALL
    be stored with reason MissingTarget.
R5. WHEN the import did not resolve, THEN one unresolved row SHALL be stored: AmbiguousTarget
    for an ambiguous import, the stored reason when there is one, MissingTarget otherwise.
R6. Every imports row SHALL carry import_path_text: the segments joined by "::", with a
    trailing self dropped (core::mem::{self} gives core::mem). The alias, when present, is a
    separate field, import_alias_text.
    Part 6 builds the named external target from this text, so it is part of the contract.
R7. The evidence site SHALL be the byte range the import request stores: the whole path for
    a plain use, the leaf for a member of a group. The evidence source is the use node.
R8. Requests of kind Signature SHALL never produce a row.
R9. A glob import (use a::*) SHALL produce no row in this part; it is counted as
    imports_glob_skipped. Its bindings name the module's children, not the module.
R10. An import site in macro-expanded text, or inside no UseStatement node (a use in a
    function body), SHALL produce no row; it is counted as imports_in_expanded_items or
    imports_without_source. The checks run in this order: expanded, no use node, glob. A
    glob use in a function body is therefore counted as imports_without_source.
R11. A cfg-inactive use item has no node and no request, and SHALL produce no row.
R12. Two reads from the same store SHALL return identical rows in identical order.
```

Not in this part: `reexports` (a `pub use` gets an `imports` row like any other; itoa has
no `pub use` and the expected graph has 0 `reexports` rows).

## 3 — Design by layer

- **L1** `parseltongue-core-row-types`: nothing new. `HldRelationKindValue::Imports` and
  `HldItemKindValue::UseStatement` exist, and the validator accepts an unresolved `imports`
  row from a `LocalItem` node (checked in source by the review, Q6).
- **L2** `parseltongue-duckdb-analysis-adapter`: `namespace-final-output-records.rs` emits
  `import-resolution-site-records` next to the binding rows, from the import requests,
  their candidates and the existing `absent-sysroot` rule. As built: the row is written
  by `namespace-final-path-products.sql`, in addition to the request's unchanged path row.
  `namespace-import-request-records` stores `request_kind` (`import`, `signature`,
  `prelude`), and only kind `import` gets a row.
- **L2** `parseltongue-syntax-span-extractor`: `resolve_use_import_edges(store, scope, input,
  cancel) -> Result<UseImportEdgeSet, UseImportEdgeError>`, the same shape as
  `resolve_impl_header_edges`. It joins stored rows only and looks no name up.
- **L3** store and CLI: the new table is registered with the name-resolution stage; the
  full-graph facade adds the rows and prints `imports=` and the three counters.

## 4 — Verification matrix

```text
req        test                                 type          assertion
R1         test_import_outcome_rows_written     fixture       resolved, unresolved, sysroot-missing; a bound request has no row
R2 R3 R6 R7 test_resolved_import_names_node     fixture       use crate::Buffer as B: one row, use node to Buffer, alias B
R2 R6 R7   test_grouped_import_two_rows         fixture       use core::mem::{self, MaybeUninit}: two rows, two sites
R4 R5      test_unresolved_import_keeps_reason  fixture       not a node, ambiguous, no reason, cfg_inactive, trait_solver_needed
R8 R9 R10  test_skipped_import_sites_counted    fixture       bound, 2 globs, expanded, body use: zero rows, counts 2, 1, 1
errors     test_import_scope_cancel_errors      fixture       cancelled, foreign scope, absent outcome row
R1 R6 R7 R11 test_itoa_imports_row_counts       real source   4 stored outcomes; 4 rows, ExternalSourceMissing, 4 path texts
R12        test_imports_rows_read_twice         real source   two reads equal, no duplicate
```

Metric for every row: exact equality of source, target or reason, relation, site range and
`import_path_text`.

## 5 — Implementation plan

Each step is one commit of about 200 changed lines, test first.

1. RED: `test_itoa_imports_row_counts` on pinned itoa (0 `imports` rows today).
2. L2 analysis: `import-resolution-site-records` written (R1), fixture test first; the table
   registered in the store.
3. L2 resolver: stub, fixture tests RED, then R2 to R8.
4. L3 facade: rows in the full graph, `imports=` on the stage line; step 1 goes GREEN.
5. Counters (R9, R10) and `test_imports_rows_read_twice` (R12).
6. Refactor pass, workspace gates, one run's counts and cost.

All six steps are done. Two more were added after the second review of the built code:

- 2b. Only requests of kind `Import` get an outcome row. The first build wrote 8 rows on
  itoa (4 `Import`, 4 `Signature`); `request_kind` is now stored and 4 rows are written.
- 3b. A stored `trait_solver_needed` reason is kept, not turned into `Other`. The skip
  test uses two globs, so swapped counters fail it.

One defect was found and fixed while building step 4: the first SQL moved the 6 import
rows out of `path-resolution-site-records` (301 rows became 295) and one call lost its
target (3 resolved calls became 2). The path table is now unchanged (301 rows, no row
differs from the commit before 4d) and the import row is written in addition.

## 6 — Quality gate results

Measured at commit `5dabc3279`, release build, executable sha256 `22fd0e31...dd88a84`.

```text
gate                                             result
cargo fmt --all -- --check                       pass
clippy --workspace --all-targets -D warnings     pass
workspace tests, one thread                      494 passed, 0 failed, 4 ignored
  test_lang_items_in_sysroot                     skipped, so not a full PASS
real-source itoa tests                           28 passed, 0 failed, 0 ignored (331 s)
release build                                    pass
one run on pinned itoa, 3 times                  13.5 to 14.1 s
  peak memory, whole process tree                334 to 345 MB
  store size                                     11.5 MB
  stored                                         58 nodes, 195 edges, 4 imports, 3 resolved calls
  import-resolution-site-records                 4 rows, all unresolved, external_source_missing
  skipped import sites                           0 glob, 0 expanded, 0 without source
```

Two files are over the 200-line guide: `use-import-edge-tests.rs` (fixture builder and
five tests) and `use-import-edge-records.rs`. Neither was split.

## 7 — Open questions

- **Q1: evidence ranges against the expected graph.** Production stores `self` [2027,2031]
  and `MaybeUninit` [2033,2044]; the expected graph has `mem::{self` [2021,2031] and
  `MaybeUninit}` [2033,2045], which are not the range of any syntax node. The two plain
  uses agree. Left for the freeze in part 7; production is not changed to fit.
- **Q2: glob imports and `reexports`** are counted or left out, not stored. Enough for
  itoa (no glob, no `pub use`); a hole for any other crate.
- **Q3: an import renamed to `_`** creates no binding (`namespace-import-binding-rounds.sql`
  line 11). The outcome row of R1 still records it as resolved. No case in itoa.
- **Q5: one site with two targets (R3) has no test.** The resolver stores one row per
  distinct target, but no fixture gives one import site a type and a macro of the same
  name. No case in itoa.
- **Q6: path text for `::a` and `$crate::a`.** A leading `::` is kept as text; `$crate` is
  written as `$crate`, without the crate it stands for. Neither occurs in itoa's `use`
  items. To settle before part 6 builds external targets from `import_path_text`.
- **Q7: an import request with a `use` node and no stored outcome row** stops the graph
  build with a typed error (`InvalidEvidence`, tested). The second review suggested a
  counted skip when the unit is not mapped. Not taken: on itoa every unit with a `use` node
  is mapped, and a skip would hide a store defect.
- **Q8: ambiguous imports are tested in the resolver only.** The table fixture has no
  import with two candidates, so the `ambiguous` state is not shown at the SQL step.
- **Q4: 4d ends with stored rows, not with a match.** The expected graph has the 4 rows as
  resolved to external `core` targets. Until part 6, 0 of 4 match by target; 4 of 4 match by
  source node and 2 of 4 by exact site range.
