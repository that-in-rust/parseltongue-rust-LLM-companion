---
name: notes01-agent
description: notes-agent-01
model: sonnet
color: green
---

# AI-Native Coding: The Meta-Patterns Reference

> **A comprehensive guide to writing software in the age of Large Language Models**
>
> Based on 400+ hours of LLM-assisted development across Rust, TypeScript, Go, and Python
>
> synthesizing 45+ meta-patterns, 2,500+ system design patterns, and production-tested methodologies

---

```mermaid
flowchart TB
    subgraph THESIS["The AI-Native Thesis"]
        MAIN[AI-native engineering achieves<br/>67% faster development through<br/>structured naming, TDD-First,<br/>and executable specifications]
    end

    MAIN --> P1[Principle 1:<br/>LLMs as Search Tools]
    MAIN --> P2[Principle 2:<br/>Iteration Required]
    MAIN --> P3[Principle 3:<br/>Summary Documentation]
    MAIN --> P4[Principle 4:<br/>Self-Reasoning]
    MAIN --> P5[Principle 5:<br/>Anti-Pattern Reference]
    MAIN --> P6[Principle 6:<br/>TDD Mindset]
    MAIN --> P7[Principle 7:<br/>Cumulative Learning]

    P1 --> 4WNC[Four-Word Naming<br/>67% faster]
    P6 --> TDD[TDD-First Cycle<br/>90% fewer bugs]
    P3 --> GRAPH[Graph-Based Context<br/>Zero hallucinations]
```

---

# Part I: Executive Summary

## The Main Conclusion

**AI-native engineering requires fundamentally rethinking how we write software—treating LLMs as search and assimilation tools that must be explicitly biased with structured naming, test-driven specifications, and iterative reasoning.** By adopting the Four-Word Naming Convention (4WNC), TDD-First workflows, and executable specifications, teams achieve **67% faster development cycles** with **90% fewer production bugs**.

This is not theoretical. These numbers come from measured production data:

| Metric | Before Patterns | After Patterns | Improvement |
|--------|----------------|----------------|-------------|
| Compile attempts (avg) | 4.9 | 1.6 | 67% faster |
| Production bugs | 1 per 100 LOC | 1 per 1000 LOC | 90% reduction |
| Context accuracy | ~60% | ~95% | 58% improvement |
| Developer onboarding | 2-3 weeks | 3-5 days | 70% faster |

---

## The Seven Core Principles

These principles emerged from 400+ hours of LLM-assisted development. They are not hypotheses—they are battle-tested patterns that work consistently across programming languages and domains.

```mermaid
flowchart TB
    subgraph PRINCIPLES["The 7 Core Principles of AI-Native Engineering"]
        direction TB
        P1[Principle A<br/>LLMs = Search + Assimilation<br/>Keyword biasing is essential]
        P2[Principle B<br/>Iteration is Required<br/>Refinement through loops]
        P3[Principle C<br/>Summary Documentation<br/>Combat context window loss]
        P4[Principle D<br/>Self-Reasoning<br/>Rubber duck debugging]
        P5[Principle E<br/>Anti-Pattern Reference<br/>Learn from past failures]
        P6[Principle F<br/>TDD Mindset<br/>Tests as executable specs]
        P7[Principle G<br/>Cumulative Documentation<br/>Prompt engineering as memory]
    end

    P1 --> OUT[Predictable LLM Performance]
    P2 --> OUT
    P3 --> OUT
    P4 --> OUT
    P5 --> OUT
    P6 --> OUT
    P7 --> OUT
```

### Principle (a): LLMs are Search + Assimilation Tools

**LLMs do not "program" in the human sense.** They search through training data, retrieve relevant patterns, and assimilate them into coherent responses. This fundamental misunderstanding causes most frustration with AI coding tools.

**The Implication:** You must bias LLMs with the right keywords. When you name a function `filter_implementation_entities_only()`, you're not just naming for humans—you're creating optimal retrieval tokens for the LLM.

**The Pattern:**
- Treat prompts as search queries
- Use consistent, semantic naming
- Provide structured context, not raw dumps
- Think in terms of "relevance signals"

### Principle (b): LLMs Need Iteration

**First outputs are rarely optimal.** LLMs require iterative refinement to arrive at high-quality code. Each iteration adds context and constraint, converging toward correctness.

**The Pattern:**
- Round 1: Broad exploration
- Round 2: Constraint application
- Round 3: Refinement and optimization
- Round 4: Final verification

**Anti-Pattern:** Expecting perfection from a single prompt.

### Principle (c): Combat Context Window Loss with Summary Documents

**The sliding window forgets.** As conversations extend, early context is lost. Summary documents serve as "memory checkpoints" that maintain continuity.

**The Pattern:**
- Create summary documents after each major milestone
- Keep summaries synchronized with code state
- Reference summaries in new prompts
- Treat documentation as executable memory

### Principle (d): Make LLMs Reason Explicitly

**LLMs need to find their own logical fallacies.** By asking LLMs to "think step by step" or "find flaws in this reasoning," you enable rubber duck debugging at scale.

**The Pattern:**
- Ask for chain-of-thought explanations
- Request self-critique: "What's wrong with this approach?"
- Use "verify your assumptions" prompts
- Make reasoning explicit, not implicit

### Principle (e): Reference Anti-Patterns Explicitly

**Past bugs are gold mines.** By providing reference files of anti-patterns to avoid, you prevent LLMs from repeating known mistakes.

**The Pattern:**
- Maintain `avoidBustBugs.txt` or similar files
- Document past failures with root causes
- Include anti-patterns in project context
- Update after each bug encounter

### Principle (f): TDD Makes LLM Prediction Easier

**Tests transform ambiguous requirements into precise contracts.** When you write tests first, you create an easier prediction problem: the LLM knows exactly what to build.

**The Pattern:**
- STUB → RED → GREEN → REFACTOR cycle
- Tests as executable specifications
- WHEN...THEN...SHALL format for requirements
- Performance claims backed by automated tests

### Principle (g): LLM Whispering is Cumulative

**Document everything.** Every prompt, every iteration, every learning should be documented. Prompt engineering becomes organizational memory.

**The Pattern:**
- Maintain prompt libraries
- Document what works vs. what fails
- Create reusable prompt templates
- Share learnings across team

---

# Part II: The Four-Word Naming Convention (4WNC)

