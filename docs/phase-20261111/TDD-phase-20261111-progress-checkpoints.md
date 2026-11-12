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

### M1 publication preparation and bounded readers — REFACTOR

Extracted compute_workspace_generation_digest and streamed its ordered inputs without changing the hash
framing or table order; clarified default trait parameter names. fmt check and diff check exited 0.
Workspace Clippy --all-targets --all-features -D warnings Finished dev profile in 0.66s.
All 48 workspace tests pass (CLI 3, core 3, graph 8, scheduler 4, syntax 11, fake store 1,
cleanup plan 1, store contracts 14, build/runtime gates 3). Release Finished in 4.58s.
Unchanged Python check: Ran 5 tests in 0.349s, OK. No production todo/panic added.
Fresh run: m1-prepared-final-measurements, log /home/ubuntu/m1-prepared-production-measurements.log.
Do not rebuild/replace the release binary until all three separate runs per repository finish.

### M1 residual persistence profiling — REFACTOR

All fifteen corpus runs completed before replacing the measured release binary. Five mapper/status
results are successful; all tiling identities, sampled RSS budgets and both separate audits pass.
Final report numbers use m1-prepared-final-measurements and the unchanged 2c461fd80 binary.
Rust-analyzer: 228.278 s, 0.669 GiB; cargo-modules: 222.112 s, 0.485 GiB. Both remain above the
approximate three-minute goal; do not describe that goal as passed.

Added opt-in PARSELTONGUE_PROFILE_STAGE_WRITES diagnostics at the L3 transaction boundary, and
manual release-mode profiles for statement size, atomic unit size and an alternative JSON bulk
statement. The production batch size remains 128 and transactions remain atomic per file unit.
JSON batching is confined to tests: its quoted/NUL/Unicode payload, empty replacement and duplicate
rollback contract passes, but it is slower than bound VALUES in the measured synthetic workload.
Statement sizes 32..1024 and unit sizes 64..4096 do not show a substantial monotonic batching win.
No rejected optimization was enabled, no database/version change, and no atomicity weakening.

Separate real rust-analyzer profile: exit 0, 192.941 s, 0.664 GiB sampled tree RSS.
Exactly 2612 extraction units / 1136982 records: setup 0.460 s; cleanup 0.846 s;
INSERT 41.243 s; Concurrent commit 106.921 s (summed adapter wall intervals).
That locates remaining persistence time at commit. It does not prove which Turso internals consume
that interval, and the faster diagnostic wall time is not an attributable optimization result.
Logs: /home/ubuntu/m1-row-batch-profile-log.txt, m1-json-batch-profile-log.txt,
m1-unit-size-profile-log.txt, and rust-analyzer/m1-write-phase-profile under the dated artifacts.

Checks after the final Rust edit: fmt --all --check exit 0; adapter Clippy all targets/features
-D warnings Finished dev profile in 0.84s; adapter + CLI tests: 22 passed, 3 manual profiles ignored.
All three ignored profiles were also run explicitly in release mode and exited zero.
Next: publish the all-five S3 report, then implement M2 semantic oracle exports and name resolution.

### M2 oracle investigation — not implemented

Pinned hir/src/semantics/source_to_def.rs documents the ambiguity of a physical file in multiple
crates. Semantics::new has no active-crate selector; base-db::relevant_crates selects the file's
source-root crates. Removing other crate roots alone is insufficient when the same file is also
a dependency target with another configuration.
Candidate approach, not yet compiled/verified: duplicate file IDs in memory while preserving
canonical physical VfsPaths in a separate FileSet/SourceRoot, and point only the selected unit's
crate root at its duplicate. SourceDatabase::resolve_path uses the anchor's SourceRoot only,
so the duplicate FileSet must contain every loaded source path needed by relative modules/includes,
not merely the root file. Dependencies keep original IDs. Reuse this duplicate source set across
unit exports and swap the selected crate root; do not invent virtual filenames that change file!().
Normalize duplicate IDs back to physical paths in exported site/target identities.

PRIMARY must keep load_out_dirs_from_check=false while enabling proc macros from verified cached
dylibs; SECONDARY enables build scripts. load-cargo's public builder path and proc-macro loading
need implementation/verification; ordinary load_workspace_at alone does not establish that pairing.
Current harness remains provenance-only; no semantic exports or S5 denominators exist yet.

### M1 source closure verification — characterization

Investigated the user's fixture-leak hypothesis before changing production code. Persisted spans
all join to syntax-file-work-requests and mapped graph units. cargo-modules: 5597 byte-inventory
files, 2535 closure files, 39 local closure files / 577 local spans; 113438 dependency spans.
All 48 local tests/ files remain byte-only, with zero requests/spans. ripgrep: 1039 byte-inventory
files, 391 closure files, 78 local closure files / 4728 local spans; 16706 dependency spans.
Its ten root tests/ files have zero requests/spans. Manifests confirm cargo-modules's large normal
ra_ap_* dependency stack and ripgrep's nine library members plus binary. No production leak found.
Read-only audit tool: /home/ubuntu/corpus-source-closure-inventory; full JSON inventories are ignored
artifacts next to both original S3 summaries. Clarified report labels; no mapping/audit reruns needed.

Added test_fixture_projects_remain_unmapped BEFORE any contemplated closure fix. It passes without
production changes and covers nested fixture Cargo projects, malformed test/example/bench bytes,
library-form examples, inactive modules and an active zero-byte module. Ten ingested files, three
requests; every emitted span belongs to a requested path. The first execution failed only because
the fixture retained a Turso handle while opening a status subprocess; drop it before that subprocess.
No false product RED or closure fix is claimed. Final focused result: 1 passed, 0 failed;
fmt check exited 0; CLI all-target/all-feature Clippy -D warnings Finished in 0.55s.

M2 oracle direction follows the latest user preference: first try per-crate Module/ModuleDef traversal
and Module::resolve_mod_path; only use duplicate IDs if a fixture shows the public scoped APIs cannot
handle an actual case. SemanticsScope::speculative_resolve exists but its resolver is private and it
explicitly disregards hygiene. Do not use that heuristic as an exact answer key without verification.

### M2 oracle path export — STUB

Added typed OracleWorkspaceExportConfig / OracleSemanticExportError and
export_primary_semantic_paths signature, plus a CLI root/output route. Implementation is todo!();
the build manifest deliberately still says provenance_only. Added pinned-checkout load-cargo,
project-model, ide-db, syntax and vfs dependencies ONLY in the offline harness workspace.
Production dependency graph is untouched. Offline cargo check finished successfully in 24.22s.
Next RED: empty output, bad workspace, exact normalized targets, cfg/test exclusion and a physical
module shared by two crates whose chosen dependency aliases point to different packages.
PRIMARY must never run build scripts; cached proc-macro DLL injection still needs implementation.
No semantic export, M2 resolver/differ, or S5 result is claimed by this phase.

### M2 oracle path export — RED

Added four executable contracts: test_nameres_matches_oracle_paths (exact alias target/range and
cfg/test/example exclusions), test_shared_modules_preserve_context (one physical module, two crates,
different dependency aliases), test_empty_oracle_paths_succeed, test_missing_oracle_workspace_errors.
Ran the focused offline oracle test filter; all four fail at the intended todo!(), not a compiler or
dependency failure: "not yet implemented"; "FAILED. 0 passed; 4 failed; 1 filtered out".
Next implement PRIMARY loading with scripts disabled and proc macros enabled, verify module-first
resolution on the shared-file fixture, normalize per-site rows, then refactor and run oracle gates.
Production resolver, proc-macro cache injection, PRIMARY/SECONDARY pairing and S5 remain unfinished.

### M2 oracle base paths — GREEN

Implemented workspace-member lib/bin selection, cfg(test) OFF, native cfg/cfg_attr filtering,
use/signature-only path rows with raw byte ranges and normalized target sources. Removed the
rust_analyzer cfg flag from PRIMARY to match the production inputs. Module::resolve_mod_path
resolves the shared physical file's chosen alias to the correct left/right package in each crate;
no duplicated IDs are used. Non-module-qualified multi-crate paths currently fail closed.
PRIMARY never calls run_build_scripts. It starts the sysroot proc-macro server, but cached provider
injection is still unsupported (an explicit typed error when a cache manifest is supplied).
Missing normalized answers retain external_source_missing from the shared authoritative reason enum.
Generic/self targets, expansion provenance and per-unit feature overrides still need follow-up.

The first shared-file execution exposed a different issue before resolution: native load-cargo's
separate package roots omitted the workspace-root shared.rs. Adapted its loader (with source/license
notice) to merge local roots through the workspace include while excluding configured Cargo target,
root target and .git paths. This made the existing, unchanged shared-file test pass. No module-ID
duplication was needed. Corrected initial API mismatches: AbsPathBuf accepts &str, original_range
returns EditionedFileId, and NameRef::text is already &str.

Focused offline oracle tests: "ok. 4 passed; 0 failed; 1 filtered out" in 36.40s.
Stable Cargo's upstream unused lints.cargo warnings are expected: verified stable toolchain,
cargo 1.99.1 and rustc 1.99.1; no missing environment/setup. Pinned upstream manifests remain untouched.
This is a foundation GREEN, NOT completed M2 or a valid corpus answer key yet. Next refactor/gates,
then per-crate generic/import fixtures, validated proc-macro cache loading and paired exports.

### M2 oracle base paths — REFACTOR

Cargo configuration is now a pure constructor; path-prefix state uses try_fold instead of
mutable module/name accumulators. Clippy found a redundant guard in the adapted loader;
replaced it with an equivalent LoadingProgress pattern.

Verification: oracle cargo fmt --check exited 0 (silent); offline Clippy --all-targets
with -D warnings completed in 0.32s; all five oracle tests passed in 35.58s, including
the unchanged M0 provenance test. Offline release build completed in 1m 22s.
git diff --check passed. No production crates changed during this foundation cycle.

M2 remains incomplete: unit identity needs package/target kind (not root path alone);
type-position resolution must filter namespaces, scoped generic/self/associated paths need
coverage, and the PRIMARY cache/configuration manifest plus SECONDARY/KL1 export remain pending.
No S5 measurements or final semantic-oracle capability are claimed.

### M2 oracle scoped signatures — STUB

Added a typed package/root identity index and compilation-scope record seam. Its deliberate
todo! will be exercised by the next RED fixtures. Intended coverage: a shared lib/bin root
must retain two compilation identities; repeated modules need distinct source/target scopes;
type-position paths must not return value/macro namespace answers; shared generic/import
paths must resolve in the actual crate or fail closed. Advanced cache/cfg support remains pending.
Offline cargo check completed in 0.35s. This phase intentionally replaces the base scope seam
with todo!; base tests will be restored during the scoped-signatures GREEN.

### M2 oracle scoped signatures — RED

Five new minimal fixtures exercise same-file lib/bin identities, namespace separation,
same-file repeated module occurrences, shared-file generic parameters, and nested grouped
imports across distinct dependency aliases. Targets must retain their own compilation/module
scope, not merely a physical path/range. Focused offline test output:

```text
panicked at src/primary-path-export-rows.rs:193:5:
not yet implemented
test result: FAILED. 0 passed; 5 failed; 5 filtered out; finished in 47.58s
```

All failures reach the deliberate scope-record seam. GREEN must satisfy these assertions
and restore the five previously green oracle tests; no assertion weakening is planned.

### M2 oracle scoped signatures — GREEN

Restored all ten tests: "ok. 10 passed; 0 failed" in 36.24s. Package identities come from
Cargo's IDs; sites and targets now retain target name/kind, edition and module path. The
same lib/bin source remains two units and repeated source files remain distinct module
occurrences. Grouped use prefixes are resolved through the current Module, and generic
parameters through that module's native GenericDef/Impl parameter lists. No file IDs were
duplicated. Use native Semantics for unique source containers, not an unnecessarily broad
module-only lookup; shared containers stay module-scoped and fail closed when unsupported.

