# Native visibility negative knowledge

## Correlated two-owner marker lookups

- Failure: a native sysroot CLI child remained CPU-bound after 31 minutes.
- Reachable path: saved `/home/ubuntu/cli-native-orphan-stacks.log` reaches
  `count_stage_table_rows` through the frozen-generation Appender handoff.
- Cause shown by the pinned Turso plan: correlated two-owner IN predicates
  use only generation/attempt, scanning candidate markers for each row.
  The exact proportion of the native run's wall time is not established.
- Detection: `test_frozen_visibility_marker_probes` and
  `test_namespace_completion_marker_probes` require the final marker-key
  column in every applicable EXPLAIN QUERY PLAN lookup.
- Safer replacement: separate EXISTS subqueries, each with a complete
  primary-key equality; retain owner alternatives and frozen revision bounds.
- Proof gate: plan assertions plus all Turso atomic-visibility contracts;
  the full bounded CLI/native check remains a separate gate.

## Namespace work-unit index-name collisions

- Failure: namespace item cleanup scans its generation instead of one chunk.
- Cause: six table names generated `syntax-namespace-input-lookup`;
  CREATE INDEX IF NOT EXISTS silently created it only on scopes.
- Detection: `test_namespace_cleanup_index_coverage` checks the DELETE plan
  for every namespace-owned table, including the work-unit index column.
- Safer replacement: distinct four-word `syntax-namespace-<kind>-lookup`
  index names. Existing databases gain the missing indexes on initialization.
- Proof gate: all per-table plans and cancellation/rerun/publication tests.

## Debug-build CPU evidence is not a production memory result

- Failure signal: the bounded native run still spends substantial time in
  pre-analysis syntax writes; this must not be labeled a DuckDB SQL hang.
- Evidence: `/home/ubuntu/cli-native-write-cpu.data` contains 990 task-clock
  samples; the actual write stack calls Turso B-tree cell validation.
  The pinned `turso_core` 0.9.0-pre.5 `storage/btree.rs` gates that validation
  on `debug_assertions`; normal dev builds enable it despite DEBUG=0.
- Safer comparison: isolate the upstream dependency's debug assertions in a
  command-line-only Cargo profile override. Do not edit pinned source or
  workspace profiles, and do not treat an override run as a default-profile
  pass. Final SOP measurements must use the frozen release executable.

## check-cache-rebuild-signal

- Failure signal: a bounded release `cargo check` against the companion's
  shared target reported `Compiling libduckdb-sys v1.20000.1` and reached its
  240-second timeout before adapter checking completed.
- Evidence: `/home/ubuntu/hld-current-test-artifacts/20261113T013827Z-namespace-import-statement-trace/stub-check.log`;
  `libduckdb.a` retained hash
  `a753cf5c643d8bfe15e58fbf6f7e53c280ddf52b4bf811a791600ea7f71272ba` and
  mtime `2026-11-12 14:47:58.568027830 +0000`; no related process remained
  after timeout. These observations do not establish whether a C++ compile
  started before the timeout.
- Safer response: stop when the DuckDB system crate begins recompiling and
  do not retry until the exact warmed check artifact/cache path is verified
  or the lead authorizes a different bounded check.
- Proof gate: a later adapter gate completes within its declared ceiling,
  reports no C++ rebuild, and confirms the same archive hash and mtime.

## REQ-RUST-028.2 — Per-statement DuckDB error context

- Failure: full DuckDB error display text is not byte-identical between the
  prior whole-script `execute_batch` call and a call on the failing statement
  slice. Both report the same missing-table catalog error, but the saved
  whole-script context points to `LINE 9` and the single-slice context to
  `LINE 5`.
- Evidence: the focused failure test also executes the failing production
  statement slice directly after the same first three statements; that raw
  DuckDB error matches the traced failure text exactly. The leading catalog
  diagnostic matches the whole-script failure. Query-relative line context
  is the observed difference; the DuckDB internals causing it were not
  inspected.
- Safer comparison: compare the primary DuckDB diagnostic and autocommit /
  explicit-rollback output state between old and traced paths; additionally
  require the trace failure text to equal the raw error from that same
  `execute_batch` statement slice. Do not demand byte-identical query line
  excerpts across different SQL input strings.
- Proof gate: `test_import_batch_failure_equivalence` on the pinned DuckDB
  dependency checks the primary error, direct-slice error, statement stop
  ordinal, output state and explicit transaction rollback.
