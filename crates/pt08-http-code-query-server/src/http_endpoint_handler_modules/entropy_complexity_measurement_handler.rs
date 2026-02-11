//! Entropy complexity measurement scores endpoint handler
//!
//! # 4-Word Naming: entropy_complexity_measurement_handler
//!
//! Endpoint: GET /entropy-complexity-measurement-scores
//!
//! Calculates Shannon entropy for each entity's dependency patterns
//! to measure structural complexity and diversity.

use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::scope_filter_utilities_module::parse_scope_build_filter_clause;
use parseltongue_core::graph_analysis::{
    AdjacencyListGraphRepresentation,
    compute_all_entity_entropy,
    classify_entropy_complexity_level,
    EntropyComplexity,
};

/// Query parameters for entropy analysis
///
/// # 4-Word Name: EntropyAnalysisQueryParameters
#[derive(Debug, Deserialize)]
pub struct EntropyAnalysisQueryParameters {
    pub entity: Option<String>,
    /// Filter by folder scope (e.g., "crates||parseltongue-core")
    pub scope: Option<String>,
}

/// Single entity entropy score data
///
/// # 4-Word Name: EntityEntropyScoreData
#[derive(Debug, Serialize)]
pub struct EntityEntropyScoreData {
    pub entity: String,
    pub entropy: f64,
    pub complexity: String,
}

/// Entropy complexity response data payload
///
/// # 4-Word Name: EntropyComplexityDataPayload
#[derive(Debug, Serialize)]
pub struct EntropyComplexityDataPayload {
    pub entities: Vec<EntityEntropyScoreData>,
}

/// Entropy complexity response payload
///
/// # 4-Word Name: EntropyComplexityResponsePayload
#[derive(Debug, Serialize)]
pub struct EntropyComplexityResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: EntropyComplexityDataPayload,
    pub tokens: usize,
}

/// Error response payload structure
///
/// # 4-Word Name: ErrorResponsePayloadStructure
#[derive(Debug, Serialize)]
pub struct ErrorResponsePayloadStructure {
    pub success: bool,
    pub endpoint: String,
    pub error: String,
}

/// Handle entropy complexity measurement scores request
///
/// # 4-Word Name: handle_entropy_complexity_measurement_scores
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns entropy scores for entities
/// - Performance: O(V * E) for computing entropy
/// - Error Handling: Returns error JSON if database unavailable
pub async fn handle_entropy_complexity_measurement_scores(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<EntropyAnalysisQueryParameters>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Clone Arc inside RwLock scope, release lock
    let storage = {
        let db_guard = state.database_storage_connection_arc.read().await;
        match db_guard.as_ref() {
            Some(s) => s.clone(),
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponsePayloadStructure {
                        success: false,
                        endpoint: "/entropy-complexity-measurement-scores".to_string(),
                        error: "Database not connected".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    }; // Lock released here

    // Build graph from database
    let graph = match build_graph_from_database_edges(&storage, &params.scope).await {
        Ok(g) => g,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponsePayloadStructure {
                    success: false,
                    endpoint: "/entropy-complexity-measurement-scores".to_string(),
                    error: e,
                }),
            )
                .into_response()
        }
    };

    // Compute entropy for all entities
    let entropy_map = compute_all_entity_entropy(&graph);

    // Filter and format results
    let mut entity_data: Vec<EntityEntropyScoreData> = Vec::new();

    for (entity, &entropy) in entropy_map.iter() {
        // Apply entity filter if specified
        if let Some(ref entity_filter) = params.entity {
            if !entity.contains(entity_filter) {
                continue;
            }
        }

        let complexity = classify_entropy_complexity_level(entropy);
        let complexity_str = match complexity {
            EntropyComplexity::Low => "LOW",
            EntropyComplexity::Moderate => "MODERATE",
            EntropyComplexity::High => "HIGH",
        };

        entity_data.push(EntityEntropyScoreData {
            entity: entity.clone(),
            entropy,
            complexity: complexity_str.to_string(),
        });
    }

    // Sort by entropy descending
    entity_data.sort_by(|a, b| {
        b.entropy
            .partial_cmp(&a.entropy)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Estimate tokens
    let tokens = 50 + (entity_data.len() * 25);

    (
        StatusCode::OK,
        Json(EntropyComplexityResponsePayload {
            success: true,
            endpoint: "/entropy-complexity-measurement-scores".to_string(),
            data: EntropyComplexityDataPayload {
                entities: entity_data,
            },
            tokens,
        }),
    )
        .into_response()
}

/// Build graph from database edges query
///
/// # 4-Word Name: build_graph_from_database_edges
async fn build_graph_from_database_edges(
    storage: &Arc<parseltongue_core::storage::CozoDbStorage>,
    scope_filter: &Option<String>,
) -> Result<AdjacencyListGraphRepresentation, String> {
    // Build scope filter clause
    let scope_clause = parse_scope_build_filter_clause(scope_filter);
    let scope_join = if scope_clause.is_empty() {
        String::new()
    } else {
        format!(", *CodeGraph{{ISGL1_key: from_key, root_subfolder_L1, root_subfolder_L2}}{}", scope_clause)
    };

    let query = format!("?[from_key, to_key, edge_type] := *DependencyEdges{{from_key, to_key, edge_type}}{}", scope_join);
    let result = storage
        .raw_query(&query)
        .await
        .map_err(|e| format!("Database query failed: {}", e))?;

    let mut edges: Vec<(String, String, String)> = Vec::new();
    for row in &result.rows {
        if row.len() >= 3 {
            let from = extract_string_from_datavalue(&row[0]);
            let to = extract_string_from_datavalue(&row[1]);
            let edge_type = extract_string_from_datavalue(&row[2]);
            edges.push((from, to, edge_type));
        }
    }

    Ok(AdjacencyListGraphRepresentation::build_from_dependency_edges(&edges))
}

/// Extract string from CozoDB DataValue
///
/// # 4-Word Name: extract_string_from_datavalue
fn extract_string_from_datavalue(value: &cozo::DataValue) -> String {
    match value {
        cozo::DataValue::Str(s) => s.to_string(),
        _ => format!("{:?}", value),
    }
}
