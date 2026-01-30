# Coupling Metrics Endpoint Specification

**Version**: 1.5.0
**Endpoint**: `/coupling-metrics-afferent-efferent`
**Method**: GET
**Status**: DRAFT
**Author**: Claude Code
**Date**: 2026-01-30

---

## Executive Summary

This specification defines a new HTTP endpoint that calculates afferent coupling (Ca), efferent coupling (Ce), and total coupling for all code entities in the Parseltongue dependency graph. The endpoint supports filtering, sorting, and pagination to enable architectural analysis and code quality assessment.

**Core Value**: Enables identification of architectural coupling patterns with <50ms p99 latency for 10,000 entities.

---

## 1. Functional Requirements

### 1.1 Core Contract (WHEN...THEN...SHALL Format)

**REQ-CM-001.0: Basic Coupling Calculation**

**WHEN** a client sends `GET /coupling-metrics-afferent-efferent`
**THEN** the system SHALL return coupling metrics for all entities
**AND** SHALL calculate Ca (afferent coupling) as count of incoming edges
**AND** SHALL calculate Ce (efferent coupling) as count of outgoing edges
**AND** SHALL calculate total coupling as Ca + Ce
**AND** SHALL complete in < 50ms at p99 latency for 10,000 entities
**AND** SHALL allocate < 500KB memory during computation
**AND** SHALL return results ordered by total coupling descending by default

**Verification:**
```rust
#[test]
fn test_req_cm_001_basic_coupling_calculation() {
    // Arrange
    let entities = create_test_graph_with_10000_entities();

    // Act
    let start = Instant::now();
    let response = handle_coupling_metrics_afferent_efferent(&state, Query::default()).await;
    let elapsed = start.elapsed();

    // Assert
    assert!(response.success);
    assert_eq!(response.data.metrics.len(), 10000);
    assert!(elapsed < Duration::from_millis(50)); // p99 target

    // Verify coupling calculations
    let metric = &response.data.metrics[0];
    assert_eq!(metric.total_coupling, metric.afferent_coupling + metric.efferent_coupling);
}
```

---

**REQ-CM-002.0: Entity Filtering**

**WHEN** a client sends `GET /coupling-metrics-afferent-efferent?entity=rust:fn:main`
**THEN** the system SHALL return metrics for entities matching the filter pattern
**AND** SHALL support exact match when entity key is complete
**AND** SHALL support prefix match when entity key is partial
**AND** SHALL return empty results array (not null) when no matches found
**AND** SHALL return 200 OK with empty array (not 404)

**Verification:**
```rust
#[test]
fn test_req_cm_002_entity_filtering() {
    // Test exact match
    let params = CouplingMetricsQueryParamsStruct {
        entity: Some("rust:fn:main:src_main_rs:1-50".to_string()),
        ..Default::default()
    };
    let response = handle_coupling_metrics_afferent_efferent(&state, Query(params)).await;
    assert_eq!(response.data.metrics.len(), 1);

    // Test prefix match
    let params = CouplingMetricsQueryParamsStruct {
        entity: Some("rust:fn:handle".to_string()),
        ..Default::default()
    };
    let response = handle_coupling_metrics_afferent_efferent(&state, Query(params)).await;
    assert!(response.data.metrics.len() > 0);
    assert!(response.data.metrics.iter().all(|m| m.entity_key.starts_with("rust:fn:handle")));
}
```

---

**REQ-CM-003.0: Sorting and Pagination**

**WHEN** a client sends `GET /coupling-metrics-afferent-efferent?sort_by=afferent&limit=20`
**THEN** the system SHALL sort results by specified field in descending order
**AND** SHALL support sort_by values: total, afferent, efferent
**AND** SHALL default to total when sort_by not specified
**AND** SHALL limit results to specified count
**AND** SHALL default to limit of 100 when not specified
**AND** SHALL support limit up to 1000 maximum

