# Full-map first-call checkpoint

Recorded 2026-11-14 20:24 IST. **Bounded development checkpoint: PASS. Milestone: NOT QUALIFIED.**

This is the pinned-itoa library slice, not a whole-itoa golden comparison, all-five SOP run,
native/sysroot-inclusive pass, or final 500 MB qualification. Work after consolidation used
only the current VM. No commit, push or PR was created; the three implementation workers remain
archived/suspended. The TDD skill's commit steps are deferred pending user authorization.

## Expected, actual, missing

Expected: `<Buffer as Default>::default --calls--> Buffer::new` from actual source.

Actual: `test_original_member_graph_roundtrip` and `test_default_new_call_roundtrip` pass through
the production CLI and published/frozen Turso rows. They verify distinct canonical method
identities, exact item and call-site ranges/digests, the `Buffer::new()` bytes, impl bridges,
one lexical source container per item, body-local exclusion, the body type-prefix namespace
fact, unique associated-function selection and exactly one extracted target call.
Frozen readback preserves incomplete input, unknown visibility and missing reachability.
Strict HLD validation rejects it; the full-map stage emits no HLD output tables.

Current retained pipeline output:

```text
original inventory: body-evidence=5 impl-record=5 inactive=2 included=43
full-item-dependency-graph: nodes=45 containment=43 calls=61 resolved_calls=2 completeness=incomplete
frozen-full-map-readback: nodes=45 edges=104 completeness=incomplete
```

The 45 nodes comprise 43 original items and 2 file-area nodes. The 104 edge/evidence records
include 59 unresolved calls; they are not 104 resolved semantic relationships. The target call
is independently asserted; the second resolved call is only a pipeline count in this report.

Still missing: generic/supertrait dispatch (`Buffer::format -> private::Sealed::write` next),
complete signatures/visibility, macro/core coverage, fields/variants and broader non-function
dependency semantics. Imports, reads and other missing facts cannot be recovered by later queries.
Full-map comparison and all-five SOP qualification remain outstanding; HLD derivation is deferred.

## TDD and regressions

The journal records STUB compilation; behavioral RED for 16 projection tests, one codec test
and two real-source tests; minimum implementation; and GREEN/frozen readback.
Readback initially failed only on record ordering; both sides are normalized before exact equality.
Clippy caught a test-module placement issue; it was moved without lint suppression.

Final hardening reproduced child-file items incorrectly labelled CrateRoot in both a component
and real-source test. Both reached their intended assertion and failed with
`Some(CrateRoot) != Some(Module)`. The fix uses the persisted Cargo target root, not filename
spelling; containment remains the separate file-area relation. All 17 projection tests pass.
The source path-count regression preserves the existing 202 import/signature facts and reason
checks, while separately asserting 29 new body-prefix facts. No missing-core check was removed.

## Verification

Build environment throughout the final cycle:

```text
rustc 1.99.1 (58cf8f9c3 2026-10-13); cargo 1.99.1
CARGO_BUILD_JOBS=2 CARGO_PROFILE_RELEASE_DEBUG=0 CARGO_INCREMENTAL=0 CXXFLAGS=-g0
cargo fmt --all -- --check
cargo clippy --release --workspace --all-targets --all-features -- -D warnings
cargo test --release --workspace --all-targets --all-features -- --skip test_lang_items_in_sysroot --test-threads=1
cargo build --release --workspace --all-targets --all-features
```

- Formatting: PASS.
- Warnings-denied Clippy: PASS, 2.597308s, 758,792,192 bytes process-tree peak.
- Release all-target/all-feature build: PASS, 1.244002s, 911,904,768 bytes peak.
- Release test compilation: PASS, 36.952237s, 1,499,467,776 bytes peak.
- Bounded workspace: **415 passed, 0 failed, 4 ignored, 1 filtered**, 21 harness results;
  154.884209s, 506,855,424 bytes process-tree peak.
- All 11 real-source tests: PASS in 87.32s, included in that workspace total.
- Python SOP runner: 146 passed in 2.950s (unchanged runner).

Ignored: one subprocess fixture and three manual Turso profiling tests. Explicitly excluded:
`test_lang_items_in_sysroot`, under the prior native three-attempt stop rule. Therefore the
unrestricted workspace gate is not a PASS. Stable Cargo emits inherited parser/syntax manifest
warnings for unsupported `lints.cargo`; these are not Clippy code warnings.

Formal SOP status remains S1/S2/S4–S10 not started for this artifact and S3 blocked on the full
native path. The source-backed development checkpoint above is narrower than those gates.

## Retained measured run

```text
frozen-release-graph-executable map-workspace-full-run
  --workspace-root-path-value <artifact-dir>/pinned-itoa-source-workspace
  --workspace-store-file-path <artifact-dir>/frozen-itoa-full-map.db
  --worker-thread-count-value 2
  --sysroot-source-scope-kind excluded
  --mapping-final-stage-kind full-item-dependency-graph
```

The existing `tools/process-tree-access-auditor.py sample` wrapper measured the whole command:
**exit 0; 14.810602s; 410,705,920 bytes sampled process-tree peak**.
DuckDB reports version 2.0.0, 25.0 GiB effective memory limit and 2 threads; no interim cap.
Post-run free disk: 8,288,354,304 bytes. Build/test process peaks are not mapper memory claims.
This small diagnostic run does not qualify the final all-repository 500 MB budget.

## Frozen identities and evidence

- itoa: `af77385d0daf4d0e949e81f2588be2e44f69f086`, library, Rust 2021,
  x86_64-unknown-linux-gnu, no features, cfg(test)=false, sysroot excluded.
- Cargo manifest and both library files match the existing pin. Retained Cargo.lock SHA-256:
  `965fa758128c03489e02550fbf8f7fb0e2b36e6e82163c3cf482900cfa8d019d`.
- Parser checkout is clean at `bf6b2099d6351c37039b01447a9bb64f9a54a586`.
- Frozen executable SHA-256:
  `ec9d24df192152263ed5bf6cec0db0c3cc20b64a2498886bcc4a1b63fb85fcde`.
- Runtime input-manifest digest (canonical sorted records, 1,401 files: 147 integration and
  1,254 parser inputs): `114238fcdffe3d2b82d2459242f59954f94537a5015e54b45e3511edce073e49`.
- Frozen full-map readback attempt:
  `dd1ff9976ae965b6945d8742f1cae3a8c3b229e6124622ac093e79a0f7f92e0c/frozen/20`.

Artifact directory on the session VM:
`/home/ubuntu/hld-current-test-artifacts/20261114T140300Z-full-map-implementation/`.

Retained there: `frozen-release-graph-executable`, `verified-runtime-source-manifest.json`,
`verified-runtime-source-archive.tar.gz`, `frozen-itoa-input-manifest.json`, the pinned source
workspace, native Turso database and sidecars, `frozen-itoa-run.log`, process-tree samples,
`verify-final-workspace.log`, final Clippy/build/format logs, component RED/GREEN evidence,
`verify-runner.log` and original worker recovery patches. Earlier failed/partial logs remain
historical and are not represented as final passes. Native Turso files require its adapter;
Python sqlite3 cannot read this format. No extra database was introduced.

Current source changes are uncommitted. Any production change invalidates this executable's
evidence and requires a fresh run; this report does not authorize resuming blocked native tests.
