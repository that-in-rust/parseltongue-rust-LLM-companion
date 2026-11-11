# TDD phase-20261111 progress checkpoints

## M0 scope and contracts

Delivery/reliability mode: SPEC §3/§4/§5 owns the interfaces and M0 acceptance.
REQ 010: L1 has zero dependencies; the complete production tree excludes analysis engines/salsa.
REQ 012 M0 subset: full-digest collision safety, occurrence paths/units/config/provenance are distinct.
M0 also supplies typed errors, synchronous store traits, an isolated fake and a Turso API foundation.
M1 owns stage schemas, persistence, scheduling and full runtime behavior, not this milestone.
The external oracle is a separate workspace; M0 verifies its pin/build provenance, not semantic export gates.
No pipeline, final HLD export, corpus agreement or memory gate is claimed at M0.

Decisions:
- Root was free of Cargo manifests/crates; existing target/ is preserved and already ignored.
- Keep L1 dependency-free, including dev dependencies; use standard-library typed errors.
- .claude/skills was already tracked and correct; Part 1 preserves it.
- Named tdd-workflow-for-rust-stub-red-green-refactor skill is absent locally/plugins;
  use the explicit user sequence and the repo skills' equivalent workflow, without inventing a skill.
- Read rust-coder-02 and idiomatic-rust-coder-01 SKILL.md and relevant references before code.
- Test fixture stores are not production storage; Turso remains the only database.

## STUB — signatures compile

Status: PASS, no runtime behavior claimed. Public identity constructors and adapter methods use todo!().
- cargo check --workspace: Finished dev profile in 38.54s; exact Turso 0.9.0-pre.5 compiled.
- cargo check --offline --manifest-path tools/oracle-semantic-export-harness/Cargo.toml:
  Finished dev profile in 31.17s; links pinned checkout ide/hir in its separate workspace.
- Root and harness Cargo.lock files record the independently resolved dependency sets.
- Harness expects the specified sibling rust-tools/rust-analyzer checkout and checks its exact clean pin.
- Stable Cargo warns that upstream lints.cargo needs nightly -Zcargo-lints; this is an upstream optional
  manifest-lint feature, not a missing project environment. No upstream files/toolchain were changed.
- Next: add SPEC-named dependency/identity tests and fake-store/oracle/adapter boundary tests; record RED.
- Blockers: no build blocker; full semantic export intentionally belongs to later milestones.

## RED — executable contracts fail on stubs

- cargo test --workspace --no-fail-fast: exit 101; core 0 passed/3 failed, fake store 0/1,
  adapter/build gates 0/3. Constructors and adapter methods panic with `not yet implemented`.
- cargo test --offline --manifest-path tools/oracle-semantic-export-harness/Cargo.toml:
  exit 101; test_oracle_build_manifest_pinned 0 passed/1 failed, `not yet implemented`.
- SPEC names: test_semantic_identity_collision_safety and test_no_analysis_crates_linked;
  additional tests cover reversed/empty ranges, invalid paths/units, generation-isolated fake rows,
  Turso concurrent API, nested-runtime error and oracle provenance.
- Dependency gate additionally exposed unprovisioned cross-platform metadata dependency
  `android_system_properties v0.1.6`; this is fixture provisioning, not a forbidden-dependency result.
  Next: cargo fetch --locked before the offline metadata gate, then implement the stubs without changing tests.
- Full RED logs: /home/ubuntu/m0-red-production-tests.log and /home/ubuntu/m0-red-oracle-tests.log.
- No Turso compiler/API failure occurred. No database substitution or test weakening is authorized.

## GREEN — minimum implementations pass unchanged tests

- cargo test --workspace --offline --locked: PASS, core 3/3, fake store 1/1, adapter/build gates 3/3.
- cargo test --offline --locked --manifest-path tools/oracle-semantic-export-harness/Cargo.toml:
  PASS, oracle manifest 1/1. Stub bodies removed; tests unchanged from RED.
- cargo run --offline --locked --manifest-path tools/oracle-semantic-export-harness/Cargo.toml
  -- --oracle-build-manifest-output: built executable and recorded its BLAKE3 digest/toolchain/source pin
  in oracle-source-build-manifest.json. This artifact explicitly says provenance_only, not semantic export.
- cargo fetch --locked provisioned missing cross-platform packages; offline metadata now passes.
- Corrected our initial namespace assumption: the enum is turso::transaction::TransactionBehavior,
  not a root reexport. Source confirms the specified API exists; this was our compile error, not a SPEC
  mismatch. Concurrent also requires PRAGMA journal_mode='mvcc', applied before the smoke transaction.
- Adapter verifies async build/query/execute/prepare and concurrent begin/rollback behind a sync L3 runtime.
  This is an API foundation, not M1 schema/persistence/concurrency correctness implementation.
- Next: refactor pure validation/error diagnostics, format and run final gates; no open build blocker.

## REFACTOR — functional boundaries and final gates

