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

## M1 REFACTOR — fixture and production gates

Formatted the owned Rust implementations; collapsed nested conditions and removed unnecessary temporary
string allocations. Integration tests explicitly permit unwrap only in test crates, matching the
always-loaded rule; production unwrap remains denied. Added the external corpus measurement driver;
network auditing now covers connect/sendto/sendmsg, not only connect. No database substitution.

Final commands:
- cargo fmt --all --check: exit 0.
- cargo clippy --all-targets -- -D warnings: Finished dev profile in 0.29s.
- cargo test --workspace: all 31 tests pass; store 9/9, graph 6/6, syntax 6/6,
  workers 2/2, CLI 1/1 and M0 7/7.
- cargo build --release -p parseltongue-cli-dispatcher-binary: Finished release profile in 4.32s.
- Python process contracts: Ran 3 tests in 0.115s; OK.

Clippy initially rejected test-only unwrap and four collapsible/unnecessary allocation patterns;
these were fixed without suppressing production warnings. S3 corpus measurement is running through
tools/corpus-production-stage-measurements.py; no corpus success or resource claim is made here.

### S3 live execution handoff

The external corpus driver is still running, shell 086763, log /home/ubuntu/m1-production-measurements.log.
At the observed 11m13s rust-analyzer mapper was alive with 74,336 KiB resident; these are instantaneous
progress observations, NOT its measured peak or completed elapsed time. No error had been emitted.
The database and transaction log were growing; do not rerun another writer against this active store.
The driver proceeds sequentially through all five and writes production-stage-measurement-summary.json,
production-process-audit-result.json and hld-current-coverage-banner.json per repository when each finishes.
S3 is not yet complete, and the final report must be updated from those artifacts before M1 acceptance.

## M1 performance repair — separated telemetry and bounded SQL batches

Stopped the 17-minute traced driver, mapper, sampler and strace supervisor as requested. Its incomplete
trace is not a timing/RSS result. An independent untraced baseline exceeded three minutes; cancelled
after 3:31.34 (GNU time: user 184.76s, system 58.11s, maximum RSS 179472 KiB). This is a censored
baseline, not a completed mapping measurement.

Inspection disproved span-per-transaction writes: each stage file/unit already has one BEGIN CONCURRENT
transaction plus its marker, while immutable content uses one separate insert per file. However, every
unit and queue insertion clears all tables owned by that stage. The RED query-plan test showed cleanup
searched only generation+attempt, scanning the full generation's rows to filter the unit key.
Added generation+attempt+unit indexes and bounded 128-row INSERT statements inside the existing atomic
unit transaction. Added duplicate-in-later-batch rollback and empty-unit replacement coverage.
RED: test_stage_unit_cleanup_indexed FAILED, plan lacked stage_unit_work_key.
GREEN: adapter tests (including index plan and 260-row rollback) and CLI tests pass.
Clippy Finished dev profile in 0.56s; release build Finished release profile in 4.34s.

The S3 driver now measures an untraced mapper with the external RSS sampler, then separately audits
execve/connect (retaining cheap clone/fork ancestry metadata) and exact-path-filtered target openat.
No sampler is traced. Python separation contracts: Ran 4 tests in 0.125s; OK.
Stage elapsed timings go to stderr. The new untraced rust-analyzer attempt measured graph 0.434s and
ingestion 15.546s, then exposed a source-closure bug: raw identifier mod r#impl searched r#impl.rs
instead of impl.rs. Fix next with a focused regression; no complete S3 success is claimed.

### Raw module identifier correction

The optimized failed attempt reached the raw-module error in 32.179s; this is not a full-run throughput
comparison. Added test_module_raw_identifier_paths (RED: expected impl.rs absent), then stripped the
Rust r# escape only for module filesystem components, including inline ancestors. Source bytes and
declared span text remain unchanged. Syntax 7/7 and CLI 1/1 pass; Clippy Finished dev profile in 0.61s;
release build Finished release profile in 3.68s; Python separation tests Ran 4 tests in 0.159s, OK.
The final driver uses fresh nested evidence directories m1-batched-final-measurements to avoid
contamination by interrupted/failed stores. Log /home/ubuntu/m1-final-production-measurements.log.

### M1 bounded persistence refinement — STUB

The fresh optimized N=4 map reached an unlabelled database-is-locked error after 79.699s; its separate
audit explicitly exhausted the six bounded writer attempts. A new mixed read/write fixture with four
writers and 20,480 rows passed, so this does not prove a general read failure. The exact untraced error
operation remains unverified. Corpus file units are larger than that fixture.

Preserve the six-attempt bound and Concurrent transactions. Refine the CLI to read at most N file inputs,
parse those N in parallel, then persist their file units in deterministic order. This avoids long-unit
in-process write contention without global locks, whole-stage buffering, weaker atomicity or changes to
the adapter's independently tested concurrent-writer API. Added a generic bounded worker-map signature.

### M1 bounded persistence refinement — RED

test_pure_worker_results_ordered and test_pure_worker_errors_propagate both fail with todo!().
Contracts cover ordered bounded results, empty input, zero workers, cancellation, errors and panics.
The ripgrep run separately exposed valid comments in cfg predicates from regex-automata; add a pinned
syntax-token normalization regression for comments, raw strings/raw identifiers and boolean literals.

### M1 corpus source compatibility — RED

