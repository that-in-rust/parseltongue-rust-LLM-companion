# PageRank Importance Ranking Specification

**Version**: 1.5.0
**Endpoint**: `/pagerank-importance-ranking-view`
**Method**: GET
**Status**: DRAFT
**Author**: Claude Code
**Date**: 2026-01-30

---

## Executive Summary

This specification defines a new HTTP endpoint that ranks code entities by importance using the PageRank algorithm. PageRank identifies critical entities based on graph topology - entities that are called by many important entities score higher than those with simple coupling counts.

**Core Value**: Identifies truly important entities with <200ms p99 latency for 10,000 entities at 50 iterations, using iterative convergence.

---

## 1. Functional Requirements

### 1.1 Core Contract (WHEN...THEN...SHALL Format)

**REQ-PR-001.0: Basic PageRank Calculation**

**WHEN** a client sends `GET /pagerank-importance-ranking-view`
**THEN** the system SHALL compute PageRank scores for all entities
**AND** SHALL use iterative algorithm: PR(A) = (1-d) + d * Σ(PR(T)/C(T))
**AND** SHALL default to damping factor d=0.85
**AND** SHALL default to 50 iterations
**AND** SHALL normalize scores to range [0.0, 1.0]
**AND** SHALL complete in < 200ms at p99 latency for 10,000 entities
**AND** SHALL allocate < 1MB memory during computation
**AND** SHALL return results ordered by score descending

**Verification:**
```rust
#[test]
fn test_req_pr_001_basic_pagerank_calculation() {
    // Arrange
    let entities = create_test_graph_with_10000_entities();

    // Act
    let start = Instant::now();
    let params = PageRankQueryParamsStruct::default();
    let response = handle_pagerank_importance_ranking_view(&state, Query(params)).await;
    let elapsed = start.elapsed();

    // Assert
    assert!(response.success);
    assert_eq!(response.data.entities_analyzed, 10000);
    assert!(elapsed < Duration::from_millis(200)); // p99 target

    // Verify scores are normalized
    for entry in &response.data.rankings {
        assert!(entry.pagerank_score >= 0.0);
        assert!(entry.pagerank_score <= 1.0);
    }

    // Verify descending order
    for i in 0..response.data.rankings.len()-1 {
        assert!(response.data.rankings[i].pagerank_score >=
                response.data.rankings[i+1].pagerank_score);
    }
}
```

---

**REQ-PR-002.0: Configurable Iterations and Damping**

**WHEN** a client sends `GET /pagerank-importance-ranking-view?iterations=100&damping=0.9`
**THEN** the system SHALL use specified iteration count
**AND** SHALL use specified damping factor
**AND** SHALL validate iterations in range [1, 1000]
**AND** SHALL validate damping in range [0.0, 1.0]
**AND** SHALL return 400 Bad Request for invalid parameters
**AND** SHALL include convergence_delta in response metadata

**Verification:**
```rust
#[test]
fn test_req_pr_002_configurable_iterations_damping() {
    // Test custom parameters
    let params = PageRankQueryParamsStruct {
        iterations: Some(100),
        damping: Some(0.9),
        limit: Some(20),
    };
    let response = handle_pagerank_importance_ranking_view(&state, Query(params)).await;

    assert!(response.success);
    assert_eq!(response.data.iterations_run, 100);
    assert!((response.data.damping_factor - 0.9).abs() < 0.0001);

    // Test invalid iterations
    let params = PageRankQueryParamsStruct {
        iterations: Some(5000), // Exceeds max
        damping: None,
        limit: None,
    };
    let response = handle_pagerank_importance_ranking_view(&state, Query(params)).await;
    assert!(!response.success);
    assert!(response.error.contains("iterations must be between 1 and 1000"));

    // Test invalid damping
    let params = PageRankQueryParamsStruct {
        iterations: None,
        damping: Some(1.5), // Exceeds max
        limit: None,
    };
    let response = handle_pagerank_importance_ranking_view(&state, Query(params)).await;
    assert!(!response.success);
    assert!(response.error.contains("damping must be between 0.0 and 1.0"));
}
```

---

**REQ-PR-003.0: Early Convergence Optimization**