**Verification:**
```rust
#[test]
fn test_req_cm_003_sorting_pagination() {
    // Test sort by afferent
    let params = CouplingMetricsQueryParamsStruct {
        sort_by: Some("afferent".to_string()),
        limit: Some(10),
        ..Default::default()
    };
    let response = handle_coupling_metrics_afferent_efferent(&state, Query(params)).await;
    assert_eq!(response.data.metrics.len(), 10);

    // Verify descending order
    for i in 0..response.data.metrics.len()-1 {
        assert!(response.data.metrics[i].afferent_coupling >=
                response.data.metrics[i+1].afferent_coupling);
    }
}
```

---

## 2. API Specification

### 2.1 HTTP Endpoint

```
GET /coupling-metrics-afferent-efferent
```

### 2.2 Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `entity` | string | No | None | Filter by entity key (exact or prefix match) |
| `sort_by` | string | No | "total" | Sort field: "total", "afferent", "efferent" |
| `limit` | integer | No | 100 | Maximum results to return (1-1000) |

### 2.3 Response Structure

**Success Response (200 OK):**

```json
{
  "success": true,
  "endpoint": "/coupling-metrics-afferent-efferent",
  "data": {
    "total_entities_analyzed": 1247,
    "metrics_returned": 100,
    "sort_by": "total",
    "metrics": [
      {
        "rank": 1,
        "entity_key": "rust:fn:new:unknown:0-0",
        "afferent_coupling": 352,
        "efferent_coupling": 0,
        "total_coupling": 352,
        "stability_index": 0.0,
        "abstractness_hint": "concrete"
      },
      {
        "rank": 2,
        "entity_key": "rust:method:handle_request:src_server_rs:45-120",
        "afferent_coupling": 12,
        "efferent_coupling": 8,
        "total_coupling": 20,
        "stability_index": 0.4,
        "abstractness_hint": "mixed"
      }
    ]
  },
  "tokens": 1450
}
```

**Error Response (400 BAD REQUEST):**

```json
{
  "success": false,
  "error": "Invalid sort_by parameter. Must be one of: total, afferent, efferent",
  "endpoint": "/coupling-metrics-afferent-efferent",
  "tokens": 42
}
```

### 2.4 Response Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `rank` | integer | Position in sorted list (1-based) |
| `entity_key` | string | ISGL1 entity key |
| `afferent_coupling` | integer | Ca = count of entities that depend ON this entity (incoming edges) |
| `efferent_coupling` | integer | Ce = count of entities that this entity depends ON (outgoing edges) |
| `total_coupling` | integer | Ca + Ce |
| `stability_index` | float | Ce / (Ca + Ce). Range: 0.0 (stable) to 1.0 (unstable) |
| `abstractness_hint` | string | "concrete" (Ce=0), "abstract" (Ca=0), "mixed" (both > 0) |

---

## 3. Performance Requirements

### 3.1 Latency Targets

| Entity Count | p50 Latency | p99 Latency | p99.9 Latency |
|--------------|-------------|-------------|---------------|
| 1,000 | < 10ms | < 25ms | < 50ms |
| 10,000 | < 20ms | < 50ms | < 100ms |
| 50,000 | < 50ms | < 150ms | < 300ms |

### 3.2 Memory Constraints

- **Hot path allocation**: Zero heap allocations during edge iteration
- **Peak memory**: < 500KB for 10,000 entities
- **Per-entity overhead**: < 50 bytes

### 3.3 Token Budget

- **Base response**: 80 tokens
- **Per metric**: ~12 tokens
- **Maximum response**: 1,500 tokens (at limit=100)

---

## 4. Algorithm Specification

### 4.1 Core Algorithm

