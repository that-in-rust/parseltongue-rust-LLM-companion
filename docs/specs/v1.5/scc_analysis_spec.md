# Strongly Connected Components Analysis Endpoint Specification

**Version**: 1.5.0
**Endpoint**: `/strongly-connected-components-analysis`
**Method**: GET
**Status**: DRAFT
**Author**: Claude Code
**Date**: 2026-01-30

---

## Executive Summary

This specification defines a new HTTP endpoint that discovers all strongly connected components (SCCs) in the dependency graph using Tarjan's algorithm. An SCC is a maximal group of entities where every node can reach every other node through directed paths. This is more powerful than simple cycle detection: it reveals the complete structure of circular dependencies.

**Core Value**: Identifies architectural coupling problems with <100ms p99 latency for 10,000 entities using O(V+E) single-pass algorithm.

**Key Difference from `/circular-dependency-detection-scan`**:
- **Existing endpoint**: Detects individual cycles (A→B→A, C→D→E→C as separate cycles)
- **New endpoint**: Groups entities into maximal SCCs (reveals that A,B form one component, C,D,E form another)
- **Use case**: Architectural refactoring requires knowing which entities must be refactored together

---

## 1. Functional Requirements

### 1.1 Core Contract (WHEN...THEN...SHALL Format)

**REQ-SCC-001.0: Basic SCC Discovery**

**WHEN** a client sends `GET /strongly-connected-components-analysis`
**THEN** the system SHALL find all strongly connected components using Tarjan's algorithm
**AND** SHALL return each SCC with its member entities
**AND** SHALL identify which SCCs are cyclic (size > 1)
**AND** SHALL complete in < 100ms at p99 latency for 10,000 entities
**AND** SHALL use O(V+E) time complexity (single DFS pass)
**AND** SHALL allocate < 200KB memory during computation
**AND** SHALL order SCCs by size descending (largest first)

**Verification:**
```rust
#[test]
fn test_req_scc_001_basic_discovery() {
    // Arrange
    let state = create_test_graph_with_10000_entities();
    // Graph contains 3 SCCs:
    //   SCC1: 150 entities (large circular dependency cluster)
    //   SCC2: 8 entities (small cycle)
    //   Many singleton SCCs (entities not in cycles)

    // Act
    let start = Instant::now();
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query::default()
    ).await;
    let elapsed = start.elapsed();

    // Assert
    assert!(response.success);
    assert!(elapsed < Duration::from_millis(100)); // p99 target

    // Verify largest SCC found
    let largest_scc = &response.data.components[0];
    assert_eq!(largest_scc.size, 150);
    assert!(largest_scc.is_cyclic);
    assert_eq!(largest_scc.members.len(), 150);
}
```

---

**REQ-SCC-002.0: Component Filtering by Size**

**WHEN** a client sends `GET /strongly-connected-components-analysis?min_size=2`
**THEN** the system SHALL return only SCCs with size >= min_size
**AND** SHALL exclude singleton components (size=1) by default
**AND** SHALL support min_size range from 1 to 1000
**AND** SHALL default to min_size=2 (cyclic components only)
**AND** SHALL return empty array (not null) when no components match

**Verification:**
```rust
#[test]
fn test_req_scc_002_size_filtering() {
    // Arrange
    let state = create_test_graph_with_mixed_sccs();
    // Graph: SCC(150), SCC(8), SCC(3), 500 singletons

    // Act - Filter for size >= 5
    let params = SccAnalysisQueryParamsStruct {
        min_size: Some(5),
        include_singletons: None,
    };
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert_eq!(response.data.components.len(), 2); // SCC(150) and SCC(8)
    assert!(response.data.components.iter().all(|c| c.size >= 5));
    assert!(response.data.components.iter().all(|c| c.is_cyclic));
}
```

---

**REQ-SCC-003.0: Singleton Component Inclusion**

