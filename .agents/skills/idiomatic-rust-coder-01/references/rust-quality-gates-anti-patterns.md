# Rust Quality Gates and Anti-Patterns

Source basis:
- `/Users/amuldotexe/Downloads/rust-coder-01 (1).md`
- `/Users/amuldotexe/Downloads/S77-IdiomaticRustPatterns.md`

Run these checks before finalizing Rust changes.

## 1) Fatal Anti-Patterns

Do not ship code with these patterns:

1. Ambiguous requirements with no measurable assertions.
2. Monolithic "god object" structs with mixed responsibilities.
3. Performance claims without benchmark or contract tests.
4. Hard concrete dependencies that block unit testing.
5. Resource lifecycle leaks or missing cleanup guarantees.
6. Oversimplified domain models that ignore real constraints.
7. Layer violations (L3 dependencies leaking into L1 core).
8. Async/concurrency logic with no stress or race validation.

## 2) Requirement and Test Traceability

- Assign every requirement a stable ID: `REQ-RUST-<NNN>.<REV>`.
- Link each requirement to one or more tests.
- Fail review if any requirement lacks verification.

## 3) Naming and Structure Gate

- Use four-word naming for new symbols when feasible.
- Keep modules single-purpose and understandable.
- Preserve existing public API contracts when compatibility matters.

## 4) Runtime and Safety Gate

- Verify no blocking I/O or CPU-bound work runs directly on async executors.
- Verify no locks are held across `.await`.
- Verify cancellation and timeout paths are handled.
- Verify no secret values are logged.

## 5) Verification Command Gate

Run and require success:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --all-targets --all-features
```

Optional advanced gates for high-risk changes:

```bash
cargo test --release
cargo llvm-cov --workspace --branch --fail-under-lines 75
```

## 6) Release Readiness Checklist

- [ ] Requirements are executable and measurable.
- [ ] Tests validate functional, error, and edge behavior.
- [ ] Performance claims include explicit threshold tests.
- [ ] Documentation and examples reflect current behavior.
- [ ] No TODO/STUB/FIXME placeholders in finalized code.
