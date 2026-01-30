# Instability Index Calculation Endpoint Specification

**Version**: 1.5.0
**Endpoint**: `/instability-index-calculation-view`
**Method**: GET
**Status**: DRAFT
**Author**: Claude Code
**Date**: 2026-01-30

---

## Executive Summary

This specification defines an HTTP endpoint that calculates Robert C. Martin's Instability Index (I) for all code entities in the Parseltongue dependency graph. The endpoint extends the coupling metrics with architectural zone classification (Zone of Pain, Zone of Uselessness, Main Sequence) to enable architectural quality assessment.

**Core Value**: Instant architectural analysis (<10ms for 10k entities) by reusing cached coupling data, enabling identification of architectural violations with zero additional database queries.

**Instability Formula**: I = Ce / (Ca + Ce)
- I = 0.0: Maximally stable (only incoming dependencies)
- I = 1.0: Maximally unstable (only outgoing dependencies)
- I = 0.5: Balanced coupling

---

## 1. Functional Requirements

### 1.1 Core Contract (WHEN...THEN...SHALL Format)

**REQ-II-001.0: Instability Index Calculation**

**WHEN** a client sends `GET /instability-index-calculation-view`
**THEN** the system SHALL calculate Instability Index I = Ce / (Ca + Ce) for all entities
**AND** SHALL reuse coupling metrics data (Ca, Ce) from existing endpoint
**AND** SHALL classify each entity into architectural zones
**AND** SHALL complete in < 10ms at p99 latency for 10,000 entities
**AND** SHALL allocate < 200KB memory during computation
**AND** SHALL return results ordered by instability index descending by default
**AND** SHALL return 200 OK with empty array when no entities exist

**Verification:**
```rust
#[test]
fn test_req_ii_001_instability_index_calculation() {
    // Arrange
    let entities = create_test_graph_with_10000_entities();
    // Entity A: Ca=10, Ce=5 -> I = 5/(10+5) = 0.333
    // Entity B: Ca=0, Ce=8 -> I = 8/(0+8) = 1.0
    // Entity C: Ca=20, Ce=0 -> I = 0/(20+0) = 0.0

    // Act
    let start = Instant::now();
    let response = handle_instability_index_calculation_view(&state, Query::default()).await;
    let elapsed = start.elapsed();

    // Assert
    assert!(response.success);
    assert_eq!(response.data.metrics.len(), 10000);
    assert!(elapsed < Duration::from_millis(10)); // p99 target

    // Verify formula: I = Ce / (Ca + Ce)
    let metric_a = find_entity(&response.data.metrics, "entity_a");
    assert_eq!(metric_a.afferent_coupling, 10);
    assert_eq!(metric_a.efferent_coupling, 5);
    assert!((metric_a.instability_index - 0.333).abs() < 0.001);

    let metric_b = find_entity(&response.data.metrics, "entity_b");
    assert_eq!(metric_b.instability_index, 1.0); // Maximally unstable

    let metric_c = find_entity(&response.data.metrics, "entity_c");
    assert_eq!(metric_c.instability_index, 0.0); // Maximally stable
}
```

---

**REQ-II-002.0: Architectural Zone Classification**

**WHEN** a client requests instability index metrics
**THEN** the system SHALL classify each entity into one of four zones:
- **"zone_of_pain"**: Stable + Concrete (low I + low A) - hard to change, much responsibility
- **"zone_of_uselessness"**: Unstable + Abstract (high I + high A) - useless abstractions
- **"main_sequence"**: Balanced (I + A ≈ 1.0) - ideal architectural position
- **"unknown"**: Missing abstractness data or isolated entities

**AND** SHALL calculate distance from main sequence: D = |A + I - 1|
**AND** SHALL flag entities with D > 0.3 as architectural violations

**Verification:**
```rust
#[test]
fn test_req_ii_002_zone_classification() {
    // Arrange - Create entities in different zones
    let state = setup_test_state_with_zoned_entities().await;
    // Pain: Ca=50, Ce=5, I=0.09 (stable, concrete)
    // Useless: Ca=2, Ce=20, I=0.91 (unstable, abstract)
    // Main: Ca=10, Ce=10, I=0.5 (balanced)

    // Act
    let response = handle_instability_index_calculation_view(&state, Query::default()).await;

    // Assert
    let pain = find_entity(&response.data.metrics, "entity_pain");
    assert_eq!(pain.zone_classification, "zone_of_pain");
    assert!(pain.instability_index < 0.2);

    let useless = find_entity(&response.data.metrics, "entity_useless");
    assert_eq!(useless.zone_classification, "zone_of_uselessness");
    assert!(useless.instability_index > 0.8);

    let main_seq = find_entity(&response.data.metrics, "entity_main");
    assert_eq!(main_seq.zone_classification, "main_sequence");
    assert!((main_seq.instability_index - 0.5).abs() < 0.1);
}
```

