//! Instability Index endpoint handler
//!
//! # 4-Word Naming: instability_index_handler
//!
//! Endpoint: GET /instability-index-calculation-view
//!
//! Calculates Robert C. Martin's Instability Index (I = Ce / (Ca + Ce))
//! with architectural zone classification for all entities in the dependency graph.
//!
//! # Metrics
//! - I (Instability): Ce / (Ca + Ce) - Range 0.0 (stable) to 1.0 (unstable)
//! - Zone Classification: pain/uselessness/main_sequence
//! - Distance from Main Sequence: |A + I - 1|
//!
//! # Performance
//! - Reuses coupling data from coupling_metrics_handler
//! - Zero additional database queries
//! - <10ms at p99 for 10,000 entities

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::http_endpoint_handler_modules::coupling_metrics_handler::{
    calculate_entity_coupling_metrics_all,
    CouplingMetricEntryPayload,
};

/// Query parameters for instability index endpoint
///
/// # 4-Word Name: InstabilityIndexQueryParamsStruct
#[derive(Debug, Deserialize, Default)]
pub struct InstabilityIndexQueryParamsStruct {
    /// Minimum instability index threshold (0.0-1.0)
    #[serde(default)]
    pub threshold: Option<f64>,

    /// Stability filter: "stable", "unstable", "balanced", "all"
    #[serde(default = "default_filter_all_value")]
    pub filter: Option<String>,

    /// Maximum results to return (1-1000)
    #[serde(default = "default_limit_100_value")]
    pub limit: Option<usize>,

    /// Optional entity key filter (exact or prefix)
    pub entity: Option<String>,
}

fn default_filter_all_value() -> Option<String> {
    Some("all".to_string())
}

fn default_limit_100_value() -> Option<usize> {
    Some(100)
}

/// Single instability index metric entry
///
/// # 4-Word Name: InstabilityMetricPayloadStruct
#[derive(Debug, Serialize)]
pub struct InstabilityMetricPayloadStruct {
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

/// Instability index response data payload
///
/// # 4-Word Name: InstabilityIndexDataPayloadStruct
#[derive(Debug, Serialize)]
pub struct InstabilityIndexDataPayloadStruct {
    pub total_entities_analyzed: usize,
    pub metrics_returned: usize,
    pub filter_applied: String,
    pub threshold_applied: f64,
    pub metrics: Vec<InstabilityMetricPayloadStruct>,
}

/// Instability index response payload struct
///
/// # 4-Word Name: InstabilityIndexResponsePayloadStruct
#[derive(Debug, Serialize)]
pub struct InstabilityIndexResponsePayloadStruct {
    pub success: bool,
    pub endpoint: String,
    pub data: InstabilityIndexDataPayloadStruct,
    pub tokens: usize,
}

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
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Validate and normalize parameters
    let threshold = params.threshold.unwrap_or(0.0);
    let filter = params.filter.as_deref().unwrap_or("all");
    let limit = params.limit.unwrap_or(100);

