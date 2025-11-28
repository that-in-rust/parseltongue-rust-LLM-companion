//! Code entities list all endpoint handler
//!
//! # 4-Word Naming: code_entities_list_all_handler
//!
//! Endpoint: GET /code-entities-list-all

use axum::{
    extract::State,
    Json,
};
use serde::Serialize;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

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
/// - Postcondition: Returns all entities with summary info
/// - Performance: <100ms for up to 1000 entities
pub async fn handle_code_entities_list_all(
    State(state): State<SharedApplicationStateContainer>,
) -> Json<EntitiesListResponsePayload> {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Query entities from database
    let entities = query_all_entities_from_database(&state).await;
    let total_count = entities.len();

    // Estimate tokens (~20 per entity)
    let tokens = 50 + (total_count * 20);

    Json(EntitiesListResponsePayload {
        success: true,
        endpoint: "/code-entities-list-all".to_string(),
        data: EntitiesListDataPayload {
            total_count,
            entities,
        },
        tokens,
    })
}

/// Query all entities from database
///
/// # 4-Word Name: query_all_entities_from_database
async fn query_all_entities_from_database(
    state: &SharedApplicationStateContainer,
) -> Vec<EntitySummaryListItem> {
    let db_guard = state.database_storage_connection_arc.read().await;

    if let Some(storage) = db_guard.as_ref() {
        // Query all entities with summary fields
        let result = storage.raw_query(
            "?[key, file_path, entity_type, entity_class, language] := *CodeGraph{ISGL1_key: key, file_path, entity_type, entity_class, language}"
        ).await;

        match result {
            Ok(named_rows) => {
                named_rows.rows.iter().filter_map(|row| {
                    // Extract fields from row
                    let key = row.get(0).and_then(|v| match v {
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
    } else {
        Vec::new()
    }
}