---

**REQ-II-003.0: Threshold Filtering**

**WHEN** a client sends `GET /instability-index-calculation-view?threshold=0.7`
**THEN** the system SHALL return only entities with I >= threshold
**AND** SHALL support threshold range 0.0 to 1.0 inclusive
**AND** SHALL default to threshold=0.0 (all entities)
**AND** SHALL return 400 BAD REQUEST if threshold < 0.0 or > 1.0

**Verification:**
```rust
#[test]
fn test_req_ii_003_threshold_filtering() {
    // Arrange
    let state = setup_test_state_with_varied_instability().await;

    // Act - Filter high instability only
    let params = InstabilityIndexQueryParamsStruct {
        threshold: Some(0.7),
        ..Default::default()
    };
    let response = handle_instability_index_calculation_view(&state, Query(params)).await;

    // Assert
    assert!(response.data.metrics.iter().all(|m| m.instability_index >= 0.7));

    // Test invalid threshold
    let params = InstabilityIndexQueryParamsStruct {
        threshold: Some(1.5), // Invalid
        ..Default::default()
    };
    let response = handle_instability_index_calculation_view(&state, Query(params)).await;
    assert!(!response.success);
    assert!(response.error.contains("threshold must be between 0.0 and 1.0"));
}
```

---

**REQ-II-004.0: Stability Filter Categories**

**WHEN** a client sends `GET /instability-index-calculation-view?filter=stable`
**THEN** the system SHALL return entities based on filter:
- **"stable"**: I <= 0.3 (low instability)
- **"unstable"**: I >= 0.7 (high instability)
- **"balanced"**: 0.3 < I < 0.7
- **"all"**: No filtering (default)

**AND** SHALL return 400 BAD REQUEST for invalid filter values

**Verification:**
```rust
#[test]
fn test_req_ii_004_stability_filter() {
    // Arrange
    let state = setup_test_state_with_varied_instability().await;

    // Act - Filter stable entities only
    let params = InstabilityIndexQueryParamsStruct {
        filter: Some("stable".to_string()),
        ..Default::default()
    };
    let response = handle_instability_index_calculation_view(&state, Query(params)).await;

    // Assert
    assert!(response.data.metrics.iter().all(|m| m.instability_index <= 0.3));

    // Test unstable filter
    let params = InstabilityIndexQueryParamsStruct {
        filter: Some("unstable".to_string()),
        ..Default::default()
    };
    let response = handle_instability_index_calculation_view(&state, Query(params)).await;
    assert!(response.data.metrics.iter().all(|m| m.instability_index >= 0.7));

    // Test balanced filter
    let params = InstabilityIndexQueryParamsStruct {
        filter: Some("balanced".to_string()),
        ..Default::default()
    };
    let response = handle_instability_index_calculation_view(&state, Query(params)).await;
    assert!(response.data.metrics.iter().all(|m| {
        m.instability_index > 0.3 && m.instability_index < 0.7
    }));
}
```

---

## 2. API Specification

### 2.1 HTTP Endpoint

```
GET /instability-index-calculation-view
```

### 2.2 Query Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `threshold` | float | No | 0.0 | Minimum instability index (0.0-1.0) |
| `filter` | string | No | "all" | Stability category: "stable", "unstable", "balanced", "all" |
| `limit` | integer | No | 100 | Maximum results to return (1-1000) |
| `entity` | string | No | None | Filter by entity key (exact or prefix match) |

### 2.3 Response Structure

**Success Response (200 OK):**

```json
{
  "success": true,
  "endpoint": "/instability-index-calculation-view",
  "data": {
    "total_entities_analyzed": 1247,
    "metrics_returned": 100,
    "filter_applied": "all",
    "threshold_applied": 0.0,
    "metrics": [
      {
        "rank": 1,
        "entity_key": "rust:fn:main:src_main_rs:1-50",
        "afferent_coupling": 0,
        "efferent_coupling": 8,
        "instability_index": 1.0,
        "stability_category": "unstable",
        "zone_classification": "zone_of_uselessness",
        "distance_from_main_sequence": 0.2,
        "architectural_violation": false
      },
      {
        "rank": 2,
        "entity_key": "rust:fn:new:unknown:0-0",
        "afferent_coupling": 352,
        "efferent_coupling": 0,
        "instability_index": 0.0,
        "stability_category": "stable",
        "zone_classification": "zone_of_pain",
        "distance_from_main_sequence": 0.15,
        "architectural_violation": false
      },
      {
        "rank": 3,
        "entity_key": "rust:method:process_request:src_handler_rs:100-200",
        "afferent_coupling": 5,
        "efferent_coupling": 5,
        "instability_index": 0.5,
        "stability_category": "balanced",
        "zone_classification": "main_sequence",
        "distance_from_main_sequence": 0.0,
        "architectural_violation": false
      }
    ]
  },
  "tokens": 1200
}
```

