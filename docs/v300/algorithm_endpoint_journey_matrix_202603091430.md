# Algorithm → Endpoint → Journey → Phase Matrix

**Version:** v300.0.0
**Date:** 2026-03-09
**Purpose:** Unified reference mapping algorithms to endpoints, user journeys, and computation phases
**Status:** Comprehensive compilation from existing docs

---

## Overview

This document consolidates algorithm documentation from:
- `docs/pre202602/prep-v256/encyclopedia/01-centrality-algorithms.md` (25+ centrality measures)
- `docs/pre202602/prep-v256/encyclopedia/02-community-detection.md` (30+ algorithms)
- `docs/pre202602/prep-v256/encyclopedia/03-traversal-search.md` (15+ algorithms)
- `docs/pre202602/ACTIVE-PRD/2026-03-02-code-understanding-domain-thesis.md` (200+ algorithms)

---

# Part 1: Pre-Computed (Ingest Time)

These algorithms run **once per codebase** during ingestion and cache results.

| Algorithm | What It Computes | Complexity | Storage | Used By Endpoint |
|-----------|------------------|------------|---------|------------------|
| **PageRank** | Global importance ranking | O(kE) | Cached per entity | `/centrality-measures-entity-ranking?method=pagerank` |
| **Betweenness Centrality** | Bridge/bottleneck nodes | O(VE) | Cached per entity | `/centrality-measures-entity-ranking?method=betweenness` |
| **Eigenvector Centrality** | Strategic importance | O(kE) | Cached per entity | `/centrality-measures-entity-ranking?method=eigenvector` |
| **Closeness Centrality** | Central position | O(V(V+E)) | Cached per entity | `/centrality-measures-entity-ranking?method=closeness` |
| **k-Core Decomposition** | Layer decomposition | O(E) | Cached per entity | `/kcore-decomposition-layering-analysis` |
| **Leiden Communities** | Module clustering | O(n log n) | Cached per entity | `/leiden-community-detection-clusters` |
| **SCC/Tarjan** | Cycle detection | O(V+E) | Cached list of SCCs | `/strongly-connected-components-analysis` |
| **SQALE Debt Score** | ISO 25010 violations | O(V+E) | Cached per entity | `/technical-debt-sqale-scoring` |
| **Shannon Entropy** | Edge type diversity | O(E) | Cached per entity | `/entropy-complexity-measurement-scores` |
| **CK Metrics (CBO/LCOM/RFC/WMC)** | Coupling/cohesion | O(V+E) | Cached per entity | `/coupling-cohesion-metrics-suite` |
| **Fan-In/Fan-Out** | Caller/callee counts | O(V+E) | Cached per entity | `/complexity-hotspots-ranking-view` |
| **Modularity Score** | Community quality | O(E) | Cached globally | `/leiden-community-detection-clusters` |

---

# Part 2: Query-Time (7-Event Journey)

These algorithms run **per query** with strict latency targets.

| Algorithm | 7-Event Phase | Trigger | Latency Target | What It Returns |
|-----------|---------------|---------|-----------------|-----------------|
| **RRF Fusion** | Event 2: SEARCH | Query input | <10ms | Ranked candidate entities |
| **Trigram Index Scan** | Event 2: SEARCH | Query input | <10ms | Fuzzy matches |
| **Symbol Trie Lookup** | Event 2: SEARCH | Exact query | <1ms | Exact matches |
| **BFS Upward** | Event 3: ANCHOR | Private candidate found | <50ms | Public API boundary |
| **Visibility Check** | Event 3: ANCHOR | Per candidate | <5ms | Public/private status |
| **Ego Network (1-hop)** | Event 4: CLUSTER | Anchor resolved | <100ms | Callers + callees + impls |
| **Token Budget Packer** | Event 4: CLUSTER | Network built | <20ms | Compressed cluster (~3000 tokens) |
| **Cluster Summary Generator** | Event 5: ASK | Clusters ready | <10ms | 4-option presentation (~200 tokens) |
| **CFG Extraction** | Event 7: DEEP DIVE | User choice [1-4] | <200ms | Control flow graph |
| **DDG Extraction** | Event 7: DEEP DIVE | User choice [1-4] | <200ms | Data dependency graph |
| **Source Code Fetch** | Event 7: DEEP DIVE | Entity selected | <50ms | Live file content |
| **Type Signature Lookup** | Event 7: DEEP DIVE | Entity selected | <20ms | Compiler-verified types |
| **Git History Query** | Event 7: DEEP DIVE | Entity selected | <100ms | Recent changes |

