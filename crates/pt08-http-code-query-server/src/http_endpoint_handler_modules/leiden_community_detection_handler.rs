//! Leiden community detection clustering endpoint handler
//!
//! # 4-Word Naming: leiden_community_detection_handler
//!
//! Endpoint: GET /leiden-community-detection-clusters
//!
//! Uses the Leiden algorithm to detect communities (modules/clusters)
//! in the dependency graph based on modularity optimization.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Serialize;
use std::sync::Arc;

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use parseltongue_core::graph_analysis::{
    AdjacencyListGraphRepresentation,
    leiden_community_detection_clustering,
};

/// Single community data structure
///
/// # 4-Word Name: CommunityClusterDataStructure
#[derive(Debug, Serialize)]
pub struct CommunityClusterDataStructure {
    pub id: usize,
    pub size: usize,
    pub members: Vec<String>,
}

/// Leiden community detection response data
///
/// # 4-Word Name: LeidenCommunityDataPayload
#[derive(Debug, Serialize)]
pub struct LeidenCommunityDataPayload {
    pub community_count: usize,
    pub modularity: f64,
    pub communities: Vec<CommunityClusterDataStructure>,
}

/// Leiden community detection response payload
///
/// # 4-Word Name: LeidenCommunityResponsePayload
#[derive(Debug, Serialize)]
pub struct LeidenCommunityResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: LeidenCommunityDataPayload,
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

/// Handle Leiden community detection clusters request
///
/// # 4-Word Name: handle_leiden_community_detection_clusters
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns detected communities with modularity
/// - Performance: O(V * E) for Leiden clustering
/// - Error Handling: Returns error JSON if database unavailable
pub async fn handle_leiden_community_detection_clusters(
    State(state): State<SharedApplicationStateContainer>,
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
                        endpoint: "/leiden-community-detection-clusters".to_string(),
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
                    endpoint: "/leiden-community-detection-clusters".to_string(),
                    error: e,
                }),
            )
                .into_response()
        }
    };

    // Run Leiden community detection with default parameters
    let resolution = 1.0;
    let max_iterations = 10;
    let (community_map, modularity) = leiden_community_detection_clustering(&graph, resolution, max_iterations);

    // Group nodes by community ID
    let mut community_members: std::collections::HashMap<usize, Vec<String>> = std::collections::HashMap::new();
    for (node, &comm_id) in community_map.iter() {
        community_members.entry(comm_id).or_default().push(node.clone());
    }

    // Format communities
    let mut communities: Vec<CommunityClusterDataStructure> = Vec::new();
    for (id, members) in community_members.iter() {
        communities.push(CommunityClusterDataStructure {
            id: *id,
            size: members.len(),
            members: members.clone(),
        });
    }

    // Sort by size descending (largest communities first)
    communities.sort_by(|a, b| b.size.cmp(&a.size));

    // Estimate tokens
    let total_members: usize = communities.iter().map(|c| c.members.len()).sum();
    let tokens = 60 + (communities.len() * 25) + (total_members * 15);

    (
        StatusCode::OK,
        Json(LeidenCommunityResponsePayload {
            success: true,
            endpoint: "/leiden-community-detection-clusters".to_string(),
            data: LeidenCommunityDataPayload {
                community_count: communities.len(),
                modularity,
                communities,
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
