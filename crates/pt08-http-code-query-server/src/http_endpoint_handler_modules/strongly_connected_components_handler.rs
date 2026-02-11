//! Strongly connected components analysis endpoint handler
//!
//! # 4-Word Naming: strongly_connected_components_handler
//!
//! Endpoint: GET /strongly-connected-components-analysis
//!
//! Uses Tarjan's algorithm to find strongly connected components (SCCs)
//! and classifies their risk level based on size and coupling.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Serialize;
use std::sync::Arc;

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::scope_filter_utilities_module::parse_scope_build_filter_clause;
use axum::extract::Query;
use serde::Deserialize;
use parseltongue_core::graph_analysis::{
    AdjacencyListGraphRepresentation,
    tarjan_strongly_connected_components,
    classify_scc_risk_level,
    SccRiskLevel,
};

/// Query parameters for SCC analysis
///
/// # 4-Word Name: SccAnalysisQueryParams
#[derive(Debug, Deserialize)]
pub struct SccAnalysisQueryParams {
    /// Filter by folder scope (e.g., "crates||parseltongue-core")
    pub scope: Option<String>,
}

/// Single SCC component data
///
/// # 4-Word Name: StronglyConnectedComponentData
#[derive(Debug, Serialize)]
pub struct StronglyConnectedComponentData {
    pub id: usize,
    pub size: usize,
    pub members: Vec<String>,
    pub risk_level: String,
}

/// SCC analysis response data payload
///
/// # 4-Word Name: SccAnalysisDataPayload
#[derive(Debug, Serialize)]
pub struct SccAnalysisDataPayload {
    pub scc_count: usize,
    pub sccs: Vec<StronglyConnectedComponentData>,
}

/// SCC analysis response payload
///
/// # 4-Word Name: SccAnalysisResponsePayload
#[derive(Debug, Serialize)]
pub struct SccAnalysisResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: SccAnalysisDataPayload,
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

/// Handle strongly connected components analysis request
///
/// # 4-Word Name: handle_strongly_connected_components_analysis
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns all SCCs with risk classification
/// - Performance: O(V + E) using Tarjan's algorithm
/// - Error Handling: Returns error JSON if database unavailable
pub async fn handle_strongly_connected_components_analysis(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<SccAnalysisQueryParams>,
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
                        endpoint: "/strongly-connected-components-analysis".to_string(),
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
                    endpoint: "/strongly-connected-components-analysis".to_string(),
                    error: e,
                }),
            )
                .into_response()
        }
    };

    // Run Tarjan's algorithm
    let sccs = tarjan_strongly_connected_components(&graph);

    // Classify each SCC
    let mut scc_data: Vec<StronglyConnectedComponentData> = Vec::new();
    for (id, members) in sccs.iter().enumerate() {
        let risk_level = classify_scc_risk_level(members.len());
        let risk_str = match risk_level {
            SccRiskLevel::None => "NONE",
            SccRiskLevel::Medium => "MEDIUM",
            SccRiskLevel::High => "HIGH",
        };

        scc_data.push(StronglyConnectedComponentData {
            id,
            size: members.len(),
            members: members.clone(),
            risk_level: risk_str.to_string(),
        });
    }

    // Sort by size descending (largest SCCs first)
    scc_data.sort_by(|a, b| b.size.cmp(&a.size));

    // Estimate tokens
    let total_members: usize = scc_data.iter().map(|s| s.members.len()).sum();
    let tokens = 50 + (scc_data.len() * 30) + (total_members * 15);

    (
        StatusCode::OK,
        Json(SccAnalysisResponsePayload {
            success: true,
            endpoint: "/strongly-connected-components-analysis".to_string(),
            data: SccAnalysisDataPayload {
                scc_count: scc_data.len(),
                sccs: scc_data,
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
