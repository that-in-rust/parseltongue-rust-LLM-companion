# Code-Understanding Domain Thesis

**Version:** 1.0
**Date:** 2026-03-02
**Status:** Complete
**Purpose:** Internal strategic guide for Parseltongue v2.0.0

---

## 0. EXECUTIVE SUMMARY

This thesis presents a comprehensive analysis of the code-understanding domain, with particular focus on graph-based representations and algorithms. The research informs key architectural decisions for Parseltongue v2.0.0, a Rust LLM companion tool for OSS contributors.

### Key Findings by Track

#### 1. Integration Patterns
- **rustc_plugin framework** (used by Flowistry, Aquascope) provides the best balance of compiler access and stability
- **rust-analyzer** offers LSP-based integration with 16,104 stars and active development
- **Direct rustc_private** is fragile and breaks with compiler updates

#### 2. UX/Workflow Patterns
- **ast-grep** (12,685 stars) demonstrates successful structural code search UX
- **Flowistry** shows IDE integration for "focusing on relevant code"
- Multi-step workflows (detect, preview, confirm) are standard

#### 3. Graph Algorithms
- **Production tools use <10%** of available graph algorithms
- **Centrality, community detection, temporal analysis** are severely underutilized
- **Leiden algorithm** is superior to Louvain but absent from code tools
- **Multi-layer graphs** could unify syntax/control/data views

#### 4. Entity/Context Models
- **Code Property Graph (CPG)** is the gold standard but complex (Joern: 2,966 stars)
- **No Rust-native CPG** implementation exists (cpg-rs: 6 stars, early stage)
- **guppy** provides mature Cargo dependency graphs

#### 5. LLM Integration
- **LLMxCPG** and similar approaches combine graphs with LLMs
- **MCP (Model Context Protocol)** servers emerging for tool integration
- **Graph RAG** approaches improving for code understanding

### Direct Implications for Parseltongue v2.0.0

1. **Use petgraph** as core graph library (3,773 stars, Apache licensed, mature)
2. **Build CFG/PDG** using tree-sitter rather than full CPG initially
3. **Implement Leiden** community detection (white space opportunity)
4. **Integrate with rust-analyzer** via LSP for stability
5. **Consider multi-layer graphs** for unified code representation

### Recommended Next Actions

1. **[P0]** Add petgraph dependency and design graph schema
2. **[P0]** Build CFG construction from tree-sitter AST
3. **[P1]** Implement PageRank and betweenness centrality
4. **[P1]** Port Leiden algorithm for module clustering
5. **[P2]** Design multi-layer code graph representation

---

## 1. INTEGRATION PATTERN LANDSCAPE

### 5 Patterns for Rust Compiler Tools

| Pattern | Tools | Stability | Capability | Recommendation |
|---------|-------|-----------|------------|----------------|
| **rustc_private** | 30+ tools | LOW | HIGH | Avoid |
| **rustc_plugin** | Flowistry, Aquascope, Paralegal | MEDIUM | HIGH | Consider |
| **Charon/LLBC** | Aeneas, Hax | MEDIUM | MEDIUM | Research |
| **Stable MIR** | Emerging | HIGH | MEDIUM | Watch |
| **No Compiler** | ast-grep, tree-sitter | HIGH | MEDIUM | **Recommended** |

### Analysis

**rustc_private** provides maximum capability but breaks with every compiler update. Most tools using this pattern are abandoned or constantly playing catch-up.

**rustc_plugin** (developed by Will Crichton for Flowistry) provides a more stable abstraction layer. Tools using this pattern (Flowistry: 3,028 stars, Aquascope: 2,992 stars) demonstrate sustained development.

**No Compiler Dependency** (ast-grep: 12,685 stars) shows the most success. Using tree-sitter for parsing avoids compiler coupling entirely while still enabling sophisticated analysis.

### Parseltongue Decision

**Primary: No Compiler Dependency + rust-analyzer LSP**
- Use tree-sitter for AST/CFG construction
- Integrate with rust-analyzer for semantic information
- This provides stability while maintaining good capability

---

## 2. UX/WORKFLOW PATTERNS IN THE WILD

### The 7-Moment UX Blueprint (From Design Doc)

| Moment | Pattern Observed |
|--------|-----------------|
| 0. Intent Classification | ast-grep uses pattern matching |
| 1. Intent Confirmation | Preview before action |
| 2. Option Cards Presented | List of matches/findings |
| 3. Card Details Viewed | Expandable details |
| 4. Context Preview | Inline code preview |
| 5. Deep Dive | Full file navigation |
| 6. Action Bar | Apply/Reject/Modify |
| 7. Final Answer | Confirm changes |

### Disambiguation Patterns

