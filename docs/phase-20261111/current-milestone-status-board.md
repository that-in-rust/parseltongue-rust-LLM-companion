# current-milestone-status-board (phase-20261111)

Rewritten in place after every run report. History lives in the TDD journal and run reports.

Last updated: 2026-11-12 · Frozen release: `/home/ubuntu/parseltongue-mapper-frozen-release`, 100,415,760 bytes, SHA-256 `bbeee8c54b5bbb02e77b6aa00c44f2fcf66247cf23d46896c21ccff6f959c363`. No all-five qualification exists; prior baseline measurements remain historical, not current native/serde replays.

```text
 MILESTONE  STATE
 M0  core types, Turso store            code done; SOP S3 skeleton subset not re-run on current digest
 M1  ingest, crate graph, spans         code done; source index refreshed for current banner; final-release native replay pending
 M2  name resolution (DuckDB SQL)       mapper implementation in place: measured-only default + publication receipts + publication-status banner; all-five S5 prerequisite pending
       done   Turso→DuckDB Appender copy + digest checks; measured DuckDB constructor
       done   non-monotone SQL resolution, preludes, extern aliases, textual macro scope, macro alternation
       done   oracle path partition (module-scope scored at M2; type-relative deferred)
       done   indexed frozen visibility + distinct namespace cleanup indexes (focused tests GREEN)
       done   typed copy/normalization/resolution failures, memory telemetry, retained replay and rollback-safe EXPLAIN
       done   CLI receipt proofs reconcile published rows; 23 CLI contract tests and 64 adapter tests pass (the sysroot test is filtered)
       done   REQ-RUST-024.11 publication statuses distinguish implementation availability from published completion
       done   full-workspace Clippy, formatter check, and release build pass on the current digest; split tests fail only in the three comparator-owned assertions
       open   debug sysroot integration test still running (shell 5fba81); final-release native replay is deferred until it completes
       open   comparator · serde M2 qualification · all-five S5
 M3  proc-macro shapes                  not started
 M4–M6 signatures, impls, bodies, methods  not started
 M7  coverage + performance             not started
 M8  HLD builder, full S1–S10           not started
```

```text
 SOP STEP  LATEST RESULT ON CURRENT DIGEST
 S1  structural index        full codebase-memory index ready: 22,939 nodes / 42,996 edges; generation matches
 S2  coverage inventory      CLI src/tests have no recorded issues; adapter src has four SQL parse-partial files; scoped query complete, untruncated
S3  fixture/production map  final release built/frozen; std-enabled sysroot fixture is active on debug binary; final-release replay waits for that native-equivalent run
                             serde/rust-analyzer/ripgrep/tokio/cargo-modules corpus runs BLOCKED
 S4  oracle exports          raw path exports exist (counts only); manifests pending
 S5  per-site comparison     not-run (comparator not implemented)
 S6–S10                      not-run
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

Next actions:
1. Collect the running sysroot test result; then replay the preserved std-enabled fixture with the frozen release binary, sampled whole-tree RSS, elapsed time, rounds, and independent receipt verification.
2. Return the native checkpoint; pinned serde mapping remains unstarted until the next handoff.
3. Comparator and all-five S5 remain separately owned/pending; enforce ≤500 MB only after the all-five S5 correctness milestone.