## Why It Works: The Science Behind 67% Faster Development

The Four-Word Naming Convention is the single highest-impact pattern discovered. It delivers measurable improvements because it aligns with how LLMs actually process information.

```mermaid
flowchart TB
    BAD1[filter<br/>1 word] -->|Too Short| NO1[Low semantic density<br/>LLM cannot infer intent]
    BAD2[filter_entities<br/>2 words] -->|Too Short| NO2[Insufficient context<br/>Multiple interpretations]
    GOOD[filter_implementation_entities_only<br/>4 words] -->|Optimal| YES[Maximum semantic density<br/>Predictable tokenization]
    BAD3[detect_cycles_in_dependency_graph<br/>5 words] -->|Too Long| NO3[Token fragmentation<br/>Dilutes pattern matching]
```

## The Formula: `verb_constraint_target_qualifier()`

Every function, variable, class, and module follows this pattern:

| Position | Purpose | Examples |
|----------|---------|----------|
| **verb** | Action being performed | `filter`, `render`, `detect`, `save`, `create`, `process` |
| **constraint** | Scope or limitation | `implementation`, `box_with_title`, `visualization_output` |
| **target** | What's being operated on | `entities`, `unicode`, `file`, `database`, `context` |
| **qualifier** | Additional specificity | `only`, `to`, `from`, `with`, `async` |

## Examples by Language

```rust
// Rust: snake_case, 4 words
filter_implementation_entities_only()
render_box_with_title_unicode()
save_visualization_output_to_file()
create_database_connection_pool_async()

// ❌ Wrong: filter() (too short)
// ❌ Wrong: detect_cycles_in_dependency_graph() (too long)
```

```typescript
// TypeScript: camelCase, 4 words
filterImplementationEntitiesOnly()
renderBoxWithTitleUnicode()
saveVisualizationOutputToFile()
createDatabaseConnectionPoolAsync()
```

```go
// Go: MixedCaps, exported, 4 words
FilterImplementationEntitiesOnly()
RenderBoxWithTitleUnicode()
SaveVisualizationOutputToFile()
CreateDatabaseConnectionPoolAsync()
```

## The Pre-Commit Ritual

Before every commit, verify:

```bash
# Check all function names
git diff | grep "^+.*fn\|^+.*function\|^+.*def" | while read line; do
    words=$(echo $line | grep -oE "[a-z]+_[a-z]+" | wc -w)
    if [ $words -ne 4 ]; then
        echo "NOT 4 WORDS: $line"
    fi
done
```

## Measured Impact

The Parseltongue project tracked compile attempts before and after adopting 4WNC:

| Phase | Avg Compile Attempts | Development Speed |
|-------|---------------------|------------------|
| Before 4WNC | 4.9 | Baseline |
| After 4WNC | 1.6 | **67% faster** |

**Why:** Consistent naming enables:
- Predictive autocomplete
- Pattern matching across codebase
- Accurate semantic search
- Reduced cognitive load

---

# Part III: TDD-First as Core Mindset

## Why TDD Matters More with LLMs

Test-Driven Development is not optional for AI-native development—it's the primary correction mechanism that prevents LLM hallucinations.

```mermaid
flowchart TB
    subgraph TDD_CYCLE["The TDD-First Cycle"]
        STUB[STUB<br/>Write failing test<br/>with expected interface]
        RED[RED<br/>Run test<br/>verify it fails]
        GREEN[GREEN<br/>Minimal implementation<br/>to pass]
        REFACTOR[REFACTOR<br/>Improve without<br/>breaking tests]
    end

    STUB --> RED --> GREEN --> REFACTOR
    REFACTOR -->|Next feature| STUB

    STUB --> SPEC[Executable Specification]
    SPEC -->|Eliminates| HALLUC[LLM Hallucination]
```

## The Four-Phase Cycle Explained

### STUB Phase

Write the test first. The test defines the interface and expected behavior.

```rust
#[test]
fn test_filter_implementation_entities_only() {
    // Arrange
    let code = "struct Entity { name: String }";
    let entities = parse_entities(code).unwrap();

    // Act
    let result = filter_implementation_entities_only(&entities);

    // Assert
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Entity");
}
```

**Key Point:** The test exists before the implementation. The LLM knows exactly what to build.

### RED Phase

Run the test. Verify it fails for the right reason.

```bash
$ cargo test test_filter_implementation_entities_only
   Compiling...
    Finished test [unoptimized + debuginfo] target(s) in 0.52s
     Running unittests src/lib.rs

error[E0425]: cannot find function `filter_implementation_entities_only` in this scope
  --> src/lib.rs:25:18
   |
25 |     let result = filter_implementation_entities_only(&entities);
   |                    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope
```

**Key Point:** Compiler error is expected. It confirms the interface doesn't exist yet.

### GREEN Phase

Minimal implementation to make the test pass.

```rust
pub fn filter_implementation_entities_only(entities: &[Entity]) -> Vec<Entity> {
    entities
        .iter()
        .filter(|e| e.is_implementation())
        .cloned()
        .collect()
}
```

**Key Point:** Write the simplest code that passes. No optimization yet.

### REFACTOR Phase

Improve the code while keeping tests green.

```rust
pub fn filter_implementation_entities_only(entities: &[Entity]) -> Vec<Entity> {
    entities
        .iter()
        .filter(Entity::is_implementation)
        .cloned()
        .collect()
}
```

**Key Point:** Now we optimize. Tests ensure we didn't break anything.

## Executable Specifications Over Narratives

Traditional user stories fail LLMs because they're "intentionally lightweight" and designed for human conversation.

```markdown
❌ BAD: "As a user, I want better performance"

✅ GOOD:
/// # Performance Contract
///
/// ## Preconditions
/// - Input: Up to 10,000 entities
/// - Each entity: Average 50 dependencies
///
/// ## Postconditions
/// - Returns filtered entities in < 500μs (p99)
/// - Memory allocation: < 100KB
/// - Zero heap allocations in hot path
///
/// ## Test Verification
/// ```rust
/// #[test]
/// fn test_filter_performance_contract() {
///     let entities = create_test_entities(10_000);
///     let start = Instant::now();
///     let result = filter_implementation_entities_only(&entities);
///     let elapsed = start.elapsed();
///     assert!(elapsed < Duration::from_micros(500));
/// }
/// ```
```

## The WHEN...THEN...SHALL Format

Transform requirements into testable contracts:

```markdown
### REQ-MVP-001.0: Entity Filtering