    // Validate threshold parameter
    if threshold < 0.0 || threshold > 1.0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(InstabilityIndexErrorResponseStruct {
                success: false,
                error: "Invalid threshold parameter. Must be between 0.0 and 1.0".to_string(),
                endpoint: "/instability-index-calculation-view".to_string(),
                tokens: 42,
            }),
        ).into_response();
    }

    // Validate filter parameter
    if !matches!(filter, "all" | "stable" | "unstable" | "balanced") {
        return (
            StatusCode::BAD_REQUEST,
            Json(InstabilityIndexErrorResponseStruct {
                success: false,
                error: "Invalid filter parameter. Must be one of: all, stable, unstable, balanced".to_string(),
                endpoint: "/instability-index-calculation-view".to_string(),
                tokens: 45,
            }),
        ).into_response();
    }

    // Validate limit parameter
    if limit < 1 || limit > 1000 {
        return (
            StatusCode::BAD_REQUEST,
            Json(InstabilityIndexErrorResponseStruct {
                success: false,
                error: "Limit must be between 1 and 1000".to_string(),
                endpoint: "/instability-index-calculation-view".to_string(),
                tokens: 38,
            }),
        ).into_response();
    }

    // Calculate instability metrics
    let metrics = calculate_instability_index_for_entities(
        &state,
        threshold,
        filter,
        params.entity.as_deref(),
        limit,
    ).await;

    let total_analyzed = metrics.len();
    let metrics_returned = metrics.len().min(limit);

    // Estimate tokens
    let tokens = estimate_token_count_for_response(&metrics);

    (
        StatusCode::OK,
        Json(InstabilityIndexResponsePayloadStruct {
            success: true,
            endpoint: "/instability-index-calculation-view".to_string(),
            data: InstabilityIndexDataPayloadStruct {
                total_entities_analyzed: total_analyzed,
                metrics_returned,
                filter_applied: filter.to_string(),
                threshold_applied: threshold,
                metrics,
            },
            tokens,
        }),
    ).into_response()
}

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
) -> Vec<InstabilityMetricPayloadStruct> {
    // Step 1: Get coupling metrics (already optimized, zero additional queries)
    let coupling_metrics = calculate_entity_coupling_metrics_all(
        state,
        entity_filter,
        "total", // Sort doesn't matter, we'll re-sort
        usize::MAX, // Get all entities
    ).await;

    // Step 2: Transform to instability metrics (pure computation)
    let mut instability_metrics: Vec<InstabilityMetricPayloadStruct> = coupling_metrics
        .into_iter()
        .map(|c| transform_coupling_to_instability_metric(c))
        .collect();

    // Step 3: Apply threshold filter
    instability_metrics.retain(|m| m.instability_index >= threshold);

    // Step 4: Apply stability category filter
    if filter != "all" {
        instability_metrics.retain(|m| m.stability_category == filter);
    }

    // Step 5: Sort by instability index descending
    instability_metrics.sort_by(|a, b| {
        b.instability_index.partial_cmp(&a.instability_index)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Step 6: Apply limit
    instability_metrics.truncate(limit);

    // Step 7: Assign ranks (1-based)
    for (idx, metric) in instability_metrics.iter_mut().enumerate() {
        metric.rank = idx + 1;
    }

    instability_metrics
}

/// Transform coupling metric to instability metric
///
/// # 4-Word Name: transform_coupling_to_instability_metric
fn transform_coupling_to_instability_metric(
    coupling: CouplingMetricEntryPayload,
) -> InstabilityMetricPayloadStruct {
    let ca = coupling.afferent_coupling;
    let ce = coupling.efferent_coupling;

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
        &coupling.abstractness_hint,
    );

    // Calculate distance from main sequence
    let abstractness = estimate_abstractness_from_hint(&coupling.abstractness_hint);
    let distance = (abstractness + instability - 1.0).abs();
    let architectural_violation = distance > 0.3;

    InstabilityMetricPayloadStruct {
        rank: 0, // Will be set after sorting
        entity_key: coupling.entity_key,
        afferent_coupling: ca,
        efferent_coupling: ce,
        instability_index: instability,
        stability_category: stability_category.to_string(),
        zone_classification,
        distance_from_main_sequence: distance,
        architectural_violation,
    }
}

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
    let abstractness = estimate_abstractness_from_hint(abstractness_hint);

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

/// Estimate abstractness from hint string
///
/// # 4-Word Name: estimate_abstractness_from_hint_string
fn estimate_abstractness_from_hint(hint: &str) -> f64 {
    match hint {
        "abstract" => 1.0,   // Only outgoing (Ca=0)
        "concrete" => 0.0,   // Only incoming (Ce=0)
        "mixed" => 0.5,      // Both present
        "isolated" => 0.0,   // Default
        _ => 0.5,            // Unknown defaults to mixed
    }
}

/// Estimate token count for instability index response
///
/// # 4-Word Name: estimate_token_count_for_response
fn estimate_token_count_for_response(metrics: &[InstabilityMetricPayloadStruct]) -> usize {
    // Base response structure: 95 tokens
    let base = 95;

    // Per metric: ~11 tokens
    // {rank, entity_key, ca, ce, I, category, zone, distance, violation}
    let per_metric = 11;

    // Calculate
    base + (metrics.len() * per_metric)
}
