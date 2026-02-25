# PRD-v144: Parseltongue Comprehensive Feature Roadmap

**Date**: 2026-02-01
**Extracted From**: All PRD-research-20260131v1 documents
**Total Features**: 60+ features, bug fixes, enhancements
**PMF Rating System**: Shreyas Doshi methodology (1-100)

---

## PMF Rating Guide

- **90-100 (Must-Have)**: Solves critical workflow blocker, 10� improvement, daily use
- **70-89 (Performance)**: 3-5� improvement, high frequency + impact
- **50-69 (Power)**: Enables new capabilities, low frequency but very high impact
- **30-49 (Delight)**: Nice to have, makes users smile
- **10-29 (Polish)**: Quality improvement, no new capability

---

## TIER 1: MUST-HAVE (PMF 90-100) - Foundation & Critical Value

### 1. MCP Server Core Implementation
**PMF: 95** | **Version: v1.6** | **Effort: 2-3w**
Implement MCP STDIO transport server exposing all 14 HTTP endpoints as MCP tools for Cursor/VS Code/Claude Code.
Unlocks entire agent ecosystem - foundational for agent-native transformation, enables discovery/invocation by all MCP clients.

### 2. Budget Aware Query Planner
**PMF: 94** | **Version: v1.9** | **Effort: 2w**
Auto-optimize queries to fit token budgets via ?token_budget parameter with graceful degradation strategies.
Makes 200K context feel infinite through intelligent query planning - prevents context overflow, enables self-correction.

### 3. Entity Preview Signature Pointers
**PMF: 93** | **Version: v1.7** | **Effort: 1.5w**
Implement tiered responses (preview/summary/full) with ?detail parameter achieving 90% token reduction.
Show signatures not implementations - agents fetch full bodies on-demand via SigHash pointers, massive efficiency gain.

### 4. NPM Package Publishing Infrastructure
**PMF: 93** | **Version: v1.6** | **Effort: 2d**
Publish @parseltongue/mcp-server to npm for npx installation (npx -y @parseltongue/mcp-server).
Required for MCP server distribution - enables zero-install agent integration, critical path item.

### 5. Tree-Sitter Grammar Updates (12 Languages)
**PMF: 92** | **Version: v2.0** | **Effort: 1w**
Update to latest tree-sitter grammars for Rust/Python/JS/TS/Go/Java/C/C++/Ruby/PHP/C#/Swift.
Critical for AST-based features (cyclomatic complexity, clone detection, entropy) - foundation for all v2.0 analysis.

### 6. Query Token Budget Estimator
**PMF: 90** | **Version: v1.7** | **Effort: 1w**
Add pre-query token estimation with /query-token-budget-estimate and ?dry_run mode (�15% accuracy).
Self-correction warnings before execution - prevents context overflow, enables agents to backtrack before wasting tokens.

### 7. Technical Debt Quantification Scoring
**PMF: 90** | **Version: v2.0** | **Effort: 2.5w**
Implement SQALE method for quantifying technical debt in developer-hours to fix (127 hours = 3 weeks).
Enables business justification for refactoring - tracks growth rate, prevents debt spiral, data-driven prioritization.

---

## TIER 2: PERFORMANCE (PMF 70-89) - Significant Improvements

### 8. Layered Architecture Compliance Verification
**PMF: 89** | **Version: v2.1** | **Effort: 2w**
Implement DSM partitioning with layer constraint checking (no UI�Data shortcuts), automated CI enforcement.
Prevents architecture erosion via automated CI enforcement - 99.8% compliance rate, catches violations in PR.

### 9. Incremental Query Streaming API
**PMF: 88** | **Version: v1.6** | **Effort: 2w**
Implement Server-Sent Events (SSE) streaming for analysis endpoints with progressive results delivery.
Enables sub-second agent feedback vs. 5-10s blocking waits - critical for large codebases, improves agent responsiveness.