**WHEN** a client sends `GET /strongly-connected-components-analysis?include_singletons=true`
**THEN** the system SHALL include components with size=1 (isolated entities)
**AND** SHALL mark singletons with is_cyclic=false
**AND** SHALL override min_size filter when include_singletons=true
**AND** SHALL default to include_singletons=false

**Verification:**
```rust
#[test]
fn test_req_scc_003_singleton_inclusion() {
    // Arrange
    let state = create_test_graph_with_mixed_sccs();
    // Graph: 2 cyclic SCCs + 500 singletons

    // Act
    let params = SccAnalysisQueryParamsStruct {
        min_size: None,
        include_singletons: Some(true),
    };
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert_eq!(response.data.total_components, 502); // All components
    assert!(response.data.components.iter()
        .filter(|c| c.size == 1 && !c.is_cyclic)
        .count() == 500);
}
```

---

**REQ-SCC-004.0: Performance Contract**

**WHEN** processing a graph with 10,000 entities and 50,000 edges
**THEN** the system SHALL complete in < 100ms at p99 latency
**AND** SHALL use single-pass algorithm (Tarjan's, not Kosaraju's two-pass)
**AND** SHALL perform exactly one DFS traversal
**AND** SHALL allocate < 200KB peak memory
**AND** SHALL use zero heap allocations in hot path (DFS recursion uses stack)

**Verification:**
```rust
#[test]
fn test_req_scc_004_performance_contract() {
    // Arrange
    let state = create_dense_graph_10k_nodes_50k_edges();

    // Act
    let start = Instant::now();
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query::default()
    ).await;
    let elapsed = start.elapsed();

    // Assert
    assert!(elapsed < Duration::from_millis(100)); // p99 target
    assert!(response.data.total_components > 0);
    assert!(response.tokens <= 2000); // Token budget constraint
}
```

---

## 2. API Specification

### 2.1 HTTP Endpoint

```
GET /strongly-connected-components-analysis
```

### 2.2 Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `min_size` | integer | No | 2 | Minimum component size to include (1-1000) |
| `include_singletons` | boolean | No | false | Include size=1 components (isolated entities) |

**Parameter Validation:**
- `min_size`: Must be in range [1, 1000]. Values outside range return 400 error.
- `include_singletons`: Parsed as boolean ("true"/"false"). Invalid values return 400 error.

### 2.3 Response Structure

**Success Response (200 OK):**

```json
{
  "success": true,
  "endpoint": "/strongly-connected-components-analysis",
  "data": {
    "total_components": 47,
    "cyclic_components": 3,
    "components_returned": 3,
    "largest_component_size": 150,
    "algorithm": "tarjan",
    "components": [
      {
        "id": 1,
        "size": 150,
        "is_cyclic": true,
        "members": [
          "rust:fn:process_data_async:src_main_rs:10-50",
          "rust:fn:validate_input_schema:src_validation_rs:15-45",
          "rust:struct:DataProcessor:src_processor_rs:20-100",
          "... 147 more entities ..."
        ]
      },
      {
        "id": 2,
        "size": 8,
        "is_cyclic": true,
        "members": [
          "rust:fn:handle_request_pipeline:src_server_rs:100-150",
          "rust:fn:route_to_handler_async:src_router_rs:50-80",
          "rust:fn:middleware_chain_processor:src_middleware_rs:30-60",
          "... 5 more entities ..."
        ]
      },
      {
        "id": 3,
        "size": 3,
        "is_cyclic": true,
        "members": [
          "rust:fn:init_config_loader:src_config_rs:10-30",
          "rust:fn:parse_env_variables:src_env_rs:5-25",
          "rust:fn:validate_config_schema:src_config_rs:35-55"
        ]
      }
    ]
  },
  "tokens": 1850
}
```

**Error Response (400 BAD REQUEST):**

```json
{
  "success": false,
  "error": "Invalid min_size parameter. Must be between 1 and 1000. Got: 5000",
  "endpoint": "/strongly-connected-components-analysis",
  "tokens": 35
}
```

