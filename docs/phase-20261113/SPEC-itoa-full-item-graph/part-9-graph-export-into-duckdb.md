# Part 9 — the exported graph as a DuckDB file

Status on 2026-11-17: **built, and the full run passed.** Pinned itoa 1.0.18 went through
the frozen `parseltongue` executable, the two tables were exported, and the crate
`parseltongue-graph-dataset-loader` wrote a DuckDB file whose rows equal the exported rows:
170 node rows and 682 edge rows, every column, 0 differences (section 6).

Decided by the user on 2026-11-17: a separate crate reads the JSON a run exports, checks
it, and writes it into one DuckDB file. The mapper, the Turso store and the exported JSON
are not changed.

**Done for this part** (user, 2026-11-17): one full run in which pinned itoa goes through
`parseltongue`, the two tables are exported, and the loader writes a DuckDB file whose
rows equal the exported rows.

## 1. Rust work mode

Spec Mode and Delivery Mode. The component is a library with a small binary. The main risk
is a file that differs from the JSON without anyone noticing, so the loader reads its own
file back before it publishes it.

### Measured before writing (the S2 export of the acceptance run, executable `af84bb46…`)

```text
file                    lines  first line  rows  bytes
full-graph-nodes.json   171    header      170   298,874
full-graph-edges.json   683    header      682   5,381,169
```

- **Header:** `export_owned_table_name`, `export_record_kind_value` = `header`, and
  `export_scope_identity_record` with `run_generation_digest_value`.
- **Row:** `export_record_kind_value` = `row`, `export_row_identity_value` (text) and
  `export_row_payload_value` (object).
- **Node identity:** the identity text of a node row is JSON equal to the row's
  `node_target_identity`: 170 of 170.
- **Edge ends:** `edge_source_target_identity` is the identity of a node in 682 of 682
  rows, and `edge_resolution.target` in 681 of 681. One row has no target (`kind` =
  `Unresolved`, with a `reason`): the `cast`.
- **Identities are distinct:** 170 node identities, 682 edge identities.
- **DuckDB's `json` extension is not in the executable** (immediate backlog, entry 6), so
  the loader must not need it.

## 2. Executable requirements: REQ-RUST-067.0

The input is the two files `export-published-stage-rows` writes for one published run:
nodes (`item-effective-visibility-records`) and edges (`edge-evidence-record-table`).

```text
A1  WHEN the loader reads the export of a declarations run of pinned itoa
    THEN it SHALL write one file holding 170 node rows and 682 edge rows
A2  WHEN the three tables are read back
    THEN each payload text SHALL parse to a value equal to the JSON row's payload, and
    each other column SHALL equal the payload field it copies, for every row
A3  WHEN the same export is loaded twice
    THEN the rows read back from both files, ordered by key, SHALL be equal
A4  WHEN an input breaks one of the checks C1 to C7
    THEN the loader SHALL return that check's error and no file SHALL exist at the output path
A5  WHEN the output path exists
    THEN the loader SHALL return an error and leave that file as it was
A6  WHEN the loader runs THEN it SHALL open no file under `~/.duckdb`
A7  WHEN the loader has run THEN the two input files SHALL have the bytes they had before
```

Checks, all made before any file is created:

```text
C1  each file's first line is a header naming the expected table; every other line is a row
C2  both headers name the same run generation, and every row's run_generation_digest_full equals it
C3  a node's identity text is JSON equal to its node_target_identity
C4  no two nodes and no two edges share an identity
C5  every edge's source is a node
C6  an edge has a target that is a node, or no target and a reason
C7  each file has at least one row
```

## 3. Design (L1 / L2 / L3)

