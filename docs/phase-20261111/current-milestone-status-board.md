# Current milestone status board

Updated 2026-11-14 20:24 IST. **Bounded first-call checkpoint PASS; full map remains incomplete.**
Expected relationship: `<Buffer as Default>::default -> Buffer::new` with exact evidence.
Actual: exactly one extracted target edge is proven through the production CLI, published
graph rows and frozen readback. Canonical endpoints, source ranges/digests, impl bridges,
one lexical source container and exact `Buffer::new()` bytes are checked. Unknown visibility
and absent reachability remain explicit; strict HLD validation rejects the incomplete graph.
All three worker patches are consolidated locally and their sessions remain archived/suspended.
Projection, call resolution and CLI/storage composition are implemented and verified here.
Current evidence: [newest-first Phase 1 log](../phase-20261113/LOG-itoa-full-item-graph-progress.md) and
[append-only TDD journal](TDD-phase-20261111-progress-checkpoints.md).
The source/executable digests recorded there define the tested uncommitted snapshot.
Any subsequent production change requires fresh evidence.

## Current verified runtime snapshot — bounded, not whole-map qualification

- Pinned itoa, library, no features, Rust 2021, x86_64-unknown-linux-gnu, sysroot excluded.
- **45 nodes**: 43 active original non-body items plus 2 file areas. **104 edge/evidence records**:
  43 containment edges and 61 calls, only 2 resolved; 59 remain explicitly unresolved.
  These counts are not complete semantic coverage. Header-only lowering remains incomplete.
- All **11 source-acceptance tests** pass, including the two new full-map tests; **87.32s**.
  Child-file function ownership is Module, not CrateRoot; file-area containment is unchanged.
  Legacy 202 import/signature paths retain their original checks; 29 new body-prefix paths
  are checked separately. Full-map publication does not emit HLD tables.
- Formatting, warnings-denied release Clippy and all-target/all-feature release build: **PASS**.
- Bounded workspace: **415 passed / 0 failed / 4 ignored / 1 filtered**, 21 harness results,
  **154.884209s / 506,855,424 bytes** sampled process-tree peak (one test thread).
  The filtered test is `test_lang_items_in_sysroot`; the full workspace gate is **not PASS**.
- Python SOP runner: **146 passed / 2.950s**. No changes to that runner in this implementation.

Current bounded run report `hld-current-test-runs/20261114T140300Z-full-map-implementation/` (report removed from the tree on 2026-11-14; last present at commit a2e13a3bd):
**14.810602s / 410,705,920 bytes** for the retained single CLI run. Measured DuckDB limit:
25.0 GiB, 2 threads; no interim cap. Executable SHA-256:
`ec9d24df192152263ed5bf6cec0db0c3cc20b64a2498886bcc4a1b63fb85fcde`.
Runtime source manifest: 1,401 files (147 integration, 1,254 pinned parser inputs), digest
`114238fcdffe3d2b82d2459242f59954f94537a5015e54b45e3511edce073e49`.
Retained native Turso database, pin-checked source workspace, executable, source archive,
gate logs and process-tree samples are indexed in the report. The earlier structural report
is historical, not evidence for this newer executable.

## Formal SOP status

```text
S1      NOT STARTED — current-artifact formal corpus run
S2      NOT STARTED
S3      BLOCKED — full native path; bounded source/full-map checkpoint above passes
S4      NOT STARTED
S5      NOT STARTED
S6–S10  NOT STARTED
```

Phase 1 remains itoa-only. Phase 2 retains all-five qualification, serde first.
No whole-itoa equality, complete semantic coverage, worker-count determinism, or final 500 MB
qualification is established.

## Immediate next work and decisions

1. **OD-7 source containment decided** (REQ-RUST-043.0): one lexical source container,
   with separate impl/self-type/trait ownership metadata. No contains/imports collapse.
   File-module/area relationship and generated source/container provenance remain open;
   derived roll-up bucket attribution is deferred.
2. The selected ten subparts toward `default -> Buffer::new` now have bounded integrated evidence.
   No unresolved fact was filled by guessing and no HLD gate was weakened.
3. Next semantic target: `Buffer::format -> private::Sealed::write` through generic/supertrait lookup;
   start with a new real-source behavioral RED. It is not implemented by this checkpoint.
4. Macro provenance, fields/variants and non-function dependency decisions still block affected
   full-map reference rows. Frozen full-map readback must not depend on HLD output.
5. Everything unrelated to building/preserving/verifying this graph is in the
   [backlog](../phase-20261113/BACKLOG-deferred-work-open-decisions.md): abstractions are later queries over
   retained facts, not separate semantic pipelines. The broader six SOP/AGENTS process
   corrections still await approval (BF-05); only the requested scope correction is applied.

PRD/SPEC/ARCH/SOP/AGENTS now separate active full-map requirements from retained deferred HLD
contracts. REQ-RUST-044.0 and its queued tests are contracts, not executable acceptance evidence.

## Operational guardrails

- **No further full-native retry** under the prior three-attempt stop rule.
  The unrestricted workspace run mistakenly included it and failed on disk exhaustion before
  its semantic check; raw failure evidence is retained. The final bounded run excluded it.
- Earlier disk failures are historical. Current retained-run disk measurement:
  **8,288,354,304 bytes free**. No source, release executable or evidence was removed here.
- Codebase-memory refreshed: 26,428 nodes / 57,341 edges; 6 partially parsed files, 0 unusable/skipped.
  Use direct source inspection where flagged; the index is not a semantic oracle.
- Earlier native/all-five diagnostics remain historical, not permission to resume them.
  Details are in their retained reports and the append-only TDD journal.
- No commit or push for this work.
- Cargo targets were rebuilt after the requested cleanup. No native-sysroot retry was made.
  Work, parallel processes, integration and tests use this VM only (AGENTS rule).