### 2.4 Response Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `total_components` | integer | Total SCCs found in graph (before filtering) |
| `cyclic_components` | integer | Count of SCCs with size > 1 (actual cycles) |
| `components_returned` | integer | Count of SCCs in response (after filtering) |
| `largest_component_size` | integer | Size of largest SCC found |
| `algorithm` | string | Always "tarjan" (future-proof for alternate algorithms) |
| `id` | integer | Component identifier (1-based, ordered by size desc) |
| `size` | integer | Number of entities in this SCC |
| `is_cyclic` | boolean | True if size > 1 (actual cycle), false for singletons |
| `members` | array[string] | Entity keys in this SCC (ISGL1 format) |

---

## 3. Performance Requirements

### 3.1 Latency Targets

| Entity Count | Edge Count | p50 Latency | p99 Latency | p99.9 Latency |
|--------------|-----------|-------------|-------------|---------------|
| 1,000 | 5,000 | < 10ms | < 30ms | < 50ms |
| 10,000 | 50,000 | < 30ms | < 100ms | < 200ms |
| 50,000 | 250,000 | < 100ms | < 300ms | < 500ms |

### 3.2 Memory Constraints

- **Hot path allocation**: Zero heap allocations during DFS (stack-based recursion)
- **Peak memory**: < 200KB for 10,000 entities
- **Per-entity overhead**: < 20 bytes (index, lowlink, stack flag)

### 3.3 Token Budget

- **Base response**: 100 tokens (headers, metadata)
- **Per component (cyclic)**: ~15 tokens (id, size, metadata)
- **Per member**: ~12 tokens (entity key)
- **Maximum response**: 2,000 tokens total

**Budget Calculation Example:**
```
Response with 3 components (sizes: 150, 8, 3):
  Base: 100 tokens
  Component metadata: 3 * 15 = 45 tokens
  Members: (150 + 8 + 3) * 12 = 1,932 tokens
  Total: ~2,077 tokens ≈ 2,000 tokens (within budget)
```

**Note**: For very large SCCs (size > 100), members array should be truncated with "... N more entities ..." to stay under 2K token budget.

---

## 4. Algorithm Specification

### 4.1 Tarjan's SCC Algorithm

**Why Tarjan over Kosaraju?**
- Tarjan: Single DFS pass, O(V+E) time
- Kosaraju: Two DFS passes, O(V+E) time but 2x traversal overhead
- For web services, single-pass = lower latency

**Algorithm Overview:**
1. Maintain DFS state: index, lowlink, on_stack flag per node
2. Single DFS traversal with stack
3. When DFS returns to a root node (index == lowlink), pop stack to extract SCC
4. Continue until all nodes visited

**Pseudo-code:**
```rust
fn tarjan_scc(graph: &Graph) -> Vec<SCC> {
    let mut index = 0;
    let mut stack = Vec::new();
    let mut state = HashMap::new(); // index, lowlink, on_stack per node
    let mut sccs = Vec::new();

    for node in graph.all_nodes() {
        if !state.contains_key(node) {
            strongconnect(node, &mut index, &mut stack, &mut state, &mut sccs);
        }
    }

    sccs
}

fn strongconnect(
    v: &Node,
    index: &mut usize,
    stack: &mut Vec<Node>,
    state: &mut HashMap<Node, NodeState>,
    sccs: &mut Vec<SCC>,
) {
    // Set depth index and lowlink for v
    state.insert(v, NodeState {
        index: *index,
        lowlink: *index,
        on_stack: true,
    });
    *index += 1;
    stack.push(v.clone());

    // Explore successors
    for w in graph.successors(v) {
        if !state.contains_key(w) {
            // Successor w not yet visited; recurse
            strongconnect(w, index, stack, state, sccs);
            state.get_mut(v).unwrap().lowlink =
                min(state[v].lowlink, state[w].lowlink);
        } else if state[w].on_stack {
            // Successor w is on stack (part of current SCC)
            state.get_mut(v).unwrap().lowlink =
                min(state[v].lowlink, state[w].index);
        }
    }

    // If v is a root node, pop the stack to create SCC
    if state[v].lowlink == state[v].index {
        let mut scc = Vec::new();
        loop {
            let w = stack.pop().unwrap();
            state.get_mut(&w).unwrap().on_stack = false;
            scc.push(w.clone());
            if w == *v { break; }
        }
        sccs.push(scc);
    }
}
```

