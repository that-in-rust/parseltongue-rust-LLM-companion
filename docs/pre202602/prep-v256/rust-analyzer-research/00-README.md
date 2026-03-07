# Rust-Analyzer Architecture Analysis

**Analysis Date:** 2026-01-29
**Repository:** https://github.com/rust-lang/rust-analyzer
**Commit:** Latest (shallow clone)
**Analysis Tool:** Parseltongue v1.4.0

## Analysis Summary

This directory contains a comprehensive architectural analysis of the rust-analyzer codebase, generated using the Parseltongue dependency graph analyzer. The analysis covers architecture, control flow, and data flow patterns across the entire codebase.

### Key Findings

- **Total Code Entities:** 14,852
- **Dependency Edges:** 92,931
- **Circular Dependencies:** 0 (excellent architectural health)
- **Languages:** Rust (primary), Python (tooling)
- **Test Coverage:** 15,445 test entities

### Architectural Health Indicators

✅ **Zero Circular Dependencies** - Clean module boundaries
✅ **Layered Architecture** - Clear separation of concerns
✅ **Incremental Computation** - Salsa-based query system
✅ **Resilient Error Handling** - Partial analysis on errors
✅ **Extensive Testing** - 1:1 ratio of code to tests

## Document Organization

### 1. [Architecture Overview](./01-ARCHITECTURE-OVERVIEW.md)
**Purpose:** High-level system architecture and design patterns

**Contents:**
- Executive summary of the codebase
- Core state management (GlobalState)
- Layered architecture breakdown
- Module organization
- Key design patterns
- Complexity hotspots
- Performance characteristics

**Key Insights:**
- Event-loop driven LSP server
- Salsa incremental computation framework
- Concurrent execution with thread pools
- Multi-phase workspace loading

### 2. [Control Flow](./02-CONTROL-FLOW.md)
**Purpose:** Detailed control flow analysis through the system

**Contents:**
- Main event loop structure
- Event type taxonomy (7 types)
- Event handling pipeline
- LSP request/notification routing
- State machine transitions
- Coalescing patterns
- Performance optimizations

**Key Insights:**
- Sophisticated event coalescing for efficiency
- Quiescence detection for optimization triggers
- Operation queues for sequenced async work
- Deferred task execution for database consistency

### 3. [Data Flow](./03-DATA-FLOW.md)
**Purpose:** Data transformations and state management

**Contents:**
- VFS (Virtual File System) flow
- Salsa query layers
- LSP message transformation
- Diagnostic generation
- Workspace loading phases
- Macro expansion
- Snapshot mechanism
- Memory management

**Key Insights:**
- Four-layer query system: Syntax → Names → Types → HIR
- Snapshot isolation for non-blocking queries
- Incremental invalidation via Salsa
- Two-phase workspace loading (metadata, then build data)

### 4. [Folder Structure Diagrams](./04-FOLDER-STRUCTURE-DIAGRAMS.md)
**Purpose:** Visual guide to codebase organization with Mermaid diagrams

**Contents:**
- Overall codebase structure
- Main crates dependency layers
- Complete crates organization (mindmap)
- File-by-file structure for key crates
- Data flow between folders
- Lib directory structure
- Project model flow
- Syntax parsing flow
- Summary table of key directories

**Key Insights:**
- 36 crates organized in 5 layers
- Clear dependency hierarchy
- rust-analyzer (LSP) → IDE → HIR → Syntax → Foundation
- Support libraries in lib/ directory

### 5. [Control Flow Diagrams](./05-CONTROL-FLOW-DIAGRAMS.md)
**Purpose:** Visual guide to how code flows through the system

**Contents:**
- Main event loop control flow (comprehensive flowchart)
- LSP request handling flow
- Completion request sequence diagram
- Workspace loading state machine
- VFS change processing flowchart
- Flycheck (cargo check) flow
- Macro expansion control flow
- Goto definition flow
- Salsa query execution flow

**Key Insights:**
- 8 event types processed in main loop
- Event coalescing pattern throughout
- Sophisticated state machine for workspace loading
- Background task execution via snapshots

### 6. [Data Flow Diagrams](./06-DATA-FLOW-DIAGRAMS.md)
**Purpose:** Visual guide to data transformations

**Contents:**
- Complete data pipeline (11 layers)
- Text to syntax tree transformation
- File change to re-analysis sequence
- Workspace to CrateGraph transformation
- Name resolution data flow
- Type inference data flow
- Completion data generation
- Diagnostic generation flow
- Token to semantic token mapping
- Hover information assembly
- Inlay hints data flow
- Memory layout of key structures
- Data transformation layers summary

**Key Insights:**
- 11 transformation layers from raw text to LSP messages
- Immutable data structures with Arc for sharing
- Incremental invalidation at each layer
- Clear data ownership patterns

### 7. [API Usage Guide](./07-API-USAGE-GUIDE.md)
**Purpose:** How to use rust-analyzer's APIs

**Contents:**
- LSP API for editor integration
- IDE API (Analysis entry point)
- HIR API (Semantics entry point)
- Syntax API (SyntaxNode operations)
- API structure diagrams
- Example code for each API level
- Common usage patterns
- Testing utilities
- Configuration options
- Performance tips

