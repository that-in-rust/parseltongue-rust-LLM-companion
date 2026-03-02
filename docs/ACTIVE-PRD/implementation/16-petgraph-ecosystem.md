# petgraph Ecosystem Analysis

**Purpose:** Complete analysis of petgraph and Rust graph ecosystem
**Target:** Comprehensive API mapping, capabilities, gaps

---

## 1. PETGRAPH OVERVIEW

```
Repository: https://github.com/petgraph/petgraph
Stars: 3,773 (as of 2026-03)
License: Apache-2.0 / MIT
Language: Rust
Status: Actively maintained
```

### Why petgraph?
- De facto standard for Rust graphs
- Pure Rust, no C dependencies
- Good performance
- Active community

---

## 2. DATA STRUCTURES

### 2.1 Graph<N, E> (Directed)
```rust
use petgraph::graph::{Graph, NodeIndex, EdgeIndex};

let mut graph: Graph<&str, i32> = Graph::new();
let a = graph.add_node("A");
let b = graph.add_node("B");
graph.add_edge(a, b, 42);
```

**Characteristics:**
- Directed by default
- Nodes and edges have indices
- O(1) node/edge access
- Self-loops allowed
- Parallel edges allowed

**Memory:** ~24 bytes per node, ~24 bytes per edge

### 2.2 UnGraph<N, E> (Undirected)
```rust
use petgraph::graph::UnGraph;

let mut graph: UnGraph<&str, ()> = UnGraph::new_undirected();
```

### 2.3 GraphMap<N, E> (Hash-based)
```rust
use petgraph::graphmap::GraphMap;

let mut graph: GraphMap<&str, i32> = GraphMap::new();
graph.add_edge("A", "B", 42);
```

**Characteristics:**
- Nodes are hashable keys
- No parallel edges
- O(1) edge lookup
- Good for sparse graphs

### 2.4 MatrixGraph
```rust
use petgraph::matrix_graph::MatrixGraph;

let mut graph: MatrixGraph<(), ()> = MatrixGraph::new();
```

**Characteristics:**
- Adjacency matrix representation
- O(1) edge lookup
- Memory: O(V²)
- Good for dense graphs

### 2.5 CSR (Compressed Sparse Row)
```rust
use petgraph::csr::Csr;

let graph: Csr<(), ()> = Csr::new();
```

**Characteristics:**
- Most memory efficient
- Read-only after creation
- Best for large static graphs
- Memory: O(V + E)

---

## 3. ALGORITHMS IMPLEMENTED

### 3.1 Traversal
| Algorithm | Function | Complexity |
|-----------|----------|------------|
| BFS | `bfs_search()` | O(V + E) |
| DFS | `dfs_search()` | O(V + E) |
| DFS from multiple roots | `dfs()` | O(V + E) |

```rust
use petgraph::algo::{bfs_search, dfs_search};

// BFS
let mut bfs = bfs_search(&graph, start);
while let Some(node) = bfs.next(&graph) {
    println!("Visited: {:?}", node);
}

// DFS
let mut dfs = dfs_search(&graph, start);
while let Some(node) = dfs.next(&graph) {
    println!("Visited: {:?}", node);
}
```

### 3.2 Shortest Path
| Algorithm | Function | Weights | Complexity |
|-----------|----------|---------|------------|
| Dijkstra | `dijkstra()` | Non-negative | O((V+E) log V) |
| Bellman-Ford | `bellman_ford()` | Any | O(V × E) |
| A* | `astar()` | Non-negative | O(E) best case |
| Floyd-Warshall | Not implemented | - | - |
| Johnson | Not implemented | - | - |

```rust
use petgraph::algo::{dijkstra, astar, bellman_ford};

// Dijkstra
let distances = dijkstra(&graph, start, Some(goal), |e| *e.weight());

// A*
let path = astar(&graph, start, |n| n == goal, |e| *e.weight(), |n| heuristic(n));

// Bellman-Ford (handles negative weights)
let result = bellman_ford(&graph, start, |e| *e.weight());
```