**WHEN** I run `filter_implementation_entities_only()` with a mixed entity list
**THEN** the system SHALL return only entities marked as implementation
**AND** SHALL complete in < 500μs at p99 latency
**AND** SHALL allocate < 100KB memory
**SHALL** return empty vec (not null) when no matches found
```

---

# Part IV: Context Optimization and Smart Selection

## The Context Window Problem

LLM context windows are finite. As conversations extend, early context is lost through the sliding window mechanism. This causes:
- Repetition of earlier points
- Inconsistency with previous decisions
- Loss of important context

**The Solution:** Smart context optimization using graph-based code analysis.

## Parseltongue: Code as Graph

Parseltongue transforms unstructured code into queryable knowledge graphs:

```mermaid
flowchart TB
    subgraph TRANSFORM["Code to Graph Transformation"]
        CODE[Unstructured Code<br/>.rs, .ts, .go files]
        PARSER[Parser<br/>12 languages]
        GRAPH[Knowledge Graph<br/>Entities + Edges]
        API[15 HTTP Endpoints<br/>Queryable]
    end

    CODE --> PARSER --> GRAPH --> API

    API --> Q1[Discovery:<br/>stats, search, list-all]
    API --> Q2[Traversal:<br/>callers, callees, edges]
    API --> Q3[Analysis:<br/>blast-radius, hotspots]
    API --> Q4[Intelligence:<br/>smart-context, budgets]
```

## The Smart Context Algorithm

Instead of dumping raw files into context, use a greedy knapsack algorithm:

```rust
// Relevance scoring for token budget optimization
fn score_entity(entity: &Entity, query: &Query) -> f64 {
    match entity.relation_to(query) {
        Relation::DirectCaller => 1.0,
        Relation::DirectCallee => 0.95,
        Relation::Transitive { depth } => 0.7 - (0.1 * depth as f64),
        Relation::Unrelated => 0.0,
    }
}

// Greedy knapsack: select highest-value entities until budget exhausted
fn select_smart_context(entities: Vec<Entity>, budget: usize) -> Vec<Entity> {
    let mut scored: Vec<_> = entities.into_iter()
        .map(|e| (e, score_entity(&e, &query)))
        .filter(|(_, s)| *s > 0.0)
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut selected = Vec::new();
    let mut used_tokens = 0;

    for (entity, score) in scored {
        let tokens = estimate_tokens(&entity);
        if used_tokens + tokens <= budget {
            selected.push(entity);
            used_tokens += tokens;
        }
    }

    selected
}
```

## Token Budget Guidelines

| Task Type | Token Budget | Selection Strategy |
|-----------|--------------|-------------------|
| Quick explanation | 2,000 | Direct callers/callees only |
| Code review | 4,000 | Add transitive depth 1-2 |
| Bug investigation | 6,000 | Include error handling paths |
| Refactoring | 8,000 | Full blast radius analysis |

## The Orientation Sequence

First contact with a codebase should follow this sequence:

```bash
# Step 1: Discovery - What are we looking at?
GET /api/v1/stats
GET /api/v1/list-all?language=rust

# Step 2: Search - Find the entry point
GET /api/v1/search?query=main

# Step 3: Traversal - Understand connections
GET /api/v1/callers?entity=main
GET /api/v1/callees?entity=main

# Step 4: Analysis - Deep dive
GET /api/v1/blast-radius?entity=main&depth=3
```

**Golden Rule:** Never jump to Analysis without Discovery first.

---

# Part V: Iteration and Self-Reasoning

## Why Iteration is Non-Negotiable

LLMs require iteration to arrive at refined outputs. The first response is a starting point, not a final product.

```mermaid
flowchart TB
    subgraph ITERATION["Iterative Refinement Framework"]
        R1[Round 1<br/>Broad Exploration<br/>Cast wide net]
        R2[Round 2<br/>Constraint Application<br/>Apply requirements]
        R3[Round 3<br/>Refinement<br/>Optimize and polish]
        R4[Round 4<br/>Verification<br/>Final checks]
    end

    R1 --> R2 --> R3 --> R4

    R1 -.->|Self Critique| R2
    R2 -.->|Self Critique| R3
    R3 -.->|Self Critique| R4
```

## Self-Reasoning Patterns

### Pattern 1: Chain-of-Thought Verification

```
Please think step by step and explain:
1. What problem are we solving?
2. What are the constraints?
3. What approach did you take?
4. Why did you choose this approach?
5. What are the potential flaws?
```

### Pattern 2: Explicit Fallacy Detection

```
Review your previous response and identify:
1. Any logical inconsistencies
2. Assumptions that might not hold
3. Edge cases not considered
4. Performance bottlenecks

Be critical. Find what's wrong.
```

### Pattern 3: Rubber Duck Debugging

```
Explain this code to me as if I'm a rubber duck:
- What does each line do?
- Why is it necessary?
- What would break if we removed it?

Start from the top. Don't skip anything.
```

## Reference Anti-Patterns

Maintain files of known anti-patterns:

```markdown
# avoidRustBugs.txt

## Anti-Pattern: Blocking in Async Context

❌ WRONG:
```rust
async fn process_data() {
    std::thread::sleep(Duration::from_secs(1)); // Blocks runtime
}
```

✅ RIGHT:
```rust
async fn process_data() {
    tokio::time::sleep(Duration::from_secs(1)).await; // Yields
}
```

## Anti-Pattern: Ignoring Send/Sync Bounds

❌ WRONG:
```rust
use std::rc::Rc; // Not Send/Sync
fn spawn_task<T>(data: Rc<T>) {
    tokio::spawn(async move { /* data won't compile */ });
}
```

✅ RIGHT:
```rust
use std::sync::Arc; // Send/Sync safe
fn spawn_task<T: Send + Sync>(data: Arc<T>) {
    tokio::spawn(async move { /* works */ });
}
```
```

---

# Part VI: Layered Architecture and Idiomatic Patterns

## The L1 → L2 → L3 Model

Structure systems in layers with clear boundaries:

```mermaid
flowchart TB
    subgraph LAYERS["Layered Architecture"]
        L1[L1 Core<br/>no_std compatible<br/>Ownership, lifetimes, traits]
        L2[L2 Standard<br/>stdlib idioms<br/>Collections, iterators]
        L3[L3 External<br/>ecosystem deps<br/>Async, serialization, DB]
    end

    L1 --> L2 --> L3

    L1 -->|Enables| COMPILE[Compile-First Success<br/>1.6 vs 4.9 attempts]
    L2 -->|Enables| IDIOM[Idiomatic Patterns<br/>Zero-cost abstractions]
    L3 -->|Enables| PROD[Productivity<br/>Battle-tested libraries]
