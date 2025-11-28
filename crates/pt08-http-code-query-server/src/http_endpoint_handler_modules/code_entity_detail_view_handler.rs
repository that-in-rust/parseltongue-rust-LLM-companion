//! Code entity detail view endpoint handler
//!
//! # 4-Word Naming: code_entity_detail_view_handler
//!
//! Endpoint: GET /code-entity-detail-view/{key}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::Serialize;
use urlencoding;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

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
pub async fn handle_code_entity_detail_view(
    State(state): State<SharedApplicationStateContainer>,
    Path(encoded_key): Path<String>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // For debugging - always return not found with proper JSON
    (
        StatusCode::NOT_FOUND,
        Json(EntityDetailErrorResponse {
            success: false,
            error: format!("Entity '{}' not found", encoded_key),
            endpoint: "/code-entity-detail-view".to_string(),
            tokens: 40,
        }),
    ).into_response()
}

/// Entity data from database query
///
/// # 4-Word Name: EntityDatabaseQueryResult
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
async fn fetch_entity_details_from_database(
    state: &SharedApplicationStateContainer,
    entity_key: &str,
) -> Option<EntityDatabaseQueryResult> {
    let db_guard = state.database_storage_connection_arc.read().await;
    if let Some(storage) = db_guard.as_ref() {
        // Query for entity by key
        let query_result = storage
            .raw_query(&format!(
                "?[file_path, entity_type, entity_class, language, Current_Code] := *CodeGraph{{ISGL1_key, file_path, entity_type, entity_class, language, Current_Code}}, ISGL1_key == \"{}\"",
                entity_key.replace('"', "\\\"")
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