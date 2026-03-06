# Community Detection Algorithms for Code Graphs

**Category:** Graph Clustering / Module Detection
**Target:** 30+ algorithms documented
**Purpose:** Identify cohesive modules, architectural boundaries, and refactoring targets

---

## Overview

Community detection answers: **"Which nodes belong together?"**

In code graphs, this translates to:
- Which functions form a cohesive module?
- What are the natural architectural boundaries?
- Where should we split monolithic code?
- Which code is tightly coupled?

---

## Foundational Concepts

### Modularity
```
Definition: Quality measure for community partitions
Formula:    Q = (1/2m) Σ [A_ij - k_i k_j / 2m] δ(c_i, c_j)

Where:
- A_ij = adjacency matrix (1 if edge, 0 otherwise)
- k_i = degree of node i
- m = total edges
- δ = 1 if same community, 0 otherwise

Range: [-0.5, 1.0]
- Q > 0.3: Significant community structure
- Q > 0.5: Strong community structure
- Q > 0.7: Very strong (rare in real graphs)
```

### Resolution Limit
```
Problem: Modularity maximization may miss small communities
Solution: Resolution parameter γ
Q_γ = (1/2m) Σ [A_ij - γ k_i k_j / 2m] δ(c_i, c_j)
- γ > 1: Detects smaller communities
- γ < 1: Detects larger communities
```

---

## Algorithm Categories

### 1. MODULARITY OPTIMIZATION

#### 1.1 Louvain Algorithm
```
Inventor: Blondel et al. (2008)
Approach: Greedy multi-level modularity optimization
Complexity: O(n log n)
```

**Phases:**
1. **Local Moving**: Each node joins neighbor's community if modularity increases
2. **Aggregation**: Communities become nodes, repeat
3. **Iterate** until convergence

**Code Application:**
```
Level 0: Function-level communities
Level 1: Module-level communities
Level 2: Package-level communities
```

**Pros:**
- Fast, scales to millions of nodes
- Hierarchical output (multi-scale)
- Widely implemented

**Cons:**
- Resolution limit (misses small communities)
- Random order sensitivity
- May produce degenerate communities

**Implementation:**
```rust
struct Louvain {
    graph: Graph,
    communities: Vec<usize>,
}

impl Louvain {
    fn phase_one(&mut self) -> bool {
        let mut improved = false;
        for node in self.graph.nodes() {
            let best_community = self.find_best_community(node);
            if best_community != self.communities[node] {
                self.communities[node] = best_community;
                improved = true;
            }
        }
        improved
    }

    fn find_best_community(&self, node: NodeIndex) -> usize {
        let neighbors = self.graph.neighbors(node);
        let current = self.communities[node];
        let mut best_gain = 0.0;
        let mut best_community = current;

        for neighbor in neighbors {
            let community = self.communities[neighbor];
            let gain = self.modularity_gain(node, community);
            if gain > best_gain {
                best_gain = gain;
                best_community = community;
            }
        }
        best_community
    }
}
```

---

#### 1.2 Leiden Algorithm (RECOMMENDED)
```
Inventor: Traag, Waltman, van Eck (2019)
Approach: Louvain with refinement phase
Complexity: O(n log n)
```

**Improvements over Louvain:**
1. **Refinement Phase**: Splits poorly connected communities
2. **Smart Local Moving**: Considers multiple moves simultaneously
3. **Guarantees**: Well-connected communities

**Why Leiden > Louvain:**
```
Louvain may produce:
- Disconnected communities
- Badly connected communities
- Random variation between runs

Leiden guarantees:
- All communities are connected
- Communities are locally optimal
- Reproducible results
```

**Implementation Priority:** HIGH - This is our primary community detection algorithm

**Code Application:**
```rust
// Leiden for module detection
fn leiden_modules(graph: &CodeGraph) -> Vec<Module> {
    let mut leiden = Leiden::new(graph);

    // Phase 1: Local moving
    while leiden.local_moving() {}

    // Phase 2: Refinement
    leiden.refine();

    // Phase 3: Aggregation
    let partition = leiden.aggregate();

    partition.to_modules()
}
```

**Rust Implementations:**
- ❌ No mature Rust Leiden implementation
- ✅ Build from paper (clear algorithm)
- ✅ Reference: Python `leidenalg` package

---

