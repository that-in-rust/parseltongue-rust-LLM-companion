//! Complexity hotspots ranking endpoint handler
//!
//! # 4-Word Naming: complexity_hotspots_ranking_handler
//!
//! Endpoint: GET /complexity-hotspots-ranking-view?top=N
//!
//! Ranks entities by their coupling complexity - how many dependencies
//! they have (both inbound and outbound). High-coupling entities are
//! architectural risks.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Query parameters for complexity hotspots
///
/// # 4-Word Name: ComplexityHotspotsQueryParamsStruct
#[derive(Debug, Deserialize)]
pub struct ComplexityHotspotsQueryParamsStruct {
    /// Number of top hotspots to return (default: 10)
    pub top: Option<usize>,
}

/// Single hotspot entry in the ranking
///
/// # 4-Word Name: ComplexityHotspotEntryPayload
#[derive(Debug, Serialize)]
pub struct ComplexityHotspotEntryPayload {
    pub rank: usize,
    pub entity_key: String,
    pub inbound_count: usize,
    pub outbound_count: usize,
    pub total_coupling: usize,
}

/// Complexity hotspots response data
///
/// # 4-Word Name: ComplexityHotspotsDataPayload
#[derive(Debug, Serialize)]
pub struct ComplexityHotspotsDataPayload {
    pub total_entities_analyzed: usize,
    pub top_requested: usize,
    pub hotspots: Vec<ComplexityHotspotEntryPayload>,
}

/// Complexity hotspots response payload
///
/// # 4-Word Name: ComplexityHotspotsResponsePayload
#[derive(Debug, Serialize)]
pub struct ComplexityHotspotsResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: ComplexityHotspotsDataPayload,
    pub tokens: usize,
}

/// Handle complexity hotspots ranking view request
///
/// # 4-Word Name: handle_complexity_hotspots_ranking_view
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns entities ranked by total coupling
/// - Performance: O(E) where E is number of edges
/// - Error Handling: Returns empty hotspots if no edges
///
/// # Algorithm
/// 1. Count inbound edges per entity (edges where entity is to_key)
/// 2. Count outbound edges per entity (edges where entity is from_key)
/// 3. Calculate total_coupling = inbound + outbound
/// 4. Sort descending by total_coupling
/// 5. Return top N entities
pub async fn handle_complexity_hotspots_ranking_view(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<ComplexityHotspotsQueryParamsStruct>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    let top = params.top.unwrap_or(10);

    // Calculate coupling scores for all entities
    let hotspots = calculate_entity_coupling_scores(&state, top).await;

    let total_entities = hotspots.len();

    // Estimate tokens
    let tokens = 80 + (hotspots.len() * 25);

    (
        StatusCode::OK,
        Json(ComplexityHotspotsResponsePayload {
            success: true,
            endpoint: "/complexity-hotspots-ranking-view".to_string(),
            data: ComplexityHotspotsDataPayload {
                total_entities_analyzed: total_entities,
                top_requested: top,
                hotspots,
            },
            tokens,
        }),
    ).into_response()
}

/// Calculate entity coupling scores from edges
///
/// # 4-Word Name: calculate_entity_coupling_scores
///
/// Counts inbound and outbound edges for each entity,
/// calculates total coupling, and returns sorted list.
async fn calculate_entity_coupling_scores(
    state: &SharedApplicationStateContainer,
    top: usize,
) -> Vec<ComplexityHotspotEntryPayload> {
    let db_guard = state.database_storage_connection_arc.read().await;
    let storage = match db_guard.as_ref() {
        Some(s) => s,
        None => return Vec::new(),
    };

    // Query all edges
    let query = "?[from_key, to_key] := *DependencyEdges{from_key, to_key}";
    let edges = match storage.raw_query(query).await {
        Ok(result) => result.rows,
        Err(_) => return Vec::new(),
    };

    // Count inbound and outbound edges per entity
    let mut inbound_counts: HashMap<String, usize> = HashMap::new();
    let mut outbound_counts: HashMap<String, usize> = HashMap::new();

    for row in edges {
        if row.len() >= 2 {
            let from_key = extract_string_value_helper(&row[0]).unwrap_or_default();
            let to_key = extract_string_value_helper(&row[1]).unwrap_or_default();

            // Outbound from from_key
            *outbound_counts.entry(from_key.clone()).or_insert(0) += 1;
            // Inbound to to_key
            *inbound_counts.entry(to_key.clone()).or_insert(0) += 1;

            // Ensure both entities exist in both maps
            inbound_counts.entry(from_key).or_insert(0);
            outbound_counts.entry(to_key).or_insert(0);
        }
    }

    // Collect all unique entities
    let mut all_entities: Vec<String> = inbound_counts.keys().cloned().collect();
    for key in outbound_counts.keys() {
        if !all_entities.contains(key) {
            all_entities.push(key.clone());
        }
    }

    // Calculate coupling and build hotspots list
    let mut hotspots: Vec<(String, usize, usize, usize)> = all_entities
        .into_iter()
        .map(|entity| {
            let inbound = *inbound_counts.get(&entity).unwrap_or(&0);
            let outbound = *outbound_counts.get(&entity).unwrap_or(&0);
            let total = inbound + outbound;
            (entity, inbound, outbound, total)
        })
        .collect();

    // Sort by total coupling descending
    hotspots.sort_by(|a, b| b.3.cmp(&a.3));

    // Take top N and assign ranks
    hotspots
        .into_iter()
        .take(top)
        .enumerate()
        .map(|(idx, (entity, inbound, outbound, total))| {
            ComplexityHotspotEntryPayload {
                rank: idx + 1,
                entity_key: entity,
                inbound_count: inbound,
                outbound_count: outbound,
                total_coupling: total,
            }
        })
        .collect()
}

/// Extract string value from CozoDB DataValue
///
/// # 4-Word Name: extract_string_value_helper
fn extract_string_value_helper(value: &cozo::DataValue) -> Option<String> {
    match value {
        cozo::DataValue::Str(s) => Some(s.to_string()),
        cozo::DataValue::Null => None,
        _ => Some(format!("{:?}", value)),
    }
}
