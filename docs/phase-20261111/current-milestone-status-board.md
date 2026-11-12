# current-milestone-status-board (phase-20261111)

Rewritten in place after every run report. History lives in the TDD journal and run reports.

Last updated: 2026-11-12 · Latest runtime measurement is the isolated native M2 retry from clean integrated HEAD `825d760e8685d8d4d95ab8acd2267edf3687847b`: the supervisor enforced 2,400s, total command elapsed 2,410.252s including bounded descendant cleanup, exit 124, peak whole-tree RSS 9,234,448 KiB sampled every 0.2s; no assertion/result line. Live stderr reached completed copy/normalization and resolution rounds; its last marker was round 3 import-bindings started, with exact active query unknown. Production source remains clean and unqualified. No all-five PASS, production qualification, or frozen release exists.

```text
 MILESTONE  STATE
 M0  core types, Turso store            code done; SOP S3 skeleton subset not re-run on current digest
 M1  ingest, crate graph, spans         code done; S1/S2/S3 historical PASS (old digest) → re-run pending
 M2  name resolution (DuckDB SQL)       mapper plus runner/comparator descendants integrated but unqualified; native M2 retry BLOCKED/incomplete at 2,400s; live stderr logged a 14-round fixed-point line and later copy/normalization plus repeated resolution progress, ending at round 3 import-bindings started; exact active query unknown. Earlier forensic copy shows frozen revision 8, normalized source 221,822 rows, persisted unit-link/path/binding outputs, and no fixed-point manifest; all-five S5 not started
       code   measured-only default, generation-backed publication receipts, and stage-status banner integrated
       runner component code/tests integrated; full S3 intake is unimplemented; S4/S5 execution is not started
       comparator R9 file-backed follow-up integrated; no production comparison or final freeze
       done   non-monotone SQL resolution, preludes, extern aliases, textual macro scope, macro alternation
       done   oracle path partition (module-scope scored at M2; type-relative deferred)
       done   indexed frozen visibility + distinct namespace cleanup indexes (focused tests GREEN)
       done   typed copy/normalization/resolution failures, memory telemetry, retained replay and rollback-safe EXPLAIN
       open   review the new bounded timeout/live-stderr evidence and decide any next action separately; no automatic retry; full S3 intake; all-five serde-first qualification
 M3  proc-macro shapes                  not started
 M4–M6 signatures, impls, bodies, methods  not started
 M7  coverage + performance             not started
 M8  HLD builder, full S1–S10           not started
```

```text
 SOP STEP  LATEST RESULT ON CURRENT DIGEST
 S1  structural index        not-started (five pinned roots/locks checked read-only; no source qualification)
 S2  coverage inventory      not-started
 S3  fixture/production map  BLOCKED/incomplete: isolated native M2 retry reached the 2,400s deadline (exit 124; 2,410.252s including cleanup), not an assertion failure; full S3 intake is unimplemented
                             Live stderr recorded a 14-round fixed-point line and later repeated copy/normalization/resolution work; last marker was round 3 import-bindings started. Exact active query remains unknown. Earlier forensic copy counts remain historical; see the separate retry report.
 S4  oracle exports          NOT STARTED; no production/corpus exports or full S3 intake
 S5  per-site comparison     NOT STARTED; comparator R9 code is integrated, but no production corpus comparison or all-five qualification
 S6–S10                      NOT STARTED
```

Stop rule: retained native capture plus two resolution replays completed without raising the cap.
Copy PASS 35.398s / sampled tree peak 1323495424 bytes; normalization PASS 31.469s / 1936781312 bytes.
Resolution fails at candidate INSERT in round 2 (32.273s / 1482792960 bytes) or round 3 on reused derived state.
INSERT EXPLAIN ANALYZE independently OOMs at 4.884s. SELECT-only profile has 133 operator counts; no complete INSERT profile.
Specific join/index root cause is unresolved; no speculative fix. Final 500 MB acceptance remains deferred.

Follow-up hypotheses: H1 already false; explicit SET false OOM 5.354s / 1412620288 bytes.
H2 threads=1 OOM 9.034s / 1394860032 bytes. H3 unconstrained CTAS itself OOM
6.610s / 1409286144 bytes before target INSERT. H4 UNION DISTINCT OOM 3.324s /
1417895936 bytes; removing unused lookahead join OOM 4.807s / 1621385216 bytes.
Allocator attribution is measured: normalization's peak has 802144141 bytes in
C++ non-managed BaseAppender stacks, not Rust JSON/Turso buffering. Full adapter
suite now GREEN (57 tests); invalid completion-marker handoff fixture repaired.

