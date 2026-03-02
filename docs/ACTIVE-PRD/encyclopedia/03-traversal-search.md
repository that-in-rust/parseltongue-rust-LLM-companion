# Graph Traversal & Search Algorithms for Code Graphs

**Category:** Graph Navigation / Path Finding
**Target:** 15+ algorithms documented
**Purpose:** Navigate code relationships, find paths, explore dependencies

---

## Overview

Traversal algorithms answer: **"How do I explore the graph?"**

In code graphs, this translates to:
- What code depends on X? (upstream traversal)
- What does X depend on? (downstream traversal)
- Is there a path from A to B? (reachability)
- What's the shortest dependency chain? (shortest path)

---

## 1. BREADTH-FIRST SEARCH (BFS)

```
Approach: Level-by-level exploration
Complexity: O(V + E)
Data Structure: Queue
```

**Algorithm:**
```rust
fn bfs(graph: &DiGraph<Node, Edge>, start: NodeIndex) -> Vec<NodeIndex> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();

    queue.push_back(start);
    visited.insert(start);

    while let Some(v) = queue.pop_front() {
        result.push(v);

        for neighbor in graph.neighbors(v) {
            if visited.insert(neighbor) {
                queue.push_back(neighbor);
            }
        }
    }

    result
}
```

**Code Applications:**
- **Blast radius**: Find all code affected by a change (BFS on call graph)
- **Dependency discovery**: Find all dependencies of a module
- **Level-order analysis**: Group code by dependency distance

**petgraph:** ✅ `petgraph::algo::bfs_search`

---

## 2. DEPTH-FIRST SEARCH (DFS)

```
Approach: Deep exploration before backtracking
Complexity: O(V + E)
Data Structure: Stack (or recursion)
```

**Algorithm:**
```rust
fn dfs(graph: &DiGraph<Node, Edge>, start: NodeIndex) -> Vec<NodeIndex> {
    let mut visited = HashSet::new();
    let mut stack = vec![start];
    let mut result = Vec::new();

    while let Some(v) = stack.pop() {
        if visited.insert(v) {
            result.push(v);

            for neighbor in graph.neighbors(v) {
                if !visited.contains(&neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }

    result
}
```

**Code Applications:**
- **Call chain analysis**: Trace execution paths deeply
- **Cycle detection**: Detect circular dependencies
- **Topological sorting**: Order dependencies

**petgraph:** ✅ `petgraph::algo::dfs_search`

---

## 3. ITERATIVE DEEPENING DFS (IDDFS)

```
Approach: DFS with increasing depth limits
Complexity: O(b^d) where b = branching, d = depth
```

**Algorithm:**
```rust
fn iddfs(graph: &DiGraph<Node, Edge>, start: NodeIndex, max_depth: usize) -> Vec<NodeIndex> {
    let mut result = Vec::new();

    for depth in 0..=max_depth {
        let mut visited = HashSet::new();
        dls(graph, start, depth, &mut visited, &mut result);
    }

    result
}

fn dls(
    graph: &DiGraph<Node, Edge>,
    node: NodeIndex,
    depth: usize,
    visited: &mut HashSet<NodeIndex>,
    result: &mut Vec<NodeIndex>,
) {
    if depth == 0 {
        if visited.insert(node) {
            result.push(node);
        }
        return;
    }

    if visited.insert(node) {
        result.push(node);
        for neighbor in graph.neighbors(node) {
            if !visited.contains(&neighbor) {
                dls(graph, neighbor, depth - 1, visited, result);
            }
        }
    }
}
```

**Code Applications:**
- **Limited impact analysis**: "What's affected within 3 hops?"
- **Progressive exploration**: Show immediate effects first
- **Memory-efficient deep search**: When full BFS is too expensive

---

## 4. BIDIRECTIONAL SEARCH

```
Approach: BFS from both ends, meet in middle
Complexity: O(b^(d/2)) vs O(b^d) for BFS
```

