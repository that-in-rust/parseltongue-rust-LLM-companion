//! Dependency edges list endpoint handler
//!
//! # 4-Word Naming: dependency_edges_list_handler
//!
//! Endpoint: GET /dependency-edges-list-all?limit=N&offset=M
//!
//! Returns all dependency edges from the graph database with pagination.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Query parameters for edges list endpoint
///
/// # 4-Word Name: DependencyEdgesQueryParams
#[derive(Debug, Deserialize)]
pub struct DependencyEdgesQueryParams {
    /// Maximum edges to return (default: 100)
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Offset for pagination (default: 0)
    #[serde(default)]
    pub offset: usize,
}

fn default_limit() -> usize {
    100
}

/// Single edge data payload
///
/// # 4-Word Name: EdgeDataPayloadItem
#[derive(Debug, Serialize)]
pub struct EdgeDataPayloadItem {
    pub from_key: String,
    pub to_key: String,
    pub edge_type: String,
    pub source_location: String,
}

/// Edges list response data
///
/// # 4-Word Name: DependencyEdgesDataPayload
#[derive(Debug, Serialize)]
pub struct DependencyEdgesDataPayload {
    pub total_count: usize,
    pub returned_count: usize,
    pub limit: usize,
    pub offset: usize,
    pub edges: Vec<EdgeDataPayloadItem>,
}

/// Edges list response payload
///
/// # 4-Word Name: DependencyEdgesResponsePayload
#[derive(Debug, Serialize)]
pub struct DependencyEdgesResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: DependencyEdgesDataPayload,
    pub tokens: usize,
}

/// Handle dependency edges list request
///
/// # 4-Word Name: handle_dependency_edges_list_all
///
/// # Contract
/// - Precondition: Database connected with DependencyEdges table
/// - Postcondition: Returns paginated list of all edges
/// - Performance: <100ms response time for reasonable limits
/// - Error Handling: Returns empty list if no edges exist
///
/// # URL Pattern
/// - Endpoint: GET /dependency-edges-list-all?limit=N&offset=M
/// - Default limit: 100, offset: 0
pub async fn handle_dependency_edges_list_all(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<DependencyEdgesQueryParams>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Query all edges with pagination
    let (edges, total_count) = query_dependency_edges_paginated(&state, params.limit, params.offset).await;
    let returned_count = edges.len();

    // Estimate tokens (~40 per edge + overhead)
    let tokens = 60 + (returned_count * 40);

    (
        StatusCode::OK,
        Json(DependencyEdgesResponsePayload {
            success: true,
            endpoint: "/dependency-edges-list-all".to_string(),
            data: DependencyEdgesDataPayload {
                total_count,
                returned_count,
                limit: params.limit,
                offset: params.offset,
                edges,
            },
            tokens,
        }),
    ).into_response()
}

/// Query dependency edges with pagination
///
/// # 4-Word Name: query_dependency_edges_paginated
async fn query_dependency_edges_paginated(
    state: &SharedApplicationStateContainer,
    limit: usize,
    offset: usize,
) -> (Vec<EdgeDataPayloadItem>, usize) {
    let db_guard = state.database_storage_connection_arc.read().await;
    if let Some(storage) = db_guard.as_ref() {
        // First get total count
        let count_query = "?[count(from_key)] := *DependencyEdges{from_key}";
        let total_count = match storage.raw_query(count_query).await {
            Ok(result) => {
                if let Some(row) = result.rows.first() {
                    extract_count_value(&row[0]).unwrap_or(0)
                } else {
                    0
                }
            }
            Err(_) => 0,
        };

        // Query edges with limit and offset
        let query = format!(
            r#"
            ?[from_key, to_key, edge_type, source_location] :=
                *DependencyEdges{{from_key, to_key, edge_type, source_location}}
            :limit {}
            :offset {}
            "#,
            limit, offset
        );

        match storage.raw_query(&query).await {
            Ok(result) => {
                let edges: Vec<EdgeDataPayloadItem> = result.rows.iter()
                    .filter_map(|row| {
                        if row.len() >= 4 {
                            Some(EdgeDataPayloadItem {
                                from_key: extract_string_value(&row[0]).unwrap_or_default(),
                                to_key: extract_string_value(&row[1]).unwrap_or_default(),
                                edge_type: extract_string_value(&row[2]).unwrap_or_default(),
                                source_location: extract_string_value(&row[3]).unwrap_or_default(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                (edges, total_count)
            }
            Err(_) => (Vec::new(), 0),
        }
    } else {
        (Vec::new(), 0)
    }
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

/// Extract count value from CozoDB DataValue
///
/// # 4-Word Name: extract_count_value_helper
fn extract_count_value(value: &cozo::DataValue) -> Option<usize> {
    match value {
        cozo::DataValue::Num(n) => match n {
            cozo::Num::Int(i) => Some(*i as usize),
            cozo::Num::Float(f) => Some(*f as usize),
        },
        _ => None,
    }
}