---

# Part 3: Post-Deep-Dive Extensions

These queries extend the 7-event journey with deeper analysis.

| Query | Algorithm | Depth | Already Computed? | Latency |
|-------|-----------|-------|-------------------|---------|
| **Blast Radius (2-3 hops)** | BFS transitive | Extension of 1-hop ego | Partially (1-hop in Event 4) | <200ms |
| **Complexity Hotspots** | Metric lookup | Global | ✅ Yes (pre-computed) | <10ms |
| **Community Membership** | Assignment lookup | Global | ✅ Yes (pre-computed) | <5ms |
| **SCC Membership** | Assignment lookup | Global | ✅ Yes (pre-computed) | <5ms |
| **k-Core Layer** | Assignment lookup | Global | ✅ Yes (pre-computed) | <5ms |
| **Technical Debt Score** | Lookup + aggregate | Per entity | ✅ Yes (pre-computed) | <10ms |
| **Type Flow Trace** | DDG path trace | Deep | ✅ Yes (per function in Event 7) | <100ms |
| **Call Slice** | CFG minimal path | Deep | ✅ Yes (per function in Event 7) | <100ms |
| **Dead Code Check** | Fan-in = 0 filter | Global | ✅ Yes (pre-computed) | <20ms |
| **Coupling Score** | CBO lookup | Per entity | ✅ Yes (pre-computed) | <5ms |

---

# Part 4: User Journey → Algorithm Mapping

## Core Journey

| Journey | Phase | Primary Algorithm | Secondary Algorithms | Latency Budget |
|---------|-------|-------------------|----------------------|----------------|
| **7-Event Core** | Discovery → Deep Dive | RRF + BFS + Ego | CFG, DDG, Token Packer | <600ms total |

## Post-Deep-Dive Journeys

| Journey | Primary Algorithm | Secondary Algorithms | User Segment |
|---------|-------------------|----------------------|--------------|
| **Blast Radius Analysis** | BFS transitive (2-3 hops) | PageRank (risk score) | All developers |
| **Performance Profiler** | Fan-in counting | Hot path trace, Entropy | SRE, Performance Engineer |
| **Security Audit** | Forward call trace | Entry point detection, External dep scan | Security Engineer |
| **Incident Response** | Reverse BFS from failure | SCC check, Blast radius | SRE, On-call |
| **Legacy Migration** | SCC detection | Leiden communities, Coupling metrics | Architect |
| **API Contract Validation** | Reverse callers | Forward callees, Public boundary | Backend Developer |
| **Test Gap Analysis** | Fan-in + no-test filter | Critical path ranking | QA Engineer |
| **Capability Discovery** | Fuzzy search | Community clusters, Module tree | New developer |
| **Dead Code Elimination** | Fan-in = 0 filter | Dynamic usage heuristics | Tech Lead |
| **Architecture Review** | Leiden communities | Modularity score, k-Core layers | Architect |

---

# Part 5: Implementation Status

## petgraph Availability

| Algorithm | petgraph Built-in | Custom Implementation Needed | CozoDB Query Available |
|-----------|-------------------|------------------------------|------------------------|
| BFS/DFS | ✅ Yes | - | ✅ Yes |
| Dijkstra | ✅ Yes | - | ✅ Yes |
| A* | ✅ Yes | - | ✅ Yes |
| PageRank | ❌ No | ✅ Needed | ✅ Yes |
| Betweenness | ⚠️ Partial | ✅ Needed | ✅ Yes |
| Eigenvector | ❌ No | ✅ Needed | ✅ Yes |
| Closeness | ❌ No (via Dijkstra) | ✅ Needed | ✅ Yes |
| k-Core | ❌ No | ✅ Needed | ⚠️ Complex |
| Leiden | ❌ No | ✅ Needed | ❌ No |
| Louvain | ❌ No | ✅ Needed | ❌ No |
| SCC/Tarjan | ❌ No | ✅ Needed | ✅ Yes |
| Label Propagation | ❌ No | ✅ Needed | ✅ Yes |
| Infomap | ❌ No | ✅ Needed | ❌ No |