### 10. Information-Theoretic Entropy Complexity Measurement
**PMF: 88** | **Version: v2.0** | **Effort: 2w**
Compute Shannon entropy + conditional entropy for code complexity measurement (entropy >3.5 bits = refactor).
Objective complexity metric - correlates with developer intuition, more accurate than cyclomatic complexity alone.

### 11. SARIF Architecture Recovery Integration
**PMF: 87** | **Version: v2.0** | **Effort: 3.5w**
Implement SARIF methodology (36% better than alternatives) for inferring high-level architecture from code.
Auto-generates layer/component/connector diagrams - detects architectural violations, recovers implicit design.

### 12. Hierarchical Module Boundary Detection
**PMF: 86** | **Version: v2.0** | **Effort: 3w**
Implement Leiden algorithm for automatic module boundary detection via modularity optimization (0.73 score).
Discovers true module boundaries vs. folder structure - 90% time saved on architecture analysis (2 days � 2 hours).

### 13. Unix Piping Output Format
**PMF: 85** | **Version: v1.6** | **Effort: 1w**
Add --format flag supporting json/ndjson/tsv/csv output for Unix-style composability (grep | awk | sort).
Enables grep | awk | sort workflows - 3-5� improvement over manual JSON parsing, agent-friendly piping.

### 14. Stateful Query Pagination Bookmarks
**PMF: 85** | **Version: v1.7** | **Effort: 2w**
Implement server-side cursors with /query-pagination-cursor-create/next/close endpoints (30-minute TTL).
Stateful pause/resume queries - beyond SSE streaming, enables agent session continuity across interruptions.

### 15. Coupling and Cohesion Metrics (CK Suite)
**PMF: 85** | **Version: v2.1** | **Effort: 2w**
Implement CK metrics suite (Afferent/Efferent coupling, LCOM, Instability, Abstractness).
Enforces "low coupling, high cohesion" principle - automated CI enforcement, quantifies design quality.

### 16. K-Core Decomposition Layering
**PMF: 84** | **Version: v2.1** | **Effort: 1.5w**
Implement Batagelj-Zaversnik k-core algorithm to identify code layers by coreness (O(E) time).
Distinguishes core business logic (k-coree8) from peripheral utilities (k-core=1-2) - architectural insight.

### 17. Centrality Measures for Entity Importance
**PMF: 83** | **Version: v2.1** | **Effort: 2.5w**
Implement PageRank + Betweenness centrality with CPU-optimized power iteration.
Ranks entity importance - prioritizes documentation/testing/refactoring work, identifies architectural keystone entities.

### 18. Label Propagation Enhanced Clustering
**PMF: 82** | **Version: v2.0** | **Effort: 2w**
Replace current label propagation with GVE-LPA (vector-based) for 3-5� faster convergence.
15 iterations vs. 100 - deterministic clusters align with developer intuition 85% of time, faster module detection.

### 19. Workspace Context Multiplexing
**PMF: 82** | **Version: v1.6** | **Effort: 1.5w**
Add --workspace parameter accepting multiple database paths, parallel query execution with merged results.
Enables monorepo support - agents work across multiple codebases simultaneously, critical for large enterprises.

### 20. Tarjan's Strongly Connected Components
**PMF: 81** | **Version: v2.0** | **Effort: 1.5w**
Implement Tarjan's SCC algorithm for finding tightly coupled code knots (O(V+E) time).
Reveals hidden coupling clusters - prioritizes refactoring by SCC size, identifies circular dependency hotspots.

### 21. CozoDB Version Upgrade
**PMF: 78** | **Version: v1.8** | **Effort: 3d**
Upgrade CozoDB for improved Datalog query performance and query pipeline compilation support.
Supports v1.8 query pipeline composition compilation - performance improvements for complex queries.

### 22. Session Hot Path Cache
**PMF: 78** | **Version: v1.8** | **Effort: 1.5w**
Implement in-memory LRU cache for frequently queried entities/edges per session (>70% hit rate).
10-50� speedup on repeated queries - file-watcher invalidation, massive performance gain for agents.

