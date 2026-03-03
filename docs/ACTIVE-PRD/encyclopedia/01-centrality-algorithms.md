# Centrality Algorithms for Code Graphs

**Category:** Node Importance Measures
**Target:** 25+ centrality measures documented
**Purpose:** Identify important, critical, and high-impact code entities

---

## Overview

Centrality algorithms answer the fundamental question: **"Which nodes matter most?"**

In code graphs, this translates to:
- Which functions are most critical to the system?
- What code changes would have the most impact?
- Where should refactoring efforts focus?
- Which modules are architectural hotspots?

---

## Taxonomy of Centrality Measures

### 1. DEGREE-BASED CENTRALITY

#### 1.1 Degree Centrality
```
Definition: Number of edges connected to a node
Formula:    C_D(v) = deg(v) / (n-1)
Complexity: O(V + E)
```

**Code Application:**
- Nodes = Functions
- High degree = Many callers OR many callees
- Interpretation: "This function is heavily connected"

**In-Degree (Fan-In):**
```
C_in(v) = number of callers
Interpretation: "How many functions depend on this?"
```

**Out-Degree (Fan-Out):**
```
C_out(v) = number of callees
Interpretation: "How many dependencies does this have?"
```

**Implementation:**
```rust
fn degree_centrality(graph: &Graph<Node, Edge>) -> HashMap<NodeIndex, f64> {
    let n = graph.node_count() as f64;
    graph.node_indices()
        .map(|v| (v, graph.edges(v).count() as f64 / (n - 1.0)))
        .collect()
}
```

**petgraph:** ✅ Available via `graph.edges(v).count()`

---

#### 1.2 In-Degree Centrality (Fan-In)
```
Definition: Number of incoming edges
Formula:    C_in(v) = indeg(v) / (n-1)
Complexity: O(V)
```

**Code Application:**
- High fan-in = Library/utility function
- Critical: changes affect many callers
- Examples: `println!`, `Vec::new`, `clone()`

**Blast Radius Indicator:**
```
Fan-in > 10: Changes affect 10+ functions
Fan-in > 50: High-risk change
Fan-in > 100: Critical infrastructure
```

---

#### 1.3 Out-Degree Centrality (Fan-Out)
```
Definition: Number of outgoing edges
Formula:    C_out(v) = outdeg(v) / (n-1)
Complexity: O(V)
```

**Code Application:**
- High fan-out = Orchestrator/integrator function
- Complex: depends on many things
- Examples: `main()`, request handlers, controllers

**Complexity Indicator:**
```
Fan-out > 10: Complex function
Fan-out > 25: God function candidate
Fan-out > 50: Likely needs refactoring
```

---

### 2. EIGENVECTOR-BASED CENTRALITY

#### 2.1 Eigenvector Centrality
```
Definition: Importance based on importance of neighbors
Formula:    C_E(v) = (1/λ) Σ C_E(u) for all neighbors u
Complexity: O(V²) naive, O(kE) iterative
```

**Code Application:**
- A function is important if called by important functions
- Recursive importance propagation
- Identifies "core of the core"

**Interpretation:**
```
High eigenvector centrality:
- Called by important functions
- Part of the "backbone" of the codebase
- NOT just popular, but strategically positioned
```

**Implementation:**
```rust
fn eigenvector_centrality(graph: &Graph<Node, Edge>, iterations: usize) -> HashMap<NodeIndex, f64> {
    let n = graph.node_count();
    let mut centrality: HashMap<NodeIndex, f64> = graph.node_indices()
        .map(|v| (v, 1.0 / n as f64))
        .collect();

    for _ in 0..iterations {
        let mut new_centrality = HashMap::new();
        let mut sum_sq = 0.0;

        for v in graph.node_indices() {
            let score: f64 = graph.edges_directed(v, Direction::Incoming)
                .filter_map(|e| centrality.get(&e.source()))
                .sum();
            new_centrality.insert(v, score);
            sum_sq += score * score;
        }

        // Normalize
        let norm = sum_sq.sqrt();
        for (_, score) in &mut new_centrality {
            *score /= norm;
        }
        centrality = new_centrality;
    }

    centrality
}
```

**petgraph:** ❌ Not built-in, implement manually

---

#### 2.2 PageRank
```
Definition: Eigenvector centrality with damping factor
Formula:    PR(v) = (1-d)/n + d × Σ(PR(u)/out_deg(u))
Complexity: O(kE) where k = iterations
```