**WHEN** PageRank scores converge before max iterations
**THEN** the system SHALL detect convergence (delta < 0.0001)
**AND** SHALL stop early to save computation
**AND** SHALL report actual iterations run
**AND** SHALL report final convergence delta
**AND** SHALL maintain <200ms latency guarantee

**Verification:**
```rust
#[test]
fn test_req_pr_003_early_convergence() {
    // Arrange: Simple linear graph that converges quickly
    let state = setup_test_state_with_linear_graph().await;
    // A -> B -> C -> D (converges in ~10 iterations)

    // Act
    let params = PageRankQueryParamsStruct {
        iterations: Some(100), // Request 100
        damping: Some(0.85),
        limit: Some(10),
    };
    let response = handle_pagerank_importance_ranking_view(&state, Query(params)).await;

    // Assert
    assert!(response.data.iterations_run < 100); // Stopped early
    assert!(response.data.convergence_delta < 0.0001); // Converged
    assert!(response.data.converged); // Flag set
}
```

---

**REQ-PR-004.0: Pagination and Limiting**

**WHEN** a client sends `GET /pagerank-importance-ranking-view?limit=20`
**THEN** the system SHALL return top N entities by score
**AND** SHALL default to limit of 20 when not specified
**AND** SHALL support limit up to 100 maximum
**AND** SHALL assign ranks (1-based) after sorting
**AND** SHALL include total entities analyzed in metadata

**Verification:**
```rust
#[test]
fn test_req_pr_004_pagination_limiting() {
    // Test default limit
    let params = PageRankQueryParamsStruct::default();
    let response = handle_pagerank_importance_ranking_view(&state, Query(params)).await;
    assert_eq!(response.data.rankings.len(), 20); // Default

    // Test custom limit
    let params = PageRankQueryParamsStruct {
        iterations: None,
        damping: None,
        limit: Some(50),
    };
    let response = handle_pagerank_importance_ranking_view(&state, Query(params)).await;
    assert_eq!(response.data.rankings.len(), 50);

    // Test limit exceeds max
    let params = PageRankQueryParamsStruct {
        iterations: None,
        damping: None,
        limit: Some(500), // Exceeds max
    };
    let response = handle_pagerank_importance_ranking_view(&state, Query(params)).await;
    assert!(!response.success);
    assert!(response.error.contains("limit must be between 1 and 100"));

    // Verify ranks are sequential
    let params = PageRankQueryParamsStruct {
        limit: Some(10),
        ..Default::default()
    };
    let response = handle_pagerank_importance_ranking_view(&state, Query(params)).await;
    for (idx, entry) in response.data.rankings.iter().enumerate() {
        assert_eq!(entry.rank, idx + 1);
    }
}
```

---

## 2. API Specification

### 2.1 HTTP Endpoint

```
GET /pagerank-importance-ranking-view
```

### 2.2 Query Parameters

| Parameter | Type | Required | Default | Valid Range | Description |
|-----------|------|----------|---------|-------------|-------------|
| `iterations` | integer | No | 50 | [1, 1000] | Maximum PageRank iterations |
| `damping` | float | No | 0.85 | [0.0, 1.0] | Damping factor (probability of following edge) |
| `limit` | integer | No | 20 | [1, 100] | Number of top entities to return |

### 2.3 Response Structure

**Success Response (200 OK):**

```json
{
  "success": true,
  "endpoint": "/pagerank-importance-ranking-view",
  "data": {
    "entities_analyzed": 1247,
    "rankings_returned": 20,
    "iterations_run": 32,
    "damping_factor": 0.85,
    "converged": true,
    "convergence_delta": 0.00008,
    "computation_time_ms": 145,
    "rankings": [
      {
        "rank": 1,
        "entity_key": "rust:fn:new:unknown:0-0",
        "pagerank_score": 1.0,
        "normalized_score": 1.0,
        "inbound_edges": 352,
        "outbound_edges": 0
      },
      {
        "rank": 2,
        "entity_key": "rust:method:handle_request:src_server_rs:45-120",
        "pagerank_score": 0.847,
        "normalized_score": 0.847,
        "inbound_edges": 48,
        "outbound_edges": 12
      }
    ]
  },
  "tokens": 980
}
```

**Error Response (400 BAD REQUEST):**

