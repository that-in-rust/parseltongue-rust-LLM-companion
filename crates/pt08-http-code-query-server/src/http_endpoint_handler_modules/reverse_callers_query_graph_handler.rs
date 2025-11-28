//! Reverse callers query endpoint handler
//!
//! # 4-Word Naming: reverse_callers_query_graph_handler
//!
//! Endpoint: GET /reverse-callers-query-graph/{*entity}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Serialize;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

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
/// - Performance: <100ms response time
/// - Error Handling: Returns 404 for non-existent entities
///
/// # URL Pattern
/// - Endpoint: GET /reverse-callers-query-graph/{*entity}
/// - The {*entity} wildcard captures entity keys with colons (e.g., "rust:fn:process:src_process_rs:1-20")
/// - Entity keys are URL-encoded in requests and decoded in handler
pub async fn handle_reverse_callers_query_graph(
    State(state): State<SharedApplicationStateContainer>,
    Path(encoded_entity_key): Path<String>,
) -> impl IntoResponse {
    // Debug: Log handler entry
    println!("DEBUG: Reverse callers handler called with entity key: {}", encoded_entity_key);

    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Decode URL-encoded entity key
    let entity_key = match urlencoding::decode(&encoded_entity_key) {
        Ok(key) => key.into_owned(),
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ReverseCallersErrorResponse {
                    success: false,
                    error: format!("Invalid entity key encoding: {}", encoded_entity_key),
                    endpoint: "/reverse-callers-query-graph/{*entity}".to_string(),
                    tokens: 45,
                }),
            ).into_response();
        }
    };

    // Query for reverse callers (entities that call this entity)
    let callers = query_reverse_callers_from_database(&state, &entity_key).await;
    let total_count = callers.len();

    // Estimate tokens (~50 per caller + entity key)
    let tokens = 80 + (total_count * 50) + entity_key.len();

    if callers.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(ReverseCallersErrorResponse {
                success: false,
                error: format!("No callers found for entity: {}", entity_key),
                endpoint: "/reverse-callers-query-graph/{*entity}".to_string(),
                tokens,
            }),
        ).into_response();
    }

    (
        StatusCode::OK,
        Json(ReverseCallersResponsePayload {
            success: true,
            endpoint: "/reverse-callers-query-graph/{*entity}".to_string(),
            data: ReverseCallersDataPayload {
                total_count,
                callers,
            },
            tokens,
        }),
    ).into_response()
}

/// Query reverse callers from database by entity key
///
/// # 4-Word Name: query_reverse_callers_from_database
async fn query_reverse_callers_from_database(
    state: &SharedApplicationStateContainer,
    entity_key: &str,
) -> Vec<CallerEdgeDataPayload> {
    let db_guard = state.database_storage_connection_arc.read().await;
    if let Some(storage) = db_guard.as_ref() {
        // Query for entities that call the target entity
        // Properly escape the entity key for CozoDB query
        let escaped_entity_key = entity_key
            .replace('\\', "\\\\")
            .replace('"', "\\\"");

        let query_result = storage
            .raw_query(&format!(
                r#"
                ?[from_key, to_key, edge_type, source_location] := *DependencyEdges{{from_key, to_key, edge_type, source_location}},
                    to_key == "{}"
                "#,
                escaped_entity_key
            ))
            .await
            .ok();

        if let Some(result) = query_result {
            result
                .rows
                .into_iter()
                .filter_map(|row| {
                    if row.len() >= 4 {
                        Some(CallerEdgeDataPayload {
                            from_key: extract_string_value(&row[0])?,
                            to_key: extract_string_value(&row[1])?,
                            edge_type: extract_string_value(&row[2])?,
                            source_location: extract_string_value(&row[3]).unwrap_or_else(|| "unknown".to_string()),
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    }
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