No production dependency or storage change. Sysroot identity labels remain provisional
(sysroot+root#name); full configuration/compatibility manifests and production sysroot
materialization must settle these before corpus scoring. Next: refactor gates, import
multi-namespace and associated/Self fixtures, then exact-unit cfg and proc-macro artifacts.
The corpus selection must use all mapped normal units, not just workspace-member roots.
S5 is still unmeasured; the oracle build manifest still does not claim complete semantic support.

### M2 oracle scoped signatures — REFACTOR

Extracted target selection and the common native source-node projection into small helpers;
source-context multiplicity is computed once per module. Fixed Clippy's unnecessary lazy
evaluation in the cheap Module fallback without changing behavior.

Oracle formatting check exited 0; offline --all-targets Clippy with -D warnings completed
in 0.41s; all ten tests passed in 47.47s; offline release build completed in 1.79s.
git diff --check passed. Production dependency inputs are unchanged, so previous production
dependency gates are not rerun or newly claimed. No corpus agreement results yet.

### M2 oracle complete signatures — STUB

Introduced namespace-aware native resolution and shared-context associated-path seams,
deliberately todo!. RED will cover multi-namespace imports, Self/associated types and a
single-inline-module source-container collision. Public Type::iterate_path_candidates
requires a SemanticsScope; do not pass a first-crate scope as if it selected the current module.
Prefer native Module/GenericDef/Trait items, and only duplicate IDs if the public API proves
insufficient. Existing tests are intentionally red again until this cycle's GREEN.
Offline cargo check passed in 0.32s; no production dependency changes.

### M2 oracle complete signatures — RED

Five new fixtures fail at the intended namespace/associated-path todo seams:
test_complete_import_namespaces, test_complete_associated_types,
test_complete_trait_self, test_complete_method_generics, test_complete_inline_identity.
They cover constructor type/value namespaces, macro/type imports, T::Assoc and fully
qualified associated types across two crates, implicit Self, impl-method type parameters,
and an inline module covering its entire physical source file.

Focused offline test output: "FAILED. 0 passed; 5 failed; 0 ignored; 0 measured;
10 filtered out; finished in 46.26s". No assertions were weakened. GREEN must restore
all earlier tests as well; corpus comparison is not yet runnable.

Namespace RED clarification: preserve a single target definition with a sorted
target_namespace_kind_values set, rather than duplicate a tuple-struct definition for
its type and constructor namespaces. This keeps existing symbol-set contracts while
retaining both bindings. The strengthened fixture requires one Item target with both
namespaces and permits a trait plus macro of the same spelling. It still fails at the
namespace todo: "FAILED. 0 passed; 1 failed; ... 14 filtered out" in 23.83s.

### M2 oracle complete signatures — GREEN

All fifteen oracle tests pass: "ok. 15 passed; 0 failed" in 69.67s.
Native resolve_path_per_ns supplies import namespaces; target normalization merges
namespace sets for a single definition. Shared contexts use current-module GenericDef,
Trait items/supertraits and impl/trait-associated generic owners for the tested T::Assoc,
fully-qualified paths, Self and method-parameter cases. Source-container keys include
syntax kind so one inline module cannot masquerade as a shared file merely by covering
the same byte range as its SourceFile. No IDs were duplicated.

This is the tested subset, not complete corpus support: inherited method-only bounds,
shared local items/concrete associated lookup and expansion provenance still need exact
handling or a documented fallback. Native TypeParam::trait_bounds explicitly omits added
method bounds (hir/src/lib.rs); missing shared lookup fails rather than borrowing another
crate's Semantics. Full-unit cfg manifests, proc-macro cache, SECONDARY, production
resolution and S5 are still pending. Next: refactor gates and exact configuration inputs.

### M2 oracle complete signatures — REFACTOR

Unified namespaced-target construction. Associated-path lookup is a total Option
projection (it has no error branch), with ? for missing qualifiers; error propagation
remains in the source/identity adapters. Formatting check exited 0. Offline --all-targets
Clippy with -D warnings finished in 0.50s; all fifteen tests passed in 72.31s; offline
release build finished in 2.50s. git diff --check passed. No production inputs changed.
Remaining work above is unchanged; passing fixture tests do not constitute S5 evidence.

### M2 row-based name resolver — STUB

Added L1 scope/namespace/import/path/expansion rows, typed stage errors and the
parseltongue-port-name-resolver L2 boundary. It depends only on L1. Workspace membership
and L1 shared rows change because this persisted stage cannot be expressed without them;
no parser or hosted analysis dependency enters the engine. The macro port is synchronous.
The entry point is deliberately todo!. Offline cargo check passed in 0.27s.

The algorithm adaptation is grounded in the pinned hir-def collector's resolution_loop,
resolve_imports and resolve_import: settle imports, expand/reseed, repeat, enforce 8192
changing macro rounds, and return a typed error instead of publishing partial output.
Record native source notices. Next RED: namespace/alias/glob cycles and precedence,
edition/prelude rules, visibility, deterministic output, cancellation/input errors,
macro/import interleaving and the attempted 8193rd-round limit. Parser/MBe adapters,
atomic persistence, cfg/cache inputs, lang-item discovery and all-five S5 remain pending.

### M2 row-based name resolver — RED

Eight pure-row contracts fail at the intended entry-point todo: namespace aliases and
order independence; cyclic/diamond globs, ambiguity and explicit precedence; visibility;
generic-parent vs module/prelude boundaries; 2015 vs 2024 extern paths; empty/inactive
rows; invalid scopes/cycles/cancellation; macro-generated imports; exactly 8192 changing
rounds and rejection on attempted 8193. Output: "FAILED. 0 passed; 8 failed" in 0.00s.
The macro ports here are deterministic test doubles, not the future MBe/parser adapter;
these are not oracle-agreement or atomic-Turso-publication results.

GREEN implementation in progress: namespace-binding-normalization-rules.rs is not yet
wired and has not been checked. Intended engine: canonical namespace/priority target sets;
recompute import closure from declarations on each pass so newly explicit bindings remove
stale glob-derived aliases; accumulate sorted unique import evidence, not unbounded cycle
paths. Detect import-state cycles with Brent's algorithm (two snapshots, no arbitrary
8192 import-round cap); only changing macro rounds use FIXED_POINT_LIMIT.

Lookup should operate per namespace independently: nearest generic parent lookup stops at
module boundaries, then edition-appropriate extern/implicit preludes; explicit self/crate/
super paths bypass preludes. Intermediate segments require one type-namespace child scope.
Visibility is checked from the original requester. More than one target in one namespace
is Ambiguous with candidates, never extracted; Any may resolve one target in each namespace.

Settle imports before each macro callback; drop completed invocations from pending rows
while retaining completed keys. Canonically merge new scopes/symbols/imports/paths/semantic
outputs and detect conflicting same-key payloads. Completion-only bookkeeping is not a
changing macro round. Allow an unchanged callback after 8192 changing rounds; reject a
changing attempted 8193rd before returning any output. The budget fixture deliberately
uses only one pending invocation per round, so do not retain/clones all old request bodies.
Validate scopes, parent cycles, references and cancellation before lookup; publish nothing
here (the later L3 adapter must atomically write the complete per-crate result).

### M2 row-based name resolver — GREEN

Wired normalization and implemented resolve_crate_name_bindings through a pure round
engine and namespace-aware path lookup. Imports settle before macro callbacks; cyclic
glob evidence is deduplicated; explicit bindings supersede globs; ambiguous candidates
never become extracted answers. Parent lookup, visibility, extern/implicit preludes and
2015 use-root rules are explicit. Scope validation and cancellation fail with typed errors.
Macro completion keeps only pending request bodies and rejects changing round 8193;
no partial result escapes on errors.

Offline resolver tests: 8 passed, 0 failed, 0.26s (including both sides of the 8192
boundary). These are row-level algorithm contracts, not a real parser/MBe implementation,
Turso publication proof or oracle comparison. Next: refactor gates, followed by adapters.

### M2 row-based name resolver — REFACTOR

Restricted the lookup state's input field to its module and matched the repository's
test-only unwrap lint exemption. The first Clippy run rejected six unwraps in tests;
production code had no lint errors. After that focused correction: cargo fmt --all
--check exited 0; offline all-targets Clippy with -D warnings finished in 0.34s;
resolver tests: 8 passed, 0 failed, 0.26s; build finished in 0.09s. Core identity tests
also passed (3/3). Existing dependency/runtime gates passed (3/3, 0.20s), including
the zero-dependency L1/no hosted analysis dependency check. No real MBe expansion or
corpus agreement is claimed. Next: persisted parser projection and publication adapters.

### M2 persisted stage wiring — STUB

Added the synchronous L3 run_persisted_name_resolution seam, called after persisted syntax
extraction and before workspace publication. New CLI generations declare required stages
in the persisted workspace configuration; this also distinguishes M2 configuration from M1.
The adapter remains an intentional todo. Offline CLI cargo check passed in 0.59s.
Next RED: persisted resolved paths, per-crate atomic replacement/rollback, declared-stage
publication rejection, empty/cancelled/error cases, and lib/bin/module occurrence identity.

### M2 persisted stage wiring — RED

New CLI fixture reaches persisted source/syntax stages then fails at the adapter todo:
0 passed, 1 failed (0.34s). It requires alias-path answers and original target/module identity
in path-resolution-site-records and a completed name-resolution banner.
The new Turso fixture fails before schema writes because publication incorrectly succeeds
without its declared name-resolution stage: 0 passed, 1 failed (0.08s). It additionally
requires one atomic crate transaction spanning bindings and expanded spans, rollback of
duplicate-key writes, and unchanged old publication until the complete new stage is sealed.

### M2 persisted stage wiring — GREEN (initial source projection subset)

The L3 adapter now reads persisted graph/configuration/file bytes and completed dependency
scopes/bindings, constructs per-unit parser-derived scopes, symbols, imports and signature
paths, resolves dependencies library-first, and commits each complete crate in one Concurrent
transaction. JSON codecs preserve full unit/occurrence digests without adding L1 dependencies.
Turso owns five new M2 tables, permits crate-namespaced expanded syntax/file/request rows in
the same atomic write, and rejects publication missing any declared stage completion marker.
The CLI invokes the adapter and derives its name-resolution banner from the persisted marker.

Checks: fmt check exited zero; focused Clippy -D warnings finished in 0.53s.
CLI 6 passed (1.36s), syntax 11 passed, resolver 8 passed (0.26s), core 3 passed.
Turso stage contracts 15 passed (4.54s); dependency/runtime gates 3 passed (0.20s);
adapter MVCC smoke 1 passed and persistence profiles 2 passed / 3 ignored.
Existing include-expression and build-script-reason fixtures still pass; expression macros
are not incorrectly sent to an item-only expander. The old M1 banner assertion now requires
the explicitly implemented name-resolution stage rather than its retired placeholder.

This is NOT M2 completion: the current item-macro port intentionally returns a typed error;
no real MBe, proc-macro cache, sysroot/language-item projection, where-bound/inherent impl
completion or expanded rows are claimed. Macro/expanded/lang tables are schema/publication
support only. Parser projection still needs integration with persisted span/field evidence.
No production corpus mapping/S5 verdict has been attempted with this incomplete adapter.
The external current-harness PRIMARY collection failed on all five with exit 101:
"Tried to use attached db, but no db is attached". A serde diagnostic backtrace reaches
Semantics::resolve_path_per_ns -> analyze_impl -> infer_query -> infer_pat -> resolve_variant.
The pinned HIR exports attach_db; add an enum-pattern/local-use regression and use its
documented attached context before rerunning the oracle. Those failed exports are artifacts,
not measurements of agreement; no denominator exists.

### M2 persisted stage wiring — REFACTOR (initial source projection subset)

Replaced stringly typed syntax-kind dispatch with the pinned parser's SyntaxKind enum and
extracted the pure declaration-kind/namespace classifier. Parser trait aliases use TRAIT,
not a nonexistent TRAIT_ALIAS enum variant; the prior string arm was unreachable.
Scope labels and stored rows retain their existing representation. No feature expansion.
Fmt check exited zero; focused Clippy -D warnings finished in 0.53s. CLI contracts:
6 passed (1.37s); syntax contracts: 11 passed. The GREEN limitations above still apply.

### M2 oracle attached-database regression — RED

A minimal local use of E::A inside an enum-pattern function reproduces the all-five
PRIMARY export crash: test_local_enum_path_inference, 0 passed / 1 failed (28.57s),
at the pinned hir-ty interner's "Tried to use attached db, but no db is attached".
This verifies a reachable inference path, not a speculative dependency issue.
Environment diagnosis: active/default stable cargo/rustc 1.99.1 is loaded. The repeated
upstream "unused manifest key lints.cargo" warning comes from the pinned workspace's
nightly-only Cargo lint table, not an unloaded repo environment. No SDK config changed.

### M2 oracle attached-database regression — GREEN

The external exporter now runs its semantic traversal inside the pinned HIR attach_db
context. No production dependency or SDK file changed. All 16 oracle tests passed
(60.89s), release build finished in 3.02s, Clippy -D warnings in 0.93s; fmt check passed.
The same corpus probe now exits zero on all five. Raw original-source path counts:
rust-analyzer 88104; ripgrep 7847; tokio 22274; serde 7610; cargo-modules 1369.
These are current-harness export counts, NOT S5 denominators, rates or gate results.
Artifacts: each repo's m2-primary-path-exports directory; original failed outputs preserved
as m2-primary-unattached-exports. The probe records executable SHA-256 and pending
configuration/proc-cache/expanded-provenance qualifications. S5 comparison is still pending.

### M2 declarative macro expander — STUB

Added parseltongue-port-macro-expander with the specified pure L2 source-in/source-out API,
typed L1 expansion input/output/error and per-token definition/invocation provenance records.
The stub remains todo; offline cargo check passed (0.30s). No salsa/analysis dependency.
Copy-adaptation source notice pins the matcher/transcriber and parser-input bridge.
Source inspection found another span portability constraint: with salsa disabled, the pinned
span's opaque SyntaxContext exposes only an unsafe constructor. Preserve the no-unsafe rule
by using a compatible minimal safe span representation if TT needs it; do not enable salsa.
RED will cover item/repetition/fragment expansion, empty output, malformed/no-match errors,
token provenance, edition boundaries, cancellation and the explicit no-salsa macro tree gate.