**Key Insights:**
- 4 API levels: LSP, IDE, HIR, Syntax
- Each level appropriate for different use cases
- Snapshot pattern for background work
- Extensive example code provided

### 8. [Per-File Detailed Diagrams](./08-PER-FILE-DETAILED-DIAGRAMS.md)
**Purpose:** Detailed analysis of individual files using Parseltongue data

**Contents:**
- main_loop.rs (30 entities) - Event loop structure
- global_state.rs (44 entities) - State management
- lsp/ext.rs (152 entities) - LSP extensions
- HIR layer files (hir/lib.rs, hir-def/nameres, hir-ty/infer)
- IDE layer files (ide/lib.rs, ide-completion/lib.rs)
- Syntax layer files (syntax/lib.rs, parser/lib.rs)
- Foundation files (base-db/lib.rs, vfs/lib.rs)
- Project model files (project-model/workspace.rs)

**Key Insights:**
- All diagrams generated from Parseltongue queries
- Entity counts per file documented
- Data flow and control flow per major file
- Class diagrams for key structures

### 9. [ELI5 Guide](./09-ELI5-GUIDE.md)
**Purpose:** Explain rust-analyzer like you're 5 years old

**Contents:**
- What IS rust-analyzer (simple explanation)
- Restaurant kitchen analogy
- The 5 main parts (Waiter, Prep Cook, Chef, Plater, Memory)
- How a request works (step by step)
- The magic trick (incremental updates)
- Key concepts simplified (Syntax Tree, Salsa, LSP)
- What you can DO with it
- Folder structure simplified
- Common questions answered
- Real-world analogy

**Key Insights:**
- Accessible to complete beginners
- Uses analogies throughout (restaurant, word processor)
- Visual diagrams for each concept
- Answers "why" questions simply

### 10. [Super HQ Idiomatic Patterns](./10-SUPER-HQ-IDIOMATIC-PATTERNS.md)
**Purpose:** Extract exceptional Rust patterns (90+ quality score) from rust-analyzer

**Contents:**
- **Part 1: Type-Level Mastery** (8 patterns)
  - Phantom type with invariant `fn() -> T` (98/100)
  - Sealed traits with const values (95/100)
  - Lifetime transmutation with GC assertion (92/100)
  - Type-state pattern with DropBomb (94/100)
  - CoerceUnsized optimization with fast-paths (96/100)
  - Canonical type folding (97/100)
  - TextSize newtype defensive API (91/100)
  - Interner pattern for O(1) equality (93/100)

- **Part 2: Metaprogramming Genius** (10 patterns)
  - Adaptive token tree storage (95/100)
  - Backtracking macro matcher (94/100)
  - Hygiene-preserving macro context (96/100)
  - CoercePointee auto-derive (93/100)
  - Metavariable expression evaluation (92/100)
  - Eager macro expansion (91/100)
  - Proc macro with sentinel IDs (90/100)
  - Metavariable expression parser (91/100)
  - Macro rule heuristic selection (92/100)
  - Op enum for unified operations (90/100)

- **Part 3: Concurrency & Memory Wizardry** (10 patterns)
  - Arc-based GlobalStateSnapshot (95/100)
  - QoS-aware thread pool (92/100)
  - Salsa query system (94/100)
  - Path interner deduplication (91/100)
  - Snapshot-based undo system (93/100)
  - TLS global cache (90/100)
  - CowArc optimization (88/100)
  - VFS with lock-free interner (91/100)
  - ManuallyDrop for compile performance (89/100)
  - Lock-free proc macro files (87/100)

- **Cross-Cutting Meta-Patterns**
  - "Make illegal states unrepresentable"
  - "Parse, don't validate"
  - "Zero-cost with profiled fast-paths"
  - "Unsafe with compile-time invariant checks"
  - "Interning for O(1) equality"
  - "Arc-based snapshot isolation"
  - "Salsa queries for incremental computation"

- **Anti-Patterns to Avoid**
  - Covariant PhantomData for arena indices
  - Overusing Arc
  - Premature interning
  - Unsafe without documented invariants
  - Type-state for simple validation

- **Summary Reference Table** (all 28 patterns)
- **Appendix: Parseltongue Queries** (reproducibility)

**Key Insights:**
- **EXPANDED:** Now includes 43 patterns (28 original + 15 new in Part 4)
- All patterns extracted via Parseltongue (no grep/glob)
- Expert-level documentation with full code examples (40-100 lines each)
- Each pattern includes: Score, Location, Code, Analysis, Diagram, When to Use, Pitfalls, When NOT to Use
- Reveals meta-thinking of genius-level Rust developers
- Focus on uncommon, sophisticated patterns (not basic idioms)
- 7 cross-cutting themes identified across all patterns
- **Updated:** ~2,270 lines, 75KB of comprehensive expert documentation
- **Part 4 additions:** Transactional snapshots, undo logs, type folding delegates, caching strategies

## Analysis Methodology

### Tool: Parseltongue v1.4.0

Parseltongue is a dependency graph generator that:
1. **Ingests** entire codebases into a CozoDB database
2. **Indexes** all code entities (functions, structs, modules, etc.)
3. **Tracks** dependency relationships (caller/callee edges)
4. **Exposes** HTTP API for querying architecture

### Analysis Process