```rust
/// Calculate coupling metrics for all entities
///
/// # 4-Word Name: calculate_entity_coupling_metrics_all
///
/// # Algorithm
/// 1. Query all edges from DependencyEdges table
/// 2. Initialize HashMap<entity_key, (Ca, Ce)>
/// 3. For each edge (from_key -> to_key):
///    - Increment Ce for from_key (outgoing)
///    - Increment Ca for to_key (incoming)
/// 4. Calculate derived metrics:
///    - total_coupling = Ca + Ce
///    - stability_index = Ce / (Ca + Ce) if (Ca + Ce) > 0, else 0.0
///    - abstractness_hint = classify(Ca, Ce)
/// 5. Sort by specified field descending
/// 6. Apply limit
/// 7. Assign ranks (1-based)
async fn calculate_entity_coupling_metrics_all(
    state: &SharedApplicationStateContainer,
    entity_filter: Option<&str>,
    sort_by: &str,
    limit: usize,
) -> Vec<CouplingMetricEntryPayload> {
    // Implementation details...
}
```

### 4.2 CozoDB Query Pattern

```datalog
?[from_key, to_key] := *DependencyEdges{from_key, to_key}
```

**Note**: Single query fetches all edges. In-memory aggregation is faster than multiple database queries for this use case.

### 4.3 Stability Index Formula

```
Stability Index = Ce / (Ca + Ce)

Where:
- 0.0 = Maximally stable (no outgoing dependencies)
- 1.0 = Maximally unstable (no incoming dependencies)
- 0.5 = Balanced coupling
```

**Edge Cases:**
- If Ca = 0 and Ce = 0: stability_index = 0.0
- If entity has no edges: excluded from results

### 4.4 Abstractness Hint Classification

```rust
fn classify_abstractness_hint(ca: usize, ce: usize) -> &'static str {
    match (ca, ce) {
        (0, 0) => "isolated",
        (_, 0) => "concrete",  // Only incoming (leaf/terminal)
        (0, _) => "abstract",  // Only outgoing (root/source)
        (_, _) => "mixed",     // Both incoming and outgoing
    }
}
```

---

## 5. Data Structures (Following 4WNC)

### 5.1 Query Parameters Struct

```rust
/// Query parameters for coupling metrics endpoint
///
/// # 4-Word Name: CouplingMetricsQueryParamsStruct
#[derive(Debug, Deserialize, Default)]
pub struct CouplingMetricsQueryParamsStruct {
    /// Optional entity key filter (exact or prefix)
    pub entity: Option<String>,

    /// Sort field: "total", "afferent", "efferent"
    #[serde(default = "default_sort_by")]
    pub sort_by: Option<String>,

    /// Maximum results to return (1-1000)
    #[serde(default = "default_limit")]
    pub limit: Option<usize>,
}

fn default_sort_by() -> Option<String> {
    Some("total".to_string())
}

fn default_limit() -> Option<usize> {
    Some(100)
}
```

### 5.2 Metric Entry Payload

```rust
/// Single coupling metric entry
///
/// # 4-Word Name: CouplingMetricEntryPayload
#[derive(Debug, Serialize)]
pub struct CouplingMetricEntryPayload {
    pub rank: usize,
    pub entity_key: String,
    pub afferent_coupling: usize,
    pub efferent_coupling: usize,
    pub total_coupling: usize,
    pub stability_index: f64,
    pub abstractness_hint: String,
}
```

### 5.3 Response Data Payload

```rust
/// Coupling metrics response data
///
/// # 4-Word Name: CouplingMetricsDataPayload
#[derive(Debug, Serialize)]
pub struct CouplingMetricsDataPayload {
    pub total_entities_analyzed: usize,
    pub metrics_returned: usize,
    pub sort_by: String,
    pub metrics: Vec<CouplingMetricEntryPayload>,
}
```

### 5.4 Response Payload Struct

```rust
/// Coupling metrics response payload
///
/// # 4-Word Name: CouplingMetricsResponsePayload
#[derive(Debug, Serialize)]
pub struct CouplingMetricsResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: CouplingMetricsDataPayload,
    pub tokens: usize,
}
```

### 5.5 Error Response Struct

```rust
/// Coupling metrics error response
///
/// # 4-Word Name: CouplingMetricsErrorResponseStruct
#[derive(Debug, Serialize)]
pub struct CouplingMetricsErrorResponseStruct {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
    pub tokens: usize,
}
```