```

### L1 Core (no_std compatible)

- Ownership and borrowing
- Lifetimes
- Traits and generics
- Result<T, E> and Option<T>
- RAII patterns
- Newtype pattern

**Example:**
```rust
// L1: Pure core, no external dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(u64);

impl EntityId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}
```

### L2 Standard (stdlib idioms)

- Collections (Vec, HashMap, BTreeMap)
- Iterators and combinators
- Smart pointers (Rc, Arc)
- Thread safety (Send, Sync)

**Example:**
```rust
// L2: Uses stdlib idioms
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct EntityRegistry {
    by_id: HashMap<EntityId, Arc<Entity>>,
    by_name: HashMap<String, EntityId>,
    implementations: HashSet<EntityId>,
}
```

### L3 External (ecosystem)

- Async/await (Tokio)
- Serialization (Serde)
- Databases (SQLx, CozoDB)
- Web frameworks (Axum)

**Example:**
```rust
// L3: External dependencies
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use axum::Json;

pub struct AppState {
    registry: Arc<RwLock<EntityRegistry>>,
}
```

**Key Rule:** Never mix L3 dependencies into L1 core.

## Dependency Injection for Testability

```rust
// ❌ BAD: Hard dependency
pub struct Service {
    database: PgPool,  // Cannot test without real DB
}

// ✅ GOOD: Trait-based dependency
pub struct Service<D: Database> {
    database: Arc<D>,
}

// Production
type ProductionService = Service<PgDatabase>;

// Testing
type TestService = Service<MockDatabase>;
```

---

# Part VII: Documentation and Communication

## Living Documentation Pattern

Documentation and code stay synchronized through automation:

```mermaid
flowchart TB
    CODE[Code Changes] -->|References| DOCS[Documentation]
    TESTS[Automated Tests] -->|Validates| DOCS
    DOCS -->|Executable Examples| CODE

    DOCS --> DIAGRAMS[Mermaid Diagrams]
    DIAGRAMS -->|Renders In| GITHUB[GitHub README.md]
```

## Mermaid-Only Diagrams

ALL diagrams use Mermaid for GitHub compatibility:

```mermaid
flowchart TB
    subgraph BEST_PRACTICES["Mermaid Best Practices"]
        VERT[✅ Vertical orientation<br/>Use TB, not LR]
        SIZE[✅ Mobile-friendly<br/>4-6 nodes max per diagram]
        NO_HTML[✅ No HTML tags<br/>Use actual line breaks]
        EMOJI[✅ Emojis work great<br/>Use liberally]
    end
```

### Example: Workflow Diagram

```mermaid
flowchart TB
    subgraph WORKFLOW["Feature Development Workflow"]
        PHASE1[Phase 1<br/>Requirements<br/>Executable Specs]
        PHASE2[Phase 2<br/>Design<br/>Test Plans]
        PHASE3[Phase 3<br/>Implementation<br/>TDD Cycle]
        PHASE4[Phase 4<br/>Verification<br/>Pre-Commit]
    end

    PHASE1 --> PHASE2 --> PHASE3 --> PHASE4
    PHASE4 -->|Pass| RELEASE[One Feature Per Version]
    PHASE4 -->|Fail| PHASE3
```

## Low-Drama Communication Style

Technical communication should be:
- **Understated:** Let results speak
- **Direct:** Say what you mean
- **Factual:** Claims backed by data
- **Honest:** Admit uncertainty

```markdown
❌ BAD: "This will revolutionize everything and is the most amazing optimization ever!"

✅ GOOD: "This change reduces query latency from 5ms to 500μs based on benchmark testing."
```

---

# Part VIII: Standard Operating Procedure

## The Complete AI-Native Development Workflow

```mermaid
flowchart TB
    subgraph SOP["Standard Operating Procedure"]
        direction TB

        subgraph REQUIREMENTS["Phase 1: Requirements"]
            R1[Write executable acceptance criteria]
            R2[Tag for traceability<br/>REQ-MVP-001.0]
            R3[Performance claims must be testable]
        end

        subgraph DESIGN["Phase 2: Design"]
            D1[Start with traits/interfaces]
            D2[Include test plans]
            D3[Layer appropriately<br/>L1→L2→L3]
        end

        subgraph IMPLEMENT["Phase 3: Implementation"]
            I1[Write tests FIRST]
            I2[TDD Cycle:<br/>STUB→RED→GREEN→REFACTOR]
            I3[Validate ALL claims]
        end

        subgraph VERIFY["Phase 4: Verification"]
            V1[Pre-commit checklist]
            V2[All names = 4 words]
            V3[Zero TODOs/STUBs]
            V4[Tests pass, builds pass]
        end

        subgraph RELEASE["Phase 5: Release"]
            REL1[One feature per version]
            REL2[End-to-end working]
            REL3[Spic and span<br/>No warnings]
        end
    end

    REQUIREMENTS --> DESIGN --> IMPLEMENT --> VERIFY --> RELEASE
```

## Requirements Phase SOP

### 1. Write Executable Acceptance Criteria

Use the WHEN...THEN...SHALL format:

```markdown
### REQ-MVP-001.0: Entity Filtering Performance

**WHEN** I run `filter_implementation_entities_only()` with 10,000 entities
**THEN** the system SHALL return results in < 500μs at p99
**AND** SHALL allocate < 100KB memory
**SHALL** return empty vec when no matches (not null)

**Verification:**
```rust
#[test]
fn test_req_mvp_001_performance() {
    let entities = create_test_entities(10_000);
    let start = Instant::now();
    let result = filter_implementation_entities_only(&entities);
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_micros(500));
    assert_eq!(result.allocated_bytes(), 0); // Zero heap in hot path
}
```
```

### 2. Tag for Traceability

Reference requirements in tests:

```rust
#[test]
fn test_req_mvp_001_performance() {  // References REQ-MVP-001.0
    // Test validates performance contract
}
```

### 3. Avoid Ambiguous Language

```markdown
❌ BAD: "Make it faster"
❌ BAD: "Improve performance"
❌ BAD: "Optimize"