### 3.3 Connectivity
| Algorithm | Function | Purpose |
|-----------|----------|---------|
| Connected Components | `connected_components()` | Count components |
| Strongly Connected | `kosaraju_scc()` | SCC decomposition |
| Tarjan SCC | Not separate | Use kosaraju_scc |
| Is Cyclic | `is_cyclic_directed()` | Cycle detection |
| Toposort | `toposort()` | Topological sort |

```rust
use petgraph::algo::{connected_components, kosaraju_scc, is_cyclic_directed, toposort};

let num_components = connected_components(&graph);
let sccs = kosaraju_scc(&graph);
let has_cycle = is_cyclic_directed(&graph);
let sorted = toposort(&graph, None);
```

### 3.4 Minimum Spanning Tree
| Algorithm | Function | Graph Type |
|-----------|----------|------------|
| Prim | `min_spanning_tree()` | Undirected |
| Kruskal | `min_spanning_tree()` | Undirected |

```rust
use petgraph::algo::min_spanning_tree;

let mst = min_spanning_tree(&graph);
```

### 3.5 Matching
| Algorithm | Function | Purpose |
|-----------|----------|---------|
| Maximum Matching | Not implemented | - |

### 3.6 Flow
| Algorithm | Function | Purpose |
|-----------|----------|---------|
| Max Flow | Not in core | External needed |

---

## 4. WHAT PETGRAPH LACKS

### Centrality Algorithms
```
❌ PageRank
❌ Betweenness centrality
❌ Closeness centrality
❌ Eigenvector centrality
❌ k-core decomposition
```

### Community Detection
```
❌ Louvain
❌ Leiden
❌ Label propagation
❌ Infomap
❌ Modularity calculation
```

### Graph Matching
```
❌ Subgraph isomorphism (VF2)
❌ Graph edit distance
❌ Maximum common subgraph
```

### Subgraph Mining
```
❌ Frequent subgraph mining
❌ Motif detection
❌ Graphlet counting
```

### Temporal Analysis
```
❌ Dynamic graph support
❌ Temporal algorithms
```

---

## 5. ECOSYSTEM LIBRARIES

### 5.1 petgraph-algo
```rust
// Unofficial extension
// Adds some algorithms missing from petgraph
```

### 5.2 graphalgs
```
Repository: https://github.com/starovoid/graphalgs
Stars: 30
Status: Less maintained

Adds:
- Some centrality measures
- Additional path algorithms
```

### 5.3 rustwork
```
Python networkx-like API for Rust
Status: Early development
```

### 5.4 daggy
```
Repository: https://github.com/mitchmindtree/daggy
Purpose: Directed acyclic graphs
Uses petgraph internally
```

### 5.5 ena
```
Repository: https://github.com/rust-lang/ena
Purpose: Union-find (disjoint sets)
Used in rustc
```

---

## 6. IMPLEMENTATION GAPS TO FILL

### Priority 1: Essential for Parseltongue

| Algorithm | petgraph | Need to Build | Complexity |
|-----------|----------|---------------|------------|
| PageRank | ❌ | ✅ Build | Medium |
| Betweenness | ❌ | ✅ Build | Medium |
| Leiden | ❌ | ✅ Build | Medium |
| k-Core | ❌ | ✅ Build | Low |
| Harmonic closeness | ❌ | ✅ Build | Low |

### Priority 2: Important

| Algorithm | petgraph | Need to Build | Complexity |
|-----------|----------|---------------|------------|
| Eigenvector centrality | ❌ | ✅ Build | Medium |
| Label propagation | ❌ | ✅ Build | Low |
| Subgraph isomorphism | ❌ | ✅ Build | High |
| Modularity | ❌ | ✅ Build | Low |

### Priority 3: Nice to Have

| Algorithm | petgraph | Need to Build | Complexity |
|-----------|----------|---------------|------------|
| Infomap | ❌ | ✅ Build | Medium |
| Graph edit distance | ❌ | ✅ Build | High |
| Motif detection | ❌ | ✅ Build | High |

