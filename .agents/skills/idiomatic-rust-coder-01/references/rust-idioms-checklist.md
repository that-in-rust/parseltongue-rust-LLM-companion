# Rust Idioms Checklist

Source basis:
- `/Users/amuldotexe/Downloads/S77-IdiomaticRustPatterns.md`
- `/Users/amuldotexe/Downloads/idiomatic_rust_tdd_patterns.md`

Use this checklist while writing or reviewing Rust code.

## 1) API and Type Design

- Prefer `&[T]` and `&str` in public APIs over `&Vec<T>` and `&String`.
- Use `AsRef<T>` where ergonomic conversions improve call sites.
- Use newtypes for identifiers and invariants.
- Use `From` for infallible conversions and `TryFrom` for fallible conversions.
- Keep invalid states unrepresentable with enums and domain types.

## 2) Error Handling

- Use `thiserror` for library-facing error enums.
- Use `anyhow` at application orchestration boundaries.
- Propagate errors with `?` and attach context at I/O or boundary operations.
- Avoid `unwrap` and `expect` in production paths.

## 3) Async and Concurrency Hygiene

- Avoid blocking calls in async contexts; use `spawn_blocking` for CPU-bound work.
- Avoid holding mutex or rwlock guards across `.await`.
- Use bounded channels for backpressure.
- Support cancellation paths for long-running async operations.
- Validate shared-state behavior with concurrent stress tests.

## 4) Ownership and Memory

- Prefer borrowing over cloning in hot paths.
- Use iterator pipelines for data transforms when they improve clarity.
- Keep lock scope small and explicit.
- Choose `Arc` for shared multi-threaded ownership and `Rc` for single-threaded.

## 5) Architecture Boundaries

- Keep L1 core free of L3 dependencies.
- Keep domain logic testable without network, database, or runtime dependencies.
- Use traits for dependency injection where test seams are needed.
- Keep modules cohesive and purpose-specific.

## 6) Testing Depth

- Start with TDD: `STUB -> RED -> GREEN -> REFACTOR`.
- Cover happy path, error path, and edge path for each requirement.
- Add property-based tests for invariant-heavy logic.
- Add integration tests for cross-module workflows.
- Add performance tests only when performance constraints exist.

## 7) Tooling Gates

- Run `cargo fmt --all -- --check`.
- Run `cargo clippy --all-targets --all-features -- -D warnings`.
- Run `cargo test --all-targets --all-features`.
- Run `cargo build --all-targets --all-features`.

## 8) Docs and Observability

- Keep rustdoc examples executable and aligned with current behavior.
- Use structured tracing at system boundaries in async services.
- Avoid logging secrets or sensitive payloads.

## 9) High-Value Review Questions

- Does this API expose concrete containers unnecessarily?
- Are error variants actionable and contextualized?
- Are any locks or borrows held too long?
- Does async code block the runtime?
- Do tests prove every major claim in the implementation?