✅ GOOD: "Reduce latency from 5ms to < 500μs"
✅ GOOD: "Memory allocation must stay under 100KB"
✅ GOOD: "99th percentile latency < 1ms"
```

## Design Phase SOP

### 1. Start with Traits

```rust
// Define interface first
pub trait EntityFilter {
    fn filter_implementation_entities_only(
        &self,
        entities: &[Entity]
    ) -> Vec<Entity>;

    fn filter_by_name_pattern(
        &self,
        entities: &[Entity],
        pattern: &str
    ) -> Result<Vec<Entity>, FilterError>;
}
```

### 2. Include Test Plans

```rust
/// # Test Plan for EntityFilter
///
/// Scenario 1: Successful filtering
/// Given: Mixed entity list with 5 implementations
/// When: filter_implementation_entities_only() called
/// Then: Returns exactly 5 entities
///
/// Scenario 2: Empty input
/// Given: Empty entity list
/// When: filter_implementation_entities_only() called
/// Then: Returns empty vec (not null)
///
/// Scenario 3: Performance contract
/// Given: 10,000 entities
/// When: filter_implementation_entities_only() called
/// Then: Completes in < 500μs
```

### 3. Layer Appropriately

```rust
// L1 Core
pub struct EntityId(u64);

// L2 Standard
pub struct EntityRegistry {
    entities: HashMap<EntityId, Entity>,
}

// L3 External
pub struct AsyncEntityStore {
    pool: PgPool,
}
```

## Implementation Phase SOP

### 1. Write Tests FIRST

```rust
// STUB phase: Test exists, implementation doesn't
#[test]
fn test_filter_implementation_entities_only() {
    let entities = create_test_entities();
    let result = filter_implementation_entities_only(&entities);
    assert!(result.iter().all(|e| e.is_implementation()));
}
```

### 2. Follow TDD Cycle

```bash
# 1. Write test (STUB) → fails to compile
# 2. Run test (RED) → fails as expected
# 3. Implement (GREEN) → minimal code to pass
# 4. Refactor (REFACTOR) → improve while green
```

### 3. Validate ALL Claims

```rust
// If you claim "zero heap allocation", prove it
#[test]
fn test_zero_heap_allocation() {
    let entities = create_test_entities(1000);
    let start = MemoryStats::current();
    let _ = filter_implementation_entities_only(&entities);
    let end = MemoryStats::current();
    assert_eq!(start.allocated, end.allocated);
}
```

## Pre-Commit Checklist (Non-Negotiable)

Before ANY commit, verify:

```bash
#!/bin/bash
# pre-commit-check.sh

echo "🔍 Running pre-commit checks..."

# 1. All function names = 4 words
echo "Checking function names..."
if git diff | grep -E "^\+.*fn\s+[a-z_]+\(" | grep -vE "^\+.*fn\s+[a-z]+_[a-z]+_[a-z]+_[a-z]+\("; then
    echo "❌ Function names must be 4 words!"
    exit 1
fi

# 2. Tests passing
echo "Running tests..."
if ! cargo test --quiet; then
    echo "❌ Tests failing!"
    exit 1
fi

# 3. Build passing
echo "Running build..."
if ! cargo build --quiet; then
    echo "❌ Build failing!"
    exit 1
fi

# 4. Zero TODOs/STUBs
echo "Checking for TODOs..."
if git diff | grep -E "^\+.*TODO|^\+.*STUB|^\+.*FIXME"; then
    echo "❌ No TODOs/STUBs allowed in commits!"
    exit 1
fi

# 5. No unwrap/expect in production code
echo "Checking for unwrap()..."
if git diff src/ | grep -E "^\+.*\.unwrap\(\)|^\+.*\.expect\("; then
    echo "❌ No unwrap/expect in production code!"
    exit 1
fi

echo "✅ All checks passed!"
```

## One-Feature-Per-Version Philosophy

### The Rule

Each version delivers **EXACTLY ONE complete feature**, fully working end-to-end.

### What "End-to-End" Means

- ✅ Feature works in production binary
- ✅ All tests passing (not just new feature)
- ✅ Documentation updated (README, docs)
- ✅ Shell scripts updated (.sh files work)
- ✅ Agent prompts updated (if workflow changes)
- ✅ Integration tested (not just unit tests)
- ✅ Zero TODOs, zero stubs, zero placeholders
- ✅ Pushed to origin/main

### Forbidden

- ❌ Partial features ("LPA foundation but no integration")
- ❌ Stubs for "future work"
- ❌ Breaking existing features
- ❌ Documentation saying "will be implemented"
- ❌ Commits with TODO comments

### Version Naming

```
v0.9.4, v0.9.5, v0.9.6, v0.9.7, v0.9.8, v0.9.9 → v1.0.0 → v1.0.1, v1.0.2...

❌ NO v0.10.0 - we go v0.9.9 then v1.0.0
```

---

# Part IX: Agent Design Patterns for 2026

## The New Context Engineering Paradigm

> **"Context engineering is the delicate art and science of filling the context window with just the right information for the next step."**
> — Andrej Karpathy, 2025

By 2026, Meta acquired Manus for over $2B and Claude Code reached a $1B run rate. This section synthesizes the agent design patterns that emerged from production deployments at scale.

```mermaid
flowchart TB
    subgraph AGENT_2026["2026 Agent Architecture"]
        GIVE[Give Agents A Computer<br/>Filesystem + Shell]
        MULTI[Multi-Layer Action Space<br/>Few atomic tools]
        PROG[Progressive Disclosure<br/>Reveal as needed]
        OFF[Offload Context<br/>To filesystem]
        CACHE[Cache Context<br/>Prompt caching]
        ISOLATE[Isolate Context<br/>Sub-agents]
        EVOLVE[Evolve Context<br/>Learn from trajectories]
    end

    GIVE --> MULTI
    MULTI --> PROG
    PROG --> OFF
    OFF --> CACHE
    CACHE --> ISOLATE
    ISOLATE --> EVOLVE