---

## 7. PERFORMANCE CHARACTERISTICS

### Memory Usage
```
Graph<N, E>:
- Node: 24 bytes + N
- Edge: 24 bytes + E

GraphMap<N, E>:
- Node: size_of<N>()
- Edge: size_of<E>() + overhead

CSR:
- Node: 4 bytes (index)
- Edge: 8 bytes (target + weight)
```

### Operation Complexity
```
| Operation | Graph | GraphMap | CSR | Matrix |
|-----------|-------|----------|-----|--------|
| Add node | O(1) | O(1) | - | - |
| Add edge | O(1) | O(1) | - | - |
| Remove node | O(V+E) | O(deg) | - | - |
| Remove edge | O(E) | O(1) | - | - |
| Edge lookup | O(deg) | O(1) | O(1) | O(1) |
| Neighbors | O(1)* | O(1) | O(1) | O(V) |
```

---

## 8. INTEGRATION PATTERNS

### 8.1 Building from AST
```rust
use petgraph::Graph;
use syn::Item;

fn build_call_graph(items: &[Item]) -> Graph<String, ()> {
    let mut graph = Graph::new();

    for item in items {
        if let Item::Fn(func) = item {
            let name = func.sig.ident.to_string();
            let node = graph.add_node(name.clone());

            // Find function calls in body
            // Add edges for each call
        }
    }

    graph
}
```

### 8.2 Working with NodeIndex
```rust
use petgraph::graph::NodeIndex;

// NodeIndex is a newtype wrapper around usize
let idx: NodeIndex = graph.add_node("data");

// Access node data
let node_data = &graph[idx];

// Access edges
for edge in graph.edges(idx) {
    println!("Edge to {:?}: {:?}", edge.target(), edge.weight());
}
```

### 8.3 Edge Iteration
```rust
// All edges from a node
for edge in graph.edges(node) {
    let target = edge.target();
    let weight = edge.weight();
}

// Incoming edges
for edge in graph.edges_directed(node, Direction::Incoming) {
    // ...
}

// All edges in graph
for edge in graph.edge_references() {
    // ...
}
```

---

## 9. COMPARISON WITH ALTERNATIVES

| Feature | petgraph | graphalgs | rustwork |
|---------|----------|-----------|----------|
| Maturity | High | Medium | Low |
| Algorithms | Basic | Extended | NetworkX-like |
| Performance | High | Medium | Unknown |
| Documentation | Good | Limited | Limited |
| Community | Large | Small | Small |

---

## 10. RECOMMENDATIONS FOR PARSERLTONGUE

### Use petgraph For:
✅ Graph storage (Graph<N, E> for code graphs)
✅ Basic traversal (BFS, DFS)
✅ Shortest path (Dijkstra, A*)
✅ SCC (Kosaraju)
✅ Cycle detection
✅ Topological sort

### Build Custom:
🔨 PageRank
🔨 Betweenness centrality
🔨 Leiden community detection
🔨 k-core decomposition
🔨 Blast radius calculation
🔨 Code-specific graph analysis

### Architecture:
```rust
pub struct CodeGraph {
    graph: Graph<CodeNode, CodeEdge>,
    // Custom algorithm implementations
    pagerank_cache: Option<HashMap<NodeIndex, f64>>,
    community_cache: Option<HashMap<NodeIndex, usize>>,
}

impl CodeGraph {
    pub fn pagerank(&mut self) -> &HashMap<NodeIndex, f64> {
        if self.pagerank_cache.is_none() {
            self.pagerank_cache = Some(self.compute_pagerank());
        }
        self.pagerank_cache.as_ref().unwrap()
    }
}
```

---

## 11. REFERENCES

- petgraph docs: https://docs.rs/petgraph
- petgraph repo: https://github.com/petgraph/petgraph
- graphalgs: https://github.com/starovoid/graphalgs
- daggy: https://github.com/mitchmindtree/daggy
