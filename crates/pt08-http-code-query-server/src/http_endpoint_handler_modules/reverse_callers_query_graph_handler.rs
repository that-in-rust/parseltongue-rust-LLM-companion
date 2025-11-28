//! Reverse callers query endpoint handler
//!
//! # 4-Word Naming: reverse_callers_query_graph_handler
//!
//! Endpoint: GET /reverse-callers-query-graph?entity={key}

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Query parameters for reverse callers endpoint
///
/// # 4-Word Name: ReverseCallersQueryParams
#[derive(Debug, Deserialize)]
pub struct ReverseCallersQueryParams {
    /// Entity key to find reverse dependencies for (required)
    pub entity: String,
}

/// Caller edge data
///
/// # 4-Word Name: CallerEdgeDataPayload
#[derive(Debug, Serialize)]
pub struct CallerEdgeDataPayload {
    pub from_key: String,
    pub to_key: String,
    pub edge_type: String,
    pub source_location: String,
}

/// Reverse callers response data
///
/// # 4-Word Name: ReverseCallersDataPayload
#[derive(Debug, Serialize)]
pub struct ReverseCallersDataPayload {
    pub total_count: usize,
    pub callers: Vec<CallerEdgeDataPayload>,
}

/// Reverse callers response for success case
///
/// # 4-Word Name: ReverseCallersResponsePayload
#[derive(Debug, Serialize)]
pub struct ReverseCallersResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: ReverseCallersDataPayload,
    pub tokens: usize,
}

/// Reverse callers error response
///
/// # 4-Word Name: ReverseCallersErrorResponse
#[derive(Debug, Serialize)]
pub struct ReverseCallersErrorResponse {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
    pub tokens: usize,
}

/// Handle reverse callers query request
///
/// # 4-Word Name: handle_reverse_callers_query_graph
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns list of entities that call the target
/// - Performance: <100ms response time (leverages existing pt01 optimization)
/// - Error Handling: Returns 404 for non-existent entities
///
/// # URL Pattern
/// - Endpoint: GET /reverse-callers-query-graph?entity={key}
/// - Query parameter approach avoids AXUM colon routing limitations
/// - Entity keys can contain colons without URL encoding issues
/// - Example: /reverse-callers-query-graph?entity=rust:fn:process:src_process_rs:1-20
pub async fn handle_reverse_callers_query_graph(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<ReverseCallersQueryParams>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Validate entity parameter
    let entity_key = if params.entity.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ReverseCallersErrorResponse {
                success: false,
                error: "Entity parameter is required".to_string(),
                endpoint: "/reverse-callers-query-graph".to_string(),
                tokens: 35,
            }),
        ).into_response();
    } else {
        params.entity
    };

    // Use existing production-ready dependency analysis from pt01
    let callers = query_reverse_callers_using_pt01_methods(&state, &entity_key).await;
    let total_count = callers.len();

    // Estimate tokens (~50 per caller + entity key)
    let tokens = 80 + (total_count * 50) + entity_key.len();

    if callers.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(ReverseCallersErrorResponse {
                success: false,
                error: format!("No callers found for entity: {}", entity_key),
                endpoint: "/reverse-callers-query-graph".to_string(),
                tokens,
            }),
        ).into_response();
    }

    (
        StatusCode::OK,
        Json(ReverseCallersResponsePayload {
            success: true,
            endpoint: "/reverse-callers-query-graph".to_string(),
            data: ReverseCallersDataPayload {
                total_count,
                callers,
            },
            tokens,
        }),
    ).into_response()
}

/// Query reverse callers using existing pt01 production-ready methods
///
/// # 4-Word Name: query_reverse_callers_using_pt01_methods
async fn query_reverse_callers_using_pt01_methods(
    state: &SharedApplicationStateContainer,
    entity_key: &str,
) -> Vec<CallerEdgeDataPayload> {
    let db_guard = state.database_storage_connection_arc.read().await;
    if let Some(storage) = db_guard.as_ref() {
        // Use existing production-ready get_reverse_dependencies() method from pt01
        // This method is already optimized (<5ms per query) and battle-tested
        match storage.get_reverse_dependencies(entity_key).await {
            Ok(reverse_deps) => {
                // For each reverse dependency, get the full edge details
                let mut callers = Vec::new();

                for from_key in reverse_deps {
                    // Query for the edge details including edge_type and source_location
                    if let Some(edge_details) = get_edge_details_between_entities(storage, &from_key, entity_key).await {
                        callers.push(CallerEdgeDataPayload {
                            from_key,
                            to_key: entity_key.to_string(),
                            edge_type: edge_details.edge_type,
                            source_location: edge_details.source_location,
                        });
                    }
                }

                callers
            }
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    }
}

/// Edge details between two entities
///
/// # 4-Word Name: EdgeDetailsBetweenEntities
#[derive(Debug)]
struct EdgeDetailsBetweenEntities {
    edge_type: String,
    source_location: String,
}

/// Get edge details between two entities
///
/// # 4-Word Name: get_edge_details_between_entities
async fn get_edge_details_between_entities(
    storage: &parseltongue_core::storage::CozoDbStorage,
    from_key: &str,
    to_key: &str,
) -> Option<EdgeDetailsBetweenEntities> {
    // Escape keys for CozoDB query
    let escaped_from_key = from_key
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let escaped_to_key = to_key
        .replace('\\', "\\\\")
        .replace('"', "\\\"");

    let query_result = storage
        .raw_query(&format!(
            r#"
            ?[edge_type, source_location] := *DependencyEdges{{from_key, to_key, edge_type => source_location}},
                from_key == "{}",
                to_key == "{}"
            "#,
            escaped_from_key, escaped_to_key
        ))
        .await
        .ok()?;

    if let Some(row) = query_result.rows.first() {
        if row.len() >= 2 {
            return Some(EdgeDetailsBetweenEntities {
                edge_type: extract_string_value(&row[0]).unwrap_or_else(|| "Unknown".to_string()),
                source_location: extract_string_value(&row[1]).unwrap_or_else(|| "unknown".to_string()),
            });
        }
    }

    None
}

/// Extract string value from CozoDB DataValue
///
/// # 4-Word Name: extract_string_value
fn extract_string_value(value: &cozo::DataValue) -> Option<String> {
    match value {
        cozo::DataValue::Str(s) => Some(s.to_string()),
        cozo::DataValue::Null => None,
        _ => Some(format!("{:?}", value)),
    }
}