```json
{
  "success": false,
  "error": "Invalid damping parameter. Must be between 0.0 and 1.0",
  "endpoint": "/pagerank-importance-ranking-view",
  "tokens": 42
}
```

### 2.4 Response Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `rank` | integer | Position in sorted list (1-based) |
| `entity_key` | string | ISGL1 entity key |
| `pagerank_score` | float | Raw PageRank score from algorithm |
| `normalized_score` | float | Score normalized to [0.0, 1.0] range |
| `inbound_edges` | integer | Number of entities calling this entity |
| `outbound_edges` | integer | Number of entities this entity calls |
| `entities_analyzed` | integer | Total entities in graph |
| `rankings_returned` | integer | Number of results returned (≤ limit) |
| `iterations_run` | integer | Actual iterations executed |
| `damping_factor` | float | Damping factor used |
| `converged` | boolean | Whether algorithm converged before max iterations |
| `convergence_delta` | float | Maximum score change in final iteration |
| `computation_time_ms` | integer | Time spent computing PageRank |

---

## 3. Performance Requirements

### 3.1 Latency Targets

| Entity Count | Iterations | p50 Latency | p99 Latency | p99.9 Latency |
|--------------|-----------|-------------|-------------|---------------|
| 1,000 | 50 | < 20ms | < 50ms | < 100ms |
| 10,000 | 50 | < 80ms | < 200ms | < 400ms |
| 10,000 | 100 | < 150ms | < 350ms | < 700ms |

### 3.2 Memory Constraints

- **Per-entity overhead**: 2 x 8 bytes (score + previous score) = 16 bytes
- **Peak memory**: < 1MB for 10,000 entities (10k * 16 bytes = 160KB + overhead)
- **Zero allocations**: After initial HashMap creation, zero heap allocations in hot loop

### 3.3 Token Budget

- **Base response**: 120 tokens (metadata)
- **Per ranking entry**: ~40 tokens
- **Default (limit=20)**: 120 + (20 * 40) = 920 tokens ✓
- **Maximum (limit=100)**: 120 + (100 * 40) = 4,120 tokens ✓

---

## 4. Algorithm Specification

### 4.1 PageRank Formula

**Standard PageRank:**
```
PR(A) = (1-d) + d * Σ(PR(T_i) / C(T_i))

Where:
- PR(A) = PageRank score of entity A
- d = damping factor (default 0.85)
- T_i = entities with edges pointing to A
- C(T_i) = total outbound edges from T_i
- Σ = sum over all inbound entities
```

**Interpretation:**
- (1-d) = base score (random jump probability)
- d * Σ(...) = weighted contribution from incoming entities
- Higher scores come from being referenced by high-scoring entities

### 4.2 Iterative Algorithm

```rust
/// Calculate PageRank scores iteratively
///
/// # 4-Word Name: calculate_pagerank_scores_iteratively
///
/// # Algorithm
/// 1. Build adjacency lists from edges
/// 2. Initialize all entities with score = 1.0 / N
/// 3. For each iteration:
///    a. For each entity A:
///       - Sum contributions from all inbound entities
///       - PR(A) = (1-d) + d * Σ(PR(T_i) / C(T_i))
///    b. Calculate max delta across all entities
///    c. If max_delta < 0.0001, break (converged)
/// 4. Normalize scores to [0.0, 1.0] range
/// 5. Sort by score descending
/// 6. Take top N, assign ranks
async fn calculate_pagerank_scores_iteratively(
    state: &SharedApplicationStateContainer,
    iterations: usize,
    damping: f64,
    limit: usize,
) -> PageRankComputationResult {
    // Implementation details...
}
```

### 4.3 Convergence Detection

```rust
/// Check if PageRank has converged
///
/// # 4-Word Name: check_pagerank_convergence_threshold
///
/// Convergence occurs when maximum score change < 0.0001
fn check_pagerank_convergence_threshold(
    current_scores: &HashMap<String, f64>,
    previous_scores: &HashMap<String, f64>,
) -> (bool, f64) {
    let mut max_delta = 0.0;

    for (key, current) in current_scores {
        let previous = previous_scores.get(key).unwrap_or(&0.0);
        let delta = (current - previous).abs();
        max_delta = max_delta.max(delta);
    }

    (max_delta < 0.0001, max_delta)
}
```

