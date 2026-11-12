# current-milestone-status-board (phase-20261111)

Rewritten in place after every run report. History lives in the TDD journal and run reports.

Last updated: 2026-11-12 · Frozen release: `/home/ubuntu/parseltongue-mapper-frozen-release`, SHA-256 `1719d2f2de1d70b7e57eed6e7dc6b1988f99fbe3b60b2513732e37f94804cd33`. No all-five qualification exists; prior baseline measurements remain historical, not current native/serde replays.

```text
 MILESTONE  STATE
 M0  core types, Turso store            code done; SOP S3 skeleton subset not re-run on current digest
 M1  ingest, crate graph, spans         code done; S1/S2/S3 historical PASS (old digest) → re-run pending
 M2  name resolution (DuckDB SQL)       mapper implementation in place: measured-only default + publication receipts; all-five S5 prerequisite pending
       done   Turso→DuckDB Appender copy + digest checks; measured DuckDB constructor
       done   non-monotone SQL resolution, preludes, extern aliases, textual macro scope, macro alternation
       done   oracle path partition (module-scope scored at M2; type-relative deferred)
       done   indexed frozen visibility + distinct namespace cleanup indexes (focused tests GREEN)
       done   typed copy/normalization/resolution failures, memory telemetry, retained replay and rollback-safe EXPLAIN
       done   CLI receipt proofs reconcile published rows; 20 CLI contract tests and 64 adapter tests pass
       done   fmt, workspace clippy, and optimized release build; full workspace split gate has the comparator-owned failures documented in TDD
       open   sysroot integration test still running (shell 5fba81); full native/serde mapping not started
       open   comparator · serde M2 qualification · all-five S5
 M3  proc-macro shapes                  not started
 M4–M6 signatures, impls, bodies, methods  not started
 M7  coverage + performance             not started
 M8  HLD builder, full S1–S10           not started
```

```text
 SOP STEP  LATEST RESULT ON CURRENT DIGEST
 S1  structural index        full codebase-memory index ready: 22,913 nodes / 42,999 edges; generation matches
 S2  coverage inventory      CLI src/tests have no recorded issues; adapter src has four SQL parse-partial files; scoped query complete, untruncated
S3  fixture/production map  native full mapping and serde mapping not started in this handoff; sysroot CLI integration test remains active
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
1. Collect the running sysroot test result, then complete scoped index coverage and GREEN/REFACTOR commits.
2. When directed, run the required full native mapping followed by pinned-serde mapping with measured default, memory tags, process-tree peak RSS, elapsed time and rounds. Do not start either from this handoff.
3. Comparator and all-five S5 remain separately owned/pending; enforce ≤500 MB only after the all-five S5 correctness milestone.