**Code Application:**
- Damping factor (d=0.85) handles disconnected components
- More stable than eigenvector for code graphs
- Google's original algorithm - battle-tested

**Parameters for Code:**
```
d = 0.85 (standard)
iterations = 20-50 (usually converges)
tolerance = 1e-6 (stopping criterion)
```

**Interpretation:**
```
PageRank > 0.01: Important function
PageRank > 0.05: Core infrastructure
PageRank > 0.10: Entry point / critical path
```

**Implementation:**
```rust
fn pagerank(
    graph: &DiGraph<Node, Edge>,
    damping: f64,
    iterations: usize,
) -> HashMap<NodeIndex, f64> {
    let n = graph.node_count() as f64;
    let mut pr: HashMap<NodeIndex, f64> = graph.node_indices()
        .map(|v| (v, 1.0 / n))
        .collect();

    for _ in 0..iterations {
        let mut new_pr = HashMap::new();

        for v in graph.node_indices() {
            let rank: f64 = graph.edges_directed(v, Direction::Incoming)
                .map(|e| {
                    let u = e.source();
                    let out_deg = graph.edges_directed(u, Direction::Outgoing).count();
                    pr[&u] / out_deg.max(1) as f64
                })
                .sum();

            new_pr.insert(v, (1.0 - damping) / n + damping * rank);
        }

        pr = new_pr;
    }

    pr
}
```

**petgraph:** ❌ Not built-in, but petgraph has algo module

---

#### 2.3 Katz Centrality
```
Definition: Eigenvector with attenuation for distance
Formula:    C_K(v) = α Σ C_K(u) + β
Complexity: O(V³) matrix, O(kE) iterative
```

**Code Application:**
- α (alpha) controls attenuation per hop
- β (beta) gives baseline importance to all nodes
- Better for directed graphs with sinks

**Parameters:**
```
α < 1/λ_max (largest eigenvalue)
β = 1.0 (baseline)
```

**petgraph:** ❌ Not built-in

---

#### 2.4 HITS (Hyperlink-Induced Topic Search)
```
Definition: Separates hubs (point out) from authorities (pointed to)
Formula:    Authority: a(v) = Σ h(u)
            Hub: h(v) = Σ a(u)
Complexity: O(kE)
```

**Code Application:**
- **Authorities** = Functions many call (libraries, utilities)
- **Hubs** = Functions that call many (orchestrators, controllers)

**Interpretation:**
```
High Authority: Use as dependency
High Hub: Use as entry point
Both High: Architecture connector
```

**petgraph:** ❌ Not built-in

---

### 3. SHORTEST PATH-BASED CENTRALITY

#### 3.1 Betweenness Centrality
```
Definition: Fraction of shortest paths passing through node
Formula:    C_B(v) = Σ σ_st(v) / σ st
Complexity: O(VE) Brandes algorithm
```

**Code Application:**
- Identifies "bridge" functions
- Critical for information flow
- Removing high-betweenness nodes fragments graph

**Interpretation:**
```
High betweenness:
- This function connects otherwise separate parts
- Bottleneck for cross-module calls
- Target for decoupling/refactoring
```

**Code Pattern Detection:**
```
Betweenness > 0.1: Adapter/facade pattern
Betweenness > 0.2: God object antipattern
Betweenness > 0.3: Architecture smell
```

**Implementation (Brandes):**
```rust
fn betweenness_centrality(graph: &DiGraph<Node, Edge>) -> HashMap<NodeIndex, f64> {
    let mut bc: HashMap<NodeIndex, f64> = graph.node_indices()
        .map(|v| (v, 0.0))
        .collect();

    for s in graph.node_indices() {
        // BFS from s
        let mut stack = Vec::new();
        let mut predecessors: HashMap<NodeIndex, Vec<NodeIndex>> = HashMap::new();
        let mut sigma: HashMap<NodeIndex, f64> = HashMap::new();
        let mut dist: HashMap<NodeIndex, i64> = HashMap::new();

        // Initialize
        for v in graph.node_indices() {
            predecessors.insert(v, Vec::new());
            sigma.insert(v, 0.0);
            dist.insert(v, -1);
        }
        *sigma.get_mut(&s).unwrap() = 1.0;
        *dist.get_mut(&s).unwrap() = 0;

        let mut queue = VecDeque::new();
        queue.push_back(s);

        while let Some(v) = queue.pop_front() {
            stack.push(v);
            for w in graph.neighbors_directed(v, Direction::Outgoing) {
                if dist[&w] < 0 {
                    queue.push_back(w);
                    *dist.get_mut(&w).unwrap() = dist[&v] + 1;
                }
                if dist[&w] == dist[&v] + 1 {
                    *sigma.get_mut(&w).unwrap() += sigma[&v];
                    predecessors.get_mut(&w).unwrap().push(v);
                }
            }
        }

        // Accumulation
        let mut delta: HashMap<NodeIndex, f64> = graph.node_indices()
            .map(|v| (v, 0.0))
            .collect();

        while let Some(w) = stack.pop() {
            for v in &predecessors[&w] {
                let sigma_w = sigma[&w];
                let sigma_v = sigma[v];
                let delta_w = delta[&w];
                *delta.get_mut(v).unwrap() += (sigma_v / sigma_w) * (1.0 + delta_w);
            }
            if w != s {
                *bc.get_mut(&w).unwrap() += delta[&w];
            }
        }
    }

    // Normalize
    let n = graph.node_count() as f64;
    let norm = (n - 1.0) * (n - 2.0);
    for (_, score) in &mut bc {
        *score /= norm;
    }

    bc
}
```