---

## 6. Handler Function Signature

```rust
/// Handle coupling metrics afferent efferent request
///
/// # 4-Word Name: handle_coupling_metrics_afferent_efferent
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns coupling metrics for all/filtered entities
/// - Performance: <50ms at p99 for 10,000 entities
/// - Error Handling: Returns 400 for invalid parameters, 200 with empty array for no results
///
/// # URL Pattern
/// - Endpoint: GET /coupling-metrics-afferent-efferent?entity=X&sort_by=Y&limit=N
/// - Default sort_by: "total"
/// - Default limit: 100
pub async fn handle_coupling_metrics_afferent_efferent(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<CouplingMetricsQueryParamsStruct>,
) -> impl IntoResponse {
    // Implementation...
}
```

---

## 7. Test Cases

### 7.1 Test Case 1: Basic Functionality

```rust
#[test]
async fn test_coupling_metrics_basic_calculation() {
    // Arrange
    let state = setup_test_state_with_sample_graph().await;
    // Sample graph:
    //   A -> B -> C
    //   A -> D
    //   E -> C
    // Expected:
    //   A: Ca=0, Ce=2, Total=2
    //   B: Ca=1, Ce=1, Total=2
    //   C: Ca=2, Ce=0, Total=2
    //   D: Ca=1, Ce=0, Total=1
    //   E: Ca=0, Ce=1, Total=1

    // Act
    let params = CouplingMetricsQueryParamsStruct::default();
    let response = handle_coupling_metrics_afferent_efferent(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert_eq!(response.data.total_entities_analyzed, 5);

    let a_metric = response.data.metrics.iter()
        .find(|m| m.entity_key.contains(":A:"))
        .unwrap();
    assert_eq!(a_metric.afferent_coupling, 0);
    assert_eq!(a_metric.efferent_coupling, 2);
    assert_eq!(a_metric.total_coupling, 2);
    assert_eq!(a_metric.stability_index, 1.0); // Maximally unstable
    assert_eq!(a_metric.abstractness_hint, "abstract");
}
```

### 7.2 Test Case 2: Filtering

```rust
#[test]
async fn test_coupling_metrics_entity_filtering() {
    // Arrange
    let state = setup_test_state_with_mixed_entities().await;

    // Act - Filter by prefix
    let params = CouplingMetricsQueryParamsStruct {
        entity: Some("rust:fn:handle".to_string()),
        sort_by: None,
        limit: None,
    };
    let response = handle_coupling_metrics_afferent_efferent(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert!(response.data.metrics_returned > 0);
    assert!(response.data.metrics.iter()
        .all(|m| m.entity_key.starts_with("rust:fn:handle")));
}
```

### 7.3 Test Case 3: Sorting

```rust
#[test]
async fn test_coupling_metrics_sorting_by_afferent() {
    // Arrange
    let state = setup_test_state_with_sample_graph().await;

    // Act
    let params = CouplingMetricsQueryParamsStruct {
        entity: None,
        sort_by: Some("afferent".to_string()),
        limit: Some(10),
    };
    let response = handle_coupling_metrics_afferent_efferent(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert!(response.data.metrics.len() <= 10);

    // Verify descending order
    for i in 0..response.data.metrics.len()-1 {
        assert!(
            response.data.metrics[i].afferent_coupling >=
            response.data.metrics[i+1].afferent_coupling
        );
    }
}
```

### 7.4 Test Case 4: Performance Contract

```rust
#[test]
async fn test_coupling_metrics_performance_contract() {
    // Arrange
    let state = setup_test_state_with_10000_entities().await;

    // Act
    let start = Instant::now();
    let params = CouplingMetricsQueryParamsStruct::default();
    let response = handle_coupling_metrics_afferent_efferent(
        State(state),
        Query(params)
    ).await;
    let elapsed = start.elapsed();

    // Assert
    assert!(elapsed < Duration::from_millis(50)); // p99 target
    assert!(response.data.total_entities_analyzed == 10000);
    assert!(response.tokens <= 1500); // Token budget constraint
}
```