**Error Response (400 BAD REQUEST):**

```json
{
  "success": false,
  "error": "Invalid threshold parameter. Must be between 0.0 and 1.0",
  "endpoint": "/instability-index-calculation-view",
  "tokens": 38
}
```

### 2.4 Response Field Definitions

| Field | Type | Description |
|-------|------|-------------|
| `rank` | integer | Position in sorted list (1-based) |
| `entity_key` | string | ISGL1 entity key |
| `afferent_coupling` | integer | Ca = incoming dependencies |
| `efferent_coupling` | integer | Ce = outgoing dependencies |
| `instability_index` | float | I = Ce / (Ca + Ce). Range: 0.0 (stable) to 1.0 (unstable) |
| `stability_category` | string | "stable" (I≤0.3), "balanced" (0.3<I<0.7), "unstable" (I≥0.7) |
| `zone_classification` | string | Architectural zone: "zone_of_pain", "zone_of_uselessness", "main_sequence", "unknown" |
| `distance_from_main_sequence` | float | D = \|A + I - 1\|. Smaller is better. |
| `architectural_violation` | boolean | true if D > 0.3 (architectural problem) |

---

## 3. Performance Requirements

### 3.1 Latency Targets

| Entity Count | p50 Latency | p99 Latency | p99.9 Latency |
|--------------|-------------|-------------|---------------|
| 1,000 | < 2ms | < 5ms | < 10ms |
| 10,000 | < 5ms | < 10ms | < 20ms |
| 50,000 | < 15ms | < 30ms | < 50ms |

**Rationale**: Instability calculation reuses coupling data already computed by `/coupling-metrics-afferent-efferent`. No additional database queries required - pure in-memory computation.

### 3.2 Memory Constraints

- **Hot path allocation**: Zero heap allocations during index calculation
- **Peak memory**: < 200KB for 10,000 entities
- **Per-entity overhead**: < 20 bytes (just the index float + classification)

### 3.3 Token Budget

- **Base response**: 95 tokens
- **Per metric**: ~11 tokens
- **Maximum response**: 1,200 tokens (at limit=100)

---

## 4. Algorithm Specification

### 4.1 Core Algorithm

```rust
/// Calculate instability index metrics for all entities
///
/// # 4-Word Name: calculate_instability_index_for_entities
///
/// # Algorithm
/// 1. Reuse coupling metrics from calculate_entity_coupling_metrics_all()
///    - Already have Ca and Ce for each entity
/// 2. For each entity:
///    a. Calculate I = Ce / (Ca + Ce)
///       - If Ca + Ce = 0: I = 0.0 (isolated entity)
///    b. Classify stability: stable (I≤0.3), balanced (0.3<I<0.7), unstable (I≥0.7)
///    c. Classify zone (requires abstractness hint from coupling metrics)
///    d. Calculate D = |A + I - 1| (distance from main sequence)
///    e. Flag architectural violation if D > 0.3
/// 3. Filter by threshold (I >= threshold)
/// 4. Filter by category (stable/unstable/balanced/all)
/// 5. Sort by instability index descending
/// 6. Apply limit
/// 7. Assign ranks (1-based)
///
/// # Performance
/// - Reuses existing coupling data (zero DB queries)
/// - O(n log n) for sorting only
/// - < 10ms for 10,000 entities
async fn calculate_instability_index_for_entities(
    state: &SharedApplicationStateContainer,
    threshold: f64,
    filter: &str,
    entity_filter: Option<&str>,
    limit: usize,
) -> Result<Vec<InstabilityMetricPayload>, HandlerError> {
    // Implementation details...
}
```

### 4.2 Instability Index Formula

```
I = Ce / (Ca + Ce)

Where:
- Ce = Efferent coupling (outgoing dependencies)
- Ca = Afferent coupling (incoming dependencies)
- I ∈ [0.0, 1.0]

Special Cases:
- Ca = 0, Ce = 0: I = 0.0 (isolated entity, default to stable)
- Ca > 0, Ce = 0: I = 0.0 (maximally stable - concrete implementations)
- Ca = 0, Ce > 0: I = 1.0 (maximally unstable - abstract interfaces)
```

### 4.3 Stability Category Classification

```rust
/// Classify stability category based on instability index
///
/// # 4-Word Name: classify_stability_category_from_index
fn classify_stability_category_from_index(instability: f64) -> &'static str {
    match instability {
        i if i <= 0.3 => "stable",      // Low instability = high stability
        i if i >= 0.7 => "unstable",    // High instability = low stability
        _ => "balanced",                 // Middle ground
    }
}
```

### 4.4 Architectural Zone Classification