**petgraph:** ✅ Available in `petgraph::algo::betweenness_centrality` (unofficial)

---

#### 3.2 Closeness Centrality
```
Definition: Inverse of average distance to all other nodes
Formula:    C_C(v) = (n-1) / Σ d(v,u)
Complexity: O(V(V+E)) for all nodes
```

**Code Application:**
- How "close" is this function to all others?
- Identifies centrally positioned code
- Good for finding "middleware" functions

**Interpretation:**
```
High closeness:
- Can quickly reach/impact other functions
- Good location for logging/monitoring
- Minimal indirection to rest of codebase
```

**Implementation:**
```rust
fn closeness_centrality(graph: &DiGraph<Node, Edge>) -> HashMap<NodeIndex, f64> {
    let n = graph.node_count() as f64;
    let mut cc = HashMap::new();

    for v in graph.node_indices() {
        let distances = dijkstra(graph, v, None, |_| 1.0);
        let sum: f64 = distances.values().map(|&d| d as f64).sum();
        if sum > 0.0 {
            cc.insert(v, (n - 1.0) / sum);
        } else {
            cc.insert(v, 0.0);
        }
    }

    cc
}
```

**petgraph:** ✅ Can use `petgraph::algo::dijkstra` as base

---

#### 3.3 Harmonic Centrality
```
Definition: Sum of inverse distances (handles disconnected)
Formula:    C_H(v) = Σ 1/d(v,u) for u ≠ v
Complexity: O(V(V+E))
```

**Code Application:**
- Handles disconnected components (common in code)
- More robust than closeness for real codebases
- Better for modular architectures

**petgraph:** ❌ Not built-in

---

### 4. SUBGRAPH-BASED CENTRALITY

#### 4.1 Subgraph Centrality
```
Definition: Participation in subgraphs (cycles, cliques)
Formula:    C_S(v) = Σ (A^k)_vv / k!
Complexity: O(V³) matrix computation
```

**Code Application:**
- Identifies nodes in tightly coupled groups
- High value = part of many cycles
- Detects circular dependencies

**petgraph:** ❌ Not built-in, requires matrix ops

---

#### 4.2 k-Core Centrality
```
Definition: Maximum k where node belongs to k-core
Formula:    k-core = maximal subgraph where all nodes have degree ≥ k
Complexity: O(E)
```

**Code Application:**
- k-core decomposition layers the graph
- Higher k = more "core" to the system
- Identifies complexity hotspots

**k-Core for Code:**
```
k=1: Leaf functions (easy to change)
k=2: Bridge functions (connect leaves to core)
k=3: Core functions (tightly interconnected)
k≥5: Complexity hotspot (refactoring target)
```

**Implementation:**
```rust
fn k_core_decomposition(graph: &mut DiGraph<Node, Edge>) -> HashMap<NodeIndex, usize> {
    let mut k_core: HashMap<NodeIndex, usize> = HashMap::new();
    let mut degrees: HashMap<NodeIndex, usize> = graph.node_indices()
        .map(|v| (v, graph.edges(v).count()))
        .collect();

    let mut k = 1;
    while graph.node_count() > 0 {
        // Remove nodes with degree < k
        let to_remove: Vec<NodeIndex> = graph.node_indices()
            .filter(|&v| degrees[&v] < k)
            .collect();

        if to_remove.is_empty() {
            // All remaining nodes are in k-core
            for v in graph.node_indices() {
                k_core.insert(v, k);
            }
            k += 1;
        } else {
            for v in to_remove {
                k_core.insert(v, k - 1);
                // Update neighbor degrees
                for neighbor in graph.neighbors(v) {
                    *degrees.get_mut(&neighbor).unwrap() -= 1;
                }
                graph.remove_node(v);
            }
        }
    }

    k_core
}
```

