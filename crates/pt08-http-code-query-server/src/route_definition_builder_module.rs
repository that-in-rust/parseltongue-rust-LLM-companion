//! Route definition builder for HTTP server
//!
//! # 4-Word Naming: route_definition_builder_module
//!
//! v1.7.3: Routes nested under /{mode}/ prefix (db or mem).
//! Health check + shutdown at root level (no prefix).

use axum::{
    Router,
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;

use crate::http_server_startup_runner::SharedApplicationStateContainer;
use crate::http_endpoint_handler_modules::{
    server_health_check_handler,
    codebase_statistics_overview_handler,
    code_entities_list_all_handler,
    code_entity_detail_view_handler,
    code_entities_fuzzy_search_handler,
    reverse_callers_query_graph_handler,
    forward_callees_query_graph_handler,
    dependency_edges_list_handler,
    blast_radius_impact_handler,
    circular_dependency_detection_handler,
    complexity_hotspots_ranking_handler,
    semantic_cluster_grouping_handler,
    api_reference_documentation_handler,
    smart_context_token_budget_handler,
    incremental_reindex_file_handler,
    file_watcher_status_handler,
    strongly_connected_components_handler,
    technical_debt_sqale_handler,
    kcore_decomposition_layering_handler,
    centrality_measures_entity_handler,
    entropy_complexity_measurement_handler,
    coupling_cohesion_metrics_handler,
    leiden_community_detection_handler,
    ingestion_coverage_folder_handler,
    ingestion_diagnostics_coverage_handler,
    folder_structure_discovery_handler,
};

/// Build API routes (nested under mode prefix)
///
/// # 4-Word Name: build_api_routes_subrouter
fn build_api_routes_subrouter() -> Router<SharedApplicationStateContainer> {
    Router::new()
        .route(
            "/codebase-statistics-overview-summary",
            get(codebase_statistics_overview_handler::handle_codebase_statistics_overview_summary)
        )
        .route(
            "/code-entities-list-all",
            get(code_entities_list_all_handler::handle_code_entities_list_all)
        )
        .route(
            "/code-entity-detail-view",
            get(code_entity_detail_view_handler::handle_code_entity_detail_view)
        )
        .route(
            "/code-entities-search-fuzzy",
            get(code_entities_fuzzy_search_handler::handle_code_entities_fuzzy_search)
        )
        .route(
            "/reverse-callers-query-graph",
            get(reverse_callers_query_graph_handler::handle_reverse_callers_query_graph)
        )
        .route(
            "/forward-callees-query-graph",
            get(forward_callees_query_graph_handler::handle_forward_callees_query_graph)
        )
        .route(
            "/dependency-edges-list-all",
            get(dependency_edges_list_handler::handle_dependency_edges_list_all)
        )
        .route(
            "/blast-radius-impact-analysis",
            get(blast_radius_impact_handler::handle_blast_radius_impact_analysis)
        )
        .route(
            "/circular-dependency-detection-scan",
            get(circular_dependency_detection_handler::handle_circular_dependency_detection_scan)
        )
        .route(
            "/complexity-hotspots-ranking-view",
            get(complexity_hotspots_ranking_handler::handle_complexity_hotspots_ranking_view)
        )
        .route(
            "/semantic-cluster-grouping-list",
            get(semantic_cluster_grouping_handler::handle_semantic_cluster_grouping_list)
        )
        .route(
            "/smart-context-token-budget",
            get(smart_context_token_budget_handler::handle_smart_context_token_budget)
        )
        .route(
            "/incremental-reindex-file-update",
            post(incremental_reindex_file_handler::handle_incremental_reindex_file_request)
        )
        .route(
            "/file-watcher-status-check",
            get(file_watcher_status_handler::handle_file_watcher_status_check)
        )
        .route(
            "/strongly-connected-components-analysis",
            get(strongly_connected_components_handler::handle_strongly_connected_components_analysis)
        )
        .route(
            "/technical-debt-sqale-scoring",
            get(technical_debt_sqale_handler::handle_technical_debt_sqale_scoring)
        )
        .route(
            "/kcore-decomposition-layering-analysis",
            get(kcore_decomposition_layering_handler::handle_kcore_decomposition_layering_analysis)
        )
        .route(
            "/centrality-measures-entity-ranking",
            get(centrality_measures_entity_handler::handle_centrality_measures_entity_ranking)
        )
        .route(
            "/entropy-complexity-measurement-scores",
            get(entropy_complexity_measurement_handler::handle_entropy_complexity_measurement_scores)
        )
        .route(
            "/coupling-cohesion-metrics-suite",
            get(coupling_cohesion_metrics_handler::handle_coupling_cohesion_metrics_suite)
        )
        .route(
            "/leiden-community-detection-clusters",
            get(leiden_community_detection_handler::handle_leiden_community_detection_clusters)
        )
        .route(
            "/ingestion-coverage-folder-report",
            get(ingestion_coverage_folder_handler::handle_ingestion_coverage_folder_report)
        )
        .route(
            "/ingestion-diagnostics-coverage-report",
            get(ingestion_diagnostics_coverage_handler::handle_ingestion_diagnostics_coverage_report)
        )
        .route(
            "/folder-structure-discovery-tree",
            get(folder_structure_discovery_handler::handle_folder_structure_discovery_tree)
        )
}

/// Handle POST /shutdown — graceful server stop
///
/// # 4-Word Name: handle_shutdown_request_endpoint
async fn handle_shutdown_request_endpoint(
    State(state): State<SharedApplicationStateContainer>,
) -> impl IntoResponse {
    state.shutdown_notify_signal_arc.notify_one();
    (StatusCode::OK, Json(json!({
        "success": true,
        "message": "Server shutting down"
    })))
}

/// Build the complete router with mode-prefixed API routes
///
/// # 4-Word Name: build_complete_router_instance
///
/// v1.7.3: All API routes nested under /{mode}/ prefix.
/// Root-level routes: health check, API docs, shutdown.
/// Wrong-prefix requests get clear error message.
pub fn build_complete_router_instance(
    state: SharedApplicationStateContainer,
    mode: &str,
) -> Router {
    let api_routes = build_api_routes_subrouter();
    let wrong_mode = if mode == "db" { "mem" } else { "db" };
    let mode_string = mode.to_string();
    let wrong_mode_string = wrong_mode.to_string();

    // Wrong-prefix fallback router returns helpful error
    let wrong_prefix_fallback = Router::new()
        .fallback(move || async move {
            (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "error": format!(
                        "This server runs in /{mode}/ mode. Use /{mode}/ prefix.",
                        mode = mode_string
                    )
                })),
            )
        });

    Router::new()
        // Root-level endpoints (no prefix)
        .route(
            "/server-health-check-status",
            get(server_health_check_handler::handle_server_health_check_status)
        )
        .route(
            "/api-reference-documentation-help",
            get(api_reference_documentation_handler::handle_api_reference_documentation_help)
        )
        .route("/shutdown", post(handle_shutdown_request_endpoint))
        // API routes nested under /{mode}/
        .nest(&format!("/{}", mode), api_routes)
        // Wrong prefix gets clear error
        .nest(&format!("/{}", wrong_mode_string), wrong_prefix_fallback)
        .with_state(state)
}