### 4.4 Score Normalization

```rust
/// Normalize PageRank scores to [0.0, 1.0]
///
/// # 4-Word Name: normalize_pagerank_scores_to_range
fn normalize_pagerank_scores_to_range(
    scores: &HashMap<String, f64>
) -> HashMap<String, f64> {
    let max_score = scores.values()
        .copied()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(1.0);

    let min_score = scores.values()
        .copied()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);

    let range = max_score - min_score;

    if range == 0.0 {
        // All scores equal - return all as 1.0
        scores.keys()
            .map(|k| (k.clone(), 1.0))
            .collect()
    } else {
        scores.iter()
            .map(|(k, v)| {
                let normalized = (v - min_score) / range;
                (k.clone(), normalized)
            })
            .collect()
    }
}
```

---

## 5. Data Structures (Following 4WNC)

### 5.1 Query Parameters Struct

```rust
/// Query parameters for PageRank ranking
///
/// # 4-Word Name: PageRankQueryParamsStruct
#[derive(Debug, Deserialize, Default)]
pub struct PageRankQueryParamsStruct {
    /// Maximum iterations (1-1000, default: 50)
    #[serde(default = "default_iterations")]
    pub iterations: Option<usize>,

    /// Damping factor (0.0-1.0, default: 0.85)
    #[serde(default = "default_damping")]
    pub damping: Option<f64>,

    /// Number of top results (1-100, default: 20)
    #[serde(default = "default_limit")]
    pub limit: Option<usize>,
}

fn default_iterations() -> Option<usize> {
    Some(50)
}

fn default_damping() -> Option<f64> {
    Some(0.85)
}

fn default_limit() -> Option<usize> {
    Some(20)
}
```

### 5.2 Ranking Entry Payload

```rust
/// Single PageRank ranking entry
///
/// # 4-Word Name: PageRankRankingEntryPayload
#[derive(Debug, Serialize)]
pub struct PageRankRankingEntryPayload {
    pub rank: usize,
    pub entity_key: String,
    pub pagerank_score: f64,
    pub normalized_score: f64,
    pub inbound_edges: usize,
    pub outbound_edges: usize,
}
```

### 5.3 Response Data Payload

```rust
/// PageRank ranking response data
///
/// # 4-Word Name: PageRankRankingDataPayload
#[derive(Debug, Serialize)]
pub struct PageRankRankingDataPayload {
    pub entities_analyzed: usize,
    pub rankings_returned: usize,
    pub iterations_run: usize,
    pub damping_factor: f64,
    pub converged: bool,
    pub convergence_delta: f64,
    pub computation_time_ms: u128,
    pub rankings: Vec<PageRankRankingEntryPayload>,
}
```

### 5.4 Response Payload Struct

```rust
/// PageRank ranking response payload
///
/// # 4-Word Name: PageRankRankingResponsePayload
#[derive(Debug, Serialize)]
pub struct PageRankRankingResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: PageRankRankingDataPayload,
    pub tokens: usize,
}
```

### 5.5 Error Response Struct

```rust
/// PageRank ranking error response
///
/// # 4-Word Name: PageRankRankingErrorResponseStruct
#[derive(Debug, Serialize)]
pub struct PageRankRankingErrorResponseStruct {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
    pub tokens: usize,
}
```

### 5.6 Internal Computation Result

```rust
/// Internal PageRank computation result
///
/// # 4-Word Name: PageRankComputationResultStruct
struct PageRankComputationResultStruct {
    pub scores: HashMap<String, f64>,
    pub iterations_run: usize,
    pub converged: bool,
    pub convergence_delta: f64,
    pub computation_time_ms: u128,
}
```

---

## 6. Handler Function Signature

```rust
/// Handle PageRank importance ranking view request
///
/// # 4-Word Name: handle_pagerank_importance_ranking_view
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns entities ranked by PageRank score
/// - Performance: <200ms at p99 for 10,000 entities at 50 iterations
/// - Error Handling: Returns 400 for invalid parameters, 200 with empty array for no entities
///
/// # URL Pattern
/// - Endpoint: GET /pagerank-importance-ranking-view?iterations=N&damping=D&limit=L
/// - Default iterations: 50
/// - Default damping: 0.85
/// - Default limit: 20
pub async fn handle_pagerank_importance_ranking_view(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<PageRankQueryParamsStruct>,
) -> impl IntoResponse {
    // Implementation...
}
```

