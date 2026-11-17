# Part 10 — the mapper runs without a DuckDB extension file

Back to the [index](README.md).

Status on 2026-11-17: **specified, not built.** Decided by the user on 2026-11-17: the
mapper must run on a machine that has no `~/.duckdb` directory.

**The defect.** The graph stage runs `LOAD json`. The executable does not contain the
`json` extension, so DuckDB reads
`~/.duckdb/extensions/v2.0.0/linux_amd64/json.duckdb_extension`. On a machine without that
file the run stops with `IO Error: Extension ... not found` (seen 2026-11-16). The file is
an input no step of the SOP freezes.

**The change.** Build the `json` extension into the executable and remove the `LOAD`.

## 1. Rust work mode

- **Component:** the DuckDB adapter crate, `parseltongue-duckdb-analysis-adapter`.
- **Modes:** Delivery and Reliability. The risk is an input outside the freeze, not design.
- **Measured before writing** (2026-11-17, commit `f2380e602`):
  - One `LOAD json` in production code: `hld-original-source-projection.rs`, line 103.
  - The SQL of the graph projection calls `json_extract_string`, `json_extract` and
    `json_array_length` (`hld-original-source-projection.rs`,
    `original-item-graph-projection.rs`), so the functions are needed.
  - The crate `duckdb` 1.20000.1 has a feature `json`, which compiles the extension into
    the bundled library. The adapter enables `bundled` only.
- **Not measured:** build time and executable size with the feature. Predicted: the build
  of DuckDB runs once more (about 15 minutes); the size grows.

## 2. Executable requirements: REQ-RUST-068.0

```text
A1  WHEN the mapper runs on pinned itoa with HOME set to an empty directory
    THEN it SHALL exit 0 and publish the full item graph
A2  WHEN that run has ended THEN the HOME directory SHALL still be empty
A3  WHEN a run is traced THEN no path that contains `.duckdb/` SHALL be opened
A4  WHEN the graph of a run with scope `declarations` is compared with the frozen expected
    graph THEN it SHALL be equal, as before this part (170 nodes, 682 edge rows)
```

Out of scope: the other DuckDB settings of the run, and the loader crate, which loads no
extension.

## 3. Design (L1 / L2 / L3)

- **L3 only.** Two edits in the adapter crate:
  - `Cargo.toml`: `duckdb` gains the feature `json`.
  - `hld-original-source-projection.rs`: the statement `LOAD json;` is removed. The
    extension is part of the library, so there is nothing to load. If a build lacks the
    feature, the first JSON function fails with DuckDB's "function does not exist", which
    names the cause.
- `autoinstall_known_extensions` and `autoload_known_extensions` stay `false`.
- No row type, table or trait changes. The executable's digest changes, so the acceptance
  run of `753665eb9` no longer describes the current executable (SOP: any production
  change invalidates a run).

## 4. Verification matrix

```text
req      test                                   type          assertion
A1 A2    test_empty_home_graph_published        real source   exit 0; node and edge tables published;
                                                              the HOME directory has no entry
A3       strace of one run                      manual        0 opens with `.duckdb/` in the path
A4       test_itoa_whole_graph_equality,        real source   unchanged tests pass
         test_itoa_core_member_targets
```

The test passes `CARGO_HOME` and `RUSTUP_HOME` of the real home, because the run starts
`cargo metadata` and `rustc`, which are found through them.

## 5. Implementation plan

```text
10.1  test_empty_home_graph_published, seen failing with "Extension ... not found"
10.2  the feature and the removed LOAD; the test passes
10.3  A3 by strace with ~/.duckdb moved away; the 61 real-source tests; log entry
```

## 6. Quality gate results

Not run: nothing is built.

## 7. Open questions

- **The loader crate** enables `bundled` only. In a workspace build Cargo gives it the
  `json` feature too, because both crates use the same `duckdb`. It loads no extension
  either way.
