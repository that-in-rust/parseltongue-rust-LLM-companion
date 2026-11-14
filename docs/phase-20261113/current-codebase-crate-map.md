# Current codebase crate map — phase-20261113

What the code is today, measured at commit `745881721` on 2026-11-14. This file describes
the tree as it stands; it is not a requirement, a plan or a run result. The
[SOP](SOP-phase-20261113-full-item-dependency-graph.md) cites it for command names and for
which stage does what. Re-measure after a structural change instead of editing by hand.

## How it was measured

```bash
npm install -g @legacycodehq/clarity          # clarity 0.35.0
clarity show crates --collapse > crates-collapsed.dot
```

The file-level DOT graph was reduced to crate-level edges by counting file -> file edges whose
source file is under a crate's `src/`. Command names, flags and stage kinds were read from
`crates/parseltongue-cli-dispatcher-binary/src/main.rs`.

Size: 10 workspace crates, 103 tracked `.rs` files under `crates/`, 48,756 lines. That count
includes test files and the vendored pinned itoa source used as a test input.

## Crate dependencies

Arrows read "depends on". The number is how many file-level edges clarity found.

```text
 cli-dispatcher-binary    -> core-row-types(6) table-store-traits(6) duckdb-analysis-adapter(5)
                             syntax-span-extractor(3) crate-graph-loader(1)
                             stage-worker-scheduler(1) turso-store-adapter(1)
 duckdb-analysis-adapter  -> core-row-types(31) table-store-traits(9) turso-store-adapter(3)
 syntax-span-extractor    -> core-row-types(10) table-store-traits(6) port-macro-expander(1)
 turso-store-adapter      -> core-row-types(7) table-store-traits(5)
 port-macro-expander      -> core-row-types(5)
 crate-graph-loader       -> core-row-types(2) table-store-traits(1)
 table-store-traits       -> core-row-types(4)
 stage-worker-scheduler   -> core-row-types(1)
 core-row-types           -> (nothing)
 macro-span-records       -> (no cross-crate edge found by clarity)
```

`macro-span-records` is used by `port-macro-expander` under the dependency name `ra_ap_span`
(a path override in the workspace `Cargo.toml`), which is why clarity sees no edge.

Every crate name carries the `parseltongue-` prefix, dropped above for width.

Layering as the edges show it:

```text
 L1  core-row-types            typed rows, identities, graph validation; depends on nothing
 L2  table-store-traits        store and analysis ports
 L3  turso-store-adapter       durable store, frozen snapshots, graph input and publication
 L3  duckdb-analysis-adapter   relational analysis, projection, materialization, comparison
     crate-graph-loader        Cargo metadata and sysroot context -> crate graph rows
     syntax-span-extractor     parsing, impl headers, namespace input, call resolution
     port-macro-expander       declarative and built-in macro expansion
     stage-worker-scheduler    worker scheduling (one 67-line file)
     cli-dispatcher-binary     the one binary; wires the stages together
```

One edge crosses between the two L3 adapters: `duckdb-analysis-adapter` uses
`turso-store-adapter` from `oracle-path-comparison-storage.rs`, from two test modules and from
one example. Only the first is production code.

## Largest source files per crate

```text
 cli-dispatcher-binary     main.rs 1,208 · namespace-pipeline-publication-facade.rs 496
                           published-stage-export-facade.rs 187
                           hld-source-publication-facade.rs 86 · full-graph-checkpoint-facade.rs 82
 core-row-types            hld-materialization-core-records.rs 1,026 · lib.rs 443
                           name-resolution-stage-records.rs 230
 crate-graph-loader        lib.rs 546 · sysroot-compilation-context-rows.rs 98
 duckdb-analysis-adapter   oracle-path-comparison-storage.rs 2,388
                           hld-materialization-duckdb-store.rs 1,312
                           hld-original-source-projection.rs 1,041
                           oracle-path-comparison-core.rs 996
                           original-item-graph-projection.rs 386   (35 files in total)
 port-macro-expander       macro-rule-matching-logic.rs 1,091 · macro-token-transcription-logic.rs 648
                           rule-pattern-parsing-logic.rs 538 · macro-expansion-rule-engine.rs 492
 syntax-span-extractor     syntax-namespace-input-extraction.rs 1,633 · lib.rs 742
                           impl-header-source-records.rs 507
                           original-call-resolution-records.rs 411
 table-store-traits        lib.rs 322 · hld-graph-input-store-port.rs 78
                           hld-graph-publication-port.rs 55
 turso-store-adapter       hld-graph-input-turso.rs 1,636 · hld-graph-publication-turso.rs 1,238
                           stage-table-store-backend.rs 871
```

