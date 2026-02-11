//! Reverse callers query endpoint handler
//!
//! # 4-Word Naming: reverse_callers_query_graph_handler
//!
//! Endpoint: GET /reverse-callers-query-graph?entity={key}
//!
//! v1.0.4 FIX: Added fuzzy key matching for stdlib function calls.
//! Edge targets may use simplified keys like `rust:fn:new:unknown:0-0`
//! but we query with full entity keys like `rust:method:new:__path:38-54`.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::scope_filter_utilities_module::parse_scope_build_filter_clause;

/// Extract function name from ISGL1 key for fuzzy matching
///
/// # 4-Word Name: extract_function_name_key
///
/// # Examples
/// - "rust:fn:main:path:1-50" -> Some("main")
/// - "rust:method:new:path:38-54" -> Some("new")
/// - "invalid" -> None
fn extract_function_name_key(key: &str) -> Option<&str> {
    // ISGL1 key format: language:type:name:path:lines
    // Example: rust:fn:main:__crates_parseltongue_src_main_rs:18-40
    let parts: Vec<&str> = key.split(':').collect();
    if parts.len() >= 3 {
        Some(parts[2])
    } else {
        None
    }
}

/// Query parameters for reverse callers endpoint
///
/// # 4-Word Name: ReverseCallersQueryParams
#[derive(Debug, Deserialize)]
pub struct ReverseCallersQueryParams {
    /// Entity key to find reverse dependencies for (required)
    pub entity: String,
    /// Filter by folder scope (e.g., "crates||parseltongue-core")
    pub scope: Option<String>,
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

    // Use direct query method compatible with test database setup
    let callers = query_reverse_callers_direct_method(&state, &entity_key, &params.scope).await;
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

/// Query reverse callers using direct CozoDB queries with fuzzy matching
///
/// # 4-Word Name: query_reverse_callers_direct_method
///
/// # v1.0.4 Fix
/// Added fuzzy key matching: Edge targets may use simplified keys like
/// `rust:fn:new:unknown:0-0` but we query with full entity keys.
/// Now matches on function name when exact key match fails.
async fn query_reverse_callers_direct_method(
    state: &SharedApplicationStateContainer,
    entity_key: &str,
    scope_filter: &Option<String>,
) -> Vec<CallerEdgeDataPayload> {
    // Clone Arc, release lock, then await
    let storage = {
        let db_guard = state.database_storage_connection_arc.read().await;
        match db_guard.as_ref() {
            Some(s) => s.clone(),
            None => return Vec::new(),
        }
    }; // Lock released here

    let escaped_entity_key = entity_key
        .replace('\\', "\\\\")
        .replace('"', "\\\"");

    // Build scope filter clause
    let scope_clause = parse_scope_build_filter_clause(scope_filter);
    let scope_join = if scope_clause.is_empty() {
        String::new()
    } else {
        format!(", *CodeGraph{{ISGL1_key: from_key, root_subfolder_L1, root_subfolder_L2}}{}", scope_clause)
    };

    // v1.0.4: Build fuzzy matching query based on function name
    let query = match extract_function_name_key(entity_key) {
        Some(func_name) => {
            let escaped_func_name = func_name
                .replace('\\', "\\\\")
                .replace('"', "\\\"");
            // Fuzzy match: exact key OR keys ending with function name pattern
            format!(
                r#"
                ?[from_key, to_key, edge_type, source_location] := *DependencyEdges{{from_key, to_key, edge_type, source_location}},
                    (to_key == "{}" or
                     starts_with(to_key, "rust:fn:{}:") or
                     starts_with(to_key, "rust:method:{}:")){}
                "#,
                escaped_entity_key, escaped_func_name, escaped_func_name, scope_join
            )
        }
        None => {
            // Fallback to exact match only
            format!(
                r#"
                ?[from_key, to_key, edge_type, source_location] := *DependencyEdges{{from_key, to_key, edge_type, source_location}},
                    to_key == "{}"{}
                "#,
                escaped_entity_key, scope_join
            )
        }
    };

    match storage.raw_query(&query).await {
        Ok(result) => {
            let mut callers = Vec::new();
            for row in result.rows {
                if row.len() >= 4 {
                    callers.push(CallerEdgeDataPayload {
                        from_key: extract_string_value(&row[0]).unwrap_or_else(|| "Unknown".to_string()),
                        to_key: extract_string_value(&row[1]).unwrap_or_else(|| entity_key.to_string()),
                        edge_type: extract_string_value(&row[2]).unwrap_or_else(|| "Unknown".to_string()),
                        source_location: extract_string_value(&row[3]).unwrap_or_else(|| "unknown".to_string()),
                    });
                }
            }
            callers
        }
        Err(_) => Vec::new(),
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