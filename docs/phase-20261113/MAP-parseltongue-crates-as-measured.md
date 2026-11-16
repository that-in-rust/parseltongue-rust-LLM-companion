# MAP — the parseltongue crates as measured

Measured at commit `37ea1755d` on 2026-11-16. Re-measure after a structural change; do not
edit numbers by hand. This file records structure. It does not say any test passes or any
graph is complete.

## The answer

**Ten crates, 138 Rust files, 70,497 lines. The itoa full item graph is produced by one CLI
stage that wires five of them together. Eight of the ten edge kinds have a producer. No code
exists for lowered signature types, type inference in bodies or a trait solver.**

- Two crates hold 62% of the non-test code: the DuckDB adapter (11,710 lines) and the syntax
  extractor (11,234), of 36,745.
- A large share of the tree is earlier work this phase does not use: the HLD builder and
  stores, and the oracle comparison.

## The stage that builds the graph

`parseltongue map-workspace-full-run --mapping-final-stage-kind full-item-dependency-graph`,
read from `full-graph-checkpoint-facade.rs`:

```text
1  freeze the source rows       Turso   publish_analysis_input_snapshot
2  project nodes and three      DuckDB  derive_original_graph_input, in a run-local database
   edge kinds                           (contains, generates, expands)
3  resolve five edge kinds      Rust    resolve_original_call_edges, resolve_signature_type_edges,
                                        resolve_impl_header_edges, resolve_body_read_edges,
                                        resolve_use_import_edges (syntax-span-extractor)
4  add endpoint nodes           Rust    collect_builtin_endpoint_nodes,
                                        collect_external_endpoint_nodes
5  write the graph              Turso   write_full_graph_input
6  read it back                 Turso   assemble_full_graph_input
7  compare                      the read-back must equal what was written, or the run fails
```

It prints `stage=full-item-dependency-graph` and `stage=frozen-full-map-readback` on stderr.
Both end with `completeness=incomplete`; that value is written in the code, not computed.
Unresolved calls are kept as rows with a reason.

## Which edge kinds have a producer

The edge type `HldRelationKindValue` has ten kinds. Outside tests, store codecs and the
earlier HLD code:

```text
contains    original-item-graph-projection.rs, generated-item-graph-projection.rs   DuckDB adapter
generates   generated-item-graph-projection.rs                                      DuckDB adapter
expands     macro-call-expands-projection.rs                                        DuckDB adapter
calls       original-call-resolution-records.rs                                     syntax extractor
uses_type   signature-type-edge-records.rs                                          syntax extractor
implements  impl-header-implements-records.rs                                       syntax extractor
reads       body-value-read-records.rs                                              syntax extractor
imports     use-import-edge-records.rs                                              syntax extractor
reexports, derives                                                                  no producer
```

## Crates

Every name carries the `parseltongue-` prefix. Lines exclude test files, fixtures and examples.

```text
crate                     lines   tests  role
core-row-types            2,122     23   typed rows, identities, graph validation; no dependencies
table-store-traits          494      1   store and analysis ports
turso-store-adapter       4,055     75   durable store, frozen snapshots, graph write and read-back
duckdb-analysis-adapter  11,710    187   name resolution rounds, node and structural edge projection
crate-graph-loader          644     12   Cargo metadata and sysroot context to crate graph rows
syntax-span-extractor    11,234    177   parsing, impl headers, namespace input, five edge resolvers
port-macro-expander       3,916     22   declarative and built-in macro expansion
macro-span-records          236      2   span types the expander uses, under the name ra_ap_span
stage-worker-scheduler       67      4   worker scheduling
cli-dispatcher-binary     2,267    129   the one binary; wires the stages
```

"Tests" counts `#[test]` attributes in the crate, the 60 real-source itoa tests included.

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
--sysroot-source-scope-kind     included (default) | excluded | declarations (SPEC part 8)
--stage-owned-table-name        --stage-row-filter-value       --json
```

## Real-source tests on pinned itoa

`crates/parseltongue-cli-dispatcher-binary/tests/itoa-source-acceptance-tests.rs`, 60 tests
in 5,980 lines, over the vendored source in `tests/pinned-itoa-source-tree/`.
`test_itoa_whole_graph_equality` compares the whole stored graph with the frozen expected
graph.

## Code in the tree that this phase does not use

```text
hld-materialization-duckdb-store.rs      1,329 lines   HLD builder
hld-original-source-projection.rs        1,043         original-top-level-containment stage (*)
hld-graph-publication-turso.rs           1,257         HLD publication
oracle-path-comparison-storage.rs        2,388         oracle comparison
oracle-path-comparison-core.rs             996         oracle comparison
tools/hld-current-qualification-runner.py              SOP runner for the earlier five-repository
                                                       scope; defaults to deleted report folders
```

(*) One function in that file is used: `open_source_projection_store` opens the DuckDB
database of the graph stage.

`hld-graph-input-turso.rs` (1,728 lines) is used: it stores and reads back the full graph
despite its name.

## How it was measured

```bash
git ls-files 'crates/*.rs' | wc -l                 # files; lines with xargs cat | wc -l
# per crate: lines of .rs files whose path has no "test", "fixture" or "example"
git ls-files crates/<crate> | grep '\.rs$' | grep -vi 'test\|fixture\|example' | xargs cat | wc -l
# per crate: tests
git ls-files crates/<crate> | grep '\.rs$' | xargs grep -hc '^\s*#\[\(tokio::\)\?test\]'
# producers of an edge kind
grep -rl 'HldRelationKindValue::<Kind>' --include=*.rs crates | grep -vi test
```

Crate dependencies were read from each `Cargo.toml`.
Command names, flags and stage kinds were read from
`crates/parseltongue-cli-dispatcher-binary/src/main.rs`.