- **A new crate, `parseltongue-graph-dataset-loader`:** a library and a binary of that
  name. It depends on `duckdb` (the workspace's pinned version, bundled), `serde_json`,
  `sha2`, `thiserror` and `tempfile`, and on no crate of this workspace. `parseltongue`
  does not depend on it, so the frozen executable's bytes do not change.
- **L1, pure:** `parse_graph_export_text` (C1, C2, C7) and `check_graph_export_rows`
  (C2 to C6) take text and return `GraphDatasetRowTables`, the rows of the three tables.
- **L3, effects:** `write_graph_dataset_file` and `read_graph_dataset_file` are the only
  code that touches DuckDB or the file system. `load_graph_export_files` joins the two.
- **Errors:** one enum, `GraphDatasetLoadError`, one variant per check, carrying the file,
  the line number and the identity where they apply.
- **No JSON type in DuckDB.** A nested value is stored as compact JSON text with keys
  sorted. Each row keeps its whole payload as such text, so no field is lost; the other
  columns are copies of payload fields for SQL.
- **A node's key** is the compact sorted JSON of its `node_target_identity`. An edge's
  source and target columns hold the same form, so they join to the node table.
- **All or nothing.** The file is written in a temporary directory beside the output,
  closed, opened again read-only and read back. Only when the rows read back equal the
  rows checked is it linked to the output path, with a call that fails when the path
  exists.

```text
table                     one row per   columns
full_graph_dataset_info   file          run_generation_digest_value, node_row_count_value,
                                        edge_row_count_value, node_file_digest_sha256,
                                        edge_file_digest_sha256
full_graph_node_rows      node          node_identity_key_text (primary key),
                                        node_qualified_path_text, node_input_category,
                                        node_item_kind_value, node_owner_kind_value,
                                        node_source_relative_path,
                                        node_function_body_ancestry_flag,
                                        node_effective_visibility_kind,
                                        node_reachable_outside_crate_flag,
                                        node_signature_text, node_payload_json_text
full_graph_edge_rows      edge site     edge_identity_key_text (primary key),
                                        edge_relation_kind, source_node_identity_key,
                                        target_node_identity_key (null without a target),
                                        edge_resolution_kind_value,
                                        edge_unresolved_reason_value,
                                        site_source_file_path, site_byte_start_value,
                                        site_byte_end_value, edge_dispatch_metadata_text,
                                        edge_payload_json_text
```

## 4. Verification matrix

```text
req_id   test                                  type         assertion
C1       test_export_header_line_required      unit         no header, or another table's header: error
C2       test_run_generation_rows_agree        unit         a row or the second header of another run: error
C3       test_node_identity_text_checked       unit         identity text unlike node_target_identity: error
C4       test_duplicate_row_identity_rejected  unit         two nodes, or two edges, with one identity: error
C5       test_edge_source_node_required        unit         source not in the node file: error
C6       test_edge_target_node_required        unit         target not a node; no target and no reason: error
C7       test_export_file_rows_required        unit         header only: error
A2 A3    test_small_export_dataset_roundtrip   unit, DuckDB rows read back equal the rows checked; two loads equal
A4       test_failed_check_writes_nothing      unit, files  output path absent after a failed check
A5 A7    test_existing_output_path_kept        unit, files  output and inputs keep their bytes
A1 A2 A7 test_itoa_export_dataset_roundtrip    real source  170 and 682 rows; every row equals its JSON row;
                                                            the export files keep their bytes
A4       test_wrong_flag_usage_exit            binary       wrong flags: exit 2, usage text, no file
A4       test_missing_export_file_exit         binary       unreadable export: exit 1, no file
A6       strace of the binary, once            manual       no path under ~/.duckdb opened
```

The real-source test runs the `parseltongue` executable on a copy of pinned itoa with scope
`declarations`, exports the two tables with the same executable, runs the loader binary,
and compares the DuckDB rows with rows it parses from the JSON itself, not through the
loader's parser. The executable is `PARSELTONGUE_FROZEN_EXECUTABLE_PATH` when set,
otherwise `target/release/parseltongue`; when neither exists the test fails and says so.

## 5. Implementation plan

```text
9.1  crate, error enum, parser with C1, C2, C7           unit tests first
9.2  checks C3 to C6 and the table rows                  unit tests first
9.3  write, read back, all or nothing (A2 to A5, A7)     unit tests first
9.4  binary and the real-source test (A1, A2)
9.5  A6 by strace; workspace gates; the full run recorded in the log
```

All five steps are done. Steps 9.1 to 9.3 were test first: the eleven unit tests ran
against stubs and 10 failed (the eleventh tests only the JSON text form). The real-source
test and the two binary tests of 9.4 were written after the code and were never seen to
fail.

## 6. Quality gate results

Run on 2026-11-17 at commit `c21eacd2b`. Commands and digests are in the log entry of
that day.

```text
format            cargo fmt --all -- --check                                  exit 0
lint, crate       clippy -p parseltongue-graph-dataset-loader, -D warnings    exit 0
lint, workspace   clippy --workspace --all-targets --all-features             exit 0
tests, crate      14 pass, 0 fail: 11 unit, 2 binary, 1 real source
tests, workspace  25 test binaries, 657 pass, 0 fail, 4 ignored (1,131 s);
                  test_lang_items_in_sysroot skipped, so not a full PASS
build             cargo build --release --workspace --all-targets             exit 0
```

The full run, by acceptance criterion:

```text
A1  170 node rows and 682 edge rows stored; the info row holds both counts,       PASS
    the run generation and the sha256 of each input file
A2  every stored row equals its JSON row in all 11 columns; each payload text    PASS
    parses back to the exported payload; 681 edges join to a target node, 1 has
    no target and the reason TraitSolverNeeded
A3  two loads of the same export to two paths store equal rows in all three      PASS
    tables
A4  an input that breaks a check gives that check's error (7 unit tests) and     PASS
    leaves no file (test_failed_check_writes_nothing); in the binary, wrong
    flags exit 2 and an unreadable input exits 1, no file either way
A5  a second load onto the same path exits 1 and the file keeps its digest       PASS
A6  strace of one load: 0 opens under ~/.duckdb, 0 connect calls, 0 child        PASS
    processes
A7  both export files keep their sha256; neither is opened for writing           PASS
```

- **Measured:** mapper 22.0 s and 354.8 MB peak; loader 0.87 s and 87.9 MB; the DuckDB
  file is 7,876,608 bytes.
- **Who checked A1 and A2:** the test `test_itoa_export_dataset_roundtrip`, and a Python
  script outside the repository that reads the file with the PyPI `duckdb` package and
  shares no code with the loader. Both report 0 differences.
- **Not shown:** any repository other than itoa, any scope other than `declarations`, and
  an export larger than 682 edge rows.

## 7. Open questions

- **Columns for single fields of the lookup evidence** are not made. They stay in the
  payload text until a query needs one.
- **A CLI verb in `parseltongue`** is not added: it would change the frozen executable.
- **The frozen expected graph** is not an input. The input is a run's export only.
