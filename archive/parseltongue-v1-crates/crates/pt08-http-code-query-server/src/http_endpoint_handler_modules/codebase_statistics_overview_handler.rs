//! Codebase statistics overview endpoint handler
//!
//! # 4-Word Naming: codebase_statistics_overview_handler
//!
//! Endpoint: GET /codebase-statistics-overview-summary

use axum::{
    extract::State,
    Json,
};
use serde::Serialize;

use crate::http_server_startup_runner::SharedApplicationStateContainer;

/// Statistics overview response data
///
/// # 4-Word Name: StatisticsOverviewDataPayload
#[derive(Debug, Serialize)]
pub struct StatisticsOverviewDataPayload {
    pub code_entities_total_count: usize,
    pub test_entities_total_count: usize,
    pub dependency_edges_total_count: usize,
    pub languages_detected_list: Vec<String>,
    pub database_file_path: String,
}

/// Statistics overview response
///
/// # 4-Word Name: StatisticsOverviewResponsePayload
#[derive(Debug, Serialize)]
pub struct StatisticsOverviewResponsePayload {
    pub success: bool,
    pub endpoint: String,
    pub data: StatisticsOverviewDataPayload,
    pub tokens: usize,
}

/// Handle codebase statistics overview summary request
///
/// # 4-Word Name: handle_codebase_statistics_overview_summary
///
/// # Contract
/// - Precondition: Database loaded
/// - Postcondition: Returns entity/edge counts
/// - Performance: <50ms
pub async fn handle_codebase_statistics_overview_summary(
    State(state): State<SharedApplicationStateContainer>,
) -> Json<StatisticsOverviewResponsePayload> {
    // Update last request timestamp
    state.update_last_request_timestamp().await;

    // Query actual counts from database if connected
    let (code_count, test_count, edges_count) = state.query_entity_counts_from_database().await;

    // Read static metadata from state
    let stats = state.codebase_statistics_metadata_arc.read().await;

    let data = StatisticsOverviewDataPayload {
        code_entities_total_count: code_count,
        test_entities_total_count: test_count,
        dependency_edges_total_count: edges_count,
        languages_detected_list: stats.languages_detected_list_vec.clone(),
        database_file_path: stats.database_file_path_string.clone(),
    };

    // Estimate token count (rough: ~10 tokens per field)
    let tokens = 50;

    Json(StatisticsOverviewResponsePayload {
        success: true,
        endpoint: "/codebase-statistics-overview-summary".to_string(),
        data,
        tokens,
    })
}