- **Flowistry**: Visual highlighting of "relevant code"
- **ast-grep**: Pattern-based matching with variables
- **Joern**: Query language for CPG traversal

### Option Card Presentation

- **List view** with expandable details (most common)
- **Diff view** for change preview
- **Graph visualization** for dependencies

### Parseltongue Decision: UX Differentiators

1. **Graph-aware presentation**: Show code relationships visually
2. **Impact preview**: "Blast radius" of changes using graph analysis
3. **Community view**: Module clusters from community detection
4. **Centrality indicators**: Highlight "important" code

---

## 3. GRAPH ALGORITHMS FOR CODE-GRAPHS

### 3.1 Taxonomy (from arXiv Research)

#### Algorithm Categories

| Category | Algorithms | Code Application |
|----------|-----------|------------------|
| **Traversal** | BFS, DFS, A* | Reachability, dependency chains |
| **Path/Distance** | Dijkstra, Floyd-Warshall | Impact distance, coupling |
| **Centrality** | PageRank, Betweenness, Closeness | Code importance, critical paths |
| **Community Detection** | Leiden, Louvain, Infomap | Module clustering, architecture |
| **Subgraph Mining** | Frequent patterns, motifs | Code patterns, anti-patterns |
| **GNN** | GCN, GAT, GraphSAGE | Vulnerability detection, similarity |
| **Temporal** | TGN, dynamic embeddings | Code evolution |
| **Multi-layer** | Multiplex analysis | Unified code views |

#### Centrality Measures (arXiv:2507.06164)

| Measure | Complexity | Use Case |
|---------|-----------|----------|
| Degree | O(n) | Local importance |
| PageRank | O(kE) | Global importance |
| Betweenness | O(VE) | Critical paths |
| Closeness | O(V+E) | Centrality in module |
| K-core | O(E) | Complexity hotspots |

### 3.2 Production Landscape (from GitHub Research)

#### Graph Libraries in Rust

| Library | Stars | Algorithms | License |
|---------|-------|------------|---------|
| **petgraph** | 3,773 | Traversal, shortest path, simple centrality | Apache-2.0 |
| **neo4j-labs/graph** | 431 | High-performant algorithms | MIT |
| **graphalgs** | 30 | Extended petgraph | MIT |

#### What's in Production

- **petgraph**: BFS, DFS, Dijkstra, Bellman-Ford, simple centrality
- **guppy**: Dependency graph queries
- **Joern**: CPG-based pattern matching

#### What's NOT in Production

- **Leiden community detection**: 0 mature Rust implementations
- **Temporal graph analysis**: No code evolution tools
- **k-core decomposition**: Not used for complexity analysis
- **Multi-layer graphs**: Theoretical only

### 3.3 Novel Alpha (Cross-Pollination)

| Source Domain | Algorithm | Code Application | Opportunity |
|---------------|-----------|------------------|-------------|
| **Bioinformatics** | Diffusion-based ranking | Code importance scoring | HIGH |
| **Social Networks** | Influence propagation | Change impact analysis | MEDIUM |
| **Fraud Detection** | Anomaly detection in graphs | Bug/vulnerability patterns | HIGH |
| **Knowledge Graphs** | Entity resolution | Type inference | MEDIUM |

#### White Space Opportunities

1. **Leiden for Module Clustering**
   - Superior to Louvain (resolution limit fixed)
   - No Rust implementation for code
   - Could identify cohesive module groups

2. **k-Core for Complexity Hotspots**
   - Identifies densely connected subgraphs
   - Natural fit for "code complexity" visualization
   - Simple to implement

3. **Temporal Code Graphs**
   - Track code evolution over time
   - Identify "stable" vs "volatile" areas
   - Support archaeology queries

4. **Multi-Dimensional Code Graphs**
   - Combine syntax/control/data/types layers
   - Cross-layer analysis
   - Unified view of code structure

### 3.4 Parseltongue Decision Matrix

| Capability | Decision | Rationale | Timeline |
|------------|----------|-----------|----------|
| Graph storage | **BUY (petgraph)** | Mature, Apache licensed | Immediate |
| CFG construction | **BUILD** | Use tree-sitter | v2.0 |
| PDG construction | **BUILD** | Add data dependencies | v2.0 |
| CPG (full) | **DEFER** | Complex, wait for cpg-rs | Future |
| PageRank | **BUILD** | Simple on petgraph | v2.0 |
| Betweenness | **BUILD** | Available in petgraph | v2.0 |
| Leiden | **BUILD** | No Rust alternative | v2.1 |
| Louvain | **SKIP** | Leiden is superior | - |
| GNN | **DEFER** | Research stage | Future |
| Visualization | **BUY (egui_graphs)** | petgraph compatible | v2.0 |

