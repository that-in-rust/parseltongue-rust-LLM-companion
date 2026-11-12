# hld-current-test-report — namespace phase isolation

2026-11-12. **BLOCKED: resolution INSERT exceeds the unchanged DuckDB cap.**
This is a diagnostic prerequisite report, not an all-five SOP qualification.
Instrumentation builds changed between probes; no one-frozen-executable
S1/S2/S3 or S5 PASS is claimed. Root-cause correction has not reached GREEN.

## Retained input and policy

Artifacts: `/home/ubuntu/hld-current-test-artifacts/20261112-namespace-phase-probes/`.
Frozen Turso input: `.tmpHNrSfz/store.db`, 1,784,647,680 bytes.

```text
generation a0e2244ff9e661f5f5a8e8bfe4f378ebb72a0a5817164e0777170f78c5c1524a
attempt    61756ec4f80940e2fa3759996677c26d1b6d2396f201473a296a319c42fa18d1
input SHA256 e5e03f0e13e0aeb3a2abe17be43a941b982d4cb120432da8704959d16448b224
input BLAKE3 13f8c30699766161853f173906c7c0e3f80c6caea01dbb9f56ea299e228b31c8
DuckDB cap 1,288,490,188 bytes — NOT raised
```

The constructor still disables extension installation/loading and external
access and uses run-local spill. Final 500 MB tree-RSS acceptance is deferred
per the correctness-first revision; these are measurements, not memory PASSes.
Parser/oracle pin remains `bf6b2099d6351c37039b01447a9bb64f9a54a586`, patch
SHA256 `89146042a1c1629ef67c730c0e180dec15264577f675de1c090dca2d87da6bfa`.

## Separate processes, same frozen input

The Appender copied and verified 281,680 rows. Separate replay used
`separate-namespace-analysis-state.duckdb`. Tree sampling was the existing
auditor's 10 ms sampler; resolution/profiling trees include a timeout supervisor.

```text
copy:
  PASS; 281680 rows; phase 35.398s; process wall 35.548s
  sampled tree peak 1,323,495,424 bytes
  post-phase duckdb_memory 628,883,456 bytes; temporary 0
  pragma database_size 633.5 MiB; memory_usage 599.7 MiB; limit 1.1 GiB
normalization:
  PASS; 281680 rows; phase 31.469s; process wall 31.652s
  sampled tree peak 1,936,781,312 bytes
  post-phase duckdb_memory 371,326,720 bytes; temporary 12,910,592
  pragma database_size 899.7 MiB; memory_usage 354.1 MiB; limit 1.1 GiB
resolution, first replay:
  FAIL in round 2; phase 32.273s; process wall 32.538s
  sampled tree peak 1,482,792,960 bytes
  post-rollback duckdb_memory 98,104,432 bytes; temporary 443,777,024
  pragma database_size 1.5 GiB; memory_usage 93.5 MiB; limit 1.1 GiB
resolution, memory-instrumented replay:
  FAIL in round 3; phase 72.936s; process wall 73.292s
  sampled tree peak 1,750,413,312 bytes
  post-rollback duckdb_memory 23,457,568 bytes; temporary 913,670,144
  pragma database_size 1.9 GiB; memory_usage 22.3 MiB; limit 1.1 GiB
```

DuckDB before/after samples are **not peaks**. Tree peaks are sampled, not
OS high-water guarantees. Replays share immutable source rows but the second
reuses partially populated derived tables; they are not identical cold-state
timing trials. No fixed point or final completed round count was obtained.

Initial fixture capture failed differently: resolution exhausted disk after
64.816s. With only 2.4 GiB free, retaining input and spill filled the volume.
Preserved the old debug CLI, then removed only 25 GiB reproducible `target/debug`
cache. After 27 GiB became free, the isolated replay reproduced the original
DuckDB OOM. ENOSPC is not mislabeled as the original memory cause.

## Identified operation and reachable chain

`namespace-frozen-resolution-rounds.rs::resolve_namespace_sql_rounds` calls
`recompute_path_candidates` each round after unit links/scope relations.
`namespace-path-candidate-records.rs:46–56` deletes old scoped candidates,
then inserts the recursive SELECT from `namespace-path-candidate-derivation.sql`.
Runtime progress shows links, cycle validation, scope relations, and preceding
round(s) complete; the error follows the immediate pre-INSERT memory line.

[The failing INSERT](failing-namespace-insert-query.sql) is independently
EXPLAIN ANALYZEd in a transaction with the same scoped DELETE, and rollback
on success/error. The two session temp inputs are equivalent scoped CTEs.
It independently fails after 4.884s, tree wall 4.963s, sampled tree peak
1,558,245,376 bytes:

```text
Error: Resolution { source: StoreBackendFailureError { error_context_detail_text:
"Out of Memory Error: failed to pin block of size 256.0 KiB (1.1 GiB/1.1 GiB used) ..." } }
```

The failing operation is established. A wrong join or a specific allocation
as the root cause is **not** established. No speculative index removal or cap
increase was made. Stop after the retained capture plus two resolution
replays; do not launch another full native attempt without a narrower change.

## EXPLAIN ANALYZE operator counts

The full INSERT aborts and returns no runtime operator profile. Its SELECT
body completes after reopening the file: 3.676s probe time, sampled tree peak
1,402,781,696 bytes. JSON profile reports engine time 2.909580285s,
peak_buffer_memory=1,289,658,512 bytes, peak_temp_dir_size=603,357,184 bytes,
and total_memory_allocated=4,980,211,712 bytes (cumulative, **not** peak).
The reported buffer peak slightly exceeds the configured byte value; it is
reported as emitted, not represented as a larger configured cap.