### 4.2 CozoDB Query Pattern

**Step 1: Fetch all edges**
```datalog
?[from_key, to_key] := *DependencyEdges{from_key, to_key}
```

**Step 2: Build adjacency list in memory**
```rust
let mut graph: HashMap<String, Vec<String>> = HashMap::new();
for (from, to) in edges {
    graph.entry(from).or_default().push(to);
}
```

**Step 3: Run Tarjan's algorithm**
```rust
let sccs = tarjan_find_strongly_connected_components(&graph);
```

**Note**: In-memory algorithm is faster than iterative CozoDB queries for graph algorithms.

---

## 5. Data Structures (Following 4WNC)

### 5.1 Query Parameters Struct

```rust
/// Query parameters for SCC analysis endpoint
///
/// # 4-Word Name: SccAnalysisQueryParamsStruct
#[derive(Debug, Deserialize, Default)]
pub struct SccAnalysisQueryParamsStruct {
    /// Minimum component size to include
    #[serde(default = "default_min_size")]
    pub min_size: Option<usize>,

    /// Include singleton components (size=1)
    #[serde(default = "default_include_singletons")]
    pub include_singletons: Option<bool>,
}

fn default_min_size() -> Option<usize> {
    Some(2) // Exclude singletons by default
}

fn default_include_singletons() -> Option<bool> {
    Some(false)
}
```

### 5.2 Component Entry Payload

```rust
/// Single SCC component entry
///
/// # 4-Word Name: SccComponentEntryPayload
#[derive(Debug, Serialize)]
pub struct SccComponentEntryPayload {
    /// Component ID (1-based, ordered by size desc)
    pub id: usize,

    /// Number of entities in component
    pub size: usize,

    /// True if size > 1 (actual cycle)
    pub is_cyclic: bool,

    /// Entity keys in this component
    pub members: Vec<String>,
}
```

### 5.3 Response Data Payload

```rust
/// SCC analysis response data
///
/// # 4-Word Name: SccAnalysisDataPayload
#[derive(Debug, Serialize)]
pub struct SccAnalysisDataPayload {
    /// Total components found (before filtering)
    pub total_components: usize,

    /// Count of cyclic components (size > 1)
    pub cyclic_components: usize,

    /// Count of components in response
    pub components_returned: usize,

    /// Size of largest component
    pub largest_component_size: usize,

    /// Algorithm used
    pub algorithm: String,

    /// List of components
    pub components: Vec<SccComponentEntryPayload>,
}
```

### 5.4 Response Payload Struct

```rust
/// SCC analysis response payload
///
/// # 4-Word Name: SccAnalysisResponsePayload
#[derive(Debug, Serialize)]
pub struct SccAnalysisResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: SccAnalysisDataPayload,
    pub tokens: usize,
}
```

### 5.5 Error Response Struct

```rust
/// SCC analysis error response
///
/// # 4-Word Name: SccAnalysisErrorResponseStruct
#[derive(Debug, Serialize)]
pub struct SccAnalysisErrorResponseStruct {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
    pub tokens: usize,
}
```

### 5.6 Internal Node State

```rust
/// Tarjan algorithm node state
///
/// # 4-Word Name: TarjanNodeStateStruct
#[derive(Debug, Clone)]
struct TarjanNodeStateStruct {
    /// DFS discovery index
    index: usize,

    /// Lowest index reachable from this node
    lowlink: usize,

    /// Whether node is on DFS stack
    on_stack: bool,
}
```

---

## 6. Handler Function Signature

