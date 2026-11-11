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
