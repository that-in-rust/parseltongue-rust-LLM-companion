//! Coupling and cohesion metrics suite endpoint handler
//!
//! # 4-Word Naming: coupling_cohesion_metrics_handler
//!
//! Endpoint: GET /coupling-cohesion-metrics-suite
//!
//! Computes Chidamber & Kemerer (CK) metrics suite including CBO, LCOM,
//! RFC, and WMC to assess object-oriented design quality.

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
    compute_ck_metrics_suite,
    grade_ck_metrics_health,
    HealthGrade,
};

/// Query parameters for CK metrics
///
/// # 4-Word Name: CkMetricsQueryParameters
#[derive(Debug, Deserialize)]
pub struct CkMetricsQueryParameters {
    pub entity: Option<String>,
    /// Filter by folder scope (e.g., "crates||parseltongue-core")
    pub scope: Option<String>,
}

/// Single entity CK metrics data
///
/// # 4-Word Name: EntityCkMetricsData
#[derive(Debug, Serialize)]
pub struct EntityCkMetricsData {
    pub entity: String,
    pub cbo: usize,
    pub lcom: f64,
    pub rfc: usize,
    pub wmc: usize,
    pub health_grade: String,
}

/// CK metrics response data payload
///
/// # 4-Word Name: CkMetricsDataPayload
#[derive(Debug, Serialize)]
pub struct CkMetricsDataPayload {
    pub entities: Vec<EntityCkMetricsData>,
}

/// CK metrics response payload
///
/// # 4-Word Name: CkMetricsResponsePayload
#[derive(Debug, Serialize)]
pub struct CkMetricsResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: CkMetricsDataPayload,
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

/// Handle coupling cohesion metrics suite request
///
/// # 4-Word Name: handle_coupling_cohesion_metrics_suite
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns CK metrics for entities
/// - Performance: O(V * E) for computing metrics
/// - Error Handling: Returns error JSON if database unavailable
pub async fn handle_coupling_cohesion_metrics_suite(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<CkMetricsQueryParameters>,
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
                        endpoint: "/coupling-cohesion-metrics-suite".to_string(),
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
                    endpoint: "/coupling-cohesion-metrics-suite".to_string(),
                    error: e,
                }),
            )
                .into_response()
        }
    };

    // Compute CK metrics for all entities
    let nodes = graph.retrieve_all_graph_nodes();

    // Filter and format results
    let mut entity_data: Vec<EntityCkMetricsData> = Vec::new();

    for entity in nodes.iter() {
        // Apply entity filter if specified
        if let Some(ref entity_filter) = params.entity {
            if !entity.contains(entity_filter) {
                continue;
            }
        }

        let metrics = compute_ck_metrics_suite(&graph, entity);
        let health_grade = grade_ck_metrics_health(&metrics);

        let health_str = match health_grade {
            HealthGrade::A => "A",
            HealthGrade::B => "B",
            HealthGrade::C => "C",
            HealthGrade::D => "D",
            HealthGrade::F => "F",
        };

        entity_data.push(EntityCkMetricsData {
            entity: entity.clone(),
            cbo: metrics.cbo,
            lcom: metrics.lcom,
            rfc: metrics.rfc,
            wmc: metrics.wmc,
            health_grade: health_str.to_string(),
        });
    }

    // Sort by CBO (coupling) descending
    entity_data.sort_by(|a, b| b.cbo.cmp(&a.cbo));

    // Estimate tokens
    let tokens = 50 + (entity_data.len() * 35);

    (
        StatusCode::OK,
        Json(CkMetricsResponsePayload {
            success: true,
            endpoint: "/coupling-cohesion-metrics-suite".to_string(),
            data: CkMetricsDataPayload {
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