```rust
/// Handle strongly connected components analysis request
///
/// # 4-Word Name: handle_strongly_connected_components_analysis
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns all SCCs using Tarjan's algorithm
/// - Performance: <100ms at p99 for 10,000 entities, O(V+E) complexity
/// - Error Handling: Returns 400 for invalid parameters, 200 with empty array for no SCCs
///
/// # Algorithm
/// Uses Tarjan's strongly connected components algorithm:
/// - Single DFS pass with index/lowlink tracking
/// - Stack-based SCC extraction when root node found
/// - O(V+E) time, O(V) space complexity
///
/// # URL Pattern
/// - Endpoint: GET /strongly-connected-components-analysis?min_size=N&include_singletons=BOOL
/// - Default min_size: 2 (cyclic only)
/// - Default include_singletons: false
pub async fn handle_strongly_connected_components_analysis(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<SccAnalysisQueryParamsStruct>,
) -> impl IntoResponse {
    // Implementation...
}
```

---

## 7. Test Cases

### 7.1 Test Case 1: Basic SCC Discovery

```rust
#[test]
async fn test_scc_basic_discovery() {
    // Arrange
    let state = setup_test_graph_simple_sccs().await;
    // Graph structure:
    //   SCC1: A ⇄ B (cycle)
    //   SCC2: C → D → E → C (3-way cycle)
    //   SCC3: F (singleton)
    //   SCC4: G (singleton)

    // Act
    let params = SccAnalysisQueryParamsStruct::default();
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert!(response.success);
    assert_eq!(response.data.total_components, 4);
    assert_eq!(response.data.cyclic_components, 2);
    assert_eq!(response.data.components_returned, 2); // min_size=2 excludes singletons

    // Verify largest component (C,D,E)
    let largest = &response.data.components[0];
    assert_eq!(largest.size, 3);
    assert!(largest.is_cyclic);
    assert_eq!(largest.members.len(), 3);

    // Verify second component (A,B)
    let second = &response.data.components[1];
    assert_eq!(second.size, 2);
    assert!(second.is_cyclic);
}
```

### 7.2 Test Case 2: Size Filtering

```rust
#[test]
async fn test_scc_size_filtering() {
    // Arrange
    let state = setup_test_graph_varied_scc_sizes().await;
    // SCCs: size 50, size 10, size 5, size 2, 20 singletons

    // Act - Filter for size >= 10
    let params = SccAnalysisQueryParamsStruct {
        min_size: Some(10),
        include_singletons: None,
    };
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert_eq!(response.data.components_returned, 2); // size 50 and size 10
    assert!(response.data.components.iter().all(|c| c.size >= 10));
    assert_eq!(response.data.components[0].size, 50);
    assert_eq!(response.data.components[1].size, 10);
}
```

### 7.3 Test Case 3: Singleton Inclusion

```rust
#[test]
async fn test_scc_singleton_inclusion() {
    // Arrange
    let state = setup_test_graph_with_singletons().await;
    // Graph: 2 cyclic SCCs + 100 isolated entities

    // Act
    let params = SccAnalysisQueryParamsStruct {
        min_size: None,
        include_singletons: Some(true),
    };
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert_eq!(response.data.total_components, 102);
    assert_eq!(response.data.cyclic_components, 2);
    assert_eq!(response.data.components_returned, 102); // All components

    // Verify singletons marked correctly
    let singletons: Vec<_> = response.data.components.iter()
        .filter(|c| c.size == 1)
        .collect();
    assert_eq!(singletons.len(), 100);
    assert!(singletons.iter().all(|c| !c.is_cyclic));
}
```

### 7.4 Test Case 4: Performance Contract

```rust
#[test]
async fn test_scc_performance_contract() {
    // Arrange
    let state = setup_dense_graph_10k_entities_50k_edges().await;

    // Act
    let start = Instant::now();
    let params = SccAnalysisQueryParamsStruct::default();
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query(params)
    ).await;
    let elapsed = start.elapsed();

    // Assert
    assert!(elapsed < Duration::from_millis(100)); // p99 target
    assert!(response.data.total_components > 0);
    assert!(response.tokens <= 2000); // Token budget constraint
}
```