```
1. Clone rust-analyzer (shallow)
   ↓
2. Download Parseltongue binary (macOS ARM64)
   ↓
3. Run indexing: pt01-folder-to-cozodb-streamer
   - Processed 1,446 files
   - Created 14,852 entities
   - Built 92,931 dependency edges
   - Duration: ~21 seconds
   ↓
4. Start query server: pt08-http-code-query-server
   - 15 REST endpoints available
   - Port 7779 (auto-selected)
   ↓
5. Query architecture via HTTP API
   - /codebase-statistics-overview-summary
   - /complexity-hotspots-ranking-view
   - /circular-dependency-detection-scan
   - /code-entity-detail-view
   - /blast-radius-impact-analysis
   ↓
6. Document findings
   - Architecture overview
   - Control flow patterns
   - Data flow transformations
```

### Query Examples

**Get entity details:**
```bash
curl http://localhost:7779/code-entity-detail-view?key=rust:struct:GlobalState:__crates_rust-analyzer_src_global_state_rs:84-198
```

**Find complexity hotspots:**
```bash
curl http://localhost:7779/complexity-hotspots-ranking-view?top=20
```

**Check for cycles:**
```bash
curl http://localhost:7779/circular-dependency-detection-scan
```

**Blast radius analysis:**
```bash
curl http://localhost:7779/blast-radius-impact-analysis?entity=rust:fn:main_loop:...&hops=2
```

## Architecture Highlights

### Core Design Principles

1. **Incremental Computation**
   - Salsa query framework for memoization
   - Automatic dependency tracking
   - Smart invalidation on changes

2. **Resilient Parsing**
   - Recovers from syntax errors
   - Provides IDE features on invalid code
   - Partial ASTs enable analysis

3. **Layered Abstraction**
   - Clear boundaries: LSP ← IDE ← HIR ← Syntax
   - Each layer has well-defined APIs
   - Dependency flow is unidirectional

4. **Concurrent Execution**
   - Main event loop on primary thread
   - Background task pools for expensive work
   - Snapshot mechanism for lock-free reads

### Key Components

**GlobalState (crates/rust-analyzer/src/global_state.rs)**
- Central orchestrator
- 114 lines of fields
- Manages all subsystems

**Main Loop (crates/rust-analyzer/src/main_loop.rs)**
- Event-driven architecture
- 7 event types processed
- Sophisticated coalescing

**VFS (crates/vfs/)**
- In-memory file system
- Change tracking
- Incremental updates

**HIR (crates/hir-*/)**
- High-level intermediate representation
- Name resolution
- Type inference
- Semantic analysis

**Salsa (database layer)**
- Incremental computation engine
- Query memoization
- Dependency tracking

## Complexity Hotspots

**Top 10 Most-Coupled Entities:**

1. `Some` - 2,476 callers (Option construction)
2. `new` - 2,002 callers (Constructor pattern)
3. `check_assist` - 1,720 callers (Test infrastructure)
4. `check` - 1,647 callers (Validation)
5. `map` - 1,647 callers (Functional transforms)
6. `into` - 1,458 callers (Type conversions)
7. `clone` - 1,455 callers (Data cloning)
8. `syntax` - 1,345 callers (Syntax tree access)
9. `iter` - 1,086 callers (Iterator creation)
10. `kind` - 959 callers (Type discrimination)

**Analysis:** High coupling is primarily in:
- Standard Rust patterns (Option, Result, iterators)
- Test infrastructure (check_assist, check)
- Syntax tree navigation (syntax, kind)

## Performance Characteristics

### Optimization Strategies

1. **Incremental Analysis**
   - Only re-analyzes changed portions
   - Salsa caches unchanged results
   - Short-circuit propagation

2. **Lazy Evaluation**
   - Queries computed on-demand
   - Unused code not analyzed
   - Defers expensive operations

3. **Event Coalescing**
   - Batches similar events
   - Reduces notification overhead
   - Single loop turn for multiple events

4. **Snapshot Isolation**
   - Background tasks use snapshots
   - No blocking of main thread
   - Stale reads acceptable for IDE

5. **Garbage Collection**
   - Triggered when idle
   - Revision-based
   - Removes unused query results

### Performance Monitoring

- Loop duration tracked
- Warns on >100ms iterations (when quiescent)
- Helps identify bottlenecks
- Tracing spans for profiling

## Architectural Strengths

1. ✅ **Zero Circular Dependencies** - Clean module structure
2. ✅ **Incremental by Design** - Minimal redundant computation
3. ✅ **Resilient Error Handling** - Graceful degradation
4. ✅ **Modular Architecture** - Clear separation of concerns
5. ✅ **Extensive Testing** - High test coverage
6. ✅ **Performance Conscious** - Multiple optimization layers
7. ✅ **Concurrent Execution** - Efficient use of threads

## Notable Patterns

### 1. Event Coalescing
**Problem:** Too many small events overwhelm system
**Solution:** Drain channel with `try_recv()` in tight loop
**Benefit:** Reduces overhead, batches updates

### 2. Deferred Task Queue
**Problem:** Database-dependent work in sync handlers
**Solution:** Queue tasks, execute after `process_changes()`
**Benefit:** Maintains consistency, avoids blocking

### 3. Operation Queues (OpQueue)
**Problem:** Async operations need sequencing
**Solution:** Queue pattern with should_start/op_completed
**Benefit:** Ensures correct ordering (metadata → build data → proc macros)

