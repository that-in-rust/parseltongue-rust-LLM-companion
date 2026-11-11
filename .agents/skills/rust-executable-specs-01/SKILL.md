---
name: rust-executable-specs-01
description: Unify executable specifications, idiomatic Rust architecture, and reliability-first Rust delivery for libraries, CLIs, async services, app backends, and contained FFI wrappers. Use when Codex needs to decide how to approach Rust work, convert requests into REQ-RUST contracts, choose L1/L2/L3 boundaries, apply typed-error and async/concurrency safety patterns, select verification strategy, enforce four-word naming where feasible, or produce quality gates and implementation plans.
---

# Rust Executable Specs 01

Use this skill to turn Rust requests into decision-complete work packets that are spec-first, architecture-aware, and reliability-biased.

## Mode Selection

Choose one or more modes before planning or coding:

- `Spec Mode`: use for ambiguous feature requests, missing acceptance criteria, greenfield planning, or any request that needs `REQ-RUST-*` contracts first.
- `Delivery Mode`: use for implementation and refactor work that needs L1/L2/L3 boundaries, trait seams, TDD sequencing, and verification order.
- `Reliability Mode`: use for public APIs, error surfaces, async and cancellation, concurrency, unsafe, FFI, diagnostics, or release-hardening.
- `Review Mode`: use for code or design review when the main question is whether the Rust stays safe, diagnosable, and maintainable under change.

Default combinations:

- vague feature request: `Spec Mode` + `Delivery Mode`
- async service or parser/protocol work: `Delivery Mode` + `Reliability Mode`
- library or public API work: `Spec Mode` + `Reliability Mode`
- code review: `Review Mode` + `Reliability Mode`

## Workflow

1. Classify the task and choose the active mode set.
- Name the component type first: library, CLI, async service, backend, parser/protocol, FFI wrapper, or review.
- Decide whether the main risk is ambiguity, design, reliability, or review depth.

2. Write executable requirements.
- Use `REQ-RUST-<NNN>.<REV>` identifiers.
- Express behavior as `WHEN...THEN...SHALL`.
- Reject vague claims such as "faster" or "more reliable" without thresholds.

3. Define Rust architecture before coding.
- Separate boundaries into L1 core, L2 std, L3 external.
- Keep L3 dependencies out of L1.
- Define trait seams before concrete adapters when testability or substitution matters.

4. Choose the verification mix deliberately.
- Pick from unit, integration, doctest, compile-fail, property, fuzz, `loom`, performance, and coverage gates.
- Match tests to the real risk surface instead of defaulting to unit tests only.

5. Apply reliability rules.
- Prefer newtypes, explicit conversions, and invalid-states-unrepresentable design.
- Keep library errors typed and boundary-aware.
- Avoid blocking and lock guards across `.await`.
- Prefer bounded concurrency, cancellation awareness, and minimal unsafe surfaces.

6. Apply project conventions selectively.
- Use four-word names when feasible for new functions and commands.
- Preserve public API compatibility when renaming is risky; add adapters or wrappers when useful.
- Keep modules cohesive and documentation synchronized with behavior.

7. Run quality gates and anti-pattern checks before handoff.
- Require `cargo fmt`, `clippy -D warnings`, tests, and build success.
- Reject work that lacks requirement coverage, measurable performance evidence, or concurrency validation where it matters.

## Output Contract

Return results in this order:

1. `Rust Work Mode`
2. `Executable Requirements`
3. `Rust Design (L1/L2/L3 + Traits)`
4. `Verification Matrix`
5. `Implementation Plan`
6. `Quality Gate Results`
7. `Open Questions`

Use this traceability shape:

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |

## Guardrails

- Keep invalid states unrepresentable with strong types where practical.
- Treat async, cancellation, and concurrency as correctness problems, not only performance concerns.
- Avoid forcing every local convention onto every Rust task; use the lightest rule set that preserves correctness and compatibility.
- Do not recommend unsafe, lock-free, or proc-macro-heavy patterns as defaults unless the task truly requires them.
- Keep reviews skeptical about malformed input, non-UTF-8 boundaries, backpressure, cancellation, and API evolution.

## Context Strategy

Load progressively:

1. Read [references/reference-map.md](./references/reference-map.md) first.
2. Read [references/rust-executable-specs-playbook.md](./references/rust-executable-specs-playbook.md) for vague requests, new features, TDD planning, and L1/L2/L3 design work.
3. Read [references/rust-reliability-reference.md](./references/rust-reliability-reference.md) for public API, error, async, concurrency, unsafe, FFI, and release-hardening questions.
4. Read [references/rust-conventions-and-gates.md](./references/rust-conventions-and-gates.md) before final verification and when project-local conventions might matter.

For large files, prefer heading search such as:

- `rg '^##|^###' references/rust-reliability-reference.md`
- `rg '^##|^###' references/rust-executable-specs-playbook.md`

## References

- [Reference map](./references/reference-map.md)
- [Rust executable specs playbook](./references/rust-executable-specs-playbook.md)
- [Rust reliability reference](./references/rust-reliability-reference.md)
- [Rust conventions and gates](./references/rust-conventions-and-gates.md)