**Algorithm:**
```rust
fn bidirectional_search(
    graph: &DiGraph<Node, Edge>,
    start: NodeIndex,
    goal: NodeIndex,
) -> Option<Vec<NodeIndex>> {
    if start == goal {
        return Some(vec![start]);
    }

    let mut forward_visited = HashMap::new();
    let mut backward_visited = HashMap::new();
    let mut forward_queue = VecDeque::new();
    let mut backward_queue = VecDeque::new();

    forward_visited.insert(start, None);
    backward_visited.insert(goal, None);
    forward_queue.push_back(start);
    backward_queue.push_back(goal);

    while !forward_queue.is_empty() || !backward_queue.is_empty() {
        // Forward step
        if let Some(v) = forward_queue.pop_front() {
            for neighbor in graph.neighbors(v) {
                if backward_visited.contains_key(&neighbor) {
                    return Some(reconstruct_path(&forward_visited, &backward_visited, v, neighbor, goal));
                }
                if !forward_visited.contains_key(&neighbor) {
                    forward_visited.insert(neighbor, Some(v));
                    forward_queue.push_back(neighbor);
                }
            }
        }

        // Backward step
        if let Some(v) = backward_queue.pop_front() {
            for neighbor in graph.neighbors_directed(v, Direction::Incoming) {
                if forward_visited.contains_key(&neighbor) {
                    return Some(reconstruct_path(&forward_visited, &backward_visited, neighbor, v, goal));
                }
                if !backward_visited.contains_key(&neighbor) {
                    backward_visited.insert(neighbor, Some(v));
                    backward_queue.push_back(neighbor);
                }
            }
        }
    }

    None
}
```

**Code Applications:**
- **Connection queries**: "Is module A connected to module B?"
- **Dependency path finding**: Fast path between distant code
- **Bottleneck detection**: Find meeting points (critical nodes)

---

## 5. A* SEARCH

```
Approach: Best-first search with heuristic
Complexity: O(E) best case, O(V log V) worst
```

**Algorithm:**
```rust
fn astar<F, H>(
    graph: &DiGraph<Node, Edge>,
    start: NodeIndex,
    goal: NodeIndex,
    cost: F,
    heuristic: H,
) -> Option<Vec<NodeIndex>>
where
    F: Fn(&Edge) -> f64,
    H: Fn(NodeIndex) -> f64,
{
    let mut open_set = BinaryHeap::new();
    let mut g_score: HashMap<NodeIndex, f64> = HashMap::new();
    let mut came_from: HashMap<NodeIndex, NodeIndex> = HashMap::new();

    g_score.insert(start, 0.0);
    open_set.push(State { node: start, f: heuristic(start) });

    while let Some(State { node: current, .. }) = open_set.pop() {
        if current == goal {
            return Some(reconstruct(&came_from, current));
        }

        for edge in graph.edges(current) {
            let neighbor = edge.target();
            let tentative_g = g_score[&current] + cost(&edge.weight());

            if tentative_g < g_score.get(&neighbor).unwrap_or(&f64::INFINITY) {
                came_from.insert(neighbor, current);
                g_score.insert(neighbor, tentative_g);
                let f = tentative_g + heuristic(neighbor);
                open_set.push(State { node: neighbor, f });
            }
        }
    }

    None
}
```

**Code Applications:**
- **Optimal dependency path**: Minimize "cost" (coupling, complexity)
- **Code archaeology**: Find paths respecting constraints
- **Directed impact analysis**: Goal-directed search

**Heuristics for Code:**
```
h(n) = 0  (becomes Dijkstra)
h(n) = package_distance(n, goal)  (same package = closer)
h(n) = 1 / call_frequency(n, goal)  (frequently called = closer)
```

**petgraph:** ✅ `petgraph::algo::astar`

---

## 6. DIJKSTRA'S ALGORITHM

```
Approach: Weighted shortest path
Complexity: O((V + E) log V) with priority queue
```

**Code Applications:**
- **Weighted dependency paths**: Edge weight = coupling strength
- **Minimum impact path**: Find path with least risk
- **Distance computation**: How "far" is A from B?