---

## 4. ENTITY/CONTEXT MODELS

### Code Entity Representations

#### Statement Level
- **AST nodes**: Language constructs
- **CFG basic blocks**: Control flow units
- **CPG nodes**: Combined representation

#### Function Level
- **Function signature**: Name, parameters, return type
- **Call graph nodes**: Interprocedural relationships
- **Function embeddings**: Semantic vectors

#### Module Level
- **Module graphs**: Import/export relationships
- **Dependency edges**: Use relationships
- **Community membership**: Cluster assignment

#### Crate/Ecosystem Level
- **Cargo dependency graph**: Package relationships
- **Supply chain graph**: Transitive dependencies
- **Version evolution**: Temporal edges

### Graph Schema Design

```rust
// Proposed Parseltongue graph schema
struct CodeGraph {
    // Nodes
    statements: Vec<StatementNode>,
    functions: Vec<FunctionNode>,
    modules: Vec<ModuleNode>,
    crates: Vec<CrateNode>,

    // Edges
    control_flow: Vec<CFEdge>,
    data_flow: Vec<DFEdge>,
    calls: Vec<CallEdge>,
    contains: Vec<ContainsEdge>,
    depends_on: Vec<DependencyEdge>,
}

struct StatementNode {
    id: NodeId,
    ast_kind: AstKind,
    text: String,
    span: Span,
    metrics: StatementMetrics,
}

struct FunctionNode {
    id: NodeId,
    name: String,
    signature: String,
    complexity: f64,
    centrality: f64,
    community: Option<CommunityId>,
}
```

### Freshness and Versioning

- **Incremental updates**: Track changed files
- **Version tags**: Mark graph versions
- **Diff graphs**: Store changes only

---

## 5. LLM INTEGRATION PATTERNS

### Current Approaches

| Pattern | Tools | Description |
|---------|-------|-------------|
| **Context packaging** | Cursor, Continue | Select relevant code for LLM |
| **Graph RAG** | GRAG, GNN-RAG | Graph-based retrieval |
| **MCP servers** | rust-analyzer-mcp, mcp-joern | Protocol-based integration |
| **LLMxCPG** | Research (2025) | CPG-guided LLM analysis |

### Token Budgeting Strategies

1. **Centrality-based**: Include high-centrality code first
2. **Community-based**: Include representative from each cluster
3. **Impact-based**: Include code in blast radius
4. **Query-focused**: Include relevant subgraph

### Parseltongue Decision: LLM Integration Architecture

```
┌─────────────────────────────────────────────────┐
│                   Parseltongue                   │
├─────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────────────────┐ │
│  │ Code Graph  │    │   Graph Algorithms      │ │
│  │ (petgraph)  │───▶│ - Centrality            │ │
│  └─────────────┘    │ - Community Detection   │ │
│        │            │ - Impact Analysis       │ │
│        ▼            └─────────────────────────┘ │
│  ┌─────────────┐              │                 │
│  │  Context    │◀─────────────┘                 │
│  │  Selector   │                                │
│  └─────────────┘                                │
│        │                                        │
│        ▼                                        │
│  ┌─────────────┐                                │
│  │    LLM      │                                │
│  │ Integration │                                │
│  └─────────────┘                                │
└─────────────────────────────────────────────────┘
```

---

## 6. STRATEGIC RECOMMENDATIONS

### Priority-Ranked Action Items

#### P0: Critical Path (Must Do for v2.0.0)

| # | Action | Why Critical | Effort | Dependencies |
|---|--------|--------------|--------|--------------|
| 1 | Add petgraph dependency | Foundation for all graph work | Low | None |
| 2 | Design graph schema | Enables all subsequent work | Medium | #1 |
| 3 | Build CFG from tree-sitter | Core analysis capability | Medium | #2 |
| 4 | Implement PageRank | Code importance scoring | Low | #2 |
| 5 | Build context selector | LLM integration | Medium | #3, #4 |

#### P1: High Priority (Should Do)

| # | Action | Impact | Effort | Dependencies |
|---|--------|--------|--------|--------------|
| 6 | Implement betweenness centrality | Critical path identification | Low | P0 |
| 7 | Build PDG construction | Data flow analysis | Medium | P0 |
| 8 | Implement Leiden algorithm | Module clustering | Medium | P0 |
| 9 | Add k-core decomposition | Complexity hotspots | Low | P0 |
| 10 | Build visualization layer | User-facing value | Medium | P0 |

#### P2: Medium Priority (Nice to Have)

| # | Action | Impact | Effort | Dependencies |
|---|--------|--------|--------|--------------|
| 11 | Temporal graph tracking | Code evolution | Medium | P1 |
| 12 | Multi-layer graph support | Unified views | High | P1 |
| 13 | GNN experimentation | Advanced analysis | High | P1 |
| 14 | Cross-language support | Broader applicability | High | P1 |

