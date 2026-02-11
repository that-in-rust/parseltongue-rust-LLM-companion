//! Code entities list all endpoint handler
//!
//! # 4-Word Naming: code_entities_list_all_handler
//!
//! Endpoint: GET /code-entities-list-all

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::scope_filter_utilities_module::parse_scope_build_filter_clause;

/// Query parameters for entities list endpoint
///
/// # 4-Word Name: EntitiesListQueryParams
#[derive(Debug, Deserialize)]
pub struct EntitiesListQueryParams {
    /// Filter entities by type (e.g., "function", "struct", "class")
    pub entity_type: Option<String>,
    /// Filter entities by folder scope (e.g., "src||core" or "src")
    pub scope: Option<String>,
}

/// Entity summary for list response
///
/// # 4-Word Name: EntitySummaryListItem
#[derive(Debug, Serialize)]
pub struct EntitySummaryListItem {
    pub key: String,
    pub file_path: String,
    pub entity_type: String,
    pub entity_class: String,
    pub language: String,
}

/// Entities list data payload
///
/// # 4-Word Name: EntitiesListDataPayload
#[derive(Debug, Serialize)]
pub struct EntitiesListDataPayload {
    pub total_count: usize,
    pub entities: Vec<EntitySummaryListItem>,
}

/// Entities list response payload
///
/// # 4-Word Name: EntitiesListResponsePayload
#[derive(Debug, Serialize)]
pub struct EntitiesListResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: EntitiesListDataPayload,
    pub tokens: usize,
}

/// Handle code entities list all request
///
/// # 4-Word Name: handle_code_entities_list_all
///
/// # Contract
/// - Precondition: Database loaded
/// - Postcondition: Returns entities with optional type filtering
/// - Performance: <100ms for up to 1000 entities
pub async fn handle_code_entities_list_all(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<EntitiesListQueryParams>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Query entities from database with optional filtering
    let entities = query_entities_with_filter_from_database(&state, params.entity_type, &params.scope).await;
    let total_count = entities.len();

    // Estimate tokens (~20 per entity)
    let tokens = 50 + (total_count * 20);

    (
        StatusCode::OK,
        Json(EntitiesListResponsePayload {
            success: true,
            endpoint: "/code-entities-list-all".to_string(),
            data: EntitiesListDataPayload {
                total_count,
                entities,
            },
            tokens,
        }),
    ).into_response()
}

/// Query entities from database with optional filtering
///
/// # 4-Word Name: query_entities_with_filter_from_database
async fn query_entities_with_filter_from_database(
    state: &SharedApplicationStateContainer,
    entity_type_filter: Option<String>,
    scope_filter: &Option<String>,
) -> Vec<EntitySummaryListItem> {
    // Clone Arc, release lock, then await
    let storage = {
        let db_guard = state.database_storage_connection_arc.read().await;
        match db_guard.as_ref() {
            Some(s) => s.clone(),
            None => return Vec::new(),
        }
    }; // Lock released here

    // Build scope filter clause
    let scope_clause = parse_scope_build_filter_clause(scope_filter);

    // Build query based on whether we have filters
    let query = match entity_type_filter {
        Some(filter) => format!(
            "?[key, file_path, entity_type, entity_class, language] := *CodeGraph{{ISGL1_key: key, file_path, entity_type, entity_class, language, root_subfolder_L1, root_subfolder_L2}}{}, entity_type == \"{}\"",
            scope_clause,
            filter.replace('"', "\\\"")
        ),
        None => format!(
            "?[key, file_path, entity_type, entity_class, language] := *CodeGraph{{ISGL1_key: key, file_path, entity_type, entity_class, language, root_subfolder_L1, root_subfolder_L2}}{}",
            scope_clause
        ),
    };

    let result = storage.raw_query(&query).await;

    match result {
            Ok(named_rows) => {
                named_rows.rows.iter().filter_map(|row| {
                    // Extract fields from row
                    let key = row.first().and_then(|v| match v {
                        cozo::DataValue::Str(s) => Some(s.to_string()),
                        _ => None,
                    })?;
                    let file_path = row.get(1).and_then(|v| match v {
                        cozo::DataValue::Str(s) => Some(s.to_string()),
                        _ => None,
                    })?;
                    let entity_type = row.get(2).and_then(|v| match v {
                        cozo::DataValue::Str(s) => Some(s.to_string()),
                        _ => None,
                    })?;
                    let entity_class = row.get(3).and_then(|v| match v {
                        cozo::DataValue::Str(s) => Some(s.to_string()),
                        _ => None,
                    })?;
                    let language = row.get(4).and_then(|v| match v {
                        cozo::DataValue::Str(s) => Some(s.to_string()),
                        _ => None,
                    })?;

                    Some(EntitySummaryListItem {
                        key,
                        file_path,
                        entity_type,
                        entity_class,
                        language,
                    })
                }).collect()
        }
        Err(_) => Vec::new(),
    }
}