#### 1.3 Fast Greedy (Clauset-Newman-Moore)
```
Inventor: Clauset, Newman, Moore (2004)
Approach: Agglomerative hierarchical clustering
Complexity: O(md log n) where d = dendrogram depth
```

**Process:**
1. Start with each node as own community
2. Greedily merge pairs with highest modularity gain
3. Build dendrogram
4. Cut at optimal level

**Code Application:**
- Good for: Small to medium codebases (<100K nodes)
- Produces: Hierarchical module structure

---

### 2. LABEL PROPAGATION

#### 2.1 Label Propagation Algorithm (LPA)
```
Inventor: Raghavan, Albert, Kumara (2007)
Approach: Nodes adopt majority label of neighbors
Complexity: O(m) per iteration
```

**Process:**
1. Each node gets unique label
2. Iterate: adopt most common neighbor label
3. Converge when stable

**Pros:**
- Extremely fast
- Near-linear time
- Simple implementation

**Cons:**
- Non-deterministic (run multiple times)
- May produce single large community
- Sensitive to initialization

**Code Application:**
```rust
fn label_propagation(graph: &Graph) -> Vec<usize> {
    let mut labels: Vec<usize> = (0..graph.node_count()).collect();

    loop {
        let mut changed = false;
        for v in graph.node_indices() {
            let neighbor_labels: Vec<usize> = graph.neighbors(v)
                .map(|n| labels[n.index()])
                .collect();

            let new_label = most_common(&neighbor_labels);
            if labels[v.index()] != new_label {
                labels[v.index()] = new_label;
                changed = true;
            }
        }
        if !changed { break; }
    }

    labels
}
```

---

#### 2.2 SLPA (Speaker-Listener Label Propagation)
```
Inventor: Xie, Szymanski (2013)
Approach: LPA with memory
Complexity: O(t × m) where t = iterations
```

**Improvement over LPA:**
- Each node maintains label memory
- Probabilistic label selection
- Better for overlapping communities

---

### 3. RANDOM WALK BASED

#### 3.1 Walktrap
```
Inventor: Pons & Latapy (2005)
Approach: Distance based on random walk probabilities
Complexity: O(n² log n)
```

**Idea:**
- Random walks tend to stay in communities
- Distance between nodes = difference in walk distributions
- Hierarchical clustering on distance matrix

**Code Application:**
- Good for: Dense code graphs
- Captures: Indirect relationships

---

#### 3.2 Infomap
```
Inventor: Rosvall & Bergstrom (2008)
Approach: Minimize description length of random walk
Complexity: O(m) per iteration
```

**Idea:**
- Encode random walk trajectory
- Good partition = efficient encoding
- Uses Huffman coding principles

**Pros:**
- Information-theoretic foundation
- No resolution limit
- Handles hierarchical structure

**Cons:**
- Non-deterministic
- Multiple runs needed

**Code Application:**
```
Infomap reveals:
- Module boundaries (flow-based)
- Information bottlenecks
- Optimal abstraction levels
```

---

### 4. SPECTRAL METHODS

#### 4.1 Spectral Clustering
```
Approach: Cluster nodes using graph Laplacian eigenvectors
Complexity: O(n³) for eigendecomposition, O(n²) for k-means
```

**Process:**
1. Compute normalized Laplacian: L = I - D^(-1/2) A D^(-1/2)
2. Find k smallest eigenvectors
3. Cluster rows of eigenvector matrix (k-means)

**Code Application:**
- Precise but expensive
- Good for: Small graphs (<10K nodes)
- Use when: Quality > Speed

---

### 5. STATISTICAL METHODS

#### 5.1 Stochastic Block Model (SBM)
```
Approach: Generative model for communities
Complexity: O(n²) inference
```

**Model:**
```
P(edge between i,j) = θ_c_i * θ_c_j * B_c_i,c_j

Where:
- c_i = community of node i
- θ_i = node propensity
- B_ab = community interaction matrix
```

**Variants:**
- **Degree-corrected SBM**: Accounts for degree heterogeneity
- **Hierarchical SBM**: Nested communities
- **Dynamic SBM**: Communities over time

---

### 6. NEURAL METHODS

#### 6.1 Graph Neural Network Community Detection
```
Approach: Learn community assignments end-to-end
Complexity: Depends on architecture
```

**Models:**
- GNN + Differentiable clustering
- Deep community detection
- Community-aware node embeddings

---

## Comprehensive Algorithm Comparison