### M2 declarative macro expander — RED

All eight behavior fixtures fail at the intended entry-point todo; the macro-specific
dependency-tree gate already passes. Summary: 1 passed / 8 failed (0.06s).
Fixtures cover macro_rules arm selection, macro-2 input, nested/optional repetition,
Rust type/expression/path/item fragments, empty output, invalid definition/invocation and
no-match errors, definition-versus-invocation token ranges, 2021/2024 expression semantics
and cancellation. Generated normal-case item text must parse with the pinned parser.

### M2 declarative macro expander — GREEN

The pinned matcher/transcriber/parser-input files have been mechanically copied by
tools/adapt-macro-expansion-sources.py into four-word source paths. Database arguments
became explicit edition callbacks. A pure lexer/TT facade now invokes the copied engine and
renders source plus per-token original/output byte ranges. All nine contracts pass (0.06s),
plus the copied parser doctest. Core's three identity tests and the three existing runtime/
dependency gates pass. The complete production workspace dependency gate retains L1 zero
dependencies and no salsa/HIR/IDE/base-db. No unchecked unwrap/expect or unsafe block in the
port. Cancellation and malformed/no-match failures return typed errors without partial output.

To avoid unsafe construction of the upstream salsa-free opaque SyntaxContext, a minimal
compatible ra_ap_span package lives in crates/parseltongue-macro-span-records and is patched
through the root Cargo manifest. It has safe span/file/context primitives and no salsa,
using only the already-pinned parser's Edition/TextRange/TextSize. TT/intern/stdx/parser
are pinned at 0.0.362; cargo fetch provisioned TT/intern/cov-mark dependencies successfully.
No upstream checkout was modified. TT compiles against the small compatibility surface.

One newly authored RED expectation was corrected against the pinned transcriber: nontrivial
expr fragments are deliberately parenthesized (wrap_in_parens in the copied transcriber) to
preserve precedence. The fixture now expects that authoritative expansion, rather than
changing the algorithm to drop protective parentheses. No existing pre-M2 test was altered.

Initial Clippy identifies two REFACTOR items in copied matching code: 11 scratch arguments
and a single-pattern match. Next: group matcher scratch state, simplify the branch, verify
span record primitives and finalize the pure engine. Persisted item expansion, hygiene-aware
virtual files, built-ins/sysroot/lang items and the differ/S5 remain pending. The existing
DB-backed item port still explicitly errors; no successful corpus mapper or S5 gate is claimed.

### M2 declarative macro expander — REFACTOR

Grouped borrowed NFA scratch references into MacroMatchingScratchState and simplified the
single-pattern branch, without changing arm or repetition ordering. Added final cancellation
check after rendering, safe span identity/edition roundtrip and cross-anchor/fixup join tests,
plus expression precedence, $crate definition origin and UTF-8 token-range fixtures.

Verification after the last Rust edit: fmt --all --check exited zero; focused Clippy with
all targets and -D warnings finished successfully (1.01s). Macro contracts 12 passed (0.10s);
span compatibility 2 passed; copied parser doctest 1 passed. No unchecked unwrap/expect,
salsa reference or unsafe block in the copied production modules or safe span component.
The earlier L1 identity (3 passed) and existing dependency/runtime gates (3 passed) have
unchanged inputs since their successful run; the macro tree gate also passes in this suite.

Next integration boundary: replace PendingItemMacroPort in
crates/parseltongue-syntax-span-extractor/src/name-resolution-stage-adapter.rs with a
pure-source adapter around expand_declarative_macro_items. Its token origins distinguish
definition versus invocation source and retain byte ranges; the adapter must join them to
real compilation units, module scopes, invocation IDs and persisted span/field evidence.
Do not treat plain rendered $crate as ordinary parseable source: retain its definition crate
through the syntax bridge/normalization. Add virtual file/provenance and expanded evidence
rows atomically with the crate fixed point. Built-ins, sysroot/lang items, complete generic/
impl projection, exact PRIMARY/SECONDARY configuration and differ/S5 still remain.

### M2 persisted macro adapter — STUB

Replaced the PendingItemMacroPort wiring with a typed PersistedItemMacroPort owning the
source projection behind a mutex (the expansion trait is Send + Sync). Expansion remains
todo until RED. The resolver now supplies resolved macro bindings using its existing
namespace/path lookup after import settling; expansion does not duplicate lookup policy.
The stage retains its existing one-transaction-per-crate publication boundary.
Offline cargo check for the syntax extractor passed (2.48s). Next RED: generated item/path
answers, nested invocation provenance, cross-crate $crate hygiene, empty expansion, expanded
span/field rows and failure preserving the previously published generation.

### M2 persisted macro adapter — RED

Three CLI/store contract tests cover nested expansion chains and expanded span/field
evidence, cross-crate $crate with same-named caller items, distinct virtual files for
two invocations, empty expansion and failure preserving the published generation.
The existing span_expansion_provenance_chain spelling is used for evidence assertions.
The RED run printed: test result: FAILED. 0 passed; 3 failed; 6 filtered out.
All three failures reached the intended persisted adapter todo after syntax-spans.

### M2 persisted macro adapter — GREEN

The resolver's settled macro targets now feed expand_declarative_macro_items. Source
definition metadata survives library publication/reexports. Expansion creates stable
invocation-keyed virtual files, full invocation chains, absolute token-origin evidence,
expanded spans/ownership/fields/body sites, names, imports and later macro requests.
$crate is normalized only at recorded token positions, with its defining unit retained
separately; original bytes/ranges remain authoritative, including syntax-walker evidence.
All crate rows publish through the existing atomic name-resolution unit transaction;
immutable content bytes can be written earlier but are not visible as published files.

CLI contracts: 8 passed / 1 failed initially. The failure was the newly authored
whitespace-sensitive $crate source-text assertion, not resolution: the renderer emits
spaces between tokens. Normalizing that comparison produced 1 passed / 8 filtered.
The assertions still require both targets to be the definition crate, not caller names.
Affected tests: core 3, resolver 8, syntax 11, adapter 2 (+3 ignored profiles), store
contracts 15, dependency/build gates 3 all passed. Focused all-target Clippy -D warnings,
fmt check and diff check passed. A rustc lab probe rejected a captured type used in a
new nested module, confirming not to override ordinary captured paths to caller scope.

Next REFACTOR: factor token provenance and projection offsets; add further hygiene
contracts. Built-ins, sysroot/preludes/lang items, exact oracle units and S5 remain pending.

### M2 persisted macro adapter — REFACTOR

Replaced positional projection counters with typed named offsets. Added a two-crate,
two-round regression with a captured UTF-8 identifier and $crate propagated through a
generated macro definition. It resolves the original foreign type despite a caller
shadow with the same spelling, and records both invocation IDs.
CLI contracts: 10 passed; syntax contracts: 11 passed. Focused all-target Clippy
with -D warnings, formatting and diff checks passed. No new subprocess path was added.
The pure engine's source/provenance remains the bridge; all expanded evidence is committed
with its crate fixed point. Built-in item support is next, then sysroot/preludes/lang
items, exact mapped oracle units and all-five S5. No agreement rate is claimed yet.

### M2 built-in macro arguments — STUB

Added dependency-free typed built-in inputs/results and a pure expansion signature in
the macro port. Nested macro lookup is an explicit read-only port so a user macro named
concat cannot silently be treated as the compiler builtin. Environment rows distinguish
known-present, known-absent and unavailable data; this avoids pretending option_env is
None merely because an arbitrary process environment key was not supplied.
Offline macro-port cargo check passed in 0.42s. Next RED covers supported builtins,
metadata-only environment inputs, scope shadowing, malformed/empty arguments and stage
include publication. No filesystem/process/environment access is added to L2.

### M2 built-in macro arguments — RED

Pure tests: 0 passed / 6 failed, each at the intended todo (12 declarative tests
filtered). CLI include and unavailable-source contracts: 0 passed / 2 failed, showing
missing included names and missing build_script evidence. A separate type-macro contract
fails because the item adapter currently tries to parse a type expansion as items; GREEN
will constrain item callbacks to item positions rather than change type semantics.
The test cancellation method spelling was corrected before collecting the RED results.

The metadata extension adds a failing graph contract: CARGO_PKG_AUTHORS is Null rather
than First:Second; the same case covers homepage, repository, license/readme paths,
rust-version and prerelease/build metadata separation. This is RED before the loader edit.

### M2 built-in macro arguments — GREEN

Pure expansion supports cfg, concat, stringify, env and option_env; include returns a
path request to L3. Nested builtins use settled macro-namespace bindings, not text-only
dispatch. Unsupported macros and unavailable environment inputs have explicit reasons;
OUT_DIR is build_script. Known absence is distinct from unavailable process environment.
The graph loader supplies documented CARGO_PKG fields, including authors and correct
prerelease/build separation. No arbitrary process environment values are read.

The persisted adapter expands includes from immutable ingested bytes, preserves caller
cfg/unit context, records virtual source and expanded evidence, and emits final unresolved
macro rows only after resolution converges. Relative include bases preserve the original
call-site source directory across virtual expansions. A rustc lab with separate physical
definition/caller files rejected a definition-directory target and accepted the caller
target (metadata output under /home/ubuntu; /dev/null required an unwritable /dev temp).
The probe does not establish a separate cross-crate case.

Item callbacks now exclude type macros. Builtin prelude rows are seeded only when an
item invocation is visited: an initial eager seed broke the empty baseline (25 versus
19 bindings); lazy seeding restores the unchanged existing contract, without editing it.
The new test module follows the existing test-only unwrap lint allowance.

Checks: pure macro 18 tests + 1 doctest; CLI 13; graph 9; resolver 8; syntax 11 all pass.
Affected all-target/all-feature Clippy -D warnings passed in 3.00s; fmt and diff checks
are clean. Logs: /home/ubuntu/m2-builtin-*-green*.log. Builtin argument nesting over 128
is explicitly unavailable, not invented output. Next: refactor argument parsing and
expand edge/shadowing checks, then sysroot/preludes/lang items, exact oracle manifest,
differ and all-five S5. No S5 measurement exists.

### M2 built-in macro arguments — REFACTOR

Replaced positional splitter state with named fields and kept char/string values
distinct. Added raw cfg strings, optional trailing commas and comment-only tails.
The added pure edge tests first failed (6 pass / 2 fail), then passed with the fixes.
Added persisted call-site include-base and nested user-macro shadowing regressions.
The include-base assertion initially used the DTO rather than stored JSON field name;
corrected to path_target_symbol_rows after checking the codec. No production fix was
needed for that assertion.

Final checks: macro port 20 tests + 1 doctest passed; CLI 15 tests passed (1.96s).
Macro/CLI Clippy passed (3.20s); final changed CLI test Clippy passed (1.22s).
Formatting and diff checks passed. Earlier graph/resolver/syntax passes remain valid;
their implementation inputs were unchanged in this refactor. Reindexed the repository:
26,869 nodes / 58,402 edges; six unrelated query files have best-effort partial parses,
zero unusable files. Sysroot/preludes/lang items are next. No S5 rate is claimed.

### M2 sysroot/prelude contexts — STUB

Added typed no_std/no_core/implicit-prelude input and signatures for sysroot graph
extension, source-context flags, demand detection and pinned lang-item recognition.
Offline graph/syntax check passed (1.06s); final syntax signature check passed (0.34s).
The active toolchain is /home/ubuntu/.rustup/toolchains/stable-x86_64-unknown-linux-gnu;
matching rust-src/library contains core, alloc and std (all editions read from Cargo.toml).
An assumed version-named directory did not exist; the returned sysroot path does.

Implementation plan: detect standard-library demand from reached closure files (not
unreachable byte inventory) before final generation preparation. If needed, extend the
graph with semantic-support sysroot units, preserving target cfg and independent
features/editions. Restart staging in a separate scope before preparing the final
generation so no old closure-unit rows can leak after graph augmentation. Ingest the
library source tree as bytes; only active module roots enter semantic closure. Sysroot
units support resolution but are excluded from mapped user-unit S5 denominators.
Empty/local-only inputs need not instantiate std; existing empty contracts remain valid.
Prelude imports honor crate-level cfg_attr, no_std, no_core and no_implicit_prelude.
Lang-item rows use the pinned hir-def recognized names, never a hard-coded count of 169.

### M2 sysroot/prelude contexts — RED

Graph extension: 0 pass / 3 fail at todo (target cfg/edition separation, missing matching
sources, empty input). Syntax: existing 11 pass / new 3 fail at todo (crate flags with
cfg_attr, lexical demand ignoring strings/comments, recognized lang names including fn).
CLI native-sysroot fixture fails on unresolved Option; initial stored-scope-field spelling
was corrected against the codec before collecting the authoritative RED result.
The pinned lang_item.rs sym-row scan currently returns 170 names; fn_ is the Rust-symbol
alias for the attribute spelling fn. Do not use the SPEC's earlier 169 observation as a
required count. std Cargo.toml currently has no default feature declaration.