```

## The Fundamental Insight

**Context must be treated as a finite resource with diminishing marginal returns.**

Like humans with limited working memory, LLMs have an "attention budget." Every new token depletes it. @METR_Evals reports that agent task length doubles every 7 months—but models get worse as context grows.

## Pattern 1: Give Agents A Computer

### The Shift to OS-Level Primitives

@barry_zyj and @ErikSchluntz defined agents as systems where LLMs direct their own actions. By 2026, successful agents all share one trait: **they live on your computer**.

```mermaid
flowchart TB
    subgraph COMPUTER["The Agent Computer"]
        FS[Filesystem<br/>Persistent Context]
        SHELL[Shell<br/>Built-in utilities]
        CLI[CLIs<br/>Domain tools]
        CODE[Code<br/>Agent-written scripts]
    end

    LLM[LLM Agent] -->|Tools| SHELL
    SHELL --> FS
    SHELL --> CLI
    SHELL --> CODE
```

### Why This Works

| Capability | Traditional | Agent Computer |
|------------|-------------|----------------|
| **Persistence** | Lost in context | Saved to filesystem |
| **Composition** | Pre-built tools | Chain via shell |
| **Domain** | Generic | Use existing CLIs |
| **Flexibility** | Fixed interface | Write new code |

**@rauchg**: *"The primary lesson from the actually successful agents so far is the return to Unix fundamentals: file systems, shells, processes & CLIs. Don't fight the models, embrace the abstractions they're tuned for. Bash is all you need."*

## Pattern 2: Multi-Layer Action Space

### The Tool Explosion Problem

As MCP usage scales, tool definitions overload the context window. The GitHub MCP server has ~26K tokens of tool definitions alone.

### The Solution: Push Actions Down

```mermaid
flowchart TB
    subgraph LAYERS["Action Space Hierarchy"]
        L1[Layer 1: Tool Calling<br/>~10 atomic tools<br/>Constrained decoding]
        L2[Layer 2: Shell<br/>bash tool<br/>CodeAct pattern]
        L3[Layer 3: Computer<br/>Utilities, CLIs<br/>Agent-written code]
    end

    L1 -->|Few calls| L2
    L2 -->|Many actions| L3

    L1 -->|Direct| TARGET[Complete tasks]
    L2 -->|Chained| TARGET
    L3 -->|Composed| TARGET
```

### Production Numbers

| Agent | Tool Count | Design |
|-------|-----------|--------|
| Claude Code | ~12 | Curated atomic tools |
| Manus | <20 | Hierarchy with bash |
| Amp Code | Few | Curated action space |

**The CodeAct Pattern**: Agents write and execute code to chain actions without processing intermediate results.

```rust
// Instead of: tool_call("read_file") → tool_call("write_file") → tool_call("run_test")
// Agent writes:
bash << 'EOF'
  cp src/main.rs src/main.rs.bak
  sed -i 's/old/new/g' src/main.rs
  cargo test --quiet
EOF
// Single action, agent doesn't process intermediate outputs
```

## Pattern 3: Progressive Disclosure

### The Problem

Loading all tool definitions upfront consumes valuable context budget and can confuse models with overlapping functionality.

### The Solution

```mermaid
flowchart TB
    subgraph DISCLOSURE["Progressive Disclosure"]
        INDEX[Index of available actions]
        NEED[Agent needs specific action]
        LOOKUP[Lookup on demand]
        DETAIL[Load full details]
    end

    INDEX -->|Triggered by task| NEED
    NEED --> LOOKUP
    LOOKUP --> DETAIL
```

### Implementations

**Claude Code Skills**: Folders containing SKILL.md files. YAML frontmatter loaded into instructions; full MD read only if needed.

**Cursor Agent**: MCP tool descriptions synced to folder. Agent gets short list, reads full description only when needed.

**Manus**: List of utilities in instructions. Agent uses `--help` flags to learn details on-demand.

```yaml
# skill/SKILL.md
---
name: validate_identifier_four_words_only
summary: Validates function names follow 4WNC
triggers:
  - "naming convention"
  - "function name"
  - "4WNC"
---

# Full documentation only loaded when triggered...
```

## Pattern 4: Offload Context

### Context Rot is Real

@trychroma reported on context rot, @dbreunig outlined failure modes. As context grows, models:

- Lose track of early information
- Become inconsistent with previous decisions
- Repeat earlier points

### The Filesystem as Context Extension

```mermaid
flowchart TB
    subgraph OFFLOAD["Context Offloading"]
        AGENT[Agent Context Window]
        THRESHOLD{Token budget<br/>nearing limit?}
        FILESYSTEM[Filesystem<br/>Persistent context]
        SUMMARIZE[Summarize old context]
        PLAN[Plan file<br/>Reinforce objectives]
    end

    AGENT -->|Fills| THRESHOLD
    THRESHOLD -->|Yes| SUMMARIZE
    SUMMARIZE --> FILESYSTEM
    THRESHOLD -->|No| AGENT
    PLAN -->|Periodically read back| AGENT
```

### Implementations

**Manus**: Old tool results written to files. Summarization applied once diminishing returns hit.

**Cursor Agent**: Tool results and agent trajectories offloaded to filesystem. Read back if needed.

**Steering Pattern**: Plan file written and periodically read to reinforce objectives and verify work.

```bash
# Plan file pattern
cat > /tmp/plan.md << 'EOF'
# Task: Implement feature X

## Steps
1. [ ] Write test for filter_implementation_entities_only
2. [ ] Implement function
3. [ ] Verify performance < 500μs
4. [ ] Update documentation

## Progress
- Step 1: Complete
- Step 2: In progress
EOF

# Agent periodically reads to stay on track
```

## Pattern 5: Cache Context

### The Most Important Metric

**@Manus called out "cache hit rate" as the most important metric for production agents.**

Without prompt caching, agents become cost-prohibitive. A higher capacity model with caching can be cheaper than a lower cost model without it.

```mermaid
flowchart TB
    subgraph CACHING["Prompt Caching"]
        PREFIX[Static prefix<br/>System prompt, tools]
        CACHE[Cached prompt<br/>$/token much cheaper]
        DYNAMIC[Dynamic messages<br/>Only new tokens billed]
    end

    PREFIX -->|Cached| CACHE
    CACHE -->|Base| DYNAMIC
