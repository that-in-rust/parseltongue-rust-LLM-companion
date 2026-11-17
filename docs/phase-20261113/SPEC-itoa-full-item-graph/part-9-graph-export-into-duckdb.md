# Part 9 — the exported graph as a DuckDB file

Status on 2026-11-17: **specified, not built.** Decided by the user on 2026-11-17: a
separate crate reads the JSON a run exports, checks it, and writes it into one DuckDB file.
The mapper, the Turso store and the exported JSON are not changed.

## Requirement

REQ-RUST-067.0. Given the two files `export-published-stage-rows` writes for a published
run (nodes: `item-effective-visibility-records`; edges: `edge-evidence-record-table`), the
loader writes one DuckDB file whose rows equal the JSON rows, one for one, or it writes
nothing and returns a typed error.

## 1. Measurement (2026-11-17, the S2 export of the acceptance run, executable `af84bb46…`)

```text
file                    lines  first line  rows  bytes
full-graph-nodes.json   171    header      170   298,874
full-graph-edges.json   683    header      682   5,381,169
```

- **Header:** `export_owned_table_name`, `export_record_kind_value` = `header`, and
  `export_scope_identity_record` with `run_generation_digest_value`.
- **Row:** `export_record_kind_value` = `row`, `export_row_identity_value` (text) and
  `export_row_payload_value` (object).
- **Node identity:** the identity text of a node row equals the compact JSON, keys sorted,
  of its `node_target_identity`: 170 of 170.
- **Edge ends:** the same form of `edge_source_target_identity` is a node identity in 682
  of 682 rows, and of `edge_resolution.target` in 681 of 681. One row has no target
  (`kind` = `Unresolved`, a `reason`): the `cast`.
- **Identities are distinct:** 170 node identities, 682 edge identities.
- **DuckDB's `json` extension is not in the executable** (immediate backlog, entry 6), so
  the loader must not need it.

## 2. Design

- **A new crate, `parseltongue-graph-dataset-loader`:** a library and a binary of the same
  name. It depends on `duckdb` (the pinned version, bundled), `serde_json`, `sha2` and
  `thiserror`, and on no other crate of this workspace. The `parseltongue` executable does
  not depend on it, so its bytes do not change.
- **Pure core, effects at the edge.** `parse_graph_export_text` and
  `check_graph_export_rows` take text and return values; only `write_graph_dataset_file`
  touches DuckDB and the file system.
- **No JSON type in DuckDB.** A nested value is stored as compact JSON text, keys sorted.
  Each row also keeps its whole payload as text, so no field of the JSON is lost; the
  other columns are copies of payload fields for SQL.
- **All or nothing.** The file is written under a temporary name in the output directory
  and renamed after the last check. An output path that exists is an error.

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

## 3. Checks before anything is written

```text
C1  each file's first line is a header naming the expected table; every other line is a row
C2  both headers name the same run generation, and every row's run_generation_digest_full equals it
C3  a node's identity text equals the compact sorted JSON of its node_target_identity
C4  no two nodes and no two edges share an identity
C5  every edge's source is a node
C6  an edge has a target that is a node, or no target and a reason
C7  each file has at least one row
```

Each failed check is one variant of `GraphDatasetLoadError`, carrying the file, the line
number and the identity where they apply.

## 4. Acceptance criteria

```text
A1  WHEN the loader reads the export of a declarations run of pinned itoa
    THEN it writes one file holding 170 node rows and 682 edge rows
A2  WHEN the three tables are read back
    THEN each payload text parses to a value equal to the JSON row's payload, and each
    other column equals the payload field it copies, for every row
A3  WHEN the same export is loaded twice
    THEN `SELECT *` ordered by the key returns equal rows from both files
A4  WHEN an input breaks one of C1 to C7
    THEN the loader returns that check's error and no file exists at the output path
A5  WHEN the output path exists THEN the loader returns an error and leaves the file as it was
A6  WHEN the loader runs THEN it opens no file under `~/.duckdb` (strace, once)
```

## 5. Tests

- **Unit, one per check C1 to C7** (A4), on rows written in the test, each seen failing
  first.
- **Real source** (A1 to A3, A5): `test_itoa_export_dataset_roundtrip` in the loader
  crate. It runs the `parseltongue` executable on a copy of pinned itoa with scope
  `declarations`, exports the two tables, loads them, and reads the file back. The
  executable is `PARSELTONGUE_FROZEN_EXECUTABLE_PATH` when set, otherwise
  `target/release/parseltongue`; when neither exists the test fails and says so.

## 6. Steps

```text
9.1  crate, row types, parse and C1, C2, C7           unit tests
9.2  C3 to C6                                         unit tests
9.3  write and read back, all or nothing (A3 to A5)   unit tests on a small export
9.4  binary and the real-source test (A1, A2)
9.5  A6 once by strace; gates; log entry
```

## 7. Not in this part

- Columns for single fields of the lookup evidence. They stay inside the payload text.
- A CLI verb in `parseltongue`, and any change to the export format.
- Loading the frozen expected graph. The input is a run's export only.