GREEN continuation details: extend_sysroot_crate_graph lives in graph-loader's new
sysroot-compilation-context-rows.rs (still todo). The graph loader already depends on
toml 0.9, so parse the library package manifests without a new dependency. Core/alloc/std
units have unit_sysroot_context_flag=true and unit_mapped_target_flag=false; syntax/name
stage selection and ingestion must admit semantic-support units explicitly. Ingestion
must walk their common library parent to inventory stdarch/portable-simd sibling sources,
while preserving each actual package root for source identities. Existing M1 closure
discovery still decides which files enter semantic contexts.

After the first reached-source discovery, scan only syntax-file-work-requests bytes for
standard/prelude names. If needed, build crate-level contexts from each mapped root file,
augment the graph, and restart graph/ingest/discovery in a distinct staging scope before
compute_workspace_generation_digest; do not mix old closure rows with augmented units.
The final generation must hash the augmented graph and sysroot source inventory. Keep
the original Cargo unit keys stable when adding implicit system links. System units form
core -> alloc -> std dependency order (std depends on core and alloc); normal units use
flags for implicit std/core and explicit extern crates. Register aliases/edition prelude
glob imports before resolution. The existing adapter currently has no ExternCrate AST
visitor, so add it for explicit 2015/alloc aliases and nested declarations. Handle
no_implicit_prelude without suppressing language primitives: builtin-priority lookup
needs to remain available even when standard prelude lookup is disabled.

Lang rows already have a declared Turso owner (lang-item-definition-table, name-resolution),
so no new table/config is needed. Collect recognized active lang attributes during syntax
namespace projection (including non-public items and expanded source), link source/range,
unit/provenance and target identity, then publish with the crate's existing atomic rows.
No GREEN implementation or corpus run has occurred for this subphase yet.

### M2 sysroot/prelude contexts — parser compatibility RED blocker

GREEN implementation started: sysroot graph and source-context helpers, semantic-support
unit ingestion/discovery, separate augmented staging, foreign prelude bindings, explicit
extern-crate aliases, and atomic per-crate lang-item output. These implementation edits
remain UNCOMMITTED and are not accepted GREEN. Graph tests passed (9 existing + 3 new);
syntax tests passed (11 existing + 3 new) before adding the blocker regression.
The CLI suite's first implementation run had 15 existing passes / 1 new failure.
Fixed a TOML API mismatch (Value::from_str parses a value, toml::from_str parses a document).
Also corrected the newly authored CLI assertion to use full generic path source text;
path sites preserve Option<Vec<u8>>, not just Option. No existing test expectation changed.

The native integration then reached actual matching rust-src and failed during closure
discovery, BEFORE final generation preparation, on core/src/clone.rs offset 14967:
WorkspaceNotCompilingError: expected an item. rustc --version is
rustc 1.99.1 (58cf8f9c3 2026-10-13), and rustup component list --installed includes rust-src.
The returned sysroot is /home/ubuntu/.rustup/toolchains/stable-x86_64-unknown-linux-gnu.
Its core/src/lib.rs line 273 unconditionally declares pub mod clone; line 131 enables
final_associated_functions. clone.rs lines 382–392 define Share with final fn share.
This is a reached standard-library source, not an unreachable fixture or inferred path.

Disproof check: reduced the source to an ordinary trait method and extraction passed.
Adding only final reproduced expected-an-item at byte 18. Then called the pinned SDK
SourceFile::parse directly, bypassing our compatibility wrapper: it also returned
SyntaxError("expected an item", 18..18). The newly added test_sysroot_final_trait_methods
records this failing requirement. Thus the observed native failure is not explained by
our wrapper, TOML reader or new prelude code. Syntax and parser dependencies are both
exactly =0.0.362. No parser recovery, token stripping, dependency-pin substitution or
external rust-analyzer checkout edit has been made.

Evidence:
- /home/ubuntu/m2-sysroot-pure-green.log: graph 9+3 pass; syntax 14 pass.
- /home/ubuntu/m2-sysroot-native-green.log: native fixture fails in 38.81s on clone.rs.
- /home/ubuntu/m2-sysroot-parser-blocker.log: wrapper reduction fails at byte 18.
- /home/ubuntu/m2-sysroot-parser-direct-blocker.log: direct pinned SDK fails at byte 18.

REQ-RUST-004.2 explicitly makes pinned-parser/toolchain incompatibility block corpus
acceptance. Decision needed: authorize focused parser compatibility work for Rust 1.99
final associated functions, including review of its effect on the separately pinned
oracle. Do not silently ignore the token or report this as a noncompiling user workspace.
After that decision: finish native sysroot integration, add edition/no_std/no_core/
no_implicit_prelude/extern alias integration cases, improve demand detection to cover
all prelude identifiers (AsRef/AsMut and memory functions are currently missing), run
affected tests and Clippy, then GREEN/REFACTOR commits. Exact oracle manifests, differ,
all-five S5 numbers and disagreement categories remain unstarted; no S5 rate exists.

# Parser/oracle source alignment — 2026-11-11

- The earlier checkout's item grammar has no final modifier. Fetched upstream history
  (unshallowed before history inspection); newest upstream master is
  2a362a41a8eb6e090f1a12d9b712df85af6f0c33, dated 2026-11-10.
  Its grammar explicitly consumes final and includes final_assoc_fn.
- Updated the corpus/oracle pin and linked production parser and syntax directly to
  that same clean checkout, including macro parsing. No token patch or checkout source edit.
  A shared build verifier checks revision and tracked cleanliness; generation identity
  includes the parser manifest; a dependency gate checks both manifest paths.
- Native checkout qualification: cargo fetch --locked exited 0; cargo check --workspace
  --locked printed Finished dev profile in 32.16s. Direct native-source syntax parse
  and extractor regression: test_sysroot_final_trait_methods passed.
- M1 requalification uses the explicit --mapping-final-stage-kind syntax-spans;
  default remains name-resolution. It skips M2 sysroot augmentation and resolver work,
  and the banner honestly leaves name-resolution not yet implemented.
- Working-tree workspace tests excluding the pending native sysroot integration:
  102 passed; 0 failed; 3 manual performance tests ignored; 1 M2 case filtered.
  The parser-source gate passes and production still has zero L1 dependencies and no
  semantic-analysis/salsa packages. Oracle tests: 16 passed; 0 failed.
  Production Clippy all targets/features with -D warnings passed; Python contracts:
  Ran 5 tests in 0.320s / OK. Release build completed in 14.98s.
- The repeated upstream warning "unused manifest key lints.cargo (may be supported in
  a future version)" was diagnosed against the blueprint, rustup active-toolchain
  (stable default), and cargo 1.99.1: these optional upstream nightly Cargo lints
  are unsupported on stable. The environment is loaded; no missing dependency or
  degraded Rust build was found. Upstream manifests were not edited.
- Frozen executable /home/ubuntu/m1-parser-aligned-parseltongue is running the
  sequential all-five untraced measurements plus separate process and target audits.
  Driver log /home/ubuntu/m2-parser-aligned-corpus.log; evidence per repository at
  parser-aligned-m1-measurements. Count deltas remain pending until completion.
- Sysroot implementation remains separate unfinished working-tree work, not claimed
  GREEN. With final now accepted, native discovery advances to core/src/ops/try_trait.rs
  and fails at offset 16948 on a const move closure. A direct native syntax regression
  bypassing the wrapper reproduces expected fn, trait or impl at 14..14; removing only
  const parses. No workaround applied. Full reachability evidence and scope decision
  will be recorded after the required all-five M1 rerun.

## Parser alignment S3 completion and const-closure compatibility RED

- Parser source alignment GREEN is committed/pushed as 21535ea3b.
  Native checkout remains clean at 2a362a41a8eb6e090f1a12d9b712df85af6f0c33.
- All-five M1 measurements and separate audits are complete. Source tiling holds,
  all command exits are zero, and every process/target audit passes with the
  exact configured frozen executable. Peak memory is below 2 GiB on all five.
  See the appended report section for all counts/deltas/time/RSS and qualifications.
  Rust-analyzer deltas: files +5, spans +118, fields +207, body sites +885.
  All four other corpora have zero count deltas.
- The renamed frozen binary exposed a driver root-PID basename assumption.
  Corrected it to match the configured executable path exactly; all child/argv/network
  checks remain unchanged. Tests: Ran 6 tests in 0.326s / OK.
  Original traces were reanalysed, not rewritten. Serde's first capture contains an
  unfinished ???? event and remains failed; a fresh cold-cache audit passed.
- New RED test_sysroot_const_closure_support calls native SourceFile::parse directly:
  ordinary move tail and const-move let initializer pass; const-move tail fails with
  SyntaxError("expected fn, trait or impl", 14..14), 0 passed / 1 failed.
  Native default CLI fixture fails at core/src/ops/try_trait.rs offset 16948.
- Reachability is verified: selected sysroot's core lib enables const_closures;
  its unconditional ops module unconditionally loads try_trait; wrap_mut_1 has the
  exact tail closure. expressions.rs stmt dispatches items::opt_item before expr_stmt;
  items.rs consumes const whenever the next token is not a brace, misclassifying move.
  atom.rs closure_expr already supports const, as the let-initializer control proves.
  This is not a missing prelude binding or a Parseltongue token-wrapper defect.
- Scope decision needed: authorize a small, jointly pinned parser/oracle source
  grammar fix for statement-position const closures. The newest upstream master
  alone cannot parse this selected toolchain's core. Do not strip tokens, suppress
  errors, or silently select an older sysroot. No external checkout edits made.
- Sysroot GREEN/REFACTOR and manifests/differ/S5 remain incomplete. Preserve the
  seven existing uncommitted sysroot files. No agreement numerator/denominator exists.

## Authorized const closure grammar GREEN — 2026-11-12

- Local upstream fork branch: devin/1794451883-statement-const-closures.
  Base 2a362a41a8eb6e090f1a12d9b712df85af6f0c33; patch
  bf6b2099d6351c37039b01447a9bb64f9a54a586. Parser and oracle pin that same clean fork.
- opt_item returns its unconsumed marker for const followed by move/pipe, before
  both modifier parsing AND the later const-item fallback. Merely excluding the
  modifier guard was insufficient: native tests exposed the fallback.
  No token rewriting, suppression, source substitution, or toolchain changes.
- Upstream inline fixtures cover const/move, empty/typed parameters, tails and
  semicolon statements, plus missing-body recovery. Generated success/error
  snapshots inspected: CLOSURE_EXPR owns const; error case still reports expression.
- Native parser: 333 passed; xtask: 13 passed; affected parser Clippy and xtask tidy
  exit zero. Existing stable-Cargo optional-nightly-lint warnings remain unchanged.
  Direct production const-closure regression: 1 passed.
  Oracle tests: 16 passed; release build succeeded. Manifest digest refreshed.
- Rustc evidence (AI policy): compiler/rustc_parse/src/parser/mod.rs at installed
  58cf8f9c35b355199727627d07ae34a4e2846d67, check_const_closure and
  parse_constness_ explicitly avoid parsing closures as const items; expr.rs
  dispatches them to parse_expr_closure. Citation is in the patch commit.
- Reviewable patch is tools/parser-const-closure-statements.patch. The 2.9 KiB
  tools/parser-source-fork-bundle.bundle publishes the exact fork git object without
  pushing to upstream or creating a PR. Fetch bundle HEAD after cloning upstream;
  git bundle verify requires the recorded base commit. Both artefacts are in-repo.
- Patched native sysroot integration advanced beyond parsing and remains running,
  paused while all-five untraced S3 measurements execute (SIGCONT 1091013 afterwards).
  It is not yet GREEN. Preserve its seven implementation files uncommitted.
- Frozen M1 binary: /home/ubuntu/m1-const-patched-parseltongue. Evidence suffix:
  const-patched-m1-measurements. Keep prior parser-aligned evidence unchanged.
  Next: finish S3, resume native sysroot integration, then sysroot GREEN/REFACTOR,
  exact oracle mapped units, differ and all-five S5. No S5 number claimed.
- Patched workspace tests: 103 passed, 3 manual profiles ignored; only pending
  native sysroot integration filtered. Fork cargo fetch --locked and cargo check
  --workspace --locked both exit zero; check reports 9.28s.
- Frozen executable SHA-256:
  f528d38b7e4b0aaadf2b83f4d5b9b9eb7336ed4e9462ff15846f3027a202be9f.
  Driver shell 190221; log /home/ubuntu/m2-const-patched-corpus.log.
- All-five patched S3 completed (driver exit zero): tiling, memory and both audits
  pass, zero process violations. Counts and deltas are in the human report.
  Native integration PID 1091013 SIGCONT-resumed after measurements. It has not
  completed; shell 787eb5, log /home/ubuntu/m2-patched-sysroot-integration.log.
  GDB attach was denied by ptrace policy; no privilege/sysctl workaround attempted.

## Sysroot integration continuation RED — 2026-11-12

- Native debug integration completed with no parser errors, then failed at
  StageInputMissingError: module repr. Its 2030.18-second wall time includes the
  deliberate S3 pause; syntax write timing includes that pause and is not profiling.
- Reachable case: core/src/io/error.rs selects #[cfg_attr(...,path=
  "error/repr_bitpacked.rs")] mod repr. M1 discovers it relative to the physical
  source directory; M2 incorrectly used the default non-mod.rs module directory.
  Portable CLI regression test_path_attributes_follow_source fails identically at
  module selected. Existing APIs unchanged; no new stub surface.