### 7.5 Test Case 5: Edge Cases

```rust
#[test]
async fn test_scc_edge_cases() {
    // Test 1: Empty graph
    let state = setup_empty_database().await;
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query(SccAnalysisQueryParamsStruct::default())
    ).await;
    assert_eq!(response.data.total_components, 0);
    assert_eq!(response.data.components.len(), 0);

    // Test 2: Graph with only singletons
    let state = setup_graph_all_singletons().await;
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query(SccAnalysisQueryParamsStruct::default())
    ).await;
    assert_eq!(response.data.cyclic_components, 0);
    assert_eq!(response.data.components_returned, 0); // default excludes singletons

    // Test 3: Invalid min_size
    let state = setup_test_graph_simple_sccs().await;
    let params = SccAnalysisQueryParamsStruct {
        min_size: Some(5000), // Exceeds max of 1000
        include_singletons: None,
    };
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query(params)
    ).await;
    assert!(!response.success);
    assert!(response.error.contains("min_size must be between 1 and 1000"));

    // Test 4: Single large SCC (entire graph is one cycle)
    let state = setup_complete_cycle_graph().await;
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query(SccAnalysisQueryParamsStruct::default())
    ).await;
    assert_eq!(response.data.total_components, 1);
    assert_eq!(response.data.cyclic_components, 1);
    assert!(response.data.components[0].is_cyclic);
}
```

### 7.6 Test Case 6: Tarjan Algorithm Correctness

```rust
#[test]
async fn test_scc_tarjan_correctness() {
    // Arrange - Graph from Tarjan's original paper
    let state = setup_tarjan_paper_example_graph().await;
    // Graph structure (Tarjan 1972):
    //   1 → 2 → 3 → 1 (SCC: {1,2,3})
    //   2 → 4 → 5 → 6 → 4 (SCC: {4,5,6})
    //   6 → 7 (singleton: {7})
    //   7 → 8 ⇄ 9 (SCC: {8,9})

    // Act
    let params = SccAnalysisQueryParamsStruct {
        min_size: Some(1),
        include_singletons: Some(true),
    };
    let response = handle_strongly_connected_components_analysis(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert_eq!(response.data.total_components, 4);

    // Verify each expected SCC
    let scc_sizes: Vec<usize> = response.data.components.iter()
        .map(|c| c.size)
        .collect();
    assert!(scc_sizes.contains(&3)); // {1,2,3}
    assert!(scc_sizes.contains(&3)); // {4,5,6}
    assert!(scc_sizes.contains(&2)); // {8,9}
    assert!(scc_sizes.contains(&1)); // {7}
}
```

---

## 8. Example API Usage

### 8.1 Find All Cyclic Components (Default)

```bash
curl -s "http://localhost:7777/strongly-connected-components-analysis" | jq '.'
```

**Response:**
```json
{
  "success": true,
  "endpoint": "/strongly-connected-components-analysis",
  "data": {
    "total_components": 47,
    "cyclic_components": 3,
    "components_returned": 3,
    "largest_component_size": 150,
    "algorithm": "tarjan",
    "components": [
      {
        "id": 1,
        "size": 150,
        "is_cyclic": true,
        "members": ["rust:fn:process_data_async:src_main_rs:10-50", "..."]
      }
    ]
  },
  "tokens": 1850
}
```

### 8.2 Find Large Cycles Only (min_size >= 10)

```bash
curl -s "http://localhost:7777/strongly-connected-components-analysis?min_size=10" | jq '.'
```

### 8.3 Include All Components (Even Singletons)

```bash
curl -s "http://localhost:7777/strongly-connected-components-analysis?include_singletons=true" | jq '.'
```

### 8.4 Find Small Cycles (2-5 entities)

```bash
curl -s "http://localhost:7777/strongly-connected-components-analysis?min_size=2" | jq '.data.components[] | select(.size <= 5)'
```

---

## 9. Comparison with Existing Endpoint

### 9.1 `/circular-dependency-detection-scan` (Existing)