```rust
/// Classify architectural zone based on instability and abstractness
///
/// # 4-Word Name: classify_architectural_zone_from_metrics
///
/// # Zones (from Robert C. Martin)
/// - Zone of Pain: Stable + Concrete (I≈0, A≈0)
///   - Hard to change, much responsibility
///   - Example: God classes, utility singletons
///
/// - Zone of Uselessness: Unstable + Abstract (I≈1, A≈1)
///   - Useless abstractions with no callers
///   - Example: Over-engineered interfaces
///
/// - Main Sequence: I + A ≈ 1
///   - Ideal balance between stability and abstractness
///   - Example: Well-designed modules
fn classify_architectural_zone_from_metrics(
    instability: f64,
    abstractness_hint: &str,
) -> String {
    // Estimate abstractness from hint
    let abstractness = match abstractness_hint {
        "abstract" => 1.0,   // Only outgoing (Ca=0)
        "concrete" => 0.0,   // Only incoming (Ce=0)
        "mixed" => 0.5,      // Both present
        "isolated" => 0.0,   // Default
        _ => return "unknown".to_string(),
    };

    // Calculate distance from main sequence
    let distance = (abstractness + instability - 1.0).abs();

    // Classify zone
    if instability < 0.3 && abstractness < 0.3 {
        "zone_of_pain".to_string()
    } else if instability > 0.7 && abstractness > 0.7 {
        "zone_of_uselessness".to_string()
    } else if distance < 0.3 {
        "main_sequence".to_string()
    } else {
        "unknown".to_string()
    }
}
```

### 4.5 Distance from Main Sequence

```
D = |A + I - 1|

Where:
- A = Abstractness (0.0 = concrete, 1.0 = abstract)
- I = Instability (0.0 = stable, 1.0 = unstable)
- D ∈ [0.0, 1.0]

Interpretation:
- D = 0.0: On the main sequence (ideal)
- D < 0.3: Close to main sequence (acceptable)
- D > 0.3: Architectural violation (needs refactoring)

Main Sequence Line: A + I = 1
- Low instability requires high abstractness
- High instability should have low abstractness
```

---

## 5. Data Structures (Following 4WNC)

### 5.1 Query Parameters Struct

```rust
/// Query parameters for instability index endpoint
///
/// # 4-Word Name: InstabilityIndexQueryParamsStruct
#[derive(Debug, Deserialize, Default)]
pub struct InstabilityIndexQueryParamsStruct {
    /// Minimum instability index threshold (0.0-1.0)
    #[serde(default)]
    pub threshold: Option<f64>,

    /// Stability filter: "stable", "unstable", "balanced", "all"
    #[serde(default = "default_filter_all")]
    pub filter: Option<String>,

    /// Maximum results to return (1-1000)
    #[serde(default = "default_limit_100")]
    pub limit: Option<usize>,

    /// Optional entity key filter (exact or prefix)
    pub entity: Option<String>,
}

fn default_filter_all() -> Option<String> {
    Some("all".to_string())
}

fn default_limit_100() -> Option<usize> {
    Some(100)
}
```

### 5.2 Instability Metric Payload

```rust
/// Single instability index metric entry
///
/// # 4-Word Name: InstabilityMetricPayload
#[derive(Debug, Serialize)]
pub struct InstabilityMetricPayload {
    pub rank: usize,
    pub entity_key: String,
    pub afferent_coupling: usize,
    pub efferent_coupling: usize,
    pub instability_index: f64,
    pub stability_category: String,
    pub zone_classification: String,
    pub distance_from_main_sequence: f64,
    pub architectural_violation: bool,
}
```

### 5.3 Response Data Payload

```rust
/// Instability index response data payload
///
/// # 4-Word Name: InstabilityIndexDataPayload
#[derive(Debug, Serialize)]
pub struct InstabilityIndexDataPayload {
    pub total_entities_analyzed: usize,
    pub metrics_returned: usize,
    pub filter_applied: String,
    pub threshold_applied: f64,
    pub metrics: Vec<InstabilityMetricPayload>,
}
```

### 5.4 Response Payload Struct

```rust
/// Instability index response payload struct
///
/// # 4-Word Name: InstabilityIndexResponsePayload
#[derive(Debug, Serialize)]
pub struct InstabilityIndexResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: InstabilityIndexDataPayload,
    pub tokens: usize,
}
```

### 5.5 Error Response Struct

```rust
/// Instability index error response struct
///
/// # 4-Word Name: InstabilityIndexErrorResponseStruct
#[derive(Debug, Serialize)]
pub struct InstabilityIndexErrorResponseStruct {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
    pub tokens: usize,
}
```

---

## 6. Handler Function Signature