- Additional pure demand regression fails on AsRef (also covers AsMut and prelude
  memory functions). Installed core/prelude/v1.rs exports these names explicitly.
- Native source lang-attribute spelling inventory found additional names absent from
  the old recognition whitelist. Persist actual reached lang attributes rather than
  silently discarding newer compiler evidence; recognize support separately.
- Fixes remain uncommitted pending native GREEN; release integration shell 84b53b,
  log /home/ubuntu/m2-native-sysroot-release.log (no suspended mapper remains).

## Source macro documentation RED — 2026-11-12

- Source-relative path regression GREEN: 1 passed, 0 failed (0.53s).
- Native release integration now reaches macro expansion; fails with
  WorkspaceNotCompilingError: InvalidMacroDefinitionError: unexpected macro token ///.
  Parser accepts all reached native files; this is the adapted MBe input lexer,
  not another upstream parser defect.
- Focused test_macro_documentation_token_origins reproduces InvalidDefinition(
  "unexpected macro token /// hello"). New pinned parser exposes INNER_DOC_COMMENT /
  OUTER_DOC_COMMENT separately from trivia; the local lexer treated them as punct.
- Adapt pinned syntax-bridge convert_doc_comment / DocCommentDesugarMode::Mbe:
  emit proper doc attribute TT and raw-string token; retain original comment span
  for all derived tokens (definition or invocation). Do not strip comments or patch
  Rust source. Cover line/block, inner/outer, empty docs, quoting, meta matching,
  invocation provenance, unterminated comments and delimiter-limit errors.

## Source macro documentation GREEN — 2026-11-12

- Ported pinned syntax-bridge MBe doc conversion into the existing source-token
  facade, preserving full source-comment range and definition/invocation anchor.
  Raw delimiters are selected from contents and checked for lexer limits.
- cargo test -p parseltongue-port-macro-expander:
  test result: ok. 21 passed; 0 failed; plus 1 passing doctest.
- cargo clippy -p parseltongue-port-macro-expander --all-targets -- -D warnings:
  Finished dev profile in 2.15s. Existing upstream stable-Cargo manifest lint
  warnings remain diagnosed nightly-only warnings, not an environment failure.
- No additional upstream parser defect; parser/oracle fork pin unchanged.
- Native all-CLI release test shell e94f0c, log
  /home/ubuntu/m2-sysroot-cli-contracts-green.log. Graph/syntax shell 2b20d0, log
  /home/ubuntu/m2-sysroot-graph-syntax-green.log. Distinct fixture stores only.

## Native integration continuation checkpoint — 2026-11-12

- Pushes verified: ae0a9ad62 (patched S3), 702bd7032 (module/prelude RED),
  0069f4ee5 (doc macro RED), d67318e81 (doc macro GREEN).
- Last all-CLI release run: 17 passed, 1 failed in 133.29s. Only native
  test_lang_items_in_sysroot fails, now with WorkspaceNotCompilingError:
  WorkspaceNotCompilingError: invalid attribute source. Source discovery and
  extraction complete (618 contexts, 358595 rows). No further parser defect found.
- Leading explanation (not yet verified on the native failing attribute):
  MBe TT renderer emits legal whitespace between # and [. The extractor's
  collect_effective_node_attributes uses textual strip_prefix("#[") / "#![",
  rejecting legal token spacing. Focused test_attribute_token_spacing_rules is
  the next RED; replace textual delimiter stripping with ast::Attr::meta syntax.
  Keep source bytes/ranges unchanged. Then rerun native integration.
- Graph contracts: 9 + 3 passed. Syntax contracts before new spacing RED:
  16 passed. Macro contracts: 21 passed + 1 doctest; Clippy passed.
- Seven sysroot GREEN files remain uncommitted. They additionally fix source-
  relative path overrides, demand for AsRef/AsMut/prelude memory functions, and
  retention of reached lang attributes absent from the old recognition whitelist.
  Lang rows now carry provenance and recognized/unknown support separately.
- Sysroot GREEN/REFACTOR not committed. Exact oracle manifests, differ and S5
  remain unimplemented/unmeasured. No live mapper remains. Logs under /home/ubuntu/
  m2-sysroot-cli-contracts-green.log and m2-attribute-token-spacing-red.log.

## Native macro declaration continuation STUB — 2026-11-12

- AST metadata fixes legal spaced attributes; full syntax suite passes.
- Native integration advances to InvalidMacroDefinitionError: expected semicolon.
  Added source/byte diagnostics: core/src/marker.rs at byte 1726, marker_impls
  declared with macro (not macro_rules), multi-rule comma delimiters.
- The existing facade selects parse_macro2 only when a single-rule argument pattern
  exists, conflating multi-rule macro declarations with legacy macro_rules.
  Pinned MBe already implements parse_macro2(None, body) with comma delimiters.
- Add explicit macro_declaration_style_flag to the pure expansion input (required
  shared type change: optional pattern cannot represent this distinction). STUB
  constructors default false; next RED exercises declaration multi-rules.
- No further upstream parser defect: existing pinned grammar accepts the source.

## AST metadata attribute GREEN — 2026-11-12

- collect_effective_node_attributes now reads ast::Attr::meta, preserving source
  bytes/ranges and accepting legal whitespace around attribute punctuation.
- /home/ubuntu/m2-attribute-ast-green.log: syntax contracts 17 passed, zero failed.
  Native mapping advances through all 618 sysroot/user extraction contexts and
  core namespace projection; the old textual-delimiter error is no longer reached.
- Macro declaration GREEN uses parse_macro2(None, body) when the explicit syntax
  declaration flag is true. Pure contracts: 22 passed plus one doctest. The portable
  persisted CLI declaration case passes (1 passed, 18 filtered). Complete native
  publication is still under verification, not GREEN.
- Storage exhaustion was diagnosed: root filesystem 124 GiB, zero available.
  Safely removed only disposable dev build caches and an orphaned failed-test
  temporary store (cargo clean --profile dev for the five affected packages).
  Recovered 13 GiB without deleting corpus evidence or source fixtures.
- Native profile: /home/ubuntu/m2-native-indexed-profile.log; input fixture and
  diagnostic store remain /home/ubuntu/m2-native-resolver-fixture and
  /home/ubuntu/m2-native-resolver-store.db. No S5 denominator/rate exists yet.
- Refactor candidate replaces full-binding-vector scans for each glob import
  with the existing scope lookup index. Eight resolver contracts pass; Clippy
  passes. It does not change candidate visibility or precedence.

## Native prelude grouped-self RED — 2026-11-12

- Full release CLI suite: 18 passed, 1 failed in 1297.67 seconds. Mapping now
  completes and publishes, but Option<Vec<u8>> remains external_source_missing.
- Verified reachable source: both installed core/src/prelude/v1.rs:47 and
  std/src/prelude/v1.rs:41 import Option with Option::{self, None, Some}.
  Projection preserves grouped self as a terminal SelfModule path segment;
  L2 only handles SelfModule in the prefix, rejecting it in the name tail.
- New pure regression test_grouped_self_import_namespace fails Unresolved versus
  Resolved. It also requires self imports to select only the type namespace,
  not a tuple constructor's value namespace. A portable persisted alias fixture
  reproduces the same failure independently of sysroot loading.
- Diff review found a separate reachable configuration gap: no sysroot demand
  returns before crate attributes are persisted. A no_std/no_implicit_prelude
  crate using an explicit renamed dependency and u8 publishes null flags.
  Portable suite: 18 passed, these two new cases failed, native case filtered.
- Safely losslessly compressed historical m1-prepared-final-measurements and
  parser-aligned-m1-measurements store images/sidecars with gzip -1n. gzip -t
  passed. Latest const-patched stores, all logs, traces and JSON evidence remain
  unchanged. Restore any historical image with gzip -d FILE.gz. Available disk
  is now 32 GiB; diagnostic publication is about 4 GiB.
- Pinned parser checkout remains clean at bf6b2099d6351c37039b01447a9bb64f9a54a586.
  tools/parser-const-closure-statements.patch SHA-256 remains
  89146042a1c1629ef67c730c0e180dec15264577f675de1c090dca2d87da6bfa.
- No sysroot GREEN claim; no oracle mapped-unit manifest or S5 rate yet.

## Native sysroot publication GREEN — 2026-11-12

- Fixed terminal grouped-self imports in L2 import lookup. They resolve the
  prefix in the type namespace only. The persisted portable fixture and pure
  type-versus-constructor regression now pass. This clears the previously
  verified Option::{self,...} failure in installed std/core preludes.
- Persisted crate-root no_std/no_core/no_implicit_prelude flags even when the
  reached closure has no sysroot demand; no support units are added there.
  Explicit renamed extern crates and primitive u8 resolve in that branch.
- Committing the preserved sysroot implementation: independent core/alloc/std
  source contexts, active-closure demand, separate augmented staging, edition
  preludes, explicit extern aliases, retained discovered lang attributes and
  incoherent flags, provenance, and atomic per-crate name publication. Support
  units remain unmapped. Also preserves source-relative path and macro-decl
  fixes required to reach native publication, plus opt-in round diagnostics.
- Checks, all on the same final source:
  - L2 release suite: 9 passed, 0 failed (0.07 seconds).
  - Portable release CLI: 20 passed, 0 failed, native case filtered (0.98 s).
  - Native release CLI test_lang_items_in_sysroot: 1 passed, 0 failed,
    20 filtered (1202.91 s). It checks Option, Vec, core::ops::Range,
    std::string::String, five discovered lang names, and provenance arrays.
  - Focused all-target Clippy -D warnings: Finished dev in 0.78 seconds.
  - cargo fmt --all and git diff --check: exit zero.
- Earlier unchanged graph contracts (9+3), syntax contracts (17), and pure
  macro contracts (22 plus one doctest) remain passing; not rerun.
- Runtime is still large. No M2 performance envelope claim is made. Next
  REFACTOR removes repeated owned grouping-key clones in pure normalization,
  preserving precedence, merge order, visibility and canonical output.
- Exact mapped-unit manifests, PRIMARY/SECONDARY differ and all-five S5 remain
  pending. Additional lead reminder recorded: when reaching S6/S7 use
  codebase-memory list_projects, match root_path, inspect schema, then consume
  complete paginated coverage per SOP; continue strict phase checkpoints.

## Functional Rust and sysroot REFACTOR — 2026-11-12

- Added forbid(unsafe_code) and cfg_attr(not(test), deny(clippy::unwrap_used,
  clippy::expect_used, clippy::panic)) to all ten production crate roots and
  the shared parser-source build script. There were no new production lint
  hits to suppress or bypass. Test-only unwrap allowances remain test-only.
- Converted clear authored accumulators/traversals to iterator/try_fold/
  try_for_each: Turso output validation, graph source-root collection and
  traversal, sysroot row assembly, syntax errors/child traversal, and foreign
  binding input collection. Async database cursors and state-machine/fixed-point
  loops remain explicit where an async fold would obscure ordering or add
  boxing. Copy-adapted MBe loops are unchanged; source notice records why.
- Pure name normalization now stable-sorts owned rows and consumes contiguous
  groups instead of cloning large scope/target keys into grouping maps.
  Stable grouping preserves precedence, candidate order, visibility union
  order, evidence deduplication and canonical output.
- cargo test --release --workspace: 112 unit/integration passes plus one
  doctest, zero failures, three intentionally ignored manual profiles.
  Native CLI is now fully green: 21 passed, 0 failed in 935.51 seconds.
  Other result counts: core 3; graph 9+3; macros 22; resolver 9; scheduler 4;
  syntax 17; trait fake 1; backend 2+15; dependency gates 4; safe span 2.
- cargo clippy --workspace --all-targets --all-features -- -D warnings:
  Finished dev profile in 1.40 seconds. cargo fmt --all --check and
  git diff --check: exit zero. No isolated performance-envelope claim.
- Verification logs: /home/ubuntu/m2-functional-rust-refactor-tests.log and
  /home/ubuntu/m2-functional-rust-refactor-clippy.log.

## Exact mapped-unit manifest continuation findings

- Critical denominator correction: graph loader lib.rs selects all reached
  normal-dependency libraries as mapped target units, plus workspace bins.
  Existing oracle selects only workspace-member lib/bin roots. Therefore raw
  counts (especially cargo-modules 1369) are not valid S5 denominators.
  Manifest selection must use every published unit_mapped_target_flag=true
  row, not infer selection from workspace membership. Sysroot support stays
  unmapped. Preserve host/target and independent cfg/features.
- Plan next STUB/RED: read-only published-stage JSONL export (header includes
  generation/attempt even when empty), exact unit/config/source-closure
  manifest export, then harness validation/selection/configuration.
  Unknown tables/missing publication/malformed input must fail explicitly.
- Existing oracle loader keeps proc-macro load errors and invalid UTF-8
  omissions silently; exact export must record them and block affected
  relations. Cached proc-macro injection is still an explicit unsupported
  branch. SECONDARY build-script output can supply cache artifacts, but
  PRIMARY must inject only dylibs, not build-script cfg/env.