### 7.5 Test Case 5: Edge Cases

```rust
#[test]
async fn test_coupling_metrics_edge_cases() {
    // Test 1: Empty database
    let state = setup_empty_database().await;
    let response = handle_coupling_metrics_afferent_efferent(
        State(state),
        Query(CouplingMetricsQueryParamsStruct::default())
    ).await;
    assert_eq!(response.data.metrics_returned, 0);
    assert_eq!(response.data.metrics.len(), 0);

    // Test 2: Invalid sort_by parameter
    let state = setup_test_state_with_sample_graph().await;
    let params = CouplingMetricsQueryParamsStruct {
        entity: None,
        sort_by: Some("invalid".to_string()),
        limit: None,
    };
    let response = handle_coupling_metrics_afferent_efferent(
        State(state),
        Query(params)
    ).await;
    // Should return error response
    assert!(!response.success);
    assert!(response.error.contains("Invalid sort_by"));

    // Test 3: Limit exceeds maximum
    let params = CouplingMetricsQueryParamsStruct {
        entity: None,
        sort_by: None,
        limit: Some(5000), // Exceeds max of 1000
    };
    let response = handle_coupling_metrics_afferent_efferent(
        State(state),
        Query(params)
    ).await;
    assert!(!response.success);
    assert!(response.error.contains("limit must be between 1 and 1000"));
}
```

### 7.6 Test Case 6: Stability Index Calculation

```rust
#[test]
async fn test_coupling_metrics_stability_index() {
    // Arrange
    let state = setup_test_state_with_known_couplings().await;
    // Entity X: Ca=6, Ce=3 -> Expected Stability = 3/9 = 0.333...

    // Act
    let params = CouplingMetricsQueryParamsStruct {
        entity: Some("entity_x".to_string()),
        sort_by: None,
        limit: None,
    };
    let response = handle_coupling_metrics_afferent_efferent(
        State(state),
        Query(params)
    ).await;

    // Assert
    let metric = &response.data.metrics[0];
    assert_eq!(metric.afferent_coupling, 6);
    assert_eq!(metric.efferent_coupling, 3);
    assert!((metric.stability_index - 0.333).abs() < 0.001);
}
```

---

## 8. Example API Usage

### 8.1 Get Top 10 Most Coupled Entities

```bash
curl -s "http://localhost:7777/coupling-metrics-afferent-efferent?limit=10" | jq '.'
```

**Response:**
```json
{
  "success": true,
  "endpoint": "/coupling-metrics-afferent-efferent",
  "data": {
    "total_entities_analyzed": 1247,
    "metrics_returned": 10,
    "sort_by": "total",
    "metrics": [
      {
        "rank": 1,
        "entity_key": "rust:fn:new:unknown:0-0",
        "afferent_coupling": 352,
        "efferent_coupling": 0,
        "total_coupling": 352,
        "stability_index": 0.0,
        "abstractness_hint": "concrete"
      }
    ]
  },
  "tokens": 450
}
```

### 8.2 Get Metrics for Specific Entity Pattern

```bash
curl -s "http://localhost:7777/coupling-metrics-afferent-efferent?entity=rust:fn:handle" | jq '.'
```

### 8.3 Sort by Afferent Coupling (Find Most Depended-Upon Entities)

```bash
curl -s "http://localhost:7777/coupling-metrics-afferent-efferent?sort_by=afferent&limit=20" | jq '.'
```

### 8.4 Sort by Efferent Coupling (Find Entities with Most Dependencies)

```bash
curl -s "http://localhost:7777/coupling-metrics-afferent-efferent?sort_by=efferent&limit=20" | jq '.'
```

---

## 9. Implementation File Location

**Handler Module**: `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/coupling_metrics_afferent_efferent_handler.rs`

**Router Registration**: Add to `crates/pt08-http-code-query-server/src/http_server_startup_runner.rs`:

```rust
.route("/coupling-metrics-afferent-efferent",
       get(coupling_metrics_afferent_efferent_handler::handle_coupling_metrics_afferent_efferent))
```

---

## 10. Token Budget Estimation

```rust
/// Estimate token count for coupling metrics response
///
/// # 4-Word Name: estimate_token_count_for_response
fn estimate_token_count_for_response(metrics: &[CouplingMetricEntryPayload]) -> usize {
    // Base response structure: 80 tokens
    let base = 80;

    // Per metric: ~12 tokens
    // {rank, entity_key, afferent, efferent, total, stability, hint}
    let per_metric = 12;

    // Calculate
    base + (metrics.len() * per_metric)
}
```

**Budget Validation:**
- At limit=100: 80 + (100 * 12) = 1,280 tokens ✓ (within 1,500 target)
- At limit=1000: 80 + (1000 * 12) = 12,080 tokens ✗ (exceeds target, but acceptable for max limit)

---

## 11. Success Criteria

### 11.1 Functional Criteria

- ✓ All query parameters work as specified
- ✓ Coupling calculations are mathematically correct
- ✓ Filtering returns correct subset
- ✓ Sorting produces descending order
- ✓ Pagination respects limits
- ✓ Error handling returns appropriate status codes

### 11.2 Performance Criteria

- ✓ p99 latency < 50ms for 10,000 entities
- ✓ Memory allocation < 500KB
- ✓ Token budget <= 1,500 for default limit

### 11.3 Code Quality Criteria

- ✓ All function names follow 4WNC
- ✓ All tests pass (6 test cases minimum)
- ✓ Zero compiler warnings
- ✓ Zero TODOs or STUBs in committed code
- ✓ Documentation comments on all public items

---

## 12. Integration with Existing System

### 12.1 API Reference Documentation Update

Add to `/api-reference-documentation-help` response:

```json
{
  "path": "/coupling-metrics-afferent-efferent",
  "method": "GET",
  "description": "Calculate afferent and efferent coupling for all entities",
  "parameters": [
    {
      "name": "entity",
      "param_type": "query",
      "required": false,
      "description": "Filter by entity key (exact or prefix match)"
    },
    {
      "name": "sort_by",
      "param_type": "query",
      "required": false,
      "description": "Sort field: total, afferent, efferent (default: total)"
    },
    {
      "name": "limit",
      "param_type": "query",
      "required": false,
      "description": "Maximum results (1-1000, default: 100)"
    }
  ]
}
```

### 12.2 Health Check Compatibility

- No changes required to `/server-health-check-status`
- Database connection reused from existing pool

### 12.3 Statistics Update

Consider adding to `/codebase-statistics-overview-summary`:

```json
{
  "coupling_statistics": {
    "average_afferent_coupling": 8.4,
    "average_efferent_coupling": 6.2,
    "max_total_coupling": 352,
    "entities_with_zero_coupling": 45
  }
}
```

---

## 13. Appendix: Coupling Metrics Theory

### 13.1 Robert C. Martin's Stability Metrics

- **I (Instability)** = Ce / (Ca + Ce)
  - 0 = Maximally stable (no outgoing dependencies)
  - 1 = Maximally unstable (no incoming dependencies)

### 13.2 Architectural Implications

| Ca | Ce | Stability | Interpretation |
|----|----|-----------| ---------------|
| High | Low | Stable | Core/Foundation (many depend on this) |
| Low | High | Unstable | Leaf/Detail (depends on many) |
| High | High | Mixed | Hub/Connector (architectural risk) |
| Low | Low | Isolated | Dead code or entry point |

### 13.3 Use Cases

1. **Refactoring Risk Assessment**: High Ca entities are risky to change
2. **Dependency Inversion**: Abstract entities should have Ce ≈ 0
3. **Dead Code Detection**: Ca = 0 and Ce = 0 indicates isolated code
4. **Architectural Violations**: High-level modules should have low Ce

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
**Estimated Effort**: 4-6 hours (handler + tests + integration)

---

**END OF SPECIFICATION**