### 23. Token Estimation Accuracy Improvement
**PMF: 75** | **Version: v1.7** | **Effort: included**
Improve token estimation from �30% (heuristic) to �15% (entity metadata-based calculation).
Critical for v1.7 budget estimator accuracy - prevents over/under-estimation, enables reliable planning.

### 24. Spectral Graph Partition Decomposition
**PMF: 75** | **Version: v2.2** | **Effort: 3w**
Implement normalized spectral clustering with Lanczos eigenvector approximation for balanced partitioning.
Microservice extraction planning with quantified cut cost - 80% time saved on architecture planning.

### 25. ISG Query Composition Pipeline
**PMF: 72** | **Version: v1.8** | **Effort: 2w**
Add /graph-query-pipeline-compose for server-side operation chaining (search|filter|traverse|rank|limit).
Composable graph ops compiled to Datalog - reduces round-trips, enables complex queries in single request.

### 26. Random Walk Probability Impact
**PMF: 72** | **Version: v2.1** | **Effort: 2w**
Implement Monte Carlo random walks for probabilistic blast radius analysis (89% vs. 5% probability).
Quantifies change impact as probability - reduces over-notification by 60%, more accurate than deterministic.

### 27. Code Clone Detection via AST Edit Distance
**PMF: 70** | **Version: v2.2** | **Effort: 3w**
Implement Zhang-Shasha tree edit distance for Type-1/2/3 clone detection (87% recall).
Catches semantic clones with different variable names - identifies refactoring opportunities, reduces duplication.

### 28. Node2Vec Entity Embeddings CPU
**PMF: 70** | **Version: v2.2** | **Effort: 3.5w**
Implement Node2Vec (random walk + Skip-Gram) for 128-dimensional entity embeddings.
Enables semantic similarity searches - exploration by similarity navigation, finds structurally similar code.

---

## TIER 3: POWER (PMF 50-69) - Advanced Capabilities

### 29. Subgraph Export Local Execution
**PMF: 68** | **Version: v1.8** | **Effort: 2w**
Add /graph-subgraph-export-json for exporting ISG subgraphs as standalone JSON files.
Enables custom graph algorithms - offline analysis, integration with NetworkX/Neo4j, power user workflows.

### 30. Weisfeiler-Lehman Graph Kernel Similarity
**PMF: 68** | **Version: v2.2** | **Effort: 3w**
Implement WL graph kernel for structural similarity comparison (finds similar dependency patterns).
Finds code entities with similar dependency patterns - regardless of naming, identifies architectural patterns.

### 31. Dynamic Tool Discovery Endpoint
**PMF: 65** | **Version: v1.6** | **Effort: 1w**
Add MCP resource parseltongue://capabilities returning live tool schemas without restart.
Enables agents to discover tools dynamically - adaptability to new features, runtime introspection.

### 32. RefDiff Refactoring Detection History
**PMF: 65** | **Version: v2.3** | **Effort: 4w**
Implement RefDiff algorithm for detecting refactorings in git history (13 refactoring types).
Tracks entity lineage across renames/moves/extractions - builds genealogy graph, evolution analysis.

### 33. Cyclomatic Complexity per Entity
**PMF: 65** | **Version: v2.0** | **Effort: 1w**
Compute McCabe cyclomatic complexity from tree-sitter AST control flow (CC d10 threshold).
CC d10 threshold enforcement in CI/CD - indicates minimum test cases needed, classic quality metric.

### 34. Graph Query DSL Endpoint
**PMF: 62** | **Version: v1.6** | **Effort: 2w**
Add /graph-query-dsl-execute accepting declarative DSL for custom graph traversals.
Enables complex queries beyond 14 fixed endpoints - power users, custom analysis workflows.

### 35. CLI Flag: --format (json/ndjson/tsv/csv)
**PMF: 60** | **Version: v1.6** | **Effort: included**
Add --format flag to all CLI tools for output format selection (core to Unix piping).
Core to v1.6 Unix piping feature - enables composability with standard Unix tools.

