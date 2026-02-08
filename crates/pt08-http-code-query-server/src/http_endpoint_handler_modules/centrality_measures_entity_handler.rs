//! Centrality measures entity ranking endpoint handler
//!
//! # 4-Word Naming: centrality_measures_entity_handler
//!
//! Endpoint: GET /centrality-measures-entity-ranking
//!
//! Computes PageRank or Betweenness centrality scores to identify
//! the most important entities in the dependency graph.

use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use parseltongue_core::graph_analysis::{
    AdjacencyListGraphRepresentation,
    compute_pagerank_centrality_scores,
    compute_betweenness_centrality_scores,
};

/// Query parameters for centrality analysis
///
/// # 4-Word Name: CentralityAnalysisQueryParameters
#[derive(Debug, Deserialize)]
pub struct CentralityAnalysisQueryParameters {
    pub method: Option<String>,
    pub top: Option<usize>,
}

/// Single entity centrality score data
///
/// # 4-Word Name: EntityCentralityScoreData
#[derive(Debug, Serialize)]
pub struct EntityCentralityScoreData {
    pub entity: String,
    pub score: f64,
    pub rank: usize,
}

/// Centrality measures response data payload
///
/// # 4-Word Name: CentralityMeasuresDataPayload
#[derive(Debug, Serialize)]
pub struct CentralityMeasuresDataPayload {
    pub method: String,
    pub entities: Vec<EntityCentralityScoreData>,
}

/// Centrality measures response payload
///
/// # 4-Word Name: CentralityMeasuresResponsePayload
#[derive(Debug, Serialize)]
pub struct CentralityMeasuresResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: CentralityMeasuresDataPayload,
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

/// Handle centrality measures entity ranking request
///
/// # 4-Word Name: handle_centrality_measures_entity_ranking
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns centrality scores for entities
/// - Performance: O(V * E) for PageRank, O(V * E^2) for betweenness
/// - Error Handling: Returns error JSON if database unavailable
pub async fn handle_centrality_measures_entity_ranking(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<CentralityAnalysisQueryParameters>,
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
                        endpoint: "/centrality-measures-entity-ranking".to_string(),
                        error: "Database not connected".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    }; // Lock released here

    // Build graph from database
    let graph = match build_graph_from_database_edges(&storage).await {
        Ok(g) => g,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponsePayloadStructure {
                    success: false,
                    endpoint: "/centrality-measures-entity-ranking".to_string(),
                    error: e,
                }),
            )
                .into_response()
        }
    };

    // Determine method (default: pagerank)
    let method = params
        .method
        .as_deref()
        .unwrap_or("pagerank")
        .to_lowercase();

    // Compute centrality scores
    let scores = match method.as_str() {
        "betweenness" => compute_betweenness_centrality_scores(&graph),
        _ => {
            // PageRank defaults: damping=0.85, max_iterations=100, tolerance=1e-6
            compute_pagerank_centrality_scores(&graph, 0.85, 100, 1e-6)
        }
    };

    // Convert to sorted list
    let mut entity_scores: Vec<(String, f64)> = scores.into_iter().collect();
    entity_scores.sort_by(|a, b| {
        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Apply top filter if specified
    if let Some(top_n) = params.top {
        entity_scores.truncate(top_n);
    }

    // Format results with rank
    let entity_data: Vec<EntityCentralityScoreData> = entity_scores
        .into_iter()
        .enumerate()
        .map(|(idx, (entity, score))| EntityCentralityScoreData {
            entity,
            score,
            rank: idx + 1,
        })
        .collect();

    // Estimate tokens
    let tokens = 50 + (entity_data.len() * 30);

    (
        StatusCode::OK,
        Json(CentralityMeasuresResponsePayload {
            success: true,
            endpoint: "/centrality-measures-entity-ranking".to_string(),
            data: CentralityMeasuresDataPayload {
                method,
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
) -> Result<AdjacencyListGraphRepresentation, String> {
    let query = "?[from_key, to_key, edge_type] := *DependencyEdges{from_key, to_key, edge_type}";
    let result = storage
        .raw_query(query)
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