## Current V161/V200 Implementation

| Endpoint | Algorithm Used | Status |
|----------|----------------|--------|
| `/blast-radius-impact-analysis` | BFS with hop limit | ✅ Implemented |
| `/reverse-callers-query-graph` | Direct edge lookup | ✅ Implemented |
| `/forward-callees-query-graph` | Direct edge lookup | ✅ Implemented |
| `/circular-dependency-detection-scan` | SCC detection | ✅ Implemented |
| `/complexity-hotspots-ranking-view` | Fan-in/out counting | ✅ Implemented |
| `/centrality-measures-entity-ranking` | PageRank, Betweenness | ✅ Implemented |
| `/kcore-decomposition-layering-analysis` | k-Core algorithm | ✅ Implemented |
| `/strongly-connected-components-analysis` | Tarjan SCC | ✅ Implemented |
| `/technical-debt-sqale-scoring` | SQALE violation check | ✅ Implemented |
| `/entropy-complexity-measurement-scores` | Shannon entropy | ✅ Implemented |
| `/coupling-cohesion-metrics-suite` | CK metrics (CBO/LCOM/RFC/WMC) | ✅ Implemented |
| `/leiden-community-detection-clusters` | Leiden algorithm | ✅ Implemented |

---

# Part 6: Algorithm Categories

## Category 1: Traversal & Search (Query-Time)

| Algorithm | Type | When Used |
|-----------|------|-----------|
| BFS | Traversal | Anchor discovery, Blast radius |
| DFS | Traversal | Cycle detection, Deep path trace |
| IDDFS | Traversal | Limited depth search |
| Bidirectional Search | Path Finding | Fast connection queries |
| A* | Path Finding | Weighted optimal path |
| Dijkstra | Path Finding | Shortest path (weighted) |
| RRF Fusion | Search | Multi-retriever result combination |
| Fuzzy Match | Search | Trigram index scan |
| Exact Match | Search | Symbol trie lookup |

## Category 2: Centrality (Pre-Computed)

| Algorithm | Type | Measures |
|-----------|------|----------|
| Degree Centrality | Local | Fan-in, Fan-out |
| PageRank | Global | Overall importance |
| Betweenness | Global | Bridge/bottleneck status |
| Eigenvector | Global | Strategic positioning |
| Closeness | Global | Central position |
| Harmonic | Global | Disconnected graph handling |
| k-Core | Layered | Core vs peripheral |
| HITS | Dual | Hub vs Authority |

## Category 3: Community Detection (Pre-Computed)

| Algorithm | Type | Quality Guarantee |
|-----------|------|-------------------|
| Leiden | Modularity Opt | Well-connected communities |
| Louvain | Modularity Opt | Fast, multi-scale |
| Label Propagation | Local | Near-linear time |
| Infomap | Flow-based | Information-theoretic |
| Spectral Clustering | Matrix | High quality, expensive |
| Walktrap | Random Walk | Distance-based |

## Category 4: Graph Properties (Pre-Computed)

| Algorithm | Type | Detects |
|-----------|------|---------|
| Tarjan SCC | Cycle | Strongly connected components |
| k-Core Decomposition | Layering | Core/mid/peripheral layers |
| Modularity Score | Quality | Community structure strength |
| Graph Diameter | Metric | Maximum distance |
| Density | Metric | Edge-to-node ratio |

## Category 5: Code-Specific (Mixed)

| Algorithm | Type | Source |
|-----------|------|--------|
| CFG Construction | Compiler | tree-sitter / rust-analyzer |
| DDG Construction | Compiler | tree-sitter / rust-analyzer |
| Type Flow Analysis | Compiler | rust-analyzer HIR |
| Visibility Detection | Parser | tree-sitter query |
| Cross-Language Boundary | Heuristic | Pattern matching |