### 36. CLI Flag: --stream
**PMF: 58** | **Version: v1.6** | **Effort: included**
Add --stream flag to enable SSE streaming mode (core to streaming feature).
Core to v1.6 streaming feature - opt-in progressive results delivery.

### 37. Hierarchical Memory Persistence Layer
**PMF: 58** | **Version: v1.6** | **Effort: 2w**
Implement 3-tier agent memory (L1: working set, L2: session, L3: long-term) with query result caching.
Enables agent conversation context persistence - across sessions, reduces re-querying overhead.

### 38. Semantic Query Prompt Templates
**PMF: 55** | **Version: v1.6** | **Effort: 1.5w**
Implement MCP prompts for common queries (analyze-blast-radius, find-circular-deps, show-callers).
Enables natural language queries via MCP prompt system - improves discoverability, easier agent interaction.

### 39. HTTP Response Headers (X-Estimated-Tokens)
**PMF: 55** | **Version: v1.7** | **Effort: 1d**
Add X-Estimated-Tokens header to all query endpoints supporting token budget estimation.
Supports v1.7 token budget estimation - enables agents to pre-check without ?dry_run parameter.

---

## TIER 4: DELIGHT (PMF 30-49) - Nice to Have

### 40. HTTP Response Headers (X-Cache-Status)
**PMF: 45** | **Version: v1.8** | **Effort: 1d**
Add X-Cache-Status header (HIT/MISS) to all query endpoints for cache visibility.
Supports v1.8 session hot path cache visibility - debugging, performance monitoring.

### 41. Documentation: Query Pipeline Examples
**PMF: 42** | **Version: v1.8** | **Effort: 3d**
Add 20+ pipeline query examples to documentation helping power users adopt composition features.
Helps power users adopt v1.8 composition features - reduces learning curve, showcases advanced patterns.

### 42. Documentation: Unix Piping Examples
**PMF: 40** | **Version: v1.6** | **Effort: 3d**
Add 20+ examples of Unix piping workflows (grep | awk | sort) to documentation.
Helps users adopt v1.6 composability features - showcases real-world patterns, reduces friction.

### 43. Four-Word Naming Enforcement Tool
**PMF: 35** | **Version: v2.0** | **Effort: 2d**
Add pre-commit hook to enforce 4-word naming convention compliance automatically.
Maintains codebase consistency - prevents naming violations, automated quality enforcement.

---

## INFRASTRUCTURE & SUPPORTING FEATURES

### MCP SDK Rust Integration
**Type: External Dependency** | **Version: v1.6** | **Effort: included**
Integrate @modelcontextprotocol/sdk-rust for STDIO transport (foundation for MCP server).
Foundation for v1.6 MCP server implementation - critical path external dependency.

### SigHashLookup CozoDB Table
**Type: Database Schema** | **Version: v1.7** | **Effort: included**
Add CozoDB relation for BLAKE3 signature hash � entity key mapping (supports preview/pointer).
Supports v1.7 preview/pointer system - enables pointer-based full entity retrieval.

### QueryCursorState CozoDB Table
**Type: Database Schema** | **Version: v1.7** | **Effort: included**
Add CozoDB relation for pagination cursor state persistence (30-minute TTL).
Supports v1.7 stateful pagination - enables pause/resume across agent sessions.

### Query Parameter: ?detail=preview|pointer|full
**Type: API Enhancement** | **Version: v1.7** | **Effort: included**
Add detail parameter to all entity-returning endpoints (core to preview/pointer feature).
Core to v1.7 preview/pointer feature - 90% token reduction capability.

### Query Parameter: ?dry_run=true
**Type: API Enhancement** | **Version: v1.7** | **Effort: included**
Add dry_run parameter for estimation-only mode (no execution, just token cost).
Core to v1.7 token budget estimator - safe way to check costs before execution.