Integrated-source checks and readiness (2026-11-12):
- Python runner tests: 115/115 PASS in 0.766s; process-tree contract tests: 6/6 PASS in 0.317s.
- `cargo fmt --all -- --check`: PASS. All-target/all-feature Clippy: PASS in 3.97s. Workspace release build: PASS in 11.47s; the warmed release DuckDB archive hash stayed unchanged.
- Earlier all-target/all-feature debug-profile workspace tests did not reach a test harness: the 120s command timed out during compilation after Cargo printed `Compiling libduckdb-sys`. Only `test_lang_items_in_sysroot` was requested as an exclusion; that M2 path-resolution/lang-item test remained pending.
- The separate warmed release-profile workspace suite with only `test_lang_items_in_sysroot` skipped completed: 270 passed, 0 failed, 3 ignored, 1 filtered. Those 270 do not include the native test. Its isolated release attempt then timed out at 2,400.11s (exit 124), peak RSS 8,786,596 KiB sampled every 0.2s, with no assertion/result and no memory cap. Exact harness/CLI digests and argv/environment are in the run report and external summary.
- Earlier postmortem checkpoints record the reader-unavailable state and the source/lock finding that bundled DuckDB is 2.0.0. The later session-built reader resolved that prerequisite without a Python wheel or C++ rebuild; read-only inspection of an external copy returned measured manifest and table counts. Historical journal entries are preserved.
- Latest forensic finding: original and copy DuckDB hashes match before/after, and no DuckDB WAL sidecar exists. Copy and normalization completed for frozen revision 8; unit-link outputs and path/binding rows persisted; fixed-point completion is not evidenced. The exact last SQL operation remains unknown. This query evidence is not original-run DuckDB memory/settings telemetry.
- CLI contract helper regression: 27 passed, 0 failed, 1 ignored (`test_child_fixture_waits_release`), and 1 filtered (`test_lang_items_in_sysroot`) in 4.29s, exit 0, using the warmed locked/offline release target. The ignored fixture supplies live-stderr, exit/output, and output-read cases. No DuckDB C++ rebuild occurred; the native M2 test remains excluded.
- Reviewed oracle branch `eecdf0d3700d33b7522709f01cfbf24fbdba9e90` is integrated. REQ-RUST-027.0 limits the advertised capability to mapped path exports; it does not establish full S4 support or qualification. The child's reviewed RED/34-test/fmt/Clippy/release gates were not repeated here; no corpus export or native M2 rerun occurred.
- Separate native M2 rerun preflight `20261112T222609Z-native-m2-regression-preflight` is BLOCKED before launch: persistent `/dev/root` had 3,213,532 KiB available (98% used), less than the preserved prior 4.0G fixture footprint. No test or Cargo process ran; this is not a timeout or assertion result. The earlier timeout report remains historical and S3 remains BLOCKED/incomplete.
- Diagnostic-only run report: `hld-current-test-runs/20261112T203325Z-native-m2-regression/hld-current-test-report.md`.
- The all-target/all-feature debug build was not run because the timed-out test gate reached `libduckdb-sys`; further debug compilation could require a bundled native build, which was not authorized. No test result or completed debug native build is inferred.
- Five pinned corpus roots and lock hashes match the read-only inventory; stable Rust 1.99.1 `rust-src` core/alloc/std sources are present. `strace`, Cargo, rustc, Python, and process tools are available. The external pinned Python provider `blake3==1.0.10` is provisioned and its vectors/provider selection were verified, but the qualification runner has not been invoked and no corpus run directory was created.
- No memory cap was set, no corpus/native qualification was run, and no release artifact was frozen.

Next actions:
1. Review the new timed-out native retry and retained live phase evidence; do not infer a precise stalled query from the last marker and do not start another retry automatically.
2. Keep full S3 intake, all corpus runs, S4–S10, and qualification not started until the prerequisites and next step are reviewed.
3. The external pinned Python BLAKE3 provider is ready, but keep the qualification runner unused until reviewed.
4. Preserve the no-interim-memory-cap policy; no all-five PASS is claimed. The retry used no external memory cap; its live DuckDB logger reported the application's 25.0 GiB setting.