### 4. Snapshot Mechanism
**Problem:** Background queries block main thread
**Solution:** Arc-based snapshots of database
**Benefit:** Lock-free reads, stale data acceptable

### 5. Two-Phase Workspace Loading
**Problem:** Build scripts slow, but metadata fast
**Solution:** Load metadata first, build data later
**Benefit:** Quick initial analysis, full analysis when ready

## Technology Stack

### Core Libraries

- **Salsa:** Incremental computation framework
- **Rowan:** Red-green tree for syntax
- **Crossbeam:** Concurrency primitives
- **Rayon:** Data parallelism
- **LSP-Server:** LSP protocol handling

### Project Structure

- **crates/** - Rust workspace crates (40+)
- **lib/** - Supporting libraries
- **editors/code/** - VS Code extension (TypeScript)
- **xtask/** - Build automation

## Future Exploration

### Recommended Deep Dives

1. **Salsa Integration**
   - How queries are defined
   - Invalidation strategies
   - Memoization mechanisms

2. **Macro Expansion**
   - Declarative macro expansion (mbe)
   - Proc macro IPC architecture
   - Hygiene handling

3. **Type Inference**
   - Unification algorithm
   - Trait solving
   - Next-gen trait solver

4. **IDE Features**
   - Completion scoring
   - Import suggestion
   - Code actions

## Using This Analysis

### For Contributors
- Understand overall architecture before diving into code
- Identify where to add new features
- Understand performance implications

### For Researchers
- Study incremental computation in practice
- Analyze LSP server architecture
- Examine Rust compiler internals usage

### For Tool Builders
- Learn from mature Rust project structure
- Study event-driven architecture
- Understand IDE feature implementation

## Reproduction

To reproduce this analysis:

```bash
# 1. Clone rust-analyzer
git clone --depth 1 https://github.com/rust-lang/rust-analyzer.git
cd rust-analyzer

# 2. Download Parseltongue
curl -L -o parseltongue https://github.com/that-in-rust/parseltongue-dependency-graph-generator/releases/download/v1.4.0/parseltongue
chmod +x parseltongue

# 3. Index codebase
./parseltongue pt01-folder-to-cozodb-streamer

# 4. Start query server
./parseltongue pt08-http-code-query-server --db "rocksdb:parseltongue20260129211500/analysis.db"

# 5. Query architecture
curl http://localhost:7779/codebase-statistics-overview-summary
curl http://localhost:7779/complexity-hotspots-ranking-view?top=20
curl http://localhost:7779/circular-dependency-detection-scan
```

## References

- **Rust-Analyzer:** https://github.com/rust-lang/rust-analyzer
- **Parseltongue:** https://github.com/that-in-rust/parseltongue-dependency-graph-generator
- **Salsa:** https://github.com/salsa-rs/salsa
- **LSP Specification:** https://microsoft.github.io/language-server-protocol/

## License

This analysis is documentation of the rust-analyzer project architecture.
Rust-analyzer itself is licensed under MIT OR Apache-2.0.

## Acknowledgments

- Rust-analyzer team for excellent architectural design
- Parseltongue for enabling comprehensive codebase analysis
- Salsa for incremental computation framework

---

### 11. [Dependency Graph Pyramid Analysis](./11-DEPENDENCY-GRAPH-PYRAMID-ANALYSIS.md) ⭐ **NEW**
**Purpose:** Complete dependency analysis using Minto Pyramid Principle for graph database migration

**Motivation:** Understand rust-analyzer's architecture from 30,000 ft to micro-level to enable rewriting with graph databases (Neo4j/Memgraph)

**Data Sources:**
- Parseltongue: 14,852 entities, 92,931 edges
- 39 internal crates with full dependency mapping  
- Cargo.toml workspace dependency analysis

**Document Structure:**

#### **Level 1: 30,000 Foot View** - Overall Architecture
- Five-layer onion architecture (Foundation → Syntax → HIR → IDE → Application)
- 39 crates organized by dependency intensity
- Foundation crates analysis: `edition` (35 reverse deps), `stdx` (22), `syntax` (16)
- Architectural statistics by layer
- Dependency intensity heatmap
- Critical dependency paths (longest chain: 10 hops)

#### **Level 2: 10,000 Foot View** - Crate-Level Dependencies
- Complete 39×39 crate dependency matrix
- Forward dependencies: Who does each crate depend on?
- Reverse dependencies: Who depends on each crate?
- Key dependency clusters:
  - **HIR Cluster** (4 crates): Strict layering with `hir` → `hir-ty` → `hir-def` → `hir-expand`
  - **IDE Cluster** (6 crates): Hub-and-spoke with `ide-db` as central hub
- Most critical dependencies (breaking these affects 15+ crates)
- Island crates (unused/test-only): `proc-macro-test`, `syntax-fuzz`

#### **Level 3: 1,000 Foot View** - Module-Level Dependencies
- Internal module structure of key crates:
  - **hir-ty**: 3-tier (Query layer, Core algorithms, Utilities)
  - **syntax**: Parser facade + Typed AST layer
- Module dependency graphs with Mermaid diagrams
- Cross-module dependency sequences (e.g., ide-completion → hir → hir-ty → infer)

#### **Level 4: 100 Foot View** - Entity-Level Micro-Diagrams
- Entity type distribution (54% methods, 20% impls, 12% functions)
- Example entity relationship graphs:
  - Type inference entities (InferenceContext, TypeVariableTable, trait Unify)
  - Salsa query dependency DAG
- Micro-diagrams showing function call chains

#### **Graph Database Mapping Strategy** 🎯

**Why Graph DB?**
- Current problems: Salsa is opaque, debugging is hard, no global impact view
- Benefits: Native relationship traversal, impact analysis in O(edges), visualization, flexible schema

**Proposed Schema:**
```cypher
// Node types
(:Crate), (:Module), (:File)
(:Struct), (:Enum), (:Trait), (:Function), (:Method), (:Impl)
(:Type), (:TypeVariable), (:TraitBound)
(:Query), (:QueryResult)  // Salsa replacement

// Relationship types
-[:DEPENDS_ON]->  // Crate dependencies
-[:CONTAINS]->    // Containment
-[:CALLS]->       // Function calls
-[:USES_TYPE]->   // Type usage
-[:IMPLEMENTS]->  // Trait implementations
-[:INVALIDATED_BY]->  // Query invalidation
```

**Example Cypher Queries:**
- **Impact analysis:** "What breaks if I change struct X?"
- **Longest chain:** Find deepest dependency path
- **Circular deps:** Detect violations (should return empty)
- **PageRank:** Find most central types

**Migration Strategy:**
1. **Phase 1 (3mo):** Dual-write (Salsa + Graph DB in parallel)
2. **Phase 2 (6mo):** Read migration (Cypher queries for non-critical paths)
3. **Phase 3 (6mo):** Write migration (Graph triggers replace Salsa)
4. **Phase 4 (ongoing):** Optimization (Indexes, partitioning)

**Graph DB Comparison:**
| Database | License | Performance | Rust Client | Recommendation |
|----------|---------|-------------|-------------|----------------|
| Neo4j | AGPL | Good | neo4rs | Production-ready |
| **Memgraph** | BSL | **Excellent (in-memory)** | rsmgclient | **✅ Best for IDE** |
| RedisGraph | Redis Source | Excellent | redis-rs | Prototyping |
| Neptune | Proprietary | Good | gremlin-rs | AWS only |

**Proof of Concept:**
- Cypher CREATE statements for all 39 crates
- Sample queries showing transitive dependencies
- Graph trigger examples for incremental updates

**Key Takeaways:**
1. Rust-analyzer's DAG structure is **perfect for graph databases**
2. No circular dependencies at crate level
3. Query dependencies form a DAG (Salsa invariant preserved)
4. Graph DB enables new capabilities: impact analysis, visualization, performance profiling
5. Migration path exists with clear phases

**Recommended Tech Stack:** Memgraph (in-memory, Cypher, fast for IDE responsiveness)

**Artifacts Generated:**
- `internal-crate-dependency-graph.json` - Full 39-crate dependency graph
- `rust-analyzer-dependency-graph.json` - 14,852 entities from Parseltongue
- `crate-analysis.json` - Entity counts by crate


---

### 12. [Deep Analysis Plan: Salsa → CozoDB Migration](./12-DEEP-ANALYSIS-PLAN-SALSA-TO-COZODB.md) 📋 **RESEARCH PLAN**
**Purpose**: Comprehensive 17-week research plan for evaluating graph database migration

**Scope**: Replace Salsa (incremental computation) with CozoDB (graph database)?

**Timeline**: 4 months (17 weeks) of rigorous research before any code

**Key Research Areas**:
1. **CozoDB Deep Dive** (Weeks 1-3)
   - Datalog language mastery
   - Rust API integration
   - Scalability validation (14,852 entities → 1M+ test)
   - Schema design exploration

2. **Salsa Internals** (Weeks 1-3)
   - Red-green algorithm deep dive
   - Source code analysis (`salsa-rs/salsa`)
   - Rust-analyzer's Salsa usage patterns
   - Memory model analysis

3. **Incremental Computation Theory** (Weeks 1-3)
   - Academic papers: Adapton, Self-Adjusting Computation
   - Can Datalog express IC semantics?
   - Performance models (persistent vs in-memory)

4. **Technical Validation** (Weeks 4-7)
   - POC: Salsa query → CozoDB Datalog
   - Type system representation in graph
   - Performance benchmarks (parallel queries, incremental updates)

5. **Deep Challenges** (Weeks 8-11)
   - Fundamental mismatch analysis
   - Memory overhead calculation
   - Query invalidation strategy

6. **Migration Strategy** (Weeks 12-15)
   - 4-phase plan risk analysis
   - Rollback strategy
   - Hybrid architecture design

7. **Decision Framework** (Weeks 16-17)
   - GO/NO-GO rubric
   - Performance criteria
   - Final recommendation

**Critical Findings from Initial Research**:
- CozoDB: 100K QPS transactional, 250K+ read-only (from [docs](https://docs.cozodb.org/))
- **Limitation**: ONLY snapshot isolation (no weaker consistency)
- Salsa's red-green algorithm: Revision-based, lazy, shallow checking
- Performance gap: Salsa microseconds, CozoDB milliseconds

**Most Likely Outcome**: **Hybrid architecture**
- Keep Salsa for hot path (type checking, completion)
- Add CozoDB for analysis (impact analysis, visualization)
- Timeline: 12-18 months to hybrid, never to full migration

**Deliverables**:
1. Research Report (Week 3): CozoDB capabilities, Salsa internals, IC theory
2. Proof-of-Concept (Week 7): Salsa vs CozoDB benchmarks
3. Migration Risk Analysis (Week 11): Phase-by-phase risks
4. Technical Spec (Week 15): Architecture, schema, API design
5. Final Recommendation (Week 17): GO/NO-GO with evidence

---

### 13. [Rubber Duck Debugging: Salsa ↔ CozoDB](./13-RUBBER-DUCK-DEBUGGING-SALSA-COZODB.md) 🦆 **CRITICAL ANALYSIS**
**Purpose**: Think out loud, validate assumptions, challenge conclusions

**Method**: Rubber duck debugging + Web research validation

**Key Questions & Findings**:

#### Q1: Why does rust-analyzer use Salsa instead of a graph database?
**Answer**: 
- Salsa implements red-green incremental algorithm from rustc
- Zero-cost abstractions: O(1) snapshots via Arc cloning
- Lazy evaluation: Only recompute when queries are invoked
- **Validated**: [Salsa official docs](https://salsa-rs.github.io/salsa/overview.html)

#### Q2: Fundamental difference between Salsa's query DAG vs persistent graph DB?
**Answer**:
| Aspect | Salsa | CozoDB | Winner for IDE? |
|--------|-------|--------|-----------------|
| Latency | Microseconds | Milliseconds | **Salsa** 100x faster |
| Snapshot cost | O(1) Arc clone | Transaction overhead | **Salsa** zero-cost |
| Complex queries | Hard | Easy (Datalog) | **CozoDB** native |
| Memory | ~200MB | ~400-1000MB | **Salsa** 2-5x smaller |

**Conclusion**: Salsa wins for real-time, CozoDB wins for analysis

#### Q3: Can CozoDB replace Salsa's snapshot mechanism?
**Critical Finding** (from [CozoDB docs](https://docs.cozodb.org/en/latest/stored.html)):
> "CozoDB supports ONLY snapshot isolation (SI) as its consistency level."

**Performance Impact**:
- Salsa snapshot: 0.001ms (Arc clone)
- CozoDB transaction: ~1ms (begin → execute → commit)
- **Slowdown: 1000x** for snapshot creation

**Verdict**: ❌ Deal-breaker for hot path queries

#### Q4: How would query invalidation work in CozoDB?
**Salsa's Approach**:
- Lazy: Only recompute when invoked
- Shallow checking: Stop at first unchanged dependency
- Automatic: Framework handles revision tracking

**CozoDB's Challenge**:
- Datalog is eager (semi-naive evaluation)
- Would need manual revision tracking
- **Not incremental by default**

**Conclusion**: Must implement red-green ON TOP OF Datalog (extra complexity!)

#### Q5: Memory overhead - 14,852 entities?
**Calculated Estimates**:
- Salsa: **297 MB** (1.485M query results × 200 bytes)
- CozoDB (RocksDB): **412 MB** (base data + overhead)
- Overhead: **1.4x** (not as bad as feared!)
- **Realistic with real usage: 2x-3x**

#### Q6: Can Datalog express Salsa's incremental semantics?
**Problems Identified**:
1. **Lazy vs Eager**: Datalog computes all derivations, Salsa is lazy
2. **Shallow checking**: Datalog recursively evaluates, Salsa stops early
3. **Revision tracking**: Automatic in Salsa, manual in Datalog

**Honest Conclusion**: ❌ Datalog cannot natively express Salsa's semantics

**Critical Insight Table**:
| IDE Operation | Latency Budget | Salsa | CozoDB (est) | Acceptable? |
|---------------|----------------|-------|--------------|-------------|
| Completion | <20ms | 5-10ms | 50-100ms | ❌ NO |
| Type hover | <50ms | 10-20ms | 100-200ms | ❌ NO |
| Goto definition | <100ms | 20-50ms | 100-300ms | ⚠️ Borderline |
| Impact analysis | <1s | N/A (hard) | 100-500ms | ✅ YES! |

**The Verdict**: **Hybrid Architecture is the ONLY Viable Option**

**Proposed Architecture**:
```
┌──────────────────────────────────────────┐
│  SALSA (HOT PATH - Real-time)           │
│  - Type checking, completion, diagnostics│
├──────────────────────────────────────────┤
│  COZODB (COLD PATH - Analysis)          │
│  - Impact analysis, refactoring, metrics │
└──────────────────────────────────────────┘
         ↕ Data Sync (Async)
```

**Novel Ideas**:
1. **Salsa → CozoDB Export Tool**: Background task exports Salsa state to CozoDB
2. **CozoDB-Powered LSP Extensions**: New analysis commands (impactAnalysis, dependencyGraph)
3. **Hybrid Query Routing**: Smart router chooses backend per query type

**Updated Decision Matrix**:
| Approach | Performance | Risk | Features | Recommendation |
|----------|-------------|------|----------|----------------|
| Full Migration | ❌ Unacceptable | 🔴 Critical | Same | ❌ **DO NOT PURSUE** |
| Hybrid (Salsa + CozoDB) | ✅ Fast | 🟡 Medium | ✅ +Graph | ✅ **RECOMMENDED** |
| Export Tool Only | ✅ Zero impact | 🟢 Low | ⚠️ Offline | Conservative fallback |

**Next Steps** (Week 1):
1. Write toy Salsa + CozoDB example (confirm performance assumptions)
2. Deep-dive CozoDB triggers (test invalidation capabilities)
3. Prototype export tool (measure overhead)

**Sources**: All claims validated with:
- [CozoDB Official Docs](https://www.cozodb.org/)
- [Salsa Red-Green Algorithm](https://salsa-rs.github.io/salsa/reference/algorithm.html)
- [Salsa Algorithm Explained](https://medium.com/@eliah.lakhin/salsa-algorithm-explained-c5d6df1dd291)
- [CozoDB Performance Benchmarks](https://docs.cozodb.org/en/latest/releases/v0.3.html)
- [Memgraph vs Neo4j](https://memgraph.com/blog/neo4j-vs-memgraph)

**Conclusion**: After rigorous analysis, **hybrid architecture** is the only technically sound approach. Full migration would be **high-risk with likely failure** due to fundamental performance mismatches.

---

## Summary: Complete Analysis Suite

This directory now contains **13 comprehensive documents** (240KB+ total) covering:

### Architectural Understanding (Docs 1-9):
- System architecture, control flow, data flow
- Folder structure, diagrams, API usage
- Per-file analysis, ELI5 explanations
- **Outcome**: Deep understanding of rust-analyzer internals

### Advanced Patterns (Doc 10):
- 43 expert-level Rust patterns (90+ quality score)
- Type-level mastery, metaprogramming, concurrency wizardry
- **Outcome**: Meta-thinking patterns from genius developers

### Dependency Graph Analysis (Doc 11):
- Pyramid analysis (30,000 ft → 100 ft micro-diagrams)
- Complete 39-crate dependency graph
- Graph database mapping strategy (Cypher schema, queries)
- **Outcome**: Roadmap for potential graph DB migration

### Deep Research (Docs 12-13):
- 17-week research plan for Salsa → CozoDB evaluation
- Rubber duck debugging with web validation
- Critical analysis of technical feasibility
- **Outcome**: Evidence-based recommendation against full migration

**Key Findings**:
1. Rust-analyzer has a **perfect DAG structure** (zero circular deps)
2. 39 crates in strict 5-layer architecture
3. Salsa's red-green algorithm is **fundamentally different** from graph DB queries
4. Performance gap: Salsa (microseconds) vs CozoDB (milliseconds) = **100x-1000x**
5. **Hybrid architecture** (Salsa + CozoDB) is the only viable path forward

**Artifacts Generated**:
- `rust-analyzer-dependency-graph.json` (2.8MB): 14,852 entities
- `internal-crate-dependency-graph.json` (12KB): 39 crates with full deps
- `crate-analysis.json` (7.8KB): Entity counts by crate

**Total Documentation**: ~275KB markdown + 3MB JSON data

---

### 14. [Rust-Analyzer vs Tree-Sitter: Entity Key Extraction](./09-RUST-ANALYZER-vs-TREE-SITTER-ENTITY-KEYS.md) 🔑 **CRITICAL FOR V216**
**Purpose**: Compare entity extraction capabilities for V216 primary key requirements

**Context**: V216 PRD requires canonical entity keys with this format:
```
language|||kind|||scope|||name|||file_path|||discriminator
```

**Conclusion**: **Rust-Analyzer is strictly superior for Rust-only codebases**

**Key Findings**:

| V216 Requirement | Rust-Analyzer | Tree-Sitter |
|------------------|---------------|-------------|
| `scope` | ✅ Full module path via `find_path()` | ❌ Cannot resolve |
| `discriminator` | ✅ Type signatures, generics | ❌ No type info |
| Cross-file resolution | ✅ Via HIR layer | ❌ Syntax only |

**Rust-Analyzer's Key APIs**:
1. **`ModuleDefId`** - Canonical entity identity enum (FunctionId, StructId, TraitId, etc.)
2. **`HasModule` trait** - Every entity knows its containing module
3. **`find_path()`** - Returns qualified path respecting edition, cfg, re-exports
4. **Type inference** - Full signatures for overload disambiguation

**Tree-Sitter's Limitations**:
- Produces syntax-level tags (byte ranges, names)
- Cannot follow imports across files
- No semantic understanding (types, scopes)
- Cannot distinguish overloaded methods

**Recommendation for V216**:
- **Commit to Rust-only** → Use rust-analyzer's HIR layer
- **Multi-language** → Accept degraded keys for non-Rust (no proper scope)

**Visual Comparison**:
```
Tree-Sitter:  fn authenticate_user → scope: ❓ UNKNOWN
Rust-Analyzer: fn authenticate_user → scope: auth::service::AuthService
```

**Decision Matrix**:
| Factor | Rust-Analyzer | Tree-Sitter |
|--------|---------------|-------------|
| Semantic accuracy | ★★★★★ | ★★☆☆☆ |
| Scope resolution | ✅ Qualified paths | ❌ Unknown |
| Type information | ✅ Full inference | ❌ None |
| Language support | 1 (Rust) | 12+ languages |

**Sources Analyzed**: `01-hir-ty-patterns.md`, `02-hir-def-patterns.md`, `03-tags-patterns.md`, `05-ide-assists-patterns.md`, `19-cross-cutting-architecture-patterns.md`

---

### 15. [V216 Semantic Context Compressor Thesis](./14-V216-SEMANTIC-CONTEXT-COMPRESSOR-THESIS.md) 🎯 **THE THESIS DOCUMENT**
**Purpose**: Rubber duck debugging of the 3-layer LLM context pipeline thesis

**Method**: Synthesized through rubber duck debugging against rust-analyzer's documented architecture

**The 3-Layer Pipeline**:
```
Layer 1: Semantic Search  → Query → Entity IDs (non-keyword matching)
Layer 2: Entity Anchoring → Entity ID → Signature + Location + Containment
Layer 3: Relationship Expansion → Anchor + Budget → Related Entity Set
```

**Core Thesis Statement**:
> **Parseltongue v216 is a semantic context compressor.**
> It takes a free-form text query from an LLM and produces a maximally information-dense context window through 3 layers: mapping, anchoring, and expansion.

**Key Insights**:

1. **Why Non-Keywords Work**: Keywords are grammar (40 words). Identifiers are semantics (10,000+). Match against semantic fingerprints.

2. **Token Budget Reality**: 10k tokens is way more than needed. Well-structured extraction lands at 2-4k for complex queries, 500-1000 for simple ones.

3. **Information Density Formula**:
   - Signatures over bodies → 10x density increase
   - Relationships over implementations → 5x density increase
   - Types over values → 3x density increase

4. **The Extraction Points**:
   - ItemTree → signatures without bodies
   - DefMap → module structure and visibility
   - TraitImpls → type relationships

5. **What NOT to Build**:
   - Skip full type inference
   - Skip macro expansion details
   - Skip LSP event loop
   - Skip snapshot mechanism

**The Deliverable**: A binary that loads workspace → extracts graph → persists → exposes 3-layer queries via MCP.

**Key Conclusion**: "Rust-analyzer already IS the graph database we need — we just need to serialize what it computes."

---

### 16. [API Stability Verification](./15-API-STABILITY-VERIFICATION.md) 🔍 **FACT-CHECKED**
**Purpose**: Verify whether "API instability" claims in Rust tooling are hearsay or factual

**Method**: GitHub API queries, crates.io data, commit history analysis

**Key Findings**:

| Claim | Reality |
|-------|---------|
| "ra_ap_* APIs break weekly" | **EXAGGERATED** — evcxr updates weekly without issues |
| "Must pin exact versions" | **FALSE** — evcxr and cargo-shear use loose versioning |
| "rustc APIs break every nightly" | **PARTIALLY TRUE** — but Miri shows 1% build fix rate |
| "rustdoc JSON format changes constantly" | **TRUE** — 20 versions in 2025 |

**Real-World Evidence**:

| Project | Strategy | Behind Current | CI Status |
|---------|----------|----------------|-----------|
| evcxr (Google) | Loose (`"0.0.307"`) | 14 versions | ✓ 12 consecutive updates passed |
| cargo-shear | Loose (`"0.0.320"`) | 1 version | ✓ Passing |
| cargo-modules | Exact (`"=0.0.289"`) | 32 versions | ✓ Passing (overly cautious) |

**Breaking Change Estimates**:
- ra_ap_*: 1 in 10-20 releases (~5-10%)
- rustc_private: 1 in 5-10 syncs (~10-20%)
- rustdoc JSON: 5-10 format changes/year

**Conclusion**: Original document's stability claims are **30-40% overstated**. APIs do change, but not catastrophically. Loose versioning works fine.

**Recommendation for Parseltongue**:
```toml
# RECOMMENDED: Loose versioning
ra_ap_hir = "0.0.321"
ra_ap_ide = "0.0.321"

# NOT RECOMMENDED: Exact pinning (overly cautious)
# ra_ap_hir = "=0.0.321"
```


---

### 17. [Rustc Tool Ecosystem Analysis](./16-RUSTC-TOOL-ECOSYSTEM-ANALYSIS.md) 🔬 **COMPREHENSIVE**
**Purpose:** Map all tools using Rust compiler internals for Parseltongue's design

**Method:** Git cloned 25 tools, analyzed 45+ total

**Key Findings:**

| Category | Tools | API Pattern |
|----------|-------|-------------|
| Formal Verification | 9 | MIR → SMT/theorem prover |
| Static Analysis | 12 | MIR/HIR analysis |
| Linting | 4 | AST/HIR traversal |
| Codegen Backends | 7 | MIR → native code |
| Information Flow | 3 | MIR dataflow |

**Five Integration Patterns:**
1. Direct rustc_private (30+ tools)
2. rustc_plugin framework (4 tools)
3. Charon/LLBC decoupling (5+ tools)
4. Stable MIR / rustc_public (1 migrating)
5. No compiler dependency (5+ tools)

**Recommendations for Parseltongue:**
- Use ra_ap_* crates (5-10% break rate, loose versioning works)
- Target HIR for semantic info, not MIR unless needed
- 10 essential use cases identified with accuracy requirements

**Deep Extension Feasibility:**
- Intent router, dual-lane search, entity-wrap: HIGH
- Proof-carrying context, counterfactual: MEDIUM
- Rustc deep mode trigger: LOW
