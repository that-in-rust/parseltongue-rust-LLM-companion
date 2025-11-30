//! Code entities fuzzy search endpoint handler
//!
//! # 4-Word Naming: code_entities_fuzzy_search_handler
//!
//! Endpoint: GET /code-entities-search-fuzzy?q=search-term

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Query parameters for fuzzy search endpoint
///
/// # 4-Word Name: FuzzySearchQueryParams
#[derive(Debug, Deserialize)]
pub struct FuzzySearchQueryParams {
    /// Search query string
    pub q: String,
}

/// Entity summary for search results
///
/// # 4-Word Name: SearchResultEntityItem
#[derive(Debug, Serialize)]
pub struct SearchResultEntityItem {
    pub key: String,
    pub file_path: String,
    pub entity_type: String,
    pub entity_class: String,
    pub language: String,
}

/// Fuzzy search data payload
///
/// # 4-Word Name: FuzzySearchDataPayload
#[derive(Debug, Serialize)]
pub struct FuzzySearchDataPayload {
    pub total_count: usize,
    pub entities: Vec<SearchResultEntityItem>,
}

/// Fuzzy search response payload
///
/// # 4-Word Name: FuzzySearchResponsePayload
#[derive(Debug, Serialize)]
pub struct FuzzySearchResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: FuzzySearchDataPayload,
    pub tokens: usize,
}

/// Fuzzy search error response
///
/// # 4-Word Name: FuzzySearchErrorResponse
#[derive(Debug, Serialize)]
pub struct FuzzySearchErrorResponse {
    pub success: bool,
    pub error: String,
    pub endpoint: String,
    pub tokens: usize,
}

/// Handle code entities fuzzy search request
///
/// # 4-Word Name: handle_code_entities_fuzzy_search
///
/// # Contract
/// - Precondition: Database loaded
/// - Postcondition: Returns entities matching search term
/// - Performance: <200ms for 1000 entities
pub async fn handle_code_entities_fuzzy_search(
    State(state): State<SharedApplicationStateContainer>,
    Query(params): Query<FuzzySearchQueryParams>,
) -> impl IntoResponse {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Validate search query
    if params.q.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(FuzzySearchErrorResponse {
                success: false,
                error: "Search query cannot be empty".to_string(),
                endpoint: "/code-entities-search-fuzzy".to_string(),
                tokens: 35,
            }),
        ).into_response();
    }

    // Search entities in database
    let entities = search_entities_by_query_from_database(&state, &params.q).await;
    let total_count = entities.len();

    // Estimate tokens (~20 per entity + query length)
    let tokens = 60 + (total_count * 20) + params.q.len();

    (
        StatusCode::OK,
        Json(FuzzySearchResponsePayload {
            success: true,
            endpoint: "/code-entities-search-fuzzy".to_string(),
            data: FuzzySearchDataPayload {
                total_count,
                entities,
            },
            tokens,
        }),
    ).into_response()
}

/// Search entities by query string from database
///
/// # 4-Word Name: search_entities_by_query_from_database
async fn search_entities_by_query_from_database(
    state: &SharedApplicationStateContainer,
    search_query: &str,
) -> Vec<SearchResultEntityItem> {
    let db_guard = state.database_storage_connection_arc.read().await;

    if let Some(storage) = db_guard.as_ref() {
        // Query all entities from CodeGraph - don't require Current_Code to exist
        let query = "?[key, file_path, entity_type, entity_class, language] := *CodeGraph{ISGL1_key: key, file_path, entity_type, entity_class, language}";

        let result = storage.raw_query(query).await;

        match result {
            Ok(named_rows) => {
                let search_lower = search_query.to_lowercase();
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

                    // Filter entities that contain the search term (case-insensitive)
                    if key.to_lowercase().contains(&search_lower) {
                        Some(SearchResultEntityItem {
                            key,
                            file_path,
                            entity_type,
                            entity_class,
                            language,
                        })
                    } else {
                        None
                    }
                }).collect()
            }
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    }
}