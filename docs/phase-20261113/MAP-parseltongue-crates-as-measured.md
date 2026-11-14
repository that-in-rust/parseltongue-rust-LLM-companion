# MAP — the parseltongue crates as measured

Measured at commit `54685f5a4` on 2026-11-14. Re-measure after a structural change; do not
edit numbers by hand. This file records structure. It does not say any test passes or any
graph is complete.

## The answer

**Ten crates, 103 Rust files, 50,210 lines. The itoa full item graph is produced by one CLI
stage that wires five of them together. Only two edge kinds have a producer, `contains` and
`calls`, and no code exists for signature lowering, body type inference, method lookup or
trait selection.**

- More than a third of the non-test code is the DuckDB adapter (11,053 of about 29,500 lines).
- A large share of the tree is earlier work this phase does not use: the HLD builder and
  stores, and the oracle comparison.

## The stage that builds the graph

`parseltongue map-workspace-full-run --mapping-final-stage-kind full-item-dependency-graph`,
read from `full-graph-checkpoint-facade.rs`:

```text
1  freeze the source rows       Turso   publish_analysis_input_snapshot
2  project nodes and contains   DuckDB  derive_original_graph_input, in a run-local database
3  resolve calls                Rust    resolve_original_call_edges (syntax-span-extractor)
4  write the graph              Turso   write_full_graph_input
5  read it back                 Turso   assemble_full_graph_input
6  compare                      the read-back must equal what was written, or the run fails
```

It prints `stage=full-item-dependency-graph` and `stage=frozen-full-map-readback` on stderr.
Both end with `completeness=incomplete`; that value is written in the code, not computed.
Unresolved calls are kept as rows with a reason.

## Which edge kinds have a producer

The edge type `HldRelationKindValue` has eight kinds. Outside tests and store codecs:

```text
contains   original-item-graph-projection.rs        this phase's stage
calls      original-call-resolution-records.rs      this phase's stage
imports, uses_type, reads, implements, reexports, derives
           only in the Turso and DuckDB codecs; nothing produces them
```

## Crates

Every name carries the `parseltongue-` prefix. Lines exclude test files, fixtures and examples.

```text
crate                     lines   tests  role
core-row-types            1,930     11   typed rows, identities, graph validation; no dependencies
table-store-traits          494      1   store and analysis ports
turso-store-adapter       3,939     74   durable store, frozen snapshots, graph write and read-back
duckdb-analysis-adapter  11,053    168   name resolution rounds, node and contains projection
crate-graph-loader          644     12   Cargo metadata and sysroot context to crate graph rows
syntax-span-extractor     5,132     57   parsing, impl headers, namespace input, call resolution
port-macro-expander       3,916     22   declarative and built-in macro expansion
macro-span-records          236      2   span types the expander uses, under the name ra_ap_span
stage-worker-scheduler       67      4   worker scheduling
cli-dispatcher-binary     2,076     79   the one binary; wires the stages
```

Dependencies, arrows read "depends on":

```text
cli-dispatcher-binary    -> core-row-types, table-store-traits, duckdb-analysis-adapter,
                            syntax-span-extractor, crate-graph-loader,
                            stage-worker-scheduler, turso-store-adapter
duckdb-analysis-adapter  -> core-row-types, table-store-traits, turso-store-adapter
syntax-span-extractor    -> core-row-types, table-store-traits, port-macro-expander
turso-store-adapter      -> core-row-types, table-store-traits
port-macro-expander      -> core-row-types, macro-span-records
crate-graph-loader       -> core-row-types, table-store-traits
table-store-traits       -> core-row-types
stage-worker-scheduler   -> core-row-types
```

The one edge between the two adapters, DuckDB to Turso, comes from
`oracle-path-comparison-storage.rs`, which this phase does not use.

## Commands and flags

```text
map-workspace-full-run        run the pipeline on a workspace and write the store   USED
export-published-stage-rows   export rows of one published stage table              USED
export-mapped-unit-manifest   print the mapped-unit manifest as JSON
status-coverage-banner-show   coverage banner of a store                            not this phase
oracle-edge-differ-run        compare a store with oracle exports                   not this phase
```

```text
--workspace-root-path-value     --workspace-store-file-path    --worker-thread-count-value
--mapping-final-stage-kind      syntax-spans | name-resolution (default) |
                                original-top-level-containment | full-item-dependency-graph
--sysroot-source-scope-kind     included (default) | excluded
--stage-owned-table-name        --stage-row-filter-value       --json
```

## Real-source tests on pinned itoa

`crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs`, 11 tests
over the vendored source in `tests/pinned-itoa-source-tree/`. Two cover the full graph:
`test_original_member_graph_roundtrip` and `test_default_new_call_roundtrip`.

## Code in the tree that this phase does not use

```text
hld-materialization-duckdb-store.rs      1,312 lines   HLD builder
hld-original-source-projection.rs        1,041         original-top-level-containment stage
hld-graph-publication-turso.rs           1,238         HLD publication
oracle-path-comparison-storage.rs        2,388         oracle comparison
oracle-path-comparison-core.rs             996         oracle comparison
tools/hld-current-qualification-runner.py              SOP runner for the earlier five-repository
                                                       scope; defaults to deleted report folders
```

`hld-graph-input-turso.rs` (1,636 lines) is used: it stores and reads back the full graph
despite its name.

## How it was measured

```bash
git ls-files 'crates/*.rs' | wc -l                 # files; lines with xargs cat | wc -l
clarity show crates --collapse                     # crate dependencies (clarity 0.35.0)
codebase-memory-mcp cli index_repository --repo_path "$PWD" --mode full
```

Command names, flags and stage kinds were read from
`crates/parseltongue-cli-dispatcher-binary/src/main.rs`.