- External base-db reexports salsa; all_crates(db) returns base Crate inputs,
  whose salsa setters can override cfg_options after loading, avoiding private
  CrateGraphBuilder internals and package-name/version conflation. Verify
  exact manifest cfg on each selected compilation unit. SECONDARY adds only
  recorded build-script deltas to the same baseline.
- Module-scoped shared-file resolver remains preferred. It explicitly errors
  on unexportable shared local items; if reached, use the authorized duplicated
  file-id fallback with canonical paths in separate FileSets/SourceRoots,
  preferably cloning only the exact active closure from the manifest. Do not
  rewrite physical paths or hide export failures.
- No exact manifests, differ or S5 measurements exist yet. S6/S7 reminder is
  task 4; use project root matching, schema inspection and complete pagination.

## Exact mapped-unit manifests STUB — 2026-11-12

- CLI seams: export-published-stage-rows (table plus optional textual filter)
  and export-mapped-unit-manifest. Both require an existing store; export
  routes do not initialize or mutate the stage schema. Implementations
  currently return typed not-implemented errors.
- Offline harness seam: export_mapped_semantic_paths with explicit
  PRIMARY/SECONDARY configuration and manifest. CLI accepts the manifest and
  optional proc-macro cache. Legacy qualified raw export remains separate.
- STUB checks: production CLI cargo check Finished dev in 1.11 seconds;
  isolated oracle cargo check --release Finished release in 26.81 seconds.
- Next RED covers generation-bearing empty exports, normal-dependency unit
  selection/source closure, unknown tables/unpublished stores, exact cfg/edition,
  and explicit missing/duplicate/malformed oracle inputs.

## Exact mapped-unit manifests RED — 2026-11-12

- CLI five contracts fail at the intended typed STUBs: published JSONL rows,
  generation-bearing empty filter, exact dependency/physical source closure,
  empty mapped units, explicit missing-publication/table/store errors.
  Result: 0 passed, 5 failed (0.28 seconds).
- Offline oracle four contracts fail at the intended STUB: exact selected
  dependency units and cfg overrides, SECONDARY same baseline, empty selection,
  malformed version/duplicate units/missing closure errors.
  Result: 0 passed, 4 failed, 16 existing filtered (0.12 seconds).
- The oracle fixture exercises a reachable feature mismatch: normal dependency
  enables wanted with default-features=false, while dev-dependency enables
  default/wrong. cargo metadata can unify the dev feature; selected normal
  lib/bin manifest must force wanted-only cfg. Test also requires paths from
  the non-root dependency, not only workspace-member target roots.
- RED logs: /home/ubuntu/m2-mapped-manifest-red-tests.log and
  /home/ubuntu/m2-mapped-oracle-red-tests.log. No GREEN edits yet.
- Producer header schema in tests: export_record_kind_value=header,
  export_scope_identity_record, export_owned_table_name; rows have
  export_record_kind_value=row, export_row_identity_value,
  export_row_payload_value. Even zero rows must carry the header.
- Manifest test schema: manifest_format_version_value=1,
  manifest_target_context_kind=compiler_default, manifest_compiler_version_text,
  manifest_scope_identity_record, manifest_configuration_state_record,
  manifest_unit_context_rows. Unit entries preserve raw graph fields plus
  unit_semantic_identity_record (canonical cfg hash), unit_source_root_paths
  and unit_source_closure_rows (requests joined with content digests).
- Production currently has no target-triple CLI override and config stores
  compiler-default target cfg, not an explicit triple. Preserve that exact
  context; do not invent a triple or add a fourth direct compiler command.
- Important input/output metadata gap for following GREEN: production path and
  target scope records currently omit role/cfg digest. Add canonical semantic
  context/site kind evidence where required for exact differ keys; do not
  weaken matching to root path alone. Core semantic hash is blake3 of JSON
  [unit_target_cfg_values, unit_enabled_feature_values, edition, role].
- Code inspection verified base-db::all_crates and base-db::salsa reexport,
  allowing direct input cfg setters after workspace loading. Manifest override
  should happen before semantic queries; validate exact unit matches and source
  closure. SECONDARY build-script deltas must not leak into PRIMARY.
- No active background process, no S5 numbers. Available disk remains 32 GiB.

### Exact mapped-unit manifest GREEN

- Confirmed RED 12ac03491739b54e63a14260175cd5112be5b387 is on the
  remote research branch. The producer now exports published stage rows with
  a mandatory generation header, including empty exports, and exact unit
  manifests including reached normal-dependency libraries, not only members.
- Mapping stamps compiler and parser-source identities into the immutable
  workspace configuration. Export reads those snapshot identities, not the
  export executable's current compiler. Older snapshots missing them fail
  explicitly and must be regenerated for S5. No runtime discovery command
  was added; the syntax crate exposes its existing build-time compiler text.
- Manifests declare the lib/bin, normal-only, tests/bench/example/build-script/
  proc-macro exclusion policy. Unmapped host and sysroot metadata remain
  present. Source closures join discovery requests to stored content digests.
  Oracle validation checks compiler/pin, root, generation, cfg/edition/role,
  graph and semantic identity uniqueness, and active-source bytes/UTF-8.
- Oracle cfg inputs are overwritten before semantic queries. Mapped normal
  dependencies participate; SDK support does not enter denominators. Host
  metadata sharing a target source root cannot override target scope identity.
  Missing/ambiguous unit matches fail rather than becoming empty path sets.
- Production and oracle site/target scopes include canonical semantic unit
  identities; site context distinguishes imports and signatures. Oracle
  envelopes retain scope/mode/count plus proc-macro loading and source-omission
  observations. These observations are not proc-macro invocation counts.
- Checks: producer contracts 5 passed; syntax contracts 17 passed; oracle
  full suite initially 20 passed, followed by 6 mapped-manifest contracts
  after validation/host-role additions (the 16 legacy cases were unchanged).
  Workspace and oracle all-target Clippy -D warnings passed. Formatting and
  diff checks exited zero.
- Native CLI regression initially reported 20 passed / 1 failed in 907.07s,
  including a passing installed-sysroot/lang-item case. The failure selected
  a unit by package root alone, although its fixture has both a mapped lib
  and an unmapped custom-build host unit under that root. Selection now
  requires the target lib and additionally asserts the unmapped host row
  exists. The corrected case passed (1 passed / 20 filtered, 0.24s). No
  production flag assertion was relaxed, and no second native/sysroot run
  was needed because that case's inputs were unchanged.
- Logs: /home/ubuntu/m2-mapped-manifest-green-tests.log,
  /home/ubuntu/m2-mapped-oracle-green-tests.log,
  /home/ubuntu/m2-mapped-oracle-final-tests.log,
  /home/ubuntu/m2-mapped-native-regression-tests.log,
  /home/ubuntu/m2-mapped-flag-selector-tests.log,
  /home/ubuntu/m2-mapped-syntax-regression-tests.log,
  /home/ubuntu/m2-mapped-manifest-clippy.log and
  /home/ubuntu/m2-mapped-oracle-clippy.log.
- Qualification is not S5 completion: cached proc-macro artifact injection
  still explicitly rejects a supplied cache; SECONDARY with selected build
  scripts requires recorded configuration and is not yet qualified. The
  following comparison phase must implement those inputs and build-script
  deltas before claiming PRIMARY/SECONDARY agreement. Differ and all-five S5
  measurements remain outstanding. S6/S7 still require matched memory
  project roots, schema inspection and complete paginated coverage.
- Parser/oracle pin remains bf6b2099d6351c37039b01447a9bb64f9a54a586,
  upstream 2a362a41a8eb6e090f1a12d9b712df85af6f0c33 plus
  tools/parser-const-closure-statements.patch, SHA-256
  89146042a1c1629ef67c730c0e180dec15264577f675de1c090dca2d87da6bfa.
  No additional parser patch or external-checkout edit was made.

### Exact mapped-unit manifest REFACTOR

- Stream file-source metadata through the existing store visitor, retaining
  only path-to-digest values for closure joins instead of complete file-row
  JSON. Closure validation/errors and exported schemas are unchanged.
  The visitor's bounded mutable accumulator avoids retaining the raw row
  vector and is required by the streaming port interface.
- Producer contracts: 5 passed / 0 failed. Dispatcher all-target/all-feature
  Clippy -D warnings passed. No speed or RSS improvement is claimed without
  a separate measurement. Logs:
  /home/ubuntu/m2-mapped-manifest-refactor-tests.log and
  /home/ubuntu/m2-mapped-refactor-clippy.log.
- Next: cached proc-macro artifacts, recorded SECONDARY build-script deltas,
  qualified path/provenance exports and the differ, then fresh all-five M2
  snapshots/S5 results. Old S3 stores are not suitable manifest sources
  because their immutable configurations predate snapshot compiler/pin fields.

### Oracle artifact-cache input STUB

- Next comparison-phase boundary: offline Cargo check may prepare proc-macro
  dylibs and record build-script observations. PRIMARY must inject only
  verified cached dylib paths into the loader; cfg/env/OUT_DIR from build
  scripts stay disabled. SECONDARY receives recorded configuration deltas.
- Added build_oracle_artifact_cache and the explicit CLI output route as
  STUBs. New cache artifacts must bind compiler/pin, published manifest scope,
  exact host-proc-macro semantic units/features, source roots and dylib bytes.
  Required missing/ambiguous/tampered artifacts must fail, not silently drop
  macro exports. Artifact loading observations are distinct from invocation
  provenance and S5 exclusions, which remain following work.

### Oracle artifact-cache input RED

- Added real offline proc-macro compilation/load contracts, an empty-artifact
  cache retaining scope, missing/tampered/scope/compiler/pin cache errors,
  and explicit compiler failure. The PRIMARY check requires cached loading
  and resolution of a type generated by an attribute macro while build
  scripts remain disabled.
- Four new tests fail at the intended artifact-cache STUB, not at fixture
  compilation. Log: /home/ubuntu/m2-oracle-cache-red-tests.log.
- Inspection: the private workspace loader receives per-crate macro path
  Results from to_crate_graph. Replace just those paths using validated
  source-root-to-dylib mappings before load_proc_macro; do not populate
  workspace build-script cfg/env/OUT_DIR in PRIMARY. This avoids modifying
  the external rust-analyzer checkout or its private build-output types.

### Oracle artifact-cache input GREEN

- Added offline, locked normal-target Cargo preparation with compiler-version
  validation. This is an explicit offline-teacher route, not a production
  mapper subprocess or a PRIMARY build-script configuration change.
- Cache binds publication scope, parser/compiler, exact host-proc-macro unit
  and semantic identities/features, absolute dylib path and BLAKE3 bytes.
  Required missing, duplicate, ambiguous, stale and tampered cache inputs
  fail explicitly. Loader substitutes only verified dylib paths and emits
  cached/workspace loading origin; PRIMARY still disables build scripts.
- Retained build-script cfgs, env names plus canonical env-value digest, and
  generated output directory as preparation observations. Env values are
  never written into cache JSON. They are not applied to PRIMARY.
- Found/fixed normal-target preparation edge case: explicit --lib --bins
  rejects a binary-only workspace. Failing fixture printed
  "no library targets found"; use Cargo check's default lib+bin selection
  with --workspace, --offline and --locked instead.
- Full affected oracle harness: 26 passed / 0 failed, 9.13s. Final preparation
  contracts after binary-only and hashed-env additions: 6 passed / 0 failed,
  2.78s. Release all-target clippy -D warnings passed (0.45s); diff check
  passed. Logs: /home/ubuntu/m2-oracle-cache-green-tests.log,
  /home/ubuntu/m2-oracle-cache-final-tests.log,
  /home/ubuntu/m2-oracle-cache-clippy.log. External checkout remains clean.
- S5 is still unmeasured. Next: SECONDARY exact-baseline cfg plus verified
  build-script deltas, invocation/expansion provenance and proc exclusions,
  the differ, full all-five runs, then S6/S7 complete codebase-memory coverage.

### Oracle artifact-cache input REFACTOR

- Replaced cache-path accumulation with try_fold and extracted canonical
  build-script env hashing for SECONDARY verification. No protocol or
  configuration behavior change.
- Preparation/cache contracts: 6 passed / 0 failed (2.80s); release all-target
  clippy -D warnings passed (0.45s); diff check passed.
- Logs: /home/ubuntu/m2-oracle-cache-refactor-tests.log and
  /home/ubuntu/m2-oracle-cache-refactor-clippy.log. The already-passing legacy
  and manifest suites were not rerun because their inputs did not change.

### SECONDARY recorded-script deltas STUB

- Added a private boundary for baseline-plus-recorded-script cfg/env/output
  qualification. No-script SECONDARY retains the identical manifest baseline;
  script-bearing inputs stop explicitly at the STUB. PRIMARY never invokes
  this boundary or applies these inputs.
- SECONDARY now accepts cached observations as the required recorded input
  rather than rejecting all selected packages with build scripts.

### SECONDARY recorded-script deltas RED

