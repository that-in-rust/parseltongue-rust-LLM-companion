# HLD current test report — native CLI blocker

## Verdict and scope

Date: 2026-11-12. Overall: **BLOCKED** at task 1 / the S3 prerequisite fixture
suite. This is a persistent failed-precondition report, **not** an all-five
S1/S2/S3 qualification or an M2 completion claim. The three-attempt stop rule
was reached; no fourth native attempt or RAM tuning was performed.

First failed clause: S3 requires the applicable SPEC fixture/build suite to
pass before corpus mapping. `test_lang_items_in_sysroot` fails in the release
executable with a DuckDB out-of-memory error at the retained ~1.2 GiB cap.
The deferred final 500 MB whole-tree target is NOT the cause of this stop.

## Executable and source identity

```text
branch: research/tiny-rust-code-mapper
candidate source base HEAD: 73a3c66c7
candidate executable: /home/ubuntu/hld-current-test-artifacts/20261112-native-cli-blocked/parseltongue
executable SHA-256: bfd1fc7f9d7f503dffcb2fd43730928ab619ed01b416dc9af4ac08c5cde3173d
executable BLAKE3: 06d8b8b90ff8e2afa53d28bbdcf1e33465a40e18ccc7c77ac446a07a7da432dc
executable size: 100310904 bytes
```

This is a preserved **candidate**, not a qualified frozen all-five release.
The original CLI wiring remains uncommitted in
`crates/parseltongue-cli-dispatcher-binary/src/main.rs` and
`crates/parseltongue-cli-dispatcher-binary/src/namespace-pipeline-publication-facade.rs`.
Its exact patch is archived as `namespace-publication-worktree-changes.patch`:
BLAKE3 `183e5b41a8afc46e71379a373799c675cb88e4f81a321528c9809742b39c6ba2`.
No CLI GREEN/REFACTOR completion is claimed.

Parser AND oracle fork commit:
`bf6b2099d6351c37039b01447a9bb64f9a54a586`.
Patch: `tools/parser-const-closure-statements.patch`.
Patch SHA-256:
`89146042a1c1629ef67c730c0e180dec15264577f675de1c090dca2d87da6bfa`.
The parser checkout HEAD and patch digest were rechecked during this attempt.

Rustc/Cargo: stable 1.99.1. The pinned upstream parser/syntax manifests emit
the stable-Cargo `unused manifest key lints.cargo` warning. The environment
was checked; no upstream manifest, toolchain or workspace profile was changed.

## Bounded native attempts

### Attempt 1 — default dev-profile contract suite

```text
CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CXXFLAGS=-g0 \
PARSELTONGUE_PROFILE_STAGE_WRITES=1 \
timeout --signal=TERM --kill-after=10s 600s \
cargo test -p parseltongue-cli-dispatcher-binary \
  --test workspace-cli-contract-tests -- --test-threads=1 --nocapture

exit: 124
last test: test_lang_items_in_sysroot
preceding tests: 9 passed
```

This candidate had the two independent marker probes, before the additional
namespace cleanup-index collision fix. No complete suite PASS resulted.

### Attempt 2 — isolate upstream dev-profile CPU overhead

The requested build environment remained unchanged. A command-line-only
Cargo override disabled `turso_core` debug assertions and set that package's
optimization level to 1; no config/source edits were made for the override.

```text
cargo --config 'profile.dev.package.turso_core.debug-assertions=false' \
  --config 'profile.dev.package.turso_core.opt-level=1' build \
  -p parseltongue-cli-dispatcher-binary
Finished `dev` profile [unoptimized] target(s) in 1m 28s

timeout --signal=TERM --kill-after=10s 600s \
cargo --config 'profile.dev.package.turso_core.debug-assertions=false' \
  --config 'profile.dev.package.turso_core.opt-level=1' test \
  -p parseltongue-cli-dispatcher-binary --test workspace-cli-contract-tests \
  test_lang_items_in_sysroot -- --exact --nocapture

exit: 124
test test_lang_items_in_sysroot has been running for over 60 seconds
```

This included the distinct namespace work-unit indexes. It is a diagnostic
variant, not a replacement default-profile pass. An observed parent RSS of
818324 KiB is only an observation, not a sampled whole-process-tree peak.

### Attempt 3 — release isolation

Both commands used `CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CXXFLAGS=-g0`.

```text
cargo build --release -p parseltongue-cli-dispatcher-binary
Finished `release` profile [optimized] target(s) in 23m 31s

timeout --signal=TERM --kill-after=10s 600s \
cargo test --release -p parseltongue-cli-dispatcher-binary \
  --test workspace-cli-contract-tests -- --test-threads=1 --nocapture

stage=syntax-batches mode=Extraction units=618 records=358595 load_seconds=0.142 parse_seconds=3.644 write_seconds=41.973
stage=syntax-spans elapsed_seconds=45.858
Error: StoreBackendFailureError { error_context_detail_text: "Out of Memory Error: could not allocate block of size 256.0 KiB (1.1 GiB/1.1 GiB used)\n\nPossible solutions:\n* Reducing the number of threads (SET threads=X)\n* Disabling insertion-order preservation (SET preserve_insertion_order=false)\n* Increasing the memory limit (SET memory_limit='...GB')\n\nSee also https://duckdb.org/docs/current/guides/performance/how_to_tune_workloads" }

test result: FAILED. 20 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 279.08s
exit: 101
```