| Algorithm | Complexity | Deterministic | Hierarchy | Overlapping | Code Best For |
|-----------|------------|---------------|-----------|-------------|---------------|
| Louvain | O(n log n) | ❌ | ✅ | ❌ | Large codebases |
| **Leiden** | O(n log n) | ⚠️ | ✅ | ❌ | **RECOMMENDED** |
| Fast Greedy | O(md log n) | ✅ | ✅ | ❌ | Medium codebases |
| LPA | O(m) | ❌ | ❌ | ❌ | Fast/rough |
| SLPA | O(tm) | ❌ | ❌ | ✅ | Overlapping modules |
| Walktrap | O(n² log n) | ✅ | ✅ | ❌ | Dense graphs |
| Infomap | O(m) | ❌ | ✅ | ❌ | Flow analysis |
| Spectral | O(n³) | ✅ | ❌ | ❌ | High quality |
| SBM | O(n²) | ⚠️ | ✅ | ✅ | Statistical rigor |

---

## Code-Specific Adaptations

### Function-Level Communities
```
Nodes: Functions
Edges: Calls (weight = call frequency)
Algorithm: Leiden with γ=1.0
Output: Function groups (potential modules)
```

### Module-Level Communities
```
Nodes: Modules/files
Edges: Imports (weight = import count)
Algorithm: Leiden with γ=0.8
Output: Package groups (potential crates)
```

### Semantic Communities
```
Nodes: Any code entity
Edges: Semantic similarity (from embeddings)
Algorithm: Spectral clustering
Output: Conceptually related code
```

---

## Implementation Priority for Parseltongue

### Phase 1 (BUILD NOW)
1. ✅ **Leiden Algorithm** - Primary community detection
2. ✅ Louvain (as Leiden fallback/baseline)
3. ✅ Label Propagation (fast approximation)

### Phase 2 (BUILD NEXT)
4. Infomap (flow-based analysis)
5. Spectral clustering (high-quality small graphs)
6. SLPA (overlapping communities)

### Phase 3 (DEFER)
7. Stochastic Block Models (research)
8. GNN-based detection (experimental)

---

## Leiden Implementation (Detailed)