**What it does:**
- Detects individual cycles using DFS with coloring
- Returns each cycle as a separate path (A→B→A, C→D→E→C)
- Multiple overlapping cycles reported separately

**Example output:**
```json
{
  "cycles": [
    {"length": 2, "path": ["A", "B", "A"]},
    {"length": 3, "path": ["C", "D", "E", "C"]},
    {"length": 4, "path": ["D", "E", "F", "D"]}  // Overlaps with previous cycle
  ]
}
```

**Limitation**: Cannot answer "which entities must be refactored together?"

### 9.2 `/strongly-connected-components-analysis` (New)

**What it does:**
- Groups entities into maximal SCCs
- Each entity appears in exactly one SCC
- Reveals complete structure of circular dependencies

**Example output:**
```json
{
  "components": [
    {"id": 1, "size": 4, "members": ["C", "D", "E", "F"]},  // Single SCC
    {"id": 2, "size": 2, "members": ["A", "B"]}
  ]
}
```

**Advantage**: Clearly identifies architectural coupling units.

### 9.3 Use Case Comparison

| Use Case | Use Existing Endpoint | Use New Endpoint |
|----------|----------------------|------------------|
| "Does my code have cycles?" | ✅ Faster for boolean check | ❌ Overkill |
| "Show me cycle paths" | ✅ Returns actual paths | ❌ Only members |
| "Which entities must be refactored together?" | ❌ Ambiguous with overlaps | ✅ Precise grouping |
| "Find largest coupling cluster" | ❌ No size aggregation | ✅ Sorted by size |
| "Architectural refactoring planning" | ❌ Incomplete view | ✅ Complete structure |

### 9.4 Performance Comparison

| Metric | Existing (DFS + Coloring) | New (Tarjan) |
|--------|--------------------------|--------------|
| Time Complexity | O(V+E) | O(V+E) |
| Graph Traversals | 1 DFS | 1 DFS |
| Memory Overhead | O(V) colors | O(V) state |
| Output Size | Variable (overlaps) | Minimal (no overlaps) |

**Conclusion**: Both are O(V+E), but Tarjan gives more actionable results.

---

## 10. Implementation File Location

**Handler Module**: `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/strongly_connected_components_handler.rs`

**Router Registration**: Add to `crates/pt08-http-code-query-server/src/http_server_startup_runner.rs`:

```rust
.route("/strongly-connected-components-analysis",
       get(strongly_connected_components_handler::handle_strongly_connected_components_analysis))
```

---

## 11. Token Budget Estimation

```rust
/// Estimate token count for SCC response
///
/// # 4-Word Name: estimate_token_count_for_response
fn estimate_token_count_for_response(
    components: &[SccComponentEntryPayload],
) -> usize {
    // Base response structure: 100 tokens
    let base = 100;

    // Per component metadata: ~15 tokens
    let component_overhead = components.len() * 15;

    // Per member entity key: ~12 tokens
    let total_members: usize = components.iter()
        .map(|c| c.members.len())
        .sum();
    let member_tokens = total_members * 12;

    base + component_overhead + member_tokens
}
```

**Budget Validation:**
- Small response (3 components, 20 total members): 100 + 45 + 240 = 385 tokens ✓
- Medium response (10 components, 100 members): 100 + 150 + 1,200 = 1,450 tokens ✓
- Large response (3 components, 150 members): 100 + 45 + 1,800 = 1,945 tokens ✓ (under 2K)

**Truncation Strategy for Large SCCs:**
```rust
// If SCC size > 100, truncate members array
if scc.size > 100 {
    let truncated = &scc.members[..50];
    members = format!("{:?}... {} more entities ...", truncated, scc.size - 50);
}
```

---

## 12. Success Criteria

### 12.1 Functional Criteria

- ✓ Tarjan's algorithm correctly identifies all SCCs
- ✓ SCCs sorted by size descending
- ✓ Filtering by min_size works correctly
- ✓ Singleton inclusion flag works correctly
- ✓ Each entity appears in exactly one SCC
- ✓ Error handling returns appropriate status codes
- ✓ Empty graph returns empty components array (not null)