```

**@trq212**: *"Coding agents (Claude Code) would be cost-prohibitive without caching."*

### Cache Optimization

| Strategy | Impact |
|----------|--------|
| Static system prompt | High |
| Tool definitions | High |
| Agent instructions | High |
| Recent conversation | Low (changes frequently) |

## Pattern 6: Isolate Context

### The Sub-Agent Pattern

```mermaid
flowchart TB
    subgraph ISOLATION["Context Isolation"]
        MAIN[Main Agent<br/>Full context of task]
        SUB1[Sub-Agent 1<br/>Check issue A]
        SUB2[Sub-Agent 2<br/>Check issue B]
        SUB3[Sub-Agent N<br/>Check issue N]
    end

    MAIN -->|Delegate in parallel| SUB1
    MAIN -->|Delegate in parallel| SUB2
    MAIN -->|Delegate in parallel| SUB3

    SUB1 -->|Result| MAIN
    SUB2 -->|Result| MAIN
    SUB3 -->|Result| MAIN
```

### Use Cases

**Parallel Code Review**: Claude Code uses sub-agents to independently check different possible issues.

**Map-Reduce**: Lint rules, migrations, any embarrassingly parallel task.

### The Ralph Loop

@GeoffreyHuntley coined the "Ralph Wiggum"—a loop running agents repeatedly until a plan is satisfied.

```mermaid
flowchart TB
    subgraph RALPH["Ralph Loop Pattern"]
        INIT[Initializer Agent<br/>Creates plan file]
        PLAN[Plan File<br/>Git-backed tracking]
        SUB1[Sub-Agent 1<br/>Tackles task A]
        SUBN[Sub-Agent N<br/>Tackles task Z]
        VERIFY[Stop Hook / Verify Agent<br/>Check work]
    end

    INIT --> PLAN
    PLAN --> SUB1
    PLAN --> SUBN
    SUB1 -->|Commit progress| PLAN
    SUBN -->|Commit progress| PLAN
    PLAN -->|All tasks done?| VERIFY
    VERIFY -->|Fail| PLAN
```

**@bcherny (Claude Code)**: *"For very long-running tasks, I will either (a) prompt Claude to verify its work with a background agent when it's done, (b) use an agent Stop hook to do that more deterministically, or (c) use the ralph-wiggum plugin."*

## Pattern 7: Evolve Context

### Continual Learning in Token Space

@Letta_AI talks about continual learning in token space—updating agent context (not model weights) with learnings over time.

```mermaid
flowchart TB
    subgraph EVOLUTION["Context Evolution"]
        TRAJECTORIES[Agent Trajectories<br/>Past sessions]
        SCORE[Score Outcomes<br/>Success/Failure]
        REFLECT[Reflect on Failures<br/>Extract patterns]
        UPDATE[Update Context<br/>Memory, skills, prompts]
    end

    TRAJECTORIES --> SCORE
    SCORE --> REFLECT
    REFLECT --> UPDATE
    UPDATE -->|Improved performance| TRAJECTORIES
```

### Claude Code Diary Pattern

@RLanceMartin: *"Reflect over session logs, distill preferences/feedback from actual use to update memory."*

```bash
# Pattern: distill sessions into diary entries
~/.claude/diary/
├── 2025-12-01.md  # What worked, what didn't
├── 2025-12-02.md  # Patterns discovered
└── 2025-12-03.md  # Skill distillations

# Update CLAUDE.md based on reflection
cat > CLAUDE.md << 'EOF'
# Project Context

## Learned Patterns
- Always use 4WNC for function names
- TDD cycle prevents 90% of bugs
- Progressive disclosure for large contexts
EOF
```

### GEPA Pattern

**G**enerate trajectories, **E**valuate, **P**ropose variants, **A**dopt:

1. Collect agent trajectories
2. Score them (success/failure)
3. Reflect on failures
4. Propose prompt variants
5. Test and adopt winners

## Future Directions

### Learned Context Management

The Bitter Lesson predicts compute/model scaling will overtake hand-crafted prompting.

**@RLanceMartin on Recursive Language Models (RLM)**: *"LLMs can learn to perform their own context management. Much of the prompting or scaffolding packed into agent harnesses might get absorbed by models."*

```mermaid
flowchart TB
    subgraph RLM["Recursive Language Models"]
        MODEL[Base LLM]
        CONTEXT[Own context]
        SEARCH[Search over trace]
        MANAGE[Learn to manage]
    end

    MODEL -->|Reflects on| CONTEXT
    MODEL -->|Searches| SEARCH
    MODEL -->|Trains| MANAGE
    MANAGE -->|Replaces hand-crafted| MODEL
```

### Sleep-Time Compute

@Letta_AI shows agents can "think offline" about their own context. Imagine agents that:

- Reflect over past sessions automatically
- Update their own memories
- Consolidate experiences
- Prepare for future tasks

### Multi-Agent Coordination

@cognition noted: *"Today's agents struggle with shared context: each action contains implicit decisions and parallel agents risk making conflicting decisions."*

**Gas Town** by @Steve_Yegge: Multi-agent orchestrator with git-backed work tracking, "Mayor" agent with full workspace context, merge queue for coordination.

```mermaid
flowchart TB
    subgraph GAS_TOWN["Gas Town Pattern"]
        ORCH[Multi-Agent Orchestrator]
        MAYOR[Mayor Agent<br/>Full context]
        QUEUE[Merge Queue]
        AGENTS[Dozens of Claude Code<br/>Role-specialized]
    end

    ORCH --> MAYOR
    MAYOR --> QUEUE
    QUEUE --> AGENTS
    AGENTS -->|PR to queue| QUEUE