**petgraph:** ✅ `petgraph::algo::dijkstra`

---

## 7. BELLMAN-FORD

```
Approach: Handles negative weights
Complexity: O(V × E)
```

**Code Applications:**
- **Negative weight edges**: "This dependency reduces complexity"
- **Dependency graph with costs**: Some edges beneficial

---

## 8. FLOYD-WARSHALL

```
Approach: All-pairs shortest paths
Complexity: O(V³)
```

**Algorithm:**
```rust
fn floyd_warshall(graph: &DiGraph<Node, Edge>) -> Matrix<f64> {
    let n = graph.node_count();
    let mut dist = Matrix::new(n, n, f64::INFINITY);

    // Initialize
    for v in graph.node_indices() {
        dist[(v, v)] = 0.0;
        for edge in graph.edges(v) {
            dist[(v.index(), edge.target().index())] = 1.0; // or edge weight
        }
    }

    // Dynamic programming
    for k in 0..n {
        for i in 0..n {
            for j in 0..n {
                dist[(i, j)] = dist[(i, j)].min(dist[(i, k)] + dist[(k, j)]);
            }
        }
    }

    dist
}
```

**Code Applications:**
- **Diameter computation**: Maximum distance in codebase
- **Closeness centrality**: Precompute all distances
- **Reachability matrix**: Who can reach whom?

**Limitations:** Only practical for <5000 nodes

---

## 9. JOHNSON'S ALGORITHM

```
Approach: All-pairs shortest paths with negative weights
Complexity: O(V² log V + V×E)
```

**Code Applications:**
- Efficient all-pairs for sparse graphs
- Better than Floyd-Warshall for large sparse code graphs

---

## 10. UNIFORM COST SEARCH (UCS)

```
Approach: Dijkstra without full graph knowledge
Complexity: O(E log V)
```

**Code Applications:**
- Exploring graphs too large for memory
- Incremental codebase exploration

---

## 11. GREEDY BEST-FIRST SEARCH

```
Approach: Always expand most promising node
Complexity: O(b^m) worst case
```

**Code Applications:**
- Fast approximate path finding
- When optimality not required

---

## 12. BEAM SEARCH

```
Approach: Greedy with limited frontier (beam width k)
Complexity: O(k × b × d)
```

**Code Applications:**
- **Top-k paths**: "Show me the 3 most likely call chains"
- **Memory-efficient exploration**: Large codebases

---

## 13. RANDOM WALK

```
Approach: Random neighbor selection
Complexity: O(steps)
```

**Algorithm:**
```rust
fn random_walk(
    graph: &DiGraph<Node, Edge>,
    start: NodeIndex,
    steps: usize,
) -> Vec<NodeIndex> {
    let mut path = vec![start];
    let mut current = start;
    let mut rng = rand::thread_rng();

    for _ in 0..steps {
        let neighbors: Vec<_> = graph.neighbors(current).collect();
        if neighbors.is_empty() {
            break;
        }
        current = *neighbors.choose(&mut rng).unwrap();
        path.push(current);
    }

    path
}
```

**Code Applications:**
- **Sampling**: Random code exploration
- **PageRank computation**: Random walk basis
- **Community detection**: Walk-based similarity

---

## 14. RANDOM WALK WITH RESTART (RWR)

```
Approach: Random walk with probability of returning to start
Complexity: O(steps)
```

**Algorithm:**
```rust
fn random_walk_with_restart(
    graph: &DiGraph<Node, Edge>,
    start: NodeIndex,
    restart_prob: f64,
    steps: usize,
) -> HashMap<NodeIndex, f64> {
    let mut visits: HashMap<NodeIndex, f64> = HashMap::new();
    let mut current = start;
    let mut rng = rand::thread_rng();

    for _ in 0..steps {
        *visits.entry(current).or_insert(0.0) += 1.0;

        if rng.gen::<f64>() < restart_prob {
            current = start;
        } else {
            let neighbors: Vec<_> = graph.neighbors(current).collect();
            if neighbors.is_empty() {
                current = start;
            } else {
                current = *neighbors.choose(&mut rng).unwrap();
            }
        }
    }

    // Normalize
    let total = steps as f64;
    for count in visits.values_mut() {
        *count /= total;
    }

    visits
}
```

