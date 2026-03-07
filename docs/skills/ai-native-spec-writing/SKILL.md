---
name: ai-native-spec-writing
description: Write executable specifications, requirements, and design documents using AI-native patterns. Covers Four-Word Naming Convention (4WNC), WHEN/THEN/SHALL acceptance criteria, TDD-first spec cycle, contract-driven development, and one-feature-per-version release philosophy. Use this skill before writing any code.
---

# AI-Native Spec Writing

## Overview

Use this skill to write specifications that LLMs can implement accurately. Traditional user stories fail LLMs because they are designed for human conversation. This skill transforms ambiguous requirements into executable blueprints with testable contracts.

Based on 400+ hours of LLM-assisted development with measured results:

| Metric | Before | After | Improvement |
|--------|--------|-------|----------------|
| Compile attempts (avg) | 4.9 | 1.6 | 67% faster |
| Production bugs | 1 per 100 LOC | 1 per 1000 LOC | 90% reduction |
| Context accuracy | ~60% | ~95% | 58% improvement |

## When To Use

Use this skill when:
- Writing requirements or acceptance criteria for a new feature.
- Designing interfaces, traits, or module contracts before implementation.
- Reviewing a spec or PRD for LLM-implementability.
- Naming new functions, crates, commands, or folders.
- Planning a version release.

Do not use this skill for implementation details or coding patterns (use `idiomatic-rust-coding` instead).

## Core Principles

### Principle 1: LLMs Are Search Tools

LLMs do not "program" -- they search training data and assimilate patterns. You must bias them with the right keywords through structured naming and explicit context.

### Principle 2: Iteration Is Required

First outputs are rarely optimal. Plan for multi-round refinement:
- Round 1: Broad exploration
- Round 2: Constraint application
- Round 3: Refinement and optimization
- Round 4: Final verification

### Principle 3: Tests Are Executable Specifications

Tests transform ambiguous requirements into precise contracts. When you write tests first, you create an easier prediction problem for the LLM.

## Workflow

### Phase 1: Requirements (WHEN/THEN/SHALL Format)

Transform every requirement into a testable contract:

```markdown
### REQ-MVP-001.0: Entity Filtering Performance

**WHEN** I run `filter_implementation_entities_only()` with 10,000 entities
**THEN** the system SHALL return results in < 500us at p99
**AND** SHALL allocate < 100KB memory
**SHALL** return empty vec when no matches (not null)
```

Rules:
- Tag every requirement with an ID (REQ-MVP-001.0) for traceability.
- Replace ambiguous language ("faster", "better", "easier") with measurable criteria.
- Every performance claim must have a corresponding test.

### Phase 2: Design (Contract-Driven)

Write interface contracts with preconditions, postconditions, and error conditions:

```rust
/// Message creation with deduplication contract
///
/// # Preconditions
/// - User authenticated with room access
/// - Content: 1-10000 chars, sanitized HTML
/// - client_message_id: valid UUID
///
/// # Postconditions
/// - Returns Ok(Message<Persisted>) on success
/// - Inserts row into 'messages' table
/// - Deduplication: returns existing if client_message_id exists
///
/// # Error Conditions
/// - MessageError::Authorization if user lacks room access
/// - MessageError::InvalidContent if content violates constraints
/// - MessageError::Database on persistence failure
pub async fn create_message_with_deduplication(
    &self,
    content: String,
    room_id: RoomId,
    user_id: UserId,
    client_message_id: Uuid,
) -> Result<Message<Persisted>, MessageError>;
```

Include test plans alongside every interface:

```rust
/// # Test Plan
///
/// Scenario 1: Successful creation
/// Given: valid user in room and valid content
/// When: create_message_with_deduplication is called
/// Then: returns Ok(Message<Persisted>)
///
/// Scenario 2: Deduplication
/// Given: message with client_message_id X already exists
/// When: new message with same client ID X is created
/// Then: returns Ok(existing Message) - no duplicate created
```

### Phase 3: Implementation Spec (TDD Cycle)

Structure implementation tasks as STUB -> RED -> GREEN -> REFACTOR:

1. **STUB**: Write the test first. The test defines the interface and expected behavior.
2. **RED**: Run the test. Verify it fails for the right reason (compiler error or assertion).
3. **GREEN**: Write the minimal implementation to make the test pass.
4. **REFACTOR**: Improve the code while keeping tests green.

### Phase 4: Pre-Commit Verification

Before any commit, verify:
- All function/crate/command names follow 4WNC (exactly 4 words).
- All tests pass (`cargo test --all`).
- Build passes (`cargo build --release`).
- Zero TODOs, STUBs, or PLACEHOLDERs in committed code.
- No `unwrap()`/`expect()` in production code.

## Four-Word Naming Convention (4WNC)

The single highest-impact pattern. All names must be exactly 4 words.

**Formula**: `verb_constraint_target_qualifier()`

| Position | Purpose | Examples |
|----------|---------|----------|
| **verb** | Action | `filter`, `render`, `detect`, `save`, `create` |
| **constraint** | Scope | `implementation`, `box_with_title`, `visualization_output` |
| **target** | Operand | `entities`, `unicode`, `file`, `database` |
| **qualifier** | Specificity | `only`, `to`, `from`, `with`, `async` |

```rust
// Functions: snake_case, 4 words
filter_implementation_entities_only()    // Good
render_box_with_title_unicode()          // Good
save_visualization_output_to_file()      // Good

// Crates: hyphen-separated, 4 words
pt01-folder-to-cozodb-streamer           // Good
pt08-http-code-query-server              // Good

// Bad examples
filter_entities()                        // Too short (2 words)
detect_cycles_in_dependency_graph()      // Too long (5 words)
```

## One-Feature-Per-Version Philosophy

Each version delivers EXACTLY ONE complete feature, fully working end-to-end:
- Feature works in production binary.
- All tests passing (not just new feature).
- Documentation updated.
- Shell scripts updated.
- Integration tested.
- Zero TODOs, zero stubs, zero placeholders.

Forbidden:
- Partial features ("foundation but no integration").
- Stubs for "future work".
- Breaking existing features.
- Documentation saying "will be implemented".

## Diagrams

ALL diagrams must be in Mermaid format for GitHub compatibility. Use vertical orientation (TB), keep to 4-6 nodes per diagram, and use line breaks for readability.

## LLM Response Contract

When writing specs with this skill, deliver:
- Requirements in WHEN/THEN/SHALL format with IDs.
- Interface contracts with preconditions, postconditions, error conditions.
- Test plans for every interface.
- All names validated against 4WNC.
- Mermaid diagrams where architecture needs visualization.

## Anti-Patterns to Avoid

- Ambiguous specifications ("As a user, I want better performance").
- Unsubstantiated performance claims (no test to verify).
- Requirements without IDs (breaks traceability).
- Mixing spec writing with implementation details.
- Skipping the TDD cycle (writing code before tests).

## Resources

- Source: `agent-room-of-requirements/agents-used-202512/notes01-agent.md` (Parts I-VIII, X)
- Source: `agent-room-of-requirements/agents-used-202512/rust-coder-01.md` (Layers 1-9)
- Related skill: `idiomatic-rust-coding` (for implementation patterns)
- Related skill: `context-engineering-agents` (for agent design patterns)
