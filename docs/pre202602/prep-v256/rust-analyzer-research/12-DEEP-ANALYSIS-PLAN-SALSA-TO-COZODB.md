# ULTRATHINK: Deep Analysis Plan - Salsa → CozoDB Migration for Rust-Analyzer

**Document Type**: Research & Analysis Plan
**Scope**: Comprehensive technical validation of replacing Salsa with CozoDB
**Timeline**: 17 weeks (4 months)
**Outcome**: GO/NO-GO decision with evidence-based recommendation

---

## Executive Summary

This plan outlines a rigorous, research-heavy approach to determine whether rust-analyzer should migrate from Salsa (incremental computation framework) to CozoDB (embedded graph database). The analysis revealed several critical insights:

### Key Findings from Initial Research:

1. **CozoDB Architecture** (from web research):
   - Embedded, transactional, Datalog-based graph database
   - Storage: RocksDB, SQLite, in-memory, WASM
   - Performance: 100K QPS transactional, 250K+ QPS read-only
   - **Critical limitation**: ONLY supports snapshot isolation (no weaker consistency levels)

2. **Fundamental Mismatch Identified**:
   - **Salsa optimizes for**: Microsecond latency, many tiny changes (user typing)
   - **CozoDB optimizes for**: Millisecond latency, complex graph queries on stable data
   - This tension suggests **full replacement may not be viable**

3. **Most Likely Outcome**: **Hybrid architecture**
   - Keep Salsa for hot path (type checking, completion)
   - Add CozoDB for analysis (impact analysis, refactoring, visualization)
   - Timeline: 12-18 months to hybrid, likely never to full migration

### Critical Questions to Answer:

1. Can CozoDB match Salsa's performance for incremental queries? (Unlikely)
2. Can Datalog express Salsa's incremental computation semantics? (To be proven)
3. Is the performance tradeoff worth the new capabilities? (Depends on benchmarks)
4. Should we pursue hybrid architecture instead of full replacement? (Probable YES)

---

## PHASE 1: FOUNDATIONAL RESEARCH (Weeks 1-3)

### 1.1 CozoDB Deep Dive

#### Completed Web Research Findings:

