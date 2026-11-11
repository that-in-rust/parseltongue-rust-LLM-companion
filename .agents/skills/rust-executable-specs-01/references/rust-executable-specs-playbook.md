# Rust Executable Specs Playbook

Use this playbook when the Rust task needs executable requirements, L1/L2/L3 design, trait seams, TDD sequencing, or a concrete verification plan before implementation.

## Table of Contents

1. Rust Work Packet Template
2. Requirement Contract Template
3. L1/L2/L3 Planning Scaffold
4. Trait-First Interface Template
5. Verification Matrix Template
6. TDD Loop
7. Rust Test Skeletons
8. Vague-to-Executable Rewrite Patterns
9. Review Questions

## 1) Rust Work Packet Template

Use this output shape when the request is not ready for immediate coding:

```markdown
## Rust Work Mode
- Spec Mode
- Delivery Mode

## Executable Requirements
### REQ-RUST-001.0: <title>
**WHEN** ...
**THEN** the system SHALL ...
**AND** SHALL ...
**SHALL** ...

## Rust Design (L1/L2/L3 + Traits)
- L1 core:
- L2 std:
- L3 external:
- Trait seams:

## Verification Matrix
| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |

## Implementation Plan
1. STUB
2. RED
3. GREEN
4. REFACTOR
5. VERIFY

## Quality Gate Results
- fmt:
- clippy:
- tests:
- build:
```

## 2) Requirement Contract Template

```markdown
### REQ-RUST-001.0: <short title>

**WHEN** <trigger or input condition>
**THEN** the system SHALL <primary behavior>
**AND** SHALL <secondary measurable behavior>
**SHALL** <edge-case or default behavior>
```

Authoring rules:

- Keep one independently testable behavior per `SHALL` line.
- Use explicit units for latency, memory, throughput, retries, or concurrency limits.
- Split functional requirements from non-functional requirements when they would otherwise blur together.
- Use `REQ-RUST-*` for Rust-specific packets even when the user starts with a generic request.

## 3) L1/L2/L3 Planning Scaffold

Use this scaffold before coding:

| layer | role | allowed dependencies | design question |
| --- | --- | --- | --- |
| `L1 Core` | domain invariants, types, traits, protocol/state rules | `core` or dependency-light standard types only | what must stay stable, testable, and runtime-agnostic? |
| `L2 Std` | collections, ownership orchestration, error composition, iterators, sync primitives | Rust stdlib | what orchestration belongs in Rust std without framework coupling? |
| `L3 External` | async runtime, database, HTTP, serialization, tracing, FFI adapters | external crates and runtime integration | what touches the outside world and should stay behind traits/adapters? |

Planning rules:

- Keep `L3` dependencies out of `L1`.
- Put invariants and validated types as low in the stack as practical.
- Add a trait seam when infrastructure, time, randomness, or external IO needs substitution in tests.

## 4) Trait-First Interface Template

```rust
pub trait EntityStoreBehavior {
    fn load_entity_by_id(
        &self,
        entity_id: EntityId,
    ) -> Result<Option<StoredEntity>, EntityStoreError>;
}

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

Use trait-first design when:

- IO or adapters should be isolated from domain logic
- the task needs test doubles or integration seams
- API behavior should be settled before storage or transport details

## 5) Verification Matrix Template

```markdown
| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| REQ-RUST-001.0 | TEST-UNIT-001 | unit | validates parsed entity ids | correctness |
| REQ-RUST-001.0 | TEST-INTEG-002 | integration | persists and reloads valid entities | integration |
| REQ-RUST-002.0 | TEST-DOC-003 | doctest | public example compiles and behaves as documented | usability |
| REQ-RUST-003.0 | TEST-CF-004 | compile-fail | rejects invalid generic usage | misuse |
| REQ-RUST-004.0 | TEST-PROP-005 | property | round-trip preserves invariant | invariant |
| REQ-RUST-005.0 | TEST-LOOM-006 | concurrency | shared state remains race-safe | safety |
| REQ-RUST-006.0 | TEST-PERF-007 | performance | p99 < 500us for 10k entities | latency |
```

Choose the smallest useful set, not the biggest possible set.

## 6) TDD Loop

```markdown
1. STUB
- Write tests for each `REQ-RUST-*`.
- Define fixtures, expected outputs, and interface signatures.

2. RED
- Run tests and confirm the failure reason matches the missing behavior.

3. GREEN
- Implement the minimum code that satisfies the failing tests.

4. REFACTOR
- Improve naming, decomposition, and ownership ergonomics while tests stay green.

5. VERIFY
- Run formatting, lints, tests, build, and any higher-risk gates such as doctest, compile-fail, property, fuzz, `loom`, or coverage.
```

## 7) Rust Test Skeletons

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
async fn test_req_rust_002_pipeline_integration() -> anyhow::Result<()> {
    let fixture = create_pipeline_fixture_valid_case().await?;
    let output = run_processing_pipeline_async(&fixture).await?;
    assert_eq!(output.status, ProcessingStatus::Completed);
    Ok(())
}
```

### Doctest

```rust
/// ```
/// let user_id = UserId::try_from("user_123")?;
/// assert_eq!(user_id.as_str(), "user_123");
/// # Ok::<(), UserIdError>(())
/// ```
```

### Compile-Fail

```rust
#[test]
fn test_req_rust_003_invalid_usage_compile_fails() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/invalid_usage.rs");
}
```

### Property

```rust
proptest! {
    #[test]
    fn test_req_rust_004_round_trip_invariant(input in valid_entity_strategy()) {
        let encoded = encode_entity_for_wire(&input)?;
        let decoded = decode_entity_from_wire(&encoded)?;
        prop_assert_eq!(decoded, input);
    }
}
```

### Concurrency

```rust
#[cfg(loom)]
#[test]
fn test_req_rust_005_shared_state_loom_model() {
    loom::model(|| {
        // model the smallest interesting concurrent interaction here
    });
}
```

### Performance Contract

```rust
#[test]
fn test_req_rust_006_performance_contract() {
    use std::time::{Duration, Instant};
    let input = create_test_entities_large_set(10_000);
    let start = Instant::now();
    let _ = filter_implementation_entities_only(&input);
    assert!(start.elapsed() < Duration::from_micros(500));
}
```

## 8) Vague-to-Executable Rewrite Patterns

| vague statement | executable rewrite |
| --- | --- |
| "Make it faster." | "Reduce p99 latency from 5ms to less than 1ms for 10k entities." |
| "Improve reliability." | "Return typed validation errors for malformed input without panics or detached task leaks." |
| "Handle async correctly." | "Propagate cancellation, avoid blocking in async contexts, and keep lock guards out of `.await`." |
| "Make the API cleaner." | "Accept borrowed inputs, preserve semver-safe public types, and reject invalid states at construction time." |

## 9) Review Questions

- Which modes are active for this task, and which one is primary?
- What is the smallest set of `REQ-RUST-*` contracts that makes the work testable?
- Which types and invariants belong in `L1` instead of leaking into adapters?
- Which failure modes deserve compile-fail, property, fuzz, or `loom` coverage instead of unit tests alone?
- Which public names must stay stable even if a four-word alias would be cleaner for new code?