279.08s is the suite wall time, NOT a production corpus mapping time.
45.858s is the native fixture's reported syntax-stage time. Whole-tree peak
RSS was not measured by the process auditor in this prerequisite test.
Resolution rounds are unavailable: there is no completed
`stage=namespace-sql-fixed-point` banner. Do not report zero rounds.

## Reachable causes and remaining uncertainty

The old orphan mapper was stopped. Its actual saved GDB stack reached
`count_stage_table_rows -> copy_generation -> copy_published_stage_rows ->
refresh_frozen_inputs -> run_namespace_analysis_pipeline`.
Pinned Turso EXPLAIN QUERY PLAN showed the correlated two-owner IN predicates
using only generation/attempt, not the final marker key. Separate EXISTS
probes restore complete-key lookups without changing visibility semantics.

A second proven defect: six namespace input table names produced one index
name. IF NOT EXISTS left five tables without work-unit indexes. Per-table
DELETE plans exposed this; each input kind now has a distinct four-word
lookup-index name. The earlier hypothesis rejection based only on seeing
CREATE INDEX in the initializer was insufficient and was corrected.

The debug write CPU profile has 990 samples, no lost samples, and 31.82%
inclusive `debug_validate_cells_core`. Its `cfg(debug_assertions)` gate was
checked in pinned `turso_core` 0.9.0-pre.5. The release run disproves debug
overhead as the only blocker: it fails at the enforced DuckDB cap.

The exact DuckDB operation that exhausts the cap is **unverified**. The
facade performs copy, normalization and SQL resolution before printing its
completion banner and currently erases that substage context from errors.
No allocation-contributor or Cartesian-join claim is made. The failed test's
temporary store was removed by its fixture cleanup; logs and executable are
preserved, not an immutable native source snapshot.

Recommended single next action: cut the next step into a retained native
input snapshot plus independently timed/error-tagged copy, normalization and
resolution probes at the SAME cap. Identify the failing substage before any
memory change. Raising the cap or further full-run tuning was not authorized
or attempted after the third failure.

## Focused GREEN checks — progress, not milestone completion

```text
timeout --signal=TERM --kill-after=5s 180s \
cargo test -p parseltongue-turso-store-adapter -- --nocapture

test result: ok. 5 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 0.72s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.18s
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.38s

cargo clippy -p parseltongue-turso-store-adapter \
  -p parseltongue-cli-dispatcher-binary --all-targets -- -D warnings
Finished `dev` profile [unoptimized] target(s) in 1.92s

cargo fmt --all -- --check
FORMAT_CHECK_EXIT=0
```

These used the requested build environment. Three ignored cases are manual
store profiling fixtures, not the new regressions. New REQ-tagged regressions:
`test_frozen_visibility_marker_probes`,
`test_namespace_completion_marker_probes`,
`test_namespace_cleanup_index_coverage`.

Commits: `ae6c28e06` visibility RED; `3d6563b1f` index coverage RED;
`c315d3421` focused GREEN; `8cb50f6f2` explicit no-further-refactor checkpoint;
`73a3c66c7` bounded native attempt checkpoint. All pushed.

## All-five SOP status

The canonical pin manifest was checked: all five checkout HEADs match and
tracked sources are clean. This is a preflight only; no fresh S1 index, S2
coverage pagination or corpus S3 execution was performed. The required order
was retained: do not start serde or the other four before fixing the CLI
prerequisite. No historical M1 evidence is carried forward.

```text
serde          S1 not-started; S2 not-started; S3 BLOCKED: native CLI fixture fails
rust-analyzer  S1 not-started; S2 not-started; S3 BLOCKED: native CLI fixture fails
ripgrep        S1 not-started; S2 not-started; S3 BLOCKED: native CLI fixture fails
tokio          S1 not-started; S2 not-started; S3 BLOCKED: native CLI fixture fails
cargo-modules  S1 not-started; S2 not-started; S3 BLOCKED: native CLI fixture fails
```

Per-repository peaks, wall times and rounds: **not measured**.
Process/network/extension/target audits: **not-started**.
Serde M2 qualification, exact oracle manifests, SQL comparator, all-five S5
and S6/S7: **not-started in this run**. The R7 traceability/typed-error follow-up
is also not completed; these passing focused tests do not establish R7 PASS.

## Evidence retention

Large evidence directory:
`/home/ubuntu/hld-current-test-artifacts/20261112-native-cli-blocked/`.
Every archived artifact has path, byte size, BLAKE3 and SHA-256 in the adjacent
committed `artifact-content-digest-manifest.json`. It includes the full release
failure log, both bounded dev attempts, CPU trace/report, actual orphan
stack, RED/ GREEN logs, executable and original CLI patch.

The three temporarily paused VS Code rust-analyzer processes (1532439,
1782164, 1797409) were resumed with SIGCONT and verified `Sl` after builds.
No mapper remains running. No visual surface changed.

## Build identity and report integrity

Full candidate source base:
`73a3c66c7cc422f96e252c742b9c062c458eec2a` plus the archived CLI patch above.
Adjacent `candidate-source-build-identity.json` records the full compiler
version, platform, Cargo/lock/pin-manifest hashes and configured engine cap.
It distinguishes the configured cap from an unobserved DuckDB current_setting.

Evidence integrity check output:
```text
ARTIFACT_MANIFEST_VERIFIED=15
REPORT_BLOCKED_STATUS_VERIFIED=5
```
All archived sizes and both hashes were recomputed successfully. This is a
report-integrity check, not a production or SOP qualification PASS.