#### P3: Future Consideration

| # | Action | When to Revisit |
|---|--------|-----------------|
| 15 | Full CPG implementation | When cpg-rs matures |
| 16 | GNN-based vulnerability detection | When training data available |
| 17 | Multi-language support | Based on user demand |

### Risk Assessment

#### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| petgraph limitations | Low | Medium | Extend with custom algorithms |
| tree-sitter CFG accuracy | Medium | High | Validate with rust-analyzer |
| Performance at scale | Medium | High | Profile early, use incremental |
| Leiden implementation complexity | Medium | Low | Start with existing papers |

#### Strategic Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Joern releases Rust support | Low | Medium | Differentiate on UX |
| GNN approaches become dominant | Medium | Medium | Keep research branch |
| LLM context windows expand | High | Low | Focus on precision over recall |

### Decision Dependencies

```
petgraph ──────▶ graph schema ──────▶ CFG construction
                      │                      │
                      ▼                      ▼
              centrality impl ◀──────── PDG construction
                      │
                      ▼
              context selector ───────▶ LLM integration
                      │
                      ▼
              Leiden algorithm ───────▶ module clustering
```

---

## Appendix A: Research Methodology

### arXiv Research
- Used WebSearch with `site:arxiv.org` queries
- Focused on 2023-2026 papers
- Prioritized survey papers for breadth

### GitHub Research
- Used `gh search repos` CLI commands
- Filtered by language (Rust) where relevant
- Analyzed 100+ repositories at high level
- Deep analysis of 20+ key repositories

### Cross-Reference
- Validated arXiv findings against GitHub
- Identified gaps (papers without code)
- Identified opportunities (algorithms not applied to code)

---

## Appendix B: Key Papers Reviewed

### Graph Theory (Phase 0a)
1. Critical Nodes Identification Survey (arXiv:2507.06164)
2. Dynamic Community Detection with Leiden (arXiv:2405.11658)
3. A Survey of Dynamic Graph Neural Networks (arXiv:2404.18211)
4. TGLib: Temporal Graph Analysis (arXiv:2209.12587)
5. Graph Foundation Models Survey (arXiv:2310.11829)

### Code Graphs (Phase 0b)
1. LLMxCPG (arXiv:2507.16585)
2. GNN-Powered Vulnerability Path Discovery (arXiv:2507.17888)
3. Semantic Code Graph (arXiv:2310.02128)
4. Boosting Vulnerability Detection (arXiv:2506.21014)

### Intersection (Phase 0c)
1. Devign (NeurIPS 2019)
2. ReGVD (arXiv:2110.07317)
3. MVD (ICSE 2022)
4. VulCNN (ICSE 2022)

---

## Appendix C: Key Repositories Analyzed

### Graph Libraries
- petgraph/petgraph (3,773 stars)
- neo4j-labs/graph (431 stars)
- starovoid/graphalgs (30 stars)

### Code Analysis
- joernio/joern (2,966 stars)
- ast-grep/ast-grep (12,685 stars)
- biomejs/gritql (4,422 stars)

### Rust Tools
- rust-lang/rust-analyzer (16,104 stars)
- willcrichton/flowistry (3,028 stars)
- cognitive-engineering-lab/aquascope (2,992 stars)
- brownsys/paralegal (44 stars)

### Dependency Analysis
- guppy-rs/guppy (258 stars)
- rust-secure-code/cargo-supply-chain (348 stars)
- jplatte/cargo-depgraph (213 stars)

### Community Detection
- oliveira-sh/pymocd (17 stars)
- fa-leiden-cd (0 stars)

---

## Appendix D: Code-as-Graph Dimension Matrix

| Representation | Level | Syntax | Control | Data | Type | Semantic | Time |
|----------------|-------|--------|---------|------|------|----------|------|
| AST | Statement | X | - | - | - | - | - |
| CFG | Statement | - | X | - | - | - | - |
| DFG | Statement | - | - | X | - | - | - |
| PDG | Statement | - | X | X | - | - | - |
| CPG | Statement | X | X | X | - | - | - |
| Call Graph | Function | - | X | - | - | - | - |
| Type Graph | Module | - | - | - | X | - | - |
| SDG | Program | - | X | X | - | - | - |
| Dependency Graph | Module | - | - | X | X | - | - |
| Temporal Graph | Any | - | - | - | - | - | X |

---

*Thesis completed: 2026-03-02*
*Research conducted using arXiv and GitHub CLI*
*For questions, refer to research journal at docs/plans/2026-03-02-research-journal.md*