```rust
/// Handle instability index calculation view request
///
/// # 4-Word Name: handle_instability_index_calculation_view
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns instability metrics for all/filtered entities
/// - Performance: <10ms at p99 for 10,000 entities (reuses coupling data)
/// - Error Handling: Returns 400 for invalid parameters, 200 with empty array for no results
///
/// # URL Pattern
/// - Endpoint: GET /instability-index-calculation-view?threshold=X&filter=Y&limit=N
/// - Default threshold: 0.0
/// - Default filter: "all"
/// - Default limit: 100
pub async fn handle_instability_index_calculation_view(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<InstabilityIndexQueryParamsStruct>,
) -> impl IntoResponse {
    // Implementation...
}
```

---

## 7. Test Cases

### 7.1 Test Case 1: Basic Instability Calculation

```rust
#[test]
async fn test_instability_index_basic_calculation() {
    // Arrange
    let state = setup_test_state_with_sample_graph().await;
    // Sample graph:
    //   A -> B -> C
    //   A -> D
    //   E -> C
    // Expected:
    //   A: Ca=0, Ce=2 -> I = 2/2 = 1.0 (maximally unstable)
    //   B: Ca=1, Ce=1 -> I = 1/2 = 0.5 (balanced)
    //   C: Ca=2, Ce=0 -> I = 0/2 = 0.0 (maximally stable)
    //   D: Ca=1, Ce=0 -> I = 0/1 = 0.0 (maximally stable)
    //   E: Ca=0, Ce=1 -> I = 1/1 = 1.0 (maximally unstable)

    // Act
    let params = InstabilityIndexQueryParamsStruct::default();
    let response = handle_instability_index_calculation_view(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert_eq!(response.data.total_entities_analyzed, 5);

    let a_metric = find_entity(&response.data.metrics, ":A:");
    assert_eq!(a_metric.afferent_coupling, 0);
    assert_eq!(a_metric.efferent_coupling, 2);
    assert_eq!(a_metric.instability_index, 1.0);
    assert_eq!(a_metric.stability_category, "unstable");

    let b_metric = find_entity(&response.data.metrics, ":B:");
    assert_eq!(b_metric.instability_index, 0.5);
    assert_eq!(b_metric.stability_category, "balanced");

    let c_metric = find_entity(&response.data.metrics, ":C:");
    assert_eq!(c_metric.instability_index, 0.0);
    assert_eq!(c_metric.stability_category, "stable");
}
```

### 7.2 Test Case 2: Threshold Filtering

```rust
#[test]
async fn test_instability_index_threshold_filtering() {
    // Arrange
    let state = setup_test_state_with_varied_instability().await;

    // Act - Filter high instability only (≥0.7)
    let params = InstabilityIndexQueryParamsStruct {
        threshold: Some(0.7),
        filter: None,
        limit: None,
        entity: None,
    };
    let response = handle_instability_index_calculation_view(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert!(response.data.metrics.iter().all(|m| m.instability_index >= 0.7));
    assert_eq!(response.data.threshold_applied, 0.7);

    // Verify all returned entities are unstable
    assert!(response.data.metrics.iter()
        .all(|m| m.stability_category == "unstable"));
}
```

### 7.3 Test Case 3: Stability Category Filter

```rust
#[test]
async fn test_instability_index_stability_filter() {
    // Arrange
    let state = setup_test_state_with_varied_instability().await;

    // Act - Filter stable entities only
    let params = InstabilityIndexQueryParamsStruct {
        threshold: None,
        filter: Some("stable".to_string()),
        limit: None,
        entity: None,
    };
    let response = handle_instability_index_calculation_view(
        State(state),
        Query(params)
    ).await;

    // Assert
    assert!(response.data.metrics.iter().all(|m| m.instability_index <= 0.3));
    assert!(response.data.metrics.iter().all(|m| m.stability_category == "stable"));
    assert_eq!(response.data.filter_applied, "stable");
}
```

### 7.4 Test Case 4: Zone Classification

```rust
#[test]
async fn test_instability_index_zone_classification() {
    // Arrange
    let state = setup_test_state_with_zoned_entities().await;
    // Pain: Ca=50, Ce=5, I=0.09 (stable + concrete)
    // Useless: Ca=2, Ce=20, I=0.91 (unstable + abstract)
    // Main: Ca=10, Ce=10, I=0.5 (balanced)

    // Act
    let params = InstabilityIndexQueryParamsStruct::default();
    let response = handle_instability_index_calculation_view(
        State(state),
        Query(params)
    ).await;

    // Assert
    let pain = find_entity(&response.data.metrics, "entity_pain");
    assert_eq!(pain.zone_classification, "zone_of_pain");
    assert!(pain.instability_index < 0.2);
    assert!(pain.distance_from_main_sequence > 0.5); // Far from main sequence

    let useless = find_entity(&response.data.metrics, "entity_useless");
    assert_eq!(useless.zone_classification, "zone_of_uselessness");
    assert!(useless.instability_index > 0.8);
    assert!(useless.distance_from_main_sequence > 0.5);

    let main_seq = find_entity(&response.data.metrics, "entity_main");
    assert_eq!(main_seq.zone_classification, "main_sequence");
    assert!((main_seq.instability_index - 0.5).abs() < 0.2);
    assert!(main_seq.distance_from_main_sequence < 0.3); // Close to main sequence
}
```