### 12.2 Performance Criteria

- ✓ p99 latency < 100ms for 10,000 entities
- ✓ Memory allocation < 200KB
- ✓ Token budget <= 2,000 tokens
- ✓ Single DFS pass (verified via instrumentation)

### 12.3 Code Quality Criteria

- ✓ All function names follow 4WNC
- ✓ All tests pass (6 test cases minimum)
- ✓ Zero compiler warnings
- ✓ Zero TODOs or STUBs in committed code
- ✓ Documentation comments on all public items
- ✓ Algorithm matches Tarjan's original paper (1972)

---

## 13. Integration with Existing System

### 13.1 API Reference Documentation Update

Add to `/api-reference-documentation-help` response:

```json
{
  "path": "/strongly-connected-components-analysis",
  "method": "GET",
  "description": "Find strongly connected components using Tarjan's algorithm",
  "parameters": [
    {
      "name": "min_size",
      "param_type": "query",
      "required": false,
      "description": "Minimum component size (1-1000, default: 2)"
    },
    {
      "name": "include_singletons",
      "param_type": "query",
      "required": false,
      "description": "Include size=1 components (default: false)"
    }
  ]
}
```

### 13.2 Statistics Update

Consider adding to `/codebase-statistics-overview-summary`:

```json
{
  "scc_statistics": {
    "total_components": 47,
    "cyclic_components": 3,
    "largest_component_size": 150,
    "average_component_size": 2.4,
    "singleton_components": 44
  }
}
```

---

## 14. Appendix: Tarjan's Algorithm Theory

### 14.1 Historical Context

**Paper**: "Depth-first search and linear graph algorithms" (Robert Tarjan, 1972)
**Contribution**: First O(V+E) algorithm for SCC discovery using single DFS pass

### 14.2 Key Concepts

**Index**: DFS discovery time (order in which nodes are visited)
**Lowlink**: Lowest index reachable from this node via DFS tree or back edges
**Root Node**: Node where index == lowlink (start of new SCC)

### 14.3 Invariants

1. All nodes in an SCC have the same lowlink value
2. When a root node is found, all nodes on the stack up to root form the SCC
3. Each node is visited exactly once
4. Each edge is examined exactly once

### 14.4 Correctness Proof (Sketch)

**Claim**: When a root node v is identified (index == lowlink), all nodes on stack above v belong to same SCC.

**Proof**:
- If w is on stack above v, then there's a path v → ... → w (DFS tree path)
- If lowlink[v] == index[v], then all back edges from descendants of v lead to v or descendants of v
- Therefore, v and all nodes on stack above v are mutually reachable
- This is a maximal SCC (no other nodes can be added)

### 14.5 Comparison with Other Algorithms

| Algorithm | Passes | Time | Space | Notes |
|-----------|--------|------|-------|-------|
| Tarjan (1972) | 1 DFS | O(V+E) | O(V) | Stack-based |
| Kosaraju (1978) | 2 DFS | O(V+E) | O(V) | Easier to understand |
| Gabow (2000) | 1 DFS | O(V+E) | O(V) | Path-based, simpler |

**Why Tarjan for Parseltongue?**
- Single pass = lower latency in web service
- Stack-based = predictable memory usage
- Industry standard = well-tested implementations

---

## 15. Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-30 | Claude Code | Initial specification |

---

## 16. Sign-Off

**Specification Status**: DRAFT - Ready for Review
**Implementation Status**: NOT STARTED
**Target Release**: Parseltongue v1.5.0
**Estimated Effort**: 6-8 hours (algorithm + handler + tests + integration)

**Reviewer Notes:**
- Verify algorithm matches Tarjan 1972 paper
- Validate token budget calculation for large SCCs
- Test with real-world graphs (Parseltongue on itself)
- Consider member truncation strategy for components > 100 entities

---

**END OF SPECIFICATION**