Test modules are left out of this list; the largest are
`hld-materialization-contract-tests.rs` (3,821 lines) and `hld-publication-contract-tests.rs`
(1,968 lines).

## Commands the binary accepts

```text
 map-workspace-full-run        run the pipeline on a workspace and write the store
 status-coverage-banner-show   print the coverage banner of an existing store
 export-mapped-unit-manifest   print the mapped-unit manifest as JSON
 export-published-stage-rows   export rows of one published stage table
 oracle-edge-differ-run        compare the store with PRIMARY and SECONDARY oracle exports
```

Flags:

```text
 --workspace-root-path-value       --workspace-store-file-path     --worker-thread-count-value
 --mapping-final-stage-kind        --sysroot-source-scope-kind
 --stage-owned-table-name          --stage-row-filter-value
 --primary-semantic-input-path     --secondary-semantic-input-path --agreement-store-file-path
 --json
```

`--mapping-final-stage-kind` takes one of four values; `name-resolution` is the default:

```text
 syntax-spans                     stop after source closure and syntax extraction
 name-resolution                  stop after namespace analysis (default)
 original-top-level-containment   earlier checkpoint: top-level contains edges only, run through
                                  the HLD builder and published as the five HLD tables
 full-item-dependency-graph       the active Phase 1 stage, described below
```

`--sysroot-source-scope-kind` takes `included` (default) or `excluded` (REQ-RUST-038.0).

There is no `hld-current-export-sorted` command in the binary. Rows are exported with
`export-published-stage-rows`.

## What the full-item-dependency-graph stage does

Read from `full-graph-checkpoint-facade.rs`, in order:

```text
 1  freeze the source rows       store.publish_analysis_input_snapshot
 2  project nodes and contains   DuckDB derive_original_graph_input, in a run-local database
                                 <store>.full-map-source-analysis/<generation>/<attempt>/
                                 original-item-graph-projection.duckdb
 3  resolve calls in Rust        syntax-span-extractor resolve_original_call_edges
 4  write the graph input        Turso write_full_graph_input
 5  read it back                 Turso assemble_full_graph_input
 6  compare                      the frozen read-back must equal the source-derived bundle,
                                 otherwise the run fails
```

It prints two lines on stderr, `stage=full-item-dependency-graph` and
`stage=frozen-full-map-readback`. Both end with `completeness=incomplete`, and that value is
written in the code rather than computed: the stage cannot report a complete graph today.
Unresolved calls are kept as rows with a reason, not dropped.

The stage does not select interface nodes, collapse paths, build roll-ups or switch the serving
manifest. That code exists separately (`hld-materialization-duckdb-store.rs`,
`hld-graph-publication-turso.rs`). It is reached from real source only by the
`original-top-level-containment` stage (`publish_original_source_hld`), for top-level
containment; everything else about it is exercised by component tests on typed fixtures.

## Real-source tests on pinned itoa

`crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs` holds 11 tests
over the vendored source in `tests/pinned-itoa-source-tree/`:

```text
 test_original_member_graph_roundtrip    test_default_new_call_roundtrip
 test_impl_header_row_contents           test_itoa_pipeline_reads_published
 test_itoa_top_level_containment         test_source_visibility_scope_equivalence
 test_source_module_owner_paths          test_source_fixture_scope_rules
 test_source_mutation_identity_changes   test_sysroot_scope_excluded_units
 test_sysroot_reached_names_reasoned
```

Four tests in the workspace are marked `#[ignore]`: one subprocess fixture in
`workspace-cli-contract-tests.rs` and three manual insert profiles in
`stage-table-store-backend.rs`.

## What this file does not claim

It records structure, not correctness. Nothing here says a test passes, a graph is complete or
a repository is qualified. No run report exists in the tree as of commit `745881721`.
