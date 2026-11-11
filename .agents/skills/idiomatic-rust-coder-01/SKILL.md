---
name: idiomatic-rust-coder-01
description: Design and implement Rust features with executable specifications, TDD-first delivery, and idiomatic architecture. Use when Codex is asked to write or refactor Rust crates, services, CLIs, async workflows, or embedded components and must enforce requirement traceability, layered architecture across L1, L2, and L3 boundaries, four-word naming discipline for new symbols, and test-validated performance and concurrency claims.
---

# Idiomatic Rust Coder 01

Use this skill to turn Rust requests into production-ready plans and code constraints that compile, test, and scale.

## Workflow

1. Convert request to executable contracts.
- Write `REQ-RUST-<NNN>.<REV>` identifiers.
- Express behavior as `WHEN...THEN...SHALL`.
- Reject ambiguous terms such as "faster" without thresholds.

2. Define architecture before coding.
- Separate boundaries into L1 core, L2 std, L3 external.
- Keep L3 dependencies out of L1.
- Define traits for dependency injection before concrete implementations.

3. Plan tests first (TDD-first).
- Execute `STUB -> RED -> GREEN -> REFACTOR -> VERIFY`.
- Cover unit, integration, error-path, and concurrency cases.
- Add performance tests only when metrics are specified.

4. Apply idiomatic Rust constraints.
- Prefer expression-oriented style and iterators over mutable accumulator code.
- Accept `&[T]`, `&str`, or `AsRef` in public APIs instead of `&Vec<T>` or `&String`.
- Use newtypes for domain identifiers and invariants.
- Use `thiserror` for library error enums and `anyhow` for app composition boundaries.
- Avoid `unwrap` and `expect` in production paths.
- Avoid blocking calls in async contexts.
- Avoid holding locks across `.await`.

5. Enforce naming and module discipline.
- Use four-word names for new functions and commands when feasible.
- Apply the same clarity rule to new crates, folders, and top-level commands when renaming or creating new surfaces is practical.
- Keep modules focused and cohesive.
- Preserve compatibility when existing public APIs cannot be renamed; add adapters if needed.

6. Run verification gates.
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`
- `cargo build --all-targets --all-features`

## Output Contract

Return results in this order:

1. `Executable Requirements`
2. `Rust Design (L1/L2/L3 + Traits)`
3. `Test Matrix`
4. `Implementation Plan`
5. `Quality Gate Results`
6. `Open Questions`

Use requirement-to-test traceability:

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |

## Guardrails

- Keep invalid states unrepresentable with strong types.
- Require measurable evidence for performance claims.
- Stress-test shared state and cancellation paths for async/concurrency code.
- Keep docs and tests synchronized with behavior changes.
- Prefer battle-tested crates over speculative abstractions for MVP work.

## Context Strategy

Load resources progressively:

1. Load this `SKILL.md`.
2. Load [Rust executable specs templates](./references/rust-executable-specs-templates.md) for contract and test scaffolds.
3. Load [Rust idioms checklist](./references/rust-idioms-checklist.md) for implementation decisions.
4. Load [Rust quality gates and anti-patterns](./references/rust-quality-gates-anti-patterns.md) before final verification.

## References

- [Rust executable specs templates](./references/rust-executable-specs-templates.md)
- [Rust idioms checklist](./references/rust-idioms-checklist.md)
- [Rust quality gates and anti-patterns](./references/rust-quality-gates-anti-patterns.md)
