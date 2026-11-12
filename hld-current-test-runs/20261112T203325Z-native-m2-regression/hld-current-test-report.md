# Native M2 regression postmortem — 2026-11-12

**Scope:** bounded, diagnostic-only postmortem of the separately budgeted native sysroot regression. This is **not** an executed full SOP runner run, corpus qualification, or acceptance report. No source implementation, corpus, pin, or database changes were made. The reviewed oracle metadata correction `f1e61976064008f1b001862b3a92cfa23242d8f4` was not integrated.

## Gate status

| SOP step | Status | Evidence |
|---|---|---|
| S1–S2 | NOT STARTED | No corpus source qualification or coverage inventory was run in this step. |
| S3 | **BLOCKED / incomplete** | The isolated native M2 test hit the 40-minute ceiling and exited 124. No assertion failure or test result was observed. A later bounded read-only inspection of an external DuckDB copy found completed copy/normalization evidence, persisted unit-link/path/binding rows, and no fixed-point completion manifest; the exact last SQL operation remains unknown. |
| S4–S10 | **NOT STARTED** | No corpus runs, oracle export, production comparison, or qualification runner invocation occurred. |

All five corpus runs remain NOT STARTED. There is no all-five PASS, production qualification, or frozen release.

## Native test execution

Exact argv:

```text
cargo test -p parseltongue-cli-dispatcher-binary --test workspace-cli-contract-tests --all-features --release --locked --offline -- test_lang_items_in_sysroot --exact --nocapture
```

The external sampler declared **1,200 seconds expected / 2,400 seconds maximum**. Its summary records elapsed `2400.11` seconds, exit status `124`, `timed_out=true`, `suite_results=[]`, and `memory_cap=null`. Peak process-tree RSS was **8,786,596 KiB** across a peak of five processes, sampled by summing descendant `/proc` `VmRSS` every **0.2 seconds**. No RAM cap was set.

The warmed release target and relevant environment were:

```text
CARGO_TARGET_DIR=/home/ubuntu/repos/parseltongue-rust-LLM-companion/target
CARGO_BUILD_JOBS=2
CARGO_PROFILE_DEV_DEBUG=0
CXXFLAGS=-g0
RUST_LOG=info
PARSELTONGUE_NATIVE_FIXTURE_DIRECTORY=/home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/fixtures
CARGO_HOME=<unset>
RUSTUP_TOOLCHAIN=<unset>
RUSTUP_HOME=<unset>
RUSTFLAGS=<unset>
CARGO_ENCODED_RUSTFLAGS=<unset>
CARGO_INCREMENTAL=<unset>
CARGO_TERM_COLOR=<unset>
```

The exact argv and full allowlisted environment (including `PATH`) are in the external summary. Executed binary identities:

```text
test harness workspace_cli_contract_tests-f13681de9efffe60
  SHA-256 c6e8f11398b5820175b3d19e132844d17d07e0ba64cc3c83f1c44421156a8e52
CLI target/release/parseltongue
  SHA-256 b36a6d972d106ad42499fd9517adfd9bc7c186457752f34429f9a00954f5aa73
```

The captured log ends with:

```text
Finished `release` profile [optimized] target(s) in 13.36s
Running tests/workspace-cli-contract-tests.rs (.../workspace_cli_contract_tests-f13681de9efffe60)

running 1 test
native_fixture_directory=.../fixtures/.tmp3EVlWn
test test_lang_items_in_sysroot has been running for over 60 seconds
```

There is no `test result:` line or assertion output. This is a timeout, **not** an assertion failure and not a pass.

The earlier release workspace suite is a separate result:

```text
cargo test --workspace --all-targets --all-features --release --locked --offline -- --skip test_lang_items_in_sysroot
270 passed; 0 failed; 3 ignored; 1 filtered
```

Those 270 passing tests do not include `test_lang_items_in_sysroot`; that test was filtered in the earlier suite and timed out in this isolated attempt. The three pre-existing ignored manual profiles were inspected read-only and not run:

- `test_profile_json_bulk_batches` — `manual JSON bulk insert profile`
- `test_profile_bulk_batch_sizes` — `manual bounded bulk insert profile`
- `test_profile_bounded_unit_sizes` — `manual bounded transaction profile`

All three are in `crates/parseltongue-turso-store-adapter/src/stage-table-store-backend.rs` at lines 607–608, 644–645, and 693–694 respectively.