**Code Applications:**
- **Relevance scoring**: How related is X to start node?
- **Personalized PageRank**: Single-node importance
- **Recommendation**: "Code related to this function"

---

## 15. METROPOLIS-HASTINGS ON GRAPHS

```
Approach: MCMC sampling with acceptance probability
Complexity: O(steps)
```

**Code Applications:**
- **Uniform sampling**: Sample code uniformly
- **Graph statistics**: Estimate properties without full traversal

---

## 16. CHINESE POSTMAN (EULERIAN PATH)

```
Approach: Visit every edge exactly once
Complexity: O(V²) for augmentation
```

**Code Applications:**
- **Complete coverage testing**: Ensure all paths tested
- **Code tour**: Visit all dependencies systematically

---

## 17. HAMILTONIAN PATH (APPROXIMATION)

```
Approach: Visit every node exactly once
Complexity: NP-complete, use heuristics
```

**Code Applications:**
- **Optimal execution order**: Single-pass through all code
- **Tour generation**: Code review sequencing

---

## Comparative Analysis

| Algorithm | Complexity | Use Case | petgraph |
|-----------|------------|----------|----------|
| BFS | O(V+E) | Level exploration | ✅ |
| DFS | O(V+E) | Deep exploration, cycles | ✅ |
| IDDFS | O(b^d) | Limited depth search | ❌ |
| Bidirectional | O(b^(d/2)) | Fast path finding | ❌ |
| A* | O(E) best | Weighted optimal path | ✅ |
| Dijkstra | O((V+E)logV) | Shortest path | ✅ |
| Bellman-Ford | O(V×E) | Negative weights | ❌ |
| Floyd-Warshall | O(V³) | All-pairs (small graphs) | ❌ |
| Random Walk | O(steps) | Sampling, PageRank | ❌ |
| RWR | O(steps) | Relevance scoring | ❌ |

---

## Implementation Priority for Parseltongue

### Phase 1 (Already in petgraph)
1. ✅ BFS
2. ✅ DFS
3. ✅ Dijkstra
4. ✅ A*

### Phase 2 (Build Next)
5. Bidirectional search
6. Random Walk with Restart
7. Iterative deepening DFS

### Phase 3 (Specialized)
8. Floyd-Warshall (for small graphs)
9. Beam search
10. Chinese Postman

---

## Code-Specific Usage Patterns

### Blast Radius Analysis
```rust
// Find all functions affected by changing `func`
fn blast_radius(graph: &DiGraph, func: NodeIndex, max_hops: usize) -> HashSet<NodeIndex> {
    let mut affected = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((func, 0));

    while let Some((node, hops)) = queue.pop_front() {
        if hops > max_hops {
            continue;
        }

        for caller in graph.neighbors_directed(node, Direction::Incoming) {
            if affected.insert(caller) {
                queue.push_back((caller, hops + 1));
            }
        }
    }

    affected
}
```

### Dependency Path
```rust
// Find shortest path from main() to func
fn dependency_path(graph: &DiGraph, main: NodeIndex, func: NodeIndex) -> Option<Vec<NodeIndex>> {
    // Use BFS for unweighted
    bfs_path(graph, main, func)
}
```

### Reachability Check
```rust
// Can A reach B?
fn can_reach(graph: &DiGraph, from: NodeIndex, to: NodeIndex) -> bool {
    let mut visited = HashSet::new();
    let mut stack = vec![from];

    while let Some(v) = stack.pop() {
        if v == to {
            return true;
        }
        if visited.insert(v) {
            for neighbor in graph.neighbors(v) {
                stack.push(neighbor);
            }
        }
    }

    false
}
```
