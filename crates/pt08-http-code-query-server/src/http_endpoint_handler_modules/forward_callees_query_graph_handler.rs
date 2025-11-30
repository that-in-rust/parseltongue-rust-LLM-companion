//! Forward callees query endpoint handler
//!
//! # 4-Word Naming: forward_callees_query_graph_handler
//!
//! Endpoint: GET /forward-callees-query-graph?entity={key}
//!
//! Returns all entities that the target entity calls/depends on.
//! This is the inverse of reverse_callers - shows what THIS entity uses.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Query parameters for forward callees endpoint
///
/// # 4-Word Name: ForwardCalleesQueryParams
#[derive(Debug, Deserialize)]
pub struct ForwardCalleesQueryParams {
    /// Entity key to find forward dependencies for (required)
    pub entity: String,
}

/// Callee edge data
///
/// # 4-Word Name: CalleeEdgeDataPayload
#[derive(Debug, Serialize)]
pub struct CalleeEdgeDataPayload {
    pub from_key: String,
    pub to_key: String,
    pub edge_type: String,
    pub source_location: String,
}

/// Forward callees response data
///
/// # 4-Word Name: ForwardCalleesDataPayload
#[derive(Debug, Serialize)]
pub struct ForwardCalleesDataPayload {
    pub total_count: usize,
    pub callees: Vec<CalleeEdgeDataPayload>,
}

/// Forward callees response for success case
///
/// # 4-Word Name: ForwardCalleesResponsePayload
#[derive(Debug, Serialize)]
pub struct ForwardCalleesResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: ForwardCalleesDataPayload,
    pub tokens: usize,
}

/// Forward callees error response
///
/// # 4-Word Name: ForwardCalleesErrorResponse
#[derive(Debug, Serialize)]
pub struct ForwardCalleesErrorResponse {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
    pub tokens: usize,
}

/// Handle forward callees query request
///
/// # 4-Word Name: handle_forward_callees_query_graph
///
/// # Contract
/// - Precondition: Database connected with dependency edges
/// - Postcondition: Returns list of entities that the target calls
/// - Performance: <100ms response time
/// - Error Handling: Returns 404 for entities with no dependencies
///
/// # URL Pattern
/// - Endpoint: GET /forward-callees-query-graph?entity={key}
/// - Query parameter approach avoids AXUM colon routing limitations
/// - Entity keys can contain colons without URL encoding issues
/// - Example: /forward-callees-query-graph?entity=rust:fn:main:src_main_rs:1-10
pub async fn handle_forward_callees_query_graph(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<ForwardCalleesQueryParams>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Validate entity parameter
    let entity_key = if params.entity.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ForwardCalleesErrorResponse {
                success: false,
                error: "Entity parameter is required".to_string(),
                endpoint: "/forward-callees-query-graph".to_string(),
                tokens: 35,
            }),
        ).into_response();
    } else {
        params.entity
    };

    // Query forward callees using direct CozoDB query
    let callees = query_forward_callees_direct_method(&state, &entity_key).await;
    let total_count = callees.len();

    // Estimate tokens (~50 per callee + entity key)
    let tokens = 80 + (total_count * 50) + entity_key.len();

    if callees.is_empty() {
        return (
            StatusCode::NOT_FOUND,
            Json(ForwardCalleesErrorResponse {
                success: false,
                error: format!("No callees found for entity: {}", entity_key),
                endpoint: "/forward-callees-query-graph".to_string(),
                tokens,
            }),
        ).into_response();
    }

    (
        StatusCode::OK,
        Json(ForwardCalleesResponsePayload {
            success: true,
            endpoint: "/forward-callees-query-graph".to_string(),
            data: ForwardCalleesDataPayload {
                total_count,
                callees,
            },
            tokens,
        }),
    ).into_response()
}

/// Query forward callees using direct CozoDB queries
///
/// # 4-Word Name: query_forward_callees_direct_method
///
/// Queries DependencyEdges where from_key == entity_key to find
/// all entities that this entity calls/depends on.
async fn query_forward_callees_direct_method(
    state: &SharedApplicationStateContainer,
    entity_key: &str,
) -> Vec<CalleeEdgeDataPayload> {
    let db_guard = state.database_storage_connection_arc.read().await;
    if let Some(storage) = db_guard.as_ref() {
        // Escape entity key for CozoDB query
        let escaped_entity_key = entity_key
            .replace('\\', "\\\\")
            .replace('"', "\\\"");

        // Query for edges where this entity is the source (from_key)
        let query = format!(
            r#"
            ?[from_key, to_key, edge_type, source_location] := *DependencyEdges{{from_key, to_key, edge_type, source_location}},
                from_key == "{}"
            "#,
            escaped_entity_key
        );

        match storage.raw_query(&query).await {
            Ok(result) => {
                let mut callees = Vec::new();
                for row in result.rows {
                    if row.len() >= 4 {
                        callees.push(CalleeEdgeDataPayload {
                            from_key: extract_string_value(&row[0]).unwrap_or_else(|| entity_key.to_string()),
                            to_key: extract_string_value(&row[1]).unwrap_or_else(|| "Unknown".to_string()),
                            edge_type: extract_string_value(&row[2]).unwrap_or_else(|| "Unknown".to_string()),
                            source_location: extract_string_value(&row[3]).unwrap_or_else(|| "unknown".to_string()),
                        });
                    }
                }
                callees
            }
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
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