[All 133 operators](failing-query-operator-cardinalities.csv) include stable
tree path, output rows, intermediate bytes, time, and join/scan details.
These are SELECT-body counts on the post-first-replay state, **not** full
INSERT/index costs or a profile of the later post-round-2 state.

```text
lexical-parent TABLE_SCAN        204440 rows   300143456 intermediate bytes
lexical seed HASH_JOIN           221297 rows   148172004 intermediate bytes
path-source TABLE_SCAN           97705 rows   114354008 intermediate bytes
scope-module TABLE_SCAN          61955 rows    82643648 intermediate bytes
recursive walk CTE              493552 rows    51550336 intermediate bytes
binding-round TABLE_SCAN         55292 rows    50058240 intermediate bytes
name-step minimum-rank WINDOW    77759 rows     8423572 intermediate bytes
final ambiguity WINDOW           62554 rows     7506480 intermediate bytes
final projection                 62554 rows     8006912 intermediate bytes
```

Intermediate bytes are per-operator output volume, not resident allocation
stacks. No scopes×prelude-names cartesian appears in this profiled body.
Before insertion, ART_INDEX was 87,293,952 bytes in rounds 1–2 and 114,819,072
in round 3: this disproves an assertion that an almost-cap-sized index alone
causes the failure. SELECT success does not rule out INSERT/index or live
iteration residency as contributors. Their relative contribution is unresolved.

Recommended next cut: deterministic batches of complete request_key groups,
since walk/rank/ambiguity/dedup are request-local, plus visibility-ancestor
filtering to active request scopes. Require a RED full-vs-batched equality
fixture and native success at this cap before calling this a root-cause fix.

## Verification

RED phase tests failed at the diagnostic STUB. After implementation:

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 0.04s
```

Additional insertion rollback RED failed at its STUB. Final adapter gate:

```text
test namespace_analysis_phase_diagnostics::tests::test_namespace_insert_plan_rollback ... ok
test namespace_analysis_phase_diagnostics::tests::test_namespace_query_explain_rows ... ok
test namespace_analysis_phase_diagnostics::tests::test_namespace_query_explain_error ... ok
test namespace_analysis_phase_diagnostics::tests::test_namespace_phase_failure_context ... ok
test namespace_analysis_phase_diagnostics::tests::test_namespace_phase_empty_success ... ok
test namespace_analysis_phase_diagnostics::tests::test_namespace_telemetry_failure_preserves ... ok
test result: FAILED. 54 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.15s
```

Only failure: existing `test_turso_published_appender_handoff`, unchanged from
HEAD, submits output rows under `__complete__`; unchanged backend rejects them:
`Turso store: stage completion markers cannot own output rows`.
Both sides were checked in HEAD, not merely assumed unrelated. It is not fixed
by weakening the production guard or changing test assertions.

Clippy `--release -p parseltongue-cli-dispatcher-binary -p
parseltongue-duckdb-analysis-adapter --all-targets -- -D warnings`, release
binary/example build, fmt and diff checks completed. Full native fixture and
full adapter gate remain RED; no CLI GREEN is claimed.

## Corpus SOP state — first failed clause

```text
serde          S1 NOT-STARTED; S2 NOT-STARTED; S3 BLOCKED; S5 NOT-RUN
rust-analyzer  S1 NOT-STARTED; S2 NOT-STARTED; S3 BLOCKED; S5 NOT-RUN
ripgrep        S1 NOT-STARTED; S2 NOT-STARTED; S3 BLOCKED; S5 NOT-RUN
tokio          S1 NOT-STARTED; S2 NOT-STARTED; S3 BLOCKED; S5 NOT-RUN
cargo-modules  S1 NOT-STARTED; S2 NOT-STARTED; S3 BLOCKED; S5 NOT-RUN
```

First failed shared S3 prerequisite: native full-M2 CLI fixture cannot complete
resolution under the cap. No corpus peak/time/rounds or S5 agreement invented.
Original CLI publication WIP plus typed call-site wiring remains uncommitted in
`main.rs` and `namespace-pipeline-publication-facade.rs`, preserved as
`preserved-cli-publication-wiring.patch` in artifacts and a retained git stash.
Serde M2, comparator, all-five runs, and final memory qualification remain open.

## Executable identities and final check output

Diagnostic implementation commits: `1db884db2` and `4363fca99` plus the
preserved, uncommitted two-file CLI wiring patch. The CLI is not a qualified
release candidate. Its latest SHA256 is
`f2e4212eb6833b4814a5d48aa0eaf14a5311d88c0557f80ef0a4ae2d04dfea7f`,
BLAKE3 `7e25c6951f540fe90d8966263ed304476afdb6dcf756b698dc04708e8f4fd478`.
The measured INSERT-probe executable SHA256 was
`f1210f0c42152eba3d65c44221dccdb0e5232f474ef1ce609809bee433ea2ead`,
BLAKE3 `46c1e645bffd624bf63925d7ef41f738b7934dd2ba012ce2fd5b96b3255632aa`.
It was subsequently rebuilt only to list `insert` in its usage error; both
identity records are retained, rather than calling them byte-identical.

```text
release build: Finished `release` profile [optimized] target(s) in 1.04s
Clippy:        Finished `release` profile [optimized] target(s) in 0.30s
```

Formatting/diff checks exited zero. The editor rust-analyzer PID 1804565 was
resumed with SIGCONT and verified Sl after native work; the old requested PID
1532439 no longer exists. No mapper/profile process is left running.
