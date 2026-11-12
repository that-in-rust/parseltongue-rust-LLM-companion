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
