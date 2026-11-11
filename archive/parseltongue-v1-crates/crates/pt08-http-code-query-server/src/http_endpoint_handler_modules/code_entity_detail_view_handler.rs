//! Code entity detail view endpoint handler
//!
//! # 4-Word Naming: code_entity_detail_view_handler
//!
//! Endpoint: GET /code-entity-detail-view?key={entity_key}
//!
//! v1.0.4 FIX: Changed from path parameter to query parameter
//! because ISGL1 entity keys contain colons which conflict with Axum routing.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::scope_filter_utilities_module::parse_scope_build_filter_clause;

/// Query parameters for entity detail endpoint
///
/// # 4-Word Name: EntityDetailQueryParams
#[derive(Debug, Deserialize)]
pub struct EntityDetailQueryParams {
    /// Entity key (required)
    pub key: String,
    /// Filter by folder scope (e.g., "crates||parseltongue-core")
    pub scope: Option<String>,
}

/// Entity detail response data
///
/// # 4-Word Name: EntityDetailDataPayload
#[derive(Debug, Serialize)]
pub struct EntityDetailDataPayload {
    pub key: String,
    pub file_path: String,
    pub entity_type: String,
    pub entity_class: String,
    pub language: String,
    pub code: String,
}

/// Entity detail response for success case
///
/// # 4-Word Name: EntityDetailResponsePayload
#[derive(Debug, Serialize)]
pub struct EntityDetailResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: EntityDetailDataPayload,
    pub tokens: usize,
}

/// Entity detail error response
///
/// # 4-Word Name: EntityDetailErrorResponse
#[derive(Debug, Serialize)]
pub struct EntityDetailErrorResponse {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
    pub tokens: usize,
}

/// Handle code entity detail view request
///
/// # 4-Word Name: handle_code_entity_detail_view
///
/// # Contract
/// - Precondition: Database connected
/// - Postcondition: Returns entity details or 404 error
/// - Performance: <100ms
///
/// # v1.0.4 Fix
/// Changed from Path parameter to Query parameter because ISGL1 entity keys
/// contain colons (e.g., rust:fn:main:path:1-50) which conflict with Axum routing.
pub async fn handle_code_entity_detail_view(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<EntityDetailQueryParams>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Validate entity key parameter
    let entity_key = if params.key.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(EntityDetailErrorResponse {
                success: false,
                error: "Entity key parameter is required".to_string(),
                endpoint: "/code-entity-detail-view".to_string(),
                tokens: 45,
            }),
        ).into_response();
    } else {
        params.key
    };

    // Query for entity details
    if let Some(entity_details) = fetch_entity_details_from_database(&state, &entity_key, &params.scope).await {
        let tokens = 100 + entity_details.code.len();
        (
            StatusCode::OK,
            Json(EntityDetailResponsePayload {
                success: true,
                endpoint: "/code-entity-detail-view".to_string(),
                data: EntityDetailDataPayload {
                    key: entity_details.key,
                    file_path: entity_details.file_path,
                    entity_type: entity_details.entity_type,
                    entity_class: entity_details.entity_class,
                    language: entity_details.language,
                    code: entity_details.code,
                },
                tokens,
            }),
        ).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(EntityDetailErrorResponse {
                success: false,
                error: format!("Entity '{}' not found", entity_key),
                endpoint: "/code-entity-detail-view".to_string(),
                tokens: 40,
            }),
        ).into_response()
    }
}

/// Entity data from database query
///
/// # 4-Word Name: EntityDatabaseQueryResult
#[derive(Debug, Serialize)]
struct EntityDatabaseQueryResult {
    key: String,
    file_path: String,
    entity_type: String,
    entity_class: String,
    language: String,
    code: String,
}

/// Fetch entity details from database by key
///
/// # 4-Word Name: fetch_entity_details_from_database
///
/// # v1.5.4 Fix: RwLock Deadlock Prevention
/// Previously held RwLock read guard across .await boundary, causing deadlock.
/// Now clones Arc<CozoDbStorage> inside lock scope, releases lock, then awaits.
async fn fetch_entity_details_from_database(
    state: &SharedApplicationStateContainer,
    entity_key: &str,
    scope_filter: &Option<String>,
) -> Option<EntityDatabaseQueryResult> {
    // CRITICAL: Clone Arc inside scope, release lock BEFORE .await
    // Holding RwLock across .await causes deadlock (tokio rule violation)
    let storage = {
        let db_guard = state.database_storage_connection_arc.read().await;
        db_guard.as_ref().cloned()? // Clone Arc<CozoDbStorage>, lock released at scope end
    };

    // Properly escape the entity key for CozoDB query
    let escaped_entity_key = entity_key
        .replace('\\', "\\\\")
        .replace('"', "\\\"");

    // Build scope filter clause
    let scope_clause = parse_scope_build_filter_clause(scope_filter);

    // Query for entity by key - now safe, no lock held
    let query_result = storage
        .raw_query(&format!(
            "?[file_path, entity_type, entity_class, language, Current_Code] := *CodeGraph{{ISGL1_key, file_path, entity_type, entity_class, language, Current_Code, root_subfolder_L1, root_subfolder_L2}}{}, ISGL1_key == \"{}\"",
            scope_clause,
            escaped_entity_key
        ))
        .await
        .ok()?;

    // Extract first row if exists
    if let Some(row) = query_result.rows.first() {
        if row.len() >= 5 {
            return Some(EntityDatabaseQueryResult {
                key: entity_key.to_string(),
                file_path: extract_string_value(&row[0])?,
                entity_type: extract_string_value(&row[1])?,
                entity_class: extract_string_value(&row[2])?,
                language: extract_string_value(&row[3])?,
                code: extract_string_value(&row[4])?,
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