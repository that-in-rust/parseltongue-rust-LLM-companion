//! Coupling metrics endpoint handler
//!
//! # 4-Word Naming: coupling_metrics_handler
//!
//! Endpoint: GET /coupling-metrics-afferent-efferent
//!
//! Calculates afferent coupling (Ca), efferent coupling (Ce), and stability index
//! for all entities in the dependency graph. Enables architectural analysis
//! and identification of coupling hotspots.
//!
//! # Metrics
//! - Ca (Afferent): Count of incoming edges (entities that depend ON this)
//! - Ce (Efferent): Count of outgoing edges (entities this depends ON)
//! - Stability Index: Ce / (Ca + Ce) - Range 0.0 (stable) to 1.0 (unstable)

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Query parameters for coupling metrics endpoint
///
/// # 4-Word Name: CouplingMetricsQueryParamsStruct
#[derive(Debug, Deserialize, Default)]
pub struct CouplingMetricsQueryParamsStruct {
    /// Optional entity key filter (exact or prefix)
    pub entity: Option<String>,

    /// Sort field: "total", "afferent", "efferent"
    pub sort_by: Option<String>,

    /// Maximum results to return (1-1000)
    pub limit: Option<usize>,
}

/// Single coupling metric entry
///
/// # 4-Word Name: CouplingMetricEntryPayload
#[derive(Debug, Serialize, Clone)]
pub struct CouplingMetricEntryPayload {
    pub rank: usize,
    pub entity_key: String,
    pub afferent_coupling: usize,
    pub efferent_coupling: usize,
    pub total_coupling: usize,
    pub stability_index: f64,
    pub abstractness_hint: String,
}

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

/// Coupling metrics response payload
///
/// # 4-Word Name: CouplingMetricsResponsePayloadStruct
#[derive(Debug, Serialize)]
pub struct CouplingMetricsResponsePayloadStruct {
    pub success: bool,
    pub endpoint: String,
    pub data: CouplingMetricsDataPayload,
    pub tokens: usize,
}

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
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Validate and normalize parameters
    let sort_by = params.sort_by.as_deref().unwrap_or("total");
    let limit = params.limit.unwrap_or(100);

    // Validate sort_by parameter
    if !matches!(sort_by, "total" | "afferent" | "efferent") {
        return (
            StatusCode::BAD_REQUEST,
            Json(CouplingMetricsErrorResponseStruct {
                success: false,
                error: "Invalid sort_by parameter. Must be one of: total, afferent, efferent".to_string(),
                endpoint: "/coupling-metrics-afferent-efferent".to_string(),
                tokens: 42,
            }),
        ).into_response();
    }

    // Validate limit parameter
    if limit < 1 || limit > 1000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(CouplingMetricsErrorResponseStruct {
                success: false,
                error: "Limit must be between 1 and 1000".to_string(),
                endpoint: "/coupling-metrics-afferent-efferent".to_string(),
                tokens: 38,
            }),
        ).into_response();
    }

    // Calculate coupling metrics
    let metrics = calculate_entity_coupling_metrics_all(
        &state,
        params.entity.as_deref(),
        sort_by,
        limit,
    ).await;

    let total_analyzed = metrics.len();
    let metrics_returned = metrics.len().min(limit);

    // Estimate tokens
    let tokens = estimate_token_count_for_response(&metrics);

    (
        StatusCode::OK,
        Json(CouplingMetricsResponsePayloadStruct {
            success: true,
            endpoint: "/coupling-metrics-afferent-efferent".to_string(),
            data: CouplingMetricsDataPayload {
                total_entities_analyzed: total_analyzed,
                metrics_returned,
                sort_by: sort_by.to_string(),
                metrics,
            },
            tokens,
        }),
    ).into_response()
}

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
pub async fn calculate_entity_coupling_metrics_all(
    state: &SharedApplicationStateContainer,
    entity_filter: Option<&str>,
    sort_by: &str,
    limit: usize,
) -> Vec<CouplingMetricEntryPayload> {
    let db_guard = state.database_storage_connection_arc.read().await;
    let storage = match db_guard.as_ref() {
        Some(s) => s,
        None => return Vec::new(),
    };

    // Query all dependency edges
    let query = "?[from_key, to_key] := *DependencyEdges{from_key, to_key}";
    let edges = match storage.raw_query(query).await {
        Ok(result) => result.rows,
        Err(_) => return Vec::new(),
    };

    // Build coupling maps: entity -> (afferent_count, efferent_count)
    let mut coupling_map: HashMap<String, (usize, usize)> = HashMap::new();

    for row in edges {
        if row.len() >= 2 {
            let from_key = extract_string_value(&row[0]).unwrap_or_default();
            let to_key = extract_string_value(&row[1]).unwrap_or_default();

            if from_key.is_empty() || to_key.is_empty() {
                continue;
            }

            // Increment Ce (efferent) for from_key - it depends ON to_key
            let from_entry = coupling_map.entry(from_key.clone()).or_insert((0, 0));
            from_entry.1 += 1;

            // Increment Ca (afferent) for to_key - from_key depends ON it
            let to_entry = coupling_map.entry(to_key.clone()).or_insert((0, 0));
            to_entry.0 += 1;
        }
    }

    // Convert to metrics and apply filter
    let mut metrics: Vec<CouplingMetricEntryPayload> = coupling_map
        .into_iter()
        .filter(|(entity_key, _)| {
            // Apply entity filter if specified
            match entity_filter {
                Some(filter) => {
                    entity_key == filter || entity_key.starts_with(filter)
                }
                None => true,
            }
        })
        .map(|(entity_key, (ca, ce))| {
            let total = ca + ce;
            let stability_index = if total > 0 {
                ce as f64 / total as f64
            } else {
                0.0
            };
            let abstractness_hint = classify_abstractness_hint(ca, ce);

            CouplingMetricEntryPayload {
                rank: 0, // Will be set after sorting
                entity_key,
                afferent_coupling: ca,
                efferent_coupling: ce,
                total_coupling: total,
                stability_index,
                abstractness_hint: abstractness_hint.to_string(),
            }
        })
        .collect();

    // Sort by specified field in descending order
    match sort_by {
        "afferent" => {
            metrics.sort_by(|a, b| b.afferent_coupling.cmp(&a.afferent_coupling));
        }
        "efferent" => {
            metrics.sort_by(|a, b| b.efferent_coupling.cmp(&a.efferent_coupling));
        }
        _ => {
            // Default: sort by total
            metrics.sort_by(|a, b| b.total_coupling.cmp(&a.total_coupling));
        }
    }

    // Apply limit
    metrics.truncate(limit);

    // Assign ranks (1-based)
    for (idx, metric) in metrics.iter_mut().enumerate() {
        metric.rank = idx + 1;
    }

    metrics
}

/// Classify abstractness hint based on coupling
///
/// # 4-Word Name: classify_abstractness_hint_coupling
fn classify_abstractness_hint(ca: usize, ce: usize) -> &'static str {
    match (ca, ce) {
        (0, 0) => "isolated",
        (_, 0) => "concrete",  // Only incoming (leaf/terminal)
        (0, _) => "abstract",  // Only outgoing (root/source)
        (_, _) => "mixed",     // Both incoming and outgoing
    }
}

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

/// Extract string value from CozoDB DataValue
///
/// # 4-Word Name: extract_string_value_helper
fn extract_string_value(value: &cozo::DataValue) -> Option<String> {
    match value {
        cozo::DataValue::Str(s) => Some(s.to_string()),
        cozo::DataValue::Null => None,
        _ => Some(format!("{:?}", value)),
    }
}