```rust
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, HashSet};

pub struct Leiden {
    graph: DiGraph<Node, Edge>,
    partition: Vec<usize>,
    resolution: f64,
}

impl Leiden {
    pub fn new(graph: DiGraph<Node, Edge>) -> Self {
        let n = graph.node_count();
        Self {
            graph,
            partition: (0..n).collect(),
            resolution: 1.0,
        }
    }

    /// Phase 1: Local moving of nodes
    fn local_moving(&mut self) -> bool {
        let mut improved = false;
        let nodes: Vec<NodeIndex> = self.graph.node_indices().collect();

        for v in nodes {
            let best_community = self.find_best_community(v);
            if best_community != self.partition[v.index()] {
                self.partition[v.index()] = best_community;
                improved = true;
            }
        }
        improved
    }

    /// Find community that maximizes modularity gain
    fn find_best_community(&self, v: NodeIndex) -> usize {
        let neighbors: Vec<NodeIndex> = self.graph.neighbors(v).collect();
        let neighbor_communities: HashSet<usize> = neighbors.iter()
            .map(|n| self.partition[n.index()])
            .collect();

        let current = self.partition[v.index()];
        let mut best_community = current;
        let mut best_gain = 0.0;

        for c in neighbor_communities {
            let gain = self.modularity_gain(v, c);
            if gain > best_gain {
                best_gain = gain;
                best_community = c;
            }
        }
        best_community
    }

    /// Calculate modularity gain from moving node to community
    fn modularity_gain(&self, v: NodeIndex, target_community: usize) -> f64 {
        let m = self.graph.edge_count() as f64;
        let k_v = self.graph.edges(v).count() as f64;

        // Sum of edges to target community
        let sigma_tot: f64 = self.graph.edges(v)
            .filter(|e| {
                let neighbor = e.target();
                self.partition[neighbor.index()] == target_community
            })
            .count() as f64;

        // Sum of degrees in target community
        let k_c: f64 = self.graph.node_indices()
            .filter(|n| self.partition[n.index()] == target_community)
            .map(|n| self.graph.edges(n).count() as f64)
            .sum();

        let resolution = self.resolution;
        (sigma_tot / m) - (resolution * k_v * k_c / (2.0 * m * m))
    }

    /// Phase 2: Refinement - ensure communities are well-connected
    fn refinement(&mut self) {
        for community in self.unique_communities() {
            let nodes: Vec<NodeIndex> = self.graph.node_indices()
                .filter(|n| self.partition[n.index()] == community)
                .collect();

            // Check if community is well-connected
            if !self.is_well_connected(&nodes) {
                // Split into smaller, well-connected sub-communities
                self.split_community(&nodes);
            }
        }
    }

    fn is_well_connected(&self, nodes: &[NodeIndex]) -> bool {
        if nodes.len() <= 2 {
            return true;
        }

        // Check if each node has enough internal connections
        let threshold = (nodes.len() as f64).sqrt() as usize;
        for &v in nodes {
            let internal: usize = self.graph.neighbors(v)
                .filter(|n| nodes.contains(n))
                .count();
            if internal < threshold {
                return false;
            }
        }
        true
    }

    fn split_community(&mut self, nodes: &[NodeIndex]) {
        // Use label propagation on subgraph
        let mut labels: HashMap<NodeIndex, usize> = nodes.iter()
            .enumerate()
            .map(|(i, &n)| (n, i))
            .collect();

        for _ in 0..10 {
            for &v in nodes {
                let neighbor_labels: Vec<usize> = self.graph.neighbors(v)
                    .filter(|n| nodes.contains(n))
                    .map(|n| labels[&n])
                    .collect();

                if !neighbor_labels.is_empty() {
                    let most_common = self.most_common(&neighbor_labels);
                    labels.insert(v, most_common);
                }
            }
        }

        // Update partition
        for (&v, &label) in &labels {
            self.partition[v.index()] = label;
        }
    }

    /// Phase 3: Aggregation
    fn aggregate(&mut self) {
        // Create new graph where communities become nodes
        let communities = self.unique_communities();
        let mut new_graph = DiGraph::new();

        // Map community to new node index
        let community_nodes: HashMap<usize, NodeIndex> = communities.iter()
            .map(|&c| (c, new_graph.add_node(Node::Community(c))))
            .collect();

        // Add edges between communities
        for edge in self.graph.edge_references() {
            let source_comm = self.partition[edge.source().index()];
            let target_comm = self.partition[edge.target().index()];

            if source_comm != target_comm {
                let source = community_nodes[&source_comm];
                let target = community_nodes[&target_comm];
                new_graph.add_edge(source, target, Edge::Aggregated);
            }
        }

        self.graph = new_graph;
        self.partition = (0..new_graph.node_count()).collect();
    }

    pub fn run(&mut self) -> Vec<usize> {
        // Iterative Leiden
        loop {
            // Phase 1
            while self.local_moving() {}

            // Check convergence
            let current_modularity = self.modularity();

            // Phase 2
            self.refinement();

            // Phase 3
            self.aggregate();

            let new_modularity = self.modularity();
            if new_modularity <= current_modularity {
                break;
            }
        }

        self.partition.clone()
    }

    fn modularity(&self) -> f64 {
        let m = self.graph.edge_count() as f64;
        let mut q = 0.0;

        for e in self.graph.edge_references() {
            let i = e.source();
            let j = e.target();
            let k_i = self.graph.edges(i).count() as f64;
            let k_j = self.graph.edges(j).count() as f64;

            if self.partition[i.index()] == self.partition[j.index()] {
                q += 1.0 - (self.resolution * k_i * k_j / (2.0 * m));
            }
        }

        q / (2.0 * m)
    }

    fn unique_communities(&self) -> Vec<usize> {
        let mut communities: HashSet<usize> = HashSet::new();
        for &c in &self.partition {
            communities.insert(c);
        }
        communities.into_iter().collect()
    }

    fn most_common(&self, labels: &[usize]) -> usize {
        let mut counts: HashMap<usize, usize> = HashMap::new();
        for &label in labels {
            *counts.entry(label).or_insert(0) += 1;
        }
        counts.into_iter().max_by_key(|(_, c)| *c).map(|(l, _)| l).unwrap_or(0)
    }
}
```

---

## References

1. Traag, V.A., Waltman, L., van Eck, N.J. (2019). "From Louvain to Leiden"
2. Blondel, V.D. et al. (2008). "Fast Unfolding of Communities"
3. Rosvall, M., Bergstrom, C.T. (2008). "Maps of Random Walks"
4. Raghavan, U.N. et al. (2007). "Near Linear Time Algorithm for Community Detection"