The completed diagnostic pass failed all five (no coverage publication): rust-analyzer lock conflict;
ripgrep cfg comments; tokio CRLF inside half's multiline doc strings; serde path attributes in syn's
non-mod.rs source; cargo-modules contextual safe! macro names in bitvec. Syntax regressions reproduce
the latter three (8 pass, 3 fail). A separate fixture checks that test-only workspace members cannot
seed normal dependency contexts. Only lib/bin and proc-macro provider roots seed those contexts.

Keep raw bytes and raw byte coordinates authoritative. Parse compiler-normalized CRLF with a lossless
offset restoration map. Use the already-transitive pinned ra_ap_parser public token-input/tree-builder
APIs to retain contextual identifiers in macro paths (macro_rules definitions remain special); no
dependency version changes, source rewriting, vendored patch or semantic-engine dependency is needed.

### M1 bounded persistence and source compatibility — GREEN

Syntax 11/11 pass, including all three source compatibility regressions and cfg-token normalization.
Remaining workspace 32/32 pass: graph 8, scheduler 4, CLI 1, core 3, fake store 1, Turso 15.
The test-only-root fixture initially lacked required Cargo metadata fields; copied complete target/node
shapes and reran against the unfiltered implementation to obtain the intended graph-row assertion
failure, then restored selected-root filtering. No production validation was relaxed.
The source guard fixture checks normal/empty bytes and rejects nested target files plus symlink aliases.
Python separation contracts: Ran 4 tests in 0.128s, OK. Direct parser dependency is the same pinned
already-transitive version; cargo tree gate still passes (L1 dependency-free, no forbidden semantic crates).
strace now uses --seccomp-bpf to avoid stopping the mapper on excluded syscalls.

### M1 bounded persistence and source compatibility — REFACTOR

Extracted pure build_syntax_file_input from the CLI's store-read loop so loading, CPU parsing and
ordered publication remain visibly distinct. cargo fmt --all --check exited 0; workspace Clippy
--all-targets --all-features -D warnings Finished dev profile in 0.89s; affected CLI test 1/1 passes
(1.21s). Other GREEN checks cover unchanged crates (43 total tests across commands). Release build
Finished release profile in 5.71s. No API/version substitution, database change or global mutable lock.
Next fresh corpus pass: m1-ordered-final-measurements; log /home/ubuntu/m1-ordered-production-measurements.log.

### M1 measured publication budget miss — STUB

Stopped the fresh driver/sampler/mapper (812572/812573/812574) before accepting any result. At 5:07:
crate graph 0.307s, ingest 11.804s, syntax 186.792s; publication still running; mapper observed RSS
2,471,044 KiB (a lower bound, above 2 GiB). The sampler itself consumed 54.1% CPU by scanning every
host process every 10ms. Replace that scan with traversal of children of every thread in the mapper tree.

Publication's executed path rewrites the generation key on every output row in one Concurrent transaction;
the last emitted timing is syntax and publication is the next call. This retains a corpus-sized MVCC write
set. Fix the lifecycle: discover source closure into small persisted request rows, compute its final digest,
prepare/rekey only those small rows before emitting spans, then atomically publish only the ready pointer.
No alias generations, partial reader publication or weaker transaction semantics.

Add synchronous prepare/count/visitor ports (STUB) for bounded status aggregation and preparation.
Profile load/parse/write separately on the next untraced attempt; no completed S3 success is claimed.

### M1 publication preparation and bounded readers — RED

Rust: stage-row visitor/count, scope preparation atomicity, and rejection of late preparation fail at
todo!(); reason-count CLI regression fails (a variable named build_script is falsely counted).
Python: test_tree_walk_stays_local errors with NotImplementedError. Its fake proc tree covers descendants
from non-main threads, grandchildren, duplicates/cycles, vanished children, unrelated processes and denial.
The ordinary extensionless include fixture already passes; preserve its digest/tiling contract.

GREEN implementation plan: add early prepare after a discovery-only pass. Discovery uses the SAME file-unit
keys and writes only module-source-closure-requests, not spans/fields/body rows; it does NOT write syntax's
global __complete__ marker. Full extraction then ignores discovery's per-unit complete markers and overwrites
those units with full output rows (normal cleanup removes discovery request rows, avoiding duplicate keys).
The prepared scope already has the final generation digest, so final publish validates all stage completions
and updates the manifest pointer without rewriting corpus rows. Existing direct-publication API stays compatible.
Prepare rejects scopes already containing spans/ownership/fields/body data and changes no published pointer.
Status uses SQL counts and a streaming visitor for inactive/build-script counts and ownership ranges.
Add per-compilation-unit reason counts using actual reason fields, never payload substring matching.

### M1 publication preparation and bounded readers — GREEN

Implemented the plan above. Focused Rust checks: CLI 3/3, traits 1/1, adapter cleanup 1/1,
adapter contracts 14/14, dependency/runtime gates 3/3 (22 total); Python 5/5, including a real
64 MiB child allocation. fmt --all --check and git diff --check exited 0. Workspace Clippy
--all-targets --all-features -D warnings Finished dev profile in 0.72s.
The publication pointer remains unchanged during preparation, missing completions and cancellation.
No database/version/process-policy substitution. Source discovery intentionally adds a bounded second
CPU parsing pass rather than retaining ASTs across stages or aliasing generation identities.
Next: refactor the snapshot hash to the streaming port, then fresh m1-prepared-final-measurements.