### Query Parameter: ?session=<id>
**Type: API Enhancement** | **Version: v1.8** | **Effort: included**
Add session parameter to all endpoints for hot path caching (10-50� speedup on hits).
Core to v1.8 session cache feature - enables per-agent caching strategies.

### Query Parameter: ?token_budget=<N>
**Type: API Enhancement** | **Version: v1.9** | **Effort: included**
Add token_budget parameter for auto-optimization (enables graceful degradation).
Core to v1.9 budget planner feature - automatic query optimization to fit constraints.

---

## BUG FIXES (Already Completed or Included)

### File Watcher Deadlock Fix
**Status: Completed v1.4.2** | **PMF: N/A (Critical Fix)**
Fix debouncer-based file watcher deadlock issue preventing event detection.
Already resolved in v1.4.2 - mentioned in git history as completed.

### Token Estimation Accuracy Improvement
**Status: Included in v1.7** | **PMF: 75 (Performance)**
Improve token estimation from �30% to �15% accuracy using entity metadata.
Included in v1.7 budget estimator feature - critical for reliable planning.

---

## ADDITIONAL v2.0+ FEATURES (From Research Papers)

### Feature #18: Program Slicing Backward Forward
**PMF: 72** | **Version: v2.1** | **Effort: 2.5w**
Implement static program slicing for extracting code slices relevant to specific variables/statements.
Enables focused analysis - test prioritization, change impact analysis, surgical debugging.

### Feature #19: Triangle Counting Cohesion Metrics
**PMF: 68** | **Version: v2.2** | **Effort: 1.5w**
Implement fast triangle counting (O(m�dmax)) for clustering coefficient computation.
Quantifies local cohesion - complements global modularity metrics, identifies tight clusters.

### Feature #20: UMAP 2D Code Layout Projection
**PMF: 75** | **Version: v2.1** | **Effort: 2w**
Implement UMAP dimensionality reduction for 2D/3D visualization of codebase structure.
Fast visualization (42s on 3.1GHz Intel for MNIST scale) - preserves global structure better than t-SNE.

### Feature #21: Dependency Structure Matrix Visualization
**PMF: 78** | **Version: v2.1** | **Effort: 1.5w**
Implement DSM visualization with hierarchical clustering and layer identification.
Matrix-based architecture view - identifies layer violations, architectural drift, coupling patterns.

### Feature #22: Interactive Force Layout Graph
**PMF: 65** | **Version: v2.2** | **Effort: 2w**
Implement D3.js-style force-directed graph layout for interactive exploration.
Interactive exploration - zoom/pan/filter, live updates on codebase changes, visual debugging.

### Feature #23: Git Churn Hotspot Correlation
**PMF: 73** | **Version: v2.1** | **Effort: 1.5w**
Correlate git churn metrics with complexity/coupling to identify high-risk code areas.
Identifies high-risk areas - frequently changed + high complexity = bug-prone, prioritizes refactoring.

### Feature #24: Temporal Graph Evolution Snapshots
**PMF: 68** | **Version: v2.2** | **Effort: 2.5w**
Track ISG evolution over time with git commit snapshots and temporal queries.
Architecture evolution tracking - trend analysis, drift detection, historical comparisons.

### Feature #25: Incremental Graph Update Performance
**PMF: 88** | **Version: v2.0** | **Effort: 2w**
Optimize incremental ISG updates (file watcher � entity/edge delta � <100ms update).
Real-time analysis - sub-100ms updates on file changes, enables live architectural monitoring.

### Feature #26: Parallel Graph Algorithm Execution
**PMF: 86** | **Version: v2.0** | **Effort: 2.5w**
Implement parallel execution for graph algorithms on multicore CPUs (96-core benchmarks).
Enterprise-scale performance - 315� speedup over sequential on 96-core, handles massive codebases.

### Feature #27: Graph Compression Sparse Storage
**PMF: 82** | **Version: v2.0** | **Effort: 1.5w**
Implement CSR (Compressed Sparse Row) storage for graph edges reducing memory by 60-80%.
Memory efficiency - 50K entities in <6GB RAM, enables laptop-based analysis of large codebases.