**Architecture & Capabilities:**
- **CozoDB** is an embedded, transactional, relational-graph-vector database using Datalog for queries
- **Source**: https://www.cozodb.org/
- **GitHub**: https://github.com/cozodb/cozo
- **Storage backends**: RocksDB, SQLite, in-memory, WASM
- **Rust API**: `cozo` crate (https://docs.rs/cozo)

**Performance Characteristics** (from official benchmarks):
- **Transactional throughput**: ~100K QPS for mixed read-write
- **Read-only throughput**: 250K+ QPS
- **PageRank performance**:
  - 10K vertices, 120K edges: 50ms
  - 100K vertices, 1.7M edges: 1 second
  - 1.6M vertices, 32M edges: 30 seconds
- **Two-hop traversal**: <1ms for 1.6M vertices/31M edges
- **Source**: https://docs.cozodb.org/en/latest/releases/v0.3.html
- **Wiki benchmark**: https://github.com/cozodb/cozo/wiki/Cozo-is-an-extremely-performant-graph-database-that-runs-everywhere

**Memory Characteristics**:
- RAII-based memory management
- Memory usage scales with rows touched (not total DB size)
- Efficient for queries on large databases

**Transactional Guarantees** (CRITICAL FINDING):
- **ONLY snapshot isolation supported** (from docs: https://docs.cozodb.org/en/latest/stored.html)
- No weaker consistency levels available
- MVCC for concurrency control
- All queries run in transactions with automatic retry on conflict
- Durability guaranteed on transaction completion

**Key Implication**: CozoDB's mandatory snapshot isolation may not match Salsa's more flexible consistency model.

#### Remaining Research Tasks:

**A. Datalog Language Mastery**
- [ ] Read complete CozoDB Datalog tutorial (https://docs.cozodb.org/en/latest/tutorial.html)
- [ ] Compare Datalog vs Cypher for graph queries
- [ ] Study recursive Datalog queries with aggregations
- [ ] Understand fixed-point computation in Datalog (relates to incremental computation)

**B. Rust API Deep Dive**
- [ ] Review `cozo` crate API documentation (https://docs.rs/cozo/latest/cozo/)
- [ ] Test embedded mode vs client-server performance
- [ ] Benchmark in-memory backend vs RocksDB for 14,852 entities
- [ ] Study thread safety model (concurrent queries?)

**C. Scalability Validation**
- [ ] Create POC with synthetic data: 100K nodes, 1M edges
- [ ] Measure query latency, memory usage, transaction throughput
- [ ] Test with rust-analyzer's actual scale:
  - 14,852 code entities
  - ~92,931 edges (current Parseltongue analysis)
  - Estimate: 10x-100x more if including type variables, trait bounds, query dependencies

**D. Schema Design Exploration**
- [ ] How to model Salsa's "revisions" in CozoDB?
- [ ] Can CozoDB's stored relations efficiently track query memoization?
- [ ] How to represent "dirty" vs "clean" query results?

### 1.2 Salsa Internals Deep Dive

#### Completed Web Research Findings:

**The Red-Green Algorithm**:
- Tracks **revisions** (incremented on each input change)
- **Source**: https://salsa-rs.github.io/salsa/reference/algorithm.html
- Stores memoized values + dependencies for each tracked function
- On re-invoke: checks if dependencies changed in current revision
- **Excellent explanation**: https://medium.com/@eliah.lakhin/salsa-algorithm-explained-c5d6df1dd291

**Query System** (from official docs):
- Queries as functions K → V
- **Source**: https://salsa-rs.github.io/salsa/overview.html
- **Input queries**: Base facts supplied by client
- **Derived queries**: Pure functions computing from inputs
- Dependency graph is implicit (built during execution)

**Memoization Strategy**:
- Results stored with dependency metadata
- **Shallow checking**: Only checks changed dependencies, stops at first unchanged
- **Critical optimization**: "If A depends on B and C, and B hasn't changed, C won't be checked"

#### Remaining Research Tasks:

**A. Salsa Source Code Analysis**
- [ ] Clone `salsa-rs/salsa` repository (https://github.com/salsa-rs/salsa)
- [ ] Read `salsa/src/runtime/` - dependency tracking internals
- [ ] Study `DependencyGraph` data structure
- [ ] Analyze revision management (`Revision` type, increment logic)
- [ ] Understand `DatabaseKeyIndex` - how are query results indexed?

**B. Rust-Analyzer's Salsa Usage**
- [ ] Search rust-analyzer for `#[salsa::query_group]` macros
- [ ] Map all query groups to understand dependency layers
- [ ] Find examples of:
  - LRU queries vs tracked queries
  - Durability settings
  - Cycle handling
- [ ] Estimate: How many distinct query types? (200-500?)

**C. Memory Model Analysis**
- [ ] How does Salsa store memoized data? (HashMap? BTreeMap?)
- [ ] Memory overhead per query result?
- [ ] Garbage collection mechanism? (ref: https://github.com/rust-lang/rust-analyzer/issues/73)
- [ ] Can we extract Salsa's runtime dependency graph for visualization?

**D. Snapshot Mechanism**
- [ ] How do Salsa snapshots work? (Arc cloning?)
- [ ] Cost of creating snapshot? (Expected: O(1))
- [ ] Can multiple snapshots coexist?
- [ ] How does snapshot isolation differ from CozoDB's?

### 1.3 Incremental Computation Theory

#### Academic Papers to Read:

**Primary Papers**:
1. **Adapton** (PLDI '14): "Composable, demand-driven incremental computation"
   - ACM DL: https://dl.acm.org/doi/abs/10.1145/2666356.2594324
   - Full PDF: https://www.cs.tufts.edu/~jfoster/papers/cs-tr-5027.pdf
   - Key concept: Demanded computation graph (DCG)
   - **Question**: How does DCG compare to Salsa's dependency tracking?

2. **Self-Adjusting Computation** (Umut Acar thesis, 2005)
   - PDF: https://www.cs.cmu.edu/~rwh/students/acar.pdf
   - Foundational work on dynamic dependence graphs
   - Understand "change propagation" algorithms
   - **Compare** to Salsa's red-green algorithm

3. **miniAdapton** (arXiv 2016): Minimal IC in Scheme
   - arXiv: https://arxiv.org/abs/1609.05337
   - Understand core IC abstractions
   - **Question**: Could Datalog implement Adapton-style IC?

4. **Recent survey**: "Incremental Computation: What Is the Essence?" (PEPM 2024)
   - ACM DL: https://dl.acm.org/doi/10.1145/3635800.3637447

#### Research Questions:

- [ ] **Q1**: Do any IC frameworks use graph databases as backing store?
  - Hypothesis: No, most use in-memory data structures
  - Validate: Search "graph database incremental computation" in Google Scholar

- [ ] **Q2**: Can Datalog express incremental computation semantics?
  - Hypothesis: Yes, via recursive rules + stratification
  - Test: Write Datalog rules for "if A changed, recompute B"

- [ ] **Q3**: Performance overhead of persistent graphs vs in-memory?
  - Hypothesis: 10x-100x slower for writes, faster for complex queries
  - Validate: Benchmark CozoDB vs Salsa on identical workload

### 1.4 Alternative Architectures Survey

#### Alternatives to Evaluate:

**1. Other Graph Databases**

**Memgraph**:
- In-memory, Cypher, BSL license
- **Performance**: 41x lower latency than Neo4j (https://memgraph.com/blog/neo4j-vs-memgraph)
- Cons: Not embedded (requires separate process), no Datalog
- Rust client: `rsmgclient`

**Neo4j Embedded**:
- Only for JVM languages
- Pros: Mature, excellent tooling
- Cons: Can't embed in Rust binary

**IndraDB**:
- Native Rust graph database (https://github.com/indradb/indradb)
- Pros: Pure Rust, embeddable
- Cons: Less mature, no Datalog, smaller ecosystem

**2. Incremental Computation Libraries**
- Salsa 2.0 (in development)
- Adapton.rs (if exists?)
- Custom solution: DAG library + memoization

**3. Hybrid Approaches**
- Keep Salsa for queries, add CozoDB for visualization
- Use CozoDB as read-only mirror (dual-write)
- Use CozoDB only for "global" queries, Salsa for fast path

#### Decision Matrix:

| Solution | Embedded | Performance | Rust API | Query Language | Incremental? | Verdict |
|----------|----------|-------------|----------|----------------|--------------|---------|
| Salsa | ✓ | Excellent | Native | Rust macros | ✓ | Baseline |
| CozoDB | ✓ | Good | Native | Datalog | ? | Research |
| Memgraph | ✗ | Excellent | rsmgclient | Cypher | ✗ | Hybrid? |
| IndraDB | ✓ | Unknown | Native | Rust API | ✗ | Fallback |
| Custom | ✓ | Unknown | Native | Rust | ✓ | High risk |

---

## PHASE 2: TECHNICAL VALIDATION (Weeks 4-7)

### 2.1 Proof-of-Concept: Salsa Query in CozoDB

**Goal**: Can we replicate Salsa's query semantics in Datalog?

#### POC 1: Simple Dependency Chain

**Salsa version** (pseudo-code):
```rust
#[salsa::input]
fn source_text(db: &dyn Db, file: FileId) -> String;

#[salsa::tracked]
fn parse(db: &dyn Db, file: FileId) -> SyntaxTree {
    let text = source_text(db, file);
    Parser::parse(&text)
}

#[salsa::tracked]
fn type_check(db: &dyn Db, file: FileId) -> TypeInfo {
    let tree = parse(db, file);
    TypeChecker::check(tree)
}
```

**CozoDB version** (Datalog):
```datalog
# Stored relations (inputs)
:create source_text {file: String, revision: Int, text: String}

# Derived relations (queries)
?[file, parse_result, revision] :=
    source_text[file, revision, text],
    parse_result = parse_fn(text)

:replace parse_cache {
    file: String,
    =>
    parse_result: String,
    revision: Int
}

?[file, type_result, revision] :=
    parse_cache[file, parse_result, revision],
    type_result = type_check_fn(parse_result)

:replace type_cache {
    file: String,
    =>
    type_result: String,
    revision: Int
}
```

**Questions to Answer**:
- [ ] How to invoke Rust functions from Datalog? (CozoDB's UDF mechanism)
- [ ] How to implement revision tracking? (manual vs automatic)
- [ ] How to invalidate caches on `source_text` change? (triggers? manual DELETE?)

#### POC 2: Incremental Invalidation

**Scenario**: Change `source_text` for "main.rs", only recompute affected queries.

**Salsa behavior**:
1. User calls `set_source_text(db, "main.rs", new_text)`
2. Salsa increments revision
3. Next call to `type_check(db, "main.rs")` detects change, recomputes
4. Calls to `type_check(db, "other.rs")` use cached value

**CozoDB implementation** (pseudo-Datalog):
```datalog
# On source_text change
?[file, new_rev] :=
    source_text[file, old_rev, old_text],
    new_rev = old_rev + 1,
    # Delete old cached values
    *parse_cache {file @ file, *}
    *type_cache {file @ file, *}
```

**Benchmarks**:
- [ ] Measure latency: Salsa vs CozoDB for 100 sequential changes
- [ ] Expected: Salsa 10x-100x faster (in-memory vs database writes)
- [ ] Acceptable: CozoDB within 10x (if other benefits justify)

### 2.2 Type System Representation

**Goal**: Can CozoDB model Rust's complex type system?

#### Example: Generic Types

**Type**: `Vec<T>` where `T: Clone`

**CozoDB Schema** (proposed):
```datalog
# Nodes
:create types {
    type_id: String,
    kind: String,  # "Adt", "TypeParam", "Ref", etc.
}

# Edges
:create type_has_param {
    parent_type: String,
    position: Int,
    param_type: String,
}

:create type_has_bound {
    type_param: String,
    trait_id: String,
}

# Query: Find all types with Clone bound
?[type_id] :=
    types[type_id, "TypeParam"],
    type_has_bound[type_id, trait_id],
    traits[trait_id, "Clone"]
```

**Test Cases**:
- [ ] Model: `fn foo<T: Clone>(x: T) -> Vec<T>`
- [ ] Model: `struct Bar<'a, T> where T: 'a { x: &'a T }`
- [ ] Model: `trait Foo { type Item; }`
- [ ] Verify: Can we query these correctly?

#### Challenges:
- [ ] Lifetimes: How to represent `'a`, `'static`, variance?
- [ ] Associated types: `<T as Iterator>::Item`
- [ ] Higher-ranked trait bounds: `for<'a> F: Fn(&'a str)`

### 2.3 Performance Benchmarks

**Goal**: Find the breaking point - at what scale does CozoDB become unusable?

#### Benchmark 1: Parallel Queries

**Scenario**: 10 threads simultaneously query different expressions

**Salsa version**:
```rust
let snapshot = db.snapshot();  // O(1), Arc clone
rayon::scope(|s| {
    for expr in exprs {
        s.spawn(|_| {
            let ty = snapshot.type_of_expr(expr);
        });
    }
});
```

**CozoDB version**:
```rust
let db = DbInstance::new(/*...*/)?;
rayon::scope(|s| {
    for expr in exprs {
        s.spawn(|_| {
            let result = db.run_script("?[type] := type_of[expr, type]", /*...*/);
        });
    }
});
```

**Metrics**:
- [ ] Throughput (queries/second)
- [ ] Latency (p50, p95, p99)
- [ ] Memory usage
- [ ] Contention (lock wait time)

**Expected**: Salsa wins due to lock-free reads

#### Benchmark 2: Incremental Updates

**Scenario**: User types in editor - 100 consecutive file changes

**Metrics**:
- [ ] Time to invalidate cached queries
- [ ] Time to recompute dirty queries
- [ ] Peak memory usage

**Expected**: Salsa wins due to in-memory operation

#### Benchmark 3: Complex Graph Queries

**Scenario**: "Find all types that depend on struct X" (impact analysis)

**CozoDB Datalog**:
```datalog
# Find all types transitively depending on X
?[dependent_type] :=
    types["X", _],
    type_dependencies[type, "X"]

?[dependent_type] :=
    *?[intermediate_type],
    type_dependencies[dependent_type, intermediate_type]
```

**Metrics**:
- [ ] Query execution time
- [ ] Memory usage

**Expected**: CozoDB wins (native graph traversal)

---

## PHASE 3: DEEP TECHNICAL CHALLENGES (Weeks 8-11)

### 3.1 The Fundamental Mismatch

**Critical Observation**:

#### Salsa's Model:
- Revisions: Monotonically increasing counter
- Memoization: Query results cached with revision
- Invalidation: Lazy (on next query invocation)
- Snapshots: Arc-based, zero-cost, immutable views

#### CozoDB's Model:
- Transactions: Snapshot isolation, MVCC
- Persistence: All data written to storage
- Invalidation: Requires explicit DELETE + recompute
- Snapshots: Transactions provide isolation, but NOT zero-cost

**Tension Identified**:
- Salsa optimizes for *many small changes* (user typing)
- CozoDB optimizes for *complex queries* on *relatively stable data*

**Hypothesis**: CozoDB is NOT a drop-in replacement for Salsa. Better as complementary tool.

### 3.2 Memory Overhead Analysis

**Estimate Salsa Memory Usage**:

Assumptions:
- 14,852 code entities
- 100 query results per entity on average
- 1.4M memoized query results total

Per query result:
- Query key: 8-16 bytes
- Value: ~32 bytes (average)
- Dependency list: ~5 deps × 8 bytes = 40 bytes
- Revision: 8 bytes

**Estimate: 100-200 bytes per query result**
→ 1.4M × 150 bytes = **210 MB**

**Estimate CozoDB Memory Usage**:

If storing all query results as graph nodes:
- 14,852 code entities (nodes)
- 1.4M query results (nodes)
- 1.4M × 5 = 7M dependency edges

CozoDB overhead (RocksDB):
- Node: ~64 bytes
- Edge: ~32 bytes

Estimate: 1.414M × 64 + 7M × 32 = **91 MB + 224 MB = 315 MB**

**Conclusion**: CozoDB ~1.5x memory overhead (acceptable)

**But** (accounting for RocksDB overhead):
- Write amplification
- Index overhead
- Transaction log

**Realistic estimate**: **2x-5x memory overhead**

### 3.3 Query Invalidation Strategy

**The Core Problem**: How to implement Salsa's red-green algorithm in CozoDB?

#### Option 1: Revision-Based Invalidation

```datalog
:create query_results {
    query_key: String,
    =>
    result_value: String,
    computed_at_revision: Int,
}

:create input_changes {
    input_key: String,
    changed_at_revision: Int,
}

# Check if query is stale
?[query_key, is_stale] :=
    query_results[query_key, _, computed_rev],
    query_dependencies[query_key, dep_key],
    input_changes[dep_key, changed_rev],
    is_stale = (changed_rev > computed_rev)
```

**Pros**: Mirrors Salsa's approach
**Cons**: Requires maintaining dependency graph manually

#### Option 2: Trigger-Based Invalidation

```datalog
:create trigger invalidate_on_input_change {
    on: UPDATE(inputs),
    action: {
        ?[query_key] :=
            inputs[changed_key, _],
            query_dependencies[query_key, changed_key]

        :delete query_results { query_key @ query_key }
    }
}
```

**Pros**: Automatic
**Cons**: CozoDB may not support triggers (verify)

#### Option 3: Dual-Write Pattern

```rust
fn set_input(db: &mut Db, key: K, value: V) {
    // Update Salsa
    db.salsa.set(key, value);

    // Replicate to CozoDB
    db.cozo.run_script("
        :put inputs { key: $key, value: $value }
    ", params! { "key" => key, "value" => value })?;
}
```

**Pros**: Best of both worlds
**Cons**: Consistency challenges, double memory

**Recommendation**: Option 3 (hybrid) for Phase 1

---

## PHASE 4: MIGRATION STRATEGY DESIGN (Weeks 12-15)

### 4.1 Four-Phase Migration - Risk Analysis

#### Phase 1: Dual-Write (Months 1-3)

**Proposed**:
- Keep Salsa as source of truth
- Write all updates to CozoDB in parallel
- Validate consistency

**Risks**:
1. **Consistency**: CozoDB write fails but Salsa succeeds
   - Mitigation: Transactional wrapper
   - Problem: Salsa doesn't support transactions

2. **Performance**: 2x write latency
   - Acceptable? Only if writes <5% of operations

3. **Schema evolution**: CozoDB schema changes
   - Mitigation: Version schema, support migration

**Validation Test**:
- [ ] Implement dual-write for ONE query group (`syntax::parse`)
- [ ] Measure overhead: <10% acceptable, >20% unacceptable
- [ ] Run for 1 week on real workload

#### Phase 2: Read Migration (Months 4-9) - DANGER ZONE

**Proposed**:
- Implement read queries in Datalog
- A/B test: Salsa vs CozoDB

**Risks**:
1. **Semantic mismatch**: Datalog ≠ Salsa semantics
   - Mitigation: Property-based testing

2. **Performance regression**: Users notice slower IDE
   - Mitigation: Only migrate queries with <50ms latency budget

3. **Debugging hell**: Bugs in TWO systems
   - Mitigation: Comprehensive logging, easy rollback

**Validation Test**:
- [ ] Migrate `type_of_expr` to CozoDB
- [ ] Fuzz test: 10,000 random programs, compare results
- [ ] Benchmark: Must be within 2x of Salsa

**Abort Condition**: If >10% queries fail validation, STOP

#### Phase 3: Write Migration (Months 10-15) - POINT OF NO RETURN

**Risks**:
1. **Data loss**: Bug in CozoDB trigger corrupts state
2. **Performance collapse**: Without Salsa optimizations, IDE unusable
3. **Community backlash**: Breaking changes

**Go/No-Go Decision**:
- If Phase 2 shows CozoDB can't match Salsa performance, STOP
- Cost: 9 months wasted, but Salsa still works

#### Phase 4: Optimization (Months 16+)

**Note**: Only reach this if Phase 3 succeeds

### 4.2 Rollback Strategy

| Phase | Rollback Difficulty | Data Loss Risk | Code Churn |
|-------|---------------------|----------------|------------|
| 1 | Easy | None | Low |
| 2 | Medium | None | Medium |
| 3 | **HARD** | **HIGH** | **HIGH** |
| 4 | Impossible | Critical | Catastrophic |

**Recommendation**: Set strict criteria for Phase 2. Do NOT proceed to Phase 3 unless:
1. CozoDB performance within 2x of Salsa for 95% of queries
2. Zero semantic mismatches in fuzz testing
3. Community approval (RFC accepted)

### 4.3 Alternative: Hybrid Architecture

**Proposal**: Salsa for hot path, CozoDB for analysis

```
┌─────────────────────────────────────┐
│         Rust-Analyzer               │
├─────────────────────────────────────┤
│  Salsa (Incremental Queries)        │ ← Fast path
├─────────────────────────────────────┤
│  CozoDB (Global Analysis)           │ ← Slow path
└─────────────────────────────────────┘
```

**Use Cases for CozoDB**:
1. Impact analysis: "What breaks if I change this struct?"
2. Cross-crate refactoring
3. Dependency visualization
4. Architectural analysis
5. Performance profiling

**Benefits**:
- **Low risk**: Salsa remains
- **High value**: NEW features impossible with Salsa
- **Incremental**: Can add features one-by-one

**Drawbacks**:
- Complexity: Maintain two systems
- Consistency: May diverge
- Memory: 1.5x-2x overhead

**Recommendation**: Start with hybrid, only go full CozoDB if PROVEN beneficial

---

## PHASE 5: DECISION FRAMEWORK (Weeks 16-17)

### 5.1 GO/NO-GO Rubric

#### Criteria 1: Performance

| Metric | Salsa | CozoDB | Threshold | Pass? |
|--------|-------|--------|-----------|-------|
| Type checking (10K LOC) | 50ms | ? | <100ms | TBD |
| Completion latency | 10ms | ? | <20ms | TBD |
| Incremental recompile | 5ms | ? | <15ms | TBD |
| Impact analysis | N/A | ? | <500ms | TBD |

**Rule**: CozoDB within 2x for ALL critical paths, or NO-GO

#### Criteria 2: Complexity

**Rule**: If CozoDB adds >30% code complexity, NO-GO

#### Criteria 3: Capabilities

**Rule**: CozoDB must enable ≥3 high-value NEW features, or NO-GO

#### Criteria 4: Risk

| Risk | Probability | Impact | Severity |
|------|-------------|--------|----------|
| Performance regression | 60% | High | **CRITICAL** |
| Semantic bugs | 40% | High | **CRITICAL** |
| Maintenance burden | 80% | Medium | HIGH |

**Rule**: If any CRITICAL risk >50%, need mitigation

### 5.2 Final Recommendation Framework

```
IF (Performance criteria MET) AND
   (Complexity acceptable) AND
   (≥3 new high-value features) AND
   (Critical risks mitigated)
THEN
    IF (Full migration benefits > costs)
    THEN Phase 3 (Full Migration)
    ELSE Hybrid Architecture
ELSE
    STOP - Keep Salsa, CozoDB for offline analysis only
```

---

## DELIVERABLES

### Deliverable 1: Research Report (Week 3)

**Contents**:
1. CozoDB capabilities summary (10 pages)
2. Salsa internals documentation (15 pages)
3. Incremental computation theory (8 pages)

**Sources**:
- CozoDB docs: https://docs.cozodb.org/
- Salsa docs: https://salsa-rs.github.io/salsa/
- Adapton paper: https://www.cs.tufts.edu/~jfoster/papers/cs-tr-5027.pdf

### Deliverable 2: Proof-of-Concept (Week 7)

**Repository**: `rust-analyzer-cozodb-poc`

**Contents**:
1. `salsa-baseline/`: Minimal Salsa example
2. `cozodb-equivalent/`: Same logic in Datalog
3. `benchmarks/`: Performance comparison
4. `type-system-model/`: Graph representation
5. `docs/`: Findings

**Success**: CozoDB within 10x of Salsa (POC stage)

### Deliverable 3: Migration Risk Analysis (Week 11)

**Document**: `MIGRATION_RISKS.md` (20 pages)

### Deliverable 4: Technical Spec (Week 15)

**Document**: `COZODB_INTEGRATION_SPEC.md` (40 pages)

### Deliverable 5: Recommendation (Week 17)

**Document**: `FINAL_RECOMMENDATION.md` (5 pages)

**TL;DR**: One paragraph - GO or NO-GO?

---

## CRITICAL INSIGHTS

### Why This Might Be a BAD Idea

**Honest assessment**:

1. **Performance impedance mismatch**: Salsa optimized for microsecond latency. CozoDB optimized for millisecond analytics.

2. **Snapshot isolation is NOT free**: CozoDB transactions have overhead. Salsa's Arc snapshots are zero-cost.

3. **Wrong tool for incremental computation**: Datalog great for recursion, but IC requires fine-grained change propagation.

4. **You can't query what you can't compute**: If CozoDB too slow for type inference, fancy queries don't help.

### Why This Might Be a GOOD Idea

1. **Unlocks new capabilities**: Impact analysis, visualization, refactoring game-changing for large projects.

2. **Debugging**: Query "why did this recompute?" helps RA developers.

3. **Hybrid is viable**: Keep Salsa for hot path, add CozoDB for analysis.

4. **Research value**: Understanding trade-offs advances state of the art.

### The Probable Outcome

**Prediction**:
- Phases 1-2 succeed (dual-write, POC)
- Phase 3 attempted but performance unacceptable
- **Final recommendation: Hybrid architecture**
- CozoDB for:
  - IDE analysis features
  - Developer tooling
  - Documentation generation
- Salsa retained for:
  - Incremental type checking
  - Real-time IDE features
  - Query memoization

**Timeline**: 12-18 months to hybrid, never to full migration

**Value**: Medium-high (new features justify complexity)

---

## SOURCES & REFERENCES

### CozoDB Resources:
- Official Documentation: https://docs.cozodb.org/
- GitHub Repository: https://github.com/cozodb/cozo
- Rust API Docs: https://docs.rs/cozo
- Stored Relations: https://docs.cozodb.org/en/latest/stored.html
- Tutorial: https://docs.cozodb.org/en/latest/tutorial.html
- Performance Benchmarks: https://docs.cozodb.org/en/latest/releases/v0.3.html
- Wiki: https://github.com/cozodb/cozo/wiki

### Salsa Resources:
- Official Docs: https://salsa-rs.github.io/salsa/
- Red-Green Algorithm: https://salsa-rs.github.io/salsa/reference/algorithm.html
- Overview: https://salsa-rs.github.io/salsa/overview.html
- GitHub: https://github.com/salsa-rs/salsa
- Algorithm Explained: https://medium.com/@eliah.lakhin/salsa-algorithm-explained-c5d6df1dd291

### Academic Papers:
- Adapton (PLDI '14): https://www.cs.tufts.edu/~jfoster/papers/cs-tr-5027.pdf
- Self-Adjusting Computation (Acar): https://www.cs.cmu.edu/~rwh/students/acar.pdf
- miniAdapton: https://arxiv.org/abs/1609.05337

### Other Graph Databases:
- Memgraph vs Neo4j: https://memgraph.com/blog/neo4j-vs-memgraph
- IndraDB: https://github.com/indradb/indradb

### Rust-Analyzer:
- Architecture Guide: https://rust-analyzer.github.io/book/contributing/architecture.html
- Issue #73 (GC): https://github.com/rust-lang/rust-analyzer/issues/73
- Trait Resolution: https://rustc-dev-guide.rust-lang.org/traits/resolution.html

---

**END OF DEEP ANALYSIS PLAN**