---

## 7. Test Cases

### 7.1 Test Case 1: Basic PageRank Calculation

```rust
#[test]
async fn test_pagerank_basic_calculation() {
    // Arrange
    let state = setup_test_state_with_sample_graph().await;
    // Graph structure:
    //   A -> B
    //   A -> C
    //   B -> C
    //   C -> D
    //
    // Expected PageRank (intuitive):
    //   C = highest (receives from A and B)
    //   D = moderate (receives from C which is important)
    //   B = moderate (receives from A)
    //   A = lowest (receives from nothing)

    // Act
    let params = PageRankQueryParamsStruct {
        iterations: Some(50),
        damping: Some(0.85),
        limit: Some(10),
    };
    let response = handle_pagerank_importance_ranking_view(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert!(response.success);
    assert_eq!(response.data.entities_analyzed, 4);

    // C should have highest score
    let top_entity = &response.data.rankings[0];
    assert!(top_entity.entity_key.contains(":C:"));
    assert!(top_entity.pagerank_score > 0.0);

    // A should have lowest score
    let bottom_entity = &response.data.rankings[3];
    assert!(bottom_entity.entity_key.contains(":A:"));
}
```

### 7.2 Test Case 2: Convergence Detection

```rust
#[test]
async fn test_pagerank_convergence_detection() {
    // Arrange: Simple graph that converges quickly
    let state = setup_test_state_with_linear_graph().await;

    // Act
    let params = PageRankQueryParamsStruct {
        iterations: Some(100), // Request many
        damping: Some(0.85),
        limit: Some(10),
    };
    let response = handle_pagerank_importance_ranking_view(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert!(response.data.converged);
    assert!(response.data.iterations_run < 100); // Stopped early
    assert!(response.data.convergence_delta < 0.0001);
}
```

### 7.3 Test Case 3: Parameter Validation

```rust
#[test]
async fn test_pagerank_parameter_validation() {
    let state = setup_test_state_with_sample_graph().await;

    // Test invalid iterations (too high)
    let params = PageRankQueryParamsStruct {
        iterations: Some(5000),
        damping: None,
        limit: None,
    };
    let response = handle_pagerank_importance_ranking_view(
        State(state.clone()),
        Query(params)
    ).await;
    assert!(!response.success);
    assert!(response.error.contains("iterations"));

    // Test invalid damping (negative)
    let params = PageRankQueryParamsStruct {
        iterations: None,
        damping: Some(-0.5),
        limit: None,
    };
    let response = handle_pagerank_importance_ranking_view(
        State(state.clone()),
        Query(params)
    ).await;
    assert!(!response.success);
    assert!(response.error.contains("damping"));

    // Test invalid limit (too high)
    let params = PageRankQueryParamsStruct {
        iterations: None,
        damping: None,
        limit: Some(500),
    };
    let response = handle_pagerank_importance_ranking_view(
        State(state),
        Query(params)
    ).await;
    assert!(!response.success);
    assert!(response.error.contains("limit"));
}
```

### 7.4 Test Case 4: Performance Contract

```rust
#[test]
async fn test_pagerank_performance_contract() {
    // Arrange
    let state = setup_test_state_with_10000_entities().await;

    // Act
    let start = Instant::now();
    let params = PageRankQueryParamsStruct {
        iterations: Some(50),
        damping: Some(0.85),
        limit: Some(20),
    };
    let response = handle_pagerank_importance_ranking_view(
        State(state),
        Query(params)
    ).await;
    let elapsed = start.elapsed();

    // Assert
    assert!(elapsed < Duration::from_millis(200)); // p99 target
    assert_eq!(response.data.entities_analyzed, 10000);
    assert!(response.data.rankings.len() <= 20);
    assert!(response.tokens <= 1000); // Token budget constraint
}
```

### 7.5 Test Case 5: Score Normalization