### Feature #28: Approximate Algorithms Massive Graphs
**PMF: 75** | **Version: v2.0** | **Effort: 2w**
Implement approximate algorithms with provable guarantees for 100K+ entity graphs.
Massive codebase support - <5% error with 10� speedup, enables enterprise-scale analysis.

---

## PRIORITY MATRIX (Shreyas Doshi Framework)

### Phase 1: v1.6 Foundation (6.5-7.5 weeks)
**Focus**: Agent-native foundation, MCP ecosystem integration
**Must-Have (PMF 90+)**:
1. MCP Server Core Implementation (PMF 95)
2. NPM Package Publishing (PMF 93)

**Performance (PMF 70-89)**:
3. Unix Piping Output Format (PMF 85)
4. Incremental Query Streaming API (PMF 88)
5. Workspace Context Multiplexing (PMF 82)

**Power (PMF 50-69)**:
6. Dynamic Tool Discovery Endpoint (PMF 65)
7. Semantic Query Prompt Templates (PMF 55)
8. Hierarchical Memory Persistence Layer (PMF 58)
9. Graph Query DSL Endpoint (PMF 62)

---

### Phase 2: v1.7 Agent Memory (4.5 weeks)
**Focus**: Token efficiency, context management, self-correction
**Must-Have (PMF 90+)**:
1. Entity Preview Signature Pointers (PMF 93)
2. Query Token Budget Estimator (PMF 90)

**Performance (PMF 70-89)**:
3. Stateful Query Pagination Bookmarks (PMF 85)
4. Token Estimation Accuracy Improvement (PMF 75)

---

### Phase 3: v1.8 Advanced Patterns (5.5 weeks)
**Focus**: Performance optimization, composition, caching
**Performance (PMF 70-89)**:
1. CozoDB Version Upgrade (PMF 78)
2. Session Hot Path Cache (PMF 78)
3. ISG Query Composition Pipeline (PMF 72)

**Power (PMF 50-69)**:
4. Subgraph Export Local Execution (PMF 68)

---

### Phase 4: v1.9 Intelligence (2 weeks)
**Focus**: Auto-optimization, graceful degradation
**Must-Have (PMF 90+)**:
1. Budget Aware Query Planner (PMF 94)

---

### Phase 5: v2.0 Research Features - Batch 1 (12 weeks)
**Focus**: Core infrastructure, quality metrics, performance
**Must-Have (PMF 90+)**:
1. Tree-Sitter Grammar Updates (PMF 92)
2. Technical Debt Quantification (PMF 90)

**Performance (PMF 70-89)**:
3. Information-Theoretic Entropy (PMF 88)
4. SARIF Architecture Recovery (PMF 87)
5. Hierarchical Module Boundaries (PMF 86)
6. Coupling and Cohesion Metrics (PMF 85)
7. Label Propagation Clustering (PMF 82)
8. Tarjan's SCC (PMF 81)
9. Incremental Graph Update Performance (PMF 88)
10. Parallel Graph Algorithm Execution (PMF 86)
11. Graph Compression Sparse Storage (PMF 82)
12. Approximate Algorithms (PMF 75)

---

### Phase 6: v2.1 Architectural Analysis (10 weeks)
**Focus**: Architecture insights, centrality, visualization
**Performance (PMF 70-89)**:
1. Layered Architecture Compliance (PMF 89)
2. K-Core Decomposition Layering (PMF 84)
3. Centrality Measures (PMF 83)
4. Random Walk Probability Impact (PMF 72)
5. DSM Visualization (PMF 78)
6. UMAP 2D Projection (PMF 75)
7. Git Churn Hotspot Correlation (PMF 73)

**Power (PMF 50-69)**:
8. Program Slicing (PMF 72)

---