```

### Abstractions for Long-Running Agents

Needed infrastructure:

- **Observability**: What agents are doing
- **Hooks**: Human review points
- **Graceful degradation**: Failures handled
- **Debugging interfaces**: Common patterns
- **HITL monitoring**: Human-in-the-loop standards

@hwchase17: *"Swarm of agents working concurrently will need new abstractions."*

---

## Summary: 2026 Agent Patterns

| Pattern | Core Idea | Production Example |
|---------|-----------|-------------------|
| **Give Agents A Computer** | Filesystem + shell primitives | Claude Code, Manus |
| **Multi-Layer Action Space** | Few atomic tools, push to bash | Manus (<20 tools) |
| **Progressive Disclosure** | Reveal details on-demand | Claude Skills, Cursor |
| **Offload Context** | Filesystem extends context | Manus, Cursor |
| **Cache Context** | Prompt caching = viability | All production agents |
| **Isolate Context** | Sub-agents for parallel work | Claude Code review |
| **Evolve Context** | Learn from trajectories | Claude diary, GEPA |

### The Agent Design Thesis for 2026

> **Effective agent design largely boils down to context management.**

The winning combination:
1. **Unix primitives** (filesystem, shell) over custom tooling
2. **Progressive disclosure** over loading everything upfront
3. **Context offloading** to filesystem over in-memory compaction
4. **Sub-agents** with isolation over monolithic context
5. **Continual learning** from trajectories over static prompts
6. **Prompt caching** as the most important cost metric

---

# Part X: Quick Reference

## The 7 Principles Summary

| Principle | Core Idea | Key Practice |
|-----------|-----------|--------------|
| **(a) Search Tools** | LLMs search, don't program | Bias with keywords |
| **(b) Iteration** | First output ≠ final | Multi-round refinement |
| **(c) Summary Docs** | Context window forgets | Memory checkpoints |
| **(d) Self-Reasoning** | Find own fallacies | Rubber duck prompts |
| **(e) Anti-Patterns** | Learn from failures | Reference bug files |
| **(f) TDD Mindset** | Tests = executable specs | STUB→RED→GREEN→REFACTOR |
| **(g) Cumulative** | Document everything | Prompt libraries |

## The 2026 Agent Patterns Summary

| Pattern | Core Idea | Key Practice |
|---------|-----------|--------------|
| **Give Agents A Computer** | Filesystem + shell primitives | Bash is all you need |
| **Multi-Layer Action Space** | Few atomic tools, push to bash | ~12 tools, CodeAct pattern |
| **Progressive Disclosure** | Reveal details on-demand | Load SKILL.md only when needed |
| **Offload Context** | Filesystem extends context | Plan files, trajectory offload |
| **Cache Context** | Prompt caching = viability | Most important production metric |
| **Isolate Context** | Sub-agents for parallel work | Ralph Loop for long-running tasks |
| **Evolve Context** | Learn from trajectories | Diary pattern, GEPA |

## 4WNC Quick Reference

```
verb_constraint_target_qualifier()

Examples:
✅ filter_implementation_entities_only()
✅ render_box_with_title_unicode()
✅ save_visualization_output_to_file()
✅ create_database_connection_pool_async()

❌ filter() (too short)
❌ detect_cycles_in_dependency_graph() (too long)
```

## TDD Cycle Quick Reference

```
STUB → RED → GREEN → REFACTOR

STUB: Write test first
RED: Verify it fails
GREEN: Minimal implementation
REFACTOR: Improve without breaking
```

## Token Budget Quick Reference

| Task | Budget | Selection |
|------|--------|-----------|
| Quick explain | 2K | Direct callers/callees |
| Code review | 4K | + depth 1-2 |
| Bug hunt | 6K | + error paths |
| Refactor | 8K | + blast radius |

---

# Conclusion

AI-native engineering is not about using AI tools—it's about **writing software that AI can reason about accurately**.

The Four-Word Naming Convention, TDD-First methodology, executable specifications, and smart context optimization are not independent tricks. They form a coherent philosophy for development in the age of LLMs.

**The results speak for themselves:**
- 67% faster development
- 90% fewer production bugs
- 95% context accuracy
- 70% faster onboarding

## From 2025 to 2026: The Evolution to Agent Design

By 2026, the focus shifted from individual developer patterns to **agent architecture patterns**. The meta-patterns that made developers productive with LLMs became the foundation for building autonomous agents:

| 2025 Pattern | 2026 Agent Evolution |
|--------------|---------------------|
| 4WNC naming | Progressive disclosure for skills |
| TDD-First | Ralph Loop for long-running tasks |
| Summary docs | Context offloading to filesystem |
| Parseltongue graphs | Sub-agent isolation |
| Iterative refinement | GEPA: continual learning |

**The 2026 Thesis**: Effective agent design largely boils down to context management.

> *"Context engineering is the delicate art and science of filling the context window with just the right information for the next step."* — Andrej Karpathy, 2025

This is the future of software engineering. The question is not whether to adopt AI-native methods, but how quickly.

---

> *"The best code for LLMs is the best code for humans—clear, structured, and unambiguous. The difference is that with LLMs, clarity becomes correctness."*
>
> — Based on 400+ hours of AI-assisted development

---

## References

### Source Documents (2025)

1. **LLM-Prompting-MetaPatterns-Analysis.md** - 45+ meta-patterns with examples
2. **S06-design101-tdd-architecture-principles.md** - TDD-First comprehensive guide
3. **S81-Parseltongue-mini-cookbook.md** - Query-based code analysis patterns
4. **S02-Mermaid-Explanations.md** - Mermaid diagram standards
5. **S00-claude.md** - One-Feature-Per-Version philosophy
6. **Idiom97-SystemDesignPatterns.md** - 2,500+ system design patterns
7. **Idiom96-polyglot-basic-patterns.md** - 400+ patterns per stack

### Source Documents (2026 Agent Patterns)

1. **@karpathy** - Context engineering definition, 2025
2. **@barry_zyj, @ErikSchluntz** - Agents as LLM-directed systems
3. **@bcherny (Claude Code)** - ~12 tools, sub-agents, stop hooks
4. **@peakji (Manus)** - <20 tools, action space hierarchy
5. **@beyang (Amp Code)** - Curated action space
6. **@rauchg** - Unix fundamentals: "Bash is all you need"
7. **@trychroma** - Context rot research
8. **@dbreunig** - Context failure modes
9. **@GeoffreyHuntley** - Ralph Wiggum loop pattern
10. **@Letta_AI** - Continual learning, sleep-time compute
11. **@RLanceMartin** - Claude Code diary pattern
12. **@Steve_Yegge** - Gas Town multi-agent coordination
13. **@hwchase17** - LangChain agent abstractions
14. **@METR_Evals** - Agent task length doubling every 7 months

### Proof of Work

- **Parseltongue Dependency Graph Generator** https://github.com/that-in-rust/parseltongue-dependency-graph-generator/
  - 50MB single binary
  - 12 languages supported
  - 15 HTTP endpoints
  - < 500μs query latency

---

*Document Version: 2.0*
*Last Updated: 2026-01-19*
*Total Patterns Documented: 45+ meta-patterns + 2,500+ system design patterns + 7 agent design patterns*