- Added script cfg/env/output fixture with a generated Rust include.
  PRIMARY must retain the unresolved baseline; SECONDARY must resolve the
  generated type while preserving scope/mapped-unit count and report KL1
  cfg/env-hash/generated-source identities without env values.
- Empty-script baseline and missing/stale cfg/env/output errors are covered.
  RED log: /home/ubuntu/m2-secondary-script-red-tests.log.
- Private upstream build-script fields were inspected and remain untouched.
  Verify recorded inputs against public per-crate cfg/env, then add only
  recorded cfg atoms to the manifest baseline. Collect generated-source
  digests from the loaded VFS, not a second filesystem read.

### SECONDARY recorded-script deltas GREEN

- SECONDARY verifies recorded cfg atoms, canonical env hash/names and OUT_DIR
  against the actual public per-crate cfg/env before adding just those cfgs
  to the exact PRIMARY manifest baseline. Wrong/stale inputs fail explicitly.
- Emits per-unit KL1 delta records with baseline semantic identity, cfgs,
  env hash/names, output path and generated-source BLAKE3 identities from
  loaded database text. Env values are not exported. PRIMARY remains unchanged.
- New paired fixture verifies PRIMARY-unresolved / SECONDARY-resolved paths
  plus generated include source identity and unchanged publication/unit count.
- Full affected harness: 31 passed / 0 failed (9.85s). Release all-target
  clippy -D warnings passed (3.74s). Logs:
  /home/ubuntu/m2-secondary-script-green-tests.log and
  /home/ubuntu/m2-secondary-script-green-clippy.log. Diff check passed.
- Remaining before S5: exact expansion/proc provenance, CPU/table-backed
  differ and full corpus mapping/export/measurement. No S5 number yet exists.

### SECONDARY recorded-script deltas REFACTOR

- Extracted recorded cfg-atom parsing; no behavior/schema change.
- Paired script contracts: 3 passed / 0 failed (5.07s); release all-target
  clippy -D warnings passed (3.97s); diff check passed.
- Logs: /home/ubuntu/m2-secondary-script-refactor-tests.log and
  /home/ubuntu/m2-secondary-script-refactor-clippy.log.
- Full M2 serde mapping started independently using the frozen production
  executable 30cf4b8f5f0c456d7c4fa6d6a9a960575fec24a9fe691320db23efd92443ac8a.
  Output: e2e-runs/2026-11-11/serde/s5-primary-path-agreement/.
  Shell ce054b owns the sole writer. It exports the exact manifest only after
  mapping succeeds. Do not run a second writer against this store.

### Canonical path comparison STUB

- Added explicit CPU differ route and facade boundary. Source mapping store
  uses the read-only export route (no schema initialization). Comparison rows
  will live in a separate Turso sidecar, preserving the frozen mapping
  generation; identical normalized source/output paths are rejected.
- PRIMARY alone supplies the denominator. SECONDARY-only observations are
  KL1. Unresolved PRIMARY-visible sites are nonmatches; no empty-target
  equality shortcut. M2 proc-dependent exclusions need oracle evidence and
  remain separately reported. Known oracle errors block affected scoring.

### Canonical path comparison RED

- New CLI contracts cover exact/deduplicated paths, Turso publication,
  immutable source generation, empty denominator N/A, unresolved nonmatches,
  M2 proc exclusion without excusing wrong extracted targets, scope/source
  errors and source/output alias rejection.
- Log: /home/ubuntu/m2-path-comparison-red-tests.log. Three scoring contracts
  fail at the intended comparison STUB; explicit-error contract passes.
- Additional full M2 writers use the identical frozen mapper:
  ripgrep shell 448968; tokio shell 7aa41a. Each owns only its repository's
  s5-primary-path-agreement directory/store. Serde remains shell ce054b.
- Comparison publication design: independent Turso sidecar with its own
  comparison generation and source-snapshot reference. Never append analysis
  facts to, or replace publication of, the frozen mapping database.

#### RED fixture correction and verified output

- Correction to the preceding RED entry: its first execution did not reach
  the STUB. The new fixture omitted Cargo.lock; load_workspace_crate_graph
  reads it before metadata (lib.rs:106-107), causing four setup failures with
  CrateGraphLoadError: No such file or directory (os error 2). Added offline
  generate-lockfile to the fixture, matching existing CLI contracts.
- Corrected run: 1 passed / 3 failed (0.28s); all three failures now show
  oracle path comparison not implemented. Log remains
  /home/ubuntu/m2-path-comparison-red-tests.log.
- Writer shell pair for ripgrep/tokio: 7aa41a and 448968. The asynchronous
  launch output did not label their association; use repository logs and
  mapper PIDs 1353631 (ripgrep), 1353632 (tokio), rather than assuming the
  association in the preceding entry. Serde mapper PID 1335853, shell ce054b.
- At the last observation serde had run 14m / RSS 4,386,984 KiB; ripgrep/tokio
  6m26s / RSS 3,610,668 and 3,485,264 KiB. These are live observations, not
  final performance/peak measurements or S5 results. All were in the
  name-resolution stage; no additional writer has touched their stores.

### Priority: M2 memory gate investigation

- Full runs stopped per the lead's interrupt. SIGTERM sent to ripgrep
  1353631 and tokio 1353632; their shells 7aa41a/448968 exited 1.
  Serde 1335853 had already completed before termination: shell ce054b
  exited 0, name resolution 1077.043s, total wall 18:49.46, maximum RSS
  14,073,956 KiB. This FAILS the 2 GiB gate and is not qualified S5 evidence.
- Task 6 supersedes task 3 until bounded-memory qualification exists.
- Comparator GREEN is parked UNCOMMITTED: facade and Turso backend. Its
  4 new CLI contracts passed in 0.28s, log
  /home/ubuntu/m2-path-comparison-green-tests.log; existing affected adapter
  and CLI tests and clippy have NOT yet run. Do not commit it as qualified.
- Serde's published store/manifest can support read-only diagnosis. Never
  reuse it as a passing measurement. No rust-analyzer/cargo-modules M2
  writers have been started.

### Bounded namespace state STUB and short profile

- Parked comparator GREEN source changes in
  /home/ubuntu/m2-paused-comparator-green.patch; restored its committed STUB.
  This keeps unrelated comparator changes out of memory-fix commits.
- Added shared namespace rows (Arc), an owned-input resolver seam (STUB),
  and a private-crate-row visibility predicate seam (currently no-op).
- RSS-sampled serde reproduction, frozen original binary, 120s timeout:
  3,304,857,600 bytes peak, exit 124. Evidence under
  e2e-runs/2026-11-11/serde/m2-name-memory-profiling/.
  The core projection reached 41,232 scopes, 25,705 declaration bindings,
  4,240 imports, 61,757 paths and 708 macros. The first macro round had
  253,837 settled bindings and 624 resolved invocations.
- This disproves bulk loading of already-published SDK maps as the sole
  initial cause: the budget already fails in core's own first resolution.
  Reachable retention also includes projector/input/engine copies and
  whole-round semantic rows held until one large crate transaction.
  Attribution of the full run's 14,073,956 KiB peak is not yet isolated.
- Read-only payload sizing found 2,118 expanded macro rows / 254,146,220
  export bytes, plus 2,053 unresolved rows; largest single expanded row
  10,130,358 bytes. /home/ubuntu/m2-macro-payload-size-records.json.

### Bounded namespace state RED

- STUB workspace check passed (0.36s).
- Resolver contracts: 9 passed / 2 failed. Owned-input empty case fails at
  owned namespace resolution not implemented; shared dependency case fails
  at name-resolution row references a missing scope.
- New adapter visibility contract fails as intended: pending crate chunk
  count is 1, expected 0. It checks count/read/visit before and after the
  constant-size crate completion marker, across bindings and span rows.
- Logs: /home/ubuntu/m2-bounded-namespace-red-tests.log and
  /home/ubuntu/m2-bounded-publication-red-tests.log.

### Bounded namespace GREEN working state (not qualified or committed)

- STUB bf5168136 and RED 9485766dc are pushed.
- Working implementation shares dependency namespace rows and target
  metadata through Arc, traverses only the dependency closure, consumes
  resolver input, spools macro semantic rows immediately, and streams
  final rows into 8 MiB private Turso chunks. One oversized row is allowed
  alone. A constant-size crate marker exposes all its chunks together.
  Reader count/read/visit use the same visibility predicate. Projector
  state drops after publication; only immutable namespace maps survive.
- Structural tests: shared 1,000-binding namespace pointer identity and
  release; owned empty/cancelled inputs; pending binding/span invisibility;
  bounded 20 MiB output, one oversized record, empty/cancelled/failed output;
  namespace codec round trip/error.
- Initial implementation passed 11 resolver tests, 16 stage-store
  contracts, 2 adapter unit tests (3 ignored profiles), 17 syntax contracts
  and 3 new syntax unit tests. Clippy all targets passed after test-only
  unwrap allowances and mechanical test-module relocation.
- However native CLI sysroot test exceeded the gate: PID 1409785 had
  VmHWM 3,664,948 KiB and RSS 2,283,592 KiB. SIGTERM requested rather than
  treating this as qualification. The CLI log ends 20 passed / 1 failed
  (NameResolutionCancelledError), elapsed 391.86s.
  /home/ubuntu/m2-bounded-cli-tests.log.
- Latest additional change shares exact-equal import binding rows across
  current/Brent-anchor states using Arc and bounded per-iteration borrowed
  indexes. It preserves exact comparisons, with no hash-collision shortcut
  and no global interner. Resolver tests again: 11 passed; all-target
  Clippy passed (0.47s). Logs:
  /home/ubuntu/m2-bounded-import-final-tests.log and
  /home/ubuntu/m2-bounded-namespace-clippy.log.
- A public-only dependency cache filter was considered then removed:
  dropping private rows could change unresolved candidate diagnostics.
  All dependency namespace rows are retained in the current source.
- CLI retry is ACTIVE, shell 90c6dc:
  /home/ubuntu/m2-bounded-cli-retry-tests.log.
  Latest observed native mapper PID 1416782, 529,676 KiB RSS at 1:13.
  This is NOT a peak or passing budget result. This executable was built
  before removing the experimental public-only cache filter; its own-core
  behavior is current, but subsequent dependency-cache behavior is not.
  Rerun the affected CLI test after completion using current source.
- No GREEN/REFACTOR commit yet. No new serde/full-corpus passing RSS
  measurements or all-five name-resolution times. Comparator is still its
  committed STUB, saved GREEN patch remains at
  /home/ubuntu/m2-paused-comparator-green.patch. S5 is unmeasured.
- If the newest implementation still exceeds 2 GiB, investigate lifetime
  of projection_macro_source_rows (original/parse text and token-origin
  maps), and import derivation's whole next-state temporary. Reachability
  and retained-byte contribution are not yet profiled for these candidates.

### Second budget failure and context handoff

- Retry PID 1416782 reached VmHWM 2,878,372 KiB, RSS 1,573,004 KiB;
  therefore it also FAILS the 2 GiB peak gate, despite row sharing.
  SIGTERM requested. Shell 90c6dc is no longer a qualification run.
  No passing all-five measurement or name-resolution time is claimed.
- Next targeted measurement should count pre-normalization derived rows
  versus normalized rows in normalize_scope_binding_rows. It currently
  collects the complete iterator before sorting/grouping; derive_import_
  binding_rows also collects a whole owned Vec. Core's 253,837 settled
  bindings need not bound either transient size. This path is reached by
  the observed macro round, but its retained-byte contribution remains
  unverified. An incremental priority/target-group fold is a candidate;
  preserve stable visibility-union and evidence semantics if implemented.
- Current source retains ALL dependency bindings; the experimental
  public-only cache filter was removed after the running executable was
  built. Resolver 11 tests and focused all-target Clippy pass; native CLI
  still has no passing completion. GREEN stays uncommitted.
- Latest unit output log /home/ubuntu/m2-bounded-buffer-final-tests.log:
  3 passed (bounded payload, cancellation/failure, codec round trip).
- No mapper should remain writing the temporary native-test store.

### Heap allocation profiling setup

- Installed heaptrack 1.3.0 through apt after checking the repository
  environment blueprint. Release symbols enabled with
  CARGO_PROFILE_RELEASE_DEBUG=1 CARGO_PROFILE_RELEASE_STRIP=none.
- First trace is INVALID for Rust allocations: Turso's default mimalloc
  feature installs a process-wide allocator from turso/src/lib.rs:35-37.
  nm confirms __rustc::__rust_alloc and mi_malloc_aligned in the executable;
  LD_PRELOAD was present, but heaptrack's malloc hooks saw only libc work.
  Stopped this run rather than interpreting its output as Rust evidence.
- Added feature forwarding so DEFAULT production still uses the identical
  Turso mimalloc + fts + simd feature set. Profiling command selects
  --no-default-features --features system-heap-allocation-profile on the
  CLI; this disables only mimalloc, retaining Turso and its other defaults.
  It is instrumentation, not a production allocator change. Production
  qualification must rebuild with default features and run uninstrumented.