### Phase 7: v2.2 Advanced Analysis (12 weeks)
**Focus**: Similarity, clones, embeddings, temporal analysis
**Performance (PMF 70-89)**:
1. Spectral Graph Partition (PMF 75)
2. Code Clone Detection AST (PMF 70)
3. Node2Vec Embeddings (PMF 70)

**Power (PMF 50-69)**:
4. Weisfeiler-Lehman Kernels (PMF 68)
5. Triangle Counting Cohesion (PMF 68)
6. Interactive Force Layout (PMF 65)
7. Temporal Graph Evolution (PMF 68)

---

### Phase 8: v2.3 Polish & Integration (6 weeks)
**Focus**: Evolution tracking, refactoring detection, final integration
**Power (PMF 50-69)**:
1. RefDiff Refactoring Detection (PMF 65)

**Delight (PMF 30-49)**:
2. Documentation & Examples
3. Quality Improvements
4. Four-Word Naming Enforcement (PMF 35)

---

## EFFORT SUMMARY

| Phase | Version | Weeks | Features | PMF Avg |
|-------|---------|-------|----------|---------|
| Phase 1 | v1.6 | 6.5-7.5 | 9 features | 74.2 |
| Phase 2 | v1.7 | 4.5 | 4 features | 85.7 |
| Phase 3 | v1.8 | 5.5 | 4 features | 74.0 |
| Phase 4 | v1.9 | 2 | 1 feature | 94.0 |
| Phase 5 | v2.0 | 12 | 12 features | 85.8 |
| Phase 6 | v2.1 | 10 | 8 features | 78.1 |
| Phase 7 | v2.2 | 12 | 7 features | 70.0 |
| Phase 8 | v2.3 | 6 | 4 features | 50.0 |
| **TOTAL** | **v1.6-v2.3** | **58.5-59.5** | **49 features** | **76.5 avg** |

---

## STRATEGIC RECOMMENDATIONS

### Immediate Actions (Next 2 Weeks)
1. **Start v1.6 MCP Server** (PMF 95) - Foundational, unlocks ecosystem
2. **Validate npm publishing pipeline** (PMF 93) - Required for distribution
3. **Prototype Entity Preview System** (PMF 93, v1.7) - Highest token reduction impact

### Q2 2026 Focus (v1.6-v1.7)
- Complete agent-native foundation (MCP, Unix piping, streaming)
- Implement token efficiency layer (preview/pointer, budget estimator)
- Goal: 90% token reduction, sub-second agent feedback

### Q3 2026 Focus (v1.8-v1.9)
- Performance optimization (cache, parallel, compression)
- Intelligent auto-optimization (budget planner)
- Goal: 10-50� speedup, context-aware planning

### Q4 2026 Focus (v2.0)
- Research-backed analysis features (entropy, debt, SARIF)
- Infrastructure upgrades (tree-sitter, parallel, CSR)
- Goal: Enterprise-scale codebases, automated quality gates

### 2027 Focus (v2.1-v2.3)
- Advanced architectural insights (k-core, centrality, DSM)
- Evolution tracking (churn, refactoring, temporal)
- Visualization & similarity analysis

---

## SUCCESS METRICS

### v1.6-v1.9 (Agent-Native)
- **Adoption**: 80% of developers use MCP tools weekly
- **Efficiency**: 90% token reduction via preview/pointer
- **Performance**: Sub-second agent feedback on 10K entity codebases
- **Intelligence**: 80% of queries auto-optimized to fit budgets

### v2.0-v2.3 (Analysis Depth)
- **Quality**: Technical debt growth <10 hours/quarter
- **Architecture**: Modularity score >0.7, compliance >99%
- **Scale**: 100K entity codebases analyzed in <20 seconds
- **Insight**: 50% faster architecture planning (days � hours)

---

**Document Status**: Comprehensive extraction complete
**Total Items**: 60+ features, fixes, enhancements, dependencies
**Coverage**: All PRD-research-20260131v1 documents analyzed
**Next Steps**: Review with engineering team, prioritize Phase 1 features