```rust
#[test]
async fn test_pagerank_score_normalization() {
    // Arrange
    let state = setup_test_state_with_varied_graph().await;

    // Act
    let params = PageRankQueryParamsStruct {
        iterations: Some(50),
        damping: Some(0.85),
        limit: Some(100),
    };
    let response = handle_pagerank_importance_ranking_view(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert!(response.success);

    // Verify all scores in [0.0, 1.0]
    for entry in &response.data.rankings {
        assert!(entry.normalized_score >= 0.0);
        assert!(entry.normalized_score <= 1.0);
        assert!(entry.pagerank_score >= 0.0);
    }

    // Verify top score is 1.0 (normalized max)
    let max_score = response.data.rankings[0].normalized_score;
    assert!((max_score - 1.0).abs() < 0.0001);
}
```

### 7.6 Test Case 6: Empty Graph

```rust
#[test]
async fn test_pagerank_empty_graph() {
    // Arrange
    let state = setup_empty_database().await;

    // Act
    let params = PageRankQueryParamsStruct::default();
    let response = handle_pagerank_importance_ranking_view(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert!(response.success);
    assert_eq!(response.data.entities_analyzed, 0);
    assert_eq!(response.data.rankings.len(), 0);
    assert_eq!(response.data.iterations_run, 0);
}
```

### 7.7 Test Case 7: Circular Dependencies

```rust
#[test]
async fn test_pagerank_circular_dependencies() {
    // Arrange: Circular graph A -> B -> C -> A
    let state = setup_test_state_with_circular_graph().await;

    // Act
    let params = PageRankQueryParamsStruct {
        iterations: Some(100),
        damping: Some(0.85),
        limit: Some(10),
    };
    let response = handle_pagerank_importance_ranking_view(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert!(response.success);
    assert!(response.data.converged); // Should still converge

    // All entities should have similar scores (symmetric graph)
    let scores: Vec<f64> = response.data.rankings
        .iter()
        .map(|r| r.normalized_score)
        .collect();

    let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;
    for score in scores {
        // All scores should be within 10% of average
        assert!((score - avg_score).abs() < 0.1);
    }
}
```

---

## 8. Example API Usage

### 8.1 Get Top 20 Most Important Entities (Default)

```bash
curl -s "http://localhost:7777/pagerank-importance-ranking-view" | jq '.'
```

**Response:**
```json
{
  "success": true,
  "endpoint": "/pagerank-importance-ranking-view",
  "data": {
    "entities_analyzed": 1247,
    "rankings_returned": 20,
    "iterations_run": 32,
    "damping_factor": 0.85,
    "converged": true,
    "convergence_delta": 0.00008,
    "computation_time_ms": 145,
    "rankings": [
      {
        "rank": 1,
        "entity_key": "rust:fn:new:unknown:0-0",
        "pagerank_score": 0.847,
        "normalized_score": 1.0,
        "inbound_edges": 352,
        "outbound_edges": 0
      }
    ]
  },
  "tokens": 920
}
```

### 8.2 High-Precision Ranking (More Iterations)

```bash
curl -s "http://localhost:7777/pagerank-importance-ranking-view?iterations=100&limit=10" | jq '.'
```

### 8.3 Custom Damping Factor (More Random Jumps)

```bash
curl -s "http://localhost:7777/pagerank-importance-ranking-view?damping=0.7&limit=15" | jq '.'
```

### 8.4 Fast Approximation (Fewer Iterations)

```bash
curl -s "http://localhost:7777/pagerank-importance-ranking-view?iterations=10&limit=20" | jq '.'
```

---

## 9. Implementation File Location

**Handler Module**: `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/pagerank_importance_ranking_handler.rs`

**Router Registration**: Add to `crates/pt08-http-code-query-server/src/http_server_startup_runner.rs`:

```rust
.route("/pagerank-importance-ranking-view",
       get(pagerank_importance_ranking_handler::handle_pagerank_importance_ranking_view))
```

---

## 10. Token Budget Estimation

```rust
/// Estimate token count for PageRank response
///
/// # 4-Word Name: estimate_token_count_for_response
fn estimate_token_count_for_response(rankings: &[PageRankRankingEntryPayload]) -> usize {
    // Base response structure: 120 tokens (metadata)
    let base = 120;

    // Per ranking entry: ~40 tokens
    // {rank, entity_key, pagerank_score, normalized_score, inbound, outbound}
    let per_ranking = 40;

    // Calculate
    base + (rankings.len() * per_ranking)
}
```