**petgraph:** ❌ Not built-in

---

### 5. FLOW-BASED CENTRALITY

#### 5.1 Current-Flow Centrality
```
Definition: Based on electrical current flow in network
Formula:    Uses resistance distance
Complexity: O(V³)
```

**Code Application:**
- Models information "flow" through code
- Better for undirected dependency graphs
- Captures alternative paths

**petgraph:** ❌ Not built-in

---

#### 5.2 Communicability Centrality
```
Definition: Weighted sum of all walks from node
Formula:    C_comm(v) = Σ (A^k)_vv / k!
Complexity: O(V³) matrix exponential
```

**Code Application:**
- Captures all paths, not just shortest
- Higher weight for shorter paths
- Good for "overall connectedness"

**petgraph:** ❌ Not built-in

---

### 6. TEMPORAL CENTRALITY

#### 6.1 Temporal Betweenness
```
Definition: Betweenness respecting time ordering
Complexity: O(V² × T) where T = timesteps
```

**Code Application:**
- Evolution of code importance over time
- Identify "rising star" functions
- Track architectural drift

---

#### 6.2 Snapshot Centrality
```
Definition: Centrality at each time snapshot
Complexity: O(T × base_centrality)
```

**Code Application:**
- Compare centrality across git commits
- Identify stable vs volatile importance
- Track architectural changes

---

## Comparative Analysis

| Algorithm | Complexity | Code Best For | petgraph |
|-----------|------------|---------------|----------|
| Degree | O(V+E) | Fan-in/fan-out | ✅ |
| In-Degree | O(V) | Dependency count | ✅ |
| Out-Degree | O(V) | Complexity count | ✅ |
| Eigenvector | O(kE) | Strategic importance | ❌ |
| PageRank | O(kE) | Overall importance | ❌ |
| Katz | O(kE) | Directed graphs | ❌ |
| HITS | O(kE) | Hubs vs authorities | ❌ |
| Betweenness | O(VE) | Bridges/bottlenecks | Partial |
| Closeness | O(V(V+E)) | Central position | Via dijkstra |
| Harmonic | O(V(V+E)) | Disconnected graphs | ❌ |
| Subgraph | O(V³) | Cycle participation | ❌ |
| k-Core | O(E) | Complexity layers | ❌ |
| Current-Flow | O(V³) | Alternative paths | ❌ |
| Communicability | O(V³) | All-path connectedness | ❌ |

---

## Implementation Priority for Parseltongue

### Phase 1 (BUILD NOW)
1. ✅ Degree centrality (trivial)
2. ✅ PageRank (well-documented, high value)
3. ✅ Betweenness (critical for bridges)
4. ✅ k-Core (complexity hotspots)

### Phase 2 (BUILD NEXT)
5. Harmonic closeness (handles disconnection)
6. HITS (hub/authority separation)
7. Eigenvector (strategic positioning)

### Phase 3 (DEFER)
8. Current-flow (complex, niche)
9. Communicability (matrix ops heavy)
10. Temporal variants (requires history data)

---

## Rust Implementation Notes

### Data Structures
```rust
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, VecDeque};

pub struct CentralityScores {
    pub degree: HashMap<NodeIndex, f64>,
    pub pagerank: HashMap<NodeIndex, f64>,
    pub betweenness: HashMap<NodeIndex, f64>,
    pub k_core: HashMap<NodeIndex, usize>,
}
```

### Performance Tips
1. Use `petgraph::csr::Csr` for large graphs (better cache locality)
2. Parallelize PageRank with Rayon
3. Cache shortest paths for betweenness
4. Use approximate betweenness for graphs >10K nodes

---

## References

1. Brandes, U. (2001). "A Faster Algorithm for Betweenness Centrality"
2. Page, L. et al. (1999). "The PageRank Citation Ranking"
3. Kitsak, M. et al. (2010). "Identification of Influential Spreaders"
4. Batagelj, V. & Zaversnik, M. (2003). "An O(m) Algorithm for Cores Decomposition"
