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
