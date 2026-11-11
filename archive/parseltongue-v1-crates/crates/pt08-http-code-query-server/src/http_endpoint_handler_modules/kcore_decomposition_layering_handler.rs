//! K-core decomposition layering analysis endpoint handler
//!
//! # 4-Word Naming: kcore_decomposition_layering_handler
//!
//! Endpoint: GET /kcore-decomposition-layering-analysis
//!
//! Performs k-core decomposition to identify core, periphery, and
//! intermediate layers of the dependency graph.

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
    kcore_decomposition_layering_algorithm,
    classify_coreness_layer_level,
    CoreLayer,
};

/// Query parameters for k-core analysis
///
/// # 4-Word Name: KcoreAnalysisQueryParameters
#[derive(Debug, Deserialize)]
pub struct KcoreAnalysisQueryParameters {
    pub k: Option<usize>,
    /// Filter by folder scope (e.g., "crates||parseltongue-core")
    pub scope: Option<String>,
}

/// Single entity coreness data
///
/// # 4-Word Name: EntityCorenessDataStructure
#[derive(Debug, Serialize)]
pub struct EntityCorenessDataStructure {
    pub entity: String,
    pub coreness: usize,
    pub layer: String,
}

/// K-core decomposition response data payload
///
/// # 4-Word Name: KcoreDecompositionDataPayload
#[derive(Debug, Serialize)]
pub struct KcoreDecompositionDataPayload {
    pub max_coreness: usize,
    pub entities: Vec<EntityCorenessDataStructure>,
}

/// K-core decomposition response payload
///
/// # 4-Word Name: KcoreDecompositionResponsePayload
#[derive(Debug, Serialize)]
pub struct KcoreDecompositionResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: KcoreDecompositionDataPayload,
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

/// Handle k-core decomposition layering analysis request
///
/// # 4-Word Name: handle_kcore_decomposition_layering_analysis
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns coreness values for all entities
/// - Performance: O(V * E) k-core decomposition algorithm
/// - Error Handling: Returns error JSON if database unavailable
pub async fn handle_kcore_decomposition_layering_analysis(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<KcoreAnalysisQueryParameters>,
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
                        endpoint: "/kcore-decomposition-layering-analysis".to_string(),
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
                    endpoint: "/kcore-decomposition-layering-analysis".to_string(),
                    error: e,
                }),
            )
                .into_response()
        }
    };

    // Compute k-core decomposition
    let coreness_map = kcore_decomposition_layering_algorithm(&graph);

    // Find max coreness
    let max_coreness = coreness_map.values().copied().max().unwrap_or(0);

    // Filter and format results
    let mut entity_data: Vec<EntityCorenessDataStructure> = Vec::new();

    for (entity, &coreness) in coreness_map.iter() {
        // Apply k filter if specified
        if let Some(min_k) = params.k {
            if coreness < min_k {
                continue;
            }
        }

        let layer = classify_coreness_layer_level(coreness);
        let layer_str = match layer {
            CoreLayer::Peripheral => "PERIPHERAL",
            CoreLayer::Mid => "MID",
            CoreLayer::Core => "CORE",
        };

        entity_data.push(EntityCorenessDataStructure {
            entity: entity.clone(),
            coreness,
            layer: layer_str.to_string(),
        });
    }

    // Sort by coreness descending
    entity_data.sort_by(|a, b| b.coreness.cmp(&a.coreness));

    // Estimate tokens
    let tokens = 50 + (entity_data.len() * 25);

    (
        StatusCode::OK,
        Json(KcoreDecompositionResponsePayload {
            success: true,
            endpoint: "/kcore-decomposition-layering-analysis".to_string(),
            data: KcoreDecompositionDataPayload {
                max_coreness,
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