## Initial bounded read-only fixture inspection

This section preserves the first reader-unavailable attempt. Its conclusion that persisted stage counts were unknown was superseded by the external-reader follow-up at the end of this report.

Inspection was budgeted at **5 minutes expected / 10 minutes maximum**. Existing diagnostic-tool discovery found no DuckDB CLI/Python module and no `turso`, `sqld`, `libsql`, or `sqlite-utils` CLI. Python's standard-library SQLite reader was tried against `store.db` using URI `mode=ro` and `PRAGMA query_only=ON`; the catalog query returned:

```text
sqlite3.DatabaseError: file is not a database
```

The built-in `status-coverage-banner-show` command was not used: its dispatcher calls `initialize_workspace_stage_schema()` before building the banner (`crates/parseltongue-cli-dispatcher-binary/src/main.rs:819–821`), and the adapter's local builder exposes no read-only open mode (`crates/parseltongue-turso-store-adapter/src/lib.rs:36–43`). No DuckDB file was opened or recovered. No reader was installed or rebuilt, no forensic copy or third database was created, and no database recovery/checkpoint was requested.

The fixture filesystem contains one analysis-file path tuple:

```text
generation path component: 7f193f70ee51e76584825d94d13d150d94e9f02e4e2390349d4a9a3f21a03da3
attempt path component:    20149359d13566be0d659d643a00c833d499c657a9276b1b11e286781b30256c
file: .../namespace-resolution-analysis-data.duckdb
```

The source constructs that directory as `<run_generation_digest_value>/<stage_attempt_sequence_value>` (`namespace-pipeline-publication-facade.rs:247–259`). This is **path-level evidence for one analysis-file generation/attempt tuple only**; row-level persisted snapshots and attempts were not queryable.

Filesystem metadata (UTC; the before/after listings match in size and timestamps):

| File | Bytes | mtime UTC |
|---|---:|---|
| `store.db` | 1,824,514,048 | 2026-11-12 20:42:59.480236882 |
| `store.db-log` | 1,804,767 | 2026-11-12 21:11:47.352142572 |
| `store.db-wal` | 0 | 2026-11-12 20:42:59.494236816 |
| `namespace-resolution-analysis-data.duckdb` | 2,430,873,600 | 2026-11-12 21:14:41.110329154 |

No DuckDB WAL/sidecar file appeared in the enumerated fixture tree. SHA-256 values before and after the read-only inspection were identical:

```text
store.db          773b0323ae7fe70f59ee2e0de7c6f4dcad1f50ef828be27995eed5c07b3d9633
store.db-log      ae4868db2a80f2d116b63b5db10f67421a1bed4997319b44b7d14ea44fe6e0e5
store.db-wal      e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
namespace-resolution-analysis-data.duckdb
                  33980e5fe9852a6ec52a9ee499a119ff6175b718d210240f9ab6d1a7a0e0b4f7
```

Hash and timestamp comparisons passed; the original artifacts were unchanged by inspection.

## Initial stage conclusion before reader follow-up

The following conclusion was limited to the initial attempt before the external reader became available.

The file/path evidence and captured parent log do **not** identify the last completed namespace substage. In the source, `refresh_frozen_namespace_inputs` orders copy, normalization, and resolution (`namespace-pipeline-publication-facade.rs:216–244`); unit-linking emits started/complete events and checks normalized-source manifests (`namespace-frozen-resolution-rounds.rs:23–37`); path-candidate fixed-point rounds emit per-round events; and each successful macro expansion can invoke another complete frozen-input refresh (`namespace-pipeline-publication-facade.rs:272–297`). The test helper captures the CLI child's stdout/stderr using `Command::output()` and only includes stderr in the failure assertion (`workspace-cli-contract-tests.rs:793–798`). The outer log therefore does not expose those successful-path phase events.

Because neither the Turso store catalog nor the DuckDB analysis file could be queried safely with an existing compatible read-only reader, these counts are **unknown**:

- persisted publication snapshot generations/attempts beyond the one path tuple above;
- `namespace-normalized-source-manifests` rows;
- `namespace-fixed-point-manifests` rows;
- unit-link, path-candidate, and other relevant table row counts.