**Budget Validation:**
- At limit=20 (default): 120 + (20 * 40) = 920 tokens ✓
- At limit=100 (max): 120 + (100 * 40) = 4,120 tokens ✓

---

## 11. Success Criteria

### 11.1 Functional Criteria

- ✓ PageRank algorithm produces mathematically correct scores
- ✓ Iterations parameter controls computation depth
- ✓ Damping parameter affects score distribution
- ✓ Early convergence detection works correctly
- ✓ Score normalization produces [0.0, 1.0] range
- ✓ Pagination respects limit parameter
- ✓ Error handling validates all parameters

### 11.2 Performance Criteria

- ✓ p99 latency < 200ms for 10,000 entities at 50 iterations
- ✓ Memory allocation < 1MB
- ✓ Token budget <= 1,000 for default limit
- ✓ Early convergence saves computation time

### 11.3 Code Quality Criteria

- ✓ All function names follow 4WNC
- ✓ All tests pass (7 test cases minimum)
- ✓ Zero compiler warnings
- ✓ Zero TODOs or STUBs in committed code
- ✓ Documentation comments on all public items

---

## 12. Integration with Existing System

### 12.1 API Reference Documentation Update

Add to `/api-reference-documentation-help` response:

```json
{
  "path": "/pagerank-importance-ranking-view",
  "method": "GET",
  "description": "Rank entities by importance using PageRank algorithm",
  "parameters": [
    {
      "name": "iterations",
      "param_type": "query",
      "required": false,
      "description": "Maximum iterations (1-1000, default: 50)"
    },
    {
      "name": "damping",
      "param_type": "query",
      "required": false,
      "description": "Damping factor (0.0-1.0, default: 0.85)"
    },
    {
      "name": "limit",
      "param_type": "query",
      "required": false,
      "description": "Number of top results (1-100, default: 20)"
    }
  ]
}
```

### 12.2 Health Check Compatibility

- No changes required to `/server-health-check-status`
- Database connection reused from existing pool
- Algorithm runs in-memory after edge loading

### 12.3 Statistics Update

Consider adding to `/codebase-statistics-overview-summary`:

```json
{
  "importance_statistics": {
    "top_pagerank_score": 0.847,
    "average_pagerank_score": 0.124,
    "median_pagerank_score": 0.089,
    "entities_with_zero_inbound": 245
  }
}
```

---

## 13. Appendix: PageRank Theory

### 13.1 Algorithm Background

PageRank was developed by Larry Page and Sergey Brin at Stanford (1996) to rank web pages. It models the behavior of a "random surfer" who:
- Follows links with probability d (damping factor)
- Jumps to random page with probability (1-d)

**Key Insight**: A page is important if important pages link to it.

### 13.2 Application to Code

In code dependency graphs:
- **High PageRank** = Called by many important entities (core utilities, foundational types)
- **Low PageRank** = Called by few or unimportant entities (leaf functions, dead code)

### 13.3 Damping Factor Interpretation

| Damping | Interpretation | Use Case |
|---------|---------------|----------|
| 0.5 | Equal weight: topology vs random | Flat hierarchies |
| 0.85 | Standard (Google original) | General purpose |
| 0.95 | Strong topology influence | Deep hierarchies |

### 13.4 Comparison to Simple Coupling

| Metric | What It Measures | Limitation |
|--------|------------------|------------|
| Afferent Coupling | Direct dependencies | Ignores importance of callers |
| PageRank | Transitive importance | More expensive to compute |

**Example:**
```
A -> B -> C
D -> C

Simple coupling:
- C: 2 inbound (A via B, D)

PageRank:
- C gets higher score if A is important
- Accounts for transitive influence
```

### 13.5 Convergence Properties

- **Guaranteed**: PageRank always converges for strongly connected graphs
- **Rate**: Typically converges in 20-50 iterations
- **Delta**: Convergence threshold of 0.0001 is standard

---

## 14. Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-30 | Claude Code | Initial specification |

---

## 15. Sign-Off

**Specification Status**: DRAFT - Ready for Review
**Implementation Status**: NOT STARTED
**Target Release**: Parseltongue v1.5.0
**Estimated Effort**: 6-8 hours (algorithm + handler + tests + integration)

---

**END OF SPECIFICATION**
