# current-milestone-status-board (phase-20261111)

Rewritten in place after every run report. History lives in the TDD journal and run reports.

Last updated: 2026-11-12 · No valid SOP run report exists yet (SOP R2: pre-DuckDB M1 evidence is historical only).

```text
 MILESTONE  STATE
 M0  core types, Turso store            code done; SOP S3 skeleton subset not re-run on current digest
 M1  ingest, crate graph, spans         code done; S1/S2/S3 historical PASS (old digest) → re-run pending
 M2  name resolution (DuckDB SQL)       in progress
       done   Turso→DuckDB Appender copy + digest checks; bounded DuckDB constructor
       done   non-monotone SQL resolution, preludes, extern aliases, textual macro scope, macro alternation
       done   oracle path partition (module-scope scored at M2; type-relative deferred)
       open   CLI full-run wiring (one contract test failing/hung) · comparator · serde M2 run · all-five S5
 M3  proc-macro shapes                  not started
 M4–M6 signatures, impls, bodies, methods  not started
 M7  coverage + performance             not started
 M8  HLD builder, full S1–S10           not started
```

```text
 SOP STEP  LATEST RESULT ON CURRENT DIGEST
 S1  structural index        not-run
 S2  coverage inventory      not-run
 S3  fixture/production map  not-run
 S4  oracle exports          raw path exports exist (counts only); manifests pending
 S5  per-site comparison     not-run (comparator not implemented)
 S6–S10                      not-run
```

Next three actions:
1. Fix the failing CLI full-run contract test; commit current WIP through GREEN/REFACTOR.
2. Run 1: S1/S2/S3 on all five repos with one frozen release executable → first run report.
3. serde M2 run (peak tree RSS, wall, rounds) → SQL comparator → S5 on all five.