- System heaptrack trace also became incomplete: kernel dmesg verifies the
  interpreter was OOM-killed at 26,577,164 KiB anonymous RSS (PID 1452145).
  At that time the mapper was only 112,822 pages resident. The resulting
  175.219-second trace has 608.41 MB peak heap and does NOT cover the later
  >2 GiB mapper state. No allocation fix is inferred from this partial trace.
  Switching to the explicitly permitted CLI-only feature-gated dhat allocator
  with bounded backtrace depth, retaining the default production allocator.
- Additional authorized experiment after the profile: validate pinned Turso
  recursive CTE / INSERT SELECT joins, then spike a set-based import round
  behind shared resolver contracts, alternating Rust macro expansion. Compare
  serde binding equality, rounds, runtime and RSS against the in-memory
  resolver before selecting the implementation and resuming all-five S5.

### User-approved DuckDB architecture supersedes memory work

- Stopped all profiling/mapping processes. No further in-memory optimization
  or Turso-SQL spike is authorized. The interrupted DHAT capture ended during
  syntax work: 170,991,552 peak live bytes, not a full-M2 qualification.
- Parked the coupled unfinished namespace/publication work in
  /home/ubuntu/parked-memory-resolver-changes.patch (SHA-256
  cb5d15b49a76457df93d5148ea479ccc15e50dabdd227ff1fafbd42b3053ae8e).
- 685fa8b6e is the DuckDB adapter STUB: latest stable duckdb 1.20000.1,
  bundled, L3 adapter behind AnalysisTableStorePort. Constructor, Appender
  snapshot and count APIs remain deliberate STUBs. Build check printed:
  Finished `dev` profile [unoptimized] target(s) in 8m 46s.
  Command used CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CXXFLAGS=-g0
  to bound build resources; runtime memory policy is not implemented yet.
- 932e9263c appends the architecture/PRD revisions, Mermaid diagram, ownership,
  named required RED cases, non-monotone full-set rounds, denominator partition,
  count/digest handoff, deterministic exports and 1.2 GiB engine limit.
- ec2090c3a444b651e6f95c117effcb9f9bb1ebe2 is the isolated user-approved
  deletion of archive/, tests/, dependency_queries/, entity_queries/,
  experiments/ and scripts/. Reference scan of README, AGENTS, gitignore,
  Cargo, vscode, phase docs and tools found only valid external-corpus tests/
  mentions, not links into the removed trees. Research and older docs untouched.
  Post-deletion workspace check and tests are running in shell fb1fb0:
  /home/ubuntu/legacy-deletion-check-records.log and
  /home/ubuntu/legacy-deletion-test-records.log. Tests exclude the superseded
  in-memory resolver crate's existing RED work and the known over-budget native
  test_lang_items_in_sysroot; these exclusions are not a full-suite pass.
- Mandatory next work, before resolver SQL: remove the obsolete profiler CLI
  features/tests/tools, delete the old resolver crate but MOVE its contract
  cases to DuckDB, remove old in-memory orchestration and preserve/move syntax
  scope/import/item/visibility extraction. Inspect old comparator patch before
  deleting it. Then re-index codebase-memory and verify dangling references.
- Scope discovery found another old resolver dependency beyond the primary
  caller: persisted-macro-expansion-adapter.rs uses resolve_macro_path_binding
  for nested built-in lookup. This must move behind the SQL lookup port, not
  be retained as an in-memory fallback. Macro expansion itself is retained.
- Remaining work: adapter RED/GREEN/REFACTOR and process audit; complete source
  cleanup; DuckDB resolver contract GREEN; serde measurement; SQL comparator;
  all-five S5 and S6/S7. No S5 denominator or agreement is measured.

### 2026-11-12 — DuckDB namespace contract port RED

- Deleted the obsolete parseltongue-port-name-resolver implementation, manifest
  and source notice. Its eleven contract cases moved unchanged to
  parseltongue-duckdb-analysis-adapter/src/row-resolution-contract-tests.rs.
  The DuckDB entry points are STUBs; the moved suite is intentionally RED:
  `test result: FAILED. 0 passed; 11 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`.
  Every failure reaches `not yet implemented: DuckDB non-monotone SQL resolution`.
- Removed whole-crate resolved-dependency namespace grafting and the old
  name-resolution-stage-adapter. Moved syntax extraction and input encoding
  into syntax-namespace-input-extraction, syntax-namespace-record-codec and
  syntax-namespace-input-persistence. Resolved-row decoders were deleted.
  Source scopes, declared items, imports, paths, macro invocations and extern
  declarations now have Turso input tables. Extern aliases remain syntax
  records, not Rust-side binding decisions.
- Input records are emitted in 4 MiB chunks, private until the crate completion
  marker. Restored the previously green Turso private-chunk visibility predicate
  from the parked patch and extended it to the new input owner. A new Turso
  contract checks private namespace input rows; all seventeen stage contracts pass.
- Moved ItemMacroExpansionPort to the L2 table-store traits and added
  ResolvedMacroLookupPort for primary target metadata and nested builtin lookup.
  The second old-resolver dependency is gone; lookup errors propagate rather than
  becoming missing-builtin answers. Rust macro parsing/expansion is retained.
- Removed obsolete CLI DHAT allocator/features, Turso allocator-profile feature
  indirection, the native heap profile contract, both heap summary Python tools
  and the pre-DuckDB comparator patch. Existing profiling evidence and the
  separately parked resolver patch remain. No memory fix is claimed.
- Full M2 mapping currently returns an explicit DuckDB-not-implemented error
  after input extraction. It does not publish incomplete semantic success.
  M1 remains available. Constructor policy, Appender handoff, SQL resolution,
  macro alternation, exact oracle manifests, SQL comparator and all-five
  qualification remain to be implemented.
- Final source cleanup checks:
  `cargo check --workspace`:
  `Finished `dev` profile [unoptimized] target(s) in 0.48s`.
  Syntax tests: `test result: ok. 3 passed; 0 failed` and
  `test result: ok. 17 passed; 0 failed`.
  Turso stage tests: `test result: ok. 17 passed; 0 failed`.
  Dependency/build gates: `test result: ok. 4 passed; 0 failed`.
  `cargo clippy --workspace --all-targets -- -D warnings`:
  `Finished `dev` profile [unoptimized] target(s) in 0.59s`.
  The external pinned parser/syntax `unused manifest key lints.cargo` warning
  remains the previously diagnosed upstream-toolchain warning.
- Post-legacy-deletion workspace check also completed:
  `Finished `dev` profile [unoptimized] target(s) in 10m 33s`.
  Its workspace test command stopped at three existing comparator STUB failures
  (`oracle path comparison not implemented`), not missing legacy files.
  No full-workspace test GREEN is claimed.
- Codebase-memory verification used the exact root_path match, schema, complete
  cursor pagination (150 + 150 + 124 = 424 file nodes) and coverage reconciliation.
  All 57 filesystem Rust sources under crates/ and tools/ are represented.
  Deleted resolver/extractor paths have zero graph nodes. Coverage reports no
  recorded issue under crates/; tools' only recorded gap is the intentionally
  excluded oracle target directory. This is a source-cleanup check, not S6/S7.
- Environment maintenance: removed unused release/native debug build caches to
  recover disk; the VS Code rust-analyzer process 1532439 is temporarily SIGSTOPed
  because its separate default-debug Cargo builds competed with bounded native
  builds. Resume it when native build/measurement work is finished. No mapper or
  profiler runs. The read-only environment blueprint still mentions the deleted
  experiments/spancut maintenance command; workspace Cargo commands replace it.
- Parser/oracle remain pinned together at
  bf6b2099d6351c37039b01447a9bb64f9a54a586, base
  2a362a41a8eb6e090f1a12d9b712df85af6f0c33, reviewable patch
  tools/parser-const-closure-statements.patch, SHA-256
  89146042a1c1629ef67c730c0e180dec15264577f675de1c090dca2d87da6bfa.
  No new parser defect or S5 measurement is claimed.

### 2026-11-12 — bounded DuckDB constructor and Appender handoff GREEN

- Constructor policy RED a475542bc, GREEN 9e360a351; handoff RED c1def2e46,
  GREEN 0490a37c3; export contract REFACTOR d9e169a63; pinned Turso integration
  contract 88d1d014a. All pushed on research/tiny-rust-code-mapper.
- Constructor rejects zero and above-1.2-GiB memory limits, requires an existing
  run directory, creates duckdb-analysis-spill-files under its canonical path,
  disables extension autoinstall/autoload and community extensions, and disables
  external access before returning. Tests verify INSTALL/LOAD rejection and
  inability to re-enable external access.
- DuckDB does not accept temp_directory in startup Config, and changing it after
  disabling external access returns Permission Error. The constructor therefore
  opens privately with extension loading disabled, sets the spill directory,
  then irreversibly disables external access before exposing the connection.
  No external SQL is run while access is enabled. Native policy tests pass.
- frozen-generation-appender-copy.rs streams each source table in strict unique
  row-key order through Appender. It preserves generation, attempt, source table,
  row key and exact payload. BLAKE3 uses length-framed table/key/payload bytes;
  source count/digest must match ordered DuckDB count/digest before manifests
  commit. Copy and completion manifests share one transaction; failures roll
  back. Existing snapshots cannot be overwritten. Publication is checked before
  copying and again before commit. Empty source tables receive verified manifests.
- Named tests cover normal/empty copy, unpublished/cancelled/source-failure
  rejection, wrong counts, unsorted source rows, rollback/retry and one-versus-four
  thread identical ordered exports. Tests also copy a real complete publication
  from the pinned Turso adapter and verify exact payload identity.
- Focused constructor/copy tests:
  `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.26s`.
  Final real-Turso integration:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.22s`.
  Adapter all-target Clippy -D warnings:
  `Finished \`dev\` profile [unoptimized] target(s) in 0.40s`.
  The eleven resolver contracts remain RED; no resolver GREEN is claimed.
- Native builds finished; resumed VS Code rust-analyzer PID 1532439 with SIGCONT.
  No mapper or profiler measurement was started. Remaining: SQL resolver and
  accuracy fixture tests, textual macro scopes, macro alternation, orchestration
  of a frozen input publication, process-network audit, serde M2, exact oracle
  manifests, SQL comparator and all-five S5/S6/S7. No S5 numbers exist.
- Next implementation should live in the DuckDB adapter behind the L2 ports.
  Do not reconstruct the deleted in-memory path resolver. Eleven moved contract
  cases include per-namespace alias chaining, glob ambiguity/visibility, prelude
  boundaries, edition paths, invalid scopes/cycles, shared dependency rows,
  macro additions and the 8192-changing-round boundary. The macro test ports
  simulate expansions even without a macro binding; production persisted macro
  expansion only processes expansion_resolved_macro_rows. Preserve that guard.

### DuckDB resolution accuracy and immutable source publications

- SQL strata, scope ancestry, path roots and typed contract bridge are committed
  through `f19c685a6`. The eleven moved resolver contracts passed together:
  `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 24.37s`.
- Accuracy RED `18f3ab216` captured raw import aliases and restricted reexport
  leaks. GREEN `92b6dad9f` adds per-binding visibility, importer filtering and
  raw alias normalization. Latest adapter run (the unchanged 8192 macro-limit
  fixture excluded) printed:
  `test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 39.79s`.
  Adapter all-target Clippy completed successfully in 0.40s.
- Frozen-publication RED `d796d2402` proved that the existing Turso publication
  pointer did not freeze its rows: a published work unit could be overwritten,
  and a private chunk became visible after a future completion marker appeared.
- The corresponding GREEN uses `frozen-stage-publication-records` and
  `frozen-stage-unit-memberships`, sealed atomically with publication. Existing
  work units cannot be rewritten (including through a different stage sharing
  their tables). New work units can accumulate without changing older published
  views. Republishing assigns a new snapshot attempt suffix `/frozen/N` while
  preserving the generation key. Reads/counts/visits resolve the snapshot to
  immutable work-unit memberships; future completion markers cannot expose old
  private chunks. Frozen scopes cannot be renamed or used as write scopes.
  No payload-table duplication or in-memory snapshot is required.
- Final source-publication checks:
  `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.70s`.
  Real pinned-Turso to DuckDB handoff:
  `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 37 filtered out; finished in 0.24s`.
  Turso all-target Clippy completed successfully in 0.57s. The external pinned
  syntax manifest still emits its known nightly-only `lints.cargo` warning.
- Still required: textual macro scope facts and production macro alternation;
  raw frozen-input SQL projection and CLI wiring (do not use the typed contract
  facade to rematerialize an entire repository); process/network audit;
  functional-Rust REFACTOR; serde M2 peak/wall/rounds; exact oracle manifests;
  SQL comparator and all-five S5 module/type-relative splits and disagreements.
  No S5 measurements exist. PID 1532439 is currently stopped during native
  build work; resume it when builds finish. Available disk was 7.6 GiB; existing
  oracle build caches and old e2e databases account for most usage.
- Parser/oracle identity remains fork `bf6b2099d6351c37039b01447a9bb64f9a54a586`,
  based on `2a362a41a8eb6e090f1a12d9b712df85af6f0c33`, with reviewable patch
  `tools/parser-const-closure-statements.patch`, SHA-256
  `89146042a1c1629ef67c730c0e180dec15264577f675de1c090dca2d87da6bfa`.