Available evidence does not distinguish whether execution stopped during copy, normalization, unit-linking, path-candidate fixed point, or a repeated macro refresh. No precise stalled SQL statement or root cause is inferred from the absence of telemetry.

## External evidence

- [External native-M2 evidence directory](file:///home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/)
- [Wrapper summary with exact argv/environment, exit, time, and sampled RSS](file:///home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/native-m2-summary.json)
- [Captured Cargo/test log](file:///home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/native-m2-sysroot-regression.log)
- [Executed binary identities](file:///home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/binary-identities-executed-test.txt)
- [Preserved fixture files and timestamps before inspection](file:///home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/forensic-files-before.txt)
- [Preserved fixture files and timestamps after inspection](file:///home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/forensic-files-after.txt)
- [Original DB hashes before inspection](file:///home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/original-db-sha256-before.txt)
- [Original DB hashes after inspection](file:///home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/original-db-sha256-after.txt)
- [Read-only reader inventory and prerequisite error](file:///home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/reader-prerequisite.txt)

**Disposition:** stop for review. No retry, source implementation, database recovery, commit, push, freeze, corpus run, or qualification was performed.

## Follow-up: bundled engine version prerequisite

Setup was budgeted at **5 minutes expected / 10 minutes maximum**. Before downloading a wheel or opening a database copy, I checked the adapter manifest, lockfile, and pinned `libduckdb-sys` source:

- `crates/parseltongue-duckdb-analysis-adapter/Cargo.toml:10` pins `duckdb = "=1.20000.1"` with the `bundled` feature. `Cargo.lock` pins both `duckdb` and `libduckdb-sys` at `1.20000.1`; lock checksums are respectively `d807ca47d4fa206f102ff568046e018fb84518eb0a0b5e65a165a2eb7bd4a7c2` and `0140448c472968ee5cf5ba697435deebe558b366ef1e5c080d24d9696b600725`. No lock was changed.
- `libduckdb-sys-1.20000.1/src/build_version.rs:1–11` decodes the second package-version component as DuckDB `major.minor.patch`; its tests at lines 28–31 explicitly assert that `1.20000.0` maps to `2.0.0`. Therefore pinned package version `1.20000.1` maps to engine `2.0.0`, not `1.2.0`.
- `libduckdb-sys-1.20000.1/build_bundled_cc.rs:63–68` unpacks the bundled `duckdb.tar.gz`; the bundled `duckdb/src/function/table/version/pragma_version.cpp` defaults `DUCKDB_VERSION` to `v2.0.0`. The archive SHA-256 is `14f16777c569dd0090db7c80dc7a1eba8ffbb36cf3f6e008dd9c002356162c29`.

This conflicted with the inferred Python wheel version `1.2.0`. No wheel was downloaded or installed in that step, and no database was opened then. A later handoff led to a session-built native read-only reader linked to the already-built 2.0.0 release libraries; that resolved the reader prerequisite without a Python wheel or C++ rebuild. The earlier source/lock conclusion remains authoritative: the bundled engine target is 2.0.0.

The requested reader/version decision was resolved by using the session-built executable against the bundled 2.0.0 engine. The measurements below supersede the earlier “counts unknown” conclusion; historical journal entries remain unchanged.

## Follow-up: external DuckDB forensic copy

This follow-up was bounded to read-only SQL on an external copy, within the declared **5 minutes expected / 10 minutes maximum**. No resolver/mapping, normalization/resolution replay, database recovery, extension install/load, source change, production access, or memory optimization was performed. The reader used read-only access with external access, extension autoload/autoinstall, and unsigned extensions disabled. Each catalog/count invocation had an explicit timeout (15, 25, 30, or 40 seconds).

### Reader identity and engine version

- Reader binary: /home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/duckdb-cache-forensic-reader
- Binary SHA-256: 385cf656a02cbd68434c67a76f15eaffdd73a3e67f403eb93435feebdda4db19
- Reader source: /home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/duckdb-cache-forensic-reader.rs
- Source SHA-256: e1fa096fc647163dacd4065cddbf5b786648250be22f236049cb6b781d582e30
- Reader CLI shape: executable DATABASE SELECT_QUERY. The version, catalog, and count commands used this reader with the forensic-copy path; the exact SELECT statements and returned values are preserved in the external query evidence files.
- Exact reader-build command, later supplied by the lead:
  ```text
  /usr/bin/time -f 'elapsed_seconds=%e exit_code=%x peak_rss_kib=%M' timeout 120s rustc --edition=2024 --crate-name duckdb_cache_forensic_reader /home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/duckdb-cache-forensic-reader.rs --extern duckdb=/home/ubuntu/repos/parseltongue-rust-LLM-companion/target/release/deps/libduckdb-6f870ecc95602d07.rlib -L dependency=/home/ubuntu/repos/parseltongue-rust-LLM-companion/target/release/deps -L native=/home/ubuntu/repos/parseltongue-rust-LLM-companion/target/release/build/libduckdb-sys-2da06d709f76f2a9/out -o /home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/duckdb-cache-forensic-reader
  ```
  Output: `elapsed_seconds=0.36 exit_code=0 peak_rss_kib=437232`. It linked the existing release rlibs; no C++ rebuild.
- The session-built reader's fresh file-backed scratch query returned v2.0.0. The read-only query against the forensic copy, SELECT version(), also returned v2.0.0. This agrees with the pinned libduckdb-sys source/lock, which maps package version 1.20000.1 to engine 2.0.0.
- The earlier binary-only PyPI request for duckdb==2.0.0 on Python 3.10.12 found no compatible wheel (available list ended at 1.6.0.dev214). No Python wheel was installed; the session-built reader resolved the prerequisite. The earlier package-index transcript is linked below.

### Original and copy integrity

Original DuckDB file: /home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/fixtures/.tmp3EVlWn/store.duckdb-analysis-run-files/7f193f70ee51e76584825d94d13d150d94e9f02e4e2390349d4a9a3f21a03da3/20149359d13566be0d659d643a00c833d499c657a9276b1b11e286781b30256c/namespace-resolution-analysis-data.duckdb

Forensic copy: /home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/duckdb-forensic-copy/namespace-resolution-analysis-data.duckdb

Both files were 2,430,873,600 bytes with mtime 2026-11-12 21:14:41.110329154 UTC. The original SHA-256 before and after inspection was 33980e5fe9852a6ec52a9ee499a119ff6175b718d210240f9ab6d1a7a0e0b4f7; the copy SHA-256 before and after opening read-only was the same. The post-query hash comparison reported ORIGINAL_COPY_HASH_MATCH=PASS. No DuckDB .wal sidecar existed beside the original or copy; the zero-byte store.db-wal is a sidecar for the separate Turso store.db and was not treated as a DuckDB WAL or copied.

### Persistent catalog and row counts

The copy contains 29 persistent main-schema tables. The bounded table-count query returned:

    frozen-generation-source-records=221822
    frozen-generation-table-manifests=8
    namespace-binding-declaration-records=38555
    namespace-binding-round-results=281120
    namespace-binding-visibility-records=0
    namespace-declaration-visibility-records=38555
    namespace-extern-link-records=5
    namespace-extern-source-records=5
    namespace-fixed-point-manifests=0
    namespace-import-request-records=9192
    namespace-import-visibility-records=9192
    namespace-legacy-scope-records=18413
    namespace-lexical-parent-records=204444
    namespace-macro-prelude-records=3
    namespace-macro-source-records=863
    namespace-module-parent-records=707
    namespace-normalized-source-manifests=1
    namespace-path-candidate-records=71941
    namespace-path-metadata-records=92502
    namespace-path-segment-records=110030
    namespace-path-source-requests=97716
    namespace-scope-ancestor-records=348274
    namespace-scope-context-records=61967
    namespace-scope-module-records=61959
    namespace-target-context-records=38061
    namespace-target-source-records=38882
    namespace-unit-dependency-records=6
    namespace-unit-link-records=4
    namespace-unit-source-records=4

The eight frozen table manifests are scoped to generation key 7f193f70ee51e76584825d94d13d150d94e9f02e4e2390349d4a9a3f21a03da3 and attempt key 20149359d13566be0d659d643a00c833d499c657a9276b1b11e286781b30256c/frozen/8. Their row counts sum to 221,822: crate-dependency-graph-table=5; legacy-macro-scope-records=18,413; syntax-namespace-input-externs=5; syntax-namespace-input-imports=9,189; syntax-namespace-input-items=38,878; syntax-namespace-input-macros=863; syntax-namespace-input-paths=92,502; syntax-namespace-input-scopes=61,967. The frozen source-record count and normalized manifest source_rows are also 221,822. There is one normalized-source manifest and no fixed-point manifest.

State summaries for the same resolution tables:

- Binding results: 279,860 resolved rows across 12,203 scopes; 1,260 ambiguous rows across 579 scopes (281,120 total).
- Path candidates: 71,937 resolved rows across 68,688 request keys; 4 ambiguous rows across 2 request keys (71,941 total).
- Unit linking: 4 unit-link rows, 6 dependency rows, and 5 extern-link rows; 3 generated default-prelude requests were present.

### Source-control-flow interpretation and limits

The persisted copy manifests and 221,822 source rows establish that snapshot copying produced the recorded source set. In frozen-namespace-source-codec.rs:131–145, normalization inserts the normalized-source manifest in the same transaction as the normalized rows, commits, then checkpoints; the one manifest with source_rows=221,822 is evidence that normalization completed for this scope.

namespace-unit-link-inputs.rs:31–37, 183–195 writes and commits the unit-link/dependency/extern rows, then calls prepare_default_prelude_inputs. namespace-default-prelude-inputs.rs:20–37 commits the generated default-prelude rows. The four unit links and three generated prelude requests are durable outputs from those paths. The original child phase-event stream was not captured, so no successful-path event is claimed.

namespace-path-candidate-records.rs:46–57 replaces candidates in a transaction and commits. The persisted candidate rows and binding results establish that resolution work ran for this snapshot. namespace-frozen-resolution-rounds.rs:32–45 inserts namespace-fixed-point-manifests only after resolve_namespace_sql_rounds returns successfully. No such manifest exists for the persisted scope. This means fixed-point completion is not durably evidenced; the database cannot distinguish an interruption within the path-candidate/import-binding rounds from a failure after the rounds returned but before the manifest insert. No precise stalled SQL/query is inferred.

The attempt key ends in frozen/8. frozen-stage-snapshot-rules.rs:61–74 derives that suffix from the publication revision counter; namespace-pipeline-publication-facade.rs:272–297 calls the refresh initially and after successful macro expansions. namespace-working-cache-reset.rs:5–37 drops prior namespace/frozen-generation tables and checkpoints before each refresh, so this DuckDB file retains the latest refresh, not the full sequence. The revision establishes repeated snapshot publication, but the prior snapshots and their per-pass manifests are not retained here; this copy alone does not prove the exact macro expansion count or earlier pass outcomes.

Narrowest supported conclusion: copy and normalization are complete for frozen revision 8; unit-link outputs and resolution rows are persisted; fixed-point completion is unproven because its manifest is absent. The exact last SQL operation, whether resolution returned before manifest insertion, and original-run DuckDB memory/settings/phase telemetry remain unknown. These are forensic SELECT findings, not original-run DuckDB telemetry.

### External evidence

- External reader identity/build notes: /home/ubuntu/hld-current-test-artifacts/20261112T200343Z-descendant-integrated-gates/20261112T203325Z-native-m2-regression/duckdb-forensic-reader-identity.txt
- Reader version, catalog, manifest schemas, counts, scope, state, and unit-link outputs are recorded in duckdb-forensic-version-query.txt, duckdb-forensic-catalog-query.txt, duckdb-forensic-manifest-schema-query.txt, duckdb-forensic-table-counts.txt, duckdb-forensic-manifest-details.txt, duckdb-forensic-generation-attempts.txt, duckdb-forensic-resolution-states.txt, and duckdb-forensic-unit-link-rows.txt in that external directory.
- Original/copy before-and-after identity records: duckdb-original-before.txt, duckdb-original-after.txt, duckdb-forensic-copy-before-open.txt, and duckdb-forensic-copy-after.txt.
- Reader source/binary, database copy, and query logs remain outside the repository in the external run directory. No additional database was created for forensic inspection; the separately supplied version-only scratch check was not part of these catalog queries.

S3 remains BLOCKED/incomplete from the native-test timeout, not an assertion failure. The earlier release workspace result remains separate: 270 passed, 0 failed, 3 ignored, 1 filtered; the filtered native test is not among the 270. The three ignored manual profiles remain unrun. All corpus runs and S4–S10 remain NOT STARTED. This is diagnostic postmortem evidence, not a full SOP run or acceptance result. No retry, commit, push, freeze, or qualification occurred.