### 7.5 Test Case 5: Performance Contract

```rust
#[test]
async fn test_instability_index_performance_contract() {
    // Arrange
    let state = setup_test_state_with_10000_entities().await;

    // Act
    let start = Instant::now();
    let params = InstabilityIndexQueryParamsStruct::default();
    let response = handle_instability_index_calculation_view(
        State(state),
        Query(params)
    ).await;
    let elapsed = start.elapsed();

    // Assert
    assert!(elapsed < Duration::from_millis(10)); // p99 target
    assert!(response.data.total_entities_analyzed == 10000);
    assert!(response.tokens <= 1200); // Token budget constraint

    // Verify zero additional DB queries (reuses coupling data)
    // This would be measured via instrumentation
}
```

### 7.6 Test Case 6: Edge Cases

```rust
#[test]
async fn test_instability_index_edge_cases() {
    // Test 1: Empty database
    let state = setup_empty_database().await;
    let response = handle_instability_index_calculation_view(
        State(state),
        Query(InstabilityIndexQueryParamsStruct::default())
    ).await;
    assert_eq!(response.data.metrics_returned, 0);
    assert_eq!(response.data.metrics.len(), 0);

    // Test 2: Invalid threshold (negative)
    let state = setup_test_state_with_sample_graph().await;
    let params = InstabilityIndexQueryParamsStruct {
        threshold: Some(-0.5),
        ..Default::default()
    };
    let response = handle_instability_index_calculation_view(
        State(state),
        Query(params)
    ).await;
    assert!(!response.success);
    assert!(response.error.contains("threshold must be between 0.0 and 1.0"));

    // Test 3: Invalid threshold (>1.0)
    let params = InstabilityIndexQueryParamsStruct {
        threshold: Some(1.5),
        ..Default::default()
    };
    let response = handle_instability_index_calculation_view(
        State(state),
        Query(params)
    ).await;
    assert!(!response.success);

    // Test 4: Invalid filter category
    let params = InstabilityIndexQueryParamsStruct {
        filter: Some("invalid_category".to_string()),
        ..Default::default()
    };
    let response = handle_instability_index_calculation_view(
        State(state),
        Query(params)
    ).await;
    assert!(!response.success);
    assert!(response.error.contains("Invalid filter"));

    // Test 5: Isolated entity (Ca=0, Ce=0)
    let state = setup_test_state_with_isolated_entity().await;
    let response = handle_instability_index_calculation_view(
        State(state),
        Query(InstabilityIndexQueryParamsStruct::default())
    ).await;

    let isolated = find_entity(&response.data.metrics, "isolated_entity");
    assert_eq!(isolated.afferent_coupling, 0);
    assert_eq!(isolated.efferent_coupling, 0);
    assert_eq!(isolated.instability_index, 0.0); // Default to stable
}
```

---

## 8. Example API Usage

### 8.1 Get All Entities with Instability Index

```bash
curl -s "http://localhost:7779/instability-index-calculation-view" | jq '.'
```

**Response:**
```json
{
  "success": true,
  "endpoint": "/instability-index-calculation-view",
  "data": {
    "total_entities_analyzed": 1247,
    "metrics_returned": 100,
    "filter_applied": "all",
    "threshold_applied": 0.0,
    "metrics": [
      {
        "rank": 1,
        "entity_key": "rust:fn:main:src_main_rs:1-50",
        "afferent_coupling": 0,
        "efferent_coupling": 8,
        "instability_index": 1.0,
        "stability_category": "unstable",
        "zone_classification": "zone_of_uselessness",
        "distance_from_main_sequence": 0.2,
        "architectural_violation": false
      }
    ]
  },
  "tokens": 1150
}
```

### 8.2 Find Highly Unstable Entities (Architectural Risk)

```bash
curl -s "http://localhost:7779/instability-index-calculation-view?threshold=0.8&limit=20" | jq '.'
```

**Use Case**: Identify entities with many outgoing dependencies (high change risk)

### 8.3 Find Stable Entities Only

```bash
curl -s "http://localhost:7779/instability-index-calculation-view?filter=stable" | jq '.'
```

**Use Case**: Find foundation entities (few outgoing dependencies)

### 8.4 Find Balanced Entities (Main Sequence)

```bash
curl -s "http://localhost:7779/instability-index-calculation-view?filter=balanced&limit=50" | jq '.'
```