- Extracted pure path/unit validation, destructured range inputs and improved typed error diagnostics.
  Core remains dependency-free. Small clones preserve clear ownership; no speculative cache/optimisation.
- Database handle drops before its owned runtime. Synchronous adapter rejects async-runtime entry;
  callers must also drop the adapter outside async executors, as documented.
- Formatted production and external harness; RED test assertions remain unchanged.
- cargo fmt --all -- --check, plus the harness manifest equivalent: PASS.
- cargo clippy --workspace --all-targets --all-features --offline --locked -- -D warnings:
  Finished dev profile in 0.42s, PASS.
- cargo test --workspace --all-targets --all-features --offline --locked:
  core 3 passed, fake store 1 passed, adapter/build gates 3 passed; zero failures.
- cargo build --workspace --all-targets --all-features --offline --locked:
  Finished dev profile in 0.12s, PASS.
- External harness clippy --all-targets -- -D warnings: PASS, Finished dev profile in 0.44s.
- External harness offline/locked tests: 1 passed, zero failures; executable rebuilt and manifest refreshed.
- Naming/source audit: four-word owned functions/tests/types; no todo!/unimplemented!, unsafe,
  static mut, unwrap or expect in the new Rust sources. git diff --check: PASS.
- Warning remains external only: stable Cargo ignores upstream lints.cargo; source and environment diagnosis
  recorded under STUB. The harness intentionally links salsa; the production dependency gate excludes it.
- Final production test log: /home/ubuntu/m0-final-production-tests.log.
- Next: M1 graph/ingest/parser, stage schemas, persistence/runtime integration, generation scheduler.
  Extend the harness with real semantic exports at the owning resolution milestones.
- Blockers: none for this M0 subset. No release-performance, corpus, final HLD or full semantic export claim.

## M1 preflight — blocked before STUB by process policy

- Cloned ripgrep/tokio/serde; all five checkouts exist under ~/repos/rust-tools.
  Full current HEAD pins are recorded in SPEC §8 and corpus-source-pin-manifest.json.
- Ran cargo fetch and cargo check --workspace exactly once per repo, outside production.
  All fetch/check exit codes were zero; logs are under e2e-runs/2026-11-11/<repo>/.
  Compilation outputs are isolated under /home/ubuntu/m1-corpus-build-targets/<repo>/.
  Matching rust-src library sources are present in the installed toolchain.
- A strace preflight of the exact required cargo metadata --offline command on rust-analyzer emitted
  rustc -vV and a target-information probe (--print=file-names, --crate-type, etc.).
  Both exceed REQ 001's process-tree allowlist; neither executes a build script.
- Countercheck: the same command on the warm-cache Parseltongue workspace emitted no rustc subprocesses.
  Thus cache-dependent success cannot prove that the strict allowlist works on cold corpus inputs.
- This is a preflight command trace, not a claim about a completed M1 production dispatch.
- Decision needed before STUB: permit Cargo's compiler-discovery probes as narrowly scoped descendants,
  or require cold-cache metadata runs to fail under the unchanged allowlist. Do not silently whitelist them.
- STUB/RED/GREEN/REFACTOR and S1/S2/S3 have not run. No M1 commit or passing stage count is claimed.

## M1 process-policy amendment — user decision

Direct children remain the original three commands. Only beneath metadata Cargo, rustc -vV or the
stdin crate-name ___ print-only discovery probe is permitted; no other descendants or network.
REQ 001 and SOP now require ancestry plus full-argv strace validation. Unknown flags fail closed.
The captured Cargo probe also contains --crate-type and -Wwarnings; these are not silently exempted by
the literal print-only policy and will be reported if observed in S3.
Corpus pins, qualification logs, artifact ignore rules and this amendment form the prep commit.
Next: M1 STUB, RED, GREEN and REFACTOR with SPEC-named tests and measured S1–S3 reports.

## M1 STUB — persisted boundary signatures

Added graph/ingest, pinned syntax extraction, bounded scheduler and CLI crates using SPEC component names.
L1 adds cancellation, stage scopes and generic typed stage-row envelopes with zero dependencies.
Store traits expose atomic stage units, immutable byte content and generation publication; implementations
are todo!() stubs. Graph subprocesses remain behind a command port. Syntax extraction is a pure byte input
to row/request output function; discovery requests must persist before ingestion resumes.
The external Python process/RSS auditor is scaffolded separately from production.
Next: SPEC-named failing M1 tests, including negative process arguments and publication/cancellation cases.
No runtime M1 behavior or corpus mapping pass is claimed at STUB.

STUB check: cargo check --workspace finished successfully in 29.70s (full output: /home/ubuntu/m1-stub-check.log).

## M1 RED — executable fixture contracts