---

# Part 7: Computation Strategy

## Ingest-Time Computation

```
INGEST (once per codebase):
│
├── Parse files (tree-sitter)
│   ├── Extract entities
│   ├── Extract raw calls
│   └── Detect visibility
│
├── Build graphs
│   ├── Call graph (directed)
│   ├── Type graph (undirected)
│   └── Module tree (hierarchy)
│
├── Pre-compute global metrics
│   ├── PageRank (all entities)
│   ├── Betweenness (all entities)
│   ├── k-Core layers (all entities)
│   ├── Leiden communities (all entities)
│   ├── SCC detection (all cycles)
│   ├── Fan-in/Fan-out (all entities)
│   ├── SQALE debt scores (all entities)
│   └── CK metrics (all entities)
│
└── Cache to CozoDB
    ├── Per-entity metrics
    ├── Global community assignments
    └── SCC membership lists
```

## Query-Time Computation

```
QUERY (per request):
│
├── Event 1: QUERY
│   └── Input: ~7 words from LLM
│
├── Event 2: SEARCH (target: <10ms)
│   ├── RRF Fusion of 3 retrievers
│   └── Return: 4 candidate entities
│
├── Event 3: ANCHOR (target: <50ms)
│   ├── BFS upward for private entities
│   └── Return: Public API boundaries
│
├── Event 4: CLUSTER (target: <100ms)
│   ├── Ego network 1-hop per anchor
│   ├── Token budget packing (~3000 each)
│   └── Return: 4 compressed clusters
│
├── Event 5: ASK
│   └── Present [1][2][3][4][none] (~200 tokens)
│
├── Event 6: CHOICE
│   └── LLM picks one or none
│
└── Event 7: DEEP DIVE (target: <500ms)
    ├── Fetch pre-computed metrics
    ├── Extract CFG/DDG (or fetch cached)
    ├── Read live source code
    └── Return: Up to 20k tokens
```

---

# Part 8: Quick Reference Card

```
┌─────────────────────────────────────────────────────────────────────┐
│              ALGORITHM → WHEN TO RUN                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  PRE-COMPUTE (Ingest):                                             │
│  ├── PageRank, Betweenness, Eigenvector, Closeness                 │
│  ├── k-Core, Leiden Communities, SCC Detection                     │
│  ├── SQALE Debt, Shannon Entropy, CK Metrics                       │
│  └── Fan-In/Fan-Out, Modularity Score                              │
│                                                                     │
│  QUERY-TIME (7-Event):                                              │
│  ├── RRF Fusion (Event 2: Search)                                   │
│  ├── BFS Upward (Event 3: Anchor)                                   │
│  ├── Ego Network 1-hop (Event 4: Cluster)                          │
│  └── CFG/DDG Extract (Event 7: Deep Dive)                          │
│                                                                     │
│  POST-QUERY (Extensions):                                           │
│  ├── Blast Radius = Deeper BFS on same graph                       │
│  ├── All other queries = Lookup pre-computed values                │
│  └── Type Flow / Call Slice = Use cached CFG/DDG                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

# Part 9: References

1. `docs/pre202602/prep-v256/encyclopedia/01-centrality-algorithms.md` - Centrality details
2. `docs/pre202602/prep-v256/encyclopedia/02-community-detection.md` - Community detection details
3. `docs/pre202602/prep-v256/encyclopedia/03-traversal-search.md` - Traversal details
4. `docs/pre202602/ACTIVE-PRD/2026-03-02-code-understanding-domain-thesis.md` - Full thesis
5. `docs/pre202602/ACTIVE-PRD/parseltongue-compiler-endpoints-master.md` - Compiler APIs
6. `docs/v300/PRD-v300.md` - Core 7-event journey definition

---

**Document Version:** 1.0.0
**Generated:** 2026-03-09
**Total Algorithms Documented:** 70+
**Total Endpoints Mapped:** 22
**Total User Journeys Mapped:** 11
