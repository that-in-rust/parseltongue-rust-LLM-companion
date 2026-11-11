# Rust Conventions and Gates

Use this file near the end of planning, implementation, or review to apply project-local conventions selectively and to enforce final Rust quality gates.

## 1) Selective Local Conventions

- Use four-word naming for new functions and commands when feasible.
- Apply the same clarity rule to new crates, folders, and top-level commands when creating new surfaces is practical.
- Preserve existing public API compatibility when renaming is risky; add wrappers or adapters when that reduces breakage.
- Keep modules cohesive and single-purpose.
- Treat Mermaid as a documentation convention only when the project already standardizes on it. Do not apply it as a universal Rust rule.

## 2) Implementation Conventions

- Prefer `&[T]`, `&str`, or `AsRef<T>` in public APIs over concrete container references.
- Use newtypes for identifiers and invariants.
- Use `thiserror` for library-facing error enums and `anyhow` at application orchestration boundaries.
- Avoid `unwrap` and `expect` in production paths unless an invariant is documented and justified.
- Keep docs, examples, and tests synchronized with behavior changes.

## 3) Runtime and Safety Gates

- Verify no blocking IO or CPU-heavy work runs directly on async executors.
- Verify no mutex, rwlock, or mutable borrow is held across `.await`.
- Verify cancellation, timeout, or shutdown paths are handled when async work can outlive the caller.
- Verify no secrets or sensitive payloads are logged.
- Keep unsafe code tightly fenced and document the invariants it relies on.

## 4) Verification Command Gate

Run and require success:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --all-targets --all-features
```

Optional advanced gates for higher-risk changes:

```bash
cargo test --release
cargo llvm-cov --workspace --branch --fail-under-lines 75
```

## 5) Release Readiness Checklist

- [ ] Requirements are executable and measurable.
- [ ] Each `REQ-RUST-*` has at least one linked test.
- [ ] Tests validate functional, error, and edge behavior.
- [ ] Performance claims include explicit thresholds and test method.
- [ ] Documentation and rustdoc examples reflect current behavior.
- [ ] No `TODO`, `STUB`, or `FIXME` placeholders remain in finalized code.

## 6) Anti-Patterns to Reject

- Ambiguous requirements with no measurable assertions.
- Monolithic structs or modules with mixed responsibilities.
- Hard concrete dependencies that block unit or integration testing.
- L3 dependencies leaking into L1 core.
- Blocking work or lock guards crossing `.await`.
- Public APIs that return `anyhow::Error` from library boundaries.
- Concurrency code with no stress, model, or cancellation validation.

## 7) Rules Not to Universalize

Do not treat these as default Rust quality rules:

- "Run `cargo clean` constantly."
- Repeated duplicated doctrine copied verbatim into every skill or plan.
- Four-word naming when it would break stable APIs or make names materially worse.
- Mermaid-only documentation as a universal Rust requirement.
- MVP language used as cover for skipping tests, contracts, or verification.