Added SPEC-named graph (6), syntax (6), scheduler (2), store (9), CLI (1) and external audit (2) tests.
Graph fixtures exercise missing locks, lock mutation restoration, stderr, cfg, metadata Cargo env and
build-script provenance. Syntax fixtures include empty/binary/error input, tiling, fields/body sites,
inactive modules, path/include closure and OUT_DIR. Store fixtures cover unchanged byte content, reruns,
stage ownership, missing inputs, concurrent writers, bounded retries, atomic publication and cancellation.
CLI fixture requires 1/4-worker deterministic generations, non-closure bytes-only files and retention of
the old publication after an active-file parser failure.

Observed RED: graph 0 passed/6 failed; syntax 0/6; scheduler 0/2; store 0/8 plus retry 0/1;
CLI 0/1. All these failures are todo!() panics. Python: Ran 2 tests; FAILED (errors=2), NotImplementedError.
The existing offline dependency gate also failed on an uncached cross-platform objc2 package; cargo fetch
--locked provisioned it. Do not count that infrastructure failure as a behavior RED.
Logs: /home/ubuntu/m1-red-tests.log, m1-red-cli-tests.log, m1-red-retry-tests.log,
m1-red-process-tests.log. Next: minimal implementations, then run the entire changed fixture suite.
S1 fresh indexes completed for all five repos with exit zero; S2 pagination is in progress.

## M1 GREEN work in progress — two remaining policy decisions

Prep f5aa564d8, STUB d91ac4069 and RED 63b5406d4 are pushed. M1 production bodies remain todo!();
do not mistake the boundary commits or the following external checks for M1 completion.

The external argv/ancestry auditor now passes its two tests (Ran 2 tests in 0.001s; OK).
A new cold-cache cargo metadata --offline trace still contains --crate-type bin/rlib/dylib/cdylib/staticlib/
proc-macro and -Wwarnings. The literal approved print-only grammar rejects these. A real Cargo trace
embedded beneath a synthetic mapper parent validates this rejection; a production mapper was not run.
Need a narrowly scoped flag-selector decision before claiming the process requirement can pass.

All five S1 indexes and S2 inventories were collected; per-repo metrics are in the run report.
Second blocker: rust-analyzer's last scope page has has_more=false, scope_truncated=true, total=294,
returned=14. The 15 pages contain exactly 294 entries. The literal SOP S2 gate therefore fails despite
complete accumulated enumeration; other four repos meet its final-page flags. No SOP S2 change applied.
The inventory driver preserves all responses and exits nonzero for this contradiction.

Uncommitted work: strict process auditor, complete-coverage inventory driver, updated run report and this
checkpoint. No GREEN/REFACTOR commit; no production spans, Turso rows, S3 counts, RSS or performance claim.

## M1 GREEN — implementations and approved policy refinements

Both remaining amendments are approved and recorded in SPEC/SOP. The auditor accepts only the six fixed
crate-type selectors and -Wwarnings within the approved stdin ___ print probe; output/emit/real-input
arguments remain rejected. S2 now gates stable generation, advancing offsets and total reconciliation;
all five saved inventories pass. A fresh Cargo metadata .rustc_info.json cache is discovery provenance,
not a saved build-script output; the auditor permits only that exact target-cache basename.

Implemented byte-preserving immutable content storage, stage-owned physical tables, MVCC BEGIN CONCURRENT
unit transactions with bounded conflict retries, prerequisite markers and atomic generation publication.
Graph loading protects Cargo.lock and resolves normal/build/proc-macro feature contexts separately; dev
dependencies are not mapped. Initial source ingestion stores all .rs bytes; only active module/include
closure files are parsed with pinned ra_ap_syntax. Inactive items and OUT_DIR includes carry explicit reasons.
Extraction writes semantic item occurrences, signatures, pending body sites and contiguous ownership ranges.
Module/include requests and syntax work queues are persisted; no graph/ingest/parse in-memory stage handoff.
The CLI implements map-workspace-full-run and JSON coverage; later stages explicitly say not yet implemented.

Checks: all 31 production Rust tests pass across the workspace run plus the corrected graph/CLI rerun.
Graph 6/6, syntax 6/6, scheduler 2/2, store 9/9, CLI 1/1; M0 tests 7/7.
External Python tests: Ran 3 tests in 0.149s; OK. Added a memory-sampler RED first
(NotImplementedError), then implemented process-tree RSS sampling at nominal 10 ms intervals.
Logs: /home/ubuntu/m1-green-tests.log, m1-green-cli-tests.log, m1-green-graph-cli-tests.log.

Two fixture defects were corrected without weakening production: the hand-written Cargo.lock was
noncanonical and was correctly rejected as mutated; the fixture now runs offline generate-lockfile in
the external test driver before mapping. Its bytes-only inventory is four files, not three, because its
build.rs must also be stored without execution. The lock guard and all-.rs coverage remain enforced.
An initial Rust type-inference error in optional publication decoding was fixed with an explicit error type.
Next: functional cleanup, fmt/clippy/test/build gates, S3 process/RSS measurements for all five.
