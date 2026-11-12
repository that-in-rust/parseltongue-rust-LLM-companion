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