**Use Case**: Identify well-designed modules on the main sequence

### 8.5 Find Entities in Zone of Pain

```bash
curl -s "http://localhost:7779/instability-index-calculation-view?filter=stable" | \
  jq '.data.metrics[] | select(.zone_classification == "zone_of_pain")'
```

**Use Case**: Find rigid, hard-to-change entities with high responsibility

### 8.6 Find Architectural Violations

```bash
curl -s "http://localhost:7779/instability-index-calculation-view" | \
  jq '.data.metrics[] | select(.architectural_violation == true)'
```

**Use Case**: Find entities far from main sequence (D > 0.3)

---

## 9. Implementation File Location

**Handler Module**: `/Users/amuldotexe/Desktop/OSS202601/parseltongue-dependency-graph-generator/crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/instability_index_calculation_handler.rs`

**Router Registration**: Add to `crates/pt08-http-code-query-server/src/http_server_startup_runner.rs`:

```rust
.route("/instability-index-calculation-view",
       get(instability_index_calculation_handler::handle_instability_index_calculation_view))
```

---

## 10. Integration with Existing System

### 10.1 Reuse Coupling Metrics Data

```rust
// Key optimization: Reuse existing coupling calculation
use crate::http_endpoint_handler_modules::coupling_metrics_handler::calculate_entity_coupling_metrics_all;

async fn calculate_instability_index_for_entities(
    state: &SharedApplicationStateContainer,
    // ... params
) -> Result<Vec<InstabilityMetricPayload>, HandlerError> {
    // Step 1: Get coupling metrics (already optimized, zero additional queries)
    let coupling_metrics = calculate_entity_coupling_metrics_all(
        state,
        entity_filter,
        "total", // Sort doesn't matter, we'll re-sort
        usize::MAX, // Get all entities
    ).await?;

    // Step 2: Transform to instability metrics (pure computation)
    let instability_metrics: Vec<_> = coupling_metrics
        .into_iter()
        .map(|c| {
            let ca = c.afferent_coupling;
            let ce = c.efferent_coupling;

            // Calculate instability: I = Ce / (Ca + Ce)
            let instability = if ca + ce == 0 {
                0.0 // Isolated entity defaults to stable
            } else {
                ce as f64 / (ca + ce) as f64
            };

            // Classify stability category
            let stability_category = classify_stability_category_from_index(instability);

            // Classify zone
            let zone_classification = classify_architectural_zone_from_metrics(
                instability,
                &c.abstractness_hint,
            );

            // Calculate distance from main sequence
            let abstractness = estimate_abstractness_from_hint(&c.abstractness_hint);
            let distance = (abstractness + instability - 1.0).abs();
            let architectural_violation = distance > 0.3;

            InstabilityMetricPayload {
                rank: 0, // Will be set after sorting
                entity_key: c.entity_key,
                afferent_coupling: ca,
                efferent_coupling: ce,
                instability_index: instability,
                stability_category: stability_category.to_string(),
                zone_classification,
                distance_from_main_sequence: distance,
                architectural_violation,
            }
        })
        .collect();

    // Step 3: Apply filters and sorting...
}
```

### 10.2 API Reference Documentation Update

Add to `/api-reference-documentation-help` response:

```json
{
  "path": "/instability-index-calculation-view",
  "method": "GET",
  "description": "Calculate Robert C. Martin's Instability Index (I = Ce / (Ca + Ce)) with architectural zone classification",
  "parameters": [
    {
      "name": "threshold",
      "param_type": "query",
      "required": false,
      "description": "Minimum instability index (0.0-1.0, default: 0.0)"
    },
    {
      "name": "filter",
      "param_type": "query",
      "required": false,
      "description": "Stability category: stable, unstable, balanced, all (default: all)"
    },
    {
      "name": "limit",
      "param_type": "query",
      "required": false,
      "description": "Maximum results (1-1000, default: 100)"
    },
    {
      "name": "entity",
      "param_type": "query",
      "required": false,
      "description": "Filter by entity key (exact or prefix match)"
    }
  ],
  "response_fields": [
    {
      "name": "instability_index",
      "type": "float",
      "description": "I = Ce / (Ca + Ce). Range: 0.0 (stable) to 1.0 (unstable)"
    },
    {
      "name": "zone_classification",
      "type": "string",
      "description": "Architectural zone: zone_of_pain, zone_of_uselessness, main_sequence, unknown"
    },
    {
      "name": "distance_from_main_sequence",
      "type": "float",
      "description": "D = |A + I - 1|. Smaller is better. D > 0.3 indicates architectural violation"
    }
  ]
}
```

### 10.3 Statistics Integration

Consider adding to `/codebase-statistics-overview-summary`:

