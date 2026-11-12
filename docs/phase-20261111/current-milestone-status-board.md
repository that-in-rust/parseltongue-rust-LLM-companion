# current-milestone-status-board (phase-20261111)

Rewritten in place after every run report. History lives in the TDD journal and run reports.

Last updated: 2026-11-12 · Latest measurement remains the historical baseline at `~/hld-current-test-artifacts/20261112-namespace-phase-probes/namespace-projection-lifecycle-controls/baseline-copy-storage-snapshot.log`. Mapper, runner, and comparator descendant histories are integrated through source merge tip `0f1fd73e0d655c2874fda09d5fb7d2f4e344db10`; code remains unqualified. No all-five PASS, production qualification, or frozen release exists.

```text
 MILESTONE  STATE
 M0  core types, Turso store            code done; SOP S3 skeleton subset not re-run on current digest
 M1  ingest, crate graph, spans         code done; S1/S2/S3 historical PASS (old digest) → re-run pending
 M2  name resolution (DuckDB SQL)       mapper plus runner/comparator descendants integrated but unqualified; native diagnostic unresolved; all-five S5 pending
       code   measured-only default, generation-backed publication receipts, and stage-status banner integrated
       runner component code/tests integrated; full S3 intake, S4, and S5 remain blocked/unimplemented
       comparator R9 file-backed follow-up integrated; no production comparison or final freeze
       done   non-monotone SQL resolution, preludes, extern aliases, textual macro scope, macro alternation
       done   oracle path partition (module-scope scored at M2; type-relative deferred)
       done   indexed frozen visibility + distinct namespace cleanup indexes (focused tests GREEN)
       done   typed copy/normalization/resolution failures, memory telemetry, retained replay and rollback-safe EXPLAIN
       open   native copy/resolution diagnostic; full S3 intake/S4/S5; serde M2 qualification; all-five S5
 M3  proc-macro shapes                  not started
 M4–M6 signatures, impls, bodies, methods  not started
 M7  coverage + performance             not started
 M8  HLD builder, full S1–S10           not started
```

```text
 SOP STEP  LATEST RESULT ON CURRENT DIGEST
 S1  structural index        not-started (five pinned roots/locks checked read-only; no source qualification)
 S2  coverage inventory      not-started
 S3  fixture/production map  BLOCKED: historical shared prerequisite FAIL (retained native fixture 305.41s; independent resolution OOM at superseded interim cap); native diagnostic unresolved
                             runner component tests pass, but full S3 intake is unimplemented; all five corpus runs remain BLOCKED
 S4  oracle exports          BLOCKED/not-run on the integrated source; historical raw export counts only, manifests pending; oracle fixture/build evidence is not S4 qualification
                             runner preflight would block because no BLAKE3 provider is installed
 S5  per-site comparison     BLOCKED/not-run; comparator R9 code is integrated, but no production corpus comparison or all-five qualification
 S6–S10                      not-started
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
- All-target/all-feature workspace tests did not reach a test harness: the 120s command timed out during compilation after Cargo printed `Compiling libduckdb-sys`. Only `test_lang_items_in_sysroot` was requested as an exclusion; that M2 path-resolution/lang-item test remains pending.
- The all-target/all-feature debug build was not run because the timed-out test gate reached `libduckdb-sys`; further debug compilation could require a bundled native build, which was not authorized. No test result or completed debug native build is inferred.
- Five pinned corpus roots and lock hashes match the read-only inventory; stable Rust 1.99.1 `rust-src` core/alloc/std sources are present. `strace`, Cargo, rustc, Python, and process tools are available; Python `blake3` and `b3sum` are both absent, so runner preflight is blocked. No runner invocation or run directory was created.
- No memory cap was set, no corpus/native qualification was run, and no release artifact was frozen.

Next actions:
1. Resolve the native diagnostic on the integrated tip without an interim memory cap; record effective DuckDB settings, duckdb_memory() tags, phase time, and whole-process-tree RSS.
2. Obtain a compatible all-feature debug cache or approval for a native rebuild before retrying the all-target/all-feature workspace test/build gates.
3. Provide an approved Python BLAKE3 module or `b3sum` before runner preflight; keep the runner unused until reviewed.
4. Implement full S3 intake/S4/S5 and perform serde-first all-five qualification only after prerequisites and lead review; no all-five PASS is claimed.
