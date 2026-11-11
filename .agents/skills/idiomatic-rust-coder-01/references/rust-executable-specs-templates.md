# Rust Executable Specs Templates

Source basis:
- `/Users/amuldotexe/Downloads/rust-coder-01 (1).md`
- `/Users/amuldotexe/Downloads/idiomatic_rust_tdd_patterns.md`

Use these templates to convert requests into testable Rust work packets.

## Table of Contents

1. Requirement Contract Template
2. Trait-First Interface Template
3. TDD Loop Template
4. Test Matrix Template
5. Rust Test Skeletons
6. Verification Commands

## 1) Requirement Contract Template

```markdown
### REQ-RUST-001.0: <short title>

**WHEN** <input condition or trigger>
**THEN** the system SHALL <primary behavior>
**AND** SHALL <measurable constraint>
**SHALL** <edge-case/default behavior>
```

Rules:
- Keep each `SHALL` independently testable.
- Add explicit units (`us`, `ms`, `MB`, `ops/s`) for non-functional claims.

## 2) Trait-First Interface Template

```rust
pub trait EntityProcessorBehavior {
    fn process_domain_entities_only(
        &self,
        input: &[DomainEntity],
    ) -> Result<Vec<ProcessedEntity>, ProcessingError>;
}

pub struct EntityProcessorService<S: EntityStoreBehavior> {
    store: std::sync::Arc<S>,
}
```

## 3) TDD Loop Template

```markdown
1. STUB
- Write tests for each REQ id.
- Define interface signatures and fixtures.

2. RED
- Run tests and confirm expected failures.

3. GREEN
- Implement the minimal code that satisfies failing tests.

4. REFACTOR
- Improve naming, decomposition, and ergonomics while tests stay green.

5. VERIFY
- Run formatting, lints, tests, and build.
```

## 4) Test Matrix Template

```markdown
| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| REQ-RUST-001.0 | TEST-UNIT-001 | unit | filters only implementation entities | correctness |
| REQ-RUST-001.0 | TEST-INTEG-003 | integration | preserves order across parser and filter | correctness |
| REQ-RUST-001.0 | TEST-PERF-002 | perf | p99 < 500us for 10k entities | latency |
| REQ-RUST-001.0 | TEST-CONC-004 | concurrency | no race under 100 parallel tasks | safety |
```

## 5) Rust Test Skeletons

### Unit

```rust
#[test]
fn test_req_rust_001_filter_implementation_entities_only() {
    let input = create_test_entities_mixed_set();
    let output = filter_implementation_entities_only(&input);
    assert!(output.iter().all(DomainEntity::is_implementation));
}
```

### Async Integration

```rust
#[tokio::test]
async fn test_req_rust_001_pipeline_integration() -> anyhow::Result<()> {
    let fixture = create_pipeline_fixture_valid_case().await?;
    let output = run_processing_pipeline_async(&fixture).await?;
    assert_eq!(output.status, ProcessingStatus::Completed);
    Ok(())
}
```

### Performance Contract

```rust
#[test]
fn test_req_rust_001_performance_contract() {
    use std::time::{Duration, Instant};
    let input = create_test_entities_large_set(10_000);
    let start = Instant::now();
    let _ = filter_implementation_entities_only(&input);
    assert!(start.elapsed() < Duration::from_micros(500));
}
```

### Concurrency Contract

```rust
#[tokio::test]
async fn test_req_rust_001_concurrency_contract() -> anyhow::Result<()> {
    use futures::future::join_all;
    let shared = std::sync::Arc::new(create_shared_processor_state());
    let tasks = (0..100).map(|_| {
        let shared = std::sync::Arc::clone(&shared);
        tokio::spawn(async move { shared.process_safely_async().await })
    });
    let results = join_all(tasks).await;
    assert!(results.into_iter().all(|r| r.is_ok()));
    Ok(())
}
```

## 6) Verification Commands

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --all-targets --all-features
```