```json
{
  "instability_statistics": {
    "average_instability_index": 0.52,
    "entities_in_zone_of_pain": 12,
    "entities_in_zone_of_uselessness": 8,
    "entities_on_main_sequence": 247,
    "architectural_violations_count": 20,
    "most_unstable_entity": {
      "entity_key": "rust:fn:main:src_main_rs:1-50",
      "instability_index": 1.0
    },
    "most_stable_entity": {
      "entity_key": "rust:fn:new:unknown:0-0",
      "instability_index": 0.0
    }
  }
}
```

---

## 11. Token Budget Estimation

```rust
/// Estimate token count for instability index response
///
/// # 4-Word Name: estimate_token_count_for_response
fn estimate_token_count_for_response(metrics: &[InstabilityMetricPayload]) -> usize {
    // Base response structure: 95 tokens
    let base = 95;

    // Per metric: ~11 tokens
    // {rank, entity_key, ca, ce, I, category, zone, distance, violation}
    let per_metric = 11;

    // Calculate
    base + (metrics.len() * per_metric)
}
```

**Budget Validation:**
- At limit=100: 95 + (100 * 11) = 1,195 tokens ✓ (within 1,200 target)
- At limit=1000: 95 + (1000 * 11) = 11,095 tokens (acceptable for max limit)

---

## 12. Architectural Theory Reference

### 12.1 Robert C. Martin's Metrics

**Instability (I)**: Measures how likely a module is to be affected by changes in its dependencies.

```
I = Ce / (Ca + Ce)

Where:
- Ce = Efferent Coupling (outgoing dependencies)
- Ca = Afferent Coupling (incoming dependencies)
```

**Abstractness (A)**: Ratio of abstract entities to total entities in a module (estimated via coupling pattern).

**Main Sequence**: The ideal relationship between stability and abstractness.

```
D = |A + I - 1|

Where D is the normalized distance from the main sequence.
```

### 12.2 Architectural Zones

```
        Abstractness (A)
            1.0 |
                |  Zone of
                |  Uselessness
                |       /
                |      /
            0.5 | Main/Sequence
                |    /
                |   /
                |  Zone of Pain
            0.0 |________________
                0.0    0.5    1.0
                   Instability (I)
```

**Zone of Pain** (I≈0, A≈0):
- Stable and concrete
- Many incoming dependencies, few outgoing
- Hard to change due to high responsibility
- Examples: Core utilities, database models, god classes

**Zone of Uselessness** (I≈1, A≈1):
- Unstable and abstract
- Few incoming dependencies, many outgoing
- Over-engineered abstractions with no users
- Examples: Unused interfaces, speculative frameworks

**Main Sequence** (A + I ≈ 1):
- Ideal balance
- High-level modules: stable + abstract (I≈0, A≈1)
- Low-level modules: unstable + concrete (I≈1, A≈0)
- Examples: Well-designed layers, clean architecture

### 12.3 Use Cases

| Use Case | Query | Interpretation |
|----------|-------|----------------|
| **Find Refactoring Risks** | `filter=stable` | High Ca entities are risky to change |
| **Find Leaf Nodes** | `filter=unstable` | High Ce entities are implementation details |
| **Find Architectural Violations** | `architectural_violation=true` | Entities far from main sequence |
| **Find Dead Code** | `threshold=0.0&limit=all` | Isolated entities (Ca=0, Ce=0) |
| **Find God Classes** | Zone of Pain | Stable + concrete = too much responsibility |
| **Find Over-Engineering** | Zone of Uselessness | Unstable + abstract = unused abstractions |

---

## 13. Success Criteria

### 13.1 Functional Criteria

- ✓ Instability index calculated correctly (I = Ce / (Ca + Ce))
- ✓ Threshold filtering works (I >= threshold)
- ✓ Category filtering works (stable/unstable/balanced/all)
- ✓ Zone classification accurate (pain/uselessness/main_sequence)
- ✓ Distance from main sequence calculated (D = |A + I - 1|)
- ✓ Architectural violations flagged (D > 0.3)
- ✓ Error handling for invalid parameters (400 responses)
- ✓ Empty result handling (200 with empty array)

### 13.2 Performance Criteria

- ✓ p99 latency < 10ms for 10,000 entities
- ✓ Reuses coupling data (zero additional DB queries)
- ✓ Memory allocation < 200KB
- ✓ Token budget <= 1,200 for default limit

### 13.3 Code Quality Criteria

- ✓ All function names follow 4WNC
- ✓ All tests pass (6 test cases minimum)
- ✓ Zero compiler warnings
- ✓ Zero TODOs or STUBs in committed code
- ✓ Documentation comments on all public items

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
**Estimated Effort**: 3-5 hours (handler + tests + integration)
**Dependencies**: Requires `/coupling-metrics-afferent-efferent` endpoint (v1.5.0)

---

**END OF SPECIFICATION**
